# Walk Runtime Options Explainer

## Big Idea

For a **Walk Graph Runtime**, the best design right now is still the simple one:

- save the graph as a frozen snapshot
- open it fast
- let the runtime walk forward and backward edges very cheaply

The main idea is:

**if the job is mostly "walk the map," then the storage should look like a map, not like a general database.**

## Why It Matters

Think of it like city navigation.

- If your job is to **drive around the city quickly**, you want a clean road map.
- You do **not** want a filing cabinet full of street records that you must query every time you make a turn.

That is the same choice here.

For code-scale embedded graphs, we care about:

- cold open time
- low memory use
- simple local tooling
- fast callers/callees and BFS-style traversal

So the real question is:

**what storage shape feels most like a road map?**

## Core Ideas Made Simple

### 1. Immutable Dual CSR/CSC Snapshot

This is the current leading idea for Pensieve.

It stores:

- forward roads
- backward roads
- compact node data
- all inside one frozen snapshot

This is like a very neat atlas where every place already knows:

- where you can go next
- who can arrive here

That is why it scores highest.

### 2. CSR Base Plus Tiny Mutable Overlay

This keeps the fast frozen map, but adds a small scratch pad for recent edits.

This is like:

- one printed city atlas
- plus sticky notes for temporary road changes

This is useful if rebuild-only starts to feel annoying.

But it adds complexity, so it should probably come later, not first.

### 3. Serialized Petgraph Snapshot

This is the practical prototype path.

It is easy to build and easy to explain:

- save the in-memory graph
- load it back later

This is like packing your map into a box and unpacking it every time you want to use it.

That is fine for a prototype, but weaker as a product wedge.

### 4. SQLite or LMDB Adjacency Store

This stores graph relationships in a more database-like way.

This is like keeping road information in a tidy library of index cards.

That helps when you want:

- updates
- queries
- debugging

But it is worse if the main job is just:

- open fast
- walk fast
- stay lightweight

### 5. Packed or Dynamic CSR

This is the clever version of CSR that tries to support more updates.

This is like designing a special expandable road atlas with blank spaces left for future roads.

Interesting idea, but probably too clever for the first product version.

### 6. Edge Table With Indexes

This is the most database-shaped option.

It stores edges like rows in a table and relies on indexes to answer graph questions.

This is like storing a city's roads in spreadsheets and hoping fast lookup makes it feel like a map.

It can work, but it is not centered on the real job.

### 7. Matrix-First Sparse Snapshot

This is a better shape for **Rank Graph Runtime** than **Walk Graph Runtime**.

It is more like a giant score sheet than a street atlas.

That makes it good for repeated math like:

- PageRank
- spectral methods

But less natural for:

- who calls this?
- what can I reach from here?

## Tiny Table

These scores are directional and specific to **embedded code-scale walk runtime** work.

| Architecture | PMF | Walk Speed | Cold Start | Simplicity | Mutation Fit | Debuggability | Embedded Fit | Tradeoff Balance | Overall |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Immutable Dual CSR/CSC Snapshot (`mmap`) | 86 | 97 | 98 | 88 | 22 | 61 | 97 | 91 | 93 |
| CSR Base + Tiny Mutable Overlay | 80 | 90 | 93 | 63 | 69 | 53 | 91 | 74 | 83 |
| Serialized `petgraph` Snapshot | 58 | 71 | 54 | 82 | 76 | 72 | 76 | 64 | 67 |
| SQLite / LMDB Adjacency Store | 49 | 46 | 66 | 69 | 84 | 85 | 71 | 54 | 58 |
| Packed / Dynamic CSR | 38 | 86 | 86 | 34 | 74 | 31 | 74 | 42 | 51 |
| Edge Table + Covering Indexes | 34 | 39 | 69 | 73 | 82 | 89 | 65 | 39 | 46 |
| Matrix-First Sparse Snapshot | 43 | 57 | 79 | 56 | 28 | 42 | 64 | 49 | 53 |

## What The Scores Really Mean

### PMF

This asks:

- would users actually switch because of this?

For walk runtime, the biggest real pain is:

- heavyweight tools for simple graph walking

That is why immutable dual adjacency scores highest.

### Walk Speed

This asks:

- how naturally does this shape support callers, callees, BFS, and reachability?

Road-map shaped storage wins here.

### Cold Start

This asks:

- how fast can the tool open and start answering?

`mmap` snapshots win because they do not need much unpacking.

### Simplicity

This asks:

- how easy is this to build, explain, and maintain?

Simple usually wins longer than clever.

### Mutation Fit

This asks:

- how well does this handle change without a rebuild?

This is the main weakness of frozen snapshots, but for code-scale local graphs that weakness may be acceptable.

## What To Remember

The best walk runtime is probably not the most general system.

It is probably the system that is best at one humble job:

- open a local graph fast
- walk it fast
- stay simple

That is why the current thesis still points to:

- **immutable dual forward/backward adjacency snapshots**

**For a walk runtime, the winning design is the one that behaves most like a map and least like a database.**
