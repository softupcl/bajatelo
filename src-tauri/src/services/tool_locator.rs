use std::path::PathBuf;
use std::time::Duration;

use tokio::process::Command;

use crate::models::diagnostics::{ToolDiagnostics, ToolKind, ToolStatus};

/// Límite de tiempo para ejecutar `--version` (RNF-05: diagnóstico asíncrono, no bloqueante).
pub const TOOL_TIMEOUT: Duration = Duration::from_secs(10);

/// Localiza un ejecutable en el PATH y consulta su versión con timeout.
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

/// Busca el ejecutable recorriendo `PATH` (sin depender del shell — RNF-01).
pub async fn which(program: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let dirs = std::env::split_paths(&path_var);
    for dir in dirs {
        let candidate = dir.join(program);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
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
        let name = "rea-definitely-not-a-real-binary-8f3a";
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
}
