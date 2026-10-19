use crate::app::validation;
use crate::models::errors::TaskError;
use crate::models::media::MediaInfo;
use crate::services::ytdlp;

/// Análisis real de una URL (RF-01/RF-02): valida y normaliza (V-01),
/// luego consulta al motor con argumentos cerrados (RNF-01).
#[tauri::command]
pub async fn analyze_url(url: String) -> Result<MediaInfo, TaskError> {
    let normalized = validation::validate_and_normalize_url(&url)?;
    ytdlp::analyze(&normalized).await
}
