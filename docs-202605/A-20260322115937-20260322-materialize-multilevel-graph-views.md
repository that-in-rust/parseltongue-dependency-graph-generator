# 20260322 Multi-Level Graph Materialization Note

## Core idea

Parseltongue should not just persist a raw code graph. It should persist
multiple precomputed levels of architectural understanding so the system can
zoom in, zoom out, fade unrelated regions, and answer consequence questions
quickly.

This is the code-graph equivalent of data warehousing with materialized
aggregates.

## Why this matters

If every query starts from raw nodes and raw edges, then:

- zooming is slow
- architecture comparison is slow
- simulation is expensive
- LLM interaction becomes choppy
- the system feels like a research demo instead of a product

If we precompute graph views and metrics at multiple levels, then the system can
behave more like Jarvis:

- start coarse
- identify the relevant region
- zoom into the right abstraction level
- fade unrelated clusters
- surface ramifications quickly
- compare alternative architectures without re-deriving everything from scratch

## The thesis

The database is not only for storing graph facts.

The database is for storing:

1. raw graph facts
2. materialized architectural views
3. multi-level aggregates
4. simulation overlays
5. cached metric summaries
6. branch-to-branch comparisons

This makes the graph database a persistent architectural memory, not just a
query backend.

## The levels that should exist

### Raw graph facts

- entities
- edges
- edge types
- source locations
- visibility
- ownership / containment

### Materialized architecture views

- function-level graph
- file-level graph
- module-level graph
- public-interface graph
- community / subsystem graph

### Cached metrics

- fan-in / fan-out
- SCC membership
- centrality
- k-core layer
- modularity / community assignment
- coupling / cohesion
- entropy
- blast radius summaries
- token-budget summaries

### Simulation state

- graph deltas
- scenario branches
- metric deltas
- baseline vs candidate comparisons

## The key product behavior

This is what the system should be able to do:

1. show the graph at a high level
2. zoom into a relevant subsystem
3. expand only the most important clusters
4. fade or collapse unrelated structure
5. run ramifications for a proposed change
6. compare multiple candidate architectures
7. feed only the right level of detail to an LLM

## Database plus compute split

The right split is:

- database for remembered structure
- RAM for active computation

The database should keep durable graph snapshots, rollups, simulation branches,
and cached metrics.

The compute engine should run graph algorithms, recompute affected aggregates,
and generate focused subgraphs on demand.

## The big insight

Parseltongue should behave less like "search over code" and more like a
multi-resolution architecture warehouse with graph-native simulation.

That is what unlocks:

- progressive disclosure
- semantic zoom
- fast architecture queries
- branch comparison
- Jarvis-style guided exploration

## One-line framing

Parseltongue should become a multi-resolution architecture simulation engine
where precomputed graph materializations make large-codebase understanding,
zooming, and consequence analysis fast enough to feel interactive.
