/// Adapter de FFmpeg/FFprobe (RF-06 y fase `converting` de D-12).
/// Mejora respecto a la referencia (REA E10): usa FFprobe en lugar de parsear
/// la salida de un `ffmpeg` sin objetivo para decidir copiar vs recodificar.
/// Toda ejecución con lista de argumentos (RNF-01).
use std::path::Path;

use tokio::process::Command;

use crate::models::errors::{ErrorCode, TaskError};
use crate::services::tool_locator;

use std::time::Duration;

/// Timeout de la conversión (RF-06): los archivos son locales; 24h de margen amplio.
pub const CONVERT_TIMEOUT: Duration = Duration::from_secs(24 * 3600);

async fn probe_codec(input: &Path, stream: &str) -> Result<Option<String>, TaskError> {
    let ffprobe = tool_locator::which("ffprobe")
        .await
        .ok_or(TaskError::new(ErrorCode::Ec35, None))?;
    let output = Command::new(&ffprobe)
        .args([
            "-v", "error",
            "-select_streams", stream,
            "-show_entries", "stream=codec_name",
            "-of", "csv=p=0",
        ])
        .arg(input.as_os_str())
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .map_err(|e| TaskError::new(ErrorCode::Ec35, Some(e.to_string())))?;
    if !output.status.success() {
        return Ok(None);
    }
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(if raw.is_empty() { None } else { Some(raw) })
}

/// Inspección completa de un archivo local con FFprobe (RF-06/C-20).
/// EC-30 si el archivo no es reconocible como multimedia; EC-35 si ffprobe falta.
pub async fn probe_file(input: &Path) -> Result<crate::models::media::LocalMediaInfo, TaskError> {
    let ffprobe = tool_locator::which("ffprobe")
        .await
        .ok_or(TaskError::new(ErrorCode::Ec35, None))?;
    let output = Command::new(&ffprobe)
        .args(["-v", "error", "-show_entries", "format=format_name,duration", "-of", "default=nw=1"])
        .arg(input.as_os_str())
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .map_err(|e| TaskError::new(ErrorCode::Ec30, Some(e.to_string())))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.lines().last().unwrap_or("").to_string();
        return Err(TaskError::new(ErrorCode::Ec30, Some(detail)));
    }
    let (container, duration) = crate::models::media::parse_probe_format_output(
        &String::from_utf8_lossy(&output.stdout),
    );
    if container.is_none() {
        return Err(TaskError::new(ErrorCode::Ec30, Some("formato no reconocido".into())));
    }
    let (video, audio) = probe_codecs(input).await?;
    let size_bytes = std::fs::metadata(input).map(|m| m.len() as i64).ok();
    Ok(crate::models::media::LocalMediaInfo {
        container,
        video_codec: video,
        audio_codec: audio,
        duration_sec: duration,
        size_bytes,
    })
}

/// (códec vídeo, códec audio) del primer stream de cada tipo.
pub async fn probe_codecs(
    input: &Path,
) -> Result<(Option<String>, Option<String>), TaskError> {
    let (v, a) = tokio::join!(probe_codec(input, "v:0"), probe_codec(input, "a:0"));
    Ok((v?, a?))
}

fn is_mp4_compatible_video(codec: Option<&str>) -> bool {
    codec.is_some_and(|c| {
        ["avc1", "h264", "mpeg4"].iter().any(|ok| c.starts_with(ok))
    })
}

fn is_mp4_compatible_audio(codec: Option<&str>) -> bool {
    codec.is_some_and(|c| c.starts_with("aac") || c.starts_with("mp3") || c.starts_with("mp4a"))
}

/// Ejecuta ffmpeg con los argumentos dados (timeout largo, kill_on_drop).
async fn run_ffmpeg(mut ffmpeg: Command) -> Result<(), TaskError> {
    let child = ffmpeg
        .stdin(std::process::Stdio::null())
        // El stdout/stderr se consume para evitar bloqueos por buffer lleno.
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| TaskError::new(ErrorCode::Ec35, Some(e.to_string())))?;
    let output = tokio::time::timeout(CONVERT_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| TaskError::new(ErrorCode::Ec12, Some("conversión excedió el tiempo límite".into())))?
        .map_err(|e| TaskError::new(ErrorCode::Ec32, Some(e.to_string())))?;
    if !output.status.success() {
        let tail = String::from_utf8_lossy(&output.stderr);
        let last_line = tail.lines().last().unwrap_or("").to_string();
        return Err(TaskError::new(ErrorCode::Ec32, Some(last_line)));
    }
    Ok(())
}

/// Convierte el archivo final de la descarga al formato de salida (RF-03).
/// `target` llega **ya resuelto** (conflicto numerado/preguntado/cancelado se aplicó
/// antes de la descarga en `start_download`): aquí solo se ejecuta y se verifica.
/// - Original: copia directa, sin reconvertir (REA E6).
/// - MP4: copia códecs si ya son H.264/AAC compatibles; si no, recodifica (REA E2).
/// - MP3: extrae el audio a 256 kb/s.
/// Verificación final (T2/CA-06.1): exit 0 + archivo existe + tamaño > 0.
pub async fn convert_download(
    input: &Path,
    target: &Path,
    format: crate::models::task::OutputFormat,
) -> Result<(), TaskError> {
    use crate::models::task::OutputFormat;

    if format == OutputFormat::Original {
        let copied = std::fs::copy(input, target)
            .map_err(|e| TaskError::new(ErrorCode::Ec25, Some(e.to_string())))?;
        if copied == 0 || !target.exists() {
            return Err(TaskError::new(ErrorCode::Ec34, Some("la copia no produjo bytes".into())));
        }
        return Ok(());
    }

    let bin = tool_locator::which("ffmpeg")
        .await
        .ok_or(TaskError::new(ErrorCode::Ec35, None))?;
    let mut cmd = Command::new(&bin);
    cmd.arg("-y").arg("-i").arg(input.as_os_str());
    match format {
        OutputFormat::Mp4 => {
            let (video, audio) = probe_codecs(input).await?;
            if is_mp4_compatible_video(video.as_deref())
                && is_mp4_compatible_audio(audio.as_deref())
            {
                // REA E2: copiar códecs compatibles evita reconvertir.
                cmd.arg("-c:v").arg("copy").arg("-c:a").arg("copy");
            } else {
                cmd.arg("-c:v").arg("libx264").arg("-c:a").arg("aac").arg("-b:a").arg("256k");
            }
        }
        OutputFormat::Mp3 => {
            cmd.arg("-vn").arg("-c:a").arg("libmp3lame").arg("-b:a").arg("256k");
        }
        OutputFormat::Original => unreachable!("gestionado arriba"),
    }
    cmd.arg(target.as_os_str());

    run_ffmpeg(cmd).await?;
    verify_output(target)
}

/// Verificación de salida (T2): existe + tamaño > 0 (sirve para copia y conversión).
pub fn verify_output(target: &Path) -> Result<(), TaskError> {
    let ok = std::fs::metadata(target)
        .map(|m| m.len() > 0)
        .unwrap_or(false);
    if !ok {
        return Err(TaskError::new(
            ErrorCode::Ec34,
            Some(format!("salida vacía o inexistente: {}", target.display())),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// I2 (opt-in, `cargo test -- --ignored`): usa FFmpeg/FFprobe reales y un
    /// fixture sintético generado en su lugar (sin contenido de terceros — §2).
    #[tokio::test]
    #[ignore]
    async fn integration_probe_and_convert_real() {
        use crate::models::task::OutputFormat;

        let dir = std::env::temp_dir().join("rea-i2-test");
        let _ = std::fs::create_dir_all(&dir);
        let fixture = dir.join("fixture.mp4");
        let target_mp4 = dir.join("target.mp4");
        let target_mp3 = dir.join("target.mp3");
        let _ = std::fs::remove_file(&fixture);
        let _ = std::fs::remove_file(&target_mp4);
        let _ = std::fs::remove_file(&target_mp3);

        let ffmpeg_bin = tool_locator::which("ffmpeg").await.expect("ffmpeg en I2");
        let generated = Command::new(&ffmpeg_bin)
            .args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "testsrc2=d=1", "-f", "lavfi", "-i", "sine=d=1", "-shortest"])
            .arg(&fixture)
            .output()
            .await
            .expect("generación del fixture");
        assert!(
            generated.status.success() && fixture.exists(),
            "no se generó el fixture"
        );

        // Inspección real (EC-30/EC-11 no deben ocurrir).
        let info = probe_file(&fixture).await.expect("probe_file");
        assert!(info.container.is_some());
        assert!(info.duration_sec.is_some());

        // Conversión MP4 con verificación (T2).
        convert_download(&fixture, &target_mp4, OutputFormat::Mp4)
            .await
            .expect("conversión MP4");
        assert!(std::fs::metadata(&target_mp4).map(|m| m.len() > 0).unwrap_or(false));

        // Conversión MP3 con verificación (T2).
        convert_download(&fixture, &target_mp3, OutputFormat::Mp3)
            .await
            .expect("conversión MP3");
        assert!(std::fs::metadata(&target_mp3).map(|m| m.len() > 0).unwrap_or(false));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mp4_compatibility_checks() {
        assert!(is_mp4_compatible_video(Some("avc1.640028")));
        assert!(is_mp4_compatible_video(Some("h264")));
        assert!(is_mp4_compatible_video(Some("mpeg4")));
        assert!(!is_mp4_compatible_video(Some("vp9")));
        assert!(!is_mp4_compatible_video(None));
        assert!(is_mp4_compatible_audio(Some("aac")));
        assert!(is_mp4_compatible_audio(Some("mp4a.40.2")));
        assert!(is_mp4_compatible_audio(Some("mp3")));
        assert!(!is_mp4_compatible_audio(Some("opus")));
        assert!(!is_mp4_compatible_audio(None));
    }
}
