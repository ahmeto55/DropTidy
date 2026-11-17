use crate::db;
use crate::models::{BatchInfo, OrganizeResult, UndoResult};
use crate::scanner::{categorize, extension_of, unique_path};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

pub(crate) fn new_batch_id() -> String {
    format!("batch-{}-{}", now_millis(), std::process::id())
}

/// Rename when possible; fall back to copy + delete for cross-drive moves.
fn safe_move(source: &Path, destination: &Path) -> Result<(), String> {
    if fs::rename(source, destination).is_ok() {
        return Ok(());
    }

    fs::copy(source, destination)
        .map_err(|error| format!("copy to '{}' failed: {}", destination.display(), error))?;
    fs::remove_file(source)
        .map_err(|error| format!("removing source '{}' failed: {}", source.display(), error))?;
    Ok(())
}

/// Move the selected files into their category sub-folders and record the
/// batch in SQLite so it can be undone later.
///
/// Category folders are only created for categories that actually receive a
/// file, and existing destination names are never overwritten.
pub fn organize_selected(
    paths: &[String],
    target_folder: &Path,
    connection: &mut Connection,
) -> Result<OrganizeResult, String> {
    let batch_id = new_batch_id();
    let timestamp = now_millis();
    let mut records: Vec<(String, String)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for path_string in paths {
        let source = PathBuf::from(path_string);

        let name = match source.file_name() {
            Some(name) => name.to_string_lossy().into_owned(),
            None => {
                errors.push(format!("'{}' is not a file path.", path_string));
                continue;
            }
        };

        if !source.is_file() {
            errors.push(format!("Source file no longer exists: {}", source.display()));
            continue;
        }

        let category = categorize(&extension_of(&name));
        let category_folder = target_folder.join(category.folder_name());
        if !category_folder.exists() {
            if let Err(error) = fs::create_dir_all(&category_folder) {
                errors.push(format!(
                    "Unable to create '{}': {}",
                    category_folder.display(),
                    error
                ));
                continue;
            }
        }

        // The folder now exists, so conflict resolution can inspect it.
        let destination = unique_path(&category_folder, &name);
        if let Err(error) = safe_move(&source, &destination) {
            errors.push(format!("Unable to move '{}': {}", name, error));
            continue;
        }

        records.push((
            path_string.clone(),
            destination.to_string_lossy().into_owned(),
        ));
    }

    if !records.is_empty() {
        if let Err(error) = db::insert_batch(connection, &batch_id, timestamp, &records) {
            errors.push(format!("Files were moved but undo history failed: {}", error));
        }
    }

    Ok(OrganizeResult {
        batch_id,
        moved: records.len(),
        errors,
    })
}

/// Organize one file discovered by the background watcher. It gets its own
/// undo batch so "Undo Last Sort" restores background moves exactly like
/// manual ones.
pub fn organize_single(
    path: &Path,
    target_folder: &Path,
    connection: &mut Connection,
) -> Result<(), String> {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| format!("Invalid file path: {}", path.display()))?;

    if !path.is_file() {
        return Err(format!("File no longer exists: {}", path.display()));
    }

    let category = categorize(&extension_of(&name));
    let category_folder = target_folder.join(category.folder_name());
    if !category_folder.exists() {
        fs::create_dir_all(&category_folder)
            .map_err(|error| format!("Unable to create '{}': {}", category_folder.display(), error))?;
    }

    let destination = unique_path(&category_folder, &name);
    safe_move(path, &destination)?;

    db::insert_batch(
        connection,
        &new_batch_id(),
        now_millis(),
        &[(
            path.to_string_lossy().into_owned(),
            destination.to_string_lossy().into_owned(),
        )],
    )
}

/// Restore the most recent batch: move every file back to its original path,
/// remove now-empty category folders, and drop the batch from history.
pub fn undo_last_batch(connection: &mut Connection) -> Result<UndoResult, String> {
    let batch = match db::latest_batch(connection)? {
        Some(batch) => batch,
        None => {
            return Ok(UndoResult {
                restored: 0,
                errors: vec!["There is no sort to undo yet.".to_string()],
            })
        }
    };

    let moves = db::moves_for_batch(connection, &batch.batch_id)?;
    let mut restored = 0usize;
    let mut errors: Vec<String> = Vec::new();
    let mut touched_folders: Vec<PathBuf> = Vec::new();

    for record in moves {
        let moved_file = PathBuf::from(&record.dest_path);
        let original = PathBuf::from(&record.source_path);

        if let Some(parent) = moved_file.parent() {
            touched_folders.push(parent.to_path_buf());
        }

        if !moved_file.is_file() {
            errors.push(format!("Moved file is missing: {}", moved_file.display()));
            continue;
        }

        // If the original path was refilled in the meantime, keep both files.
        let original_name = match original.file_name() {
            Some(name) => name.to_string_lossy().into_owned(),
            None => {
                errors.push(format!("Invalid original path: {}", record.source_path));
                continue;
            }
        };
        let destination = match original.parent() {
            Some(parent) => {
                if !parent.exists() && fs::create_dir_all(parent).is_err() {
                    errors.push(format!("Unable to recreate '{}'", parent.display()));
                    continue;
                }
                // No conflict → this is exactly the original path.
                unique_path(parent, &original_name)
            }
            None => original.clone(),
        };

        if let Err(error) = safe_move(&moved_file, &destination) {
            errors.push(format!("Unable to restore '{}': {}", record.dest_path, error));
            continue;
        }
        restored += 1;
    }

    // Delete category folders only when they are empty after the undo.
    for folder in touched_folders {
        if let Ok(mut entries) = fs::read_dir(&folder) {
            if entries.next().is_none() {
                let _ = fs::remove_dir(&folder);
            }
        }
    }

    db::delete_batch(connection, &batch.batch_id)?;

    Ok(UndoResult { restored, errors })
}

/// Count of files in the most recent batch, for the Undo button label.
pub fn last_batch_info(connection: &Connection) -> Result<Option<BatchInfo>, String> {
    match db::latest_batch(connection)? {
        Some(batch) => {
            let file_count = db::count_moves(connection, &batch.batch_id)?;
            Ok(Some(BatchInfo {
                batch_id: batch.batch_id,
                file_count,
                timestamp: batch.created_at,
            }))
        }
        None => Ok(None),
    }
}
