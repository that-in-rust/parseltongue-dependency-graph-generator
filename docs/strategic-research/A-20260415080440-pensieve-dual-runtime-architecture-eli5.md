# One Graph, Two Maps: The Pensieve Architecture In Plain English

## Big Idea

When a code graph is too big for RAM, the trick is not to load less of it.
The trick is to store it in a shape that only ever needs the working piece in RAM.

And the best shape depends entirely on what you are trying to do.

If the question is **"who calls this?"**, the graph should look like a road map.
If the question is **"what is most important?"**, the graph should look like a score sheet.

Trying to force one format to answer both questions well usually makes both answers worse.

The Pensieve architecture solves this by doing one ingest, then saving the same graph in two different shapes.

---

## Why It Matters

Imagine you are a cartographer.

You have one city.
You need to produce two products from that city:

- a **road atlas** for drivers who want to navigate from place to place
- a **heatmap** for urban planners who want to know which intersections matter most

You would not try to draw one map that does both jobs.
You would produce two outputs from the same city data.

That is exactly what Pensieve does with a code graph.

---

## Core Ideas Made Simple

### 1. One logical graph, not two separate worlds

This is the most important thing to say clearly up front.

We are **not** proposing:

- one walk product
- one rank product
- two separate ingests by the user

We are proposing:

- one source graph
- one canonical snapshot build
- two materialized runtime artifacts

The graph truth is one thing.
We just save it in more than one useful shape.

```text
source code
    |
    v
canonical snapshot build
    |
    +--> walk artifact      (road atlas)
    |
    +--> rank artifact      (score sheet)
    |
    +--> sidecars           (symbol index, cached metrics, search index)
```

### 2. The canonical snapshot is the source of truth

Before either materialization happens, the system builds one canonical snapshot.

Think of this as the city database.
It contains every street, building, and block, described precisely.

Fields include:

- stable entity ids
- edges with dimension labels (`calls`, `imports`, `contains`)
- visibility and confidence metadata
- a manifest that describes the whole snapshot

Neither runtime touches this directly for live queries.
It is the authoritative record that both materializations are built from.

### 3. The walk artifact is the road atlas

The walk artifact is for questions like:

- what does this function call?
- who calls this function?
- what breaks if I change this?
- what can I reach from here in two hops?

The storage shape for this is called **dual CSR/CSC adjacency**.

CSR means: for every node, here is a packed list of where it goes.
CSC means: for every node, here is a packed list of who points to it.

In plain words:

- instead of asking a database "find all edges from node A," you jump straight to a pre-packed neighbor list
- that list is stored as one contiguous slice of bytes on disk
- the runtime reads it directly, no query engine needed

For a 60GB graph on 16GB RAM, the walk runtime keeps only tiny things always in RAM:

- the manifest (tiny)
- a node-to-shard lookup table (tiny)
- a small LRU page cache

The actual neighbor lists live on disk (NVMe).
The runtime reads the slice it needs, for the node it is visiting, only when it needs it.

That is why local traversal can still feel live even when the graph is larger than RAM.

### 4. The walk artifact is sharded by node range

The graph is not stored as one giant file.
It is split into shards, where each shard owns a consecutive range of node IDs.

This matters because:

- nodes that are close in the graph tend to be close in numbering after a locality-preserving reorder
- so a BFS walk over 2-3 hops usually touches only a few shards
- which means only a few pages of NVMe data need to be read

The design rule: **never store one edge as one tiny record fetch. Store one node's neighborhood as one packed slice.**

### 5. The rank artifact is the score sheet

The rank artifact is for questions like:

- what is most central in this codebase?
- what should I look at first?
- what has the highest PageRank score?
- what is the seeded importance starting from this function?

These algorithms do not walk from one node to its neighbors.
They sweep the whole graph, update scores, sweep again, and repeat until the numbers settle.

That means the storage shape must be completely different:

- a sparse matrix bundle (normalized for iterative scoring)
- degree vectors (precomputed, used on every iteration)
- dangling node masks (which nodes have no outgoing edges)
- score vectors (`rank_curr`, `rank_prev`, `residual`)
- checkpoints (so reruns can resume from near-convergence, not from zero)

In plain words: the rank runtime stores not just the graph, but the half-finished math state.
So the next time you ask for PageRank after a small code change, it resumes from where it left off rather than starting over.

### 6. What "interactive" vs "batch" means honestly

For a 60GB graph on 16GB RAM:

| Question type | Feel |
|---|---|
| "Who calls this function?" | Interactive — can feel live |
| "2-hop blast radius from this node" | Interactive — a few NVMe reads |
| "Local seeded PageRank from this anchor" | Responsive if bounded |
| "Full global PageRank for the whole graph" | Batch — correct, but not UI-instant |

This is fine.
The product does not need every operation to feel equally fast.
It only needs the right operations to feel fast.

### 7. What Pensieve learned from Apache Iggy

The research in this folder drew heavily on how Apache Iggy (a streaming message broker) manages storage.

The lesson is **not** "copy a message broker into a graph engine."

The lesson is a storage doctrine:

- **shape bytes around the read path** — store data in the layout the query wants, not in a general format
- **keep mutable state small** — only the working slice changes; old data is sealed
- **use tiny sidecar indexes** — a small jump table in RAM points to the right bytes on disk
- **cheap reopen** — on startup, load only the manifest and jump tables, not the whole graph

For the walk runtime, "shape bytes around the read path" means: pack all of node A's neighbors together as one contiguous block.
For the rank runtime, it means: store the normalized matrix and the score vectors already in the form the iteration loop wants.

### 8. The algorithm cluster map

Not every algorithm fits equally well in each runtime.
Here is the honest summary:

| Algorithm family | Best runtime | Feel |
|---|---|---|
| BFS, DFS, reachability | Walk | Native |
| Callers / callees / blast radius | Walk | Native |
| Dead code from roots | Walk | Native |
| In-degree / out-degree | Rank sidecars | Trivial precompute |
| Global PageRank | Rank | Batch |
| Seeded / local PPR | Rank | Responsive if bounded |
| HITS, Katz, eigenvector | Rank | Batch |
| SCC, k-core | Shape-style pass | Good on static graph |
| Leiden / community detection | Shape-style pass | Good on static graph |
| Betweenness, closeness | Precompute only | Too expensive for hot path |
| Smart context selection | Parcel10, not Pensieve | Needs code semantics + product policy |
| Architecture simulation | Parcel10, not Pensieve | Needs multi-snapshot identity reconciliation |

### 9. Pensieve vs Parcel10

Two names appear throughout this research.
Here is the clean split:

**Pensieve** is the graph runtime layer.

- persists graph snapshots
- exposes forward and backward adjacency by dimension
- runs graph-native algorithms
- is domain-agnostic

**Parcel10** is the semantic product layer built on top.

- understands tree-sitter code entities
- defines what "public interface" means
- compares two snapshots and explains what changed
- produces architecture simulations and LLM-facing explanations

In the ELI5 version:

- Pensieve stores worlds
- Parcel10 compares and explains worlds

### 10. The LLM-facing pipeline (from the imperfect userflow note)

The most useful product framing for v302 is not "a graph runtime" or "a graph explorer."

It is a local graph query pipeline for LLMs, shaped like this:

```text
RETRIEVE   → FFF fuzzy search finds likely code anchors
ANCHOR     → the LLM picks the best starting point
ENRICH     → the walk runtime returns neighbors and edges
FILTER     → narrow by hops, exclude tests or comments
RANK       → score candidates by importance
BUDGET     → fit the result into a token limit
OUTPUT     → return a small trustworthy packet
```

The Tauri desktop app is the control room (start/stop the local server, manage indexes, show status).
The walk runtime is the engine under the hood.
The pipeline is the actual product the LLM sees.

### 11. Why not just use Kuzu or Neo4j?

Kuzu was a strong embedded graph database before its acquisition.
Neo4j GDS supports most of the same algorithms.

But graph databases carry overhead:

- query language surfaces
- property-graph semantics
- transaction models
- general storage responsibilities

A narrower walk runtime can drop those costs and optimize purely for:

- cold open
- forward and backward traversal
- BFS / DFS / reachability
- blast radius style walking

The competitive bet is not "build a better graph database."
The competitive bet is "build the best runtime in one narrow workload family."

---

## Tiny Example

Imagine a codebase with a `login_handler` function.

An LLM is trying to understand: "what breaks if password verification changes?"

**Step 1 — RETRIEVE**
FFF fuzzy search returns: `login_handler`, `verify_password`, `load_user_record`, `issue_session`

**Step 2 — ANCHOR**
The LLM picks `login_handler` as the center because it is public-facing.

**Step 3 — ENRICH (walk runtime)**
The walk artifact serves the outgoing neighbor slice for `login_handler`:
```text
login_handler -> load_user_record
login_handler -> verify_password
login_handler -> issue_session
```

The reverse (CSC direction) gives the backward neighbors of `verify_password`:
```text
verify_password <- login_handler
```

**Step 4 — FILTER**
The LLM narrows to 2 hops, excludes tests.

**Step 5 — RANK**
The rank sidecar scores: `login_handler` (highest), `verify_password` (second).

**Step 6 — BUDGET**
The LLM requests: top 2 nodes in under 1000 tokens.

**Step 7 — OUTPUT**
```json
{
  "anchor": "login_handler",
  "top_nodes": ["login_handler", "verify_password"],
  "edges": [
    ["login_handler", "load_user_record"],
    ["login_handler", "verify_password"],
    ["login_handler", "issue_session"]
  ],
  "freshness": "fresh",
  "confidence": "tree_sitter_structural"
}
```

The walk artifact answered steps 3 and 4.
The rank sidecar answered step 5.
Neither required loading 60GB into RAM.

---

## What To Remember

The winning pattern for a large graph on limited RAM is not to load less data — it is to store the data in the shape each job wants, load only tiny jump tables plus the active working slice, and let the rest live on NVMe until it is needed.

One graph.
One ingest.
Two materialized runtimes.
Small sidecars for search and cached metrics.

**A road map and a score sheet are both useful, but they are not the same thing — and a good graph system does not pretend otherwise.**

---

## Source Notes

This explainer synthesizes the following research documents in this folder:

- `dual-materialized-graph-runtime-eli5.md` — core dual materialization concept
- `walk-runtime-options-explainer.md` — walk storage options and scoring
- `walk-runtime-options-explainer.md` / `pensieve-walk-runtime-thesis.md` — walk runtime architecture
- `rank-runtime-ondisk-format.md` / `rank-runtime-speed-thesis.md` — rank runtime format and speed thesis
- `csr-csc-iggy-graph-walking-eli5.md` — CSR/CSC mechanics explained
- `apache-iggy-snapshot-explainer.md` — Iggy storage doctrine
- `iggy-storage-meta-patterns.md` / `iggy-storage-pattern-research.md` — storage meta-lessons
- `pensieve-runtime-architecture-summary.md` — Pensieve vs Parcel10 boundary
- `pensieve-algorithm-cluster-map.md` — algorithm cluster assignments
- `pensieve-pmf-cluster-ratings.md` — PMF ratings by cluster
- `pensieve-walk-runtime-thesis.md` — walk-runtime-first thesis
- `walk-runtime-product-map.md` — product market fit map for walk runtime
- `generic-graph-engine-thesis.md` — why "generic graph DB" is the wrong pitch
- `global-public-interface-thesis.md` — public interface graph as the real product wedge
- `kuzu-runtime-competition-thesis.md` — competitive analysis vs Kuzu
- `flatgraph-algorithm-fit-summary.md` — algorithm fit summary for flatgraph
- `build-time-walk-time-visualization-eli5.md` — build time vs walk time separation
- `imperfect-userflow-01.md` — LLM-facing pipeline framing for v302
- `walk-rank-storage-explainer.md` — plain English comparison of walk vs rank storage
- `Parseltongue-Iggy-SQLlite-Arch-Options01.md` — original architecture options research
