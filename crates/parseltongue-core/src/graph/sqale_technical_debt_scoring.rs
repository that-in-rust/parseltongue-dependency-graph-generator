//! SQALE-style technical debt scoring based on graph topology and SCCs.

use std::collections::HashMap;

use super::adjacency_list_graph::AdjacencyListGraphRepresentation;

/// SQALE debt result for a single node.
#[derive(Clone, Debug)]
pub struct SqaleDebtResult {
    /// Node identifier.
    pub node: String,
    /// Numeric debt score.
    pub debt_score: f64,
    /// Rating letter: A (best) through E (worst).
    pub debt_rating: String,
}

/// Compute SQALE-style technical debt scores for every node.
///
/// Scoring rules:
///   - +10.0 per node in a cycle (SCC with size > 1)
///   - +2.0 per outgoing edge (coupling penalty)
///   - +5.0 if the node belongs to an SCC of size ≥ 3 (HIGH risk bonus)
///
/// Rating thresholds: A (0–5), B (5–15), C (15–30), D (30–50), E (50+).
///
/// Results are returned sorted by `debt_score` descending.
pub fn compute_sqale_debt_scores(
    graph: &AdjacencyListGraphRepresentation,
    sccs: &[Vec<String>],
) -> Vec<SqaleDebtResult> {
    let nodes = graph.get_all_node_identifiers();

    // Build lookup: node -> SCC size.
    let mut scc_size_map: HashMap<String, usize> = HashMap::new();
    for scc in sccs {
        let size = scc.len();
        for node in scc {
            scc_size_map.insert(node.clone(), size);
        }
    }

    let mut results: Vec<SqaleDebtResult> = Vec::with_capacity(nodes.len());

    for node in &nodes {
        let mut score: f64 = 0.0;
        let scc_size = scc_size_map.get(node).copied().unwrap_or(1);

        // +10.0 if in a cycle (SCC size > 1)
        if scc_size > 1 {
            score += 10.0;
        }

        // +2.0 per outgoing edge
        let out_degree = graph.get_forward_degree_count(node) as f64;
        score += 2.0 * out_degree;

        // +5.0 if in SCC of size >= 3 (HIGH risk)
        if scc_size >= 3 {
            score += 5.0;
        }

        let rating = classify_debt_rating(score);
        results.push(SqaleDebtResult {
            node: node.clone(),
            debt_score: score,
            debt_rating: rating,
        });
    }

    // Sort by debt_score descending, then by node name ascending for stability.
    results.sort_by(|a, b| {
        b.debt_score
            .partial_cmp(&a.debt_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.node.cmp(&b.node))
    });

    results
}

/// Classify a debt score into a letter rating.
fn classify_debt_rating(score: f64) -> String {
    if score <= 5.0 {
        "A".to_string()
    } else if score <= 15.0 {
        "B".to_string()
    } else if score <= 30.0 {
        "C".to_string()
    } else if score <= 50.0 {
        "D".to_string()
    } else {
        "E".to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::tarjan_scc_detection::detect_strongly_connected_components;
    use crate::graph::test_fixture_reference_graphs::build_reference_graph_eight_nodes;

    #[test]
    fn test_sqale_cycle_nodes_higher_debt() {
        let g = build_reference_graph_eight_nodes();
        let sccs = detect_strongly_connected_components(&g);
        let results = compute_sqale_debt_scores(&g, &sccs);

        println!("SQALE results:");
        for r in &results {
            println!("  {} -> score={}, rating={}", r.node, r.debt_score, r.debt_rating);
        }

        let debt_map: HashMap<String, f64> = results
            .iter()
            .map(|r| (r.node.clone(), r.debt_score))
            .collect();

        // D, E, F are in a cycle of size 3 -> they get +10 (cycle) + +5 (HIGH)
        // plus their outgoing edges.
        // A is NOT in a cycle -> only outgoing edge penalty.
        let a_debt = debt_map["A"];
        let d_debt = debt_map["D"];
        let e_debt = debt_map["E"];
        let f_debt = debt_map["F"];

        assert!(
            d_debt > a_debt,
            "D debt ({d_debt}) should be > A debt ({a_debt})"
        );
        assert!(
            e_debt > a_debt,
            "E debt ({e_debt}) should be > A debt ({a_debt})"
        );
        assert!(
            f_debt > a_debt,
            "F debt ({f_debt}) should be > A debt ({a_debt})"
        );
    }

    #[test]
    fn test_sqale_ratings() {
        let g = build_reference_graph_eight_nodes();
        let sccs = detect_strongly_connected_components(&g);
        let results = compute_sqale_debt_scores(&g, &sccs);

        // All results should have a valid rating.
        let valid_ratings = ["A", "B", "C", "D", "E"];
        for r in &results {
            assert!(
                valid_ratings.contains(&r.debt_rating.as_str()),
                "Invalid rating '{}' for node {}",
                r.debt_rating,
                r.node
            );
        }
    }

    #[test]
    fn test_sqale_sorted_descending() {
        let g = build_reference_graph_eight_nodes();
        let sccs = detect_strongly_connected_components(&g);
        let results = compute_sqale_debt_scores(&g, &sccs);

        for i in 1..results.len() {
            assert!(
                results[i - 1].debt_score >= results[i].debt_score,
                "Results should be sorted by debt_score descending"
            );
        }
    }

    #[test]
    fn test_sqale_empty_graph() {
        let g = AdjacencyListGraphRepresentation::new();
        let results = compute_sqale_debt_scores(&g, &[]);
        assert!(results.is_empty());
    }
}
