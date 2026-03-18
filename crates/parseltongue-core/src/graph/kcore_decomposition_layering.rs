//! K-core decomposition: iteratively remove nodes with degree < k.

use std::collections::{HashMap, HashSet};

use super::adjacency_list_graph::AdjacencyListGraphRepresentation;

/// Compute k-core decomposition layers for all nodes in the graph.
///
/// Returns a map from node identifier to its core number. The core number of a
/// node is the largest value k such that the node belongs to the k-core (the
/// maximal subgraph where every node has degree ≥ k).
///
/// Uses the Batagelj-Zaversnik bucket-queue peeling algorithm. Edges are
/// treated as undirected (union of forward and reverse) with self-loops
/// removed.
pub fn compute_kcore_decomposition_layers(
    graph: &AdjacencyListGraphRepresentation,
) -> HashMap<String, u32> {
    let nodes = graph.get_all_node_identifiers();
    if nodes.is_empty() {
        return HashMap::new();
    }

    // Build undirected neighbor sets and initial degrees.
    let mut neighbors: HashMap<String, Vec<String>> = HashMap::new();
    let mut degree: HashMap<String, u32> = HashMap::new();

    for node in &nodes {
        let mut nbr_set: HashSet<String> = HashSet::new();
        for (target, _) in graph.get_forward_neighbor_nodes(node) {
            nbr_set.insert(target);
        }
        for (source, _) in graph.get_reverse_neighbor_nodes(node) {
            nbr_set.insert(source);
        }
        nbr_set.remove(node); // remove self-loops
        let nbrs: Vec<String> = nbr_set.into_iter().collect();
        degree.insert(node.clone(), nbrs.len() as u32);
        neighbors.insert(node.clone(), nbrs);
    }

    let max_deg = degree.values().copied().max().unwrap_or(0) as usize;

    // Bucket queue: buckets[d] = set of nodes with current degree d.
    let mut buckets: Vec<HashSet<String>> = vec![HashSet::new(); max_deg + 1];
    for (node, &d) in &degree {
        buckets[d as usize].insert(node.clone());
    }

    let mut core: HashMap<String, u32> = HashMap::new();
    let mut processed: HashSet<String> = HashSet::new();
    let mut current_k: u32 = 0;

    for d in 0..=max_deg {
        while !buckets[d].is_empty() {
            let node = buckets[d].iter().next().cloned().unwrap();
            buckets[d].remove(&node);

            if processed.contains(&node) {
                continue;
            }
            processed.insert(node.clone());

            current_k = current_k.max(d as u32);
            core.insert(node.clone(), current_k);

            // Decrease degree of unprocessed neighbors.
            if let Some(nbrs) = neighbors.get(&node).cloned() {
                for nbr in nbrs {
                    if !processed.contains(&nbr) {
                        let old_d = degree[&nbr];
                        if old_d > current_k {
                            let new_d = old_d - 1;
                            *degree.get_mut(&nbr).unwrap() = new_d;
                            buckets[old_d as usize].remove(&nbr);
                            buckets[new_d as usize].insert(nbr);
                        }
                    }
                }
            }
        }
    }

    core
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::test_fixture_reference_graphs::build_reference_graph_eight_nodes;

    #[test]
    fn test_kcore_reference_graph() {
        let g = build_reference_graph_eight_nodes();
        let cores = compute_kcore_decomposition_layers(&g);

        assert_eq!(cores.len(), 8);

        // All nodes should have a core number assigned.
        for node in &["A", "B", "C", "D", "E", "F", "G", "H"] {
            assert!(
                cores.contains_key(*node),
                "Missing core number for {node}"
            );
        }

        // G-H form a tight mutual cycle (each has undirected degree 1 to each other).
        // They should be in the same core level.
        assert_eq!(
            cores["G"], cores["H"],
            "G and H (mutual cycle) should have the same core number"
        );

        // D has the highest undirected degree in the main component (4 neighbors).
        let d_core = cores["D"];
        println!("k-core values: {cores:?}");

        // The maximum core number in the graph should be reasonable (≤ max degree).
        let max_core = cores.values().copied().max().unwrap_or(0);
        assert!(
            max_core <= 4,
            "Max core ({max_core}) should not exceed max degree"
        );

        // D should have core ≤ its degree.
        assert!(d_core <= 4, "D core ({d_core}) should be ≤ 4");

        // Nodes in the main connected component (A,B,C,D,E,F) should have
        // higher core than the isolated G-H pair.
        assert!(
            d_core >= cores["G"],
            "D core ({d_core}) should be >= G core ({})",
            cores["G"]
        );
    }

    #[test]
    fn test_kcore_empty_graph() {
        let g = AdjacencyListGraphRepresentation::new();
        let cores = compute_kcore_decomposition_layers(&g);
        assert!(cores.is_empty());
    }

    #[test]
    fn test_kcore_single_node() {
        let mut g = AdjacencyListGraphRepresentation::new();
        g.insert_graph_node_entry("X");
        let cores = compute_kcore_decomposition_layers(&g);
        assert_eq!(cores.len(), 1);
        assert_eq!(cores["X"], 0);
    }
}
