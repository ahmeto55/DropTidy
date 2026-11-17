use crate::models::Settings;
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

/// A sort batch as stored in the local history database.
pub struct Batch {
    pub batch_id: String,
    pub created_at: i64,
}

/// One source → destination move recorded inside a batch.
pub struct MoveRecord {
    pub source_path: String,
    pub dest_path: String,
}

/// Open (and initialise) the SQLite database inside the app data directory.
pub fn open_db(app_data_dir: &Path) -> Result<Connection, String> {
    fs::create_dir_all(app_data_dir)
        .map_err(|error| format!("Unable to create app data directory: {}", error))?;

    let database_path = app_data_dir.join("droptidy.db");
    let connection = Connection::open(database_path)
        .map_err(|error| format!("Unable to open undo database: {}", error))?;

    connection
        .execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS batches (
                 id         TEXT PRIMARY KEY,
                 created_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS moves (
                 id          INTEGER PRIMARY KEY AUTOINCREMENT,
                 batch_id    TEXT NOT NULL,
                 source_path TEXT NOT NULL,
                 dest_path   TEXT NOT NULL,
                 FOREIGN KEY (batch_id) REFERENCES batches(id) ON DELETE CASCADE
             );
             CREATE INDEX IF NOT EXISTS moves_batch_idx ON moves(batch_id);
             CREATE TABLE IF NOT EXISTS settings (
                 key   TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             );
             INSERT OR IGNORE INTO settings (key, value) VALUES ('auto_organize', 'false');
             INSERT OR IGNORE INTO settings (key, value) VALUES ('watch_delay_secs', '2.5');",
        )
        .map_err(|error| format!("Unable to initialise undo database: {}", error))?;

    Ok(connection)
}

/// Insert a batch and all of its moves atomically.
pub fn insert_batch(
    connection: &mut Connection,
    batch_id: &str,
    timestamp: i64,
    records: &[(String, String)],
) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| format!("Unable to start database transaction: {}", error))?;

    transaction
        .execute(
            "INSERT INTO batches (id, created_at) VALUES (?1, ?2)",
            params![batch_id, timestamp],
        )
        .map_err(|error| format!("Unable to record batch: {}", error))?;

    for (source, destination) in records {
        transaction
            .execute(
                "INSERT INTO moves (batch_id, source_path, dest_path) VALUES (?1, ?2, ?3)",
                params![batch_id, source, destination],
            )
            .map_err(|error| format!("Unable to record a move: {}", error))?;
    }

    transaction
        .commit()
        .map_err(|error| format!("Unable to commit batch: {}", error))?;
    Ok(())
}

/// Most recent batch, or `None` when nothing has been sorted yet.
pub fn latest_batch(connection: &Connection) -> Result<Option<Batch>, String> {
    let mut statement = connection
        .prepare("SELECT id, created_at FROM batches ORDER BY rowid DESC LIMIT 1")
        .map_err(|error| format!("Unable to query batches: {}", error))?;

    let rows = statement
        .query_map([], |row| {
            Ok(Batch {
                batch_id: row.get(0)?,
                created_at: row.get(1)?,
            })
        })
        .map_err(|error| format!("Unable to read batches: {}", error))?;

    for row in rows {
        return row.map(Some).map_err(|error| format!("Bad batch row: {}", error));
    }
    Ok(None)
}

/// All moves belonging to a batch, in the order they were performed.
pub fn moves_for_batch(
    connection: &Connection,
    batch_id: &str,
) -> Result<Vec<MoveRecord>, String> {
    let mut statement = connection
        .prepare(
            "SELECT source_path, dest_path FROM moves WHERE batch_id = ?1 ORDER BY id ASC",
        )
        .map_err(|error| format!("Unable to query moves: {}", error))?;

    let rows = statement
        .query_map(params![batch_id], |row| {
            Ok(MoveRecord {
                source_path: row.get(0)?,
                dest_path: row.get(1)?,
            })
        })
        .map_err(|error| format!("Unable to read moves: {}", error))?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row.map_err(|error| format!("Bad move row: {}", error))?);
    }
    Ok(records)
}

/// Number of moves in a batch (used by the Undo button).
pub fn count_moves(connection: &Connection, batch_id: &str) -> Result<usize, String> {
    connection
        .query_row(
            "SELECT COUNT(*) FROM moves WHERE batch_id = ?1",
            params![batch_id],
            |row| row.get::<_, i64>(0),
        )
        .map(|count| count as usize)
        .map_err(|error| format!("Unable to count moves: {}", error))
}

/// Remove a batch and its move records after a successful undo.
pub fn delete_batch(connection: &mut Connection, batch_id: &str) -> Result<(), String> {
    let transaction = connection
        .transaction()
        .map_err(|error| format!("Unable to start database transaction: {}", error))?;

    transaction
        .execute("DELETE FROM moves WHERE batch_id = ?1", params![batch_id])
        .map_err(|error| format!("Unable to delete move history: {}", error))?;
    transaction
        .execute("DELETE FROM batches WHERE id = ?1", params![batch_id])
        .map_err(|error| format!("Unable to delete batch history: {}", error))?;

    transaction
        .commit()
        .map_err(|error| format!("Unable to commit history deletion: {}", error))?;
    Ok(())
}

fn read_setting(connection: &Connection, key: &str) -> Option<String> {
    connection
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        )
        .ok()
}

fn write_setting(connection: &Connection, key: &str, value: &str) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|error| format!("Unable to save setting '{}': {}", key, error))?;
    Ok(())
}

/// Load persisted auto-organize preferences (with safe defaults).
pub fn get_settings(connection: &Connection) -> Result<Settings, String> {
    let auto_organize = read_setting(connection, "auto_organize")
        .map(|value| value.eq_ignore_ascii_case("true") || value == "1")
        .unwrap_or(false);
    let watch_delay_secs = read_setting(connection, "watch_delay_secs")
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value >= 0.1)
        .unwrap_or(2.5);

    Ok(Settings {
        auto_organize,
        watch_delay_secs,
    })
}

pub fn set_auto_organize(connection: &Connection, enabled: bool) -> Result<(), String> {
    write_setting(connection, "auto_organize", if enabled { "true" } else { "false" })
}

/// Target folder currently used by the watcher (empty when never configured).
pub fn get_target_folder(connection: &Connection) -> Option<String> {
    read_setting(connection, "target_folder").filter(|value| !value.is_empty())
}

pub fn set_target_folder(connection: &Connection, folder: &str) -> Result<(), String> {
    write_setting(connection, "target_folder", folder)
}

pub fn set_watch_delay_secs(connection: &Connection, seconds: f64) -> Result<(), String> {
    if !seconds.is_finite() || seconds < 0.1 {
        return Err("Watch delay must be at least 0.1 seconds.".to_string());
    }
    write_setting(connection, "watch_delay_secs", &seconds.to_string())
}
