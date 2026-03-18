//! Reference graphs used for testing graph algorithms.

use super::adjacency_list_graph::AdjacencyListGraphRepresentation;

/// Build a reference graph with 8 nodes and several edges including cycles.
///
/// ```text
/// A -> B -> D -> E -> F
/// A -> C -> D        ^
///           |        |
///           +-- F ---+ (cycle D-E-F)
///
/// G <-> H (cycle G-H)
/// ```
///
/// Nodes: A, B, C, D, E, F, G, H
/// Edges (all "calls"):
///   A->B, A->C, B->D, C->D, D->E, E->F, F->D, G->H, H->G
pub fn build_reference_graph_eight_nodes() -> AdjacencyListGraphRepresentation {
    let mut g = AdjacencyListGraphRepresentation::new();
    g.insert_graph_edge_entry("A", "B", "calls");
    g.insert_graph_edge_entry("A", "C", "calls");
    g.insert_graph_edge_entry("B", "D", "calls");
    g.insert_graph_edge_entry("C", "D", "calls");
    g.insert_graph_edge_entry("D", "E", "calls");
    g.insert_graph_edge_entry("E", "F", "calls");
    g.insert_graph_edge_entry("F", "D", "calls"); // cycle D-E-F
    g.insert_graph_edge_entry("G", "H", "calls");
    g.insert_graph_edge_entry("H", "G", "calls"); // cycle G-H
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_graph_structure() {
        let g = build_reference_graph_eight_nodes();
        assert_eq!(g.get_total_node_count(), 8);
        assert_eq!(g.get_total_edge_count(), 9);
        assert!(g.contains_node_identifier("A"));
        assert!(g.contains_node_identifier("H"));
    }
}
