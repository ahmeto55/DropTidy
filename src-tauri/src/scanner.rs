use crate::models::{Category, FileEntry, ScanResult};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

/// Extensions used by browsers/ download managers for in-progress downloads.
/// These must never be moved while the download is still active.
const IN_PROGRESS_EXTENSIONS: [&str; 4] = ["crdownload", "part", "tmp", "opdownload"];

/// Bytes sampled for the cheap first pass of duplicate detection.
const PARTIAL_HASH_BYTES: usize = 4096;

/// Platform-appropriate Downloads directory for the current user.
pub fn default_downloads_dir() -> Option<PathBuf> {
    dirs::download_dir()
}

/// Map a (lowercase) extension to its category bucket.
pub fn categorize(extension: &str) -> Category {
    match extension.to_ascii_lowercase().as_str() {
        "exe" | "msi" | "msix" | "bat" | "cmd" => Category::Installers,
        "zip" | "7z" | "rar" | "tar" | "gz" | "iso" => Category::Archives,
        "pdf" | "docx" | "doc" | "xlsx" | "xls" | "pptx" | "txt" | "md" | "csv" => {
            Category::Documents
        }
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg" | "ico" => Category::Images,
        "mp4" | "mkv" | "mov" | "avi" | "mp3" | "wav" | "flac" => Category::Media,
        "py" | "rs" | "ts" | "tsx" | "js" | "json" | "html" | "css" | "sql" | "sh" | "ps1" => {
            Category::Code
        }
        _ => Category::Other,
    }
}

/// Lower-cased extension of a file name, without the leading dot.
/// Dot-files such as `.gitignore` are treated as having no extension.
pub fn extension_of(name: &str) -> String {
    match name.rfind('.') {
        Some(index) if index > 0 && index + 1 < name.len() => {
            name[index + 1..].to_ascii_lowercase()
        }
        _ => String::new(),
    }
}

/// Files that must never be touched: OS metadata and in-progress downloads.
/// Used by both the manual scanner and the background watcher.
pub fn is_ignored_file_name(name: &str) -> bool {
    if name.eq_ignore_ascii_case("desktop.ini") {
        return true;
    }
    let extension = extension_of(name);
    IN_PROGRESS_EXTENSIONS.contains(&extension.as_str())
}

fn modified_millis(metadata: &fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

/// SHA-256 of a file. When `limit` is set, only that many leading bytes are
/// hashed — used as a cheap pre-filter before paying for a full hash.
fn hash_file(path: &Path, limit: Option<usize>) -> std::io::Result<String> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut remaining = limit.unwrap_or(usize::MAX);

    while remaining > 0 {
        let wanted = buffer.len().min(remaining);
        let read = reader.read(&mut buffer[..wanted])?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        remaining -= read;
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Split a file name into stem and (original-case) extension.
fn split_name(name: &str) -> (&str, Option<&str>) {
    match name.rfind('.') {
        Some(index) if index > 0 && index + 1 < name.len() => {
            (&name[..index], Some(&name[index + 1..]))
        }
        _ => (name, None),
    }
}

/// Return `dir/name`, appending ` (1)`, ` (2)`, ... when the path is occupied.
/// The original extension casing is preserved.
pub fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }

    let (stem, extension) = split_name(name);
    for sequence in 1..10_000 {
        let candidate_name = match extension {
            Some(ext) => format!("{} ({}).{}", stem, sequence, ext),
            None => format!("{} ({})", stem, sequence),
        };
        let candidate = dir.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    // Extremely unlikely: every numbered variant is taken. Avoid overwriting.
    dir.join(format!("{} (unresolved)", stem))
}

/// Preview destination for a file, e.g. `Downloads\Documents\report.pdf`.
/// Conflicts with already-existing files are resolved without touching them.
fn preview_destination(folder: &Path, name: &str, category: Category) -> PathBuf {
    unique_path(&folder.join(category.folder_name()), name)
}

/// Scan only the top level of `folder`. Directories, hidden OS files and
/// in-progress browser downloads are ignored.
pub fn scan_folder(folder: &Path) -> Result<ScanResult, String> {
    let entries = fs::read_dir(folder)
        .map_err(|error| format!("Unable to open '{}': {}", folder.display(), error))?;

    // (path, name, size, modified millis)
    let mut files: Vec<(PathBuf, String, u64, i64)> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|error| format!("Unable to read an entry: {}", error))?;
        if !entry
            .file_type()
            .map_err(|error| format!("Unable to inspect '{}': {}", entry.path().display(), error))?
            .is_file()
        {
            continue; // Never descend into category folders or any sub-folder.
        }

        let name = entry.file_name().to_string_lossy().into_owned();
        if is_ignored_file_name(&name) {
            continue;
        }

        let metadata = entry
            .metadata()
            .map_err(|error| format!("Unable to read metadata for '{}': {}", name, error))?;
        files.push((
            entry.path(),
            name,
            metadata.len(),
            modified_millis(&metadata),
        ));
    }

    // Duplicate detection: only files of identical size can be identical, so
    // hash cheap prefixes first and only hash full contents for prefix matches.
    let mut duplicate_groups: HashMap<usize, String> = HashMap::new();
    let mut by_size: HashMap<u64, Vec<usize>> = HashMap::new();
    for (index, item) in files.iter().enumerate() {
        by_size.entry(item.2).or_default().push(index);
    }

    for same_size in by_size.values() {
        if same_size.len() < 2 {
            continue;
        }

        let mut by_prefix: HashMap<String, Vec<usize>> = HashMap::new();
        for &index in same_size {
            if let Ok(hash) = hash_file(&files[index].0, Some(PARTIAL_HASH_BYTES)) {
                by_prefix.entry(hash).or_default().push(index);
            }
        }

        for same_prefix in by_prefix.values() {
            if same_prefix.len() < 2 {
                continue;
            }

            let mut by_full_hash: HashMap<String, Vec<usize>> = HashMap::new();
            for &index in same_prefix {
                if let Ok(hash) = hash_file(&files[index].0, None) {
                    by_full_hash.entry(hash).or_default().push(index);
                }
            }

            for (hash, group) in &by_full_hash {
                if group.len() < 2 {
                    continue;
                }
                let group_id = hash.chars().take(16).collect::<String>();
                for &index in group {
                    duplicate_groups.insert(index, group_id.clone());
                }
            }
        }
    }

    let mut result_files: Vec<FileEntry> = Vec::with_capacity(files.len());
    let mut total_size = 0u64;
    let mut duplicate_count = 0usize;

    for (index, (path, name, size, modified)) in files.into_iter().enumerate() {
        let extension = extension_of(&name);
        let category = categorize(&extension);
        let destination = preview_destination(folder, &name, category)
            .to_string_lossy()
            .into_owned();
        let duplicate_group = duplicate_groups.get(&index).cloned();
        let is_duplicate = duplicate_group.is_some();
        if is_duplicate {
            duplicate_count += 1;
        }
        total_size = total_size.saturating_add(size);

        result_files.push(FileEntry {
            path: path.to_string_lossy().into_owned(),
            name,
            extension,
            category,
            size,
            modified,
            is_duplicate,
            duplicate_group,
            destination,
        });
    }

    result_files.sort_by(|a, b| {
        a.category
            .to_string()
            .cmp(&b.category.to_string())
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(ScanResult {
        target_folder: folder.to_string_lossy().into_owned(),
        files: result_files,
        total_size,
        duplicate_count,
    })
}
