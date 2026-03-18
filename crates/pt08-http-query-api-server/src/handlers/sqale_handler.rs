//! SQALE technical debt scoring endpoint.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use parseltongue_core::graph::sqale_technical_debt_scoring::compute_sqale_debt_scores;
use parseltongue_core::graph::tarjan_scc_detection::detect_strongly_connected_components;

use crate::shared_server_app_state::SharedServerAppState;

/// GET /technical-debt-sqale-scoring
pub async fn handle_sqale(
    State(state): State<Arc<SharedServerAppState>>,
) -> Json<Value> {
    let sccs = detect_strongly_connected_components(&state.graph);
    let results = compute_sqale_debt_scores(&state.graph, &sccs);

    let items: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "node": r.node,
                "debt_score": r.debt_score,
                "debt_rating": r.debt_rating,
            })
        })
        .collect();

    // Rating distribution
    let mut dist = std::collections::HashMap::new();
    for r in &results {
        *dist.entry(r.debt_rating.clone()).or_insert(0u32) += 1;
    }

    Json(json!({
        "count": items.len(),
        "rating_distribution": dist,
        "scores": items,
    }))
}
