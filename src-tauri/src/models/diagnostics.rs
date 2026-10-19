use serde::Serialize;

/// Diagnóstico de dependencias (RF-08) — docs/spec/data-model.md §2.6.
/// Sincronizado con `src/types/contracts.ts` (ToolDiagnostics).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolKind {
    Ytdlp,
    Ffmpeg,
    Ffprobe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolStatus {
    Ok,
    Missing,
    Incompatible,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDiagnostics {
    pub tool: ToolKind,
    pub status: ToolStatus,
    pub version: Option<String>,
    pub path: Option<String>,
}
