//! 7-phase ingestion pipeline orchestrator.

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;

use parseltongue_core::edges::dependency_edge_extractor::extract_edges_from_source;
use parseltongue_core::edges::hierarchy_edge_builder::build_hierarchy_edges;
use parseltongue_core::entity_type_definitions::{
    CodeEntityRecord, EdgeRecord, EntityPrimaryKeyLocation, EntityTypeClassification,
};
use parseltongue_core::storage::turso_storage_client::TursoStorageClient;
use parseltongue_core::walker::directory_tree_scanner::{
    scan_directory_tree_recursive, FolderEntityRecord, ScannedFileRecord,
};
use parseltongue_core::walker::doc_comment_folding_logic::fold_doc_comments_into_entities;
use parseltongue_core::walker::language_config_registry::{
    detect_language_from_extension, get_language_config,
};
use parseltongue_core::walker::treesitter_entity_extractor::extract_entities_from_source;
use parseltongue_core::walker::word_count_coverage_tracker::compute_whitespace_gap_entities;

// ---------------------------------------------------------------------------
// IngestionResultSummary
// ---------------------------------------------------------------------------

/// Summary of an ingestion pipeline run.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IngestionResultSummary {
    pub files_processed: u32,
    pub entities_created: u32,
    pub edges_created: u32,
    pub files_skipped: u32,
    pub files_removed: u32,
    pub duration_ms: u64,
}

// ---------------------------------------------------------------------------
// IngestionPipelineOrchestrator
// ---------------------------------------------------------------------------

/// Orchestrates the 7-phase ingestion pipeline.
pub struct IngestionPipelineOrchestrator {
    storage: TursoStorageClient,
}

impl IngestionPipelineOrchestrator {
    /// Open a database, initialise schema, and return an orchestrator.
    pub async fn new(db_path: &str) -> Result<Self> {
        let storage = TursoStorageClient::open_database_connection_path(db_path)
            .await
            .context("failed to open database")?;
        storage
            .initialize_schema_tables_all()
            .await
            .context("failed to initialise schema")?;
        Ok(Self { storage })
    }

    /// Run all 7 phases of the ingestion pipeline.
    pub async fn run_ingestion_pipeline_phases(
        &self,
        root_path: &Path,
    ) -> Result<IngestionResultSummary> {
        let start = Instant::now();
        let root_str = root_path.to_string_lossy().to_string();

        // ---------------------------------------------------------------
        // Phase 1: SCAN
        // ---------------------------------------------------------------
        tracing::info!("Phase 1/7: SCAN");
        let (scanned_files, scanned_folders) = scan_directory_tree_recursive(root_path)
            .context("directory scan failed")?;
        tracing::info!(
            "  found {} files, {} folders",
            scanned_files.len(),
            scanned_folders.len()
        );

        // ---------------------------------------------------------------
        // Phase 2: HASH CHECK – determine changed / new files
        // ---------------------------------------------------------------
        tracing::info!("Phase 2/7: HASH CHECK");
        let codebase_id = self
            .storage
            .get_or_create_codebase_entry(&root_str)
            .await?;

        let mut changed_files: Vec<&ScannedFileRecord> = Vec::new();
        let mut files_skipped: u32 = 0;

        for file in &scanned_files {
            let stored_hash = self
                .storage
                .get_stored_file_hash_value(&file.file_path, codebase_id)
                .await?;
            match stored_hash {
                Some(h) if h == file.file_hash => {
                    files_skipped += 1;
                }
                _ => {
                    changed_files.push(file);
                }
            }
        }
        tracing::info!(
            "  {} changed/new, {} unchanged (skipped)",
            changed_files.len(),
            files_skipped
        );

        // ---------------------------------------------------------------
        // Phase 3: CHUNK – extract entities and edges for changed files
        // ---------------------------------------------------------------
        tracing::info!("Phase 3/7: CHUNK");
        let mut all_entities: Vec<CodeEntityRecord> = Vec::new();
        let mut all_edges: Vec<EdgeRecord> = Vec::new();

        let pb = if atty_stdout() {
            let pb = ProgressBar::new(changed_files.len() as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_bar()),
            );
            Some(pb)
        } else {
            None
        };

        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        for file in &changed_files {
            if let Some(ref pb) = pb {
                pb.set_message(file.file_path.clone());
                pb.inc(1);
            }

            let abs_path = root_path.join(&file.file_path);
            let content = match std::fs::read_to_string(&abs_path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("  skipping {}: {e}", file.file_path);
                    continue;
                }
            };

            // Detect language from extension
            let ext = abs_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let lang_name = detect_language_from_extension(ext);

            match &lang_name {
                Some(lang) if get_language_config(lang).is_some() => {
                    // Parsable language with tree-sitter config
                    let config = get_language_config(lang).unwrap();

                    // Extract entities
                    let mut entities = extract_entities_from_source(
                        &file.file_path,
                        &content,
                        &config,
                        codebase_id,
                        &file.file_hash,
                    )
                    .unwrap_or_else(|e| {
                        tracing::warn!("  entity extraction failed for {}: {e}", file.file_path);
                        Vec::new()
                    });

                    // Fold doc comments
                    fold_doc_comments_into_entities(&mut entities);

                    // Compute whitespace gap entities
                    let gaps = compute_whitespace_gap_entities(
                        &file.file_path,
                        &content,
                        &entities,
                        codebase_id,
                        &file.file_hash,
                    );
                    entities.extend(gaps);

                    // Extract edges
                    let edges = extract_edges_from_source(
                        &file.file_path,
                        &content,
                        &config,
                        &entities,
                        codebase_id,
                    )
                    .unwrap_or_else(|e| {
                        tracing::warn!("  edge extraction failed for {}: {e}", file.file_path);
                        Vec::new()
                    });

                    // Create FileParsable entity for this file
                    all_entities.push(CodeEntityRecord {
                        pk: EntityPrimaryKeyLocation::file_sentinel(file.file_path.clone()),
                        entity_type: EntityTypeClassification::FileParsable,
                        language: lang.clone(),
                        name: file.file_path.clone(),
                        signature: String::new(),
                        snippet: String::new(),
                        doc_comment: None,
                        wc: 0,
                        file_hash: file.file_hash.clone(),
                        is_test: false,
                        codebase_id,
                        indexed_at: now_ts,
                        rustc_scope: None,
                        rustc_sig: None,
                        visibility: None,
                        mir_calls: None,
                        trait_impls: None,
                    });

                    all_entities.extend(entities);
                    all_edges.extend(edges);
                }
                Some(lang) if lang == "config" => {
                    // Config file
                    all_entities.push(CodeEntityRecord {
                        pk: EntityPrimaryKeyLocation::file_sentinel(file.file_path.clone()),
                        entity_type: EntityTypeClassification::FileConfig,
                        language: lang.clone(),
                        name: file.file_path.clone(),
                        signature: String::new(),
                        snippet: String::new(),
                        doc_comment: None,
                        wc: content.split_whitespace().count() as u32,
                        file_hash: file.file_hash.clone(),
                        is_test: false,
                        codebase_id,
                        indexed_at: now_ts,
                        rustc_scope: None,
                        rustc_sig: None,
                        visibility: None,
                        mir_calls: None,
                        trait_impls: None,
                    });
                }
                _ => {
                    // Unparsable file
                    let lang_str = lang_name.unwrap_or_default();
                    all_entities.push(CodeEntityRecord {
                        pk: EntityPrimaryKeyLocation::file_sentinel(file.file_path.clone()),
                        entity_type: EntityTypeClassification::FileUnparsable,
                        language: lang_str,
                        name: file.file_path.clone(),
                        signature: String::new(),
                        snippet: String::new(),
                        doc_comment: None,
                        wc: content.split_whitespace().count() as u32,
                        file_hash: file.file_hash.clone(),
                        is_test: false,
                        codebase_id,
                        indexed_at: now_ts,
                        rustc_scope: None,
                        rustc_sig: None,
                        visibility: None,
                        mir_calls: None,
                        trait_impls: None,
                    });
                }
            }
        }

        // Create Folder entities from folder list
        for folder in &scanned_folders {
            all_entities.push(CodeEntityRecord {
                pk: EntityPrimaryKeyLocation::folder_sentinel(folder.folder_path.clone()),
                entity_type: EntityTypeClassification::Folder,
                language: String::new(),
                name: folder.folder_path.clone(),
                signature: String::new(),
                snippet: String::new(),
                doc_comment: None,
                wc: 0,
                file_hash: String::new(),
                is_test: false,
                codebase_id,
                indexed_at: now_ts,
                rustc_scope: None,
                rustc_sig: None,
                visibility: None,
                mir_calls: None,
                trait_impls: None,
            });
        }

        if let Some(ref pb) = pb {
            pb.finish_with_message("done");
        }

        // ---------------------------------------------------------------
        // Phase 4: WRITE – persist entities, edges, indexed_file records
        // ---------------------------------------------------------------
        tracing::info!("Phase 4/7: WRITE");
        let entities_created = all_entities.len() as u32;
        let edges_created = all_edges.len() as u32;

        // Batch upsert entities (batches of 200)
        for chunk in all_entities.chunks(200) {
            self.storage
                .batch_upsert_entity_records(chunk)
                .await
                .context("batch upsert entities failed")?;
        }

        // Batch upsert edges
        self.storage
            .batch_upsert_edge_records(&all_edges)
            .await
            .context("batch upsert edges failed")?;

        // Upsert indexed_file records with new hashes
        for file in &changed_files {
            self.storage
                .upsert_indexed_file_record(&file.file_path, codebase_id, &file.file_hash)
                .await?;
        }

        // ---------------------------------------------------------------
        // Phase 5: CLEANUP – remove stale file entities
        // ---------------------------------------------------------------
        tracing::info!("Phase 5/7: CLEANUP");
        let active_files: Vec<String> = scanned_files
            .iter()
            .map(|f| f.file_path.clone())
            .collect();
        let files_removed = self
            .storage
            .remove_stale_file_entities(&active_files, codebase_id)
            .await
            .context("stale removal failed")?;

        // ---------------------------------------------------------------
        // Phase 6: FTS REBUILD
        // ---------------------------------------------------------------
        tracing::info!("Phase 6/7: FTS REBUILD");
        self.storage
            .rebuild_fts_index_table(codebase_id)
            .await
            .context("FTS rebuild failed")?;

        // ---------------------------------------------------------------
        // Phase 7: HIERARCHY – build and upsert hierarchy edges
        // ---------------------------------------------------------------
        tracing::info!("Phase 7/7: HIERARCHY");
        // Get all entities from DB for hierarchy building
        let db_entities = self
            .storage
            .get_all_entity_records(codebase_id)
            .await
            .context("get all entities failed")?;

        let hierarchy_edges = build_hierarchy_edges(
            &scanned_folders,
            &scanned_files,
            &db_entities,
            codebase_id,
        );
        if !hierarchy_edges.is_empty() {
            self.storage
                .batch_upsert_edge_records(&hierarchy_edges)
                .await
                .context("hierarchy edge upsert failed")?;
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        let files_processed = changed_files.len() as u32;

        Ok(IngestionResultSummary {
            files_processed,
            entities_created,
            edges_created: edges_created + hierarchy_edges.len() as u32,
            files_skipped,
            files_removed,
            duration_ms,
        })
    }
}

/// Check if stdout is a TTY (for progress bar display).
fn atty_stdout() -> bool {
    atty_check()
}

#[cfg(unix)]
fn atty_check() -> bool {
    unsafe { libc_isatty(1) != 0 }
}

#[cfg(not(unix))]
fn atty_check() -> bool {
    false
}

#[cfg(unix)]
extern "C" {
    #[link_name = "isatty"]
    fn libc_isatty(fd: i32) -> i32;
}
