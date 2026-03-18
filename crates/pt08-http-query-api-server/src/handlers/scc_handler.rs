//! SCC analysis endpoint: Tarjan strongly connected component detection.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use parseltongue_core::graph::tarjan_scc_detection::{
    classify_scc_risk_level, detect_strongly_connected_components,
};

use crate::shared_server_app_state::SharedServerAppState;

/// GET /strongly-connected-components-analysis
pub async fn handle_scc(
    State(state): State<Arc<SharedServerAppState>>,
) -> Json<Value> {
    let sccs = detect_strongly_connected_components(&state.graph);

    let items: Vec<Value> = sccs
        .iter()
        .enumerate()
        .map(|(i, scc)| {
            json!({
                "index": i,
                "size": scc.len(),
                "risk": classify_scc_risk_level(scc.len()),
                "nodes": scc,
            })
        })
        .collect();

    let cycles = sccs.iter().filter(|s| s.len() > 1).count();

    Json(json!({
        "total_sccs": items.len(),
        "cyclic_sccs": cycles,
        "components": items,
    }))
}
