# Flatgraph Algorithm Fit Summary

## Core Thesis

Flatgraph is a strong fit for a specific cluster of graph workloads.

It is **not** a full replacement for a general-purpose graph database.

It is best understood as a:

- versioned graph analytics engine
- algorithm-native snapshot graph store
- persisted graph runtime for simulation-centric workloads

## Local Source Basis

This summary is based on the merged algorithm lists and workflow notes in:

- `docs/pre-400/v301-PRD.md`
- `docs/pre-400/bidirectional-workflows-01.md`
- `archived-docs/journey-of-parseltongue/v160-v165-release-notes.md`

Together, those sources describe a merged universe of graph work including:

- BFS
- DFS
- SCC / Tarjan
- PageRank
- Personalized PageRank
- In-degree / Out-degree
- Betweenness
- k-core
- Leiden
- Closeness
- Harmonic
- Eigenvector
- Katz
- HITS
- Louvain
- Label Propagation
- Infomap
- Walktrap
- Spectral clustering
- CK metrics
- Shannon entropy
- SQALE debt scoring
- snapshot diff / architecture simulation

## Where Flatgraph Works Best

| algorithm cluster | fit | reason |
| --- | --- | --- |
| BFS / DFS / reachability | Excellent | direct adjacency traversal is the native operation |
| Blast radius / dependency chain / dead code | Excellent | reachability from roots or anchors fits snapshot traversal perfectly |
| SCC / Tarjan | Excellent | static full-graph DFS passes fit forward/backward adjacency well |
| PageRank / PPR / HITS / Katz / Eigenvector | Excellent | iterative sparse scans over a fixed graph are a natural fit |
| In-degree / Out-degree | Excellent | trivial precompute from edge arrays |
| k-core | Excellent | repeated degree peeling works well on static adjacency |
| Leiden / Louvain / Label Propagation / Infomap / Walktrap | Good to excellent | community detection is snapshot-friendly; library quality matters more than storage branding |
| Cross-module coupling / boundary analysis | Good to excellent | edge counting across first-class dimensions and boundaries is straightforward |
| Snapshot diff / architecture simulation | Good if versioned | needs multiple snapshots or a mutation layer, not just one flat file |

## Where Flatgraph Is Weaker

| algorithm cluster | fit | reason |
| --- | --- | --- |
| Betweenness / closeness / harmonic | Good but costly | works, but these remain expensive and should stay precomputed |
| Spectral clustering | Mixed | wants matrix-first numeric tooling; export helps, but this is not the sweetest native fit |
| CK metrics / entropy / SQALE | Mixed to weak | these depend on code semantics, containment, and source-derived features, not just graph adjacency |
| Highly dynamic online graph analytics | Weak unless extended | pure snapshots are not enough for fast mutable graph workloads |
| Arbitrary graph pattern querying | Weak | flatgraph is not a Cypher/GQL-style query engine |

## Comparison With OSS Graph Databases

Existing OSS or OSS-adjacent graph systems already support many of these algorithms:

- Neo4j Graph Data Science
- Kuzu
- ArangoDB Pregel
- Memgraph + MAGE

So the category gap is **not**:

> a graph database that can run graph algorithms

The real gap is narrower:

> a lightweight, embedded, algorithm-native, multi-dimensional graph engine for versioned and simulation-centric workloads

## What Flatgraph Beats Graph DBs At

- cold start
- embedding inside local tools and agents
- fixed-path traversal workloads
- snapshot-based analytics
- direct control over dimensions and edge layouts
- low operational overhead

## What Graph DBs Beat Flatgraph At

- ad hoc graph querying
- general query languages
- transactions and concurrent writers
- rich secondary indexing
- multi-user serving
- mature ecosystem integrations

## Naming Consequence

The naming should follow the workload truth.

The whole system should **not** be called:

- graph database
- graph log
- first graph log

Better category names are:

- versioned graph analytics engine
- algorithm-native snapshot graph store
- persisted graph runtime

`graph log` is still a good name for the append-only mutation/history subsystem, if that layer exists.

## Practical Conclusion

Flatgraph serves the **traversal + ranking + community + decomposition + simulation** cluster well.

It serves the **general database + arbitrary query + online mutation** cluster less well.

That means the product should be positioned around:

- code graphs
- public interface graphs
- architecture simulation
- agent-facing graph retrieval

not around replacing every graph database.
