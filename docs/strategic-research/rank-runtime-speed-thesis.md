# Rank Runtime Speed Thesis

## Premise Check

If the goal is:

- embedded
- persisted
- local-machine friendly
- repeated rank-style graph computation
- maximum practical speed

then a **domain-specific persistent rank runtime** is the right direction.

This is different from:

- a generic graph database
- a generic analytics engine
- a generic table runtime

It is also different from Polars.

Polars is a very fast query engine over persisted data.

This thesis is about a runtime that persists not just data, but also the **ranking-ready shape and ranking-ready state**.

## Core Thesis

The fastest practical design for this workload is:

> **a rank-native persistent runtime with a matrix bundle plus warm-start state**

That means the runtime should persist:

- sparse matrix structure
- normalized variants
- degree vectors
- dangling-node masks
- cached rank vectors
- residuals
- convergence checkpoints

This is stronger than simply storing a graph and rerunning PageRank from scratch each time.

## Why Polars Is Not The Same Thing

Polars proves that:

- fast local analytics can win
- lazy execution matters
- persisted file formats can be scanned efficiently

But Polars is still mainly an **engine-first** system.

From the official docs:

- lazy queries run with the **Polars in-memory engine** by default
- a **streaming engine** is available for some larger-than-RAM workflows
- results can be written using sink methods like `sink_parquet`

Sources:

- [Polars LazyFrame docs](https://docs.pola.rs/api/python/stable/reference/lazyframe/)
- [Polars streaming guide](https://docs.pola.rs/user-guide/concepts/streaming/)
- [Polars GitHub](https://github.com/pola-rs/polars)

So Polars is best thought of as:

- a fast engine over persisted data

not:

- a persistent rank runtime designed to remember where ranking algorithms left off

## Best Architecture

The strongest speed-first design has two layers.

### Layer A: Rank Matrix Bundle

Persist the graph in a rank-friendly math shape.

Recommended contents:

- `A_raw`: raw sparse adjacency
- `P_pull`: normalized transition matrix in pull-friendly form
- `A_transpose` or CSC-style view
- `deg_out`: out-degree vector
- `dangling_mask`: dangling-node bitmap or vector
- optional `seed_vectors`
- optional `teleport_vector`

Why:

- ranking workloads repeatedly scan and reweight the same structure
- speed comes from avoiding repeated normalization and setup work

### Layer B: Warm-Start Rank State

Persist the algorithm state itself.

Recommended contents:

- previous rank vector
- current rank vector
- residual vector
- delta norm
- iteration count
- convergence threshold
- changed frontier or dirty blocks
- last successful checkpoint metadata

Why:

- reruns become much faster
- the runtime resumes from “almost converged” instead of restarting from zero

## Why This Is Fast

### 1. It avoids repeated preparation

Most graph systems store only the graph and force ranking code to:

- normalize
- derive degree information
- build transient vectors
- restart iteration

This runtime stores those things already prepared.

### 2. It uses the right storage shape

GraphBLAS exists because many graph algorithms are sparse linear algebra problems.

Sources:

- [GraphBLAS](https://graphblas.org/)
- [GraphBLAS C API](https://graphblas.org/docs/GraphBLAS_API_C_v2.1.0.pdf)
- [LAGraph](https://graphblas.org/LAGraph-Website/)
- [LAGraph algorithms docs](https://lagraph.readthedocs.io/en/latest/algorithms.html)

This supports the idea that rank workloads want:

- matrix-first storage
- transpose-friendly access
- vector-heavy hot paths

### 3. It persists the state the algorithm actually wants

This is the key innovation.

Instead of persisting only:

- input graph

it also persists:

- partially solved rank state

This is what lets reruns become truly fast.

## Three Innovative Directions

## 1. Warm-Start Residual Runtime

Persist:

- last rank vector
- residuals
- dirty frontier

Then resume from near convergence.

Why it matters:

- this turns persistence into actual compute speed, not just load speed

## 2. Multi-View Rank Bundle

Persist:

- raw adjacency
- normalized matrix
- transpose-friendly matrix
- cached output vectors

Why it matters:

- one graph
- several ranking-ready views
- much less recomputation

## 3. Locality-Optimized Permuted Runtime

Persist:

- node permutation
- cluster-aware or block-aware ordering
- block boundaries

Why it matters:

- same algorithm
- better memory locality
- often much faster in practice

## Additional Engineering Patterns

### Pull-friendly orientation

For PageRank-style work, bias the hot path toward:

- CSC
- or a transpose-friendly normalized matrix

This makes pull-style iteration cleaner and often more cache-friendly.

### Hot/cold section split

Hot sections:

- pointers
- indices
- values
- rank vectors
- residuals
- degree vectors

Cold sections:

- schema
- labels
- provenance
- debug payloads

This helps the OS page cache focus on what ranking actually touches.

### Alignment discipline

Arrow’s format guidance emphasizes contiguous, aligned memory and highlights 64-byte alignment as a recommendation for SIMD-friendly access.

Source:

- [Arrow Columnar Format](https://arrow.apache.org/docs/format/Columnar.html)

That suggests:

- align hot numeric arrays
- keep vector loops simple
- isolate cold metadata

### Numeric modes

Support:

- `f32` fast mode
- `f64` validation mode

This gives a practical speed knob without changing the storage contract.

## Related Research Signals

Dynamic and personalized PageRank work supports storing and updating local algorithm state instead of recomputing from scratch.

Examples:

- [Edge-based Local Push for Personalized PageRank](https://arxiv.org/abs/2203.07937)
- [Personalized PageRank on Evolving Graphs with an Incremental Index-Update Scheme](https://arxiv.org/abs/2212.10288)
- [An Incrementally Expanding Approach for Updating PageRank on Dynamic Graphs](https://arxiv.org/abs/2401.03256)
- [Lock-Free Computation of PageRank in Dynamic Graphs](https://arxiv.org/abs/2407.19562)

These papers reinforce a common idea:

- do not restart from zero if the graph changed only a little
- reuse previous state
- update affected areas intelligently

## Ecosystem Context

GitHub snapshot signals checked earlier:

- [Polars](https://github.com/pola-rs/polars): 38k+ stars
- [LAGraph](https://github.com/GraphBLAS/LAGraph): small but serious graph-math project
- [cuGraph](https://github.com/rapidsai/cugraph): GPU-heavy graph analytics

This suggests:

- the world values fast local analytics
- graph ranking still lives across multiple fragmented layers
- there is room for a sharper rank-native persisted runtime

## Practical Recommendation

If the order is:

1. build Walk Graph Runtime first
2. then build Rank Graph Runtime

then the second repo should likely start as:

### MVP

- rank matrix bundle
- pull-friendly sparse matrix orientation
- degree and dangling vectors
- cached score vectors
- benchmark harness for PageRank / PPR / HITS

### V2

- residual persistence
- checkpoint/resume
- warm-start reranking after small changes

### V3

- node reordering for locality
- multiple cached rank families
- tighter matrix-bundle format design

## Bottom Line

If the boss instruction is **maximize speed**, then yes:

the right architecture is not a generic table engine.

It is:

> **a rank-native persistent runtime that stores both the matrix bundle and the algorithm state**

That is the cleanest path to making repeated ranking workloads much faster in a local embedded setting.
