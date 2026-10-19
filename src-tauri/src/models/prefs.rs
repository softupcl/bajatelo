use serde::{Deserialize, Serialize};

/// Preferencias (RF-09) — docs/spec/data-model.md §2.5.
/// Sincronizado con `src/types/contracts.ts` (Preferences).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    pub default_destination: String,
    pub always_use_last_path: bool,
    pub never_ask_filename: bool,
    pub conflict_policy: ConflictPolicy,
    pub remember_quality: bool,
    pub remember_format: bool,
    pub remove_finished_from_queue: bool,
    /// X-02: detectar URL compatible al copiar al portapapeles (REA D1).
    pub clipboard_detection: bool,
    pub locale: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictPolicy {
    Ask,
    Number,
    Cancel,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            default_destination: String::new(), // vacío = elegirá el SO por defecto
            always_use_last_path: true,
            never_ask_filename: false,
            conflict_policy: ConflictPolicy::Ask,
            remember_quality: true,
            remember_format: true,
            remove_finished_from_queue: false,
            clipboard_detection: true,
            locale: "es".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let p = Preferences::default();
        assert!(p.always_use_last_path);
        assert!(!p.never_ask_filename);
        assert_eq!(p.conflict_policy, ConflictPolicy::Ask);
        assert!(p.remember_quality && p.remember_format);
        assert!(!p.remove_finished_from_queue);
        assert!(p.clipboard_detection);
        assert_eq!(p.locale, "es");
    }

    #[test]
    fn serializes_camel_case() {
        let p = Preferences::default();
        let json = serde_json::to_string(&p).expect("serializable");
        assert!(json.contains("defaultDestination"));
        assert!(json.contains("conflictPolicy\":\"ask\""));
    }
}
