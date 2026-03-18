//! Leiden community detection endpoint.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use parseltongue_core::graph::leiden_community_clustering::detect_leiden_community_clusters;

use crate::shared_server_app_state::SharedServerAppState;

/// GET /leiden-community-detection-clusters
pub async fn handle_leiden(
    State(state): State<Arc<SharedServerAppState>>,
) -> Json<Value> {
    let communities = detect_leiden_community_clusters(&state.graph);

    let items: Vec<Value> = communities
        .iter()
        .enumerate()
        .map(|(i, c)| {
            json!({
                "cluster_id": i,
                "size": c.len(),
                "members": c,
            })
        })
        .collect();

    Json(json!({
        "total_clusters": items.len(),
        "clusters": items,
    }))
}
