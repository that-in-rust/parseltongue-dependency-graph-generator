//! Import resolution mapper – heuristic matching of import paths to known entities.

use crate::entity_type_definitions::{CodeEntityRecord, EntityPrimaryKeyLocation};

// ---------------------------------------------------------------------------
// resolve_import_to_entity
// ---------------------------------------------------------------------------

/// Attempt to resolve an import path to a known entity using heuristic matching.
///
/// 1. Split `import_path` on `::` (Rust) or `.` (Python/JS).
/// 2. Take the last segment as the entity name.
/// 3. Search `known_entities` for a matching name.
/// 4. If multiple matches, prefer same-language match (unspecified here, first wins).
/// 5. If not found, return `None` (caller creates unresolved edge).
pub fn resolve_import_to_entity(
    import_path: &str,
    known_entities: &[CodeEntityRecord],
) -> Option<EntityPrimaryKeyLocation> {
    let entity_name = extract_last_segment(import_path);
    if entity_name.is_empty() {
        return None;
    }

    // Search for matching entity name.
    let matches: Vec<&CodeEntityRecord> = known_entities
        .iter()
        .filter(|e| e.name == entity_name)
        .collect();

    match matches.len() {
        0 => None,
        1 => Some(matches[0].pk.clone()),
        _ => {
            // Multiple matches: return the first one (best-effort).
            // A more sophisticated resolver could prefer same-language matches.
            Some(matches[0].pk.clone())
        }
    }
}

/// Extract the last segment of an import path.
///
/// Splits on `::` first (Rust-style), then `.` (Python/JS-style).
/// If neither separator is found, returns the entire path.
fn extract_last_segment(path: &str) -> &str {
    // Try Rust-style `::` separator first.
    if path.contains("::") {
        return path.rsplit("::").next().unwrap_or(path).trim();
    }
    // Try Python/JS-style `.` separator.
    if path.contains('.') {
        return path.rsplit('.').next().unwrap_or(path).trim();
    }
    path.trim()
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
            entity_type: EntityTypeClassification::Struct,
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
    // Test 5: Import resolution
    // ------------------------------------------------------------------
    #[test]
    fn test_resolve_import_found() {
        let entities = vec![
            make_entity("HashMap", "src/collections.rs", 10, 50),
            make_entity("Vec", "src/alloc.rs", 1, 100),
        ];

        let result = resolve_import_to_entity("crate::collections::HashMap", &entities);
        assert!(result.is_some(), "expected to resolve HashMap");
        let pk = result.unwrap();
        assert_eq!(pk.file_path, "src/collections.rs");
        assert_eq!(pk.start_line, 10);
        assert_eq!(pk.end_line, 50);
    }

    #[test]
    fn test_resolve_import_not_found() {
        let entities = vec![make_entity("HashMap", "src/collections.rs", 10, 50)];

        let result = resolve_import_to_entity("nonexistent::Foo", &entities);
        assert!(result.is_none(), "expected None for nonexistent import");
    }

    #[test]
    fn test_resolve_python_style() {
        let entities = vec![make_entity("path", "stdlib/os.py", 1, 20)];

        let result = resolve_import_to_entity("os.path", &entities);
        assert!(result.is_some(), "expected to resolve os.path");
        assert_eq!(result.unwrap().file_path, "stdlib/os.py");
    }

    #[test]
    fn test_extract_last_segment_rust() {
        assert_eq!(extract_last_segment("std::collections::HashMap"), "HashMap");
    }

    #[test]
    fn test_extract_last_segment_python() {
        assert_eq!(extract_last_segment("os.path"), "path");
    }

    #[test]
    fn test_extract_last_segment_simple() {
        assert_eq!(extract_last_segment("HashMap"), "HashMap");
    }
}
