//! Integration tests for the PT08 HTTP Query API Server.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::util::ServiceExt;

use parseltongue_core::entity_type_definitions::{
    CodeEntityRecord, EntityPrimaryKeyLocation, EntityTypeClassification,
};
use parseltongue_core::storage::turso_storage_client::TursoStorageClient;

use pt08_http_query_api_server::route_definition_builder::build_route_definitions;
use pt08_http_query_api_server::shared_server_app_state::build_test_server_app_state;

/// Build a test state with a few entities in an in-memory database.
async fn build_test_state() -> Arc<pt08_http_query_api_server::shared_server_app_state::SharedServerAppState> {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let store = TursoStorageClient::open_database_connection_path(db_path_str)
        .await
        .unwrap();
    store.initialize_schema_tables_all().await.unwrap();
    let codebase_id = store.get_or_create_codebase_entry("test").await.unwrap();

    let entities = vec![
        CodeEntityRecord {
            pk: EntityPrimaryKeyLocation {
                file_path: "src/main.rs".to_string(),
                start_line: 1,
                end_line: 20,
            },
            entity_type: EntityTypeClassification::Function,
            language: "rust".to_string(),
            name: "main".to_string(),
            signature: "fn main()".to_string(),
            snippet: "fn main() { println!(\"hello\"); }".to_string(),
            doc_comment: Some("Entry point".to_string()),
            wc: 5,
            file_hash: "abc123".to_string(),
            is_test: false,
            codebase_id,
            indexed_at: 0,
            rustc_scope: None,
            rustc_sig: None,
            visibility: Some("pub".to_string()),
            mir_calls: None,
            trait_impls: None,
        },
        CodeEntityRecord {
            pk: EntityPrimaryKeyLocation {
                file_path: "src/lib.rs".to_string(),
                start_line: 1,
                end_line: 15,
            },
            entity_type: EntityTypeClassification::Struct,
            language: "rust".to_string(),
            name: "AppState".to_string(),
            signature: "struct AppState".to_string(),
            snippet: "struct AppState { count: u32 }".to_string(),
            doc_comment: None,
            wc: 4,
            file_hash: "def456".to_string(),
            is_test: false,
            codebase_id,
            indexed_at: 0,
            rustc_scope: None,
            rustc_sig: None,
            visibility: Some("pub".to_string()),
            mir_calls: None,
            trait_impls: None,
        },
        CodeEntityRecord {
            pk: EntityPrimaryKeyLocation {
                file_path: "src/util.rs".to_string(),
                start_line: 5,
                end_line: 25,
            },
            entity_type: EntityTypeClassification::Function,
            language: "rust".to_string(),
            name: "helper_func".to_string(),
            signature: "fn helper_func(x: i32) -> i32".to_string(),
            snippet: "fn helper_func(x: i32) -> i32 { x + 1 }".to_string(),
            doc_comment: Some("A helper function".to_string()),
            wc: 8,
            file_hash: "ghi789".to_string(),
            is_test: false,
            codebase_id,
            indexed_at: 0,
            rustc_scope: None,
            rustc_sig: None,
            visibility: Some("pub".to_string()),
            mir_calls: None,
            trait_impls: None,
        },
    ];

    store.batch_upsert_entity_records(&entities).await.unwrap();

    let edges = vec![];

    // Keep the tempdir alive by leaking it (test only)
    std::mem::forget(dir);

    build_test_server_app_state(&entities, &edges, store, codebase_id).await
}

/// Helper to parse response body as JSON.
async fn body_to_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_health_check_returns_200_ok() {
    let state = build_test_state().await;
    let app = build_route_definitions(state);

    let req = Request::builder()
        .uri("/server-health-check-status")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_to_json(resp.into_body()).await;
    assert_eq!(json["status"], "ok");
    assert_eq!(json["version"], "3.0.0");
}

#[tokio::test]
async fn test_statistics_returns_counts() {
    let state = build_test_state().await;
    let app = build_route_definitions(state);

    let req = Request::builder()
        .uri("/codebase-statistics-overview-summary")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_to_json(resp.into_body()).await;
    assert!(json.get("entity_count").is_some(), "should have entity_count");
    assert!(json.get("edge_count").is_some(), "should have edge_count");
}

#[tokio::test]
async fn test_entity_list_returns_array() {
    let state = build_test_state().await;
    let app = build_route_definitions(state);

    let req = Request::builder()
        .uri("/code-entities-list-all")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_to_json(resp.into_body()).await;
    assert!(json["entities"].is_array(), "entities should be an array");
    assert!(json["count"].as_u64().unwrap() >= 3, "should have at least 3 entities");
}

#[tokio::test]
async fn test_entity_detail_404_for_nonexistent() {
    let state = build_test_state().await;
    let app = build_route_definitions(state);

    let req = Request::builder()
        .uri("/code-entity-detail-view/nonexistent.rs:999:999")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let json = body_to_json(resp.into_body()).await;
    assert!(json["error"].as_str().unwrap().contains("not found"));
}
