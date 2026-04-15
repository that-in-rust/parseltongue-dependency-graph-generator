# 202603221242 Architecture Simulation Thesis

## Core shift

Parseltongue should evolve from a code search / graph analysis tool into a
multi-resolution architecture simulation engine.

The key idea is:

- represent the current architecture as a graph
- apply hypothetical structural mutations to that graph
- compute exact deltas and ramifications
- let the LLM interpret the meaning of those deltas

This is a much stronger framing than "replace grep". It turns Parseltongue into
an engine for understanding, comparing, and stress-testing architectural ideas
before code is written.

## The clean separation of labor

### Graph engine

The graph engine should do the exact, deterministic work:

1. store graph state
2. materialize multiple levels of detail
3. apply structural mutations
4. compute graph deltas
5. recompute metrics and ramifications
6. serialize structured outputs

### LLM

The LLM should do the interpretive work:

1. explain what the delta means
2. suggest what code will likely need to change
3. rank candidate architectural alternatives
4. surface human-readable risks and tradeoffs

The LLM should not be the source of structural truth.

The graph engine should not try to "understand intent" in natural language.

## The core interaction model

The simplest powerful loop is:

1. mutate
2. diff
3. explain

Example:

- add an edge
- remove an edge
- collapse a node
- split a node
- extract an interface

After the mutation, the engine computes:

- newly reachable nodes
- newly unreachable nodes
- new boundary crossings
- new SCCs or removed SCCs
- centrality changes
- k-core changes
- community changes
- blast radius changes

The LLM then explains what that likely means for the architecture and code.

## The graph should be multi-resolution

This system should not expose one flat graph only.

It should support semantic zoom.

### LOD 0

- workspace
- crate
- subsystem

### LOD 1

- module
- public interface
- community

### LOD 2

- function
- method
- type
- impl

### LOD 3

- selected control-flow slices
- selected data-flow slices
- selected compiler evidence

The important part is that zoom is semantic, not just visual.

## Why the database matters

The database is not only for storing raw graph facts.

It should persist:

1. raw graph facts
2. materialized architecture views
3. multi-level aggregates
4. scenario overlays
5. metric caches
6. branch comparisons

This allows fast zoom transitions, cached ramifications, and interactive
simulation instead of recomputing everything from raw edges every time.

This is the code-graph equivalent of a warehouse with materialized rollups.

## The graph model

### Minimum node shape

Each node should be richly typed enough that the delta is self-describing.

Suggested fields:

- `id`
- `kind`
- `qualified_name`
- `visibility`
- `language`
- `boundary_tags`
- `file_path`
- `line_range`

Useful boundary tags:

- `Public`
- `Async`
- `ThreadShared`
- `IoPath`
- `Persistence`
- `ExternalDependency`

### Minimum edge shape

Suggested fields:

- `from`
- `to`
- `kind`
- `crossing`
- `weight`
- `plane`

Likely edge kinds:

- `Contains`
- `DependsOn`
- `Calls`
- `Implements`
- `InheritsFrom`
- `TypeRef`
- `DataFlow`
- `ControlDep`

Likely crossing types:

- `AsyncBoundary`
- `ThreadBoundary`
- `CrateBoundary`
- `PersistenceBoundary`

## Graph planes

It is more accurate to think in planes or layers than in "phases".

The main planes are:

1. module / crate dependency graph
2. type / trait graph
3. call graph
4. data-flow / control-dependency graph

The value comes from being able to:

- inspect them independently
- project across them
- compute ramifications across them

## Mutation primitives

The first mutation vocabulary should stay small and structural.

Recommended v1 primitives:

1. `add_edge`
2. `remove_edge`
3. `collapse_node`
4. `split_node`
5. `extract_interface`

These are enough to support meaningful architecture experiments without trying
to simulate arbitrary code edits.

## Structured delta packets

The engine output should be structured first, prose second.

Example shape:

```json
{
  "mutation": {
    "kind": "add_edge",
    "from": "connector::MongoSink::consume",
    "edge_type": "Calls",
    "to": "connector::poll_messages"
  },
  "delta": {
    "new_reachable": [
      "runtime::io_submit",
      "state::AtomicU64_counter"
    ],
    "new_boundary_crossings": [
      {
        "at": "consume -> poll_messages",
        "type": "AsyncBoundary"
      },
      {
        "at": "poll_messages -> AtomicU64_counter",
        "type": "ThreadShared"
      }
    ],
    "scc_changes": [
      {
        "kind": "introduced_cycle",
        "members": ["consume", "poll_messages"]
      }
    ],
    "centrality_delta": {
      "poll_messages": {
        "betweenness": 0.34
      }
    }
  }
}
```

This packet is what the LLM reads.

## The LLM contract

The LLM should receive:

1. the mutation
2. the affected subgraph
3. the metric deltas
4. the boundary crossings
5. confidence / provenance data
6. candidate files or entities to inspect next

The LLM should not receive a giant code dump unless the user intentionally
zooms into a region.

## Jarvis behavior

The system should feel like:

1. start coarse
2. find the relevant subsystem
3. fade unrelated clusters
4. zoom to the correct abstraction layer
5. simulate a structural change
6. show ramifications
7. compare alternatives
8. only reveal details when they matter

That means the engine must support:

- semantic zoom
- progressive disclosure
- precomputed rollups
- fast scenario comparison

## The important novelty

The novel thing here is not:

- storing a graph
- having 30 algorithms
- having a DSL

The novel thing is:

an LLM-facing architecture simulation engine for large codebases where the graph
engine computes exact structural consequences and the LLM interprets them.

That is the real thesis.

## Recommended v1

Do not start with every plane.

Start with:

1. interface dependency graph
2. call graph
3. boundary tags
4. simulation overlays
5. structured delta packets

That is enough to prove the mutation -> diff -> explain loop.

## One-line framing

Parseltongue should become a multi-resolution architecture simulation engine
that lets humans and LLMs mutate structure, inspect exact graph deltas, and
compare architectural alternatives before code is written.
