//! Centrality measures endpoint: PageRank or betweenness centrality.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use parseltongue_core::graph::centrality_pagerank_betweenness::{
    compute_betweenness_centrality_scores, compute_pagerank_centrality_scores,
};

use crate::shared_server_app_state::SharedServerAppState;

#[derive(Deserialize)]
pub struct CentralityParams {
    pub method: Option<String>,
    pub top: Option<usize>,
}

/// GET /centrality-measures-entity-ranking?method=pagerank|betweenness&top=20
pub async fn handle_centrality(
    State(state): State<Arc<SharedServerAppState>>,
    Query(params): Query<CentralityParams>,
) -> Json<Value> {
    let method = params.method.as_deref().unwrap_or("pagerank");
    let top = params.top.unwrap_or(50).min(500);

    let scores = match method {
        "betweenness" => compute_betweenness_centrality_scores(&state.graph),
        _ => compute_pagerank_centrality_scores(&state.graph, 0.85, 100, 1e-6),
    };

    let mut ranked: Vec<_> = scores.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    ranked.truncate(top);

    let items: Vec<Value> = ranked
        .iter()
        .enumerate()
        .map(|(rank, (node, score))| {
            json!({
                "rank": rank + 1,
                "node": node,
                "score": score,
            })
        })
        .collect();

    Json(json!({
        "method": method,
        "count": items.len(),
        "rankings": items,
    }))
}
