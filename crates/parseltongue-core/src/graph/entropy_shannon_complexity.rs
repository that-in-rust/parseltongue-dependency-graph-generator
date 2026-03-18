//! Shannon entropy computation over outgoing edge-type distributions.

use std::collections::HashMap;

use super::adjacency_list_graph::AdjacencyListGraphRepresentation;

/// Compute Shannon entropy scores for every node based on its outgoing edge
/// type distribution.
///
/// - Nodes with 0 edges → entropy 0.0
/// - Nodes with 1 edge type → entropy 0.0
/// - Nodes with mixed edge types → entropy > 0.0
///
/// Formula: H = −Σ pᵢ · log₂(pᵢ)
pub fn compute_shannon_entropy_scores(
    graph: &AdjacencyListGraphRepresentation,
) -> HashMap<String, f64> {
    let nodes = graph.get_all_node_identifiers();
    let mut result: HashMap<String, f64> = HashMap::with_capacity(nodes.len());

    for node in &nodes {
        let fwd = graph.get_forward_neighbor_nodes(node);
        if fwd.is_empty() {
            result.insert(node.clone(), 0.0);
            continue;
        }

        // Count frequency of each edge type.
        let mut type_counts: HashMap<&str, u32> = HashMap::new();
        for (_target, edge_type) in &fwd {
            *type_counts.entry(edge_type.as_str()).or_insert(0) += 1;
        }

        if type_counts.len() <= 1 {
            result.insert(node.clone(), 0.0);
            continue;
        }

        let total = fwd.len() as f64;
        let entropy: f64 = type_counts
            .values()
            .map(|&count| {
                let p = count as f64 / total;
                -p * p.log2()
            })
            .sum();

        result.insert(node.clone(), entropy);
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::test_fixture_reference_graphs::build_reference_graph_eight_nodes;

    #[test]
    fn test_entropy_homogeneous_edges() {
        // Reference graph uses only "calls" edges, so every node should have
        // entropy 0.0.
        let g = build_reference_graph_eight_nodes();
        let scores = compute_shannon_entropy_scores(&g);

        for (node, &h) in &scores {
            assert!(
                (h - 0.0).abs() < f64::EPSILON,
                "Node {node} should have 0 entropy for homogeneous edges, got {h}"
            );
        }
    }

    #[test]
    fn test_entropy_mixed_edges() {
        let mut g = AdjacencyListGraphRepresentation::new();
        g.insert_graph_edge_entry("A", "B", "calls");
        g.insert_graph_edge_entry("A", "C", "imports");
        g.insert_graph_edge_entry("A", "D", "calls");
        g.insert_graph_edge_entry("A", "E", "implements");

        let scores = compute_shannon_entropy_scores(&g);
        let a_entropy = scores["A"];
        println!("Entropy of A with mixed edges: {a_entropy}");

        assert!(
            a_entropy > 0.0,
            "Node A with mixed edge types should have entropy > 0.0"
        );

        // 2/4 calls, 1/4 imports, 1/4 implements
        // H = -(0.5*log2(0.5) + 0.25*log2(0.25) + 0.25*log2(0.25))
        //   = -(0.5*-1 + 0.25*-2 + 0.25*-2) = -(−0.5 − 0.5 − 0.5) = 1.5
        assert!(
            (a_entropy - 1.5).abs() < 0.01,
            "Expected entropy ≈ 1.5, got {a_entropy}"
        );
    }

    #[test]
    fn test_entropy_no_edges() {
        let mut g = AdjacencyListGraphRepresentation::new();
        g.insert_graph_node_entry("X");
        let scores = compute_shannon_entropy_scores(&g);
        assert_eq!(scores["X"], 0.0);
    }
}
