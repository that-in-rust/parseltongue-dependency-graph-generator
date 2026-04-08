# Pensieve Runtime Architecture Summary

## Core Thesis

If Pensieve is a separate product, it should be treated as a **persisted graph runtime**, not as a graph database.

Its job is to:

- persist graph snapshots
- open them quickly
- expose forward and backward adjacency by dimension
- run graph-native algorithms well
- remain domain-agnostic

Its job is **not** to:

- parse code
- understand tree-sitter entities
- define public interface semantics
- compare code snapshots semantically
- perform architecture simulation
- manage LLM context workflows

That higher-order work belongs in Parcel10.

## Product Boundary

### Pensieve owns

- graph snapshot format
- node and edge persistence
- per-dimension forward adjacency
- per-dimension backward adjacency
- dimension dictionary
- indexes and lookup tables
- graph views and projections
- graph-native algorithms

### Parcel10 owns

- tree-sitter ingestion
- code and entity taxonomy
- public interface graph rules
- test and comment policies
- stable entity identity
- snapshot-to-snapshot reconciliation
- delta packets
- architecture simulation
- LLM-facing explanation

## Runtime Architecture

Pensieve should stay boring and sharp:

1. **Snapshot file**
   - immutable graph artifact
   - header
   - node table
   - dimension dictionary
   - per-dimension adjacency sections
   - string pool
   - index sections

2. **Runtime open path**
   - `mmap` file
   - validate header
   - cast sections into read-only slices

3. **Core API**
   - `forward(dim, node_id)`
   - `backward(dim, node_id)`
   - `node(node_id)`
   - `dimensions()`

4. **Projection layer**
   - single-dimension view
   - multi-dimension blended view
   - filtered subgraph view

5. **Algorithm layer**
   - operates over graph views
   - remains independent of code semantics

## Expanded Algorithm Table

| algorithm | pensieve fit | where it belongs | note |
| --- | --- | --- | --- |
| BFS | Excellent | Pensieve core | direct adjacency traversal |
| DFS | Excellent | Pensieve core | direct adjacency traversal |
| Reachability | Excellent | Pensieve core | walk from a root or anchor |
| Reverse reachability | Excellent | Pensieve core | backward adjacency is first-class |
| Forward callers/callees traversal | Excellent | Pensieve core | native per-dimension walk |
| Blast radius | Excellent | Pensieve primitive, Parcel10 workflow | BFS over chosen dimensions |
| Dead code from roots | Excellent | Pensieve primitive, Parcel10 workflow | reachability from entry points |
| SCC / Tarjan | Excellent | Pensieve core | static full-graph pass |
| In-degree | Excellent | Pensieve core | trivial precompute |
| Out-degree | Excellent | Pensieve core | trivial precompute |
| PageRank | Excellent | Pensieve core | iterative sparse scan over snapshot |
| Personalized PageRank | Excellent | Pensieve core | seeded local ranking |
| HITS | Excellent | Pensieve core | iterative adjacency math |
| Katz | Excellent | Pensieve core | iterative adjacency math |
| Eigenvector centrality | Excellent | Pensieve core | iterative adjacency math |
| k-core | Excellent | Pensieve core | degree peeling on static graph |
| Boundary crossing counts | Good to excellent | Pensieve core | edge counting across groups or dimensions |
| Coupling ratios | Good to excellent | Pensieve core or Parcel10 thin layer | graph-derived if boundaries are first-class |
| Community detection: Leiden | Good to excellent | Pensieve algorithm pack | snapshot-friendly; implementation quality matters |
| Community detection: Louvain | Good to excellent | Pensieve algorithm pack | snapshot-friendly |
| Community detection: Label Propagation | Good to excellent | Pensieve algorithm pack | lightweight community method |
| Community detection: Infomap | Good to excellent | Pensieve algorithm pack | snapshot-friendly |
| Community detection: Walktrap | Good to excellent | Pensieve algorithm pack | snapshot-friendly |
| Betweenness centrality | Good but costly | Pensieve precompute only | too expensive for hot path |
| Closeness centrality | Good but costly | Pensieve precompute only | repeated shortest-path cost |
| Harmonic centrality | Good but costly | Pensieve precompute only | same family as closeness |
| Shortest path | Good | Pensieve core | weighted support improves usefulness |
| Core-periphery layering | Good to excellent | Pensieve algorithm pack | usually derived from k-core or communities |
| Spectral clustering | Mixed | Better above Pensieve or with matrix backend | not the sweetest native fit |
| Smart context selection | Not a Pensieve primitive | Parcel10 | mixes graph signals with product heuristics and token budgeting |
| Public boundary resolution | Not a Pensieve primitive | Parcel10 | code semantics above graph runtime |
| Test impact analysis | Not a Pensieve primitive | Parcel10 | needs test tagging and code-specific policy |
| CK metrics | Mixed to weak | Parcel10 | depend on source semantics and taxonomy |
| Shannon entropy | Mixed to weak | Parcel10 | more source/content than graph-native |
| SQALE debt scoring | Mixed to weak | Parcel10 | quality model, not graph-native |
| Snapshot diff | Not native | Parcel10 delta engine | needs multiple snapshots and identity reconciliation |
| Architecture simulation | Not native | Parcel10 delta engine | compare worlds, not just traverse one world |

## Strategic Positioning

Pensieve should be positioned as:

- a persisted graph runtime
- an embedded multi-dimensional graph engine
- a snapshot-first graph substrate

It should not be positioned as:

- a general graph database
- a Cypher replacement
- a mutable graph server

## Bottom Line

Pensieve persists worlds.

Parcel10 compares and explains worlds.

That split keeps Pensieve generic and useful, while keeping the real product differentiation in the simulation and decision-support layer.
