//! Integration tests for the Parseltongue storage layer.

use parseltongue_core::entity_type_definitions::*;
use parseltongue_core::storage::turso_storage_client::TursoStorageClient;

/// Helper: create a fresh in-memory client with schema initialized.
async fn fresh_client() -> TursoStorageClient {
    let client = TursoStorageClient::open_database_connection_path(":memory:")
        .await
        .expect("open :memory:");
    client
        .initialize_schema_tables_all()
        .await
        .expect("init schema");
    client
}

/// Helper: build a minimal [`CodeEntityRecord`].
fn make_entity(
    file_path: &str,
    start: i32,
    end: i32,
    name: &str,
    entity_type: EntityTypeClassification,
    codebase_id: i64,
) -> CodeEntityRecord {
    CodeEntityRecord {
        pk: EntityPrimaryKeyLocation {
            file_path: file_path.to_string(),
            start_line: start,
            end_line: end,
        },
        entity_type,
        language: "rust".to_string(),
        name: name.to_string(),
        signature: format!("fn {name}()"),
        snippet: format!("fn {name}() {{ }}"),
        doc_comment: Some(format!("doc for {name}")),
        wc: 5,
        file_hash: "abc123".to_string(),
        is_test: false,
        codebase_id,
        indexed_at: chrono::Utc::now().timestamp(),
        rustc_scope: None,
        rustc_sig: None,
        visibility: Some("pub".to_string()),
        mir_calls: None,
        trait_impls: None,
    }
}

// ======================================================================
// test_storage_crud
// ======================================================================

#[tokio::test]
async fn test_storage_crud() {
    let client = fresh_client().await;

    // get_or_create is idempotent.
    let id1 = client
        .get_or_create_codebase_entry("/tmp/my_project")
        .await
        .unwrap();
    let id2 = client
        .get_or_create_codebase_entry("/tmp/my_project")
        .await
        .unwrap();
    assert_eq!(id1, id2, "get_or_create should be idempotent");

    // Batch upsert 3 entities.
    let entities = vec![
        make_entity("src/a.rs", 1, 10, "alpha", EntityTypeClassification::Function, id1),
        make_entity("src/a.rs", 12, 20, "beta", EntityTypeClassification::Struct, id1),
        make_entity("src/b.rs", 1, 5, "gamma", EntityTypeClassification::Method, id1),
    ];
    client.batch_upsert_entity_records(&entities).await.unwrap();

    // get_all should return 3.
    let all = client.get_all_entity_records(id1).await.unwrap();
    assert_eq!(all.len(), 3);

    // get_by_pk for first entity.
    let found = client
        .get_entity_by_pk("src/a.rs", 1, 10, id1)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "alpha");

    // Statistics: 3 entities, 0 edges.
    let (ec, edgec) = client
        .get_codebase_statistics_summary(id1)
        .await
        .unwrap();
    assert_eq!(ec, 3);
    assert_eq!(edgec, 0);
}

// ======================================================================
// test_file_hash
// ======================================================================

#[tokio::test]
async fn test_file_hash() {
    let client = fresh_client().await;
    let cid = client
        .get_or_create_codebase_entry("/tmp/hash_test")
        .await
        .unwrap();

    // Initially None.
    let h = client
        .get_stored_file_hash_value("src/lib.rs", cid)
        .await
        .unwrap();
    assert!(h.is_none());

    // Store a hash.
    client
        .upsert_indexed_file_record("src/lib.rs", cid, "hash_v1")
        .await
        .unwrap();
    let h = client
        .get_stored_file_hash_value("src/lib.rs", cid)
        .await
        .unwrap();
    assert_eq!(h.as_deref(), Some("hash_v1"));

    // Update the hash.
    client
        .upsert_indexed_file_record("src/lib.rs", cid, "hash_v2")
        .await
        .unwrap();
    let h = client
        .get_stored_file_hash_value("src/lib.rs", cid)
        .await
        .unwrap();
    assert_eq!(h.as_deref(), Some("hash_v2"));
}

// ======================================================================
// test_fts_search
// ======================================================================

#[tokio::test]
async fn test_fts_search() {
    let client = fresh_client().await;
    let cid = client
        .get_or_create_codebase_entry("/tmp/fts_test")
        .await
        .unwrap();

    // 5 entities with distinct names; two contain "calculator".
    let entities = vec![
        make_entity("src/a.rs", 1, 10, "calculator_add", EntityTypeClassification::Function, cid),
        make_entity("src/a.rs", 12, 20, "calculator_sub", EntityTypeClassification::Function, cid),
        make_entity("src/b.rs", 1, 10, "parser_run", EntityTypeClassification::Function, cid),
        make_entity("src/b.rs", 12, 20, "lexer_tokenize", EntityTypeClassification::Function, cid),
        make_entity("src/c.rs", 1, 10, "evaluator_eval", EntityTypeClassification::Function, cid),
    ];
    client.batch_upsert_entity_records(&entities).await.unwrap();

    // Rebuild FTS.
    client.rebuild_fts_index_table(cid).await.unwrap();

    // Search for "calculator" – should find at least 2 (may be via FTS or LIKE fallback).
    let results = client
        .search_fts_entity_records("calculator", cid, 10)
        .await
        .unwrap();
    assert!(
        results.len() >= 2,
        "expected >= 2 results, got {}",
        results.len()
    );
}

// ======================================================================
// test_stale_removal
// ======================================================================

#[tokio::test]
async fn test_stale_removal() {
    let client = fresh_client().await;
    let cid = client
        .get_or_create_codebase_entry("/tmp/stale_test")
        .await
        .unwrap();

    // Entities across 3 files.
    let entities = vec![
        make_entity("src/a.rs", 1, 10, "fn_a", EntityTypeClassification::Function, cid),
        make_entity("src/b.rs", 1, 10, "fn_b", EntityTypeClassification::Function, cid),
        make_entity("src/c.rs", 1, 10, "fn_c", EntityTypeClassification::Function, cid),
    ];
    client.batch_upsert_entity_records(&entities).await.unwrap();

    // Also store indexed_files so we can verify they get cleaned up.
    client
        .upsert_indexed_file_record("src/a.rs", cid, "h1")
        .await
        .unwrap();
    client
        .upsert_indexed_file_record("src/b.rs", cid, "h2")
        .await
        .unwrap();
    client
        .upsert_indexed_file_record("src/c.rs", cid, "h3")
        .await
        .unwrap();

    // Keep only a.rs and b.rs → c.rs should be removed.
    let active = vec!["src/a.rs".to_string(), "src/b.rs".to_string()];
    let deleted = client
        .remove_stale_file_entities(&active, cid)
        .await
        .unwrap();
    assert_eq!(deleted, 1, "expected 1 stale entity deleted");

    // Verify c.rs entity is gone.
    let remaining = client.get_all_entity_records(cid).await.unwrap();
    assert_eq!(remaining.len(), 2);
    assert!(
        remaining.iter().all(|e| e.pk.file_path != "src/c.rs"),
        "src/c.rs should have been removed"
    );
}

// ======================================================================
// test_edge_records
// ======================================================================

#[tokio::test]
async fn test_edge_records() {
    let client = fresh_client().await;
    let cid = client
        .get_or_create_codebase_entry("/tmp/edge_test")
        .await
        .unwrap();

    let edges = vec![EdgeRecord {
        from_path: "src/a.rs".to_string(),
        from_start: 1,
        from_end: 10,
        to_path: "src/b.rs".to_string(),
        to_start: 1,
        to_end: 5,
        edge_type: "calls".to_string(),
        codebase_id: cid,
    }];

    client.batch_upsert_edge_records(&edges).await.unwrap();

    let all_edges = client.get_all_edge_records(cid).await.unwrap();
    assert_eq!(all_edges.len(), 1);
    assert_eq!(all_edges[0].from_path, "src/a.rs");
    assert_eq!(all_edges[0].edge_type, "calls");

    // Statistics should show 0 entities, 1 edge.
    let (ec, edgec) = client
        .get_codebase_statistics_summary(cid)
        .await
        .unwrap();
    assert_eq!(ec, 0);
    assert_eq!(edgec, 1);
}
