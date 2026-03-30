# PRD

## User Journeys

### User Journey 01 : Tauri App for indexing a folder and providing a HTTP URL to the LLM

``` text

Step 01: User downloads the Tauri App from github releases of the github repository - since we will limit it to macOS only for now, it will be a .dmg file

Step 02: User opens the Tauri App and is presented with a welcome screen, which tells the user that this app is privacy-first and does not send any data to any server, everything happens locally on the user's machine

Step 03: User sees 
    - Assuming workspaces are indexed, the user is presented with a table view of workspaces
        - top most row is Add new workspace button (click to add a new workspace)
        - each workspace row has the following information:
            - workspace name
            - workspace status
            - workspace last indexed
            - reindex button
            - start HTTP server button
            - stop HTTP server button
            - delete workspace button
            - copy HTTP URL button (click to copy the HTTP URL to the clipboard)


```


## Raw pointers

### Product anchor

``` text
  │     │ Tauri Mac app with workspace management             │     │      │ One-click from "I have code" to "my LLM understands it." No CLI. No config       │                                              │
  │ 13  │ (drag-and-drop folder, progress bar, FRESH/STALE    │ 91  │ 70   │ files. No Docker. Download .dmg, open, drag folder. The HTTP URL is the only     │ PRD-v300 (Screens 1-7), FUJ (Phase 1)        │
  │     │ badge, HTTP URL copy)                               │     │      │ thing the LLM needs.                                                             │                                              │
```

### Fixed algorithm shortlist for 5-step / 7-step journeys

This list is fixed for v301 and is accepted as the working shortlist for the graph layer.

#### Journey legend

| Journey step | Meaning |
| --- | --- |
| `SEARCH` | Query intake, candidate finding, exact/fuzzy retrieval |
| `ANCHOR` | Resolve a private hit to a public interface or presentable boundary |
| `CLUSTER` | Build a preview neighborhood around the anchor under a token cap |
| `ASK / CHOICE` | Present 2-4 options and let the LLM or human pick |
| `DEEP DIVE` | Expand to a deeper graph slice plus source, flow, and type context |
| `POST-DIVE` | Optional architecture and hotspot analysis after the main converge flow |

#### Graph algorithms

| Priority | Algorithm | Journey step(s) | Role in the user journey | Compute style | Most reliable Python implementation | Expected data form |
| --- | --- | --- | --- | --- | --- | --- |
| `P0` | `BFS` | `ANCHOR`, `DEEP DIVE`, `POST-DIVE` | Upward anchor walk, hop-limited blast radius, deep graph slice expansion | Query-time | `python-igraph` | Directed edge list with integer node ids: `(src_idx, dst_idx, weight?)` |
| `P0` | `PageRank` | `SEARCH` tie-break, `CLUSTER`, `POST-DIVE` | Global importance prior for ranking and representative-node selection | Precomputed per snapshot | `python-igraph` | Directed weighted adjacency from edge list or sparse CSR |
| `P0` | `Personalized PageRank` | `CLUSTER` | Local ranking from the chosen anchor so preview packets contain the most relevant neighbors first | Query-time or cached per hot anchor | `python-igraph` | Directed weighted adjacency plus one or more seed nodes |
| `P0` | `In-Degree` | `CLUSTER`, `POST-DIVE` | Fan-in signal for importance, utility-ness, risk, and hotspot summaries | Precomputed | `Polars` or `python-igraph` | Edge table with `dst_id`; simple grouped counts |
| `P0` | `Out-Degree` | `CLUSTER`, `POST-DIVE` | Fan-out signal for orchestration complexity and “god function” style alerts | Precomputed | `Polars` or `python-igraph` | Edge table with `src_id`; simple grouped counts |
| `P1` | `Betweenness` | `POST-DIVE`, architecture review | Finds bridge nodes, chokepoints, adapters, and risky coordination points | Precomputed | `NetworKit` | Directed or undirected graph object built from edge list; weighted optional |
| `P1` | `k-core` | `POST-DIVE`, architecture review, smart context breadth | Separates dense architectural core from periphery and supports hotspot layering | Precomputed | `NetworKit` | Graph object or sparse adjacency; usually unweighted dependency graph |
| `P1` | `Leiden` | `POST-DIVE`, architecture review, cluster coverage | Natural module/community boundaries, representative community sampling, architecture map | Precomputed | `python-igraph` | Weighted graph from edge list; optional resolution parameter and edge weights |
| `P2` | `DFS` | `DEEP DIVE`, cycle/path drill-down | Deep path trace, reachability checks, and certain cycle-oriented inspections | Query-time | `python-igraph` | Directed adjacency / edge list |
| `P2` | `Closeness` | `POST-DIVE` | Measures central position in the graph; useful for ranking entities that are topologically “near many others” | Precomputed | `NetworKit` | Graph object; weighted or unweighted shortest-path capable graph |
| `P2` | `Harmonic` | `POST-DIVE` | Closeness-like ranking that behaves better on disconnected graphs | Precomputed | `NetworKit` | Graph object with disconnected components allowed |
| `P2` | `Eigenvector` | `POST-DIVE` | Strategic importance from being connected to other important nodes | Precomputed | `python-igraph` | Directed or undirected weighted adjacency |
| `P2` | `Katz` | `POST-DIVE` | Directed-graph influence with attenuation over distance; useful when PageRank is too web-like | Precomputed | `NetworKit` | Directed weighted adjacency and attenuation parameter |
| `P2` | `HITS` | `POST-DIVE` | Separates hubs from authorities, which is useful for orchestrators vs utilities | Precomputed | `python-igraph` | Directed graph with meaningful directionality |
| `P2` | `Louvain` | `POST-DIVE`, fallback clustering | Baseline community detection and fallback when Leiden is unavailable or for benchmarking | Precomputed | `python-igraph` | Weighted graph from edge list |
| `P2` | `Label Propagation` | `POST-DIVE`, fast approximation | Very fast approximate communities for cheap first-pass clustering | Precomputed or ad hoc | `NetworKit` | Unweighted or lightly weighted graph object |
| `P2` | `Infomap` | `POST-DIVE`, flow view | Flow-oriented community detection; useful when call-flow structure matters more than modularity | Precomputed | `python-igraph` | Directed weighted graph from edge list |
| `P2` | `Walktrap` | `POST-DIVE`, dense local communities | Random-walk communities for dense graphs and local structure experiments | Precomputed | `python-igraph` | Weighted graph from edge list |
| `P2` | `Spectral` | `POST-DIVE`, small high-quality clustering | High-quality small-graph clustering and research-grade architecture partitions | Precomputed / offline | `scikit-network` | Sparse CSR adjacency matrix or affinity matrix |

#### Supporting primitives required by the journeys

These are not part of the fixed graph-algorithm shortlist above, but the PRD cannot be realized without them.

| Primitive | Journey step(s) | Role in the user journey | Most reliable Python implementation | Expected data form |
| --- | --- | --- | --- | --- |
| `RRF` | `SEARCH` | Fuses exact symbol, fuzzy, and recency lanes into one deterministic candidate ranking | Custom Python function over ranked lists | Ranked candidate lists: `[(entity_id, rank, source)]` |
| `Symbol trie` | `SEARCH` | Exact/prefix lookup for symbols, module paths, and public API names | `marisa-trie` | Static vocabulary of strings plus payload mapping to entity ids |
| `Trigram retrieval` | `SEARCH` | Fuzzy lookup for misspellings, partial names, and loose textual similarity | `RapidFuzz` for production fuzzy matching, or `scikit-learn` char-3gram sparse vectors when explicit trigram semantics are desired | Symbol strings or sparse char-ngram matrix |
| `Ego network extraction` | `CLUSTER` | Builds the 1-hop or 2-hop preview neighborhood around the anchor | `python-igraph` | Anchor node id plus full graph edge list / graph object |
| `Token budget packing` | `CLUSTER` | Keeps preview packets under a hard cap by greedily selecting high-value nodes/snippets | Custom Python / `Polars` | Candidate rows with token estimates, rank, edge role, and snippet metadata |
| `CFG extraction` | `DEEP DIVE` | Control-flow view for selected function or deep packet | Custom compiler-backed extraction, traversed with `python-igraph` | Basic-block or statement nodes with directed control-flow edges |
| `DDG extraction` | `DEEP DIVE` | Data-dependency view for selected function or cluster | Custom compiler-backed extraction, traversed with `python-igraph` | Variable/statement nodes with directed data-flow edges |
| `Type-flow` | `DEEP DIVE` | Compiler-verified types, impl edges, and trait-driven context in resolved packets | Custom compiler-backed extraction, traversed with `python-igraph` | Type/entity nodes plus typed relation edges |

#### Library selection notes

| Need | Recommended library | Why |
| --- | --- | --- |
| One primary graph engine with the least doubt | `python-igraph` | Strong coverage across traversal, ranking, and community detection in one mature stack |
| Fast centrality and core decomposition | `NetworKit` | Reliable high-performance analytics for betweenness, closeness-family metrics, and k-core |
| Matrix-first clustering and sparse adjacency workflows | `scikit-network` | Clean fit when data starts as sparse CSR from edge tables |
| Table ETL, degree counts, and feature prep | `Polars` | Best place for edge-table cleanup, joins, and simple grouped metrics |
| Validation harness | Golden fixtures + integration invariants | Trust library math, validate edge loading, graph shaping, parameters, and product outputs |

#### Data-shape defaults for the Python analytics sidecar

| Shape | Use |
| --- | --- |
| `Polars DataFrame edge table` with `src_id`, `dst_id`, `edge_kind`, `weight`, `snapshot_id` | Durable interchange shape after loading from Turso/libSQL |
| `entity_id -> node_idx` mapping table | Required before building igraph or sparse matrices |
| Integer-indexed edge list | Best universal format for `python-igraph` and `NetworKit` |
| Sparse CSR adjacency matrix | Best for `scikit-network`, matrix-style PageRank, and spectral methods |
| Per-node feature table | Useful for ranking summaries, hotspot cards, and future clustering experiments |

#### Implementation stance

- Default to a sparse representation, not a dense matrix.
- Use `Polars` for data preparation and cache tables, not as the graph engine.
- Use `python-igraph` as the default production graph runtime.
- Use `NetworKit` where centrality or k-core performance matters.
- Trust the production graph libraries for algorithm correctness.
- Validate only the integration layer: edge loading, node remapping, graph slicing, parameters, and product outputs.
- Keep a small set of golden fixtures and repeatable benchmark snapshots for regression detection.
