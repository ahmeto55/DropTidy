use serde::{Deserialize, Serialize};

/// The bucket a file is sorted into. Kept in sync with the `Category` type on
/// the TypeScript side (serde serialises unit variants as lowerCamelCase).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Category {
    Installers,
    Archives,
    Documents,
    Images,
    Media,
    Code,
    Other,
}

impl Category {
    /// Name of the sub-folder created inside the target folder for this bucket.
    pub fn folder_name(self) -> &'static str {
        match self {
            Category::Installers => "Installers",
            Category::Archives => "Archives",
            Category::Documents => "Documents",
            Category::Images => "Images",
            Category::Media => "Media",
            Category::Code => "Code",
            Category::Other => "Other",
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.folder_name())
    }
}

/// A single top-level file discovered during a scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    /// Absolute path of the source file.
    pub path: String,
    /// File name including extension.
    pub name: String,
    /// Lower-cased extension without the leading dot (empty when absent).
    pub extension: String,
    pub category: Category,
    /// Size in bytes.
    pub size: u64,
    /// Last-modified time as milliseconds since the Unix epoch.
    pub modified: i64,
    /// True when an identical (same-size, same SHA-256) file also exists.
    pub is_duplicate: bool,
    /// Identifier shared by every member of a duplicate group.
    pub duplicate_group: Option<String>,
    /// Where the file would land if organised (preview only).
    pub destination: String,
}

/// Result of a folder scan, ready for the preview table.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub target_folder: String,
    pub files: Vec<FileEntry>,
    pub total_size: u64,
    /// Number of individual files that belong to a duplicate group.
    pub duplicate_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizeResult {
    pub batch_id: String,
    pub moved: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoResult {
    pub restored: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchInfo {
    pub batch_id: String,
    pub file_count: usize,
    pub timestamp: i64,
}

/// Auto-organize preferences persisted in SQLite.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub auto_organize: bool,
    pub watch_delay_secs: f64,
}
