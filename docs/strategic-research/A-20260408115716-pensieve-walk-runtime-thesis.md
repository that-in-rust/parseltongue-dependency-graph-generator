# Pensieve Walk Runtime Thesis

## Core Thesis

Pensieve should currently be understood as a **Walk Graph Runtime**.

That means:

- it persists graph snapshots
- it opens them quickly
- it exposes forward and backward adjacency by dimension
- it makes graph traversal cheap and predictable

It should **not** currently be framed as:

- a general graph database
- a semantic code analysis engine
- a cross-snapshot simulation engine
- a universal runtime for every graph algorithm family

The strongest, most honest product statement today is:

> Pensieve is an embedded, persisted, multi-dimensional runtime for walking one frozen graph world really well.

## Why This Thesis Matters

It gives us a sharp boundary.

If we do not pick this boundary, Pensieve gets pulled in too many directions:

- query language
- schema evolution
- mutation support
- semantic interpretation
- graph diff
- architecture simulation

That would turn a strong systems idea into a blurry platform project.

The walk-runtime thesis keeps the center of gravity clear.

## What "Walk Graph Runtime" Means

The dominant questions are:

- what does this node connect to?
- who points to this node?
- what can I reach from here?
- what breaks if I start from here?
- which nodes are nearby in 1, 2, or 3 hops?

These questions are all adjacency-first.

They care most about:

- fast open time
- low memory overhead
- cheap neighborhood access
- predictable traversal latency

That is exactly what Pensieve's current architecture is good at.

## Category Definition

Pensieve belongs in this narrower category:

- persisted graph runtime
- embedded graph engine
- snapshot-first graph substrate
- walk-optimized graph store

This is different from:

- graph database
- distributed graph analytics platform
- semantic simulation engine

## Product Boundary

### Pensieve should own

- graph snapshot format
- node and edge persistence
- per-dimension forward adjacency
- per-dimension backward adjacency
- dimension dictionaries
- graph projections over selected dimensions
- traversal-friendly algorithms
- lightweight metadata and index sections

### Pensieve should not own

- tree-sitter ingestion
- code entity taxonomy
- public-interface policy
- stable entity identity across snapshots
- semantic graph delta
- architecture simulation
- LLM-specific context policies

Those higher-order jobs belong above Pensieve, in Parcel10 or a sibling product layer.

## Storage Thesis

The current best storage thesis for Pensieve is:

- immutable snapshot files
- compact node records
- per-dimension forward adjacency
- per-dimension backward adjacency
- memory-mapped open path
- zero-copy or near-zero-copy reads

In practice, this means a layout like:

1. header
2. node table
3. dimension dictionary
4. forward offsets
5. forward peers
6. backward offsets
7. backward peers
8. string pool
9. lookup index sections

This design is strong because it turns the most common graph question into:

- find offsets
- slice neighbors
- keep walking

## Why This Is A Good Fit For Code-Scale Data

For local code graphs, the data is usually small enough that:

- snapshot duplication is cheap
- rebuilds are acceptable
- developer-machine storage is not the constraint

That means we can optimize for:

- simple architecture
- good cold start
- low operational overhead
- easy embedding into tools

instead of over-optimizing for:

- online mutation
- giant distributed graphs
- storage minimization at all costs

## Algorithm Fit

Pensieve is strongest in the **Walk The Graph** family:

- BFS
- DFS
- reachability
- reverse reachability
- callers / callees
- blast radius
- dead code from roots
- shortest path

Pensieve is also a good substrate for some nearby families:

- rank algorithms like PageRank and degree-based scoring
- shape algorithms like SCC, k-core, and community detection

But those are extensions of the core thesis, not the center of it.

The center is still:

> adjacency walking on one frozen graph snapshot

## Why Not A General Graph Database

If Pensieve is pitched as a graph database, it will be compared against:

- Neo4j
- Kuzu
- ArangoDB
- Memgraph
- DuckPGQ-style graph querying

That is the wrong comparison set.

Those systems are evaluated on:

- query languages
- transactions
- mutable writes
- concurrent access
- ecosystem integrations

Pensieve is strongest on a different axis:

- embedded use
- local tooling
- cold open time
- low overhead
- direct algorithmic control

So the graph-database framing hides what is actually special.

## Why Not Simulation Yet

Simulation sounds nearby, but it is a different layer.

Walking one graph asks:

- what is true in this world?

Simulation asks:

- how do world A and world B relate?
- what stayed the same?
- what moved?
- what changed semantically?

That requires:

- stable identity
- reconciliation across snapshots
- semantic diff logic
- policy and interpretation

Pensieve should store the worlds.

Parcel10 should compare them.

## Recommended MVP

If we stay faithful to the walk-runtime thesis, Pensieve MVP should include:

- immutable snapshot writer
- immutable snapshot reader
- per-dimension forward traversal
- per-dimension backward traversal
- exact-name or key lookup
- subgraph projection by dimension
- BFS / DFS / reachability utilities
- strong benchmarking on cold open and traversal

It should explicitly defer:

- mutation layers
- transactional semantics
- semantic diff
- query languages
- LLM-facing heuristics

## Strategic Read

This thesis is good strategy because it makes Pensieve:

- smaller
- more legible
- easier to benchmark
- easier to adopt as a library
- harder to confuse with a half-built graph database

It also preserves optionality.

If later we want:

- rank runtime
- shape runtime
- batch runtime

those can be built beside the walk runtime instead of forcing one format to do every job.

## Bottom Line

Pensieve does not need to be the universal graph engine.

It only needs to be the best embedded runtime for **walking one persisted graph snapshot**.

That is already a strong and useful product.
