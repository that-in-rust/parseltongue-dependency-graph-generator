use tree_sitter::Language;

// ---------------------------------------------------------------------------
// LanguageConfigDefinition
// ---------------------------------------------------------------------------

/// Configuration for a supported programming language, including its
/// tree-sitter grammar.
#[derive(Clone)]
pub struct LanguageConfigDefinition {
    /// Canonical language name (e.g. "rust", "python").
    pub name: String,
    /// File extensions associated with this language (without the leading dot).
    pub extensions: Vec<String>,
    /// Function that returns the tree-sitter [`Language`] for this grammar.
    pub tree_sitter_language: fn() -> Language,
}

// ---------------------------------------------------------------------------
// detect_language_from_extension
// ---------------------------------------------------------------------------

/// Maps a file extension (without the leading dot) to a canonical language name.
///
/// Returns `None` for unrecognised extensions.
pub fn detect_language_from_extension(ext: &str) -> Option<String> {
    let lang = match ext {
        // Parsable languages (have tree-sitter grammars)
        "rs" => "rust",
        "py" => "python",
        "js" | "jsx" => "javascript",
        "ts" => "typescript",
        "tsx" => "tsx",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "rb" => "ruby",
        "php" => "php",
        "cs" => "csharp",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "scala" => "scala",

        // Config files (tracked but not tree-sitter parsed)
        "json" | "yaml" | "yml" | "toml" => "config",

        // Text files
        "md" | "txt" | "rst" => "text",

        _ => return None,
    };
    Some(lang.to_string())
}

// ---------------------------------------------------------------------------
// Tree-sitter language wrapper functions
//
// Each function returns `tree_sitter::Language` (v0.25). Most crates expose
// a `LANGUAGE` constant of type `LanguageFn` which converts via `.into()`.
// tree-sitter-kotlin 0.3 uses the old API (`language() -> Language` from
// tree-sitter 0.20), so we re-declare the extern symbol and construct the
// v0.25 Language manually.
// ---------------------------------------------------------------------------

fn ts_lang_rust() -> Language {
    tree_sitter_rust::LANGUAGE.into()
}

fn ts_lang_python() -> Language {
    tree_sitter_python::LANGUAGE.into()
}

fn ts_lang_javascript() -> Language {
    tree_sitter_javascript::LANGUAGE.into()
}

fn ts_lang_typescript() -> Language {
    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
}

fn ts_lang_tsx() -> Language {
    tree_sitter_typescript::LANGUAGE_TSX.into()
}

fn ts_lang_go() -> Language {
    tree_sitter_go::LANGUAGE.into()
}

fn ts_lang_java() -> Language {
    tree_sitter_java::LANGUAGE.into()
}

fn ts_lang_c() -> Language {
    tree_sitter_c::LANGUAGE.into()
}

fn ts_lang_cpp() -> Language {
    tree_sitter_cpp::LANGUAGE.into()
}

fn ts_lang_ruby() -> Language {
    tree_sitter_ruby::LANGUAGE.into()
}

fn ts_lang_php() -> Language {
    tree_sitter_php::LANGUAGE_PHP.into()
}

fn ts_lang_csharp() -> Language {
    tree_sitter_c_sharp::LANGUAGE.into()
}

fn ts_lang_swift() -> Language {
    tree_sitter_swift::LANGUAGE.into()
}

fn ts_lang_scala() -> Language {
    tree_sitter_scala::LANGUAGE.into()
}

// tree-sitter-kotlin 0.3 uses the old tree-sitter 0.20 API, which causes
// duplicate native symbol errors when linked alongside tree-sitter 0.25.
// We bypass the Rust bindings entirely and call the extern C function
// directly, constructing a v0.25 Language from the raw pointer.
//
// `extern crate` ensures the kotlin native library (libparser.a) is linked
// without pulling in tree-sitter 0.20's Rust bindings at the symbol level.
extern crate tree_sitter_kotlin;

extern "C" {
    fn tree_sitter_kotlin() -> *const ();
}

fn ts_lang_kotlin() -> Language {
    let lang_fn =
        unsafe { tree_sitter_language::LanguageFn::from_raw(tree_sitter_kotlin) };
    lang_fn.into()
}

// ---------------------------------------------------------------------------
// get_language_config
// ---------------------------------------------------------------------------

/// Returns the [`LanguageConfigDefinition`] for a given canonical language name.
///
/// Returns `None` for non-parsable categories ("config", "text") and unknown names.
pub fn get_language_config(lang_name: &str) -> Option<LanguageConfigDefinition> {
    let (extensions, ts_fn): (Vec<&str>, fn() -> Language) = match lang_name {
        "rust" => (vec!["rs"], ts_lang_rust as fn() -> Language),
        "python" => (vec!["py"], ts_lang_python),
        "javascript" => (vec!["js", "jsx"], ts_lang_javascript),
        "typescript" => (vec!["ts"], ts_lang_typescript),
        "tsx" => (vec!["tsx"], ts_lang_tsx),
        "go" => (vec!["go"], ts_lang_go),
        "java" => (vec!["java"], ts_lang_java),
        "c" => (vec!["c", "h"], ts_lang_c),
        "cpp" => (vec!["cpp", "cc", "cxx", "hpp"], ts_lang_cpp),
        "ruby" => (vec!["rb"], ts_lang_ruby),
        "php" => (vec!["php"], ts_lang_php),
        "csharp" => (vec!["cs"], ts_lang_csharp),
        "swift" => (vec!["swift"], ts_lang_swift),
        "kotlin" => (vec!["kt", "kts"], ts_lang_kotlin),
        "scala" => (vec!["scala"], ts_lang_scala),
        // "config" and "text" are tracked but have no tree-sitter grammar.
        _ => return None,
    };

    Some(LanguageConfigDefinition {
        name: lang_name.to_string(),
        extensions: extensions.into_iter().map(String::from).collect(),
        tree_sitter_language: ts_fn,
    })
}
