//! Doc-comment folding logic: merge doc-comments into following code entities.

use crate::entity_type_definitions::{CodeEntityRecord, EntityTypeClassification};

// ---------------------------------------------------------------------------
// Doc-comment detection helpers
// ---------------------------------------------------------------------------

/// Returns `true` if the given entity looks like a documentation comment.
fn is_doc_comment(entity: &CodeEntityRecord) -> bool {
    if entity.entity_type != EntityTypeClassification::Comment
        && entity.entity_type != EntityTypeClassification::DocComment
    {
        return false;
    }

    let text = entity.snippet.trim();

    match entity.language.as_str() {
        "rust" => {
            text.starts_with("///")
                || text.starts_with("//!")
                || text.starts_with("/**")
                || text.starts_with("/*!")
        }
        "python" => {
            // Triple-quoted strings used as docstrings
            text.starts_with("\"\"\"") || text.starts_with("'''")
        }
        _ => {
            // JSDoc / Javadoc / general /** */ style
            text.starts_with("/**")
        }
    }
}

/// Returns `true` if the given entity is a module-level doc comment
/// (e.g. Rust `//!` or `/*! */`).
fn is_module_doc(entity: &CodeEntityRecord) -> bool {
    if entity.entity_type != EntityTypeClassification::Comment
        && entity.entity_type != EntityTypeClassification::DocComment
    {
        return false;
    }

    let text = entity.snippet.trim();
    text.starts_with("//!") || text.starts_with("/*!")
}

// ---------------------------------------------------------------------------
// fold_doc_comments_into_entities
// ---------------------------------------------------------------------------

/// Fold doc-comments into their associated code entities.
///
/// Walk entities in order by `start_line`. When a comment looks like a doc
/// comment, merge its text into the **next** non-comment entity's
/// `doc_comment` field, then remove the comment entity from the list.
///
/// Module docs (`//!`, `/*! */`) are folded into any entity with
/// `entity_type == FileParsable` if present.
pub fn fold_doc_comments_into_entities(entities: &mut Vec<CodeEntityRecord>) {
    // Sort by start_line for correct ordering
    entities.sort_by_key(|e| e.pk.start_line);

    // First pass: collect indices of doc comments and what they fold into
    let mut removals: Vec<usize> = Vec::new();
    let mut pending_doc: Option<(usize, String)> = None; // (index, text)

    for i in 0..entities.len() {
        let is_doc = is_doc_comment(&entities[i]);
        let is_mod = is_module_doc(&entities[i]);

        if is_mod {
            // Module docs fold into file entity
            let doc_text = entities[i].snippet.clone();
            removals.push(i);
            // Find a FileParsable entity and attach
            for j in 0..entities.len() {
                if entities[j].entity_type == EntityTypeClassification::FileParsable {
                    let existing = entities[j].doc_comment.take().unwrap_or_default();
                    let merged = if existing.is_empty() {
                        doc_text.clone()
                    } else {
                        format!("{existing}\n{doc_text}")
                    };
                    entities[j].doc_comment = Some(merged);
                    break;
                }
            }
            continue;
        }

        if is_doc {
            // Flush any prior pending doc into this entity if it's also a comment,
            // or hold onto it.
            if let Some((prev_idx, prev_text)) = pending_doc.take() {
                // Previous doc had no code entity to attach to before another doc.
                // Combine them.
                let combined = format!("{prev_text}\n{}", entities[i].snippet);
                removals.push(prev_idx);
                pending_doc = Some((i, combined));
            } else {
                pending_doc = Some((i, entities[i].snippet.clone()));
            }
            continue;
        }

        // This is a code entity. Attach any pending doc.
        if let Some((doc_idx, doc_text)) = pending_doc.take() {
            entities[i].doc_comment = Some(doc_text);
            removals.push(doc_idx);
        }
    }

    // Any remaining pending doc has no code entity after it; still remove it
    if let Some((doc_idx, _)) = pending_doc {
        removals.push(doc_idx);
    }

    // Remove folded doc comments (in reverse order to preserve indices)
    removals.sort_unstable();
    removals.dedup();
    for idx in removals.into_iter().rev() {
        entities.remove(idx);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity_type_definitions::{
        CodeEntityRecord, EntityPrimaryKeyLocation, EntityTypeClassification,
    };

    fn make_entity(
        entity_type: EntityTypeClassification,
        name: &str,
        snippet: &str,
        start_line: i32,
        end_line: i32,
    ) -> CodeEntityRecord {
        CodeEntityRecord {
            pk: EntityPrimaryKeyLocation {
                file_path: "test.rs".to_string(),
                start_line,
                end_line,
            },
            entity_type,
            language: "rust".to_string(),
            name: name.to_string(),
            signature: snippet.lines().next().unwrap_or("").to_string(),
            snippet: snippet.to_string(),
            doc_comment: None,
            wc: snippet.split_whitespace().count() as u32,
            file_hash: "hash".to_string(),
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

    #[test]
    fn test_fold_doc_comment_into_function() {
        let mut entities = vec![
            make_entity(
                EntityTypeClassification::Comment,
                "",
                "/// Adds two numbers",
                1,
                1,
            ),
            make_entity(
                EntityTypeClassification::Function,
                "add",
                "fn add(a: i32, b: i32) -> i32 { a + b }",
                2,
                2,
            ),
        ];

        fold_doc_comments_into_entities(&mut entities);

        assert_eq!(entities.len(), 1, "doc comment should be removed");
        assert_eq!(entities[0].name, "add");
        assert_eq!(
            entities[0].doc_comment.as_deref(),
            Some("/// Adds two numbers")
        );
    }

    #[test]
    fn test_non_doc_comment_not_folded() {
        let mut entities = vec![
            make_entity(
                EntityTypeClassification::Comment,
                "",
                "// just a regular comment",
                1,
                1,
            ),
            make_entity(
                EntityTypeClassification::Function,
                "foo",
                "fn foo() {}",
                2,
                2,
            ),
        ];

        fold_doc_comments_into_entities(&mut entities);

        // Regular comment is NOT a doc comment, should remain
        assert_eq!(entities.len(), 2);
        assert!(entities[1].doc_comment.is_none());
    }

    #[test]
    fn test_multiple_doc_comments_merged() {
        let mut entities = vec![
            make_entity(
                EntityTypeClassification::Comment,
                "",
                "/// Line one",
                1,
                1,
            ),
            make_entity(
                EntityTypeClassification::Comment,
                "",
                "/// Line two",
                2,
                2,
            ),
            make_entity(
                EntityTypeClassification::Function,
                "bar",
                "fn bar() {}",
                3,
                3,
            ),
        ];

        fold_doc_comments_into_entities(&mut entities);

        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].name, "bar");
        let doc = entities[0].doc_comment.as_ref().unwrap();
        assert!(doc.contains("Line one"), "doc: {doc}");
        assert!(doc.contains("Line two"), "doc: {doc}");
    }
}
