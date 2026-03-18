//! Chidamber & Kemerer (CK) object-oriented metrics adapted for code graphs.

use std::collections::HashSet;

use super::adjacency_list_graph::AdjacencyListGraphRepresentation;

/// CK metrics result for a single node.
#[derive(Clone, Debug)]
pub struct CkMetricsResult {
    /// Node identifier.
    pub node: String,
    /// Coupling Between Objects – distinct connected nodes (forward + reverse,
    /// deduplicated).
    pub cbo: u32,
    /// Lack of Cohesion of Methods – simplified metric based on shared vs
    /// unshared neighbours among a node's callees.
    pub lcom: f64,
    /// Response For a Class – 1 + count of forward "calls" edges.
    pub rfc: u32,
    /// Weighted Methods per Class – count of forward edges (proxy for method
    /// complexity).
    pub wmc: u32,
}

/// Compute CK metrics for every node in the graph.
///
/// - **CBO**: count of distinct nodes connected (forward + reverse neighbours,
///   deduplicated).
/// - **RFC**: 1 + count of forward "calls" edges.
/// - **WMC**: count of forward edges.
/// - **LCOM**: `max(0, methods_without_shared_deps − methods_with_shared_deps)`.
///   "Methods" here are the node's forward callees; two callees "share a dep" if
///   they have a common forward neighbour.
pub fn compute_ck_metrics_suite(
    graph: &AdjacencyListGraphRepresentation,
) -> Vec<CkMetricsResult> {
    let nodes = graph.get_all_node_identifiers();
    let mut results: Vec<CkMetricsResult> = Vec::with_capacity(nodes.len());

    for node in &nodes {
        let fwd = graph.get_forward_neighbor_nodes(node);
        let rev = graph.get_reverse_neighbor_nodes(node);

        // --- CBO: distinct connected nodes ---
        let mut connected: HashSet<String> = HashSet::new();
        for (target, _) in &fwd {
            connected.insert(target.clone());
        }
        for (source, _) in &rev {
            connected.insert(source.clone());
        }
        connected.remove(node);
        let cbo = connected.len() as u32;

        // --- RFC: 1 + count of forward "calls" edges ---
        let calls_count = fwd.iter().filter(|(_, et)| et == "calls").count();
        let rfc = 1 + calls_count as u32;

        // --- WMC: count of forward edges ---
        let wmc = fwd.len() as u32;

        // --- LCOM: simplified ---
        // callees are the forward neighbours (the "methods" proxy)
        let callees: Vec<String> = fwd.iter().map(|(t, _)| t.clone()).collect();
        let lcom = if callees.is_empty() {
            0.0
        } else {
            // For each pair of callees, check if they share a common forward
            // neighbour (a "shared dep").
            let callee_deps: Vec<HashSet<String>> = callees
                .iter()
                .map(|c| {
                    graph
                        .get_forward_neighbor_nodes(c)
                        .into_iter()
                        .map(|(t, _)| t)
                        .collect::<HashSet<String>>()
                })
                .collect();

            let mut pairs_shared: i64 = 0;
            let mut pairs_unshared: i64 = 0;
            for i in 0..callee_deps.len() {
                for j in (i + 1)..callee_deps.len() {
                    if callee_deps[i].intersection(&callee_deps[j]).next().is_some() {
                        pairs_shared += 1;
                    } else {
                        pairs_unshared += 1;
                    }
                }
            }

            let lcom_raw = pairs_unshared - pairs_shared;
            if lcom_raw > 0 { lcom_raw as f64 } else { 0.0 }
        };

        results.push(CkMetricsResult {
            node: node.clone(),
            cbo,
            lcom,
            rfc,
            wmc,
        });
    }

    // Sort by node name for determinism.
    results.sort_by(|a, b| a.node.cmp(&b.node));
    results
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::test_fixture_reference_graphs::build_reference_graph_eight_nodes;

    #[test]
    fn test_ck_metrics_cbo_node_a() {
        let g = build_reference_graph_eight_nodes();
        let metrics = compute_ck_metrics_suite(&g);

        let a_metrics = metrics.iter().find(|m| m.node == "A").unwrap();
        println!("A metrics: cbo={}, rfc={}, wmc={}, lcom={}", a_metrics.cbo, a_metrics.rfc, a_metrics.wmc, a_metrics.lcom);

        // A -> B, A -> C (2 outgoing edges, no incoming in reference graph)
        assert!(
            a_metrics.cbo >= 2,
            "A's CBO should be >= 2, got {}",
            a_metrics.cbo
        );
    }

    #[test]
    fn test_ck_metrics_rfc_includes_calls() {
        let g = build_reference_graph_eight_nodes();
        let metrics = compute_ck_metrics_suite(&g);

        let a_metrics = metrics.iter().find(|m| m.node == "A").unwrap();
        // A has 2 forward "calls" edges: A->B, A->C. RFC = 1 + 2 = 3.
        assert_eq!(
            a_metrics.rfc, 3,
            "A's RFC should be 3 (1 + 2 calls edges)"
        );
    }

    #[test]
    fn test_ck_metrics_wmc() {
        let g = build_reference_graph_eight_nodes();
        let metrics = compute_ck_metrics_suite(&g);

        let a_metrics = metrics.iter().find(|m| m.node == "A").unwrap();
        assert_eq!(a_metrics.wmc, 2, "A's WMC should be 2 (2 forward edges)");
    }

    #[test]
    fn test_ck_metrics_all_nodes_present() {
        let g = build_reference_graph_eight_nodes();
        let metrics = compute_ck_metrics_suite(&g);
        assert_eq!(metrics.len(), 8);
    }

    #[test]
    fn test_ck_metrics_empty_graph() {
        let g = AdjacencyListGraphRepresentation::new();
        let metrics = compute_ck_metrics_suite(&g);
        assert!(metrics.is_empty());
    }
}
