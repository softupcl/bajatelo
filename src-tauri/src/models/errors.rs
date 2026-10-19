use serde::Serialize;

/// Códigos del catálogo de errores — docs/spec/error-catalog.md.
/// Sincronizado con `src/types/contracts.ts` (ErrorCode).
/// Contrato definido para los bloques 2–4 del backlog (aún sin llamadas en la app);
/// verificado por tests unitarios, por lo que el aviso de dead_code está permitido.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ErrorCode {
    #[serde(rename = "EC-01")]
    Ec01,
    #[serde(rename = "EC-02")]
    Ec02,
    #[serde(rename = "EC-03")]
    Ec03,
    #[serde(rename = "EC-10")]
    Ec10,
    #[serde(rename = "EC-11")]
    Ec11,
    #[serde(rename = "EC-12")]
    Ec12,
    #[serde(rename = "EC-13")]
    Ec13,
    #[serde(rename = "EC-20")]
    Ec20,
    #[serde(rename = "EC-21")]
    Ec21,
    #[serde(rename = "EC-22")]
    Ec22,
    #[serde(rename = "EC-23")]
    Ec23,
    #[serde(rename = "EC-24")]
    Ec24,
    #[serde(rename = "EC-25")]
    Ec25,
    #[serde(rename = "EC-26")]
    Ec26,
    #[serde(rename = "EC-30")]
    Ec30,
    #[serde(rename = "EC-31")]
    Ec31,
    #[serde(rename = "EC-32")]
    Ec32,
    #[serde(rename = "EC-33")]
    Ec33,
    #[serde(rename = "EC-34")]
    Ec34,
    #[serde(rename = "EC-35")]
    Ec35,
    #[serde(rename = "EC-40")]
    Ec40,
    #[serde(rename = "EC-41")]
    Ec41,
    #[serde(rename = "EC-42")]
    Ec42,
    #[serde(rename = "EC-43")]
    Ec43,
}

/// Error de tarea: código del catálogo + detalle técnico opcional
/// (truncado y sin secretos — CAT-3).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskError {
    pub code: ErrorCode,
    pub detail: Option<String>,
}

impl TaskError {
    #[allow(dead_code)]
    pub fn new(code: ErrorCode, detail: Option<String>) -> Self {
        Self {
            code,
            // CAT-3/RNF-04: redacta credenciales incrustadas antes de truncar.
            detail: detail
                .map(|d| crate::models::history::redact_credentials_in_text(&d))
                .map(|d| d.chars().take(500).collect()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_error_serializes_with_catalog_code() {
        let err = TaskError::new(ErrorCode::Ec20, None);
        let json = serde_json::to_string(&err).expect("serializable");
        assert!(json.contains("\"code\":\"EC-20\""), "json: {json}");
    }

    #[test]
    fn task_error_truncates_technical_detail_to_500_chars() {
        let long_detail = "x".repeat(2000);
        let err = TaskError::new(ErrorCode::Ec23, Some(long_detail));
        let detail = err.detail.expect("detail present");
        assert_eq!(detail.len(), 500);
    }

    /// RNF-04/CAT-3: ningún detalle de error puede llevar credenciales en URLs.
    #[test]
    fn task_error_never_carries_url_credentials() {
        let err = TaskError::new(
            ErrorCode::Ec23,
            Some("ERROR: https://api:user-key-12345@internal.example.org/resource falló".into()),
        );
        let detail = err.detail.expect("detail present");
        assert!(
            !detail.contains("user-key-12345"),
            "detalles con credenciales: {detail}"
        );
        assert!(detail.contains("https://internal.example.org/resource"));
    }

    #[test]
    fn task_error_without_detail_has_none() {
        let err = TaskError::new(ErrorCode::Ec01, None);
        assert!(err.detail.is_none());
    }
}
