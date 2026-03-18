//! Smart context endpoint: builds ego-network clusters within a token budget.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use parseltongue_core::graph::centrality_pagerank_betweenness::compute_pagerank_centrality_scores;
use parseltongue_core::graph::ego_network_cluster_builder::build_ego_network_cluster;

use crate::shared_server_app_state::SharedServerAppState;

#[derive(Deserialize)]
pub struct SmartContextParams {
    /// Maximum token budget (default: 4000).
    pub tokens: Option<u32>,
    /// How many top entities to build ego networks for (default: 5).
    pub top: Option<usize>,
}

/// GET /smart-context-token-budget?tokens=4000&top=5
pub async fn handle_smart_context(
    State(state): State<Arc<SharedServerAppState>>,
    Query(params): Query<SmartContextParams>,
) -> Json<Value> {
    let token_budget = params.tokens.unwrap_or(4000);
    let top_n = params.top.unwrap_or(5).min(20);

    // Rank entities by PageRank to find the most important ones.
    let scores = compute_pagerank_centrality_scores(&state.graph, 0.85, 50, 1e-5);
    let mut ranked: Vec<_> = scores.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ranked.truncate(top_n);

    // Build ego-network clusters for top entities within the budget.
    let mut clusters: Vec<Value> = Vec::new();
    let mut remaining_budget = token_budget;

    for (node, pr_score) in &ranked {
        if remaining_budget == 0 {
            break;
        }

        let cluster = build_ego_network_cluster(
            &state.graph,
            node,
            &state.entity_wc_map,
            remaining_budget,
        );

        remaining_budget = remaining_budget.saturating_sub(cluster.estimated_tokens);

        clusters.push(json!({
            "anchor": cluster.anchor,
            "pagerank": pr_score,
            "entity_count": cluster.entities.len(),
            "edge_count": cluster.edges.len(),
            "estimated_tokens": cluster.estimated_tokens,
            "entities": cluster.entities,
        }));
    }

    let total_tokens: u32 = clusters
        .iter()
        .filter_map(|c| c["estimated_tokens"].as_u64())
        .map(|t| t as u32)
        .sum();

    Json(json!({
        "token_budget": token_budget,
        "tokens_used": total_tokens,
        "cluster_count": clusters.len(),
        "clusters": clusters,
    }))
}
