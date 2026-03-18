//! CK metrics (coupling, cohesion) endpoint.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use parseltongue_core::graph::ck_metrics_coupling_cohesion::compute_ck_metrics_suite;

use crate::shared_server_app_state::SharedServerAppState;

/// GET /coupling-cohesion-metrics-suite
pub async fn handle_ck_metrics(
    State(state): State<Arc<SharedServerAppState>>,
) -> Json<Value> {
    let metrics = compute_ck_metrics_suite(&state.graph);

    let items: Vec<Value> = metrics
        .iter()
        .map(|m| {
            json!({
                "node": m.node,
                "cbo": m.cbo,
                "lcom": m.lcom,
                "rfc": m.rfc,
                "wmc": m.wmc,
            })
        })
        .collect();

    Json(json!({
        "count": items.len(),
        "metrics": items,
    }))
}
