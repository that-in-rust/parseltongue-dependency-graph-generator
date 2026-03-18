use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use walkdir::WalkDir;

use super::language_config_registry::detect_language_from_extension;

// ---------------------------------------------------------------------------
// ScannedFileRecord
// ---------------------------------------------------------------------------

/// A record representing a scanned file in the directory tree.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScannedFileRecord {
    /// File path relative to the scan root.
    pub file_path: String,
    /// SHA-256 hex digest of the file contents.
    pub file_hash: String,
    /// Size of the file in bytes.
    pub size_bytes: u64,
    /// Detected programming language (None if unknown).
    pub language: Option<String>,
}

// ---------------------------------------------------------------------------
// FolderEntityRecord
// ---------------------------------------------------------------------------

/// A record representing a discovered folder in the directory tree.
#[derive(Clone, Debug)]
pub struct FolderEntityRecord {
    /// Folder path relative to the scan root.
    pub folder_path: String,
}

// ---------------------------------------------------------------------------
// Always-ignore directory names
// ---------------------------------------------------------------------------

/// Directory names that are always skipped during scanning.
const ALWAYS_IGNORE_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "build",
    "dist",
    "__pycache__",
    ".venv",
    ".cargo",
    ".rustup",
    ".idea",
    ".vscode",
];

/// Returns `true` if the directory name matches a bazel output directory
/// pattern (`bazel-*`).
fn is_bazel_output_dir(name: &str) -> bool {
    name.starts_with("bazel-")
}

/// Returns `true` if the given file/directory name should be ignored.
fn should_ignore_dir(name: &str) -> bool {
    ALWAYS_IGNORE_DIRS.contains(&name) || is_bazel_output_dir(name)
}

/// Returns `true` if the name starts with `.` (hidden file/dir).
fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

// ---------------------------------------------------------------------------
// Maximum file size (1 MB)
// ---------------------------------------------------------------------------

const MAX_FILE_SIZE: u64 = 1_048_576; // 1 MB

// ---------------------------------------------------------------------------
// scan_directory_tree_recursive
// ---------------------------------------------------------------------------

/// Recursively scans a directory tree, returning discovered files and folders.
///
/// - Skips hidden files/directories (names starting with `.`), except the root itself.
/// - Skips directories in the [`ALWAYS_IGNORE_DIRS`] list and `bazel-*` patterns.
/// - Skips empty files (0 bytes) and files larger than 1 MB.
/// - All paths are stored relative to `root`.
pub fn scan_directory_tree_recursive(
    root: &Path,
) -> Result<(Vec<ScannedFileRecord>, Vec<FolderEntityRecord>)> {
    let mut files = Vec::new();
    let mut folders = Vec::new();

    let walker = WalkDir::new(root).follow_links(false);

    for entry in walker {
        let entry = entry?;
        let path = entry.path();

        // Compute the relative path from root.
        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Skip the root entry itself (rel_path is empty).
        if rel_path.is_empty() {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy();

        // --- Directory handling ---
        if entry.file_type().is_dir() {
            // Skip hidden directories and always-ignored directories.
            if is_hidden(&file_name) || should_ignore_dir(&file_name) {
                // This is a WalkDir filter_entry equivalent via skip:
                // We can't easily prune with the iterator, so we skip
                // and rely on prefix checking below.
                continue;
            }

            // Check that no ancestor component is hidden or ignored.
            if has_ignored_ancestor(&rel_path) {
                continue;
            }

            folders.push(FolderEntityRecord {
                folder_path: rel_path,
            });
            continue;
        }

        // --- File handling ---
        if entry.file_type().is_file() {
            // Skip hidden files.
            if is_hidden(&file_name) {
                continue;
            }

            // Check that no ancestor component is hidden or ignored.
            if has_ignored_ancestor(&rel_path) {
                continue;
            }

            // Check file size.
            let metadata = entry.metadata()?;
            let size = metadata.len();
            if size == 0 || size > MAX_FILE_SIZE {
                continue;
            }

            // Compute SHA-256 hash.
            let file_hash = compute_file_sha256_hash(path)?;

            // Detect language from extension.
            let language = path
                .extension()
                .and_then(|ext| ext.to_str())
                .and_then(|ext| detect_language_from_extension(ext));

            files.push(ScannedFileRecord {
                file_path: rel_path,
                file_hash,
                size_bytes: size,
                language,
            });
        }
    }

    Ok((files, folders))
}

/// Returns `true` if any path component is hidden or in the ignore list.
fn has_ignored_ancestor(rel_path: &str) -> bool {
    for component in rel_path.split('/') {
        if component.is_empty() {
            continue;
        }
        if is_hidden(component) || should_ignore_dir(component) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// compute_file_sha256_hash
// ---------------------------------------------------------------------------

/// Reads the file at `path` and returns its SHA-256 digest as a lowercase hex string.
pub fn compute_file_sha256_hash(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let result = hasher.finalize();
    Ok(hex_encode(&result))
}

/// Encode bytes as lowercase hex string.
fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        s.push_str(&format!("{byte:02x}"));
    }
    s
}
