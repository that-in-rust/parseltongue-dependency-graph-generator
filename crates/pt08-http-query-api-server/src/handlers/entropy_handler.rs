//! Shannon entropy complexity scores endpoint.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use parseltongue_core::graph::entropy_shannon_complexity::compute_shannon_entropy_scores;

use crate::shared_server_app_state::SharedServerAppState;

/// GET /entropy-complexity-measurement-scores
pub async fn handle_entropy(
    State(state): State<Arc<SharedServerAppState>>,
) -> Json<Value> {
    let scores = compute_shannon_entropy_scores(&state.graph);

    let mut items: Vec<Value> = scores
        .iter()
        .map(|(node, &entropy)| {
            json!({
                "node": node,
                "entropy": entropy,
            })
        })
        .collect();

    // Sort by entropy descending, then node name ascending
    items.sort_by(|a, b| {
        let a_e = a["entropy"].as_f64().unwrap_or(0.0);
        let b_e = b["entropy"].as_f64().unwrap_or(0.0);
        b_e.partial_cmp(&a_e)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a["node"].as_str().cmp(&b["node"].as_str()))
    });

    Json(json!({
        "count": items.len(),
        "scores": items,
    }))
}
