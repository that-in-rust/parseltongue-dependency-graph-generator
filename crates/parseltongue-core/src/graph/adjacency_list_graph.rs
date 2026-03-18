//! Adjacency list graph representation for code entity relationship graphs.

use std::collections::{HashMap, HashSet};

use crate::entity_type_definitions::EdgeRecord;

/// An adjacency-list graph that tracks forward and reverse edges along with
/// the set of all known node identifiers.
#[derive(Clone, Debug)]
pub struct AdjacencyListGraphRepresentation {
    /// node -> [(target, edge_type)]
    pub forward: HashMap<String, Vec<(String, String)>>,
    /// node -> [(source, edge_type)]
    pub reverse: HashMap<String, Vec<(String, String)>>,
    /// All known node identifiers.
    pub nodes: HashSet<String>,
}

impl AdjacencyListGraphRepresentation {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self {
            forward: HashMap::new(),
            reverse: HashMap::new(),
            nodes: HashSet::new(),
        }
    }

    /// Insert a node into the graph (no-op if already present).
    pub fn insert_graph_node_entry(&mut self, node: &str) {
        self.nodes.insert(node.to_string());
    }

    /// Insert a directed edge from `from` to `to` with the given `edge_type`.
    /// Also inserts both endpoints as nodes.
    pub fn insert_graph_edge_entry(&mut self, from: &str, to: &str, edge_type: &str) {
        self.insert_graph_node_entry(from);
        self.insert_graph_node_entry(to);
        self.forward
            .entry(from.to_string())
            .or_default()
            .push((to.to_string(), edge_type.to_string()));
        self.reverse
            .entry(to.to_string())
            .or_default()
            .push((from.to_string(), edge_type.to_string()));
    }

    /// Return forward (outgoing) neighbors of `node` as `(target, edge_type)` pairs.
    pub fn get_forward_neighbor_nodes(&self, node: &str) -> Vec<(String, String)> {
        self.forward.get(node).cloned().unwrap_or_default()
    }

    /// Return reverse (incoming) neighbors of `node` as `(source, edge_type)` pairs.
    pub fn get_reverse_neighbor_nodes(&self, node: &str) -> Vec<(String, String)> {
        self.reverse.get(node).cloned().unwrap_or_default()
    }

    /// Out-degree of `node`.
    pub fn get_forward_degree_count(&self, node: &str) -> usize {
        self.forward.get(node).map_or(0, |v| v.len())
    }

    /// In-degree of `node`.
    pub fn get_reverse_degree_count(&self, node: &str) -> usize {
        self.reverse.get(node).map_or(0, |v| v.len())
    }

    /// All node identifiers (unordered).
    pub fn get_all_node_identifiers(&self) -> Vec<String> {
        self.nodes.iter().cloned().collect()
    }

    /// Total number of nodes.
    pub fn get_total_node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Total number of edges.
    pub fn get_total_edge_count(&self) -> usize {
        self.forward.values().map(|v| v.len()).sum()
    }

    /// Build a graph from a slice of [`EdgeRecord`]s.
    ///
    /// Node IDs are formatted as `"path:start:end"`.
    pub fn build_from_edge_records(edges: &[EdgeRecord]) -> Self {
        let mut graph = Self::new();
        for e in edges {
            let from_id = format!("{}:{}:{}", e.from_path, e.from_start, e.from_end);
            let to_id = format!("{}:{}:{}", e.to_path, e.to_start, e.to_end);
            graph.insert_graph_edge_entry(&from_id, &to_id, &e.edge_type);
        }
        graph
    }

    /// Check whether a node identifier is present in the graph.
    pub fn contains_node_identifier(&self, node: &str) -> bool {
        self.nodes.contains(node)
    }
}

impl Default for AdjacencyListGraphRepresentation {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph_zero_counts() {
        let g = AdjacencyListGraphRepresentation::new();
        assert_eq!(g.get_total_node_count(), 0);
        assert_eq!(g.get_total_edge_count(), 0);
    }

    #[test]
    fn test_insert_node_and_contains() {
        let mut g = AdjacencyListGraphRepresentation::new();
        g.insert_graph_node_entry("A");
        assert!(g.contains_node_identifier("A"));
        assert!(!g.contains_node_identifier("B"));
        assert_eq!(g.get_total_node_count(), 1);
        assert_eq!(g.get_total_edge_count(), 0);
    }

    #[test]
    fn test_insert_edge_neighbors() {
        let mut g = AdjacencyListGraphRepresentation::new();
        g.insert_graph_edge_entry("A", "B", "calls");

        // Forward: A -> [B]
        let fwd = g.get_forward_neighbor_nodes("A");
        assert_eq!(fwd.len(), 1);
        assert_eq!(fwd[0].0, "B");
        assert_eq!(fwd[0].1, "calls");

        // Reverse: B <- [A]
        let rev = g.get_reverse_neighbor_nodes("B");
        assert_eq!(rev.len(), 1);
        assert_eq!(rev[0].0, "A");
        assert_eq!(rev[0].1, "calls");

        // Degrees
        assert_eq!(g.get_forward_degree_count("A"), 1);
        assert_eq!(g.get_reverse_degree_count("B"), 1);
        assert_eq!(g.get_forward_degree_count("B"), 0);
        assert_eq!(g.get_reverse_degree_count("A"), 0);

        // Both nodes inserted
        assert!(g.contains_node_identifier("A"));
        assert!(g.contains_node_identifier("B"));
        assert_eq!(g.get_total_node_count(), 2);
        assert_eq!(g.get_total_edge_count(), 1);
    }

    #[test]
    fn test_build_from_edge_records() {
        let edges = vec![
            EdgeRecord {
                from_path: "src/a.rs".to_string(),
                from_start: 1,
                from_end: 10,
                to_path: "src/b.rs".to_string(),
                to_start: 5,
                to_end: 20,
                edge_type: "calls".to_string(),
                codebase_id: 1,
            },
            EdgeRecord {
                from_path: "src/b.rs".to_string(),
                from_start: 5,
                from_end: 20,
                to_path: "src/c.rs".to_string(),
                to_start: 30,
                to_end: 40,
                edge_type: "imports".to_string(),
                codebase_id: 1,
            },
        ];

        let g = AdjacencyListGraphRepresentation::build_from_edge_records(&edges);
        assert_eq!(g.get_total_node_count(), 3);
        assert_eq!(g.get_total_edge_count(), 2);
        assert!(g.contains_node_identifier("src/a.rs:1:10"));
        assert!(g.contains_node_identifier("src/b.rs:5:20"));
        assert!(g.contains_node_identifier("src/c.rs:30:40"));

        let fwd = g.get_forward_neighbor_nodes("src/a.rs:1:10");
        assert_eq!(fwd.len(), 1);
        assert_eq!(fwd[0].0, "src/b.rs:5:20");
    }
}
