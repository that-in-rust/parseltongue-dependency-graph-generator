//! K-core decomposition endpoint.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use parseltongue_core::graph::kcore_decomposition_layering::compute_kcore_decomposition_layers;

use crate::shared_server_app_state::SharedServerAppState;

#[derive(Deserialize)]
pub struct KcoreParams {
    /// Optional: filter to nodes with core number >= k.
    pub k: Option<u32>,
}

/// GET /kcore-decomposition-layering-analysis?k=2
pub async fn handle_kcore(
    State(state): State<Arc<SharedServerAppState>>,
    Query(params): Query<KcoreParams>,
) -> Json<Value> {
    let cores = compute_kcore_decomposition_layers(&state.graph);
    let filter_k = params.k.unwrap_or(0);

    let mut items: Vec<Value> = cores
        .iter()
        .filter(|(_, &core)| core >= filter_k)
        .map(|(node, &core)| {
            json!({
                "node": node,
                "core_number": core,
            })
        })
        .collect();

    // Sort by core number descending, then node name ascending
    items.sort_by(|a, b| {
        let a_core = a["core_number"].as_u64().unwrap_or(0);
        let b_core = b["core_number"].as_u64().unwrap_or(0);
        b_core
            .cmp(&a_core)
            .then_with(|| a["node"].as_str().cmp(&b["node"].as_str()))
    });

    let max_core = cores.values().copied().max().unwrap_or(0);

    Json(json!({
        "filter_k": filter_k,
        "max_core": max_core,
        "count": items.len(),
        "nodes": items,
    }))
}
