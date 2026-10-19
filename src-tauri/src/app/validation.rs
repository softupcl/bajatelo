/// Validación y normalización de URLs (V-01 / RF-01) — AGENTS.md §7:
/// solo http(s), sin ejecución de esquemas peligrosos, normalización
/// de YouTube siguiendo el patrón observado en la referencia (REA A3).
use crate::models::errors::{ErrorCode, TaskError};

pub fn validate_and_normalize_url(input: &str) -> Result<String, TaskError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(TaskError::new(ErrorCode::Ec01, None));
    }

    let parsed = url::Url::parse(trimmed).map_err(|_| TaskError::new(ErrorCode::Ec02, None))?;

    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err(TaskError::new(ErrorCode::Ec02, None)),
    }

    let host = parsed
        .host_str()
        .filter(|h| !h.is_empty())
        .ok_or_else(|| TaskError::new(ErrorCode::Ec03, None))?;

    // El host debe parecer un hostname (punto/localhost/corchetes IPv6),
    // evita cadenas tipo "javascript:" tras el esquema.
    let looks_like_host =
        host.contains('.') || host == "localhost" || host.starts_with('[');
    if !looks_like_host {
        return Err(TaskError::new(ErrorCode::Ec03, None));
    }

    Ok(normalize_youtube(parsed))
}

/// Normalización de YouTube (REA A3, verificado en video.cpp:65-74):
/// - `www.youtube.com/watch?…` → solo se conserva `v=`
/// - `youtu.be/<id>` → `https://www.youtube.com/watch?v=<id>`
fn normalize_youtube(url: url::Url) -> String {
    let host = url.host_str().unwrap_or("").to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host).to_string();

    if host == "youtube.com" && url.path() == "/watch" {
        let v = url
            .query_pairs()
            .find(|(k, _)| k == "v")
            .map(|(_, v)| v.into_owned());
        if let Some(v) = v {
            if !v.is_empty() {
                return format!("https://www.youtube.com/watch?v={v}");
            }
        }
        return url.to_string();
    }

    if host == "youtu.be" {
        let id = url.path().trim_start_matches('/');
        if !id.is_empty() {
            return format!("https://www.youtube.com/watch?v={id}");
        }
        return url.to_string();
    }

    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_code(input: &str) -> ErrorCode {
        validate_and_normalize_url(input).unwrap_err().code
    }

    #[test]
    fn accepts_and_passes_through_https() {
        assert_eq!(
            validate_and_normalize_url("https://example.com/video").unwrap(),
            "https://example.com/video"
        );
        assert_eq!(
            validate_and_normalize_url("  http://example.com/v  ").unwrap(),
            "http://example.com/v"
        );
    }

    #[test]
    fn rejects_empty_and_blank() {
        assert_eq!(err_code(""), ErrorCode::Ec01);
        assert_eq!(err_code("    "), ErrorCode::Ec01);
    }

    #[test]
    fn rejects_unsupported_schemes() {
        assert_eq!(err_code("file:///etc/passwd"), ErrorCode::Ec02);
        assert_eq!(err_code("javascript:alert(1)"), ErrorCode::Ec02);
        assert_eq!(err_code("ftp://example.com/x"), ErrorCode::Ec02);
        assert_eq!(err_code("data:text/plain,hola"), ErrorCode::Ec02);
    }

    #[test]
    fn rejects_not_a_url_and_bad_host() {
        assert_eq!(err_code("no es una url"), ErrorCode::Ec02);
        assert_eq!(err_code("http://"), ErrorCode::Ec02);
        // Host sin punto ni localhost reconocible.
        assert_eq!(err_code("http://foo"), ErrorCode::Ec03);
    }

    #[test]
    fn accepts_localhost() {
        assert!(validate_and_normalize_url("http://localhost:1420/").is_ok());
    }

    #[test]
    fn normalizes_youtube_watch_to_only_v_param() {
        assert_eq!(
            validate_and_normalize_url(
                "https://www.youtube.com/watch?v=abc123&t=30s&list=PLxyz"
            )
            .unwrap(),
            "https://www.youtube.com/watch?v=abc123"
        );
    }

    #[test]
    fn normalizes_youtu_be_short_links() {
        assert_eq!(
            validate_and_normalize_url("https://youtu.be/abc123?si=otro").unwrap(),
            "https://www.youtube.com/watch?v=abc123"
        );
    }

    #[test]
    fn leaves_other_urls_untouched() {
        assert_eq!(
            validate_and_normalize_url("https://vimeo.com/123456").unwrap(),
            "https://vimeo.com/123456"
        );
    }
}
