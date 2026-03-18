//! Statistics overview endpoint: returns entity/edge counts for the codebase.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::shared_server_app_state::SharedServerAppState;

/// GET /codebase-statistics-overview-summary
pub async fn handle_statistics(
    State(state): State<Arc<SharedServerAppState>>,
) -> Json<Value> {
    let result = state
        .store
        .get_codebase_statistics_summary(state.codebase_id)
        .await;

    match result {
        Ok((entity_count, edge_count)) => Json(json!({
            "codebase_id": state.codebase_id,
            "entity_count": entity_count,
            "edge_count": edge_count,
            "graph_nodes": state.graph.get_total_node_count(),
            "graph_edges": state.graph.get_total_edge_count(),
        })),
        Err(e) => Json(json!({
            "error": format!("failed to retrieve statistics: {e}")
        })),
    }
}
