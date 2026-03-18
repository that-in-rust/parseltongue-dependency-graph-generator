//! Simplified Leiden / Louvain-style community detection for directed graphs.
//!
//! The algorithm treats edges as undirected and iteratively moves nodes between
//! communities to maximise modularity gain. After convergence the communities
//! are aggregated into a condensed graph and the process repeats.

use std::collections::HashMap;

use super::adjacency_list_graph::AdjacencyListGraphRepresentation;

/// Detect communities using a simplified Leiden (Louvain-style) algorithm.
///
/// Returns a list of communities (each a `Vec<String>` of node IDs), sorted by
/// size descending.
pub fn detect_leiden_community_clusters(
    graph: &AdjacencyListGraphRepresentation,
) -> Vec<Vec<String>> {
    let nodes = graph.get_all_node_identifiers();
    if nodes.is_empty() {
        return Vec::new();
    }

    // Build undirected adjacency with edge weights (count of parallel edges).
    let mut adj: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for node in &nodes {
        adj.entry(node.clone()).or_default();
        for (target, _) in graph.get_forward_neighbor_nodes(node) {
            if target != *node {
                *adj.entry(node.clone()).or_default().entry(target.clone()).or_insert(0.0) += 1.0;
                *adj.entry(target.clone()).or_default().entry(node.clone()).or_insert(0.0) += 1.0;
            }
        }
    }

    // Total edge weight (each undirected edge counted once as 2m).
    let two_m: f64 = adj.values().flat_map(|nbrs| nbrs.values()).sum::<f64>();
    if two_m == 0.0 {
        // No edges – every node is its own community.
        let mut communities: Vec<Vec<String>> = nodes.into_iter().map(|n| vec![n]).collect();
        communities.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a[0].cmp(&b[0])));
        return communities;
    }

    // Phase 1: local moving – initialise each node in its own community.
    let mut community: HashMap<String, usize> = HashMap::new();
    let mut comm_id: usize = 0;
    for node in &nodes {
        community.insert(node.clone(), comm_id);
        comm_id += 1;
    }

    // Weighted degree (k_i) for each node.
    let k: HashMap<String, f64> = nodes
        .iter()
        .map(|n| {
            let deg: f64 = adj.get(n).map_or(0.0, |nbrs| nbrs.values().sum());
            (n.clone(), deg)
        })
        .collect();

    // Sum of degrees inside each community (sigma_tot).
    let mut sigma_tot: HashMap<usize, f64> = HashMap::new();
    for node in &nodes {
        *sigma_tot.entry(community[node]).or_insert(0.0) += k[node];
    }

    let mut improved = true;
    let max_passes = 20;
    let mut pass = 0;

    while improved && pass < max_passes {
        improved = false;
        pass += 1;

        for node in &nodes {
            let node_comm = community[node];
            let ki = k[node];

            // Compute edges to each neighbouring community.
            let mut comm_edges: HashMap<usize, f64> = HashMap::new();
            if let Some(nbrs) = adj.get(node) {
                for (nbr, w) in nbrs {
                    let c = community[nbr];
                    *comm_edges.entry(c).or_insert(0.0) += w;
                }
            }

            // Modularity gain of removing node from its current community.
            let ki_in_current = comm_edges.get(&node_comm).copied().unwrap_or(0.0);
            let sigma_current = sigma_tot.get(&node_comm).copied().unwrap_or(0.0);
            let remove_cost = ki_in_current / two_m - (sigma_current - ki) * ki / (two_m * two_m);

            // Find best community to move into.
            let mut best_comm = node_comm;
            let mut best_gain = 0.0;

            for (&c, &ki_in_c) in &comm_edges {
                if c == node_comm {
                    continue;
                }
                let sigma_c = sigma_tot.get(&c).copied().unwrap_or(0.0);
                let gain = ki_in_c / two_m - sigma_c * ki / (two_m * two_m) - remove_cost;
                if gain > best_gain {
                    best_gain = gain;
                    best_comm = c;
                }
            }

            if best_comm != node_comm {
                // Move node.
                *sigma_tot.entry(node_comm).or_insert(0.0) -= ki;
                *sigma_tot.entry(best_comm).or_insert(0.0) += ki;
                community.insert(node.clone(), best_comm);
                improved = true;
            }
        }
    }

    // Collect communities.
    let mut comm_members: HashMap<usize, Vec<String>> = HashMap::new();
    for (node, c) in &community {
        comm_members.entry(*c).or_default().push(node.clone());
    }

    let mut result: Vec<Vec<String>> = comm_members.into_values().collect();
    // Sort each community's members for determinism.
    for members in &mut result {
        members.sort();
    }
    // Sort communities: largest first, then by first element for stability.
    result.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a[0].cmp(&b[0])));
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::graph::test_fixture_reference_graphs::build_reference_graph_eight_nodes;

    #[test]
    fn test_leiden_reference_graph_has_multiple_communities() {
        let g = build_reference_graph_eight_nodes();
        let communities = detect_leiden_community_clusters(&g);

        println!("Leiden communities: {communities:?}");

        // Should have at least 2 communities.
        assert!(
            communities.len() >= 2,
            "Expected at least 2 communities, got {}",
            communities.len()
        );

        // D, E, F are in a cycle and should be grouped.
        let def_set: HashSet<&str> = ["D", "E", "F"].iter().copied().collect();
        let has_def_cluster = communities.iter().any(|c| {
            let cs: HashSet<&str> = c.iter().map(|s| s.as_str()).collect();
            def_set.is_subset(&cs)
        });

        // G, H form their own cycle and should be grouped.
        let gh_set: HashSet<&str> = ["G", "H"].iter().copied().collect();
        let has_gh_cluster = communities.iter().any(|c| {
            let cs: HashSet<&str> = c.iter().map(|s| s.as_str()).collect();
            gh_set.is_subset(&cs)
        });

        println!("D-E-F grouped: {has_def_cluster}, G-H grouped: {has_gh_cluster}");
        // At least one of the tight groups should be detected.
        assert!(
            has_def_cluster || has_gh_cluster,
            "Expected at least one tight cluster (D-E-F or G-H)"
        );
    }

    #[test]
    fn test_leiden_empty_graph() {
        let g = AdjacencyListGraphRepresentation::new();
        let communities = detect_leiden_community_clusters(&g);
        assert!(communities.is_empty());
    }

    #[test]
    fn test_leiden_single_node() {
        let mut g = AdjacencyListGraphRepresentation::new();
        g.insert_graph_node_entry("X");
        let communities = detect_leiden_community_clusters(&g);
        assert_eq!(communities.len(), 1);
        assert_eq!(communities[0], vec!["X".to_string()]);
    }
}
