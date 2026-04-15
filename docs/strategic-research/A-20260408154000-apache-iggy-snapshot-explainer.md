# Apache Iggy Snapshot Explainer

## Big Idea

If Parseltongue is **snapshot only**, Apache Iggy can be used to build a real persisted graph system, but it should be treated like a very good log-backed storage layer, not like a magical graph engine.

In simple words:

- **Iggy can store the graph snapshot well**
- **Iggy cannot make graph walking free**
- **`petgraph` will still be faster for hot in-memory graph work**

So the honest answer is:

- **yes, this can work**
- **no, it will not beat `petgraph` on raw traversal speed**
- **yes, it may still be good enough for a serious product if designed around snapshots**

## Why It Matters

Think of this like choosing between:

- a **warehouse** that is very good at storing boxes in order
- and a **workbench** where all your tools are already laid out in front of you

Apache Iggy is more like the warehouse.

It is very good at:

- writing data in order
- keeping it durable
- reopening it later
- reading it back in structured chunks

`petgraph` is more like the workbench.

It is very good at:

- keeping graph nodes and edges in memory
- following pointers quickly
- doing graph walks with very little overhead

If your product needs **persisted snapshots** and you are willing to rebuild them when reindexing, Iggy becomes much more reasonable.

That is because snapshot-only work removes the hardest part:

- no constant graph mutation
- no branch merges
- no fancy online updates
- no pretending this is a general graph database

## Core Ideas Made Simple

### 1. Why Snapshot-Only Helps So Much

The hardest part of graph storage is usually not saving the graph once.

The hardest part is:

- changing it cheaply
- keeping indexes up to date
- supporting many query patterns at once

Your current assumption removes most of that pain.

If the rule is:

- parse the repo
- build one graph snapshot
- write it once
- recreate it when needed

then Iggy is playing a much friendlier game.

It can do what it likes best:

- append data
- index it by position
- reopen it quickly

### 2. What "Good Iggy Design" Would Actually Mean

The wrong design is:

- one message per edge
- one query = scan a giant bucket
- hope the broker somehow acts like an adjacency store

That would work, but badly.

The better design is:

- one snapshot = one stream
- one topic for entities
- one topic for outgoing calls
- one topic for incoming calls
- one topic for containment edges
- one topic for exports/public interface
- one topic for manifest metadata

Most importantly:

- store one **packed adjacency blob** per entity, not one tiny message per edge

That means each entity can say:

- here is who I call
- here is who calls me
- here is what I contain

all in one compact stored chunk.

### 3. Why `petgraph` Is Still Faster

`petgraph` lives in memory.

That means it can usually answer graph questions by:

- following direct pointers
- touching fewer layers
- avoiding network or broker protocol overhead

Iggy-backed traversal has more work:

1. find the right partition or record
2. read the stored blob
3. decode it
4. possibly repeat for the next hop

So even if Iggy is fast **for persisted storage**, it is still doing more work per graph hop than an in-memory graph library.

### 4. The Right Mental Model

Do not think:

> Iggy is replacing `petgraph`

Think:

> Iggy is replacing a custom persisted snapshot file, while `petgraph` remains the speed baseline for hot graph traversal.

That is the more honest comparison.

## Tiny Example

Imagine one function called `main`.

We want to answer:

- what does `main` call?
- who calls `validate`?

In a good Iggy snapshot design:

- `main` has a stored outgoing-adjacency blob
- `validate` has a stored incoming-adjacency blob
- a small manifest helps us jump to the right record fast

So the query is not:

- scan all edges everywhere

It is:

- find the entity record
- load the packed neighbor list
- decode the neighbor IDs

That is why this can be fast enough.

It is still slower than `petgraph`, but not hopelessly slow.

## Numbers To Expect

These are **estimates**, not measured benchmarks.

They assume:

- Linux
- local same-machine deployment
- snapshot is immutable after build
- stable entity IDs
- forward and reverse edges are precomputed
- adjacency is stored as one compact record per entity per dimension
- a small manifest is loaded into RAM on open

### Working-System Score

If built well, I would score it like this:

| system view | score out of 100 |
| --- | ---: |
| Can this be a real working system? | 82 |
| Raw graph-query speed on Iggy | 58 |
| Raw graph-query speed on `petgraph` | 95 |

### Rough Query Estimates

| workload | Iggy snapshot design | `petgraph` in memory | rough slowdown |
| --- | ---: | ---: | ---: |
| Entity lookup, hot | `80-400 us` | `80-500 ns` to `2 us` | `40x-1000x` |
| Outgoing adjacency, one node | `120-700 us` | `150 ns-3 us` | `40x-500x` |
| Incoming adjacency, one node | `120-800 us` | `150 ns-3 us` | `40x-500x` |
| Blast radius depth 2 | `3-20 ms` | `0.1-2 ms` | `5x-50x` |
| Blast radius depth 3 | `15-120 ms` | `1-15 ms` | `8x-80x` |
| Full snapshot rebuild, medium repo artifact | `1-4 s` | `0.5-2.5 s` | `1.2x-2.5x` |

### What These Numbers Mean In Plain English

- single-hop questions can still feel fast enough for a desktop app
- deeper traversals will show the gap much more clearly
- rebuild time is much less scary than traversal time

That last point matters a lot.

Because you said **reindexing is allowed**, the main question is not:

- "is rebuild ultra-cheap?"

The main question is:

- "are the interactive queries fast enough after rebuild?"

My answer is:

- **probably yes for focused graph questions**
- **definitely slower than `petgraph`**
- **still usable if the UX is designed around that reality**

## Reality Check

As of **April 8, 2026**, the latest official Apache Iggy release is **`0.7.0-incubating`**, published on **February 24, 2026**.

The official docs describe Iggy as a streaming system built around:

- streams
- topics
- partitions
- offsets
- segments

That is why the right stance is:

- respect Iggy for what it is
- do not pretend it is a native graph store

Its real strength is:

- durable ordered storage
- compact persisted access paths
- predictable append-first behavior

## What To Remember

If we go snapshot-only, Apache Iggy can absolutely power Parseltongue's persisted graph layer, but it should be used like a fast log-backed warehouse for packed graph records, not like a substitute for an in-memory graph engine.

The sticky sentence is:

**Iggy can store the world well, but `petgraph` is still better at running around inside it.**
