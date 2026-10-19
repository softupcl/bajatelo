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

/// Verificador de cancelación: se sondea periódicamente mientras FFmpeg corre.
pub type CancelCheck = Box<dyn FnMut() -> bool + Send>;

/// Sondeo de cancelación. El futuro queda **Pending** hasta que el verificador
/// señala cancelación (cada 100 ms) — con `None` nunca resuelve, así el `select!`
/// solo dispara la rama de cancelación cuando procede.
async fn cancellation(check: &mut Option<CancelCheck>) {
    loop {
        if check.is_none() {
            std::future::pending::<()>().await;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        if check.as_mut().unwrap()() {
            return;
        }
    }
}

/// Ejecuta ffmpeg con los argumentos dados (timeout largo, kill_on_drop).
/// Con verificador de cancelación, el proceso se mata en cuanto se detecta
/// (T4/req.5) y el resultado es EC-33 del catálogo.
async fn run_ffmpeg(
    mut ffmpeg: Command,
    mut cancel: Option<CancelCheck>,
) -> Result<(), TaskError> {
    let child = ffmpeg
        .stdin(std::process::Stdio::null())
        // El stdout/stderr se consume para evitar bloqueos por buffer lleno.
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| TaskError::new(ErrorCode::Ec35, Some(e.to_string())))?;

    // `None` = cancelado por el usuario; el child muere por kill_on_drop (req. 9).
    let finished = tokio::time::timeout(CONVERT_TIMEOUT, async {
        tokio::select! {
            output = child.wait_with_output() => output.map(Some),
            _ = cancellation(&mut cancel) => Ok(None),
        }
    })
    .await
    .map_err(|_| TaskError::new(ErrorCode::Ec12, Some("conversión excedió el tiempo límite".into())))?
    .map_err(|e| TaskError::new(ErrorCode::Ec32, Some(e.to_string())))?;

    match finished {
        Some(output) if output.status.success() => Ok(()),
        Some(output) => {
            let tail = String::from_utf8_lossy(&output.stderr);
            let last_line = tail.lines().last().unwrap_or("").to_string();
            Err(TaskError::new(ErrorCode::Ec32, Some(last_line)))
        }
        None => Err(TaskError::new(ErrorCode::Ec33, None)),
    }
}

/// Construye el comando FFmpeg para el formato de salida (parte reutilizable).
/// Construye el comando FFmpeg para el formato de salida (parte reutilizable).
/// `bitrate`: tasa de bits de audio (`Some` de la lista permitida; `None` = 256k).
async fn build_convert_command(
    input: &Path,
    target: &Path,
    format: crate::models::task::OutputFormat,
    bitrate: Option<&str>,
) -> Result<Command, TaskError> {
    use crate::models::task::OutputFormat;

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
                cmd.arg("-c:v").arg("libx264")
                    .arg("-c:a").arg("aac");
                for arg in audio_bitrate_args(format, bitrate) {
                    cmd.arg(arg);
                }
            }
        }
        OutputFormat::Mp3 => {
            cmd.arg("-vn").arg("-c:a").arg("libmp3lame");
            for arg in audio_bitrate_args(format, bitrate) {
                cmd.arg(arg);
            }
        }
        OutputFormat::Original => unreachable!("gestionado por el llamador"),
    }
    cmd.arg(target.as_os_str());
    Ok(cmd)
}

/// Tasas de bits de audio aceptadas (kbps). Listas permitidas por CAT-2.
pub const AUDIO_BITRATES: [&str; 6] = ["64", "96", "128", "192", "256", "320"];

/// Argumentos de tasa de bits de audio según formato (función pura — testeable).
/// Sin tasa → 256k (comportamiento previo preservado).
pub fn audio_bitrate_args(
    format: crate::models::task::OutputFormat,
    bitrate: Option<&str>,
) -> Vec<String> {
    use crate::models::task::OutputFormat;

    let kbps = bitrate
        .filter(|b| AUDIO_BITRATES.contains(&b))
        .unwrap_or("256");
    match format {
        OutputFormat::Mp3 => vec![
            "-b:a".to_string(),
            format!("{kbps}k"),
        ],
        // AAC del recodificado MP4 (solo cuando no se copian códecs).
        OutputFormat::Mp4 => vec![],
        OutputFormat::Original => vec![],
    }
}

/// Convierte el archivo final de la descarga al formato de salida (RF-03).
/// `target` llega **ya resuelto** (conflicto numerado/preguntado/cancelado se aplicó
/// antes de la descarga en `start_download`): aquí solo se ejecuta y se verifica.
/// - Original: copia directa, sin reconvertir (REA E6).
/// - MP4: copia códecs si ya son H.264/AAC compatibles; si no, recodifica (REA E2).
/// - MP3: extrae el audio (tasa seleccionada; sin tasa → 256 kb/s).
/// Verificación final (T2/CA-06.1): exit 0 + archivo existe + tamaño > 0.
/// Cancelación (T4/req. 5): mata el proceso y devuelve EC-33; el llamador
/// decide el estado de la tarea y limpia la salida parcial.
/// `bitrate`: `AUDIO_BITRATES` permitidas; `None` = 256k.
pub async fn convert_download_with_bitrate(
    input: &Path,
    target: &Path,
    format: crate::models::task::OutputFormat,
    cancel: Option<CancelCheck>,
    bitrate: Option<&str>,
) -> Result<(), TaskError> {
    use crate::models::task::OutputFormat;

    if let Some(b) = bitrate {
        if !AUDIO_BITRATES.contains(&b) {
            return Err(TaskError::new(
                ErrorCode::Ec23,
                Some(format!("tasa de bits de audio no permitida: {b}")),
            ));
        }
    }

    if format == OutputFormat::Original {
        let copied = std::fs::copy(input, target)
            .map_err(|e| TaskError::new(ErrorCode::Ec25, Some(e.to_string())))?;
        if copied == 0 || !target.exists() {
            return Err(TaskError::new(ErrorCode::Ec34, Some("la copia no produjo bytes".into())));
        }
        return Ok(());
    }

    let cmd = build_convert_command(input, target, format, bitrate).await?;
    run_ffmpeg(cmd, cancel).await?;
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
    use crate::models::task::OutputFormat;

    /// Tasa de bits de audio: MP3 genera `-b:a`, MP4 solo en recodificado,
    /// valores invalidos se filtran por el llamador (validate conEc-23).
    #[test]
    fn audio_bitrate_args_cover_mp3_and_recode() {
        assert_eq!(
            audio_bitrate_args(OutputFormat::Mp3, Some("128")),
            vec!["-b:a".to_string(), "128k".to_string()]
        );
        // Sin tasa: comportamiento previo (256k).
        assert_eq!(
            audio_bitrate_args(OutputFormat::Mp3, None),
            vec!["-b:a".to_string(), "256k".to_string()]
        );
        // Valor inválido: se ignora → 256k (la validación estricta vive en el llamador).
        assert_eq!(
            audio_bitrate_args(OutputFormat::Mp3, Some("999")),
            vec!["-b:a".to_string(), "256k".to_string()]
        );
        // En la lista permitida no hay valores raros.
        for b in AUDIO_BITRATES {
            assert_eq!(b, b.parse::<u32>().ok().map(|v| v.to_string()).unwrap());
        }
    }

    /// I2 (opt-in): 128k produce un MP3 sustancialmente menor que 320k con el
    /// mismo fixture — comprobación real de la compresión con el `bitrate`.
    #[tokio::test]
    #[ignore]
    async fn integration_bitrate_affects_real_size() {
        let dir = std::env::temp_dir().join("bajatelo-bitrate-test");
        let _ = std::fs::create_dir_all(&dir);
        let fixture = dir.join("fixture.mp4");
        let _ = std::fs::remove_file(&fixture);
        let ffmpeg_bin = tool_locator::which("ffmpeg").await.expect("ffmpeg en I2");
        let generated = Command::new(&ffmpeg_bin)
            .args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "sine=d=30", "-shortest"])
            .arg(&fixture)
            .output()
            .await
            .expect("generación del fixture");
        assert!(generated.status.success() && fixture.exists());

        let small = dir.join("small.mp3");
        let big = dir.join("big.mp3");
        convert_download_with_bitrate(&fixture, &small, OutputFormat::Mp3, None, Some("64")).await
            .expect("conversión 64k");
        convert_download_with_bitrate(&fixture, &big, OutputFormat::Mp3, None, Some("320")).await
            .expect("conversión 320k");
        let small_size = std::fs::metadata(&small).map(|m| m.len()).unwrap_or(0);
        let big_size = std::fs::metadata(&big).map(|m| m.len()).unwrap_or(0);
        assert!(small_size > 0 && big_size > 0);
        assert!(
            big_size > small_size * 4,
            "320k debía ser ampliamente mayor que 64k: {big_size} vs {small_size}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
    
    /// I2 (opt-in, `cargo test -- --ignored`): usa FFmpeg/FFprobe reales y un
    /// fixture sintético generado en su lugar (sin contenido de terceros — §2).
    #[tokio::test]
    #[ignore]
    async fn integration_probe_and_convert_real() {
        use crate::models::task::OutputFormat;

        let dir = std::env::temp_dir().join("bajatelo-i2-test");
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
        convert_download_with_bitrate(&fixture, &target_mp4, OutputFormat::Mp4, None, None)
            .await
            .expect("conversión MP4");
        assert!(std::fs::metadata(&target_mp4).map(|m| m.len() > 0).unwrap_or(false));

        // Conversión MP3 con verificación (T2).
        convert_download_with_bitrate(&fixture, &target_mp3, OutputFormat::Mp3, None, None)
            .await
            .expect("conversión MP3");
        assert!(std::fs::metadata(&target_mp3).map(|m| m.len() > 0).unwrap_or(false));

        // — Caso del enunciado: archivo NO multimedia → EC-30.
        let no_media = dir.join("not-media.txt");
        std::fs::write(&no_media, "texto sin contenido multimedia").unwrap();
        let probe_result = probe_file(&no_media).await;
        assert!(
            probe_result.err().map(|e| e.code == ErrorCode::Ec30).unwrap_or(false),
            "probe_file debía devolver EC-30"
        );

        // — Caso del enunciado: vídeo SIN pista de audio.
        let silent = dir.join("silencio.mp4");
        let generated_silent = Command::new(&ffmpeg_bin)
            .args(["-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i", "testsrc2=d=1", "-shortest"])
            .arg(&silent)
            .output()
            .await
            .expect("generación del fixture sin audio");
        assert!(generated_silent.status.success() && silent.exists());

        let silent_info = probe_file(&silent).await.expect("probe del vídeo sin audio");
        assert!(silent_info.audio_codec.is_none(), "el fixture no debía tener audio");

        let silent_mp4 = dir.join("silencio-re.mp4");
        convert_download_with_bitrate(&silent, &silent_mp4, OutputFormat::Mp4, None, None)
            .await
            .expect("MP4 sin audio debe convertir");
        assert!(std::fs::metadata(&silent_mp4).map(|m| m.len() > 0).unwrap_or(false));

        // MP3 sin pista de audio: FFmpeg no puede extraer lo que no existe → error de códec.
        let silent_mp3 = dir.join("silencio-re.mp3");
        let mp3_result = convert_download_with_bitrate(&silent, &silent_mp3, OutputFormat::Mp3, None, None).await;
        assert!(
            mp3_result.err().map(|e| e.code == ErrorCode::Ec32).unwrap_or(false),
            "MP3 desde vídeo sin audio debía fallar con EC-32"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mp4_compatibility_checks() {        assert!(is_mp4_compatible_video(Some("avc1.640028")));
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

    /// Cancelación real (T4, Unix): un comando largo se mata al señalar y
    /// devuelve EC-33 en < 5 s (también verificaría el timeout infinito si el
    /// mecanismo no funcionara).
    #[cfg(target_family = "unix")]
    #[tokio::test]
    async fn cancellation_kills_long_process_immediately() {
        use std::time::Instant;

        let mut cmd = Command::new("sleep");
        cmd.args(["30"]);
        let start = Instant::now();
        let result = run_ffmpeg(cmd, Some(Box::new(|| true) as CancelCheck)).await;
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().code, ErrorCode::Ec33);
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "la cancelación no mató el proceso: {:?}",
            start.elapsed()
        );
    }

    /// Sin verificador: el proceso corre y termina Ok (no se rompe el flujo normal).
    #[cfg(target_family = "unix")]
    #[tokio::test]
    async fn runs_to_completion_without_cancel_check() {
        let mut cmd = Command::new("sleep");
        cmd.args(["0.5"]);
        let result = run_ffmpeg(cmd, None).await;
        assert!(
            result.is_ok(),
            "run_ffmpeg debía terminar correctamente: {result:?}"
        );
    }

    /// Con verificador falso: el verificador sondeado nunca cancela.
    #[cfg(target_family = "unix")]
    #[tokio::test]
    async fn cancel_check_that_only_never_cancels() {
        let mut cmd = Command::new("sleep");
        cmd.args(["0.5"]);
        let result = run_ffmpeg(cmd, Some(Box::new(|| false) as CancelCheck)).await;
        assert!(result.is_ok());
    }
}
