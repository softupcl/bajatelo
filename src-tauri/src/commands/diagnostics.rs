use crate::models::diagnostics::ToolDiagnostics;
use crate::services::tool_locator;

/// Diagnóstico real de dependencias (RF-08).
/// Ejecuta `--version` de cada herramienta con timeout y clasifica el resultado.
#[tauri::command]
pub async fn get_diagnostics() -> Vec<ToolDiagnostics> {
    let (ytdlp, ffmpeg, ffprobe) = tokio::join!(
        tool_locator::locate_and_version(crate::models::diagnostics::ToolKind::Ytdlp, "yt-dlp", &["--version"]),
        tool_locator::locate_and_version(crate::models::diagnostics::ToolKind::Ffmpeg, "ffmpeg", &["-version"]),
        tool_locator::locate_and_version(crate::models::diagnostics::ToolKind::Ffprobe, "ffprobe", &["-version"]),
    );
    vec![ytdlp, ffmpeg, ffprobe]
}
