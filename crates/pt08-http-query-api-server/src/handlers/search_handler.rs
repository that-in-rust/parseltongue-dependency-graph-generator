//! Fuzzy RRF search endpoint: combines trie, trigram, FTS, and git signals.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use parseltongue_core::search::query_preprocessor_engine::preprocess_query_input_text;
use parseltongue_core::search::rank_fusion_combiner_rrf::combine_rrf_search_signals;

use crate::shared_server_app_state::SharedServerAppState;

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
    pub limit: Option<usize>,
}

/// GET /code-entities-search-fuzzy?q=query&limit=20
pub async fn handle_fuzzy_search(
    State(state): State<Arc<SharedServerAppState>>,
    Query(params): Query<SearchParams>,
) -> Json<Value> {
    let raw_query = match &params.q {
        Some(q) if !q.is_empty() => q.clone(),
        _ => {
            return Json(json!({
                "error": "missing required query parameter ?q=",
                "results": []
            }));
        }
    };

    let limit = params.limit.unwrap_or(20).min(100);
    let terms = preprocess_query_input_text(&raw_query);

    if terms.is_empty() {
        return Json(json!({
            "query": raw_query,
            "terms": terms,
            "count": 0,
            "results": []
        }));
    }

    let search_key = terms.join(" ");

    // Trie search (exact + prefix)
    let trie_results: Vec<_> = terms
        .iter()
        .flat_map(|term| state.symbol_trie.search_exact_and_prefix(term, limit))
        .collect();

    // Trigram fuzzy search
    let trigram_results: Vec<_> = terms
        .iter()
        .flat_map(|term| state.trigram_index.search_fuzzy_trigram_match(term, limit))
        .collect();

    // FTS search
    let fts_results = match state
        .store
        .search_fts_entity_records(&search_key, state.codebase_id, limit as u32)
        .await
    {
        Ok(entities) => entities
            .into_iter()
            .enumerate()
            .map(|(i, e)| {
                let score = 1.0 - (i as f64 * 0.01);
                (e.name.clone(), e.pk.clone(), score)
            })
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };

    // Combine with RRF
    let rrf_results = combine_rrf_search_signals(
        &fts_results,
        &trie_results,
        &trigram_results,
        &state.git_recency,
        limit,
    );

    let items: Vec<Value> = rrf_results
        .iter()
        .map(|r| {
            json!({
                "pk": r.entity_pk.to_string(),
                "name": r.name,
                "score": r.score,
                "signals": r.signals,
            })
        })
        .collect();

    Json(json!({
        "query": raw_query,
        "terms": terms,
        "count": items.len(),
        "results": items,
    }))
}
