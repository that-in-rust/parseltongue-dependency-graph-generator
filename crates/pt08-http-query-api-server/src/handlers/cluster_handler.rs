//! Semantic cluster grouping endpoint: Leiden community clusters.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use parseltongue_core::graph::leiden_community_clustering::detect_leiden_community_clusters;

use crate::shared_server_app_state::SharedServerAppState;

/// GET /semantic-cluster-grouping-list
pub async fn handle_clusters(
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

    let avg_size = if items.is_empty() {
        0.0
    } else {
        communities.iter().map(|c| c.len()).sum::<usize>() as f64 / communities.len() as f64
    };

    Json(json!({
        "total_clusters": items.len(),
        "average_cluster_size": avg_size,
        "clusters": items,
    }))
}
