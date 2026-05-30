

## Practical Rust implementation: SGL0.5 baseline (CPU-only, pluggable)

Below is a minimal, production-lean baseline that you can drop into a new crate (e.g., crates/pt07-analytics/src/sgl05.rs). It:

-  Builds a weighted graph from ISG and dependency data.
-  Computes communities with Label Propagation (LPA).
-  Refines clusters to fit token budgets and min/max sizes.
-  Labels clusters automatically.
-  Exports clusters.json, cluster_edges.json, cluster_assignments.json.
-  Optionally persists results to CozoDB.

Add to Cargo.toml:
```toml
[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
petgraph = "0.6"
itertools = "0.12"
rayon = "1.10"
regex = "1"
fnv = "1.0"
```

File: crates/pt07-analytics/src/sgl05.rs
```rust
use anyhow::{anyhow, Result};
use fnv::FnvHashMap as FastMap;
use fnv::FnvHashSet as FastSet;
use itertools::Itertools;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::prelude::UnGraph;
use petgraph::visit::EdgeRef;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// --- Input models (adapt to your core types) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub isgl1_key: String,
    pub language: String,
    pub file_path: String,
    pub name: String,
    pub lines_of_code: Option<usize>,
    pub signature_tokens: Option<usize>, // if PT02 produced token counts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepEdge {
    pub from_key: String,
    pub to_key: String,
    pub edge_type: String,      // e.g., "Calls", "UsesType", "Reads", "Writes"
    pub frequency: Option<f32>, // optional count or call frequency
}

/// Optional auxiliary signals; fill if you have them, else leave empty.
#[derive(Debug, Clone, Default)]
pub struct TemporalSignal {
    /// pairs and their co-change score (0..1)
    pub co_change: FastMap<(String, String), f32>,
}

#[derive(Debug, Clone, Default)]
pub struct DataFlowSignal {
    /// pairs and their data affinity (0..1) (shared types, compatible shapes)
    pub data_affinity: FastMap<(String, String), f32>,
}

#[derive(Debug, Clone, Default)]
pub struct SemanticSignal {
    /// pairs and their signature/name similarity (0..1)
    pub semantic_sim: FastMap<(String, String), f32>,
}

/// Parameters for edge weighting (tunable)
#[derive(Debug, Clone)]
pub struct WeightParams {
    pub alpha_dep: f32,     // dependency
    pub beta_data: f32,     // data-flow
    pub gamma_temp: f32,    // temporal
    pub delta_sem: f32,     // semantic
    pub max_edge_weight: f32,
}

impl Default for WeightParams {
    fn default() -> Self {
        Self {
            alpha_dep: 1.0,
            beta_data: 0.8,
            gamma_temp: 0.6,
            delta_sem: 0.4,
            max_edge_weight: 10.0,
        }
    }
}

/// Clustering constraints for LLM-friendly packs
#[derive(Debug, Clone)]
pub struct ClusterBudget {
    pub min_fun: usize,
    pub max_fun: usize,
    pub min_tokens: usize,
    pub max_tokens: usize,
}

/// --- Output models (JSON exports) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    pub cluster_id: String,
    pub cluster_name: String,
    pub level: f32, // 0.5
    pub contains: Vec<String>, // ISGL1 keys
    pub metrics: ClusterMetrics,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMetrics {
    pub cohesion: f32,          // internal_weight / (internal_weight + external_weight)
    pub coupling: f32,          // external_weight / (internal + external)
    pub modularity_local: f32,  // approximate per-cluster modularity contribution
    pub token_estimate: usize,  // rough estimate (for LLM budget)
    pub blast_radius: usize,    // number of external dependents
    pub centrality: f32,        // simple degree-based centrality proxy
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterEdge {
    pub from_cluster: String,
    pub to_cluster: String,
    pub weights: ClusterEdgeWeights,
    pub boundary_crossings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterEdgeWeights {
    pub control: f32,   // sum of call weights
    pub data: f32,      // sum of data-flow weights
    pub temporal: f32,  // sum of co-change weights
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClustersExport {
    pub level: String,            // "SGL0.5"
    pub modularity_global: f32,   // coarse estimate
    pub clusters: Vec<ClusterNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterEdgesExport {
    pub level: String, // "SGL0.5"
    pub edges: Vec<ClusterEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterAssignment {
    pub isgl1_key: String,
    pub cluster_id: String,
    pub membership_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterAssignmentsExport {
    pub assignments: Vec<ClusterAssignment>,
}

/// --- Core SGL0.5 pipeline ---

pub struct Sgl05 {
    weight_params: WeightParams,
    budget: ClusterBudget,
}

impl Sgl05 {
    pub fn new(weight_params: WeightParams, budget: ClusterBudget) -> Self {
        Self { weight_params, budget }
    }

    /// Entry-point: build clusters and export JSONs
    pub fn run(
        &self,
        entities: &[Entity],
        dep_edges: &[DepEdge],
        temporal: &TemporalSignal,
        dataflow: &DataFlowSignal,
        semantic: &SemanticSignal,
        out_dir: &Path,
    ) -> Result<()> {
        let (graph, key2ix, ix2key) =
            build_graph(entities, dep_edges, temporal, dataflow, semantic, &self.weight_params)?;

        let labels = label_propagation(&graph, 25)?; // max 25 iters
        let mut clusters = labels_to_clusters(&labels, &ix2key);
        enforce_budgets(
            &graph,
            &ix2key,
            &mut clusters,
            &self.budget,
            &self.weight_params,
        )?;

        let cluster_nodes = compute_cluster_nodes(
            &graph,
            &ix2key,
            &clusters,
            entities,
            dep_edges,
            &self.weight_params,
        )?;

        let cluster_edges = compute_cluster_edges(
            &graph,
            &ix2key,
            &clusters,
            dep_edges,
            temporal,
            dataflow,
        )?;

        let assignments = compute_assignments(&clusters);

        // Exports
        let modularity_global = approx_modularity(&graph, &clusters);
        let clusters_export = ClustersExport {
            level: "SGL0.5".to_string(),
            modularity_global,
            clusters: cluster_nodes,
        };
        let edges_export = ClusterEdgesExport {
            level: "SGL0.5".to_string(),
            edges: cluster_edges,
        };
        let assignments_export = ClusterAssignmentsExport { assignments };

        fs::create_dir_all(out_dir)?;
        write_json(out_dir.join("clusters.json"), &clusters_export)?;
        write_json(out_dir.join("cluster_edges.json"), &edges_export)?;
        write_json(out_dir.join("cluster_assignments.json"), &assignments_export)?;
        Ok(())
    }
}

/// Build undirected weighted graph with composite weights.
/// Node = function/entity (ISGL1 key), Edge weight = α·dep + β·data + γ·temp + δ·semantic
fn build_graph(
    entities: &[Entity],
    deps: &[DepEdge],
    temporal: &TemporalSignal,
    dataflow: &DataFlowSignal,
    semantic: &SemanticSignal,
    wp: &WeightParams,
) -> Result<(UnGraph<String, f32>, FastMap<String, NodeIndex>, Vec<String>)> {
    let mut key2ix: FastMap<String, NodeIndex> = FastMap::default();
    let mut graph: UnGraph<String, f32> = Graph::default();

    // add nodes
    for e in entities {
        let ix = graph.add_node(e.isgl1_key.clone());
        key2ix.insert(e.isgl1_key.clone(), ix);
    }

    let mut edge_acc: FastMap<(NodeIndex, NodeIndex), (f32, f32, f32, f32)> = FastMap::default();

    let dep_weight = |edge: &DepEdge| -> f32 {
        match edge.edge_type.as_str() {
            "Calls" => edge.frequency.unwrap_or(1.0),
            "UsesType" | "Reads" | "Writes" => 0.5 * edge.frequency.unwrap_or(1.0),
            _ => 0.2 * edge.frequency.unwrap_or(1.0),
        }
    };

    for d in deps {
        let (Some(a), Some(b)) = (key2ix.get(&d.from_key), key2ix.get(&d.to_key)) else { continue };
        let mut pair = (*a, *b);
        if pair.0 > pair.1 {
            pair = (pair.1, pair.0);
        }
        let w_dep = dep_weight(d);

        let t_key = (d.from_key.clone(), d.to_key.clone());
        let w_temp = *temporal.co_change.get(&t_key).unwrap_or(&0.0);

        let df_key = (d.from_key.clone(), d.to_key.clone());
        let w_data = *dataflow.data_affinity.get(&df_key).unwrap_or(&0.0);

        let sem_key = (d.from_key.clone(), d.to_key.clone());
        let w_sem = *semantic.semantic_sim.get(&sem_key).unwrap_or(&0.0);

        let entry = edge_acc.entry(pair).or_insert((0.0, 0.0, 0.0, 0.0));
        entry.0 += w_dep;
        entry.1 += w_data;
        entry.2 += w_temp;
        entry.3 += w_sem;
    }

    for (pair, (w_dep, w_data, w_temp, w_sem)) in edge_acc {
        let w = (wp.alpha_dep * w_dep)
            + (wp.beta_data * w_data)
            + (wp.gamma_temp * w_temp)
            + (wp.delta_sem * w_sem);
        let w = w.min(wp.max_edge_weight);
        if w > 0.0 {
            graph.add_edge(pair.0, pair.1, w);
        }
    }

    let ix2key: Vec<String> = (0..graph.node_count())
        .map(|i| graph.node_weight(NodeIndex::new(i)).unwrap().clone())
        .collect();

    Ok((graph, key2ix, ix2key))
}

/// Label Propagation Algorithm (LPA) – simple, fast community detection.
fn label_propagation(graph: &UnGraph<String, f32>, max_iters: usize) -> Result<Vec<usize>> {
    let n = graph.node_count();
    if n == 0 {
        return Ok(vec![]);
    }
    // initial: each node is its own label
    let mut label: Vec<usize> = (0..n).collect();

    for _ in 0..max_iters {
        let mut changed = false;

        for i in 0..n {
            let ni = NodeIndex::new(i);
            let mut scores: FastMap<usize, f32> = FastMap::default();

            for e in graph.edges(ni) {
                let j = e.target().index();
                let w = *e.weight();
                let lj = label[j];
                *scores.entry(lj).or_insert(0.0) += w;
            }

            if scores.is_empty() {
                continue;
            }

            // pick label with max score; tie-break by smallest id for determinism
            let (&best_label, _) = scores
                .iter()
                .max_by(|(la, wa), (lb, wb)| {
                    wa.partial_cmp(wb)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| la.cmp(lb))
                })
                .unwrap();

            if best_label != label[i] {
                label[i] = best_label;
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }
    Ok(label)
}

/// Convert labels to clusters: label-id → members (ISGL1 keys)
fn labels_to_clusters(labels: &[usize], ix2key: &[String]) -> HashMap<String, FastSet<String>> {
    let mut map: HashMap<usize, FastSet<String>> = HashMap::new();
    for (i, &lab) in labels.iter().enumerate() {
        map.entry(lab).or_default().insert(ix2key[i].clone());
    }
    // normalize to c_1..c_k
    let mut out: HashMap<String, FastSet<String>> = HashMap::new();
    for (idx, (_, members)) in map.into_iter().sorted_by_key(|(k, _)| *k).enumerate() {
        out.insert(format!("c_{}", idx + 1), members);
    }
    out
}

/// Enforce token and size budgets by splitting too-large clusters (re-run LPA on subgraph) and merging too-small clusters into strongest neighbor.
fn enforce_budgets(
    graph: &UnGraph<String, f32>,
    ix2key: &[String],
    clusters: &mut HashMap<String, FastSet<String>>,
    budget: &ClusterBudget,
    wp: &WeightParams,
) -> Result<()> {
    // index back from key → node index
    let mut key2ix: FastMap<String, NodeIndex> = FastMap::default();
    for (i, k) in ix2key.iter().enumerate() {
        key2ix.insert(k.clone(), NodeIndex::new(i));
    }

    // helper to estimate tokens
    let estimate_tokens = |members: &FastSet<String>| -> usize {
        // Fallback: rough estimate ~ 30 tokens per function signature if not provided; adapt as needed.
        30 * members.len()
    };

    // Split pass
    loop {
        let mut did_split = false;
        // collect first to avoid mutating while iterating
        let oversized: Vec<(String, FastSet<String>)> = clusters
            .iter()
            .filter_map(|(cid, members)| {
                let toks = estimate_tokens(members);
                if members.len() > budget.max_fun || toks > budget.max_tokens {
                    Some((cid.clone(), members.clone()))
                } else {
                    None
                }
            })
            .collect();

        if oversized.is_empty() {
            break;
        }

        for (cid, members) in oversized {
            // build induced subgraph
            let node_indices: Vec<NodeIndex> = members
                .iter()
                .filter_map(|k| key2ix.get(k).cloned())
                .collect();
            if node_indices.len() < 2 {
                continue;
            }
            let sub = induced_subgraph(graph, &node_indices)?;
            let sub_labels = label_propagation(&sub, 20)?;
            // turn sublabels into 2+ parts; if it didn’t split, force a balanced split by degree
            let parts = labels_to_parts(&sub, &sub_labels)?;
            if parts.len() <= 1 {
                // fallback: split by degree rank
                let mut ranked: Vec<(NodeIndex, f32)> = node_indices
                    .iter()
                    .map(|&ni| (ni, degree_weight(graph, ni)))
                    .collect();
                ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
                let mid = ranked.len() / 2;
                let left: FastSet<String> = ranked[..mid]
                    .iter()
                    .map(|(ni, _)| ix2key[ni.index()].clone())
                    .collect();
                let right: FastSet<String> = ranked[mid..]
                    .iter()
                    .map(|(ni, _)| ix2key[ni.index()].clone())
                    .collect();

                clusters.remove(&cid);
                clusters.insert(format!("{}a", cid), left);
                clusters.insert(format!("{}b", cid), right);
                did_split = true;
            } else {
                // replace original with parts
                clusters.remove(&cid);
                for (idx, part_nodes) in parts.into_iter().enumerate() {
                    let mems: FastSet<String> = part_nodes
                        .into_iter()
                        .map(|ni| ix2key[ni.index()].clone())
                        .collect();
                    clusters.insert(format!("{}p{}", cid, idx + 1), mems);
                }
                did_split = true;
            }
        }

        if !did_split {
            break;
        }
    }

    // Merge pass (small clusters)
    loop {
        let small: Vec<(String, FastSet<String>)> = clusters
            .iter()
            .filter_map(|(cid, members)| {
                let toks = estimate_tokens(members);
                if members.len() < budget.min_fun || toks < budget.min_tokens {
                    Some((cid.clone(), members.clone()))
                } else {
                    None
                }
            })
            .collect();

        if small.is_empty() {
            break;
        }

        let mut did_merge = false;
        for (cid, members) in small {
            // find neighbor cluster with max boundary weight
            let mut best_neighbor: Option<(String, f32)> = None;

            for (nid, nmem) in clusters.iter() {
                if *nid == cid {
                    continue;
                }
                let w = boundary_weight(graph, &key2ix, &members, nmem);
                if let Some((_, bw)) = &best_neighbor {
                    if w > *bw {
                        best_neighbor = Some((nid.clone(), w));
                    }
                } else {
                    best_neighbor = Some((nid.clone(), w));
                }
            }

            if let Some((nid, _)) = best_neighbor {
                // merge cid into nid
                if let Some(base) = clusters.get_mut(&nid) {
                    for k in members.iter() {
                        base.insert(k.clone());
                    }
                }
                clusters.remove(&cid);
                did_merge = true;
            }
        }

        if !did_merge {
            break;
        }
    }

    Ok(())
}

fn induced_subgraph(
    g: &UnGraph<String, f32>,
    nodes: &[NodeIndex],
) -> Result<UnGraph<String, f32>> {
    let set: FastSet<usize> = nodes.iter().map(|n| n.index()).collect();
    let mut sub = UnGraph::<String, f32>::default();
    let mut map: FastMap<usize, NodeIndex> = FastMap::default();
    for &ni in nodes {
        let new_ix = sub.add_node(g.node_weight(ni).unwrap().clone());
        map.insert(ni.index(), new_ix);
    }
    for &ni in nodes {
        for e in g.edges(ni) {
            let a = e.source().index();
            let b = e.target().index();
            if a == b {
                continue;
            }
            if set.contains(&a) && set.contains(&b) {
                let aa = *map.get(&a).unwrap();
                let bb = *map.get(&b).unwrap();
                if sub.find_edge(aa, bb).is_none() {
                    sub.add_edge(aa, bb, *e.weight());
                }
            }
        }
    }
    Ok(sub)
}

fn labels_to_parts(
    sub: &UnGraph<String, f32>,
    labels: &[usize],
) -> Result<Vec<Vec<NodeIndex>>> {
    let mut parts: HashMap<usize, Vec<NodeIndex>> = HashMap::new();
    for i in 0..sub.node_count() {
        parts.entry(labels[i]).or_default().push(NodeIndex::new(i));
    }
    Ok(parts.into_values().collect())
}

fn degree_weight(g: &UnGraph<String, f32>, ni: NodeIndex) -> f32 {
    let mut sum = 0.0;
    for e in g.edges(ni) {
        sum += *e.weight();
    }
    sum
}

fn boundary_weight(
    g: &UnGraph<String, f32>,
    key2ix: &FastMap<String, NodeIndex>,
    a: &FastSet<String>,
    b: &FastSet<String>,
) -> f32 {
    let mut sum = 0.0;
    for ka in a.iter() {
        if let Some(&ia) = key2ix.get(ka) {
            for e in g.edges(ia) {
                let kb = g.node_weight(e.target()).unwrap();
                if b.contains(kb) {
                    sum += *e.weight();
                }
            }
        }
    }
    sum
}

fn compute_assignments(
    clusters: &HashMap<String, FastSet<String>>,
) -> Vec<ClusterAssignment> {
    let mut out = Vec::new();
    for (cid, members) in clusters.iter() {
        for k in members {
            out.push(ClusterAssignment {
                isgl1_key: k.clone(),
                cluster_id: cid.clone(),
                membership_confidence: 0.9, // baseline; adjust if you compute soft labels
            });
        }
    }
    out
}

fn compute_cluster_nodes(
    g: &UnGraph<String, f32>,
    ix2key: &[String],
    clusters: &HashMap<String, FastSet<String>>,
    entities: &[Entity],
    dep_edges: &[DepEdge],
    wp: &WeightParams,
) -> Result<Vec<ClusterNode>> {
    let ent_map: FastMap<String, &Entity> = entities
        .iter()
        .map(|e| (e.isgl1_key.clone(), e))
        .collect();

    // build index
    let mut key2ix: FastMap<String, NodeIndex> = FastMap::default();
    for (i, k) in ix2key.iter().enumerate() {
        key2ix.insert(k.clone(), NodeIndex::new(i));
    }

    let name_re = Regex::new(r"[A-Za-z0-9_]+").unwrap();

    let mut nodes = Vec::new();
    for (cid, members) in clusters.iter() {
        // internal vs external weights
        let mut w_int = 0.0f32;
        let mut w_ext = 0.0f32;
        let mut k_set: FastSet<String> = members.clone();

        for ka in members.iter() {
            if let Some(&ia) = key2ix.get(ka) {
                for e in g.edges(ia) {
                    let kb = g.node_weight(e.target()).unwrap();
                    let w = *e.weight();
                    if k_set.contains(kb) {
                        w_int += w;
                    } else {
                        w_ext += w;
                    }
                }
            }
        }
        // undirected double-count fix (each internal edge seen twice)
        w_int *= 0.5;

        // token estimate
        let mut token_est = 0usize;
        let mut blast = 0usize;
        let mut degree_sum = 0.0f32;
        let mut verbs: FastMap<String, usize> = FastMap::default();

        for k in members.iter() {
            if let Some(ent) = ent_map.get(k) {
                token_est += ent.signature_tokens.unwrap_or(30);
                // crude verb extraction from name
                for cap in name_re.find_iter(&ent.name) {
                    let w = cap.as_str().to_lowercase();
                    // pick initial verb-like tokens
                    if !w.is_empty() {
                        *verbs.entry(w).or_insert(0) += 1;
                    }
                }
            }
            if let Some(&ix) = key2ix.get(k) {
                let deg = degree_weight(g, ix);
                degree_sum += deg;
                // blast radius approximation: neighbors outside cluster
                let mut local_blast = 0usize;
                for e in g.edges(ix) {
                    let nb = g.node_weight(e.target()).unwrap();
                    if !k_set.contains(nb) {
                        local_blast += 1;
                    }
                }
                blast += local_blast;
            }
        }
        let cohesion = if w_int + w_ext > 0.0 {
            w_int / (w_int + w_ext)
        } else {
            0.0
        };
        let coupling = 1.0 - cohesion;
        let centrality = if members.is_empty() {
            0.0
        } else {
            degree_sum / (members.len() as f32)
        };

        let label = auto_label(members, &ent_map, &verbs);

        let node = ClusterNode {
            cluster_id: cid.clone(),
            cluster_name: label,
            level: 0.5,
            contains: members.iter().cloned().sorted().collect(),
            metrics: ClusterMetrics {
                cohesion,
                coupling,
                modularity_local: cohesion - coupling, // simple proxy
                token_estimate: token_est,
                blast_radius: blast,
                centrality,
            },
            warnings: Vec::new(),
        };
        nodes.push(node);
    }

    // sort by centrality desc
    nodes.sort_by(|a, b| b.metrics.centrality.total_cmp(&a.metrics.centrality));
    Ok(nodes)
}

fn auto_label(
    members: &FastSet<String>,
    ent_map: &FastMap<String, &Entity>,
    verbs: &FastMap<String, usize>,
) -> String {
    // Strategy: pick dominant verb + dominant type from names
    let top_verb = verbs
        .iter()
        .max_by_key(|(_, c)| **c)
        .map(|(v, _)| v.clone())
        .unwrap_or_else(|| "unit".to_string());

    // derive common file stem prefix
    let mut stems: Vec<String> = Vec::new();
    for k in members {
        if let Some(ent) = ent_map.get(k) {
            let s = ent
                .file_path
                .split('/')
                .last()
                .unwrap_or(&ent.file_path)
                .split('.')
                .next()
                .unwrap_or("mod")
                .to_string();
            stems.push(s);
        }
    }
    let common = longest_common_prefix(&stems);
    if common.is_empty() {
        format!("{}", top_verb)
    } else {
        format!("{}_{}", top_verb, common)
    }
}

fn longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return "".to_string();
    }
    let mut prefix = strings[0].clone();
    for s in strings.iter().skip(1) {
        let mut i = 0;
        let max = prefix.len().min(s.len());
        while i < max && prefix.as_bytes()[i] == s.as_bytes()[i] {
            i += 1;
        }
        prefix.truncate(i);
        if prefix.is_empty() {
            break;
        }
    }
    prefix
}

fn compute_cluster_edges(
    g: &UnGraph<String, f32>,
    ix2key: &[String],
    clusters: &HashMap<String, FastSet<String>>,
    dep_edges: &[DepEdge],
    temporal: &TemporalSignal,
    dataflow: &DataFlowSignal,
) -> Result<Vec<ClusterEdge>> {
    let mut key2cluster: FastMap<String, String> = FastMap::default();
    for (cid, members) in clusters.iter() {
        for k in members {
            key2cluster.insert(k.clone(), cid.clone());
        }
    }

    let mut acc: FastMap<(String, String), (f32, f32, f32, usize)> = FastMap::default();
    for d in dep_edges {
        let (Some(ca), Some(cb)) = (key2cluster.get(&d.from_key), key2cluster.get(&d.to_key)) else { continue };
        if ca == cb { continue; }
        let mut pair = (ca.clone(), cb.clone());
        if pair.0 > pair.1 {
            std::mem::swap(&mut pair.0, &mut pair.1);
        }
        let control = if d.edge_type == "Calls" { d.frequency.unwrap_or(1.0) } else { 0.0 };
        let data = *dataflow.data_affinity.get(&(d.from_key.clone(), d.to_key.clone())).unwrap_or(&0.0);
        let temp = *temporal.co_change.get(&(d.from_key.clone(), d.to_key.clone())).unwrap_or(&0.0);
        let entry = acc.entry(pair).or_insert((0.0, 0.0, 0.0, 0));
        entry.0 += control;
        entry.1 += data;
        entry.2 += temp;
        entry.3 += 1;
    }

    let mut out = Vec::new();
    for ((a, b), (control, data, temporal_w, crossings)) in acc.into_iter() {
        out.push(ClusterEdge {
            from_cluster: a,
            to_cluster: b,
            weights: ClusterEdgeWeights { control, data, temporal: temporal_w },
            boundary_crossings: crossings,
        });
    }
    // sort by control weight desc
    out.sort_by(|x, y| y.weights.control.total_cmp(&x.weights.control));
    Ok(out)
}

fn approx_modularity(g: &UnGraph<String, f32>, clusters: &HashMap<String, FastSet<String>>) -> f32 {
    // very rough proxy: avg (cohesion - coupling)
    let mut total = 0.0;
    let mut count = 0.0;
    let mut key2ix: FastMap<String, NodeIndex> = FastMap::default();
    for i in 0..g.node_count() {
        let k = g.node_weight(NodeIndex::new(i)).unwrap().clone();
        key2ix.insert(k, NodeIndex::new(i));
    }
    for (_, members) in clusters.iter() {
        let mut w_int = 0.0;
        let mut w_ext = 0.0;
        let set = members;
        for k in set.iter() {
            if let Some(&ix) = key2ix.get(k) {
                for e in g.edges(ix) {
                    let nb = g.node_weight(e.target()).unwrap();
                    if set.contains(nb) {
                        w_int += *e.weight();
                    } else {
                        w_ext += *e.weight();
                    }
                }
            }
        }
        w_int *= 0.5;
        if w_int + w_ext > 0.0 {
            total += (w_int / (w_int + w_ext)) - (w_ext / (w_int + w_ext));
            count += 1.0;
        }
    }
    if count == 0.0 { 0.0 } else { total / count }
}

fn write_json<T: Serialize>(path: impl AsRef<Path>, val: &T) -> Result<()> {
    let s = serde_json::to_string_pretty(val)?;
    fs::write(path, s)?;
    Ok(())
}
```

Minimal CLI wrapper (optional):
```rust
// crates/pt07-analytics/src/bin/pt02-level00-clustered.rs
use anyhow::Result;
use std::path::PathBuf;
use clap::Parser;
use pt07_analytics::sgl05::*;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    entities: PathBuf, // PT02 L1 export of entities (or merged source)
    #[arg(long)]
    edges: PathBuf,    // PT02 L0 or L1 edges
    #[arg(long)]
    out_dir: PathBuf,
    #[arg(long, default_value = "0.5")]
    level: f32,
}

fn main() -> Result<()> {
    // load entities/edges JSON, build signals (temporal/data/semantic if available) and run SGL0.5::run(...)
    Ok(())
}
```

CozoDB schema (if persisting):
```datalog
:create SemanticClusters {
    cluster_id: String =>
    cluster_name: String,
    level: Float,
    cohesion: Float,
    coupling: Float,
    modularity_local: Float,
    token_estimate: Int,
    blast_radius: Int,
    centrality: Float
}

:create ClusterAssignments {
    isgl1_key: String =>
    cluster_id: String,
    membership_confidence: Float
}

:create ClusterEdges {
    from_cluster: String,
    to_cluster: String =>
    control_w: Float,
    data_w: Float,
    temporal_w: Float,
    boundary_crossings: Int
}
```

Write persistence (conceptual):
```datalog
# :put rows for each export; ensure transactions
```

## Variations you can plug in (algos used elsewhere)

-  Louvain/Leiden (modularity maximization)
    - Pros: High-quality communities; multi-resolution via resolution parameter; widely used (e.g., social networks).
    - Cons: More complex to implement; slower than LPA but still fast.
    - Plan: Keep the Sgl05 trait and add Sgl05Louvain impl; run Louvain globally, then hierarchical refine per cluster to meet token budgets.

-  Label Propagation (LPA) [baseline above]
    - Pros: Very fast, parallelizable, few knobs.
    - Cons: Sometimes unstable on sparse graphs; lower modularity than Louvain/Leiden.

-  Markov Clustering (MCL)
    - Pros: Great for flow-based communities (random walks, expansion+inflation).
    - Cons: Requires matrix ops; memory heavier.
    - Use when: Call graphs have strong flow patterns (e.g., pipelines).

-  Spectral clustering (normalized Laplacian, eigengap)
    - Pros: Finds balanced cuts; good for detecting macro boundaries.
    - Cons: Eigen-decomposition can be heavy on very large graphs; careful with numeric stability.

-  Hierarchical agglomerative clustering (Ward/average linkage)
    - Pros: Gives a full dendrogram; you can choose ISGL0.3/0.5/0.7 cuts.
    - Cons: O(n^2) similarity maintenance unless approximated.

-  Infomap (information-theoretic)
    - Pros: Minimizes description length of random walks; often yields crisp modules.
    - Cons: More work to implement; still CPU-only feasible.

Pluggable design:
```rust
pub trait CommunityDetector {
    fn detect(&self, g: &UnGraph<String, f32>) -> Result<Vec<usize>>;
}
```
Implement LPA/Louvain/MCL/Spectral under this trait, then reuse the same budget-enforcement, metrics, and export pipeline.

## What yields more insight (beyond clustering)

-  Multi-level projections: bundle ISGL4/3/2/0.5 in one multilevel_graph.json so LLM can zoom quickly.
-  Cluster-level hot paths: “payment_unit → auth_unit → db_unit” with weights; drives focused performance and bug triage.
-  Flow mismatch checks: when control flow A→B→C but data flows A⇒C; suggests removing a pass-through unit.
-  Temporal hidden deps: heavy co-change between clusters without edges → missing abstraction or cross-boundary state.
-  Architecture guardrails: run your layered rules on cluster graph, not file graph; far less noise.

## Systems programming vs full-stack: different knobs

-  Systems (C/C++/Rust):
    - Stronger weight on dependency and data-flow (\(\alpha, \beta\) higher).
    - Include-header graph matters; treat macro-generated edges carefully; favor accurate edges from libclang/rust-analyzer.
    - Concurrency/resource patterns: add edges for lock acquisition/ownership to cluster I/O and synchronization units correctly.
    - Token budgets: smaller clusters (tight interfaces, many small functions).

-  Full-stack (Rails/Java/Go/TS):
    - Stronger temporal and semantic signal (\(\gamma, \delta\) higher).
    - Add domain edges: routes → controllers → services → repos; AR associations; view render edges.
    - Token budgets: aim for cohesive feature verticals (controller+service+model slice).
    - UI/backend split: often needs two-level clustering then filtered join for feature flows.

Recommended defaults:
-  Systems: \((\alpha,\beta,\gamma,\delta) = (1.0, 0.9, 0.3, 0.3)\)
-  Full-stack: \((\alpha,\beta,\gamma,\delta) = (0.8, 0.6, 0.8, 0.6)\)

## How to get “totally correct” in practice

Absolute correctness doesn’t exist for unsupervised clustering, but you can enforce operational correctness:

-  Partition invariants
    - Every function must be in exactly one cluster (or explicitly excluded).
    - No cluster empty; cluster IDs stable across runs (given fixed seed and graph).

-  Quality gates (fail the run if thresholds not met)
    - Median cohesion ≥ 0.80; 90th percentile coupling ≤ 0.40.
    - Modularity_global ≥ 0.55 on internal repos (tunable per codebase).
    - Token guardrails: no cluster exceeds max_tokens; warn if < min_tokens.

-  Reproducibility
    - Deterministic tie-breaking; fixed iteration order; record parameters in cluster_manifest.json.

-  Tests (unit + property-based)
    - labels form a disjoint cover of nodes; merging/splitting keeps invariant.
    - Splitting reduces within-cluster diameter; merging increases.
    - Budget enforcement never increases cross-edge weight sum (within an epsilon).

-  Benchmarks on canonical repos
    - Keep golden summaries for before/after; diff cohesion/coupling/modularity; reject regressions > threshold.

Example test skeleton:
```rust
#[test]
fn clusters_form_partition() {
    // build tiny graph; run SGL0.5; assert disjoint cover and non-empty clusters
}

#[test]
fn budget_enforcement_respects_limits() {
    // construct a cluster over budget; after enforce, all clusters within token bounds
}
```

## Next steps

-  Integrate this module as pt02-level00-clustered and add pt02-context-pack to generate LLM-ready packs.
-  Add a second detector (Louvain) behind a feature flag; compare modularity and token efficiency.
-  Extend signals: feed temporal/data/semantic from existing pipelines; default to zeros if unknown.
-  Hook into CozoDB (stored relations) and TUI to make the clusters the first-class navigation unit.

If you want, I can adapt this code to your current crate layout and wire it to your existing PT02 exports and CozoDB access layer.



