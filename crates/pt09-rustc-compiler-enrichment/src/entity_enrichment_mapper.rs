use parseltongue_core::storage::turso_storage_client::TursoStorageClient;
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichmentResult {
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub rustc_scope: String,
    pub rustc_sig: Option<String>,
    pub visibility: String,
    pub mir_calls: Vec<String>,
    pub trait_impls: Vec<String>,
}

pub async fn match_enrichment_to_entities(
    results: &[EnrichmentResult],
    store: &TursoStorageClient,
    codebase_id: i64,
) -> Result<u32> {
    if results.is_empty() {
        return Ok(0);
    }

    let mut updated = 0u32;
    for result in results {
        // Try to find matching entity by file_path and start_line
        let entity = store
            .get_entity_by_pk(
                &result.file_path,
                result.start_line as i32,
                result.end_line as i32,
                codebase_id,
            )
            .await?;

        if let Some(mut entity) = entity {
            entity.rustc_scope = Some(result.rustc_scope.clone());
            entity.rustc_sig = result.rustc_sig.clone();
            entity.visibility = Some(result.visibility.clone());
            entity.mir_calls = Some(serde_json::to_string(&result.mir_calls)?);
            entity.trait_impls = Some(serde_json::to_string(&result.trait_impls)?);
            store.batch_upsert_entity_records(&[entity]).await?;
            updated += 1;
        }
    }
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enrichment_result_creation() {
        let result = EnrichmentResult {
            file_path: "src/main.rs".into(),
            start_line: 1,
            end_line: 5,
            rustc_scope: "crate::main".into(),
            rustc_sig: Some("fn()".into()),
            visibility: "pub".into(),
            mir_calls: vec!["crate::add".into()],
            trait_impls: vec![],
        };
        assert_eq!(result.rustc_scope, "crate::main");
    }

    #[tokio::test]
    async fn test_match_empty_enrichments() {
        let store =
            TursoStorageClient::open_database_connection_path(":memory:")
                .await
                .unwrap();
        store.initialize_schema_tables_all().await.unwrap();
        let cid = store
            .get_or_create_codebase_entry("/test")
            .await
            .unwrap();

        let count = match_enrichment_to_entities(&[], &store, cid)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
