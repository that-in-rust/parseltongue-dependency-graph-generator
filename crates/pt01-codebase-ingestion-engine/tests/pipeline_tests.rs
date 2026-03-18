use pt01_codebase_ingestion_engine::ingestion_pipeline_orchestrator::IngestionPipelineOrchestrator;

#[tokio::test]
async fn test_e2e_ingest() {
    let tmp = tempfile::tempdir().unwrap();
    let db_dir = tempfile::tempdir().unwrap();
    // Create a simple Rust file
    std::fs::create_dir_all(tmp.path().join("src")).unwrap();
    std::fs::write(
        tmp.path().join("src/main.rs"),
        "fn main() {}\nfn add(a: i32, b: i32) -> i32 { a + b }",
    )
    .unwrap();

    let db_path = db_dir.path().join("test.db");
    let orchestrator = IngestionPipelineOrchestrator::new(db_path.to_str().unwrap())
        .await
        .unwrap();
    let result = orchestrator
        .run_ingestion_pipeline_phases(tmp.path())
        .await
        .unwrap();

    assert!(result.files_processed >= 1);
    assert!(result.entities_created >= 2); // at least main + add
}

#[tokio::test]
async fn test_incremental_indexing() {
    let tmp = tempfile::tempdir().unwrap();
    let db_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("src")).unwrap();
    std::fs::write(tmp.path().join("src/main.rs"), "fn main() {}").unwrap();

    let db_path = db_dir.path().join("test.db");
    let orchestrator = IngestionPipelineOrchestrator::new(db_path.to_str().unwrap())
        .await
        .unwrap();

    // First ingest
    let r1 = orchestrator
        .run_ingestion_pipeline_phases(tmp.path())
        .await
        .unwrap();
    assert!(r1.files_processed >= 1);

    // Second ingest (no changes) -> should skip
    let r2 = orchestrator
        .run_ingestion_pipeline_phases(tmp.path())
        .await
        .unwrap();
    assert_eq!(r2.files_processed, 0);
    assert!(r2.files_skipped >= 1);
}

#[tokio::test]
async fn test_stale_removal() {
    let tmp = tempfile::tempdir().unwrap();
    let db_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("src")).unwrap();
    std::fs::write(tmp.path().join("src/a.rs"), "fn a() {}").unwrap();
    std::fs::write(tmp.path().join("src/b.rs"), "fn b() {}").unwrap();

    let db_path = db_dir.path().join("test.db");
    let orchestrator = IngestionPipelineOrchestrator::new(db_path.to_str().unwrap())
        .await
        .unwrap();
    orchestrator
        .run_ingestion_pipeline_phases(tmp.path())
        .await
        .unwrap();

    // Delete one file
    std::fs::remove_file(tmp.path().join("src/b.rs")).unwrap();
    let r2 = orchestrator
        .run_ingestion_pipeline_phases(tmp.path())
        .await
        .unwrap();
    assert!(r2.files_removed >= 1);
}
