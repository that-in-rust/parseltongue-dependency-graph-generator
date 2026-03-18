//! Hierarchy edge builder – constructs Contains edges for the folder/file/entity
//! containment hierarchy.

use crate::entity_type_definitions::{EdgeRecord, EdgeType};
use crate::walker::directory_tree_scanner::{FolderEntityRecord, ScannedFileRecord};
use crate::entity_type_definitions::CodeEntityRecord;

// ---------------------------------------------------------------------------
// build_hierarchy_edges
// ---------------------------------------------------------------------------

/// Build [`EdgeType::Contains`] edges for the full containment hierarchy:
///
/// - folder → child folders (parent path derived from `folder_path`)
/// - folder → files (parent path derived from `file_path`)
/// - file → code entities (file_path match)
///
/// Source PKs use folder sentinels (-1:-1) for folders and file sentinels
/// (0:0) for files.
pub fn build_hierarchy_edges(
    folders: &[FolderEntityRecord],
    files: &[ScannedFileRecord],
    entities: &[CodeEntityRecord],
    codebase_id: i64,
) -> Vec<EdgeRecord> {
    let mut edges = Vec::new();
    let edge_type_str = EdgeType::Contains.to_string();

    // Build a set of folder paths for quick lookup.
    let folder_paths: std::collections::HashSet<&str> =
        folders.iter().map(|f| f.folder_path.as_str()).collect();

    // --- folder -> child folders ---
    for folder in folders {
        let parent = parent_path(&folder.folder_path);
        if folder_paths.contains(parent.as_str()) {
            edges.push(EdgeRecord {
                from_path: parent.clone(),
                from_start: -1,
                from_end: -1,
                to_path: folder.folder_path.clone(),
                to_start: -1,
                to_end: -1,
                edge_type: edge_type_str.clone(),
                codebase_id,
            });
        }
    }

    // --- folder -> files ---
    for file in files {
        let parent = parent_path(&file.file_path);
        if folder_paths.contains(parent.as_str()) {
            edges.push(EdgeRecord {
                from_path: parent.clone(),
                from_start: -1,
                from_end: -1,
                to_path: file.file_path.clone(),
                to_start: 0,
                to_end: 0,
                edge_type: edge_type_str.clone(),
                codebase_id,
            });
        }
    }

    // --- file -> code entities ---
    for entity in entities {
        edges.push(EdgeRecord {
            from_path: entity.pk.file_path.clone(),
            from_start: 0,
            from_end: 0,
            to_path: entity.pk.file_path.clone(),
            to_start: entity.pk.start_line,
            to_end: entity.pk.end_line,
            edge_type: edge_type_str.clone(),
            codebase_id,
        });
    }

    edges
}

/// Derive the parent path from a file or folder path.
///
/// Examples:
/// - `"src/utils/helper.rs"` → `"src/utils"`
/// - `"src/utils"` → `"src"`
/// - `"src"` → `""` (root)
fn parent_path(path: &str) -> String {
    match path.rfind('/') {
        Some(idx) => path[..idx].to_string(),
        None => String::new(), // root
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity_type_definitions::{
        EntityPrimaryKeyLocation, EntityTypeClassification,
    };

    /// Helper to create a minimal CodeEntityRecord for testing.
    fn make_entity(name: &str, file_path: &str, start: i32, end: i32) -> CodeEntityRecord {
        CodeEntityRecord {
            pk: EntityPrimaryKeyLocation {
                file_path: file_path.to_string(),
                start_line: start,
                end_line: end,
            },
            entity_type: EntityTypeClassification::Function,
            language: "rust".to_string(),
            name: name.to_string(),
            signature: String::new(),
            snippet: String::new(),
            doc_comment: None,
            wc: 0,
            file_hash: String::new(),
            is_test: false,
            codebase_id: 1,
            indexed_at: 0,
            rustc_scope: None,
            rustc_sig: None,
            visibility: None,
            mir_calls: None,
            trait_impls: None,
        }
    }

    // ------------------------------------------------------------------
    // Test 4: Hierarchy edges
    // ------------------------------------------------------------------
    #[test]
    fn test_hierarchy_edges() {
        let folders = vec![
            FolderEntityRecord {
                folder_path: "src".to_string(),
            },
            FolderEntityRecord {
                folder_path: "src/utils".to_string(),
            },
        ];

        let files = vec![
            ScannedFileRecord {
                file_path: "src/main.rs".to_string(),
                file_hash: "h1".to_string(),
                size_bytes: 100,
                language: Some("rust".to_string()),
            },
            ScannedFileRecord {
                file_path: "src/utils/helper.rs".to_string(),
                file_hash: "h2".to_string(),
                size_bytes: 200,
                language: Some("rust".to_string()),
            },
        ];

        let entities = vec![
            make_entity("main", "src/main.rs", 1, 10),
            make_entity("add", "src/utils/helper.rs", 1, 5),
            make_entity("sub", "src/utils/helper.rs", 7, 12),
        ];

        let edges = build_hierarchy_edges(&folders, &files, &entities, 1);

        // All edges should be Contains.
        for edge in &edges {
            assert_eq!(edge.edge_type, "contains");
        }

        // src -> src/utils (folder -> child folder)
        let folder_to_folder: Vec<_> = edges
            .iter()
            .filter(|e| {
                e.from_path == "src"
                    && e.from_start == -1
                    && e.to_path == "src/utils"
                    && e.to_start == -1
            })
            .collect();
        assert_eq!(
            folder_to_folder.len(),
            1,
            "expected src -> src/utils folder edge"
        );

        // src -> src/main.rs (folder -> file)
        let folder_to_file: Vec<_> = edges
            .iter()
            .filter(|e| {
                e.from_path == "src"
                    && e.from_start == -1
                    && e.to_path == "src/main.rs"
                    && e.to_start == 0
            })
            .collect();
        assert_eq!(
            folder_to_file.len(),
            1,
            "expected src -> src/main.rs file edge"
        );

        // src/utils -> src/utils/helper.rs (folder -> file)
        let utils_to_helper: Vec<_> = edges
            .iter()
            .filter(|e| {
                e.from_path == "src/utils"
                    && e.from_start == -1
                    && e.to_path == "src/utils/helper.rs"
                    && e.to_start == 0
            })
            .collect();
        assert_eq!(
            utils_to_helper.len(),
            1,
            "expected src/utils -> src/utils/helper.rs file edge"
        );

        // file -> entities
        let file_to_entity: Vec<_> = edges
            .iter()
            .filter(|e| e.from_start == 0 && e.from_end == 0 && e.to_start > 0)
            .collect();
        assert_eq!(
            file_to_entity.len(),
            3,
            "expected 3 file -> entity edges, got {}",
            file_to_entity.len()
        );

        // Total: 1 (folder->folder) + 2 (folder->files) + 3 (file->entities) = 6
        assert_eq!(
            edges.len(),
            6,
            "expected 6 total hierarchy edges, got {}",
            edges.len()
        );
    }

    #[test]
    fn test_parent_path_nested() {
        assert_eq!(parent_path("src/utils/helper.rs"), "src/utils");
        assert_eq!(parent_path("src/utils"), "src");
        assert_eq!(parent_path("src"), "");
    }
}
