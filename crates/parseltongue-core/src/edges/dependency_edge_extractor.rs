//! Dependency edge extraction from source code using tree-sitter.
//!
//! For each code entity, walks its AST subtree looking for call expressions,
//! imports, trait implementations, type references, and field accesses.

use crate::entity_type_definitions::{CodeEntityRecord, EdgeRecord, EdgeType};
use crate::walker::language_config_registry::LanguageConfigDefinition;
use anyhow::{Context, Result};
use std::collections::HashSet;
use tree_sitter::{Node, Parser};

// ---------------------------------------------------------------------------
// Edge key for deduplication
// ---------------------------------------------------------------------------

/// Deduplication key: (from_path, from_start, from_end, to_path, edge_type).
type EdgeKey = (String, i32, i32, String, String);

// ---------------------------------------------------------------------------
// extract_edges_from_source
// ---------------------------------------------------------------------------

/// Extract dependency edges from a source file for the given code entities.
///
/// Re-parses the source with tree-sitter, then for each entity finds its AST
/// node by line range and walks descendants to detect:
/// - `call_expression` / `method_call_expression` → [`EdgeType::Calls`]
/// - `use_declaration` / `import_statement` / `import_from_statement` → [`EdgeType::Imports`]
/// - `impl_item` with trait → [`EdgeType::Implements`]
/// - Type identifiers in signatures → [`EdgeType::TypeRefs`]
/// - `field_expression` / `field_access` → [`EdgeType::FieldAccess`]
///
/// Edges are deduplicated: if the same (from, to, edge_type) appears multiple
/// times within one entity, only one edge is kept.
pub fn extract_edges_from_source(
    file_path: &str,
    content: &str,
    config: &LanguageConfigDefinition,
    entities: &[CodeEntityRecord],
    codebase_id: i64,
) -> Result<Vec<EdgeRecord>> {
    let mut parser = Parser::new();
    let lang = (config.tree_sitter_language)();
    parser
        .set_language(&lang)
        .context("Failed to set tree-sitter language")?;

    let tree = parser
        .parse(content, None)
        .context("Tree-sitter failed to parse content")?;

    let root = tree.root_node();
    let lang_name = config.name.as_str();

    let mut all_edges = Vec::new();

    for entity in entities {
        // Only process entities from this file.
        if entity.pk.file_path != file_path {
            continue;
        }

        // Find the AST node corresponding to this entity by line range.
        let entity_start_row = (entity.pk.start_line - 1) as usize; // 0-based
        let entity_end_row = (entity.pk.end_line - 1) as usize;

        if let Some(ast_node) = find_node_by_line_range(&root, entity_start_row, entity_end_row) {
            let mut seen: HashSet<EdgeKey> = HashSet::new();
            let mut entity_edges = Vec::new();

            walk_for_edges(
                &ast_node,
                content,
                lang_name,
                file_path,
                entity,
                codebase_id,
                &mut entity_edges,
                &mut seen,
            );

            all_edges.extend(entity_edges);
        }
    }

    Ok(all_edges)
}

// ---------------------------------------------------------------------------
// find_node_by_line_range
// ---------------------------------------------------------------------------

/// Find a tree-sitter node whose line range matches (start_row, end_row) (0-based).
///
/// Walks the tree looking for the widest (most bytes) node that spans exactly
/// the given line range. This ensures we get the full entity node (e.g.
/// `function_item`) rather than a leaf that happens to share the same lines.
fn find_node_by_line_range<'a>(root: &Node<'a>, start_row: usize, end_row: usize) -> Option<Node<'a>> {
    let mut best: Option<Node<'a>> = None;
    let mut best_size: usize = 0;

    fn search<'b>(
        node: &Node<'b>,
        start_row: usize,
        end_row: usize,
        best: &mut Option<Node<'b>>,
        best_size: &mut usize,
    ) {
        let n_start = node.start_position().row;
        let n_end = node.end_position().row;

        if n_start == start_row && n_end == end_row {
            let size = node.end_byte() - node.start_byte();
            if size > *best_size {
                *best = Some(*node);
                *best_size = size;
            }
        }

        // Only descend if the node contains the target range.
        if n_start <= start_row && n_end >= end_row {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                search(&child, start_row, end_row, best, best_size);
            }
        }
    }

    search(root, start_row, end_row, &mut best, &mut best_size);
    best
}

// ---------------------------------------------------------------------------
// walk_for_edges
// ---------------------------------------------------------------------------

/// Recursively walk descendants of `node` and emit edges.
fn walk_for_edges(
    node: &Node,
    content: &str,
    lang: &str,
    file_path: &str,
    entity: &CodeEntityRecord,
    codebase_id: i64,
    edges: &mut Vec<EdgeRecord>,
    seen: &mut HashSet<EdgeKey>,
) {
    let kind = node.kind();

    match kind {
        // --- Calls ---
        "call_expression" | "call" => {
            if let Some(target_name) = extract_call_target(node, content, lang) {
                maybe_add_edge(
                    edges,
                    seen,
                    entity,
                    &target_name,
                    EdgeType::Calls,
                    codebase_id,
                );
            }
        }
        "method_call_expression" => {
            // Rust: node.child_by_field_name("name") gives method name
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = content[name_node.byte_range()].to_string();
                maybe_add_edge(edges, seen, entity, &name, EdgeType::Calls, codebase_id);
            }
        }

        // --- Imports ---
        "use_declaration" | "import_statement" | "import_from_statement" => {
            if let Some(import_name) = extract_import_target(node, content, lang) {
                maybe_add_edge(
                    edges,
                    seen,
                    entity,
                    &import_name,
                    EdgeType::Imports,
                    codebase_id,
                );
            }
        }

        // --- Implements ---
        "impl_item" => {
            // Rust impl: look for a trait child. Pattern: `impl Trait for Type`
            if let Some(trait_name) = extract_impl_trait(node, content) {
                maybe_add_edge(
                    edges,
                    seen,
                    entity,
                    &trait_name,
                    EdgeType::Implements,
                    codebase_id,
                );
            }
        }

        // --- Type references ---
        "type_identifier" | "generic_type" => {
            let type_name = content[node.byte_range()].trim().to_string();
            if !type_name.is_empty() && !is_primitive_type(&type_name) {
                maybe_add_edge(
                    edges,
                    seen,
                    entity,
                    &type_name,
                    EdgeType::TypeRefs,
                    codebase_id,
                );
            }
        }

        // --- Field access ---
        "field_expression" | "field_access" => {
            if let Some(field_node) = node.child_by_field_name("field") {
                let field_name = content[field_node.byte_range()].to_string();
                maybe_add_edge(
                    edges,
                    seen,
                    entity,
                    &field_name,
                    EdgeType::FieldAccess,
                    codebase_id,
                );
            }
        }

        _ => {}
    }

    // Recurse into children.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_for_edges(
            &child, content, lang, file_path, entity, codebase_id, edges, seen,
        );
    }
}

// ---------------------------------------------------------------------------
// Target extraction helpers
// ---------------------------------------------------------------------------

/// Extract the target name from a call expression node.
fn extract_call_target(node: &Node, content: &str, lang: &str) -> Option<String> {
    match lang {
        "rust" => {
            // Rust call_expression: first child is the function being called.
            // Could be an identifier or a scoped_identifier.
            if let Some(func_node) = node.child_by_field_name("function") {
                let text = content[func_node.byte_range()].to_string();
                // For scoped calls like `module::func`, take the last segment.
                Some(last_segment(&text, "::"))
            } else {
                // Fallback: first child
                let child = node.child(0)?;
                let text = content[child.byte_range()].to_string();
                Some(last_segment(&text, "::"))
            }
        }
        "python" => {
            // Python call: first child is the function node.
            if let Some(func_node) = node.child_by_field_name("function") {
                let text = content[func_node.byte_range()].to_string();
                // For attribute calls like `path.join`, take the last segment.
                Some(last_segment(&text, "."))
            } else {
                let child = node.child(0)?;
                let text = content[child.byte_range()].to_string();
                Some(last_segment(&text, "."))
            }
        }
        _ => {
            // Generic: try "function" field, then first child.
            if let Some(func_node) = node.child_by_field_name("function") {
                let text = content[func_node.byte_range()].to_string();
                Some(last_segment(&text, "."))
            } else {
                let child = node.child(0)?;
                let text = content[child.byte_range()].to_string();
                Some(last_segment(&text, "."))
            }
        }
    }
}

/// Extract the import target name from an import/use node.
fn extract_import_target(node: &Node, content: &str, lang: &str) -> Option<String> {
    let text = content[node.byte_range()].trim().to_string();
    match lang {
        "rust" => {
            // `use std::collections::HashMap;`
            // Strip "use " prefix and trailing ";"
            let stripped = text
                .strip_prefix("use ")?
                .trim_end_matches(';')
                .trim();
            Some(last_segment(stripped, "::"))
        }
        "python" => {
            // `from os import path` -> "path"
            // `import os` -> "os"
            if text.starts_with("from ") {
                // `from X import Y` -> take Y
                let import_idx = text.find(" import ")?;
                let after_import = &text[import_idx + 8..];
                let name = after_import.trim().split(',').next()?.trim().to_string();
                Some(name)
            } else {
                // `import X` -> take last segment
                let stripped = text.strip_prefix("import ")?.trim();
                Some(last_segment(stripped, "."))
            }
        }
        _ => {
            // Generic: take the full text as-is
            Some(text)
        }
    }
}

/// Extract trait name from a Rust `impl_item` if it's a trait impl.
fn extract_impl_trait(node: &Node, content: &str) -> Option<String> {
    // Rust impl_item: `impl Trait for Type { ... }`
    // Look for a child with field name "trait" or scan children for the `for` keyword.
    if let Some(trait_node) = node.child_by_field_name("trait") {
        let name = content[trait_node.byte_range()].trim().to_string();
        return Some(last_segment(&name, "::"));
    }

    // Fallback: scan text for `impl TRAIT for TYPE`
    let text = content[node.byte_range()].to_string();
    let first_line = text.lines().next()?;
    if first_line.contains(" for ") {
        // Pattern: `impl SomeTrait for SomeType`
        let after_impl = first_line.strip_prefix("impl ")?.trim();
        let trait_part = after_impl.split(" for ").next()?.trim();
        return Some(last_segment(trait_part, "::"));
    }

    None
}

/// Check if a type name is a primitive that should be excluded from TypeRefs.
fn is_primitive_type(name: &str) -> bool {
    matches!(
        name,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
            | "bool"
            | "char"
            | "str"
            | "String"
            | "Self"
            | "self"
            | "int"
            | "float"
            | "None"
            | "void"
            | "string"
    )
}

/// Return the last segment of a path separated by `sep`.
fn last_segment(path: &str, sep: &str) -> String {
    path.rsplit(sep)
        .next()
        .unwrap_or(path)
        .trim()
        .to_string()
}

// ---------------------------------------------------------------------------
// maybe_add_edge (deduplication)
// ---------------------------------------------------------------------------

/// Add an edge if it hasn't already been seen for this entity.
fn maybe_add_edge(
    edges: &mut Vec<EdgeRecord>,
    seen: &mut HashSet<EdgeKey>,
    entity: &CodeEntityRecord,
    target_name: &str,
    edge_type: EdgeType,
    codebase_id: i64,
) {
    if target_name.is_empty() {
        return;
    }

    let key: EdgeKey = (
        entity.pk.file_path.clone(),
        entity.pk.start_line,
        entity.pk.end_line,
        target_name.to_string(),
        edge_type.to_string(),
    );

    if seen.contains(&key) {
        return;
    }
    seen.insert(key);

    edges.push(EdgeRecord {
        from_path: entity.pk.file_path.clone(),
        from_start: entity.pk.start_line,
        from_end: entity.pk.end_line,
        to_path: target_name.to_string(),
        to_start: -1,
        to_end: -1,
        edge_type: edge_type.to_string(),
        codebase_id,
    });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::walker::language_config_registry::get_language_config;
    use crate::walker::treesitter_entity_extractor::extract_entities_from_source;

    /// Helper: extract entities then edges from inline source.
    fn extract_test_edges(
        file_path: &str,
        source: &str,
        lang_name: &str,
    ) -> (Vec<CodeEntityRecord>, Vec<EdgeRecord>) {
        let config = get_language_config(lang_name).expect("language config");
        let entities =
            extract_entities_from_source(file_path, source, &config, 1, "test_hash").unwrap();
        let edges =
            extract_edges_from_source(file_path, source, &config, &entities, 1).unwrap();
        (entities, edges)
    }

    // ------------------------------------------------------------------
    // Test 1: Rust call edges
    // ------------------------------------------------------------------
    #[test]
    fn test_rust_call_edges() {
        let source = "fn add(a: i32, b: i32) -> i32 { a + b }\nfn helper() -> i32 { 42 }\nfn main() { add(1, 2); helper(); }";
        let (_entities, edges) = extract_test_edges("test.rs", source, "rust");

        let call_edges: Vec<_> = edges
            .iter()
            .filter(|e| e.edge_type == "calls")
            .collect();

        let targets: Vec<&str> = call_edges.iter().map(|e| e.to_path.as_str()).collect();
        assert!(
            targets.contains(&"add"),
            "expected 'add' call edge, got {targets:?}"
        );
        assert!(
            targets.contains(&"helper"),
            "expected 'helper' call edge, got {targets:?}"
        );
    }

    // ------------------------------------------------------------------
    // Test 2: Rust import edges
    // ------------------------------------------------------------------
    #[test]
    fn test_rust_import_edges() {
        let source = "use std::collections::HashMap;\nfn foo() {}";
        let (_entities, edges) = extract_test_edges("test.rs", source, "rust");

        let import_edges: Vec<_> = edges
            .iter()
            .filter(|e| e.edge_type == "imports")
            .collect();

        assert!(
            !import_edges.is_empty(),
            "expected at least one import edge"
        );
        let targets: Vec<&str> = import_edges.iter().map(|e| e.to_path.as_str()).collect();
        assert!(
            targets.contains(&"HashMap"),
            "expected 'HashMap' import edge, got {targets:?}"
        );
    }

    // ------------------------------------------------------------------
    // Test 3: Python edges (import + calls)
    // ------------------------------------------------------------------
    #[test]
    fn test_python_edges() {
        let source = "from os import path\ndef foo():\n    path.join(\"a\", \"b\")";
        let (_entities, edges) = extract_test_edges("test.py", source, "python");

        let import_edges: Vec<_> = edges
            .iter()
            .filter(|e| e.edge_type == "imports")
            .collect();
        let call_edges: Vec<_> = edges
            .iter()
            .filter(|e| e.edge_type == "calls")
            .collect();

        let import_targets: Vec<&str> = import_edges.iter().map(|e| e.to_path.as_str()).collect();
        assert!(
            import_targets.contains(&"path"),
            "expected 'path' import edge, got {import_targets:?}"
        );

        let call_targets: Vec<&str> = call_edges.iter().map(|e| e.to_path.as_str()).collect();
        assert!(
            call_targets.contains(&"join"),
            "expected 'join' call edge, got {call_targets:?}"
        );
    }

    // ------------------------------------------------------------------
    // Test 6: Edge deduplication
    // ------------------------------------------------------------------
    #[test]
    fn test_edge_deduplication() {
        let source = "fn f() { g(); g(); g(); }\nfn g() {}";
        let (_entities, edges) = extract_test_edges("test.rs", source, "rust");

        let g_calls: Vec<_> = edges
            .iter()
            .filter(|e| e.edge_type == "calls" && e.to_path == "g")
            .collect();

        assert_eq!(
            g_calls.len(),
            1,
            "expected exactly 1 Calls edge to 'g', got {}",
            g_calls.len()
        );
    }
}
