/// Adapter del motor de descarga yt-dlp (D-10 / RF-02) — patrón REA §4:
/// proceso externo con lista de argumentos cerrada, timeout y clasificación
/// de errores del catálogo.
use std::time::Duration;

use tokio::process::Command;

use crate::models::errors::{ErrorCode, TaskError};
use crate::models::media::{parse_media_info, MediaInfo};
use crate::services::tool_locator;

/// Límite del análisis de metadatos (RF-02/EC-12).
pub const ANALYZE_TIMEOUT: Duration = Duration::from_secs(45);

/// Analiza una URL **ya normalizada** con `yt-dlp -J --no-playlist`.
pub async fn analyze(url: &str) -> Result<MediaInfo, TaskError> {
    let program = tool_locator::which("yt-dlp")
        .await
        .ok_or(TaskError::new(ErrorCode::Ec13, None))?;

    let child = Command::new(&program)
        .args(["-J", "--no-playlist", url])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| TaskError::new(ErrorCode::Ec13, Some(e.to_string())))?;

    let output = tokio::time::timeout(ANALYZE_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| TaskError::new(ErrorCode::Ec12, None))?
        .map_err(|e| TaskError::new(ErrorCode::Ec10, Some(e.to_string())))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TaskError::new(
            classify_ytdlp_error(&stderr),
            Some(stderr.lines().last().unwrap_or("").to_string()),
        ));
    }

    parse_media_info(&String::from_utf8_lossy(&output.stdout))
}

/// Clasifica la salida de error de yt-dlp según el catálogo (error-catalog §6).
/// Función pura — cubierta por tests.
pub fn classify_ytdlp_error(stderr: &str) -> ErrorCode {
    let lower = stderr.to_lowercase();

    // Orden de evaluación: primero los casos más específicos.
    // Clases con frases explícitas para evitar falsos positivos de subcadenas
    // (p. ej. "webpage" contiene "age").
    const ACCESS: [&str; 9] = [
        "login",
        "log in",
        "sign in",
        "private",
        "confirm your age",
        "age-restricted",
        "members-only",
        "copyright",
        "forbidden",
    ];
    const UNAVAILABLE: [&str; 7] = [
        "not exist",
        "does not exist",
        "removed",
        "unavailable",
        "404",
        "not found",
        "deleted",
    ];
    const NETWORK: [&str; 6] = [
        "network",
        "connection",
        "timed out",
        "timeout",
        "unreachable",
        "dns",
    ];

    if ACCESS.iter().any(|k| lower.contains(k)) {
        return ErrorCode::Ec21;
    }
    if UNAVAILABLE.iter().any(|k| lower.contains(k)) {
        return ErrorCode::Ec22;
    }
    if NETWORK.iter().any(|k| lower.contains(k)) {
        return ErrorCode::Ec20;
    }
    ErrorCode::Ec23
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_network_errors_as_ec20() {
        assert_eq!(
            classify_ytdlp_error("ERROR: Unable to download webpage: Connection refused"),
            ErrorCode::Ec20
        );
        assert_eq!(
            classify_ytdlp_error("ERROR: gaierror DNS resolution failed"),
            ErrorCode::Ec20
        );
    }

    #[test]
    fn classifies_private_or_auth_errors_as_ec21() {
        assert_eq!(
            classify_ytdlp_error("ERROR: This video is private."),
            ErrorCode::Ec21
        );
        assert_eq!(
            classify_ytdlp_error("ERROR: Sign in to confirm your age"),
            ErrorCode::Ec21
        );
    }

    #[test]
    fn classifies_unavailable_videos_as_ec22() {
        assert_eq!(
            classify_ytdlp_error("ERROR: Video has been removed by the uploader"),
            ErrorCode::Ec22
        );
        assert_eq!(
            classify_ytdlp_error("ERROR: 404 not found"),
            ErrorCode::Ec22
        );
    }

    #[test]
    fn classifies_unknown_errors_as_ec23() {
        assert_eq!(
            classify_ytdlp_error("ERROR: something unexpected"),
            ErrorCode::Ec23
        );
        assert_eq!(classify_ytdlp_error(""), ErrorCode::Ec23);
    }
}
