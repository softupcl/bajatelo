/// Persistencia SQLite (D-07 / V-03) — docs/spec/data-model.md §3.
/// - `rusqlite` con feature `bundled`: SQLite se compila con la app.
/// - Journal mode WAL.
/// - Apertura con `integrity_check`: si falla, intenta restaurar `bajatelo.db.bak`;
///   si no hay backup válido, aísla el archivo corrupto y crea una base nueva
///   (error EC-41 para la UI cuando exista el canal de avisos).
/// - Sin secretos: solo historial, preferencias y cola.
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::models::history::HistoryEntry;
use crate::models::prefs::Preferences;

pub struct Store {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenOutcome {
    /// Base abierta normalmente.
    Clean,
    /// Base corrupta restaurada desde el backup.
    RecoveredFromBackup,
    /// Base corrupta sin backup válido: se creó una nueva (datos previos aislados).
    Recreated,
}

impl Store {
    /// Abre (o crea) la base en `path`, ejecutando integridad y migraciones.
    pub fn open(path: &Path) -> Result<(Self, OpenOutcome), rusqlite::Error> {
        let (conn, outcome) = open_with_recovery(path)?;
        configure(&conn)?;
        migrate(&conn)?;
        Ok((Self { conn: Mutex::new(conn) }, outcome))
    }

    /// Copia de seguridad consistente vía `VACUUM INTO` (a llamar al cerrar limpio).
    #[allow(dead_code)] // contrato V-03; se conectará al cierre de la app (D-12)
    pub fn backup(&self, backup_path: &Path) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().expect("store lock");
        if backup_path.exists() {
            std::fs::remove_file(backup_path)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        }
        conn.execute("VACUUM INTO ?1", params![backup_path.to_string_lossy()])?;
        Ok(())
    }

    /// Preferencias (RF-09): carga con valores por defecto aplicados.
    pub fn get_preferences(&self) -> Result<Preferences, rusqlite::Error> {
        let conn = self.conn.lock().expect("store lock");
        let raw: Option<String> = conn
            .query_row(
                "SELECT value FROM preferences WHERE key = 'preferences'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        match raw {
            Some(json) => Ok(serde_json::from_str(&json).unwrap_or_default()),
            None => Ok(Preferences::default()),
        }
    }

    /// Guarda el conjunto completo de preferencias en una transacción.
    pub fn save_preferences(&self, prefs: &Preferences) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().expect("store lock");
        let json = serde_json::to_string(prefs)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        conn.execute(
            "INSERT INTO preferences (key, value) VALUES ('preferences', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![json],
        )?;
        Ok(())
    }

    /// Inserta una entrada de historial (RF-07). (Se usa en D-12/C-21.)
    #[allow(dead_code)]
    pub fn history_insert(&self, entry: &HistoryEntry) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().expect("store lock");
        conn.execute(
            "INSERT INTO history (id, kind, title, portal, source_url, format, quality,
                                   destination_path, final_size_bytes, status, error_code,
                                   started_at, finished_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                entry.id,
                entry.kind,
                entry.title,
                entry.portal,
                entry.source_url,
                entry.format,
                entry.quality,
                entry.destination_path,
                entry.final_size_bytes,
                entry.status,
                entry.error_code,
                entry.started_at,
                entry.finished_at
            ],
        )?;
        Ok(())
    }

    /// Lista el historial en orden cronológico inverso (más reciente primero).
    pub fn history_list(&self) -> Result<Vec<HistoryEntry>, rusqlite::Error> {
        let conn = self.conn.lock().expect("store lock");
        let mut stmt = conn.prepare(
            "SELECT id, kind, title, portal, source_url, format, quality, destination_path,
                    final_size_bytes, status, error_code, started_at, finished_at
             FROM history ORDER BY finished_at DESC, id DESC",
        )?;
        let rows = stmt.query_map([], map_history_row)?;
        rows.collect()
    }

    /// Elimina una entrada; devuelve `true` si existía.
    pub fn history_remove(&self, id: &str) -> Result<bool, rusqlite::Error> {
        let conn = self.conn.lock().expect("store lock");
        let changed = conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
        Ok(changed > 0)
    }

    /// Borra todo el historial; devuelve el número de entradas eliminadas.
    pub fn history_clear(&self) -> Result<usize, rusqlite::Error> {
        let conn = self.conn.lock().expect("store lock");
        Ok(conn.execute("DELETE FROM history", [])?)
    }
}

fn open_with_recovery(path: &Path) -> Result<(Connection, OpenOutcome), rusqlite::Error> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        }
        return Ok((Connection::open(path)?, OpenOutcome::Clean));
    }

    let conn = Connection::open(path)?;
    if integrity_ok(&conn)? {
        return Ok((conn, OpenOutcome::Clean));
    }
    drop(conn);

    // Intento 1: restaurar desde backup.
    let bak = backup_path_for(path);
    if bak.exists() {
        std::fs::copy(&bak, path)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let conn = Connection::open(path)?;
        if integrity_ok(&conn)? {
            return Ok((conn, OpenOutcome::RecoveredFromBackup));
        }
        drop(conn);
    }

    // Intento 2: aislar el archivo corrupto y crear base nueva (EC-41 en la UI).
    let isolated = path.with_extension(format!(
        "db.corrupt-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or_default()
    ));
    std::fs::rename(path, &isolated)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    Ok((Connection::open(path)?, OpenOutcome::Recreated))
}

fn backup_path_for(db_path: &Path) -> PathBuf {
    let file = db_path.file_name().map_or_else(
        || "bajatelo.db".to_string(),
        |f| format!("{}.bak", f.to_string_lossy()),
    );
    db_path.with_file_name(file)
}

fn integrity_ok(conn: &Connection) -> Result<bool, rusqlite::Error> {
    // Si el propio check falla (p. ej. archivo no-SQLite), la base no es íntegra.
    match conn.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0)) {
        Ok(result) => Ok(result == "ok"),
        Err(_) => Ok(false),
    }
}

fn configure(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

/// Migraciones numeradas vía `PRAGMA user_version` (v1 = esquema inicial).
fn migrate(conn: &Connection) -> Result<(), rusqlite::Error> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version >= 1 {
        return Ok(());
    }
    conn.execute_batch(
        "BEGIN;
         CREATE TABLE history (
           id TEXT PRIMARY KEY,
           kind TEXT NOT NULL,
           title TEXT NOT NULL,
           portal TEXT,
           source_url TEXT,
           format TEXT,
           quality TEXT,
           destination_path TEXT NOT NULL,
           final_size_bytes INTEGER,
           status TEXT NOT NULL,
           error_code TEXT,
           started_at TEXT NOT NULL,
           finished_at TEXT NOT NULL
         );
         CREATE TABLE preferences (
           key TEXT PRIMARY KEY,
           value TEXT NOT NULL
         );
         CREATE TABLE tasks (
           id TEXT PRIMARY KEY,
           payload_json TEXT NOT NULL,
           state TEXT NOT NULL,
           updated_at TEXT NOT NULL
         );
         PRAGMA user_version = 1;
         COMMIT;",
    )
}

fn map_history_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryEntry> {
    Ok(HistoryEntry {
        id: row.get(0)?,
        kind: row.get(1)?,
        title: row.get(2)?,
        portal: row.get(3)?,
        source_url: row.get(4)?,
        format: row.get(5)?,
        quality: row.get(6)?,
        destination_path: row.get(7)?,
        final_size_bytes: row.get(8)?,
        status: row.get(9)?,
        error_code: row.get(10)?,
        started_at: row.get(11)?,
        finished_at: row.get(12)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::prefs::ConflictPolicy;

    fn temp_db_path(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir();
        let unique = format!(
            "bajatelo-test-{}-{}-{}.db",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        dir.join(unique)
    }

    fn sample_entry(id: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            kind: "download".to_string(),
            title: "Vídeo de prueba".to_string(),
            portal: Some("youtube".to_string()),
            source_url: Some("https://www.youtube.com/watch?v=abc".to_string()),
            format: Some("mp4".to_string()),
            quality: Some("1080p (HD)".to_string()),
            destination_path: "/tmp/video.mp4".to_string(),
            final_size_bytes: Some(12345),
            status: "completed".to_string(),
            error_code: None,
            started_at: "2026-10-19T10:00:00Z".to_string(),
            finished_at: "2026-10-19T10:01:00Z".to_string(),
        }
    }

    #[test]
    fn opens_new_database_and_migrates() {
        let path = temp_db_path("open");
        let (store, outcome) = Store::open(&path).expect("open");
        assert_eq!(outcome, OpenOutcome::Clean);
        assert_eq!(store.history_list().unwrap(), Vec::new());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn preferences_round_trip_with_defaults() {
        let path = temp_db_path("prefs");
        let (store, _) = Store::open(&path).unwrap();
        let mut prefs = store.get_preferences().unwrap();
        assert_eq!(prefs, Preferences::default());
        prefs.never_ask_filename = true;
        prefs.conflict_policy = ConflictPolicy::Number;
        store.save_preferences(&prefs).unwrap();
        let reloaded = store.get_preferences().unwrap();
        assert!(reloaded.never_ask_filename);
        assert_eq!(reloaded.conflict_policy, ConflictPolicy::Number);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn history_insert_list_remove_clear() {
        let path = temp_db_path("hist");
        let (store, _) = Store::open(&path).unwrap();
        store.history_insert(&sample_entry("1")).unwrap();
        store.history_insert(&sample_entry("2")).unwrap();

        let listed = store.history_list().unwrap();
        assert_eq!(listed.len(), 2);

        assert!(store.history_remove("1").unwrap());
        assert!(!store.history_remove("1").unwrap());
        assert_eq!(store.history_list().unwrap().len(), 1);

        assert_eq!(store.history_clear().unwrap(), 1);
        assert_eq!(store.history_list().unwrap().len(), 0);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn backup_creates_restorable_copy() {
        let path = temp_db_path("backup");
        let bak = backup_path_for(&path);
        let (store, _) = Store::open(&path).unwrap();
        store.history_insert(&sample_entry("1")).unwrap();
        store.backup(&bak).unwrap();
        assert!(bak.exists());
        // Reabrir la copia de backup debe tener los datos.
        let (store2, _) = Store::open(&bak).unwrap();
        assert_eq!(store2.history_list().unwrap().len(), 1);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&bak);
    }

    #[test]
    fn corrupt_database_recovers_from_backup() {
        let path = temp_db_path("corrupt");
        let bak = backup_path_for(&path);
        // Base válida con datos + backup.
        let (store, _) = Store::open(&path).unwrap();
        store.history_insert(&sample_entry("keep")).unwrap();
        store.backup(&bak).unwrap();
        drop(store);
        // Corromper la base principal.
        std::fs::write(&path, b"esto no es una base de datos sqlite").unwrap();

        let (store, outcome) = Store::open(&path).unwrap();
        assert_eq!(outcome, OpenOutcome::RecoveredFromBackup);
        assert_eq!(store.history_list().unwrap().len(), 1);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&bak);
    }

    #[test]
    fn corrupt_database_without_backup_recreates() {
        let path = temp_db_path("recreate");
        std::fs::write(&path, b"basura corrupta").unwrap();
        let (store, outcome) = Store::open(&path).unwrap();
        assert_eq!(outcome, OpenOutcome::Recreated);
        assert_eq!(store.history_list().unwrap().len(), 0);
        let _ = std::fs::remove_file(&path);
    }
}
