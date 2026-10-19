use serde::Serialize;

use crate::models::errors::TaskError;
use crate::models::media::QualityOption;
use crate::models::progress::ProgressInfo;

/// Estados de tarea (AGENTS.md §8 + pausa D-05) — docs/spec/data-model.md §1.
/// Sincronizado con `src/types/contracts.ts` (TaskState).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    Queued,
    Inspecting,
    Ready,
    Downloading,
    Pausing,
    Paused,
    Converting,
    Completed,
    Failed,
    Cancelled,
}

impl TaskState {
    /// Transiciones legales — regla T1 (docs/spec/data-model.md §1).
    pub fn can_transition_to(self, next: TaskState) -> bool {
        use TaskState::*;
        matches!(
            (self, next),
            (Queued, Inspecting)
                | (Queued, Failed)
                | (Queued, Cancelled)
                | (Inspecting, Ready)
                | (Inspecting, Failed)
                | (Inspecting, Cancelled)
                | (Inspecting, Converting) // conversión local: sin selección entre probe y ejecución (C-20)
                | (Ready, Downloading)
                | (Ready, Cancelled)
                | (Downloading, Converting)
                | (Downloading, Completed)
                | (Downloading, Failed)
                | (Downloading, Cancelled)
                | (Downloading, Pausing)
                | (Pausing, Paused)
                | (Pausing, Cancelled)
                | (Paused, Downloading)
                | (Paused, Cancelled)
                | (Converting, Completed)
                | (Converting, Failed)
                | (Converting, Cancelled)
                | (Failed, Downloading)
                | (Cancelled, Downloading)
        )
    }
}

/// Tipo de tarea (D-13: el historial distingue descarga/conversión — RF-07).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskKind {
    Download,
    Convert,
}

/// Formato de salida del clon (RF-03) — sincronizado con `OutputFormat` de TS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Mp4,
    Mp3,
    Original,
}

/// Tarea de descarga o conversión (docs/spec/data-model.md §2.1/§2.3).
/// Vive en memoria; el historial y la cola persistida usan representaciones derivadas.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    pub id: String,
    pub kind: TaskKind,
    /// Número secuencial global para ordenar (la más reciente arriba, REA C4).
    pub seq: u64,
    /// URL ya normalizada y validada (V-01). Null en conversiones locales.
    pub source_url: Option<String>,
    pub portal: Option<String>,
    pub title: Option<String>,
    pub duration_sec: Option<f64>,
    /// Archivo local de entrada (solo conversiones — RF-06).
    pub input_path: Option<String>,
    pub qualities: Vec<QualityOption>,
    pub selected_quality_id: Option<String>,
    pub selected_format: Option<OutputFormat>,
    pub destination: Option<String>,
    pub state: TaskState,
    pub progress: Option<ProgressInfo>,
    pub error: Option<TaskError>,
    pub retry_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> [TaskState; 10] {
        use TaskState::*;
        [Queued, Inspecting, Ready, Downloading, Pausing, Paused, Converting, Completed, Failed, Cancelled]
    }

    /// T1: cada transición declarada en la especificación es posible.
    #[test]
    fn legal_transitions_are_allowed() {
        use TaskState::*;
        let legal = [
            (Queued, Inspecting), (Queued, Failed), (Queued, Cancelled),
            (Inspecting, Ready), (Inspecting, Failed), (Inspecting, Cancelled),
            (Inspecting, Converting), // conversión local (C-20)
            (Ready, Downloading), (Ready, Cancelled),
            (Downloading, Converting), (Downloading, Completed), (Downloading, Failed),
            (Downloading, Cancelled), (Downloading, Pausing),
            (Pausing, Paused), (Pausing, Cancelled),
            (Paused, Downloading), (Paused, Cancelled),
            (Converting, Completed), (Converting, Failed), (Converting, Cancelled),
            (Failed, Downloading), (Cancelled, Downloading),
        ];
        for (from, to) in legal {
            assert!(from.can_transition_to(to), "{from:?} → {to:?} debería ser legal");
        }
    }

    /// T1: las transiciones no declaradas están prohibidas (muestreo de casos clave).
    #[test]
    fn illegal_transitions_are_rejected() {
        use TaskState::*;
        let illegal = [
            (Completed, Downloading),      // terminal
            (Completed, Converting),
            (Queued, Downloading),         // saltarse el análisis
            (Queued, Completed),
            (Ready, Completed),            // sin descargar, no se completa
            (Ready, Failed),               // un ready "falla" solo al descargar
            (Downloading, Ready),
            (Paused, Converting),
            (Paused, Completed),
            (Failed, Completed),
            (Inspecting, Downloading),
            (Converting, Pausing),
        ];
        for (from, to) in illegal {
            assert!(!from.can_transition_to(to), "{from:?} → {to:?} debería ser ilegal");
        }
        // Ningún estado permite cambiar a sí mismo.
        for state in all() {
            assert!(!state.can_transition_to(state));
        }
    }

    #[test]
    fn serializes_lowercase() {
        assert_eq!(serde_json::to_string(&TaskState::Downloading).unwrap(), "\"downloading\"");
        assert_eq!(serde_json::to_string(&OutputFormat::Mp3).unwrap(), "\"mp3\"");
    }
}
