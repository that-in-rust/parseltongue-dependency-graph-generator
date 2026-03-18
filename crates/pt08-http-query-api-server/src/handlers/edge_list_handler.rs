//! Edge list endpoint: returns all dependency edges in the codebase.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::shared_server_app_state::SharedServerAppState;

/// GET /dependency-edges-list-all
pub async fn handle_edge_list(
    State(state): State<Arc<SharedServerAppState>>,
) -> Json<Value> {
    let result = state
        .store
        .get_all_edge_records(state.codebase_id)
        .await;

    match result {
        Ok(edges) => {
            let items: Vec<Value> = edges
                .iter()
                .map(|e| {
                    json!({
                        "from": format!("{}:{}:{}", e.from_path, e.from_start, e.from_end),
                        "to": format!("{}:{}:{}", e.to_path, e.to_start, e.to_end),
                        "edge_type": e.edge_type,
                    })
                })
                .collect();
            Json(json!({
                "count": items.len(),
                "edges": items,
            }))
        }
        Err(e) => Json(json!({
            "error": format!("failed to retrieve edges: {e}")
        })),
    }
}
