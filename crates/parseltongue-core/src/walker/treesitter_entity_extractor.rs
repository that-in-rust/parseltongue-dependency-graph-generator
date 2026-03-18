//! Tree-sitter based entity extraction from source code.

use crate::entity_type_definitions::{
    CodeEntityRecord, EntityPrimaryKeyLocation, EntityTypeClassification,
};
use crate::walker::language_config_registry::LanguageConfigDefinition;
use anyhow::{Context, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use tree_sitter::{Node, Parser};

// ---------------------------------------------------------------------------
// classify_node_to_entity_type
// ---------------------------------------------------------------------------

/// Maps a tree-sitter node kind + language name to an [`EntityTypeClassification`].
///
/// Returns `None` for unrecognized node kinds (they should be skipped).
pub fn classify_node_to_entity_type(
    kind: &str,
    lang: &str,
) -> Option<EntityTypeClassification> {
    match lang {
        // ----- Rust -----
        "rust" => match kind {
            "function_item" => Some(EntityTypeClassification::Function),
            "struct_item" => Some(EntityTypeClassification::Struct),
            "enum_item" => Some(EntityTypeClassification::Enum),
            "trait_item" => Some(EntityTypeClassification::Trait),
            "impl_item" => Some(EntityTypeClassification::Impl),
            "type_item" => Some(EntityTypeClassification::TypeAlias),
            "const_item" => Some(EntityTypeClassification::Constant),
            "static_item" => Some(EntityTypeClassification::Static),
            "macro_definition" => Some(EntityTypeClassification::Macro),
            "mod_item" => Some(EntityTypeClassification::Module),
            "use_declaration" => Some(EntityTypeClassification::Import),
            "line_comment" | "block_comment" => Some(EntityTypeClassification::Comment),
            "attribute_item" | "inner_attribute_item" => {
                Some(EntityTypeClassification::Attribute)
            }
            _ => None,
        },

        // ----- Python -----
        "python" => match kind {
            "function_definition" => Some(EntityTypeClassification::Function),
            "class_definition" => Some(EntityTypeClassification::Class),
            "import_statement" | "import_from_statement" => {
                Some(EntityTypeClassification::Import)
            }
            // decorated_definition is handled by unwrapping to inner
            "decorated_definition" => Some(EntityTypeClassification::Function),
            "comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // ----- JavaScript -----
        "javascript" => match kind {
            "function_declaration" => Some(EntityTypeClassification::Function),
            "class_declaration" => Some(EntityTypeClassification::Class),
            "method_definition" => Some(EntityTypeClassification::Method),
            "variable_declaration" | "lexical_declaration" => {
                Some(EntityTypeClassification::Variable)
            }
            "import_statement" => Some(EntityTypeClassification::Import),
            "export_statement" => Some(EntityTypeClassification::Import),
            "comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // ----- TypeScript / TSX -----
        "typescript" | "tsx" => match kind {
            "function_declaration" => Some(EntityTypeClassification::Function),
            "class_declaration" => Some(EntityTypeClassification::Class),
            "method_definition" => Some(EntityTypeClassification::Method),
            "variable_declaration" | "lexical_declaration" => {
                Some(EntityTypeClassification::Variable)
            }
            "import_statement" => Some(EntityTypeClassification::Import),
            "export_statement" => Some(EntityTypeClassification::Import),
            "interface_declaration" => Some(EntityTypeClassification::Interface),
            "type_alias_declaration" => Some(EntityTypeClassification::TypeAlias),
            "enum_declaration" => Some(EntityTypeClassification::Enum),
            "comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // ----- Go -----
        "go" => match kind {
            "function_declaration" => Some(EntityTypeClassification::Function),
            "method_declaration" => Some(EntityTypeClassification::Method),
            "type_declaration" => Some(EntityTypeClassification::Struct), // unwrapped below
            "import_declaration" => Some(EntityTypeClassification::Import),
            "comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // ----- Java -----
        "java" => match kind {
            "class_declaration" => Some(EntityTypeClassification::Class),
            "method_declaration" => Some(EntityTypeClassification::Method),
            "interface_declaration" => Some(EntityTypeClassification::Interface),
            "constructor_declaration" => Some(EntityTypeClassification::Constructor),
            "import_declaration" => Some(EntityTypeClassification::Import),
            "line_comment" | "block_comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // ----- C -----
        "c" => match kind {
            "function_definition" => Some(EntityTypeClassification::Function),
            "struct_specifier" => Some(EntityTypeClassification::Struct),
            "enum_specifier" => Some(EntityTypeClassification::Enum),
            "comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // ----- C++ -----
        "cpp" => match kind {
            "function_definition" => Some(EntityTypeClassification::Function),
            "struct_specifier" => Some(EntityTypeClassification::Struct),
            "enum_specifier" => Some(EntityTypeClassification::Enum),
            "class_specifier" => Some(EntityTypeClassification::Class),
            "namespace_definition" => Some(EntityTypeClassification::Namespace),
            "comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // ----- Ruby -----
        "ruby" => match kind {
            "method" => Some(EntityTypeClassification::Function),
            "class" => Some(EntityTypeClassification::Class),
            "module" => Some(EntityTypeClassification::Module),
            "comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // ----- PHP -----
        "php" => match kind {
            "function_definition" => Some(EntityTypeClassification::Function),
            "class_declaration" => Some(EntityTypeClassification::Class),
            "method_declaration" => Some(EntityTypeClassification::Method),
            "interface_declaration" => Some(EntityTypeClassification::Interface),
            "comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // ----- C# -----
        "csharp" => match kind {
            "method_declaration" => Some(EntityTypeClassification::Method),
            "class_declaration" => Some(EntityTypeClassification::Class),
            "struct_declaration" => Some(EntityTypeClassification::Struct),
            "interface_declaration" => Some(EntityTypeClassification::Interface),
            "enum_declaration" => Some(EntityTypeClassification::Enum),
            "namespace_declaration" => Some(EntityTypeClassification::Namespace),
            "constructor_declaration" => Some(EntityTypeClassification::Constructor),
            "comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // ----- Swift -----
        "swift" => match kind {
            "function_declaration" => Some(EntityTypeClassification::Function),
            "class_declaration" => Some(EntityTypeClassification::Class),
            "struct_declaration" => Some(EntityTypeClassification::Struct),
            "enum_declaration" => Some(EntityTypeClassification::Enum),
            "protocol_declaration" => Some(EntityTypeClassification::Interface),
            "import_declaration" => Some(EntityTypeClassification::Import),
            "comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // ----- Kotlin -----
        "kotlin" => match kind {
            "function_declaration" => Some(EntityTypeClassification::Function),
            "class_declaration" => Some(EntityTypeClassification::Class),
            "object_declaration" => Some(EntityTypeClassification::Object),
            "import_header" => Some(EntityTypeClassification::Import),
            "line_comment" | "multiline_comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // ----- Scala -----
        "scala" => match kind {
            "function_definition" => Some(EntityTypeClassification::Function),
            "class_definition" => Some(EntityTypeClassification::Class),
            "trait_definition" => Some(EntityTypeClassification::Trait),
            "object_definition" => Some(EntityTypeClassification::Object),
            "import_declaration" => Some(EntityTypeClassification::Import),
            "comment" | "block_comment" => Some(EntityTypeClassification::Comment),
            _ => None,
        },

        // Fallback: try comment for any language
        _ => match kind {
            "comment" | "line_comment" | "block_comment" => {
                Some(EntityTypeClassification::Comment)
            }
            _ => None,
        },
    }
}

// ---------------------------------------------------------------------------
// Split-node kinds (nodes that can be descended into)
// ---------------------------------------------------------------------------

/// Returns true if this entity type is a "split node" that should have its
/// body children extracted when it exceeds 150 lines.
fn is_split_node(et: &EntityTypeClassification) -> bool {
    matches!(
        et,
        EntityTypeClassification::Class
            | EntityTypeClassification::Impl
            | EntityTypeClassification::Module
            | EntityTypeClassification::Namespace
            | EntityTypeClassification::Interface
    )
}

// ---------------------------------------------------------------------------
// is_test detection
// ---------------------------------------------------------------------------

/// Check whether a node's source text indicates a test entity.
fn detect_is_test(node_text: &str, lang: &str, content: &str, node: &Node) -> bool {
    match lang {
        "rust" => {
            // Check if the node or its preceding sibling is #[test]
            // Walk backwards from the node to find attribute items
            if node_text.contains("#[test]") || node_text.contains("#[cfg(test)]") {
                return true;
            }
            // Check preceding siblings for #[test] attribute
            let mut prev = node.prev_sibling();
            while let Some(sib) = prev {
                if sib.kind() == "attribute_item" {
                    let sib_text = &content[sib.byte_range()];
                    if sib_text.contains("#[test]") {
                        return true;
                    }
                } else {
                    break;
                }
                prev = sib.prev_sibling();
            }
            false
        }
        "python" => {
            // test_ prefix on function name
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = &content[name_node.byte_range()];
                if name.starts_with("test_") {
                    return true;
                }
            }
            false
        }
        "java" | "kotlin" => {
            // @Test annotation
            // Check previous siblings for annotations
            let mut prev = node.prev_sibling();
            while let Some(sib) = prev {
                let sib_text = &content[sib.byte_range()];
                if sib_text.contains("@Test") {
                    return true;
                }
                if sib.kind() != "annotation" && sib.kind() != "marker_annotation" {
                    break;
                }
                prev = sib.prev_sibling();
            }
            node_text.contains("@Test")
        }
        "javascript" | "typescript" | "tsx" => {
            // it(, describe(, test( patterns
            let trimmed = node_text.trim_start();
            trimmed.starts_with("it(")
                || trimmed.starts_with("it.only(")
                || trimmed.starts_with("describe(")
                || trimmed.starts_with("test(")
                || trimmed.starts_with("test.only(")
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// extract_entities_from_source
// ---------------------------------------------------------------------------

/// Extract entities from a source file using tree-sitter.
///
/// Returns a `Vec<CodeEntityRecord>` with one record per top-level entity.
pub fn extract_entities_from_source(
    file_path: &str,
    content: &str,
    config: &LanguageConfigDefinition,
    codebase_id: i64,
    file_hash: &str,
) -> Result<Vec<CodeEntityRecord>> {
    let mut parser = Parser::new();
    let lang = (config.tree_sitter_language)();
    parser
        .set_language(&lang)
        .context("Failed to set tree-sitter language")?;

    let tree = parser
        .parse(content, None)
        .context("Tree-sitter failed to parse content")?;

    let root = tree.root_node();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let mut entities = Vec::new();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor) {
        extract_node(
            &child,
            file_path,
            content,
            &config.name,
            codebase_id,
            file_hash,
            now,
            &mut entities,
            false,
        );
    }

    Ok(entities)
}

/// Recursively extract a single node into entities.
fn extract_node(
    node: &Node,
    file_path: &str,
    content: &str,
    lang: &str,
    codebase_id: i64,
    file_hash: &str,
    indexed_at: i64,
    entities: &mut Vec<CodeEntityRecord>,
    is_nested: bool,
) {
    let kind = node.kind();

    // For Python decorated_definition, unwrap to the inner definition
    if lang == "python" && kind == "decorated_definition" {
        // Find the inner function_definition or class_definition
        let mut child_cursor = node.walk();
        for child in node.children(&mut child_cursor) {
            let ck = child.kind();
            if ck == "function_definition" || ck == "class_definition" {
                extract_node(
                    &child,
                    file_path,
                    content,
                    lang,
                    codebase_id,
                    file_hash,
                    indexed_at,
                    entities,
                    is_nested,
                );
                return;
            }
        }
        // If no inner definition found, skip
        return;
    }

    // For JS/TS export_statement, unwrap to the inner declaration
    if (lang == "javascript" || lang == "typescript" || lang == "tsx")
        && kind == "export_statement"
    {
        let mut child_cursor = node.walk();
        for child in node.children(&mut child_cursor) {
            let ck = child.kind();
            if classify_node_to_entity_type(ck, lang).is_some() && ck != "export_statement" {
                extract_node(
                    &child,
                    file_path,
                    content,
                    lang,
                    codebase_id,
                    file_hash,
                    indexed_at,
                    entities,
                    is_nested,
                );
                return;
            }
        }
        // If nothing interesting inside, skip
        return;
    }

    // For Go type_declaration, unwrap to inner type_spec
    if lang == "go" && kind == "type_declaration" {
        let mut child_cursor = node.walk();
        for child in node.children(&mut child_cursor) {
            if child.kind() == "type_spec" {
                // Determine if it's a struct_type or interface_type
                let mut spec_cursor = child.walk();
                let mut entity_type = EntityTypeClassification::Struct;
                for spec_child in child.children(&mut spec_cursor) {
                    match spec_child.kind() {
                        "interface_type" => {
                            entity_type = EntityTypeClassification::Interface;
                        }
                        "struct_type" => {
                            entity_type = EntityTypeClassification::Struct;
                        }
                        _ => {}
                    }
                }
                let name = child
                    .child_by_field_name("name")
                    .map(|n| content[n.byte_range()].to_string())
                    .unwrap_or_default();
                let node_text = &content[node.byte_range()];
                let signature = node_text.lines().next().unwrap_or("").to_string();
                let start_line = node.start_position().row as i32 + 1;
                let end_line = node.end_position().row as i32 + 1;
                let wc = count_words(node_text);
                let is_test = false;

                entities.push(CodeEntityRecord {
                    pk: EntityPrimaryKeyLocation {
                        file_path: file_path.to_string(),
                        start_line,
                        end_line,
                    },
                    entity_type,
                    language: lang.to_string(),
                    name,
                    signature,
                    snippet: node_text.to_string(),
                    doc_comment: None,
                    wc,
                    file_hash: file_hash.to_string(),
                    is_test,
                    codebase_id,
                    indexed_at,
                    rustc_scope: None,
                    rustc_sig: None,
                    visibility: None,
                    mir_calls: None,
                    trait_impls: None,
                });
                return;
            }
        }
        return;
    }

    let entity_type = match classify_node_to_entity_type(kind, lang) {
        Some(et) => et,
        None => return, // unrecognized kind, skip
    };

    let node_text = &content[node.byte_range()];
    let start_line = node.start_position().row as i32 + 1;
    let end_line = node.end_position().row as i32 + 1;
    let line_count = (end_line - start_line + 1) as usize;

    // Extract name
    let name = node
        .child_by_field_name("name")
        .map(|n| content[n.byte_range()].to_string())
        .unwrap_or_default();

    let signature = node_text.lines().next().unwrap_or("").to_string();
    let wc = count_words(node_text);
    let is_test = detect_is_test(node_text, lang, content, node);

    // For split nodes exceeding 150 lines, descend into body children
    if !is_nested && is_split_node(&entity_type) && line_count > 150 {
        // Extract children from the body
        let body_node = node.child_by_field_name("body").unwrap_or_else(|| {
            // Some node kinds use "members" or have direct children
            let mut c = node.walk();
            let found = node
                .children(&mut c)
                .find(|child| {
                    child.kind().contains("body") || child.kind().contains("block")
                });
            found.unwrap_or(*node)
        });

        let mut child_cursor = body_node.walk();
        for child in body_node.children(&mut child_cursor) {
            extract_node(
                &child,
                file_path,
                content,
                lang,
                codebase_id,
                file_hash,
                indexed_at,
                entities,
                true,
            );
        }
        return;
    }

    entities.push(CodeEntityRecord {
        pk: EntityPrimaryKeyLocation {
            file_path: file_path.to_string(),
            start_line,
            end_line,
        },
        entity_type,
        language: lang.to_string(),
        name,
        signature,
        snippet: node_text.to_string(),
        doc_comment: None,
        wc,
        file_hash: file_hash.to_string(),
        is_test,
        codebase_id,
        indexed_at,
        rustc_scope: None,
        rustc_sig: None,
        visibility: None,
        mir_calls: None,
        trait_impls: None,
    });
}

/// Count whitespace-separated words in text.
fn count_words(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::walker::language_config_registry::get_language_config;

    #[test]
    fn test_rust_parsing() {
        let source = "fn add(a: i32, b: i32) -> i32 { a + b }\nstruct Point { x: f64, y: f64 }";
        let config = get_language_config("rust").unwrap();
        let entities =
            extract_entities_from_source("test.rs", source, &config, 1, "abc123").unwrap();

        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        assert!(
            names.contains(&"add"),
            "expected 'add' in {names:?}"
        );
        assert!(
            names.contains(&"Point"),
            "expected 'Point' in {names:?}"
        );

        let func = entities.iter().find(|e| e.name == "add").unwrap();
        assert_eq!(func.entity_type, EntityTypeClassification::Function);

        let strct = entities.iter().find(|e| e.name == "Point").unwrap();
        assert_eq!(strct.entity_type, EntityTypeClassification::Struct);
    }

    #[test]
    fn test_python_parsing() {
        let source = "def hello():\n    pass\nclass Foo:\n    pass";
        let config = get_language_config("python").unwrap();
        let entities =
            extract_entities_from_source("test.py", source, &config, 1, "abc123").unwrap();

        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        assert!(
            names.contains(&"hello"),
            "expected 'hello' in {names:?}"
        );
        assert!(
            names.contains(&"Foo"),
            "expected 'Foo' in {names:?}"
        );

        let func = entities.iter().find(|e| e.name == "hello").unwrap();
        assert_eq!(func.entity_type, EntityTypeClassification::Function);

        let cls = entities.iter().find(|e| e.name == "Foo").unwrap();
        assert_eq!(cls.entity_type, EntityTypeClassification::Class);
    }

    #[test]
    fn test_javascript_parsing() {
        let source = "function greet(name) { return name; }\nclass App { }";
        let config = get_language_config("javascript").unwrap();
        let entities =
            extract_entities_from_source("test.js", source, &config, 1, "abc123").unwrap();

        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        assert!(
            names.contains(&"greet"),
            "expected 'greet' in {names:?}"
        );
        assert!(
            names.contains(&"App"),
            "expected 'App' in {names:?}"
        );

        let func = entities.iter().find(|e| e.name == "greet").unwrap();
        assert_eq!(func.entity_type, EntityTypeClassification::Function);

        let cls = entities.iter().find(|e| e.name == "App").unwrap();
        assert_eq!(cls.entity_type, EntityTypeClassification::Class);
    }

    #[test]
    fn test_is_test_detection_rust() {
        let source = "#[test]\nfn test_something() { }";
        let config = get_language_config("rust").unwrap();
        let entities =
            extract_entities_from_source("test.rs", source, &config, 1, "abc123").unwrap();

        let func = entities
            .iter()
            .find(|e| e.name == "test_something")
            .expect("expected function 'test_something'");
        assert!(func.is_test, "expected is_test == true");
    }

    #[test]
    fn test_classify_unknown_returns_none() {
        assert!(classify_node_to_entity_type("some_random_kind", "rust").is_none());
    }

    #[test]
    fn test_word_count() {
        assert_eq!(count_words("fn add(a: i32, b: i32) -> i32 { a + b }"), 12);
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("   "), 0);
        assert_eq!(count_words("hello"), 1);
    }

    #[test]
    fn test_e2e_workspace_calculator() {
        // Try to parse the e2e_workspace calculator.rs
        let calc_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests-preV200/e2e_workspace/src/calculator.rs"
        );
        let content = match std::fs::read_to_string(calc_path) {
            Ok(c) => c,
            Err(_) => {
                // Use inline source if file doesn't exist
                "/// Add two numbers\npub fn add(a: i32, b: i32) -> i32 { a + b }\n/// Subtract\npub fn subtract(a: i32, b: i32) -> i32 { a - b }\n".to_string()
            }
        };

        let config = get_language_config("rust").unwrap();
        let entities = extract_entities_from_source(
            "src/calculator.rs",
            &content,
            &config,
            1,
            "hash123",
        )
        .unwrap();

        // Should find at least add and subtract functions
        let code_entities: Vec<_> = entities
            .iter()
            .filter(|e| e.entity_type == EntityTypeClassification::Function)
            .collect();
        assert!(
            code_entities.len() >= 2,
            "expected at least 2 functions, got {}",
            code_entities.len()
        );
    }
}
