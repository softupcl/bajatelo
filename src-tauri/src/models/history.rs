use serde::{Deserialize, Serialize};

/// Entrada de historial (RF-07) — docs/spec/data-model.md §2.4.
/// Sincronizado con `src/types/contracts.ts` (HistoryEntry).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    /// "download" | "convert"
    pub kind: String,
    pub title: String,
    pub portal: Option<String>,
    /// Redactada (sin credenciales) — CA-07.3.
    pub source_url: Option<String>,
    pub format: Option<String>,
    pub quality: Option<String>,
    pub destination_path: String,
    pub final_size_bytes: Option<i64>,
    /// "completed" | "failed" | "cancelled"
    pub status: String,
    pub error_code: Option<String>,
    pub started_at: String,
    pub finished_at: String,
}

/// Redacta credenciales de una URL (`user:pass@host` → `host`) — CA-07.3.
/// Función pura, cubierta por tests. (Se usa en D-12 al registrar tareas.)
#[allow(dead_code)]
pub fn redact_url(url: &str) -> String {
    match url::Url::parse(url) {
        Ok(mut parsed) => {
            if !parsed.username().is_empty() || parsed.password().is_some() {
                let _ = parsed.set_username("");
                let _ = parsed.set_password(None);
            }
            parsed.to_string()
        }
        Err(_) => "[url inválida]".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_user_and_password() {
        assert_eq!(
            redact_url("https://user:secreto@example.com/video?x=1"),
            "https://example.com/video?x=1"
        );
    }

    #[test]
    fn keeps_urls_without_credentials() {
        assert_eq!(
            redact_url("https://www.youtube.com/watch?v=abc"),
            "https://www.youtube.com/watch?v=abc"
        );
    }

    #[test]
    fn invalid_url_never_panics_nor_leaks() {
        let out = redact_url("no es una url");
        assert_eq!(out, "[url inválida]");
    }
}
