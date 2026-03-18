//! PageRank and betweenness centrality algorithms for directed graphs.

use std::collections::{HashMap, VecDeque};

use super::adjacency_list_graph::AdjacencyListGraphRepresentation;

/// Compute PageRank centrality scores for all nodes.
///
/// - `damping`: damping factor (typically 0.85)
/// - `iterations`: maximum number of iterations
/// - `convergence`: stop when max change across all nodes is below this threshold
///
/// Returns a map from node identifier to its PageRank score.
pub fn compute_pagerank_centrality_scores(
    graph: &AdjacencyListGraphRepresentation,
    damping: f64,
    iterations: u32,
    convergence: f64,
) -> HashMap<String, f64> {
    let nodes = graph.get_all_node_identifiers();
    let n = nodes.len();
    if n == 0 {
        return HashMap::new();
    }

    let n_f = n as f64;
    let initial = 1.0 / n_f;

    let mut scores: HashMap<String, f64> = nodes.iter().map(|nd| (nd.clone(), initial)).collect();

    for _iter in 0..iterations {
        let mut new_scores: HashMap<String, f64> = HashMap::with_capacity(n);

        // Accumulate contributions from incoming edges.
        for node in &nodes {
            let mut incoming_sum = 0.0;
            for (source, _) in graph.get_reverse_neighbor_nodes(node) {
                let source_out = graph.get_forward_degree_count(&source);
                if source_out > 0 {
                    incoming_sum += scores[&source] / source_out as f64;
                }
            }
            let new_val = (1.0 - damping) / n_f + damping * incoming_sum;
            new_scores.insert(node.clone(), new_val);
        }

        // Check convergence.
        let max_delta = nodes
            .iter()
            .map(|nd| (new_scores[nd] - scores[nd]).abs())
            .fold(0.0_f64, f64::max);

        scores = new_scores;

        if max_delta < convergence {
            break;
        }
    }

    scores
}

/// Compute betweenness centrality scores using Brandes' algorithm for
/// unweighted directed graphs.
///
/// For each source node s, performs a BFS from s, then accumulates dependency
/// scores back through the DAG of shortest paths. Returns raw (unnormalized)
/// betweenness scores.
pub fn compute_betweenness_centrality_scores(
    graph: &AdjacencyListGraphRepresentation,
) -> HashMap<String, f64> {
    let nodes = graph.get_all_node_identifiers();
    let n = nodes.len();

    let mut centrality: HashMap<String, f64> =
        nodes.iter().map(|nd| (nd.clone(), 0.0)).collect();

    if n < 2 {
        return centrality;
    }

    for s in &nodes {
        // BFS / shortest-path DAG from s
        let mut stack: Vec<String> = Vec::new();
        let mut predecessors: HashMap<String, Vec<String>> = HashMap::new();
        let mut sigma: HashMap<String, f64> = nodes.iter().map(|nd| (nd.clone(), 0.0)).collect();
        let mut dist: HashMap<String, i64> = nodes.iter().map(|nd| (nd.clone(), -1)).collect();
        let mut delta: HashMap<String, f64> = nodes.iter().map(|nd| (nd.clone(), 0.0)).collect();

        *sigma.get_mut(s).unwrap() = 1.0;
        *dist.get_mut(s).unwrap() = 0;

        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(s.clone());

        while let Some(v) = queue.pop_front() {
            stack.push(v.clone());
            let v_dist = dist[&v];

            for (w, _) in graph.get_forward_neighbor_nodes(&v) {
                // First visit to w?
                if dist[&w] < 0 {
                    *dist.get_mut(&w).unwrap() = v_dist + 1;
                    queue.push_back(w.clone());
                }
                // Shortest path through v?
                if dist[&w] == v_dist + 1 {
                    *sigma.get_mut(&w).unwrap() += sigma[&v];
                    predecessors.entry(w.clone()).or_default().push(v.clone());
                }
            }
        }

        // Back-propagation of dependencies.
        while let Some(w) = stack.pop() {
            if let Some(preds) = predecessors.get(&w) {
                for v in preds {
                    let contribution = (sigma[v] / sigma[&w]) * (1.0 + delta[&w]);
                    *delta.get_mut(v).unwrap() += contribution;
                }
            }
            if w != *s {
                *centrality.get_mut(&w).unwrap() += delta[&w];
            }
        }
    }

    centrality
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::test_fixture_reference_graphs::build_reference_graph_eight_nodes;

    #[test]
    fn test_pagerank_reference_graph() {
        let g = build_reference_graph_eight_nodes();
        let scores = compute_pagerank_centrality_scores(&g, 0.85, 100, 1e-6);

        assert_eq!(scores.len(), 8);

        // Scores should sum to approximately 1.0.
        let total: f64 = scores.values().sum();
        assert!(
            (total - 1.0).abs() < 0.05,
            "PageRank scores should sum to ~1.0, got {total}"
        );

        // D should have a high rank (many incoming paths: B->D, C->D, F->D).
        let d_rank = scores["D"];
        let a_rank = scores["A"];
        assert!(
            d_rank > a_rank,
            "D rank ({d_rank}) should be > A rank ({a_rank})"
        );

        println!("PageRank scores: {scores:?}");
    }

    #[test]
    fn test_pagerank_empty_graph() {
        let g = AdjacencyListGraphRepresentation::new();
        let scores = compute_pagerank_centrality_scores(&g, 0.85, 100, 1e-6);
        assert!(scores.is_empty());
    }

    #[test]
    fn test_pagerank_convergence() {
        let g = build_reference_graph_eight_nodes();
        // With tight convergence, should still produce valid results.
        let scores = compute_pagerank_centrality_scores(&g, 0.85, 1000, 1e-10);
        let total: f64 = scores.values().sum();
        assert!(
            (total - 1.0).abs() < 0.01,
            "After convergence, scores should sum to ~1.0, got {total}"
        );
    }

    #[test]
    fn test_betweenness_reference_graph() {
        let g = build_reference_graph_eight_nodes();
        let bc = compute_betweenness_centrality_scores(&g);

        assert_eq!(bc.len(), 8);

        // D is a bridge node (many shortest paths go through it).
        let d_bc = bc["D"];
        let a_bc = bc["A"];
        assert!(
            d_bc > a_bc,
            "D betweenness ({d_bc}) should be > A betweenness ({a_bc})"
        );

        // Leaf-ish nodes (G, H in their own component) should have low
        // betweenness for paths within the main component.
        println!("Betweenness centrality: {bc:?}");
    }

    #[test]
    fn test_betweenness_empty_graph() {
        let g = AdjacencyListGraphRepresentation::new();
        let bc = compute_betweenness_centrality_scores(&g);
        assert!(bc.is_empty());
    }

    #[test]
    fn test_betweenness_single_node() {
        let mut g = AdjacencyListGraphRepresentation::new();
        g.insert_graph_node_entry("X");
        let bc = compute_betweenness_centrality_scores(&g);
        assert_eq!(bc.len(), 1);
        assert_eq!(bc["X"], 0.0);
    }
}
