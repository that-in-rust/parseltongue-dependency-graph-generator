use anyhow::Result;
use clap::Parser;
use pt01_codebase_ingestion_engine::cli_argument_parser_setup::{CliArguments, CliCommand};
use pt01_codebase_ingestion_engine::ingestion_pipeline_orchestrator::IngestionPipelineOrchestrator;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = CliArguments::parse();

    match args.command {
        CliCommand::Ingest {
            path,
            db,
            verbose: _,
        } => {
            let root = Path::new(&path).canonicalize()?;
            let db_path = if db == ".parseltongue/index.db" {
                root.join(".parseltongue").join("index.db")
            } else {
                Path::new(&db).to_path_buf()
            };
            // Create .parseltongue dir if needed
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let orchestrator =
                IngestionPipelineOrchestrator::new(db_path.to_str().unwrap()).await?;
            let result = orchestrator.run_ingestion_pipeline_phases(&root).await?;
            println!(
                "Ingestion complete: {} files processed, {} entities, {} edges, {} skipped, {} removed, {}ms",
                result.files_processed,
                result.entities_created,
                result.edges_created,
                result.files_skipped,
                result.files_removed,
                result.duration_ms
            );
        }
        CliCommand::Serve { db, port } => {
            let state =
                pt08_http_query_api_server::shared_server_app_state::build_server_app_state(&db)
                    .await?;
            pt08_http_query_api_server::http_server_startup_runner::start_http_server_listener(
                state, port,
            )
            .await?;
        }
        CliCommand::Enrich { path: _, db: _ } => {
            println!("Enrichment not yet implemented");
        }
    }
    Ok(())
}
