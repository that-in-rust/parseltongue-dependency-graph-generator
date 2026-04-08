# Rank Runtime Ondisk Format

## Premise Check

If the goal is:

- embedded
- persisted
- local-machine friendly
- repeated rank-style graph computation
- maximum practical speed

then the best concrete on-disk design is not just a sparse matrix file.

It is a **rank-native persisted runtime artifact**.

That means it should persist not only:

- graph structure

but also:

- normalized math views
- hot vectors
- resumable algorithm state

## Core Thesis

The best concrete on-disk format for a speed-first Rank Graph Runtime is:

> **a sectioned binary file with a matrix bundle plus a warm-start state bundle**

This lets the runtime avoid:

- repeated normalization
- repeated vector setup
- repeated cold restarts
- repeated graph reshaping

## Why Polars Is Adjacent But Different

Polars is a fast engine over persisted data.

From official docs:

- lazy execution uses the **Polars in-memory engine** by default
- Polars also supports a **streaming engine**
- sink methods can write outputs like Parquet

Sources:

- [Polars LazyFrame docs](https://docs.pola.rs/api/python/stable/reference/lazyframe/)
- [Polars streaming docs](https://docs.pola.rs/user-guide/concepts/streaming/)
- [Polars GitHub](https://github.com/pola-rs/polars)

So Polars shows that:

- fast local analytics can win

But it does **not** persist the exact algorithm state that ranking workloads want.

## Why Matrix-First Storage Makes Sense

GraphBLAS and LAGraph exist because many ranking algorithms are sparse linear algebra problems.

Sources:

- [GraphBLAS](https://graphblas.org/)
- [GraphBLAS C API](https://graphblas.org/docs/GraphBLAS_API_C_v2.1.0.pdf)
- [LAGraph](https://graphblas.org/LAGraph-Website/)
- [LAGraph algorithms docs](https://lagraph.readthedocs.io/en/latest/algorithms.html)

This supports:

- matrix-first storage
- transpose-friendly layouts
- vector-heavy hot paths

## Why Alignment Matters

Arrow’s columnar format guidance reinforces the value of:

- contiguous buffers
- O(1) access
- 64-byte alignment when possible

Source:

- [Apache Arrow Columnar Format](https://arrow.apache.org/docs/format/Columnar.html)

This suggests rank runtime hot arrays should be aligned and physically separated from cold metadata.

## Related Research Signal

Dynamic and personalized PageRank work strongly suggests reusing state rather than recomputing from zero.

Examples:

- [Edge-based Local Push for Personalized PageRank](https://arxiv.org/abs/2203.07937)
- [Personalized PageRank on Evolving Graphs with an Incremental Index-Update Scheme](https://arxiv.org/abs/2212.10288)
- [An Incrementally Expanding Approach for Updating PageRank on Dynamic Graphs](https://arxiv.org/abs/2401.03256)
- [Lock-Free Computation of PageRank in Dynamic Graphs](https://arxiv.org/abs/2407.19562)

This supports storing:

- residuals
- frontiers
- checkpoints
- partially converged vectors

## Proposed File Layout

```text
+-----------------------------+
| Header                      |
+-----------------------------+
| Section Directory           |
+-----------------------------+
| Node Metadata               |
+-----------------------------+
| Raw Graph Structure         |
+-----------------------------+
| Rank Matrix Bundle          |
+-----------------------------+
| Hot Vectors                 |
+-----------------------------+
| Warm Start State            |
+-----------------------------+
| Cached Outputs              |
+-----------------------------+
| Optional Debug/Provenance   |
+-----------------------------+
| Footer / Checksums          |
+-----------------------------+
```

## 1. Header

Purpose:

- identify format
- define version
- record scalar/index sizes
- declare present sections

Suggested fields:

- magic bytes
- format version
- file UUID
- graph UUID
- node count
- edge count
- scalar type (`f32` / `f64`)
- index type (`u32` / `u64`)
- flags
- checksum mode

## 2. Section Directory

Purpose:

- locate each section quickly
- keep the format extensible

Suggested fields per section:

- section id
- byte offset
- byte length
- alignment
- codec flag
- checksum

## 3. Node Metadata

Purpose:

- map matrix rows back to stable node ids
- keep labels and mappings available without polluting hot compute sections

Suggested contents:

- row id -> node id
- optional reverse hash/index
- optional string pool offsets

## 4. Raw Graph Structure

Purpose:

- preserve original graph form
- support validation and alternate derived views

Suggested contents:

- raw CSC or CSR adjacency
- optional edge weights
- optional original dimension flags

## 5. Rank Matrix Bundle

This is the heart of the format.

### 5a. `P_pull`

A normalized transition matrix optimized for pull-style iteration.

Suggested representation:

- CSC-like sparse layout
- column pointers
- row indices
- values

Why:

- ranking often works naturally by destination pulling from incoming neighbors

### 5b. `P_push` or alternate transpose-friendly view

Optional second matrix view.

Why:

- some algorithms or hardware paths may prefer push orientation
- avoids recomputing transpose or alternate normalization

### 5c. Degree vectors

- out-degree
- optional in-degree

Why:

- used constantly

### 5d. Dangling mask

- bitmap or compact boolean vector

Why:

- dangling-node handling lives on the hot path for PageRank variants

### 5e. Personalization seed slots

- optional predeclared seed vectors

Why:

- speeds repeated PPR-style workloads

## 6. Hot Vectors

These should be aligned, fixed-width numeric arrays.

Suggested contents:

- `rank_curr`
- `rank_prev`
- `scratch_1`
- `scratch_2`
- optional `teleport_vector`
- optional `residual`

Why:

- they are touched every iteration
- they should be physically close and alignment-friendly

## 7. Warm Start State

Purpose:

- resume from nearly converged state

Suggested contents:

- last converged iteration count
- convergence tolerance used
- residual vector
- delta norm
- dirty block bitmap
- changed frontier or row list
- checkpoint timestamp
- source graph hash

Why:

- this is the main differentiator over generic sparse files

## 8. Cached Outputs

Purpose:

- expose immediately useful results without rerunning

Suggested contents:

- converged PageRank vector
- cached PPR vectors for common seeds
- HITS hub and authority vectors
- optional top-k sorted indexes

## 9. Optional Debug / Provenance

Purpose:

- support reproducibility and trust

Suggested contents:

- build parameters
- normalization strategy
- floating-point mode
- source graph metadata
- debug markers

This section should stay cold.

## 10. Footer / Checksums

Purpose:

- integrity
- safe loading
- corruption detection

Suggested contents:

- section checksums
- footer checksum
- trailing magic

## Hot Path Design

The hot path should look like this:

1. `mmap` file
2. validate header and section directory
3. borrow `P_pull`
4. borrow hot vectors
5. iterate:
   - scan contiguous incoming neighborhoods
   - accumulate into `rank_next`
   - use residuals to skip cold blocks when possible
6. compare convergence norm
7. checkpoint if needed

This avoids:

- rebuilding matrices
- rebuilding vectors
- touching cold metadata

## Three Strong Additions

## 1. Residual-Gated Block Skipping

Persist:

- residual per block
- dirty bitmap

Use:

- skip blocks below tolerance

Why:

- strongest practical rerun speed improvement

## 2. Multi-Resolution Rank Bundle

Persist:

- full graph matrix
- quotient graph matrix
- optional core-only matrix

Use:

- coarse solve first
- refine locally

Why:

- more ambitious, but elegant and potentially very fast

## 3. Locality-Permuted Layout

Persist:

- node permutation
- cache-friendly block boundaries
- cluster-aware ordering

Why:

- same algorithm
- better memory behavior
- strong systems-level speed gain

## Practical Recommendation

If this becomes a real repo, the rollout should be:

### MVP

- matrix bundle
- pull-friendly sparse orientation
- degree and dangling vectors
- cached rank outputs

### V2

- residual persistence
- checkpoint / resume
- warm-start reranking after small graph changes

### V3

- node reordering for locality
- multi-resolution rank views
- more algorithm families

## Bottom Line

If the instruction is **maximize speed**, the right format is not a generic sparse file.

It is:

> **a rank-native persisted runtime artifact with matrix bundle plus warm-start state**

That is the cleanest path to making repeated ranking workloads dramatically faster on local persisted graphs.
