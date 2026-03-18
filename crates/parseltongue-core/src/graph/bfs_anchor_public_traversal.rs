//! BFS traversal through reverse edges to find the nearest public ancestor.

use std::collections::{HashMap, HashSet, VecDeque};

use super::adjacency_list_graph::AdjacencyListGraphRepresentation;

/// BFS from `start_node` through REVERSE edges (callers / parents) until a
/// public node is found.
///
/// Returns the path from `start_node` to the first public ancestor (inclusive).
///
/// - If `start_node` is already public, returns `vec![start_node]`.
/// - If no public ancestor is reachable, returns `vec![start_node]` (isolated).
pub fn find_public_anchor_via_bfs(
    graph: &AdjacencyListGraphRepresentation,
    start_node: &str,
    visibility_map: &HashMap<String, bool>,
) -> Vec<String> {
    // If start node is already public, return immediately.
    if visibility_map.get(start_node).copied().unwrap_or(false) {
        return vec![start_node.to_string()];
    }

    // Standard BFS through reverse edges.
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut parent: HashMap<String, String> = HashMap::new();

    visited.insert(start_node.to_string());
    queue.push_back(start_node.to_string());

    while let Some(current) = queue.pop_front() {
        // Traverse reverse edges (callers / parents).
        for (source, _) in graph.get_reverse_neighbor_nodes(&current) {
            if visited.contains(&source) {
                continue;
            }
            visited.insert(source.clone());
            parent.insert(source.clone(), current.clone());

            // Found a public node – reconstruct path.
            if visibility_map.get(&source).copied().unwrap_or(false) {
                return reconstruct_path(start_node, &source, &parent);
            }

            queue.push_back(source);
        }
    }

    // No public ancestor found.
    vec![start_node.to_string()]
}

/// Reconstruct the path from `start` to `end` using the `parent` map.
/// The parent map stores child -> parent (i.e. target -> source in BFS).
/// We recorded parent[neighbour] = current, so we trace from `end` back to
/// `start`.
fn reconstruct_path(
    start: &str,
    end: &str,
    parent: &HashMap<String, String>,
) -> Vec<String> {
    let mut path = vec![end.to_string()];
    let mut current = end.to_string();
    while current != start {
        if let Some(p) = parent.get(&current) {
            path.push(p.clone());
            current = p.clone();
        } else {
            break;
        }
    }
    path.reverse();
    path
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a test graph:
    ///   A -> B -> D -> E
    ///   C -> D
    ///
    /// Visibility: A=public, C=public, B=private, D=private, E=private.
    fn build_bfs_test_graph() -> (AdjacencyListGraphRepresentation, HashMap<String, bool>) {
        let mut g = AdjacencyListGraphRepresentation::new();
        g.insert_graph_edge_entry("A", "B", "calls");
        g.insert_graph_edge_entry("B", "D", "calls");
        g.insert_graph_edge_entry("C", "D", "calls");
        g.insert_graph_edge_entry("D", "E", "calls");

        let mut vis = HashMap::new();
        vis.insert("A".to_string(), true);
        vis.insert("B".to_string(), false);
        vis.insert("C".to_string(), true);
        vis.insert("D".to_string(), false);
        vis.insert("E".to_string(), false);

        (g, vis)
    }

    #[test]
    fn test_bfs_start_already_public() {
        let (g, vis) = build_bfs_test_graph();
        let path = find_public_anchor_via_bfs(&g, "A", &vis);
        assert_eq!(path, vec!["A".to_string()]);
    }

    #[test]
    fn test_bfs_find_public_ancestor() {
        let (g, vis) = build_bfs_test_graph();

        // From E: reverse edges lead to D, then D's reverse neighbours are B
        // and C. C is public. Path = E -> D -> C.
        let path = find_public_anchor_via_bfs(&g, "E", &vis);
        println!("BFS path from E: {path:?}");

        assert!(!path.is_empty());
        // Path starts at E.
        assert_eq!(path[0], "E", "Path should start at E");
        let end = path.last().unwrap().as_str();
        assert!(
            visibility_map_is_public(end, &vis),
            "Path should end at a public node, ended at {end}"
        );
    }

    #[test]
    fn test_bfs_no_public_ancestor() {
        let mut g = AdjacencyListGraphRepresentation::new();
        g.insert_graph_edge_entry("X", "Y", "calls");

        let vis: HashMap<String, bool> = [
            ("X".to_string(), false),
            ("Y".to_string(), false),
        ]
        .into_iter()
        .collect();

        let path = find_public_anchor_via_bfs(&g, "Y", &vis);
        assert_eq!(path, vec!["Y".to_string()]);
    }

    #[test]
    fn test_bfs_from_d_reaches_public() {
        let (g, vis) = build_bfs_test_graph();
        let path = find_public_anchor_via_bfs(&g, "D", &vis);
        println!("BFS path from D: {path:?}");

        assert_eq!(path[0], "D");
        let end = path.last().unwrap().as_str();
        assert!(
            visibility_map_is_public(end, &vis),
            "Path from D should reach a public node"
        );
    }

    fn visibility_map_is_public(node: &str, vis: &HashMap<String, bool>) -> bool {
        vis.get(node).copied().unwrap_or(false)
    }
}
