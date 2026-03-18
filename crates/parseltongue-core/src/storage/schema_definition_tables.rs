//! Schema DDL definitions for the Parseltongue storage layer.

/// Returns all CREATE TABLE and CREATE INDEX statements needed to
/// bootstrap the Parseltongue schema.
pub fn all_schema_ddl() -> Vec<&'static str> {
    vec![
        // -----------------------------------------------------------------
        // codebases
        // -----------------------------------------------------------------
        "CREATE TABLE IF NOT EXISTS codebases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            root_path TEXT UNIQUE,
            name TEXT,
            created_at INTEGER,
            last_indexed_at INTEGER
        )",
        // -----------------------------------------------------------------
        // indexed_files
        // -----------------------------------------------------------------
        "CREATE TABLE IF NOT EXISTS indexed_files (
            file_path TEXT,
            codebase_id INTEGER,
            file_hash TEXT,
            indexed_at INTEGER,
            PRIMARY KEY (file_path, codebase_id)
        )",
        // -----------------------------------------------------------------
        // entities
        // -----------------------------------------------------------------
        "CREATE TABLE IF NOT EXISTS entities (
            file_path TEXT,
            start_line INTEGER,
            end_line INTEGER,
            entity_type TEXT,
            language TEXT,
            name TEXT,
            signature TEXT,
            snippet TEXT,
            doc_comment TEXT,
            wc INTEGER,
            file_hash TEXT,
            is_test INTEGER,
            codebase_id INTEGER,
            indexed_at INTEGER,
            rustc_scope TEXT,
            rustc_sig TEXT,
            visibility TEXT,
            mir_calls TEXT,
            trait_impls TEXT,
            PRIMARY KEY (file_path, start_line, end_line, codebase_id)
        )",
        // -----------------------------------------------------------------
        // edges
        // -----------------------------------------------------------------
        "CREATE TABLE IF NOT EXISTS edges (
            from_path TEXT,
            from_start INTEGER,
            from_end INTEGER,
            to_path TEXT,
            to_start INTEGER,
            to_end INTEGER,
            edge_type TEXT,
            codebase_id INTEGER
        )",
        // -----------------------------------------------------------------
        // Indexes
        // -----------------------------------------------------------------
        "CREATE INDEX IF NOT EXISTS idx_entities_codebase_id
            ON entities (codebase_id)",
        "CREATE INDEX IF NOT EXISTS idx_entities_file_path_codebase_id
            ON entities (file_path, codebase_id)",
        "CREATE INDEX IF NOT EXISTS idx_entities_name_codebase_id
            ON entities (name, codebase_id)",
        "CREATE INDEX IF NOT EXISTS idx_edges_from
            ON edges (from_path, from_start, from_end, codebase_id)",
        "CREATE INDEX IF NOT EXISTS idx_edges_to
            ON edges (to_path, to_start, to_end, codebase_id)",
        "CREATE INDEX IF NOT EXISTS idx_edges_codebase_id
            ON edges (codebase_id)",
        "CREATE INDEX IF NOT EXISTS idx_indexed_files_codebase_id
            ON indexed_files (codebase_id)",
    ]
}

/// Returns the DDL for an FTS5 virtual table scoped to a given codebase.
///
/// Uses `content=''` (contentless) with the `porter` tokenizer so that
/// we control the content via manual INSERT/DELETE.
pub fn fts5_create_table_sql(codebase_id: i64) -> String {
    format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS fts_entities_{codebase_id}
         USING fts5(name, signature, doc_comment, file_path, entity_type,
                    content='', tokenize='porter')"
    )
}
