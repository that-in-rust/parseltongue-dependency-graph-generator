//! Word-count coverage tracking: ensures every word in a file is accounted for.

use crate::entity_type_definitions::{
    CodeEntityRecord, EntityPrimaryKeyLocation, EntityTypeClassification,
};
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// compute_total_file_word_count
// ---------------------------------------------------------------------------

/// Count whitespace-separated words in the entire file content.
pub fn compute_total_file_word_count(content: &str) -> u32 {
    content.split_whitespace().count() as u32
}

// ---------------------------------------------------------------------------
// compute_whitespace_gap_entities
// ---------------------------------------------------------------------------

/// Compute the gap between total file word count and sum of entity word counts.
///
/// If there is a positive gap, create a `Whitespace` entity that accounts
/// for the difference (whitespace, blank lines, non-entity tokens, etc.).
pub fn compute_whitespace_gap_entities(
    file_path: &str,
    content: &str,
    entities: &[CodeEntityRecord],
    codebase_id: i64,
    file_hash: &str,
) -> Vec<CodeEntityRecord> {
    let total_wc = compute_total_file_word_count(content);
    let entity_wc_sum: u32 = entities.iter().map(|e| e.wc).sum();

    if total_wc <= entity_wc_sum {
        return Vec::new();
    }

    let gap = total_wc - entity_wc_sum;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let total_lines = content.lines().count().max(1) as i32;

    vec![CodeEntityRecord {
        pk: EntityPrimaryKeyLocation {
            file_path: file_path.to_string(),
            start_line: 1,
            end_line: total_lines,
        },
        entity_type: EntityTypeClassification::Whitespace,
        language: String::new(),
        name: "<whitespace>".to_string(),
        signature: String::new(),
        snippet: String::new(),
        doc_comment: None,
        wc: gap,
        file_hash: file_hash.to_string(),
        is_test: false,
        codebase_id,
        indexed_at: now,
        rustc_scope: None,
        rustc_sig: None,
        visibility: None,
        mir_calls: None,
        trait_impls: None,
    }]
}

// ---------------------------------------------------------------------------
// verify_word_count_coverage
// ---------------------------------------------------------------------------

/// Verify that the sum of entity word counts equals the total file word count.
///
/// Returns `true` if coverage is 100%.
pub fn verify_word_count_coverage(content: &str, entities: &[CodeEntityRecord]) -> bool {
    let total_wc = compute_total_file_word_count(content);
    let entity_wc_sum: u32 = entities.iter().map(|e| e.wc).sum();
    entity_wc_sum == total_wc
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

    fn make_entity(wc: u32, start_line: i32, end_line: i32) -> CodeEntityRecord {
        CodeEntityRecord {
            pk: EntityPrimaryKeyLocation {
                file_path: "test.rs".to_string(),
                start_line,
                end_line,
            },
            entity_type: EntityTypeClassification::Function,
            language: "rust".to_string(),
            name: "test_fn".to_string(),
            signature: "fn test_fn()".to_string(),
            snippet: "fn test_fn() {}".to_string(),
            doc_comment: None,
            wc,
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
    fn test_compute_total_file_word_count() {
        assert_eq!(compute_total_file_word_count("hello world"), 2);
        assert_eq!(compute_total_file_word_count(""), 0);
        assert_eq!(compute_total_file_word_count("  "), 0);
        assert_eq!(
            compute_total_file_word_count("fn add(a: i32) -> i32 { a }"),
            8
        );
    }

    #[test]
    fn test_compute_whitespace_gap_entities() {
        let content = "fn add(a: i32, b: i32) -> i32 { a + b }\n\n// extra words here";
        let total = compute_total_file_word_count(content);

        // Create an entity that accounts for only part of the words
        let entities = vec![make_entity(5, 1, 1)];

        let gaps = compute_whitespace_gap_entities("test.rs", content, &entities, 1, "hash");

        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].entity_type, EntityTypeClassification::Whitespace);
        assert_eq!(gaps[0].wc, total - 5);
    }

    #[test]
    fn test_no_gap_when_fully_covered() {
        let content = "fn add() {}";
        let total = compute_total_file_word_count(content);
        let entities = vec![make_entity(total, 1, 1)];

        let gaps = compute_whitespace_gap_entities("test.rs", content, &entities, 1, "hash");
        assert!(gaps.is_empty());
    }

    #[test]
    fn test_verify_word_count_coverage_true() {
        let content = "hello world foo";
        let entities = vec![make_entity(3, 1, 1)];
        assert!(verify_word_count_coverage(content, &entities));
    }

    #[test]
    fn test_verify_word_count_coverage_false() {
        let content = "hello world foo bar";
        let entities = vec![make_entity(2, 1, 1)];
        assert!(!verify_word_count_coverage(content, &entities));
    }

    #[test]
    fn test_word_count_coverage_with_gap_entity() {
        let source = "fn add(a: i32, b: i32) -> i32 { a + b }\nstruct Point { x: f64, y: f64 }";
        let config =
            crate::walker::language_config_registry::get_language_config("rust").unwrap();
        let mut entities =
            crate::walker::treesitter_entity_extractor::extract_entities_from_source(
                "test.rs", source, &config, 1, "hash",
            )
            .unwrap();

        // Compute gap and add whitespace entities
        let gaps =
            compute_whitespace_gap_entities("test.rs", source, &entities, 1, "hash");
        entities.extend(gaps);

        assert!(
            verify_word_count_coverage(source, &entities),
            "coverage should be 100% after adding gap entity. total={}, sum={}",
            compute_total_file_word_count(source),
            entities.iter().map(|e| e.wc).sum::<u32>()
        );
    }
}
