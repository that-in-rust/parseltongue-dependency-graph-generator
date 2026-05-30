# Pensieve PMF Cluster Ratings

## Scoring Lens

These are directional strategy scores, not benchmark measurements.

- **Performance score** = how well Pensieve's current snapshot + `mmap` + forward/backward dimension architecture fits the cluster
- **PMF score** = a Shreyas-style "would users switch for this?" score against the closest OSS substitutes

## Cluster Ratings

| Cluster | Pensieve Performance | Pensieve PMF vs OSS | Closest current substitutes | Strategic read |
| --- | ---: | ---: | --- | --- |
| Walk The Graph | 94/100 | 76/100 | `petgraph`, Kuzu, DuckPGQ, Memgraph, Neo4j | Strongest wedge for Pensieve. The user pain is real when simple traversal requires a heavyweight graph stack or custom persistence. |
| Rank The Graph | 87/100 | 54/100 | Neo4j GDS, Memgraph, Kuzu, DuckPGQ | Strong technical fit, weaker PMF. Ranking algorithms are useful, but already commoditized in mature systems. |
| Find The Shape | 81/100 | 63/100 | Neo4j GDS, Memgraph, ArangoDB Pregel | Good fit and better PMF than ranking because cycles, communities, and core-periphery structure are closer to real architecture questions. |
| Heavy Batch Analytics | 43/100 | 24/100 | ArangoDB Pregel, Neo4j GDS, Memgraph MAGE | Weak wedge for Pensieve. These jobs favor stronger parallel or distributed systems more than a lightweight snapshot runtime. |
| Semantic Product Layer | 18/100 | 7/100 for Pensieve alone | Not really graph DB competition; this is Parcel10 territory | This is not Pensieve's market. The real product upside exists here, but above Pensieve, not inside it. |

## Why The Scores Look Like This

### Walk The Graph

- Pensieve is basically an adjacency-walking machine.
- Forward and backward slices make BFS, callers/callees, reachability, blast radius, and dead-code style traversal feel native.
- PMF is highest here because the user pain is concrete: too much operational weight for simple graph questions.

### Rank The Graph

- PageRank, PPR, HITS, Katz, Eigenvector, and degree-based ranking fit fixed sparse snapshots well.
- Pensieve can run these efficiently, but mature graph systems already offer them with stronger surrounding tooling.
- So this cluster helps Pensieve, but will rarely be the main reason someone adopts it.

### Find The Shape

- SCC, k-core, Leiden, Louvain, coupling, and boundary counts work well on one frozen graph snapshot.
- This cluster matters because it turns raw graph storage into structure discovery.
- PMF is better than ranking because users care more about "what shape is my system?" than about another centrality score.

### Heavy Batch Analytics

- Betweenness, closeness, harmonic, and some spectral work will run, but they lose the low-latency advantage that makes Pensieve interesting.
- These are usually precompute or background jobs, not fast local loops.
- Pensieve should support them selectively, but should not lead with them.

### Semantic Product Layer

- Smart context selection, public boundary resolution, CK metrics, entropy, SQALE, snapshot diff, and architecture simulation are not graph-runtime-native questions.
- They require semantics, policy, identity matching, and product heuristics.
- This is exactly why Parcel10 should exist separately.

## Competitor Read

Pensieve should not be framed as:

- a general graph database replacement
- a Cypher or GQL competitor
- a full graph analytics platform

Existing systems already cover much of that ground:

- [Neo4j Graph Data Science](https://neo4j.com/docs/graph-data-science/current/algorithms/)
- [ArangoDB Pregel](https://docs.arangodb.com/3.11/data-science/pregel/)
- [DuckPGQ](https://duckpgq.org/)
- [Kuzu](https://github.com/kuzudb/kuzu)
- [Memgraph](https://memgraph.com/)
- [petgraph](https://docs.rs/petgraph/)

The more honest category is:

- embedded graph runtime
- persisted graph substrate
- snapshot-first graph engine

## Bottom Line

Pensieve has real PMF in the **embedded graph runtime for local traversal and structural insight** lane.

Pensieve does **not** have strong PMF as a generic graph database replacement.

The biggest PMF upside sits in Parcel10, where these graph capabilities become:

- architecture decisions
- snapshot deltas
- futures and simulations
- LLM-ready explanations
