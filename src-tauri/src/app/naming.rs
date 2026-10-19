/// Nombres de archivo seguros (RF-03/CA-03.2; REA C2 extendido): lista negra de
/// caracteres + reservas de Windows + terminadores prohibidos + contra path traversal.
/// Funciones puras — cubiertas por tests.

/// Caracteres prohibidos en nombres de archivo (REA C2 + reservas de macOS/Windows).
const FORBIDDEN: [char; 21] = [
    '#', '%', '&', '{', '}', '\\', '<', '>', '*', '?', '/', '$', '!', '\'', '"', ':', '@', '+',
    '`', '|', '=',
];

/// Reservas de dispositivos en Windows.
const WINDOWS_RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Título del análisis → nombre de archivo base seguro (sin extensión).
pub fn sanitize_filename(title: &str) -> String {
    // Caracteres prohibidos y de control → espacio (separador neutro).
    let cleaned: String = title
        .chars()
        .map(|c| if FORBIDDEN.contains(&c) || c.is_control() { ' ' } else { c })
        .collect();

    // Confian: recorta extremos, colapsa points interiores y espacios repetidos.
    let collapsed = collapse_spaces(&collapse_dots(cleaned.trim()));
    let final_name = collapsed.trim().trim_end_matches('.').trim();

    if final_name.is_empty() {
        return "descarga".to_string();
    }
    // Reservas de Windows (nombre con o sin extensión).
    let stem = final_name.split('.').next().unwrap_or("").to_ascii_uppercase();
    if WINDOWS_RESERVED.contains(&stem.as_str()) {
        return format!("resultado-{final_name}");
    }
    // Evita dotfiles ocultos y residuos de truncación (path traversal imposible:
    // ninguna secuencia de ".." sobrevive al collapse).
    if final_name.starts_with('.') {
        return format!("resultado-{final_name}");
    }
    final_name.to_string()
}

/// Colapsa "..+" interior a "." — tras esto no queda traversal unida.
fn collapse_dots(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_dots = false;
    for c in text.chars() {
        if c == '.' {
            if !in_dots {
                out.push('.');
            }
            in_dots = true;
        } else {
            out.push(c);
            in_dots = false;
        }
    }
    out
}

/// Colapsa espacios repetidos producidos por la sustitución de caracteres.
fn collapse_spaces(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_space = false;
    for c in text.chars() {
        if c == ' ' {
            if !last_space {
                out.push(' ');
            }
            last_space = true;
        } else {
            out.push(c);
            last_space = false;
        }
    }
    out
}

/// Resuelve el conflicto de destino según la política (RF-03/D-04):
/// - `Ask`: el diálogo nativo de guardado ya gestionó la confirmación → se respeta el
///   destino tal cual (sobrescritura autorizada por el usuario en el diálogo).
/// - `Number`: si existe, añade sufijos `-1`, `-2`, … (patrón REA E5).
/// - `Cancel`: si existe, error EC-26.
pub fn resolve_destination(
    target_base: &str,
    extension: &str,
    policy: crate::models::prefs::ConflictPolicy,
) -> Result<String, crate::models::errors::TaskError> {
    use crate::models::errors::{ErrorCode, TaskError};
    use crate::models::prefs::ConflictPolicy::*;

    let base = target_base.trim_end();
    match policy {
        Ask => Ok(format!("{base}.{extension}")),
        Number => {
            // Convención REA E5: `nombre.ext` existe → `nombre-1.ext`, `-2`, …
            let mut n: i64 = 0;
            loop {
                let candidate = if n == 0 {
                    format!("{base}.{extension}")
                } else {
                    format!("{base}-{n}.{extension}")
                };
                if !std::path::Path::new(&candidate).exists() {
                    return Ok(candidate);
                }
                n += 1;
                if n > 10_000 {
                    // Protección ante bucles patológicos.
                    return Err(TaskError::new(
                        ErrorCode::Ec26,
                        Some("demasiados nombres en uso".into()),
                    ));
                }
            }
        }
        Cancel => {
            if std::path::Path::new(&format!("{base}.{extension}")).exists() {
                return Err(TaskError::new(
                    ErrorCode::Ec26,
                    Some("el destino ya existe (política: cancelar)".into()),
                ));
            }
            Ok(format!("{base}.{extension}"))
        }
    }
}

/// Quita del destino propuesto la extensión final para normalizar el base
/// (la extensión real la decide la política/formato del clon).
pub fn strip_extension(path: &str) -> &str {
    let trimmed = path.trim_end();
    match trimmed.rfind('.') {
        Some(idx) if idx > trimmed.rfind('/').unwrap_or(0) + 1 => &trimmed[..idx],
        _ => trimmed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::errors::ErrorCode;
    use crate::models::prefs::ConflictPolicy;

    #[test]
    fn strip_extension_removes_only_final_extension() {
        assert_eq!(strip_extension("/tmp/vídeo.mp4"), "/tmp/vídeo");
        assert_eq!(strip_extension("/tmp/algo.sin.formato.x"), "/tmp/algo.sin.formato");
        // No "extensiones" de carpetas con punto.
        assert_eq!(strip_extension("/tmp/.oculto/archivo"), "/tmp/.oculto/archivo");
        assert_eq!(strip_extension("/tmp/archivo"), "/tmp/archivo");
        // Un dotfile es un nombre completo, no un destino con extensión: no se quita.
        assert_eq!(strip_extension(".gitignore"), ".gitignore");
    }

    #[test]
    fn removes_forbidden_characters() {
        assert_eq!(sanitize_filename("Vídeo: mañana/2013?"), "Vídeo mañana 2013");
        assert_eq!(sanitize_filename("a<b>c\\d|e*f?"), "a b c d e f");
        assert_eq!(sanitize_filename("por 50% off! gana#"), "por 50 off gana");
    }

    #[test]
    fn trims_dots_and_spaces() {
        assert_eq!(sanitize_filename("  final.  "), "final");
        // ".." interior se colapsa a "."; extremos se recortan.
        assert_eq!(sanitize_filename("final.. (2024) ."), "final. (2024)");
    }

    #[test]
    fn empty_becomes_fallback() {
        assert_eq!(sanitize_filename("???"), "descarga");
        assert_eq!(sanitize_filename(""), "descarga");
    }

    #[test]
    fn windows_reserved_names() {
        assert_eq!(sanitize_filename("CON"), "resultado-CON");
        assert_eq!(sanitize_filename("com1.txt"), "resultado-com1.txt");
    }

    #[test]
    fn traversal_attempts_are_neutered() {
        // "/" → espacio (FORBIDDEN) y ".." colapsado: punto inicial dispara
        // el prefijo "resultado-" (sin dotfiles ni traversal unida).
        assert_eq!(sanitize_filename("../etc_passwd"), "resultado-. etc_passwd");
        // ".." puro: colapsado y recortado → fallback seguro.
        assert_eq!(sanitize_filename(".."), "descarga");
    }

    #[test]
    fn no_newline_nor_control_characters() {
        assert_eq!(sanitize_filename("línea\nnueva\u{0007}"), "línea nueva");
    }

    #[test]
    fn numbered_policy_increments_for_existing_files() {
        let dir = std::env::temp_dir().join("rea-naming-test");
        let _ = std::fs::create_dir_all(&dir);
        let base = dir.join("clip").to_string_lossy().to_string();
        let _ = std::fs::remove_file(format!("{base}.mp4"));
        let _ = std::fs::remove_file(format!("{base}-1.mp4"));

        assert_eq!(
            resolve_destination(&base, "mp4", ConflictPolicy::Number).unwrap(),
            format!("{base}.mp4")
        );
        std::fs::write(format!("{base}.mp4"), b"x").unwrap();
        assert_eq!(
            resolve_destination(&base, "mp4", ConflictPolicy::Number).unwrap(),
            format!("{base}-1.mp4")
        );
        std::fs::write(format!("{base}-1.mp4"), b"x").unwrap();
        assert!(resolve_destination(&base, "mp4", ConflictPolicy::Number).is_ok());
        let _ = std::fs::remove_file(format!("{base}.mp4"));
        let _ = std::fs::remove_file(format!("{base}-1.mp4"));
    }

    #[test]
    fn cancel_policy_rejects_existing() {
        let dir = std::env::temp_dir().join("rea-naming-test");
        std::fs::create_dir_all(&dir).unwrap();
        let base = dir.join("cancelcase").to_string_lossy().to_string();
        std::fs::write(format!("{base}.mp3"), b"x").unwrap();
        assert_eq!(
            resolve_destination(&base, "mp3", ConflictPolicy::Cancel).unwrap_err().code,
            ErrorCode::Ec26
        );
        let _ = std::fs::remove_file(format!("{base}.mp3"));
    }

    #[test]
    fn ask_policy_respects_the_confirmed_target() {
        assert_eq!(
            resolve_destination("/tmp/dest", "mp4", ConflictPolicy::Ask).unwrap(),
            "/tmp/dest.mp4"
        );
    }
}
