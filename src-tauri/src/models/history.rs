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

/// Redacta las credenciales de cada URL incrustada en un texto libre
/// (detalles de error del motor — CAT-3/RNF-04), sin alterar el resto.
/// Función pura y cubierta por tests.
pub fn redact_credentials_in_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    while i < text.len() {
        let Some(rel) = text[i..].find("://") else {
            out.push_str(&text[i..]);
            break;
        };
        let colon = i + rel;
        // Solo cuenta como esquema si hay letras ASCII pegadas al "://" (hasta 10).
        let mut scheme_start = colon;
        let mut taken = 0usize;
        while scheme_start > i
            && taken < 10
            && text.as_bytes()[scheme_start - 1].is_ascii_alphanumeric()
        {
            scheme_start -= 1;
            taken += 1;
        }
        if taken == 0 {
            // "://*" sin esquema: se copia tal cual y se avanza.
            out.push_str(&text[i..colon + 3]);
            i = colon + 3;
            continue;
        }
        out.push_str(&text[i..scheme_start]);
        // La URL termina en el primer separador (salto de línea, comilla, espacio…).
        let scan_end = (colon + 3 + 320).min(text.len());
        let url_end = text[colon + 3..scan_end]
            .find(['\n', '\r', '\t', ' ', '"', '\'', '<', '>', ']'])
            .map(|e| colon + 3 + e)
            .unwrap_or(scan_end);
        out.push_str(&redact_url(&text[scheme_start..url_end]));
        i = url_end;
    }
    out
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

    #[test]
    fn redacts_credentials_embedded_in_error_text() {
        let src = "ERROR: failed to download https://user:secreto@example.com/v?id=1 after 3 tries";
        let out = redact_credentials_in_text(src);
        assert!(
            !out.contains("secreto"),
            "la credencial no puede quedar en el detalle: {out}"
        );
        assert!(out.contains("https://example.com/v?id=1"));
        assert!(out.starts_with("ERROR: failed to download "));
        assert!(out.ends_with("after 3 tries"));
    }

    #[test]
    fn text_without_urls_is_untouched() {
        let src = "ERROR: unable to download webpage: Connection refused";
        assert_eq!(redact_credentials_in_text(src), src);
    }

    #[test]
    fn scheme_like_separator_without_scheme_is_untouched() {
        let src = "respuesta ://invalida sin esquema";
        assert_eq!(redact_credentials_in_text(src), src);
    }
}
