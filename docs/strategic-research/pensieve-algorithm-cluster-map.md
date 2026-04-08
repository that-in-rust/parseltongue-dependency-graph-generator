# Pensieve Algorithm Cluster Map

## Core Table

| Algorithm | ELI5 | Cluster |
| --- | --- | --- |
| BFS | Find everything nearby step by step. | Walk The Graph |
| DFS | Go deep down one path before backing up. | Walk The Graph |
| Reachability | Check whether one thing can eventually get to another. | Walk The Graph |
| Reverse reachability | Check who can eventually reach a target. | Walk The Graph |
| Forward callers/callees | Show direct neighbors in front of or behind a node. | Walk The Graph |
| Blast radius | Show what might be affected if one node changes. | Walk The Graph |
| Dead code from roots | Find what is never reached from important starting points. | Walk The Graph |
| Shortest path | Find the quickest route between two nodes. | Walk The Graph |
| In-degree | Count how many things point in. | Rank The Graph |
| Out-degree | Count how many things point out. | Rank The Graph |
| PageRank | Score importance by who points to you. | Rank The Graph |
| Personalized PageRank | Score importance relative to one starting point. | Rank The Graph |
| HITS | Split importance into hubs and authorities. | Rank The Graph |
| Katz | Give credit for many paths, not just direct ones. | Rank The Graph |
| Eigenvector centrality | Important nodes are pointed to by important nodes. | Rank The Graph |
| SCC / Tarjan | Find tightly circular groups. | Find The Shape |
| k-core | Find the dense inner shell of the graph. | Find The Shape |
| Boundary crossing counts | Count how often edges cross group borders. | Find The Shape |
| Coupling ratios | Measure how entangled two groups are. | Find The Shape |
| Leiden | Find natural communities in the graph. | Find The Shape |
| Louvain | Group nodes into communities. | Find The Shape |
| Label Propagation | Let neighboring labels spread until groups appear. | Find The Shape |
| Infomap | Find groups by following flow through the graph. | Find The Shape |
| Walktrap | Find groups using short random walks. | Find The Shape |
| Core-periphery layering | Split dense center from sparse edge. | Find The Shape |
| Betweenness centrality | Find bridge nodes that many shortest paths pass through. | Heavy Batch Analytics |
| Closeness centrality | Find nodes that are near many others on average. | Heavy Batch Analytics |
| Harmonic centrality | A closeness-like score that works better on broken graphs. | Heavy Batch Analytics |
| Spectral clustering | Group nodes using matrix math and eigenvectors. | Heavy Batch Analytics |
| Smart context selection | Pick the best small subgraph to show a human or LLM. | Semantic Product Layer |
| Public boundary resolution | Find the public surface above some internal node. | Semantic Product Layer |
| Test impact analysis | Estimate how a change may affect tests. | Semantic Product Layer |
| CK metrics | Compute classic code-structure metrics. | Semantic Product Layer |
| Shannon entropy | Compute a complexity-like variability score. | Semantic Product Layer |
| SQALE debt scoring | Turn code-quality issues into a debt score. | Semantic Product Layer |
| Snapshot diff | Compare one graph snapshot to another. | Semantic Product Layer |
| Architecture simulation | Explore "what if this changed?" futures. | Semantic Product Layer |

## Cluster Reasoning

### Walk The Graph

These are the algorithms Pensieve is best at.

They mostly ask:

- what can I reach?
- who points here?
- what changes if I start from this node?

Pensieve's storage is built around forward and backward adjacency, so these operations are the most natural fit.

### Rank The Graph

These ask:

- what matters most?
- what should I look at first?
- which nodes are central?

Pensieve is strong here because snapshot graphs are a good fit for repeated sparse scans and cached ranking passes.

### Find The Shape

These ask:

- where are the cycles?
- what is the dense core?
- what naturally belongs together?
- where are the boundaries?

Pensieve is also strong here because these algorithms operate well on one frozen world with stable adjacency.

### Heavy Batch Analytics

These still fit technically, but they are expensive enough that they should usually run offline, be cached, or be treated as batch work.

Pensieve can support them, but they are not the right default for fast interactive workflows.

### Semantic Product Layer

These are not really Pensieve questions.

They need:

- code semantics
- policy
- stable identity across snapshots
- heuristics for humans or LLMs

That makes them Parcel10 responsibilities built on top of Pensieve, not inside Pensieve itself.

## Bottom Line

Pensieve is strongest at:

- walking the graph
- ranking the graph
- finding the shape of the graph

Pensieve is weaker or non-native for:

- expensive batch-only analytics
- code-semantic interpretation
- cross-snapshot simulation
