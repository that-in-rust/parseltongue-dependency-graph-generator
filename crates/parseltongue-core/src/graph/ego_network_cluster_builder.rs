//! Ego-network cluster construction with token-budget awareness.

use std::collections::{HashMap, HashSet};

use super::adjacency_list_graph::AdjacencyListGraphRepresentation;

/// A 1-hop ego-network cluster centred on an anchor node.
#[derive(Clone, Debug)]
pub struct EgoNetworkCluster {
    /// The anchor / centre node.
    pub anchor: String,
    /// All entity IDs included in the cluster (including the anchor).
    pub entities: Vec<String>,
    /// Edges between included entities: `(from, to, edge_type)`.
    pub edges: Vec<(String, String, String)>,
    /// Estimated token count (`sum(wc * 1.3)` for included entities).
    pub estimated_tokens: u32,
}

/// Build a 1-hop ego-network cluster around `anchor`.
///
/// Neighbourhood = anchor + forward neighbours (callees) + reverse neighbours
/// (callers) + nodes reachable via "implements" edges.
///
/// Token estimation: `sum(entity_wc[e] * 1.3)` for each entity `e` in the
/// cluster. If the total exceeds `token_budget`, entities are prioritised:
///   1. anchor (always included)
///   2. reverse neighbours (callers)
///   3. forward neighbours (callees)
/// and truncated.
pub fn build_ego_network_cluster(
    graph: &AdjacencyListGraphRepresentation,
    anchor: &str,
    entity_wc: &HashMap<String, u32>,
    token_budget: u32,
) -> EgoNetworkCluster {
    // Collect 1-hop neighbourhood.
    let fwd = graph.get_forward_neighbor_nodes(anchor);
    let rev = graph.get_reverse_neighbor_nodes(anchor);

    let mut callers: Vec<String> = Vec::new();
    let mut callees: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(anchor.to_string());

    // Reverse neighbours (callers) – highest priority after anchor.
    for (source, _) in &rev {
        if seen.insert(source.clone()) {
            callers.push(source.clone());
        }
    }

    // Forward neighbours (callees) including impl targets.
    for (target, _) in &fwd {
        if seen.insert(target.clone()) {
            callees.push(target.clone());
        }
    }

    // Sort for determinism.
    callers.sort();
    callees.sort();

    // Token estimation helper.
    let estimate_tokens = |node: &str| -> u32 {
        let wc = entity_wc.get(node).copied().unwrap_or(0);
        (wc as f64 * 1.3).ceil() as u32
    };

    // Build ordered candidate list: anchor, callers, callees.
    let mut candidates: Vec<String> = Vec::new();
    candidates.push(anchor.to_string());
    candidates.extend(callers.iter().cloned());
    candidates.extend(callees.iter().cloned());

    // Select entities within budget.
    let mut selected: Vec<String> = Vec::new();
    let mut total_tokens: u32 = 0;

    for candidate in &candidates {
        let tokens = estimate_tokens(candidate);
        if !selected.is_empty() && total_tokens + tokens > token_budget {
            // Over budget – stop adding.
            continue;
        }
        // Always include the anchor (first candidate).
        selected.push(candidate.clone());
        total_tokens += tokens;
    }

    // Collect edges between selected entities.
    let selected_set: HashSet<&str> = selected.iter().map(|s| s.as_str()).collect();
    let mut edges: Vec<(String, String, String)> = Vec::new();

    for entity in &selected {
        for (target, edge_type) in graph.get_forward_neighbor_nodes(entity) {
            if selected_set.contains(target.as_str()) {
                edges.push((entity.clone(), target, edge_type));
            }
        }
    }

    // Sort edges for determinism.
    edges.sort();

    EgoNetworkCluster {
        anchor: anchor.to_string(),
        entities: selected,
        edges,
        estimated_tokens: total_tokens,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::test_fixture_reference_graphs::build_reference_graph_eight_nodes;

    fn build_entity_wc() -> HashMap<String, u32> {
        let mut wc = HashMap::new();
        for node in &["A", "B", "C", "D", "E", "F", "G", "H"] {
            wc.insert(node.to_string(), 100); // 100 words each
        }
        wc
    }

    #[test]
    fn test_ego_network_node_d() {
        let g = build_reference_graph_eight_nodes();
        let wc = build_entity_wc();
        // Large budget so everything fits.
        let cluster = build_ego_network_cluster(&g, "D", &wc, 100_000);

        println!("Ego cluster for D: {cluster:?}");

        // Anchor should be D.
        assert_eq!(cluster.anchor, "D");

        // D's forward neighbours: E
        // D's reverse neighbours: B, C, F
        assert!(
            cluster.entities.contains(&"D".to_string()),
            "Cluster should contain anchor D"
        );
        assert!(
            cluster.entities.contains(&"E".to_string()),
            "Cluster should contain forward neighbour E"
        );
        assert!(
            cluster.entities.contains(&"B".to_string()),
            "Cluster should contain reverse neighbour B"
        );
        assert!(
            cluster.entities.contains(&"C".to_string()),
            "Cluster should contain reverse neighbour C"
        );
        assert!(
            cluster.entities.contains(&"F".to_string()),
            "Cluster should contain reverse neighbour F"
        );
    }

    #[test]
    fn test_ego_network_budget_truncation() {
        let g = build_reference_graph_eight_nodes();
        let wc = build_entity_wc();

        // Budget = 2 tokens – should only fit the anchor (100 * 1.3 = 130)
        // Actually with budget=2 only anchor is included since 130 > 2 but
        // anchor is always included.
        let cluster = build_ego_network_cluster(&g, "D", &wc, 200);

        println!("Truncated cluster: {:?}", cluster.entities);
        assert!(
            cluster.entities.contains(&"D".to_string()),
            "Anchor should always be included"
        );
        // With budget=200 and each entity ~130 tokens, only 1 entity fits.
        assert!(
            cluster.entities.len() < 5,
            "Budget should truncate to a subset, got {} entities",
            cluster.entities.len()
        );
    }

    #[test]
    fn test_ego_network_isolated_node() {
        let mut g = AdjacencyListGraphRepresentation::new();
        g.insert_graph_node_entry("X");

        let mut wc = HashMap::new();
        wc.insert("X".to_string(), 10);

        let cluster = build_ego_network_cluster(&g, "X", &wc, 100_000);
        assert_eq!(cluster.anchor, "X");
        assert_eq!(cluster.entities, vec!["X".to_string()]);
        assert!(cluster.edges.is_empty());
    }

    #[test]
    fn test_ego_network_has_edges() {
        let g = build_reference_graph_eight_nodes();
        let wc = build_entity_wc();
        let cluster = build_ego_network_cluster(&g, "D", &wc, 100_000);

        // D->E should be an edge in the cluster.
        assert!(
            cluster.edges.contains(&("D".to_string(), "E".to_string(), "calls".to_string())),
            "Cluster should contain D->E edge"
        );
    }
}
