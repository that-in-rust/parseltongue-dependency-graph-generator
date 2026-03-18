//! Complexity hotspots ranking endpoint.

use std::collections::HashSet;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use parseltongue_core::graph::tarjan_scc_detection::detect_strongly_connected_components;

use crate::shared_server_app_state::SharedServerAppState;

#[derive(Deserialize)]
pub struct HotspotsParams {
    /// Number of top hotspots to return (default: 20).
    pub top: Option<usize>,
}

/// GET /complexity-hotspots-ranking-view?top=20
///
/// Ranks entities by: edge_count (forward + reverse) + cycle membership bonus.
pub async fn handle_hotspots(
    State(state): State<Arc<SharedServerAppState>>,
    Query(params): Query<HotspotsParams>,
) -> Json<Value> {
    let top_n = params.top.unwrap_or(20).min(200);

    // Find nodes in cycles (SCCs with size > 1)
    let sccs = detect_strongly_connected_components(&state.graph);
    let cycle_nodes: HashSet<String> = sccs
        .iter()
        .filter(|scc| scc.len() > 1)
        .flat_map(|scc| scc.iter().cloned())
        .collect();

    let nodes = state.graph.get_all_node_identifiers();
    let mut scored: Vec<(String, f64)> = nodes
        .iter()
        .map(|node| {
            let fwd = state.graph.get_forward_degree_count(node) as f64;
            let rev = state.graph.get_reverse_degree_count(node) as f64;
            let edge_score = fwd + rev;
            let cycle_bonus = if cycle_nodes.contains(node) {
                10.0
            } else {
                0.0
            };
            (node.clone(), edge_score + cycle_bonus)
        })
        .collect();

    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    scored.truncate(top_n);

    let items: Vec<Value> = scored
        .iter()
        .enumerate()
        .map(|(rank, (node, score))| {
            json!({
                "rank": rank + 1,
                "node": node,
                "hotspot_score": score,
                "in_cycle": cycle_nodes.contains(node),
            })
        })
        .collect();

    Json(json!({
        "top": top_n,
        "count": items.len(),
        "hotspots": items,
    }))
}
