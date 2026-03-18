//! Tarjan's algorithm for detecting strongly connected components.

use std::collections::HashMap;

use super::adjacency_list_graph::AdjacencyListGraphRepresentation;

/// Detect all strongly connected components in the graph using Tarjan's
/// algorithm. Returns a list of SCCs sorted by size descending.
///
/// Time complexity: O(V + E)
pub fn detect_strongly_connected_components(
    graph: &AdjacencyListGraphRepresentation,
) -> Vec<Vec<String>> {
    let nodes = graph.get_all_node_identifiers();
    let mut state = TarjanState {
        index_counter: 0,
        stack: Vec::new(),
        on_stack: HashMap::new(),
        index: HashMap::new(),
        lowlink: HashMap::new(),
        result: Vec::new(),
    };

    for node in &nodes {
        if !state.index.contains_key(node.as_str()) {
            strongconnect(graph, node, &mut state);
        }
    }

    // Sort SCCs by size descending, then lexicographically by first element for stability.
    state.result.sort_by(|a, b| {
        b.len()
            .cmp(&a.len())
            .then_with(|| a[0].cmp(&b[0]))
    });

    state.result
}

/// Classify the risk level of an SCC based on its size.
pub fn classify_scc_risk_level(scc_size: usize) -> &'static str {
    match scc_size {
        0 | 1 => "NONE",
        2 => "MEDIUM",
        _ => "HIGH",
    }
}

// ---------------------------------------------------------------------------
// Internal Tarjan state & recursive helper
// ---------------------------------------------------------------------------

struct TarjanState {
    index_counter: u32,
    stack: Vec<String>,
    on_stack: HashMap<String, bool>,
    index: HashMap<String, u32>,
    lowlink: HashMap<String, u32>,
    result: Vec<Vec<String>>,
}

fn strongconnect(
    graph: &AdjacencyListGraphRepresentation,
    node: &str,
    state: &mut TarjanState,
) {
    let idx = state.index_counter;
    state.index_counter += 1;
    state.index.insert(node.to_string(), idx);
    state.lowlink.insert(node.to_string(), idx);
    state.stack.push(node.to_string());
    state.on_stack.insert(node.to_string(), true);

    // Visit forward neighbors.
    for (target, _edge_type) in graph.get_forward_neighbor_nodes(node) {
        if !state.index.contains_key(target.as_str()) {
            strongconnect(graph, &target, state);
            let target_low = state.lowlink[target.as_str()];
            let node_low = state.lowlink[node];
            if target_low < node_low {
                state.lowlink.insert(node.to_string(), target_low);
            }
        } else if *state.on_stack.get(target.as_str()).unwrap_or(&false) {
            let target_idx = state.index[target.as_str()];
            let node_low = state.lowlink[node];
            if target_idx < node_low {
                state.lowlink.insert(node.to_string(), target_idx);
            }
        }
    }

    // If node is a root node, pop the stack and generate an SCC.
    if state.lowlink[node] == state.index[node] {
        let mut component = Vec::new();
        loop {
            let w = state.stack.pop().expect("stack should not be empty");
            state.on_stack.insert(w.clone(), false);
            component.push(w.clone());
            if w == node {
                break;
            }
        }
        component.sort();
        state.result.push(component);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::test_fixture_reference_graphs::build_reference_graph_eight_nodes;

    #[test]
    fn test_tarjan_reference_graph() {
        let g = build_reference_graph_eight_nodes();
        let sccs = detect_strongly_connected_components(&g);

        // Expect 5 SCCs: {D,E,F} (3), {G,H} (2), {A} (1), {B} (1), {C} (1)
        assert_eq!(sccs.len(), 5);

        // First SCC should be the largest (size 3): D, E, F
        assert_eq!(sccs[0].len(), 3);
        assert!(sccs[0].contains(&"D".to_string()));
        assert!(sccs[0].contains(&"E".to_string()));
        assert!(sccs[0].contains(&"F".to_string()));

        // Second SCC should be size 2: G, H
        assert_eq!(sccs[1].len(), 2);
        assert!(sccs[1].contains(&"G".to_string()));
        assert!(sccs[1].contains(&"H".to_string()));

        // Remaining 3 SCCs should each be size 1
        for scc in &sccs[2..] {
            assert_eq!(scc.len(), 1);
        }
    }

    #[test]
    fn test_tarjan_empty_graph() {
        let g = AdjacencyListGraphRepresentation::new();
        let sccs = detect_strongly_connected_components(&g);
        assert_eq!(sccs.len(), 0);
    }

    #[test]
    fn test_tarjan_linear_chain() {
        let mut g = AdjacencyListGraphRepresentation::new();
        g.insert_graph_edge_entry("A", "B", "calls");
        g.insert_graph_edge_entry("B", "C", "calls");
        g.insert_graph_edge_entry("C", "D", "calls");

        let sccs = detect_strongly_connected_components(&g);
        assert_eq!(sccs.len(), 4);
        for scc in &sccs {
            assert_eq!(scc.len(), 1);
        }
    }

    #[test]
    fn test_classify_scc_risk_level() {
        assert_eq!(classify_scc_risk_level(1), "NONE");
        assert_eq!(classify_scc_risk_level(2), "MEDIUM");
        assert_eq!(classify_scc_risk_level(3), "HIGH");
        assert_eq!(classify_scc_risk_level(10), "HIGH");
    }
}
