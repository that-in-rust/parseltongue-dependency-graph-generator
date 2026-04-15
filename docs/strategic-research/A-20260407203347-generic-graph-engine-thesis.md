# Generic Graph Engine Thesis

## Premise Check

Parseltongue should not position itself as "the graph database industry has been waiting for."

That framing is strategically weak for two reasons:

- it is too broad
- it is not true that graph databases cannot run the relevant algorithms

Existing graph systems already support many of the core algorithms we care about:

- Neo4j Graph Data Science supports PageRank, SCC, k-core, Leiden, and many related algorithms
- Kuzu exposes graph algorithms like PageRank, SCC, and k-core
- ArangoDB Pregel supports PageRank, SCC, and community-style analytics
- Memgraph and MAGE position graph algorithms as a first-class capability

So the gap is not "a graph database with algorithms."

The gap is narrower and more interesting.

## Core Thesis

The real opportunity is a **persisted, embedded, multi-dimensional graph engine** optimized for:

- simulation-heavy workloads
- algorithm-first access patterns
- lightweight embedding inside developer tools and agents
- graph artifacts that are versioned, queryable, and cheap to traverse

That is different from a general graph database.

## What Is Actually Novel

The differentiator is not:

- BFS exists
- PageRank exists
- SCC exists

Those are table stakes.

The differentiator is the combination of:

- persisted flat storage
- multi-dimensional edge sets
- direct forward and backward adjacency primitives
- cheap algorithm execution over selected dimensions
- versioned graph snapshots
- optional mutation and simulation history
- embedding into code and agent workflows without database-style operational overhead

## The Right Layering

The strongest architecture split is:

1. Generic graph core
   - persisted multi-dimensional graph
   - forward and backward adjacency per dimension
   - flat storage
   - no query language requirement
   - no database theater

2. Parseltongue semantic layer
   - public interface relationship graph
   - code-aware dimensions like `calls`, `imports`, and `contains`
   - tree-sitter extraction and provenance

3. Simulation and analysis layer
   - blast radius
   - SCC
   - PageRank
   - k-core
   - Leiden or community detection
   - coupling/cohesion
   - smart context selection

This means a generic engine can be real without making the product generic.

## What A Shreyas Lens Would Push On

The key product question is:

> What painful job is meaningfully underserved?

"Configurable graph database" is not a strong wedge.

"Public interface graph simulation for code and agents" is much sharper.

That leads to a better strategy:

- build the generic graph engine as infrastructure
- let Parseltongue be the first opinionated application
- do not lead with the infrastructure story until the application proves pull

## Why "Generic Graph Database" Is The Wrong Wedge

If we lead with a generic graph database pitch, we inherit the comparison set of:

- Neo4j
- Kuzu
- ArangoDB
- Memgraph
- TigerGraph

That is a hard category to enter, and it blurs what is actually special about the system.

If we lead with an embedded graph engine for simulation-centric domains, the story is cleaner:

- code graphs
- public interface graphs
- architecture reasoning
- agent memory and retrieval graphs
- other graph-shaped workloads that want persistence plus algorithms without full DB weight

## Product Recommendation

Yes, a separate repo can make sense if the abstraction boundary is real.

But it should be framed as:

- a graph engine
- a graph runtime
- or a persisted graph core

not as a generic graph database on day one.

## Naming Recommendation

A playful codename is fine internally.

For public adoption, a clear name is better.

The public repo and crate should optimize for legibility and recall, not cleverness.

So:

- codename can be whimsical
- public name should be something boring and clear like `flatgraph` or `dimgraph`

## Bottom Line

Parseltongue can absolutely justify building a generic persisted graph engine underneath.

But the market opportunity is not:

> finally, a graph database that can run graph algorithms

The stronger and more honest opportunity is:

> a lightweight, embedded, algorithm-native, multi-dimensional graph engine for simulation-centric workloads, with Parseltongue as the first killer application
