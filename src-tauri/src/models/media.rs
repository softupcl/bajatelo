use serde::Serialize;

use crate::models::errors::{ErrorCode, TaskError};

/// Metadatos del análisis (RF-02) — docs/spec/data-model.md §2.7.
/// Sincronizado con `src/types/contracts.ts` (MediaInfo / QualityOption).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub title: Option<String>,
    pub portal: Option<String>,
    pub duration_sec: Option<f64>,
    pub qualities: Vec<QualityOption>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityOption {
    pub id: String,
    pub label: String,
    pub height: Option<i64>,
    pub fps: Option<i64>,
    pub container: String,
    pub audio_only: bool,
    /// format_id del audio emparejado, necesario para `-f video+audio` (D-12).
    pub audio_format_id: Option<String>,
    pub estimated_bytes: Option<i64>,
}

/// Contenedores aceptados (REA §5: mp4/m4a; WebM queda post-MVP X-04).
const ACCEPTED_EXTS: [&str; 2] = ["mp4", "m4a"];
/// Audio compatible con contenedor mp4 (REA B3).
const COMPAT_AUDIO_MP4: [&str; 3] = ["aac", "m4a", "mp4"];

/// Parsea la salida `-J` de yt-dlp. EC-10 si el JSON es inválido/vacío,
/// EC-11 si no hay formatos descargables compatibles.
pub fn parse_media_info(json: &str) -> Result<MediaInfo, TaskError> {
    let value: serde_json::Value = serde_json::from_str(json.trim())
        .map_err(|e| TaskError::new(ErrorCode::Ec10, Some(e.to_string())))?;

    let obj = value.as_object().ok_or(TaskError::new(ErrorCode::Ec10, None))?;
    if obj.is_empty() {
        return Err(TaskError::new(ErrorCode::Ec10, None));
    }

    let portal = obj
        .get("extractor")
        .or_else(|| obj.get("ie_key"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_lowercase());

    let title = obj
        .get("title")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let duration_sec = obj.get("duration").and_then(|v| v.as_f64());

    let formats = obj
        .get("formats")
        .and_then(|v| v.as_array())
        .ok_or(TaskError::new(ErrorCode::Ec11, None))?;

    let qualities = derive_qualities(formats);
    if qualities.is_empty() {
        return Err(TaskError::new(ErrorCode::Ec11, None));
    }

    Ok(MediaInfo {
        title,
        portal,
        duration_sec,
        qualities,
    })
}

/// Deriva las opciones de calidad de los formatos reales reportados por el motor
/// (algoritmo del patrón REA §5: separar/ordenar/deduplicar/emparejar/etiquetar).
pub fn derive_qualities(formats: &[serde_json::Value]) -> Vec<QualityOption> {
    let mut video: Vec<&serde_json::Value> = Vec::new();
    let mut audio: Vec<&serde_json::Value> = Vec::new();

    for f in formats {
        let Some(ext) = f.get("ext").and_then(|v| v.as_str()) else {
            continue;
        };
        if !ACCEPTED_EXTS.contains(&ext) {
            continue;
        }
        let vcodec = f.get("vcodec").and_then(|v| v.as_str()).unwrap_or("");
        if vcodec == "none" {
            audio.push(f);
        } else {
            video.push(f);
        }
    }

    // Vídeo: altura desc, fps desc (REA §5.2).
    video.sort_by(|a, b| {
        let key = |f: &serde_json::Value| {
            (
                f.get("height").and_then(|v| v.as_i64()).unwrap_or(0),
                f.get("fps").and_then(|v| v.as_i64()).unwrap_or(0),
            )
        };
        key(b).cmp(&key(a))
    });
    // Audio: bitrate total desc (REA §5.2).
    audio.sort_by(|a, b| {
        let tbr = |f: &serde_json::Value| f.get("tbr").and_then(|v| v.as_f64()).unwrap_or(0.0);
        tbr(b).partial_cmp(&tbr(a)).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Deduplicar vídeo por (height, fps).
    let mut seen = std::collections::HashSet::new();
    video.retain(|f| {
        let key = (
            f.get("height").and_then(|v| v.as_i64()).unwrap_or(0),
            f.get("fps").and_then(|v| v.as_i64()).unwrap_or(0),
        );
        seen.insert(key)
    });

    let total = video.len();
    let mut qualities = Vec::with_capacity(total);
    for (i, f) in video.iter().enumerate() {
        let height = f.get("height").and_then(|v| v.as_i64());
        let fps = f.get("fps").and_then(|v| v.as_i64());
        let ext = f.get("ext").and_then(|v| v.as_str()).unwrap_or("mp4");
        let id = f
            .get("format_id")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .or_else(|| f.get("format_id").and_then(|v| v.as_i64()).map(|n| n.to_string()))
            .unwrap_or_default();

        let mut label = quality_label(height, fps, ext);
        if ext == "webm" {
            label.push_str(" WebM");
        }

        // Emparejar audio compatible si el vídeo no trae pista propia (REA §5.4).
        let mut audio_format_id = None;
        let mut audio_bytes = None;
        let compatible: Vec<&&serde_json::Value> = audio
            .iter()
            .filter(|a| {
                let a_ext = a.get("ext").and_then(|v| v.as_str()).unwrap_or("");
                COMPAT_AUDIO_MP4.contains(&a_ext)
            })
            .collect();
        let needs_audio = f.get("acodec").and_then(|v| v.as_str()) == Some("none");
        if needs_audio && !compatible.is_empty() {
            let idx = if total > 1 {
                (compatible.len() - 1) * i / total
            } else {
                compatible.len() - 1
            };
            let chosen = compatible[idx];
            audio_format_id = chosen
                .get("format_id")
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .or_else(|| {
                    chosen
                        .get("format_id")
                        .and_then(|v| v.as_i64())
                        .map(|n| n.to_string())
                });
            audio_bytes = chosen.get("filesize").and_then(|v| v.as_i64());
        }

        qualities.push(QualityOption {
            id,
            label,
            height,
            fps,
            container: ext.to_string(),
            audio_only: false,
            audio_format_id,
            estimated_bytes: f.get("filesize").and_then(|v| v.as_i64()).map(|b| {
                b + audio_bytes.unwrap_or(0)
            }),
        });
    }
    qualities
}

/// Etiqueta visible: `1080p (HD)`, `2160p (4K)`, `4320p (8K)`, `NNp60` (fps≥59),
/// `unknown` si la resolución es 0/desconocida (REA B2).
pub fn quality_label(height: Option<i64>, fps: Option<i64>, _container: &str) -> String {
    let height = match height {
        Some(0) | None => return "unknown".to_string(),
        Some(h) => h,
    };
    let mut label = format!("{height}p");
    if let Some(fps) = fps {
        if fps >= 59 {
            label.push_str("60");
        }
    }
    if height >= 4000 {
        label.push_str(" (8K)");
    } else if height >= 2000 {
        label.push_str(" (4K)");
    } else if height >= 700 {
        label.push_str(" (HD)");
    }
    label
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn labels_follow_reference_pattern() {
        assert_eq!(quality_label(Some(1080), None, "mp4"), "1080p (HD)");
        assert_eq!(quality_label(Some(2160), None, "mp4"), "2160p (4K)");
        assert_eq!(quality_label(Some(4320), None, "mp4"), "4320p (8K)");
        assert_eq!(quality_label(Some(720), Some(60), "mp4"), "720p60 (HD)");
        assert_eq!(quality_label(Some(360), None, "mp4"), "360p");
        assert_eq!(quality_label(Some(0), None, "mp4"), "unknown");
        assert_eq!(quality_label(None, None, "mp4"), "unknown");
    }

    #[test]
    fn parse_invalid_json_is_ec10() {
        assert_eq!(parse_media_info("no-json").unwrap_err().code, ErrorCode::Ec10);
        assert_eq!(parse_media_info("  ").unwrap_err().code, ErrorCode::Ec10);
        assert_eq!(parse_media_info("{}").unwrap_err().code, ErrorCode::Ec10);
    }

    #[test]
    fn parse_without_formats_is_ec11() {
        assert_eq!(
            parse_media_info(r#"{"title":"x"}"#).unwrap_err().code,
            ErrorCode::Ec11
        );
        assert_eq!(
            parse_media_info(r#"{"title":"x","formats":[]}"#)
                .unwrap_err()
                .code,
            ErrorCode::Ec11
        );
    }

    #[test]
    fn derives_sorted_deduped_and_paired_qualities() {
        let json = json!({
            "title": "Demo",
            "extractor": "youtube",
            "duration": 61.5,
            "formats": [
                {"format_id": "18", "ext": "mp4", "height": 360, "fps": 30, "vcodec": "avc1", "acodec": "mp4a", "filesize": 1000},
                {"format_id": "137", "ext": "mp4", "height": 1080, "fps": 30, "vcodec": "avc1", "acodec": "none", "filesize": 5000},
                {"format_id": "136", "ext": "mp4", "height": 720, "fps": 30, "vcodec": "avc1", "acodec": "none", "filesize": 3000},
                {"format_id": "140", "ext": "m4a", "vcodec": "none", "acodec": "mp4a", "tbr": 128.0, "filesize": 800},
                {"format_id": "249", "ext": "webm", "vcodec": "none", "acodec": "opus", "tbr": 60.0, "filesize": 300}
            ]
        });
        let info = parse_media_info(json.to_string().as_str()).unwrap();
        assert_eq!(info.title.as_deref(), Some("Demo"));
        assert_eq!(info.portal.as_deref(), Some("youtube"));
        assert_eq!(info.qualities.len(), 3); // webm audio descartado; dedup ok
        assert_eq!(info.qualities[0].label, "1080p (HD)");
        assert_eq!(info.qualities[1].label, "720p (HD)");
        assert_eq!(info.qualities[2].label, "360p");
        // La 1080p no trae audio → emparejada con el mejor audio compatible (140/m4a).
        assert_eq!(info.qualities[0].estimated_bytes, Some(5800));
        // La 360p trae audio propio → tamaño sin extra.
        assert_eq!(info.qualities[2].estimated_bytes, Some(1000));
    }

    #[test]
    fn deduplicates_same_height_and_fps() {
        let json = json!([
            {"format_id": "a", "ext": "mp4", "height": 720, "fps": 30, "vcodec": "avc1", "acodec": "mp4a"},
            {"format_id": "b", "ext": "mp4", "height": 720, "fps": 30, "vcodec": "avc1.2", "acodec": "mp4a"}
        ]);
        assert_eq!(derive_qualities(json.as_array().unwrap()).len(), 1);
    }
}
