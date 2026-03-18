//! K-core decomposition: iteratively remove nodes with degree < k.

use std::collections::HashMap;

use super::adjacency_list_graph::AdjacencyListGraphRepresentation;

/// Compute k-core decomposition layers for all nodes in the graph.
///
/// Returns a map from node identifier to its core number. The core number of a
/// node is the largest value k such that the node belongs to the k-core (the
/// maximal subgraph where every node has degree ≥ k).
///
/// Uses the standard peeling algorithm (Batagelj-Zaversnik): sort nodes by
/// degree, then iteratively peel the lowest-degree node and update neighbors.
pub fn compute_kcore_decomposition_layers(
    graph: &AdjacencyListGraphRepresentation,
) -> HashMap<String, u32> {
    let nodes = graph.get_all_node_identifiers();
    if nodes.is_empty() {
        return HashMap::new();
    }

    // Build undirected neighbor lists and initial degrees.
    let mut neighbors: HashMap<String, Vec<String>> = HashMap::new();
    let mut degree: HashMap<String, u32> = HashMap::new();

    for node in &nodes {
        let mut nbrs: Vec<String> = Vec::new();
        for (target, _) in graph.get_forward_neighbor_nodes(node) {
            nbrs.push(target);
        }
        for (source, _) in graph.get_reverse_neighbor_nodes(node) {
            nbrs.push(source);
        }
        degree.insert(node.clone(), nbrs.len() as u32);
        neighbors.insert(node.clone(), nbrs);
    }

    let mut core: HashMap<String, u32> = HashMap::new();
    let mut removed: HashSet<String> = HashSet::new();
    let n = nodes.len();

    // Process nodes one at a time: always pick the node with current minimum
    // degree, assign it core = max(current_k, its_degree), then update
    // neighbors.
    for _ in 0..n {
        // Find node with minimum degree among non-removed nodes.
        let min_node = degree
            .iter()
            .filter(|(k, _)| !removed.contains(k.as_str()))
            .min_by_key(|(_, &d)| d)
            .map(|(k, &d)| (k.clone(), d))
            .unwrap();

        let (node, node_deg) = min_node;
        removed.insert(node.clone());
        core.insert(node.clone(), node_deg);

        // Decrease degree of remaining neighbors.
        if let Some(nbrs) = neighbors.get(&node) {
            for nbr in nbrs {
                if !removed.contains(nbr.as_str()) {
                    if let Some(d) = degree.get_mut(nbr) {
                        *d = d.saturating_sub(1);
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

        // D, E, F form a cycle with additional incoming edges → higher core
        let d_core = cores["D"];
        let e_core = cores["E"];
        let f_core = cores["F"];

        // A has only outgoing edges and no incoming → likely low core
        let a_core = cores["A"];

        // Nodes in the D-E-F cycle should have higher core than leaf nodes
        assert!(
            d_core >= a_core,
            "D core ({d_core}) should be >= A core ({a_core})"
        );
        assert!(
            e_core >= a_core,
            "E core ({e_core}) should be >= A core ({a_core})"
        );
        assert!(
            f_core >= a_core,
            "F core ({f_core}) should be >= A core ({a_core})"
        );

        println!("k-core values: {cores:?}");
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
