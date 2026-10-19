use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::Duration;

use tokio::process::Command;

use crate::models::diagnostics::{ToolDiagnostics, ToolKind, ToolStatus};

/// Límite de tiempo para ejecutar `--version` (RNF-05: diagnóstico asíncrono, no bloqueante).
pub const TOOL_TIMEOUT: Duration = Duration::from_secs(10);

/// Directorios de motores registrados en el arranque (D-09/C): el directorio
/// `engines` del directorio de datos de la app (donde cae el yt-dlp actualizado).
static REENGINE_DIRS: RwLock<Vec<PathBuf>> = RwLock::new(Vec::new());

/// Registra un directorio adicional de motores (directorio de datos de la app).
pub fn register_engine_dir(dir: PathBuf) {
    if let Ok(mut dirs) = REENGINE_DIRS.write() {
        dirs.push(dir);
    }
}

/// Candidatos de motores embebidos junto al ejecutable (mayor prioridad que PATH):
/// bundle macOS (`Contents/Resources/engines`), win/linux junto al exe y build local.
pub fn engine_candidate_dirs(exe_dir: &Path) -> Vec<PathBuf> {
    vec![
        exe_dir.join("../Resources/engines"),
        exe_dir.join("engines"),
        exe_dir.join("../engines"),
    ]
}

/// Busca el ejecutable (embebido → datos de app → PATH, sin shell — RNF-01).
pub async fn which(program: &str) -> Option<PathBuf> {
    // 1) Motores embebidos junto al ejecutable (prioridad sobre PATH — D-09/C).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            for dir in engine_candidate_dirs(exe_dir) {
                if let Some(found) = probe_names(&dir, program) {
                    return Some(found);
                }
            }
        }
    }
    // 2) Directorio de datos (yt-dlp actualizado por el usuario desde la app).
    if let Ok(dirs) = REENGINE_DIRS.read() {
        for dir in dirs.iter() {
            if let Some(found) = probe_names(dir, program) {
                return Some(found);
            }
        }
    }
    // 3) PATH (SO del usuario).
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if let Some(found) = probe_names(&dir, program) {
                return Some(found);
            }
        }
    }
    None
}

/// Sonda de nombres: `program` y, en Windows, `program.exe` (los motores
/// embebidos en Windows guardan su extensión nativa).
fn probe_names(dir: &Path, program: &str) -> Option<PathBuf> {
    let candidate = dir.join(program);
    if is_executable(&candidate) {
        return Some(candidate);
    }
    #[cfg(target_family = "windows")]
    if !program.to_ascii_lowercase().ends_with(".exe") {
        let with_exe = dir.join(format!("{program}.exe"));
        if is_executable(&with_exe) {
            return Some(with_exe);
        }
    }
    #[cfg(not(target_family = "windows"))]
    let _ = dir;
    None
}
/// Localiza un ejecutable (embebido → datos de app → PATH) y consulta su versión con timeout.
/// Clasificación (RF-08):
/// - no está en el PATH → `missing`
/// - existe pero no responde a tiempo o salida vacía → `incompatible`
/// - responde con versión → `ok`
pub async fn locate_and_version(
    tool: ToolKind,
    program: &str,
    version_args: &[&str],
) -> ToolDiagnostics {
    match which(program).await {
        None => ToolDiagnostics {
            tool,
            status: ToolStatus::Missing,
            version: None,
            path: None,
        },
        Some(path) => match run_version(&path, version_args).await {
            Ok(version) if !version.is_empty() => ToolDiagnostics {
                tool,
                status: ToolStatus::Ok,
                version: Some(version),
                path: Some(path.display().to_string()),
            },
            _ => ToolDiagnostics {
                tool,
                status: ToolStatus::Incompatible,
                version: None,
                path: Some(path.display().to_string()),
            },
        },
    }
}

fn is_executable(path: &PathBuf) -> bool {
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(path) {
            Ok(meta) => meta.is_file() && meta.permissions().mode() & 0o111 != 0,
            Err(_) => false,
        }
    }
    #[cfg(target_family = "windows")]
    {
        path.is_file()
    }
}

/// Ejecuta `path args` con timeout y devuelve la primera línea de stdout.
async fn run_version(path: &PathBuf, args: &[&str]) -> Result<String, std::io::Error> {
    let child = Command::new(path)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()?;

    let output = tokio::time::timeout(TOOL_TIMEOUT, child.wait_with_output())
        .await
        .map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::TimedOut, "version check timed out")
        })??;

    Ok(parse_version_output(&String::from_utf8_lossy(&output.stdout)))
}

/// Primera línea no vacía de la salida de `--version`, sin espacios extra.
/// Función pura — cubierta por tests.
pub fn parse_version_output(raw: &str) -> String {
    raw.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_extracts_first_non_empty_line() {
        assert_eq!(parse_version_output("2026.08.19\n"), "2026.08.19");
        assert_eq!(
            parse_version_output("ffmpeg version 9.0.2 Copyright (c) …\nmás líneas\n"),
            "ffmpeg version 9.0.2 Copyright (c) …"
        );
    }

    #[test]
    fn parse_version_ignores_leading_blank_lines() {
        assert_eq!(parse_version_output("\n\n  1.2.3  \n"), "1.2.3");
    }

    #[test]
    fn parse_version_empty_output_is_empty() {
        assert_eq!(parse_version_output(""), "");
        assert_eq!(parse_version_output("   \n \n"), "");
    }

    #[tokio::test]
    async fn which_returns_none_for_unknown_program() {
        let name = "bajatelo-definitely-not-a-real-binary-8f3a";
        assert!(which(name).await.is_none());
    }

    #[tokio::test]
    async fn which_finds_a_real_system_tool() {
        // `/bin/ls` existe en macOS y Linux; test honesto de búsqueda en PATH.
        if std::env::var_os("PATH").is_some() {
            let found = which("ls").await;
            assert!(found.is_some(), "no se encontró 'ls' en el PATH");
        }
    }

    #[test]
    fn engine_candidate_dirs_cover_mac_and_portable_layouts() {
        use std::path::Path;
        let exe_dir = Path::new("/Applications/Bajatelo.app/Contents/MacOS");
        let dirs = engine_candidate_dirs(exe_dir);
        let as_text: Vec<String> = dirs.iter().map(|d| d.to_string_lossy().to_string()).collect();
        // macOS: engines dentro del bundle Resources.
        assert!(as_text.iter().any(|d| d.ends_with("MacOS/../Resources/engines")));
        // Portátiles: junto al exe y el layout de build local.
        assert!(as_text.iter().any(|d| d.ends_with("MacOS/engines")));
        assert!(as_text.iter().any(|d| d.ends_with("MacOS/../engines")));
    }

    /// El directorio de datos (donde cae el yt-dlp actualizado) tiene prioridad
    /// sobre el PATH (D-09/C): registro + resolución.
    #[tokio::test]
    async fn registered_engine_dir_takes_priority_over_path() {
        let fake = std::env::temp_dir().join("bajatelo-engine-priority-test");
        std::fs::create_dir_all(&fake).unwrap();
        let fake_tool = fake.join("bajatelo-priority-tool");
        if !fake_tool.exists() {
            std::fs::write(&fake_tool, b"#!/bin/sh\n").unwrap();
        }
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&fake_tool, std::fs::Permissions::from_mode(0o755));
        }
        register_engine_dir(fake.clone());
        let found = which("bajatelo-priority-tool").await;
        assert!(found.is_some(), "el directorio registrado no se usó");
        assert_eq!(found.unwrap(), fake_tool);
        let _ = std::fs::remove_file(&fake_tool);
        let _ = std::fs::remove_dir(&fake);
    }
}
