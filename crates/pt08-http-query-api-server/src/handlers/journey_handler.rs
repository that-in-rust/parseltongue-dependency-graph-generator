//! 7-event query journey endpoint.
//!
//! Phase 1 (no `cluster` param): preprocess query, run RRF search, take top result
//! as anchor, find public anchor via BFS, build ego-network cluster, return cluster
//! summaries with estimated tokens.
//!
//! Phase 2 (with `cluster` param): return full context for the selected cluster
//! including all entity snippets.

use std::collections::VecDeque;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use parseltongue_core::graph::ego_network_cluster_builder::build_ego_network_cluster;
use parseltongue_core::search::query_preprocessor_engine::preprocess_query_input_text;
use parseltongue_core::search::rank_fusion_combiner_rrf::combine_rrf_search_signals;

use crate::shared_server_app_state::SharedServerAppState;

#[derive(Deserialize)]
pub struct JourneyParams {
    pub q: Option<String>,
    pub cluster: Option<usize>,
}

/// GET /query?q=search_term&cluster=0
pub async fn handle_journey(
    State(state): State<Arc<SharedServerAppState>>,
    Query(params): Query<JourneyParams>,
) -> Json<Value> {
    let raw_query = match &params.q {
        Some(q) if !q.is_empty() => q.clone(),
        _ => {
            return Json(json!({
                "error": "missing required query parameter ?q=",
            }));
        }
    };

    let terms = preprocess_query_input_text(&raw_query);
    if terms.is_empty() {
        return Json(json!({
            "query": raw_query,
            "phase": "error",
            "message": "no searchable terms after preprocessing",
        }));
    }

    let search_key = terms.join(" ");
    let limit = 20usize;

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

    let rrf_results = combine_rrf_search_signals(
        &fts_results,
        &trie_results,
        &trigram_results,
        &state.git_recency,
        limit,
    );

    if rrf_results.is_empty() {
        return Json(json!({
            "query": raw_query,
            "phase": "no_results",
            "message": "no matching entities found",
        }));
    }

    // Take top result as anchor
    let top = &rrf_results[0];
    let anchor_pk = top.entity_pk.to_string();

    // Find a public anchor via BFS from the top result
    let public_anchor = find_public_anchor(&state, &anchor_pk);

    // Build ego-network clusters for top results
    let budget = 8000u32;
    let mut clusters: Vec<Value> = Vec::new();
    let mut remaining_budget = budget;
    let mut used_anchors = std::collections::HashSet::new();

    // Start with public anchor, then add more from search results
    let mut anchor_candidates = vec![public_anchor.clone()];
    for r in &rrf_results[1..] {
        let pk_str = r.entity_pk.to_string();
        let pub_anchor = find_public_anchor(&state, &pk_str);
        if !used_anchors.contains(&pub_anchor) {
            anchor_candidates.push(pub_anchor);
        }
    }

    for anchor in &anchor_candidates {
        if remaining_budget == 0 {
            break;
        }
        if !used_anchors.insert(anchor.clone()) {
            continue;
        }
        let cluster = build_ego_network_cluster(
            &state.graph,
            anchor,
            &state.entity_wc_map,
            remaining_budget,
        );
        remaining_budget = remaining_budget.saturating_sub(cluster.estimated_tokens);
        clusters.push(json!({
            "anchor": cluster.anchor,
            "entity_count": cluster.entities.len(),
            "edge_count": cluster.edges.len(),
            "estimated_tokens": cluster.estimated_tokens,
            "entities": cluster.entities,
        }));
    }

    // Phase 2: if cluster param is given, return full context for that cluster
    if let Some(cluster_idx) = params.cluster {
        if cluster_idx >= clusters.len() {
            return Json(json!({
                "query": raw_query,
                "phase": 2,
                "error": format!("cluster index {} out of range (0..{})", cluster_idx, clusters.len()),
            }));
        }

        let selected = &clusters[cluster_idx];
        let entity_pks: Vec<String> = selected["entities"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut snippets: Vec<Value> = Vec::new();
        for pk_str in &entity_pks {
            let pk: parseltongue_core::entity_type_definitions::EntityPrimaryKeyLocation =
                match pk_str.parse() {
                    Ok(pk) => pk,
                    Err(_) => continue,
                };
            if let Ok(Some(entity)) = state
                .store
                .get_entity_by_pk(&pk.file_path, pk.start_line, pk.end_line, state.codebase_id)
                .await
            {
                snippets.push(json!({
                    "pk": entity.pk.to_string(),
                    "name": entity.name,
                    "entity_type": entity.entity_type.to_string(),
                    "signature": entity.signature,
                    "snippet": entity.snippet,
                    "doc_comment": entity.doc_comment,
                }));
            }
        }

        return Json(json!({
            "query": raw_query,
            "phase": 2,
            "cluster_index": cluster_idx,
            "anchor": selected["anchor"],
            "entity_count": snippets.len(),
            "context": snippets,
        }));
    }

    // Phase 1: return cluster summaries
    Json(json!({
        "query": raw_query,
        "phase": 1,
        "terms": terms,
        "search_anchor": anchor_pk,
        "public_anchor": public_anchor,
        "token_budget": budget,
        "cluster_count": clusters.len(),
        "clusters": clusters,
        "hint": "Add &cluster=N to select a cluster and get full context",
    }))
}

/// BFS from anchor to find the first public (visible) node.
fn find_public_anchor(state: &SharedServerAppState, start: &str) -> String {
    if state.entity_visibility.get(start).copied().unwrap_or(true) {
        return start.to_string();
    }

    let mut visited = std::collections::HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(start.to_string());
    queue.push_back(start.to_string());

    while let Some(node) = queue.pop_front() {
        let neighbors = state.graph.get_forward_neighbor_nodes(&node);
        let rev_neighbors = state.graph.get_reverse_neighbor_nodes(&node);

        for (nbr, _) in neighbors.iter().chain(rev_neighbors.iter()) {
            if visited.contains(nbr) {
                continue;
            }
            if state.entity_visibility.get(nbr).copied().unwrap_or(true) {
                return nbr.clone();
            }
            visited.insert(nbr.clone());
            queue.push_back(nbr.clone());
        }
    }

    // Fallback: return original
    start.to_string()
}
