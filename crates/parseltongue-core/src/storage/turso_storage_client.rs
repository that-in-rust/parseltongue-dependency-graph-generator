//! Turso / libsql storage client for Parseltongue.

use crate::entity_type_definitions::*;
use crate::storage::schema_definition_tables;
use anyhow::{Context, Result};

/// A storage client backed by a local libsql (SQLite-compatible) database.
pub struct TursoStorageClient {
    #[allow(dead_code)]
    db: libsql::Database,
    conn: libsql::Connection,
}

impl TursoStorageClient {
    // ------------------------------------------------------------------
    // 1. Open database connection
    // ------------------------------------------------------------------

    /// Open (or create) a local database at `path` and return a connected
    /// client. Pass `":memory:"` for an in-memory database.
    pub async fn open_database_connection_path(path: &str) -> Result<Self> {
        let db = libsql::Builder::new_local(path)
            .build()
            .await
            .context("failed to build local libsql database")?;

        let conn = db
            .connect()
            .context("failed to connect to libsql database")?;

        // Pragmas for better write throughput.
        // PRAGMAs return rows, so use query() rather than execute().
        conn.query("PRAGMA busy_timeout = 5000", ())
            .await
            .context("failed to set busy_timeout")?;
        conn.query("PRAGMA journal_mode = WAL", ())
            .await
            .context("failed to set journal_mode")?;

        Ok(Self { db, conn })
    }

    // ------------------------------------------------------------------
    // 2. Schema initialisation
    // ------------------------------------------------------------------

    /// Execute every DDL statement from [`schema_definition_tables::all_schema_ddl`].
    pub async fn initialize_schema_tables_all(&self) -> Result<()> {
        for ddl in schema_definition_tables::all_schema_ddl() {
            self.conn
                .execute(ddl, ())
                .await
                .with_context(|| format!("DDL failed: {ddl}"))?;
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // 3. Codebase entry
    // ------------------------------------------------------------------

    /// Return the id of the codebase with the given `root_path`, creating
    /// it if it does not yet exist.
    pub async fn get_or_create_codebase_entry(&self, root_path: &str) -> Result<i64> {
        // Try to find existing.
        let mut rows = self
            .conn
            .query(
                "SELECT id FROM codebases WHERE root_path = ?1",
                libsql::params![root_path],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let id: i64 = row.get(0)?;
            return Ok(id);
        }

        // Insert new.
        let now = chrono::Utc::now().timestamp();
        let name = root_path
            .rsplit('/')
            .next()
            .unwrap_or(root_path);

        self.conn
            .execute(
                "INSERT INTO codebases (root_path, name, created_at, last_indexed_at) VALUES (?1, ?2, ?3, ?4)",
                libsql::params![root_path, name, now, now],
            )
            .await?;

        let id = self.conn.last_insert_rowid();
        Ok(id)
    }

    /// Return the id of the first codebase in the database, or `None` if
    /// no codebases exist yet.
    pub async fn get_first_codebase_id(&self) -> Result<Option<i64>> {
        let mut rows = self
            .conn
            .query("SELECT id FROM codebases ORDER BY id ASC LIMIT 1", ())
            .await?;

        if let Some(row) = rows.next().await? {
            let id: i64 = row.get(0)?;
            return Ok(Some(id));
        }
        Ok(None)
    }

    // ------------------------------------------------------------------
    // 4. File hash lookup
    // ------------------------------------------------------------------

    /// Return the stored file hash for a file in a codebase, or `None` if
    /// the file has not been indexed.
    pub async fn get_stored_file_hash_value(
        &self,
        file_path: &str,
        codebase_id: i64,
    ) -> Result<Option<String>> {
        let mut rows = self
            .conn
            .query(
                "SELECT file_hash FROM indexed_files WHERE file_path = ?1 AND codebase_id = ?2",
                libsql::params![file_path, codebase_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let hash: String = row.get(0)?;
            return Ok(Some(hash));
        }
        Ok(None)
    }

    // ------------------------------------------------------------------
    // 5. Upsert indexed file
    // ------------------------------------------------------------------

    /// Insert or replace the indexed_files record for a file.
    pub async fn upsert_indexed_file_record(
        &self,
        file_path: &str,
        codebase_id: i64,
        file_hash: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        self.conn
            .execute(
                "INSERT OR REPLACE INTO indexed_files (file_path, codebase_id, file_hash, indexed_at)
                 VALUES (?1, ?2, ?3, ?4)",
                libsql::params![file_path, codebase_id, file_hash, now],
            )
            .await?;
        Ok(())
    }

    // ------------------------------------------------------------------
    // 6. Batch upsert entities
    // ------------------------------------------------------------------

    /// Insert or replace a batch of entity records inside a transaction.
    pub async fn batch_upsert_entity_records(&self, entities: &[CodeEntityRecord]) -> Result<()> {
        if entities.is_empty() {
            return Ok(());
        }

        self.conn.execute("BEGIN", ()).await?;

        for e in entities {
            let res = self
                .conn
                .execute(
                    "INSERT OR REPLACE INTO entities
                     (file_path, start_line, end_line, entity_type, language, name,
                      signature, snippet, doc_comment, wc, file_hash, is_test,
                      codebase_id, indexed_at, rustc_scope, rustc_sig, visibility,
                      mir_calls, trait_impls)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
                    libsql::params![
                        e.pk.file_path.as_str(),
                        e.pk.start_line as i64,
                        e.pk.end_line as i64,
                        e.entity_type.to_string(),
                        e.language.as_str(),
                        e.name.as_str(),
                        e.signature.as_str(),
                        e.snippet.as_str(),
                        opt_str(&e.doc_comment),
                        e.wc as i64,
                        e.file_hash.as_str(),
                        if e.is_test { 1i64 } else { 0i64 },
                        e.codebase_id,
                        e.indexed_at,
                        opt_str(&e.rustc_scope),
                        opt_str(&e.rustc_sig),
                        opt_str(&e.visibility),
                        opt_str(&e.mir_calls),
                        opt_str(&e.trait_impls),
                    ],
                )
                .await;

            if let Err(err) = res {
                // Attempt rollback on failure.
                let _ = self.conn.execute("ROLLBACK", ()).await;
                return Err(err.into());
            }
        }

        self.conn.execute("COMMIT", ()).await?;
        Ok(())
    }

    // ------------------------------------------------------------------
    // 7. Batch upsert edges
    // ------------------------------------------------------------------

    /// Insert a batch of edge records inside a transaction.
    pub async fn batch_upsert_edge_records(&self, edges: &[EdgeRecord]) -> Result<()> {
        if edges.is_empty() {
            return Ok(());
        }

        self.conn.execute("BEGIN", ()).await?;

        for e in edges {
            let res = self
                .conn
                .execute(
                    "INSERT INTO edges
                     (from_path, from_start, from_end, to_path, to_start, to_end, edge_type, codebase_id)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                    libsql::params![
                        e.from_path.as_str(),
                        e.from_start as i64,
                        e.from_end as i64,
                        e.to_path.as_str(),
                        e.to_start as i64,
                        e.to_end as i64,
                        e.edge_type.as_str(),
                        e.codebase_id,
                    ],
                )
                .await;

            if let Err(err) = res {
                let _ = self.conn.execute("ROLLBACK", ()).await;
                return Err(err.into());
            }
        }

        self.conn.execute("COMMIT", ()).await?;
        Ok(())
    }

    // ------------------------------------------------------------------
    // 8. Remove stale file entities
    // ------------------------------------------------------------------

    /// Delete entities, indexed_files, and edges whose `file_path` is
    /// **not** in `active_files` for the given codebase.  If
    /// `active_files` is empty, delete *everything* for that codebase.
    /// Returns the count of deleted entity rows.
    pub async fn remove_stale_file_entities(
        &self,
        active_files: &[String],
        codebase_id: i64,
    ) -> Result<u32> {
        if active_files.is_empty() {
            // Delete all for this codebase.
            let deleted = self
                .conn
                .execute(
                    "DELETE FROM entities WHERE codebase_id = ?1",
                    libsql::params![codebase_id],
                )
                .await?;

            self.conn
                .execute(
                    "DELETE FROM indexed_files WHERE codebase_id = ?1",
                    libsql::params![codebase_id],
                )
                .await?;

            self.conn
                .execute(
                    "DELETE FROM edges WHERE codebase_id = ?1",
                    libsql::params![codebase_id],
                )
                .await?;

            return Ok(deleted as u32);
        }

        // Build a comma-separated list of quoted file paths for the IN clause.
        // We use a temporary table approach for safety with large lists.
        self.conn
            .execute(
                "CREATE TEMPORARY TABLE IF NOT EXISTS _active_files (fp TEXT PRIMARY KEY)",
                (),
            )
            .await?;
        self.conn
            .execute("DELETE FROM _active_files", ())
            .await?;

        self.conn.execute("BEGIN", ()).await?;
        for fp in active_files {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO _active_files (fp) VALUES (?1)",
                    libsql::params![fp.as_str()],
                )
                .await?;
        }
        self.conn.execute("COMMIT", ()).await?;

        let deleted = self
            .conn
            .execute(
                "DELETE FROM entities WHERE codebase_id = ?1
                 AND file_path NOT IN (SELECT fp FROM _active_files)",
                libsql::params![codebase_id],
            )
            .await?;

        self.conn
            .execute(
                "DELETE FROM indexed_files WHERE codebase_id = ?1
                 AND file_path NOT IN (SELECT fp FROM _active_files)",
                libsql::params![codebase_id],
            )
            .await?;

        self.conn
            .execute(
                "DELETE FROM edges WHERE codebase_id = ?1
                 AND from_path NOT IN (SELECT fp FROM _active_files)",
                libsql::params![codebase_id],
            )
            .await?;

        self.conn
            .execute("DROP TABLE IF EXISTS _active_files", ())
            .await?;

        Ok(deleted as u32)
    }

    // ------------------------------------------------------------------
    // 9. Rebuild FTS index
    // ------------------------------------------------------------------

    /// Drop and recreate the FTS5 virtual table for a codebase, then
    /// populate it from searchable entities.  If FTS5 is not available
    /// the error is logged and silently ignored.
    pub async fn rebuild_fts_index_table(&self, codebase_id: i64) -> Result<()> {
        let table_name = format!("fts_entities_{codebase_id}");

        // Drop existing.
        let drop_sql = format!("DROP TABLE IF EXISTS {table_name}");
        if let Err(e) = self.conn.execute(&drop_sql, ()).await {
            tracing::warn!("FTS5 drop failed (skipping): {e}");
            return Ok(());
        }

        // Create.
        let create_sql = schema_definition_tables::fts5_create_table_sql(codebase_id);
        if let Err(e) = self.conn.execute(&create_sql, ()).await {
            tracing::warn!("FTS5 create failed (skipping): {e}");
            return Ok(());
        }

        // Populate from searchable entities.
        let searchable_types: Vec<String> = EntityTypeClassification::all_variants()
            .iter()
            .filter(|v| v.is_searchable())
            .map(|v| format!("'{}'", v))
            .collect();
        let type_list = searchable_types.join(",");

        let insert_sql = format!(
            "INSERT INTO {table_name} (name, signature, doc_comment, file_path, entity_type)
             SELECT name, signature, COALESCE(doc_comment, ''), file_path, entity_type
             FROM entities
             WHERE codebase_id = ?1 AND entity_type IN ({type_list})"
        );

        if let Err(e) = self
            .conn
            .execute(&insert_sql, libsql::params![codebase_id])
            .await
        {
            tracing::warn!("FTS5 populate failed (skipping): {e}");
        }

        Ok(())
    }

    // ------------------------------------------------------------------
    // 10. FTS search
    // ------------------------------------------------------------------

    /// Search for entities using FTS5.  Falls back to LIKE if FTS5
    /// fails.
    pub async fn search_fts_entity_records(
        &self,
        query: &str,
        codebase_id: i64,
        limit: u32,
    ) -> Result<Vec<CodeEntityRecord>> {
        let table_name = format!("fts_entities_{codebase_id}");

        // Try FTS5 first.
        let fts_sql = format!(
            "SELECT e.file_path, e.start_line, e.end_line, e.entity_type, e.language,
                    e.name, e.signature, e.snippet, e.doc_comment, e.wc, e.file_hash,
                    e.is_test, e.codebase_id, e.indexed_at,
                    e.rustc_scope, e.rustc_sig, e.visibility, e.mir_calls, e.trait_impls
             FROM {table_name} AS f
             JOIN entities AS e
               ON e.name = f.name AND e.file_path = f.file_path AND e.codebase_id = ?1
             WHERE {table_name} MATCH ?2
             LIMIT ?3"
        );

        match self
            .conn
            .query(
                &fts_sql,
                libsql::params![codebase_id, query, limit as i64],
            )
            .await
        {
            Ok(mut rows) => {
                let mut results = Vec::new();
                while let Some(row) = rows.next().await? {
                    results.push(row_to_entity(&row)?);
                }
                if !results.is_empty() {
                    return Ok(results);
                }
                // FTS5 returned 0 results – fall through to LIKE.
            }
            Err(e) => {
                tracing::warn!("FTS5 search failed, falling back to LIKE: {e}");
            }
        }

        // Fallback: LIKE search on name / signature / doc_comment.
        let like_pattern = format!("%{query}%");
        let mut rows = self
            .conn
            .query(
                "SELECT file_path, start_line, end_line, entity_type, language,
                        name, signature, snippet, doc_comment, wc, file_hash,
                        is_test, codebase_id, indexed_at,
                        rustc_scope, rustc_sig, visibility, mir_calls, trait_impls
                 FROM entities
                 WHERE codebase_id = ?1
                   AND (name LIKE ?2 OR signature LIKE ?2 OR doc_comment LIKE ?2)
                 LIMIT ?3",
                libsql::params![codebase_id, like_pattern.as_str(), limit as i64],
            )
            .await?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            results.push(row_to_entity(&row)?);
        }
        Ok(results)
    }

    // ------------------------------------------------------------------
    // 11. Get all entities
    // ------------------------------------------------------------------

    /// Return every entity for a codebase.
    pub async fn get_all_entity_records(
        &self,
        codebase_id: i64,
    ) -> Result<Vec<CodeEntityRecord>> {
        let mut rows = self
            .conn
            .query(
                "SELECT file_path, start_line, end_line, entity_type, language,
                        name, signature, snippet, doc_comment, wc, file_hash,
                        is_test, codebase_id, indexed_at,
                        rustc_scope, rustc_sig, visibility, mir_calls, trait_impls
                 FROM entities
                 WHERE codebase_id = ?1",
                libsql::params![codebase_id],
            )
            .await?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            results.push(row_to_entity(&row)?);
        }
        Ok(results)
    }

    // ------------------------------------------------------------------
    // 12. Get all edges
    // ------------------------------------------------------------------

    /// Return every edge for a codebase.
    pub async fn get_all_edge_records(&self, codebase_id: i64) -> Result<Vec<EdgeRecord>> {
        let mut rows = self
            .conn
            .query(
                "SELECT from_path, from_start, from_end, to_path, to_start, to_end,
                        edge_type, codebase_id
                 FROM edges
                 WHERE codebase_id = ?1",
                libsql::params![codebase_id],
            )
            .await?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            results.push(EdgeRecord {
                from_path: row.get::<String>(0)?,
                from_start: row.get::<i32>(1)?,
                from_end: row.get::<i32>(2)?,
                to_path: row.get::<String>(3)?,
                to_start: row.get::<i32>(4)?,
                to_end: row.get::<i32>(5)?,
                edge_type: row.get::<String>(6)?,
                codebase_id: row.get::<i64>(7)?,
            });
        }
        Ok(results)
    }

    // ------------------------------------------------------------------
    // 13. Get entity by PK
    // ------------------------------------------------------------------

    /// Fetch a single entity by its composite primary key.
    pub async fn get_entity_by_pk(
        &self,
        file_path: &str,
        start_line: i32,
        end_line: i32,
        codebase_id: i64,
    ) -> Result<Option<CodeEntityRecord>> {
        let mut rows = self
            .conn
            .query(
                "SELECT file_path, start_line, end_line, entity_type, language,
                        name, signature, snippet, doc_comment, wc, file_hash,
                        is_test, codebase_id, indexed_at,
                        rustc_scope, rustc_sig, visibility, mir_calls, trait_impls
                 FROM entities
                 WHERE file_path = ?1 AND start_line = ?2 AND end_line = ?3 AND codebase_id = ?4",
                libsql::params![file_path, start_line as i64, end_line as i64, codebase_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            return Ok(Some(row_to_entity(&row)?));
        }
        Ok(None)
    }

    // ------------------------------------------------------------------
    // 14. Statistics summary
    // ------------------------------------------------------------------

    /// Return `(entity_count, edge_count)` for a codebase.
    pub async fn get_codebase_statistics_summary(
        &self,
        codebase_id: i64,
    ) -> Result<(u64, u64)> {
        let entity_count = {
            let mut rows = self
                .conn
                .query(
                    "SELECT COUNT(*) FROM entities WHERE codebase_id = ?1",
                    libsql::params![codebase_id],
                )
                .await?;
            match rows.next().await? {
                Some(row) => row.get::<u64>(0)?,
                None => 0,
            }
        };

        let edge_count = {
            let mut rows = self
                .conn
                .query(
                    "SELECT COUNT(*) FROM edges WHERE codebase_id = ?1",
                    libsql::params![codebase_id],
                )
                .await?;
            match rows.next().await? {
                Some(row) => row.get::<u64>(0)?,
                None => 0,
            }
        };

        Ok((entity_count, edge_count))
    }
}

// ======================================================================
// Helpers
// ======================================================================

/// Convert an `Option<String>` to a `libsql::Value`, mapping `None` → Null.
fn opt_str(opt: &Option<String>) -> libsql::Value {
    match opt {
        Some(s) => libsql::Value::Text(s.clone()),
        None => libsql::Value::Null,
    }
}

/// Parse a [`libsql::Row`] (with the standard 19-column entity projection)
/// into a [`CodeEntityRecord`].
fn row_to_entity(row: &libsql::Row) -> Result<CodeEntityRecord> {
    let file_path: String = row.get(0)?;
    let start_line: i32 = row.get(1)?;
    let end_line: i32 = row.get(2)?;
    let entity_type_str: String = row.get(3)?;
    let language: String = row.get(4)?;
    let name: String = row.get(5)?;
    let signature: String = row.get(6)?;
    let snippet: String = row.get(7)?;
    let doc_comment: Option<String> = row.get(8)?;
    let wc: u32 = row.get(9)?;
    let file_hash: String = row.get(10)?;
    let is_test_int: i64 = row.get(11)?;
    let codebase_id: i64 = row.get(12)?;
    let indexed_at: i64 = row.get(13)?;
    let rustc_scope: Option<String> = row.get(14)?;
    let rustc_sig: Option<String> = row.get(15)?;
    let visibility: Option<String> = row.get(16)?;
    let mir_calls: Option<String> = row.get(17)?;
    let trait_impls: Option<String> = row.get(18)?;

    let entity_type: EntityTypeClassification = entity_type_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    Ok(CodeEntityRecord {
        pk: EntityPrimaryKeyLocation {
            file_path,
            start_line,
            end_line,
        },
        entity_type,
        language,
        name,
        signature,
        snippet,
        doc_comment,
        wc,
        file_hash,
        is_test: is_test_int != 0,
        codebase_id,
        indexed_at,
        rustc_scope,
        rustc_sig,
        visibility,
        mir_calls,
        trait_impls,
    })
}
