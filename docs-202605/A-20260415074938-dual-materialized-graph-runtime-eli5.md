# Dual Materialized Graph Runtime ELI5

## Big Idea

We do **not** need one magical graph storage format that does every job well.

The better idea is:

- ingest the graph once
- build one canonical snapshot of truth
- then save that same graph in **two runtime-friendly shapes**
  - one for **walking**
  - one for **ranking**

So this is **one graph, one ingest, two physical outputs**.

## Why It Matters

Think of it like the same city being stored in two useful ways.

- One version is a **road map**
- One version is a **score sheet**

If your question is:

- who calls this?
- what does this depend on?
- what breaks if I change this?

you want the **road map**.

If your question is:

- what matters most?
- what is central?
- what should I inspect first?

you want the **score sheet**.

Trying to force one format to be both usually makes both slower and messier.

## Core Ideas Made Simple

### 1. One logical graph, not two different worlds

We are **not** proposing:

- one walk product
- one rank product
- two separate ingests

We are proposing:

- one source graph
- one snapshot build
- two materialized runtime artifacts

In plain words:

the graph truth is one thing, but we save it in more than one useful shape.

### 2. The walk runtime is like a road map

The walk runtime is for:

- callers
- callees
- forward walk
- backward walk
- BFS
- reverse BFS
- blast radius
- local path finding

This storage should look like:

- forward adjacency
- backward adjacency
- compact neighbor lists
- shard and offset tables

That is why the earlier notes kept favoring:

- immutable snapshots
- dual CSR/CSC-like adjacency
- `mmap`-friendly packed sections

This helps because the runtime can jump straight to:

- who this node points to
- who points to this node

without pretending it is a general-purpose database.

### 3. The rank runtime is like a score sheet

The rank runtime is for:

- PageRank
- Personalized PageRank
- HITS
- Katz
- eigenvector-style ranking
- in-degree
- out-degree

These jobs are different.

They are less like:

- “show me this one node’s neighbors”

and more like:

- “scan the graph”
- “update scores”
- “scan again”
- “repeat until the numbers settle”

So this storage should look more like:

- sparse matrix blocks
- normalized views
- degree vectors
- dangling masks
- score vectors
- checkpoints

That is why the rank-runtime notes pointed toward a separate matrix-style artifact.

### 4. What “persistence means we do not depend on RAM” really means

Persistence does **not** mean RAM stops mattering.

It means:

- we do **not** need the whole graph in heap memory at once
- we only need the **working set**

That is the real win.

On a machine with `16GB RAM`, a graph package larger than memory is still possible if:

- the data is immutable
- the hot path is packed for sequential or block-friendly reads
- the runtime keeps only tiny indexes and active blocks in RAM
- the SSD is good

So the honest statement is:

**persistence reduces RAM pressure by shrinking the working set, not by making memory irrelevant.**

### 5. What we learned from Iggy

The useful lesson from the Iggy notes is **not**:

- “copy a broker”

The useful lesson is:

- store bytes in the shape the read path wants
- keep mutable state small
- seal old data into immutable segments
- use tiny sidecar indexes to jump to the right bytes
- reconstruct lightweight runtime state on open

For our graph world, that means:

- one canonical graph snapshot for truth
- one walk artifact shaped for traversal
- one rank artifact shaped for iterative scoring
- small sidecars for search and metrics

### 6. Search is its own thing

Search is not the same job as walking or ranking.

That is why we also talked about sidecars like:

- `FFF`
- exact symbol index
- optional recency metadata

This powers the first step:

- find likely anchors quickly

Then the walk runtime or rank sidecars can take over.

## Tiny Example

Imagine the build pipeline like this:

```text
repo / source graph
       |
       v
canonical snapshot build
       |
       +--> walk artifact
       |      - forward edges
       |      - backward edges
       |      - shard offsets
       |
       +--> rank artifact
       |      - sparse matrix blocks
       |      - degree vectors
       |      - rank checkpoints
       |
       +--> sidecars
              - fff search
              - symbol index
              - pagerank cache
              - scc / kcore / leiden
```

Now the same graph can answer two different styles of question well.

Question A:

> “Who calls `validate_token`?”

Use the **walk artifact**.

Question B:

> “What should I inspect first in this cluster?”

Use the **rank artifact** or a precomputed rank sidecar.

Same graph.
Different shape.
Better fit.

## What This Means For Very Large Graphs

For something like a `60GB` graph package on a `16GB` RAM machine:

- the walk runtime can still be designed to feel live for local traversal work
- the rank runtime can still be correct and useful
- but full global ranking should be thought of as **batch-like**, not instant

That is fine.

The product does not need every operation to be equally fast.

It only needs:

- walk questions to feel responsive
- global rank work to be persisted, resumable, and practical

## What To Remember

The smartest design here is not:

- one graph database
- one storage format
- one giant in-memory object

The smartest design is:

- one graph truth
- one ingest
- two materialized runtimes
- small sidecars for search and cached metrics

## One-Line Takeaway

The same graph should be saved in the shape each job wants, because a road map and a score sheet are both useful, but they are not the same thing.
