pub mod directory_tree_scanner;
pub mod language_config_registry;

#[cfg(test)]
mod tests {
    use super::directory_tree_scanner::{compute_file_sha256_hash, scan_directory_tree_recursive};
    use super::language_config_registry::{detect_language_from_extension, get_language_config};
    use std::io::Write;

    // -----------------------------------------------------------------------
    // Language detection tests
    // -----------------------------------------------------------------------

    #[test]
    fn detect_language_rust() {
        assert_eq!(
            detect_language_from_extension("rs"),
            Some("rust".to_string())
        );
    }

    #[test]
    fn detect_language_python() {
        assert_eq!(
            detect_language_from_extension("py"),
            Some("python".to_string())
        );
    }

    #[test]
    fn detect_language_unknown_returns_none() {
        assert_eq!(detect_language_from_extension("xyz"), None);
    }

    #[test]
    fn detect_language_typescript_variants() {
        assert_eq!(
            detect_language_from_extension("ts"),
            Some("typescript".to_string())
        );
        assert_eq!(
            detect_language_from_extension("tsx"),
            Some("tsx".to_string())
        );
    }

    #[test]
    fn detect_language_config_files() {
        assert_eq!(
            detect_language_from_extension("json"),
            Some("config".to_string())
        );
        assert_eq!(
            detect_language_from_extension("yaml"),
            Some("config".to_string())
        );
        assert_eq!(
            detect_language_from_extension("toml"),
            Some("config".to_string())
        );
    }

    #[test]
    fn detect_language_text_files() {
        assert_eq!(
            detect_language_from_extension("md"),
            Some("text".to_string())
        );
        assert_eq!(
            detect_language_from_extension("txt"),
            Some("text".to_string())
        );
    }

    #[test]
    fn detect_language_cpp_variants() {
        for ext in &["cpp", "cc", "cxx", "hpp"] {
            assert_eq!(
                detect_language_from_extension(ext),
                Some("cpp".to_string()),
                "failed for extension: {ext}"
            );
        }
    }

    #[test]
    fn detect_language_c_header() {
        assert_eq!(
            detect_language_from_extension("h"),
            Some("c".to_string())
        );
    }

    #[test]
    fn detect_language_kotlin_variants() {
        assert_eq!(
            detect_language_from_extension("kt"),
            Some("kotlin".to_string())
        );
        assert_eq!(
            detect_language_from_extension("kts"),
            Some("kotlin".to_string())
        );
    }

    // -----------------------------------------------------------------------
    // Language config (tree-sitter) tests
    // -----------------------------------------------------------------------

    #[test]
    fn get_language_config_rust_loads() {
        let cfg = get_language_config("rust").expect("rust config should exist");
        assert_eq!(cfg.name, "rust");
        assert!(cfg.extensions.contains(&"rs".to_string()));
        // Verify the tree-sitter language fn produces a valid Language
        let lang: tree_sitter::Language = (cfg.tree_sitter_language)();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang).expect("should load rust grammar");
    }

    #[test]
    fn get_language_config_python_loads() {
        let cfg = get_language_config("python").expect("python config should exist");
        assert_eq!(cfg.name, "python");
        let lang = (cfg.tree_sitter_language)();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&lang)
            .expect("should load python grammar");
    }

    #[test]
    fn get_language_config_none_for_unknown() {
        assert!(get_language_config("brainfuck").is_none());
    }

    #[test]
    fn get_language_config_none_for_config() {
        // "config" is tracked but not tree-sitter parsed
        assert!(get_language_config("config").is_none());
    }

    #[test]
    fn get_language_config_none_for_text() {
        assert!(get_language_config("text").is_none());
    }

    // -----------------------------------------------------------------------
    // SHA-256 tests
    // -----------------------------------------------------------------------

    #[test]
    fn sha256_known_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("hello.txt");
        std::fs::write(&file_path, b"hello world").unwrap();
        let hash = compute_file_sha256_hash(&file_path).unwrap();
        // SHA-256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn sha256_empty_string() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("empty_content.txt");
        // Write a single byte so the file is not actually empty (scanner skips 0-byte files)
        // but we test hashing of content "a"
        std::fs::write(&file_path, b"a").unwrap();
        let hash = compute_file_sha256_hash(&file_path).unwrap();
        assert_eq!(
            hash,
            "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb"
        );
    }

    // -----------------------------------------------------------------------
    // Directory scanner tests
    // -----------------------------------------------------------------------

    #[test]
    fn scan_directory_tree_basic() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Create nested structure
        let sub = root.join("src");
        std::fs::create_dir_all(&sub).unwrap();

        // Normal .rs file
        std::fs::write(sub.join("main.rs"), "fn main() {}").unwrap();

        // Hidden file (should be skipped)
        std::fs::write(root.join(".hidden"), "secret").unwrap();

        // Hidden directory with a file (should be skipped)
        let hidden_dir = root.join(".secret_dir");
        std::fs::create_dir_all(&hidden_dir).unwrap();
        std::fs::write(hidden_dir.join("file.rs"), "fn foo() {}").unwrap();

        // File > 1MB (should be skipped)
        let big_file = sub.join("big.rs");
        {
            let mut f = std::fs::File::create(&big_file).unwrap();
            let buf = vec![b'x'; 1_100_000];
            f.write_all(&buf).unwrap();
        }

        // Empty file (should be skipped)
        std::fs::write(sub.join("empty.rs"), "").unwrap();

        // node_modules dir (should be skipped)
        let nm = root.join("node_modules");
        std::fs::create_dir_all(&nm).unwrap();
        std::fs::write(nm.join("pkg.js"), "module.exports = {}").unwrap();

        let (files, folders) = scan_directory_tree_recursive(root).unwrap();

        // Should only have src/main.rs
        let file_paths: Vec<&str> = files.iter().map(|f| f.file_path.as_str()).collect();
        assert!(
            file_paths.contains(&"src/main.rs"),
            "expected src/main.rs in {file_paths:?}"
        );
        assert_eq!(files.len(), 1, "expected exactly 1 file, got: {file_paths:?}");

        // Check language detection
        assert_eq!(files[0].language, Some("rust".to_string()));

        // Check hash is non-empty
        assert!(!files[0].file_hash.is_empty());

        // Should have "src" folder
        let folder_paths: Vec<&str> = folders.iter().map(|f| f.folder_path.as_str()).collect();
        assert!(
            folder_paths.contains(&"src"),
            "expected 'src' in {folder_paths:?}"
        );

        // Should NOT have node_modules, .hidden, .secret_dir
        assert!(
            !folder_paths.contains(&"node_modules"),
            "node_modules should be excluded"
        );
        assert!(
            !folder_paths.contains(&".secret_dir"),
            ".secret_dir should be excluded"
        );
    }

    #[test]
    fn scan_directory_tree_ignores_target_and_build() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // target/ dir
        let target_dir = root.join("target");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("debug.rs"), "fn x(){}").unwrap();

        // build/ dir
        let build_dir = root.join("build");
        std::fs::create_dir_all(&build_dir).unwrap();
        std::fs::write(build_dir.join("out.js"), "var x;").unwrap();

        // A legit file
        std::fs::write(root.join("lib.rs"), "pub fn lib(){}").unwrap();

        let (files, folders) = scan_directory_tree_recursive(root).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_path, "lib.rs");
        assert!(folders.is_empty(), "no non-ignored folders expected");
    }
}
