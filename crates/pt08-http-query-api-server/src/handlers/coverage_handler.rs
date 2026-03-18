//! Ingestion coverage folder report endpoint.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::shared_server_app_state::SharedServerAppState;

#[derive(Deserialize)]
pub struct CoverageParams {
    /// Maximum folder depth to report (default: 3).
    pub depth: Option<usize>,
}

/// GET /ingestion-coverage-folder-report?depth=3
pub async fn handle_coverage(
    State(state): State<Arc<SharedServerAppState>>,
    Query(params): Query<CoverageParams>,
) -> Json<Value> {
    let max_depth = params.depth.unwrap_or(3);

    let entities = match state
        .store
        .get_all_entity_records(state.codebase_id)
        .await
    {
        Ok(e) => e,
        Err(e) => {
            return Json(json!({ "error": format!("failed to retrieve entities: {e}") }));
        }
    };

    // Count entities per folder at each depth level.
    let mut folder_counts: HashMap<String, u32> = HashMap::new();
    let mut folder_wc: HashMap<String, u32> = HashMap::new();

    for entity in &entities {
        let parts: Vec<&str> = entity.pk.file_path.split('/').collect();
        // Build folder paths at each depth
        for depth in 1..=max_depth.min(parts.len().saturating_sub(1)) {
            let folder = parts[..depth].join("/");
            *folder_counts.entry(folder.clone()).or_insert(0) += 1;
            *folder_wc.entry(folder).or_insert(0) += entity.wc;
        }
    }

    let mut items: Vec<Value> = folder_counts
        .iter()
        .map(|(folder, &count)| {
            let wc = folder_wc.get(folder).copied().unwrap_or(0);
            json!({
                "folder": folder,
                "entity_count": count,
                "total_wc": wc,
            })
        })
        .collect();

    items.sort_by(|a, b| {
        b["entity_count"]
            .as_u64()
            .cmp(&a["entity_count"].as_u64())
            .then_with(|| a["folder"].as_str().cmp(&b["folder"].as_str()))
    });

    Json(json!({
        "max_depth": max_depth,
        "total_entities": entities.len(),
        "folders": items,
    }))
}
