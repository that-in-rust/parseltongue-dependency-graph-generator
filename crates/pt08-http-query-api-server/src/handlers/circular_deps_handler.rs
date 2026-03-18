//! Circular dependency detection endpoint: SCCs with size > 1.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use parseltongue_core::graph::tarjan_scc_detection::{
    classify_scc_risk_level, detect_strongly_connected_components,
};

use crate::shared_server_app_state::SharedServerAppState;

/// GET /circular-dependency-detection-scan
pub async fn handle_circular_deps(
    State(state): State<Arc<SharedServerAppState>>,
) -> Json<Value> {
    let sccs = detect_strongly_connected_components(&state.graph);

    let cycles: Vec<Value> = sccs
        .iter()
        .filter(|scc| scc.len() > 1)
        .enumerate()
        .map(|(i, scc)| {
            json!({
                "cycle_id": i,
                "size": scc.len(),
                "risk": classify_scc_risk_level(scc.len()),
                "nodes": scc,
            })
        })
        .collect();

    let total_nodes_in_cycles: usize = cycles
        .iter()
        .filter_map(|c| c["size"].as_u64())
        .map(|s| s as usize)
        .sum();

    Json(json!({
        "cycle_count": cycles.len(),
        "total_nodes_in_cycles": total_nodes_in_cycles,
        "cycles": cycles,
    }))
}
