/// Actualizador del motor de descarga (D-09/C): yt-dlp se baja del release
/// oficial de GitHub, verificando integridad, y se instala en el directorio
/// de datos de la app (con prioridad sobre el PATH en `tool_locator`).
/// Toda la descarga con listas de argumentos (RNF-01) y sin shell (AGENTS §7).
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::models::errors::{ErrorCode, TaskError};

/// URL del release más reciente del proyecto oficial yt-dlp.
pub const YTDLP_LATEST_API: &str = "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest";

/// Outcome estado fina del motor de descarga actualizado (nombre corto para el UI).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineUpdateOutcome {
    pub version: String,
    pub path: String,
}

/// Nombre del binario según plataforma (Windows exige .exe).
pub fn binary_name() -> &'static str {
    if cfg!(target_family = "windows") {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    }
}

/// Asset oficial según plataforma/arquitectura destino.
pub fn asset_for(os: &str, arch: &str) -> Result<&'static str, TaskError> {
    match (os, arch) {
        ("macos", "aarch64") | ("macos", "x86_64") => Ok("yt-dlp_macos"),
        ("windows", "x86_64") => Ok("yt-dlp.exe"),
        ("linux", "x86_64") => Ok("yt-dlp_linux"),
        ("linux", "aarch64") => Ok("yt-dlp_linux_aarch64"),
        _ => Err(TaskError::new(
            ErrorCode::Ec23,
            Some(format!("plataforma sin motor fijo: {os}/{arch}")),
        )),
    }
}

/// Extrae `tag_name` del JSON del release (función pura — testeada).
pub fn parse_latest_tag(release_json: &str) -> Option<String> {
    let marker = "\"tag_name\":";
    let at = release_json.find(marker)?;
    let rest = &release_json[at + marker.len()..];
    let start = rest.find('"')?;
    let end = rest[start + 1..].find('"')?;
    Some(rest[start + 1..start + 1 + end].to_string())
}

/// Extrae el hash de una línea del SHA2-256SUMS para el asset dado (pura).
pub fn extract_hash_from_sums(sums: &str, asset: &str) -> Option<String> {
    let target_line = sums.lines().find(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next().unwrap_or("");
        let file = parts.next().unwrap_or("");
        hash.len() == 64 && file == asset
    })?;
    Some(target_line.split_whitespace().next()?.to_string())
}

/// SHA-256 hexadecimal de un archivo (función pura — testeada con fixture).
pub fn sha256_file(path: &Path) -> Result<String, TaskError> {
    let bytes = std::fs::read(path)
        .map_err(|e| TaskError::new(ErrorCode::Ec20, Some(format!("lectura del motor: {e}"))))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// Descarga con `curl` del SO (proceso con argumentos separados — RNF-01).
/// Con `out_file` escribe a fichero; devuelve la salida.
async fn curl(url: &str, out_file: Option<&Path>, ua: &str) -> Result<String, TaskError> {
    let curl = crate::services::tool_locator::which("curl")
        .await
        .ok_or_else(|| TaskError::new(ErrorCode::Ec20, Some("curl no disponible".into())))?;
    let mut cmd = tokio::process::Command::new(&curl);
    cmd.args(["-sS", "-L", "--fail", "--retry", "2", "-A", ua]);
    if let Some(out) = out_file {
        cmd.arg("-o").arg(out.as_os_str());
    }
    cmd.arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let child = cmd
        .spawn()
        .map_err(|e| TaskError::new(ErrorCode::Ec20, Some(format!("curl spawn: {e}"))))?;

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| TaskError::new(ErrorCode::Ec20, Some(format!("curl: {e}"))))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TaskError::new(
            ErrorCode::Ec20,
            Some(stderr.lines().last().unwrap_or("").to_string()),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Verifica que la URL del asset sea la oficial del release (patrón conocido)
/// y devuelve la URL de descarga segura.
fn asset_download_url(release_json: &str, asset: &str, tag: &str) -> Option<String> {
    // El patrón oficial: https://github.com/yt-dlp/yt-dlp/releases/download/<tag>/<asset>
    // Verificamos que el JSON lo contenga (evita suposiciones).
    let needle = format!("\"browser_download_url\":\"https://github.com/yt-dlp/yt-dlp/releases/download/{tag}/{asset}\"");
    release_json
        .find(&needle)
        .map(|_| format!("https://github.com/yt-dlp/yt-dlp/releases/download/{tag}/{asset}"))
}

/// Actualiza yt-dlp en el directorio dado (normalmente `<datos>/engines`).
pub async fn update_into(dir: &Path) -> Result<EngineUpdateOutcome, TaskError> {
    let ua = "bajatelo/0.1.0 (releases oficiales de yt-dlp)";
    let release_json = curl(YTDLP_LATEST_API, None, ua).await?;
    let tag = parse_latest_tag(&release_json)
        .ok_or_else(|| TaskError::new(ErrorCode::Ec23, Some("release sin tag_name".into())))?;
    let asset = asset_for(std::env::consts::OS, std::env::consts::ARCH)?;
    let url = asset_download_url(&release_json, asset, &tag)
        .ok_or_else(|| TaskError::new(ErrorCode::Ec23, Some(format!("asset {asset} no está en el release {tag}"))))?;

    std::fs::create_dir_all(dir)
        .map_err(|e| TaskError::new(ErrorCode::Ec25, Some(e.to_string())))?;
    let tmp = dir.join(".yt-dlp.tmp");
    curl(&url, Some(&tmp), ua).await?;

    let sums = curl(
        &format!("https://github.com/yt-dlp/yt-dlp/releases/download/{tag}/SHA2-256SUMS"),
        None,
        ua,
    )
    .await?;
    let expected = extract_hash_from_sums(&sums, asset)
        .ok_or_else(|| TaskError::new(ErrorCode::Ec23, Some("hash oficial no encontrado".into())))?;
    let real = sha256_file(&tmp)?;
    if real != expected {
        let _ = std::fs::remove_file(&tmp);
        return Err(TaskError::new(
            ErrorCode::Ec23,
            Some(format!("integridad fallida (SHA-256): {expected} != {real}")),
        ));
    }

    let final_name = binary_name();
    let final_path = dir.join(final_name);
    let _ = std::fs::remove_file(&final_path);
    std::fs::rename(&tmp, &final_path)
        .map_err(|e| TaskError::new(ErrorCode::Ec25, Some(e.to_string())))?;
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&final_path, std::fs::Permissions::from_mode(0o755));
    }
    Ok(EngineUpdateOutcome {
        version: tag,
        path: final_path.display().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_map_covers_the_released_platforms() {
        assert_eq!(asset_for("macos", "aarch64").unwrap(), "yt-dlp_macos");
        assert_eq!(asset_for("windows", "x86_64").unwrap(), "yt-dlp.exe");
        assert_eq!(asset_for("linux", "x86_64").unwrap(), "yt-dlp_linux");
        assert_eq!(asset_for("linux", "aarch64").unwrap(), "yt-dlp_linux_aarch64");
        assert!(asset_for("freebsd", "x86_64").is_err());
    }

    #[test]
    fn parses_latest_tag_from_release_json() {
        let json = r#"{"url":"…","tag_name":"2026.08.19","name":"yt-dlp 2026.08.19"}"#;
        assert_eq!(parse_latest_tag(json).as_deref(), Some("2026.08.19"));
        assert_eq!(parse_latest_tag("{}"), None);
    }

    #[test]
    fn extracts_hash_from_official_sums() {
        let sums = "66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a  yt-dlp.exe\n\
                    0f192b7ec147ab6288885d6351d9ab67367640029b4377576ef46dd79cf7b202  yt-dlp_macos\n";
        assert_eq!(
            extract_hash_from_sums(sums, "yt-dlp.exe").as_deref(),
            Some("66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a")
        );
        assert_eq!(extract_hash_from_sums(sums, "yt-dlp_linux"), None);
    }

    #[test]
    fn sha256_matches_known_vector() {
        let dir = std::env::temp_dir().join("bajatelo-sha-test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("x.bin");
        std::fs::write(&file, b"hola").unwrap();
        // sha256("hola") — vector conocido (verificado con shasum).
        assert_eq!(
            sha256_file(&file).unwrap(),
            "b221d9dbb083a7f33428d7c2a3c3198ae925614d70210e28716ccaa7cd4ddb79"
        );
    }
}

