use serde::Serialize;

/// Progreso (RF-04) — docs/spec/data-model.md §2.2.
/// null = indeterminado visible; prohibido fabricar valores (RF-04 CA-04.3).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressInfo {
    pub percent: Option<f64>,
    pub bytes_downloaded: Option<i64>,
    pub bytes_total: Option<i64>,
    pub speed_bps: Option<f64>,
    pub eta_seconds: Option<i64>,
    /// Fase actual del flujo (descarga o conversión).
    pub phase: Phase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Downloading,
    Converting,
}

/// Línea de salida del motor (con `--newline`) clasificada (patrón REA §4.3).
#[derive(Debug, Clone, PartialEq)]
pub enum MotorLine {
    /// `[download] Destination: <ruta>` — nuevo archivo temporal destino.
    Destination(String),
    /// `<ruta> has already been downloaded` — reanudación/nada que hacer.
    AlreadyDownloaded(String),
    /// `[Merger] Merging formats into "<ruta>"` — archivo final combinado.
    Merged(String),
    /// Progreso porcentual del archivo actual.
    Downloaded {
        percent: f64,
        /// Estimación de tamaño total en bytes del archivo actual.
        size_estimate: i64,
        speed_bps: Option<f64>,
        eta_seconds: Option<i64>,
    },
    /// `ERROR: …`
    Error(String),
    Otra,
}

const SUFFIXES: [(&str, f64); 4] = [("TiB", 4.0), ("GiB", 3.0), ("MiB", 2.0), ("KiB", 1.0)];

/// Convierte "4.15MiB" en bytes separando número y sufijo binario.
fn size_to_bytes(text: &str) -> Option<i64> {
    let idx = text.find(|c: char| c.is_ascii_alphabetic())?;
    let number: f64 = text[..idx].parse().ok()?;
    let suffix = &text[idx..];
    for (known, power) in SUFFIXES {
        if suffix.starts_with(known) {
            return Some((number * 1024f64.powf(power)).round() as i64);
        }
    }
    None
}

fn speed_to_bps(text: &str) -> Option<f64> {
    let (value_part, _) = text.split_once("/s")?;
    let value = value_part.trim();
    let idx = value.find(|c: char| c.is_ascii_alphabetic())?;
    let number: f64 = value[..idx].parse().ok()?;
    let suffix = &value[idx..];
    for (known, power) in SUFFIXES {
        if suffix.starts_with(known) {
            return Some(number * 1024f64.powf(power));
        }
    }
    if suffix.starts_with('B') {
        return Some(number);
    }
    None
}

/// Convierte "00:04" / "01:02:03" a segundos.
fn eta_to_seconds(text: &str) -> Option<i64> {
    let mut seconds = 0i64;
    let mut parts = 0usize;
    for part in text.split(':') {
        let value: i64 = part.trim().parse().ok()?;
        seconds = seconds.saturating_mul(60).saturating_add(value);
        parts += 1;
    }
    (parts >= 2).then_some(seconds)
}

/// Clasifica una línea de stdout del motor. Función pura — tests con
/// casos reales adaptados del patrón REA (video.cpp:389-450).
pub fn parse_motor_line(line: &str) -> MotorLine {
    let trimmed = line.trim();
    if let Some(rest) = stripped_prefix(trimmed, "[download] Destination: ") {
        return MotorLine::Destination(rest.trim().to_string());
    }
    if let Some(path) = extract_quoted_after(trimmed, "Merging formats into ") {
        return MotorLine::Merged(path);
    }
    if let Some(path) = before_suffix(trimmed, " has already been downloaded") {
        // "[download] /ruta" — quita el prefijo de clase del motor.
        let clean = path.trim().trim_start_matches("[download]").trim();
        return MotorLine::AlreadyDownloaded(clean.to_string());
    }
    if let Some(rest) = stripped_prefix(trimmed, "[download] ") {
        if rest.contains('%') {
            if let Some(progress) = parse_progress_body(rest) {
                return progress;
            }
        }
    }
    if trimmed.starts_with("ERROR:") {
        return MotorLine::Error(trimmed.trim_start_matches("ERROR:").trim().to_string());
    }
    MotorLine::Otra
}

fn stripped_prefix<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    text.strip_prefix(prefix)
}

/// Extrae lo que hay entre comillas después de un prefijo — `[Merger] Merging formats into "x"`.
fn extract_quoted_after(text: &str, needle: &str) -> Option<String> {
    let idx = text.find(needle)?;
    let rest = &text[idx + needle.len()..];
    let quoted = rest.trim_start_matches('"');
    let end = quoted.find('"')?;
    Some(quoted[..end].to_string())
}

fn before_suffix<'a>(text: &'a str, suffix: &str) -> Option<&'a str> {
    let end = text.find(suffix)?;
    Some(text[..end].trim())
}

/// Cuerpo de una línea de progreso: `43.0% of    4.15MiB at  890.12KiB/s ETA 00:03`.
fn parse_progress_body(body: &str) -> Option<MotorLine> {
    let percent_str = body.split_whitespace().next()?;
    let percent: f64 = percent_str.strip_suffix('%')?.parse().ok()?;

    let of_idx = body.find("of")?;
    let after_of = &body[of_idx + 2..];
    // El tamaño total puede llegar con el indicador de estimación "~" pegado
    // ("of ~ 1.00TiB") o pegado al número ("of ~4.15MiB"): salta esos tokens.
    let mut size_token = "";
    let mut saw_size = false;
    for token in after_of.trim().split_whitespace() {
        if token == "~" {
            continue;
        }
        size_token = token.trim_start_matches('~');
        saw_size = true;
        break;
    }
    if !saw_size {
        return None;
    }
    let size_estimate = size_to_bytes(size_token)?;

    let speed_bps = body
        .split(" at ")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(speed_to_bps);
    let eta_seconds = body
        .split("ETA ")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(eta_to_seconds);

    Some(MotorLine::Downloaded {
        percent,
        size_estimate,
        speed_bps,
        eta_seconds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use MotorLine::*;

    #[test]
    fn parses_destination_line() {
        assert_eq!(
            parse_motor_line("[download] Destination: /tmp/cg-yt-v1-137.mp4"),
            Destination("/tmp/cg-yt-v1-137.mp4".into())
        );
    }

    #[test]
    fn parses_progress_with_all_fields() {
        let got = parse_motor_line("[download]  43.0% of    4.15MiB at  890.12KiB/s ETA 00:03");
        match got {
            Downloaded { percent, size_estimate, speed_bps, eta_seconds } => {
                assert!((percent - 43.0).abs() < f64::EPSILON);
                assert!((size_estimate - 4_350_904).abs() <= 1024); // aprox 4.15MiB
                assert!((speed_bps.unwrap() - 890.12 * 1024.0).abs() < 1024.0);
                assert_eq!(eta_seconds, Some(3));
            }
            other => panic!("esperaba Downloaded, got {other:?}"),
        }
    }

    #[test]
    fn parses_unknown_size_and_long_eta() {
        let got = parse_motor_line("[download]  1.2% of ~ 1.00TiB at  12.50MiB/s ETA 01:02:03");
        match got {
            Downloaded { percent, size_estimate, eta_seconds, speed_bps } => {
                assert_eq!(percent, 1.2);
                assert_eq!(size_estimate, 1024f64.powi(4).round() as i64);
                assert_eq!(eta_seconds, Some(3723));
                assert!(speed_bps.is_some());
            }
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn parses_merging_and_already_downloaded() {
        assert_eq!(
            parse_motor_line("[Merger] Merging formats into \"/tmp/cg-yt-v1.mp4\""),
            Merged("/tmp/cg-yt-v1.mp4".into())
        );
        assert_eq!(
            parse_motor_line("[download] /tmp/f.mp4 has already been downloaded and merged"),
            AlreadyDownloaded("/tmp/f.mp4".into())
        );
    }

    #[test]
    fn classifies_error_lines() {
        assert_eq!(parse_motor_line("ERROR: boop"), Error("boop".into()));
        assert_eq!(parse_motor_line("[download] Destination: x"), Destination("x".into()));
        assert_eq!(parse_motor_line("algo suelto"), Otra);
    }

    #[test]
    fn size_conversions() {
        assert_eq!(size_to_bytes("1.00KiB"), Some(1024));
        assert_eq!(size_to_bytes("2.50MiB"), Some(2_621_440));
        assert_eq!(speed_to_bps("890.12KiB/s"), Some(890.12 * 1024.0));
    }
}
