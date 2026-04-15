# v301 Current Recommendation

**Date**: 2026-03-22
**Status**: Working recommendation from 22-agent rubber duck analysis. NOT the PRD.
**Input**: Parallel research session covering CPG, Datalog, GraphBLAS, DuckDB extensions, CozoDB forks, matrix graph algorithms, and "shallow CPG" architectures — all analyzed through rubber duck debugging and Shreyas Doshi strategic simulations.

---

## Part 1: Ground Truth (Rubber Duck Findings)

### Finding 1: The Graph Is Semantically Empty (BUG-1)

100% of edges store symbolic names with sentinel positions `(-1, -1)`. The function `maybe_add_edge()` at `dependency_edge_extractor.rs:417` stores `to_path: target_name.to_string()` without resolving to actual entity PKs. This means:

- `build_from_edge_records()` in `adjacency_list_graph.rs` creates phantom nodes like `"HashMap:-1:-1"`
- Every graph algorithm (SCC, PageRank, k-core, Leiden, betweenness, entropy, CK metrics) returns noise
- The 7-event journey's ANCHOR and CLUSTER phases traverse a meaningless graph

**Fix**: 2-pass ingestion. Pass 1: collect all entities, build `name -> pk` index. Pass 2: resolve symbolic edge targets against the index. Resolution priority: same-file > same-directory > first match > drop unresolved. ~100 lines across 4 files. 3-5 days.

**Files**: `dependency_edge_extractor.rs`, `import_resolution_mapper.rs` (exists but never called), `ingestion_pipeline_orchestrator.rs`, `entity_type_definitions.rs`.

### Finding 2: Git Recency Never Wired (BUG-2)

`shared_server_app_state.rs:71` — `git_recency: HashMap::new()`. The `build_git_recency_scores()` function exists but is never called during server startup. RRF search has 4 signals but the git recency signal is always zero. One-line fix at startup.

### Finding 3: pt09 Doesn't Compile (BUG-4)

`compiler_analysis_driver.rs` is 40% skeleton with 4 TODOs (rustc_sig, visibility, mir_calls, trait_impls). `mir_call_graph_extractor.rs` is 12 lines returning `vec![]`. Requires nightly toolchain. Blocks `cargo build` on stable.

**Fix**: Gate behind cargo feature flag. Defer to v302.

### Finding 4: Snippet Column Stores Full Source

Full source code stored in Turso DB. User wants NO snippets in DB — read from disk at DEEP DIVE time using `file_path + start_line:end_line`.

### Finding 5: Algorithm Bugs Found

| Algorithm | Bug | Severity | Fix |
|-----------|-----|----------|-----|
| PageRank | Missing dangling node redistribution | Medium | Add `dangling_sum / n_f` term (~10 lines) |
| Tarjan SCC | Recursive DFS, stack overflow at >5000 nodes | Medium | Iterative with explicit work stack (~40 lines) |
| k-core | Guard uses `old_d > current_k` instead of `old_d > d` | Low | 1 line change |
| Leiden | Missing refinement phase. Actually Louvain, not Leiden | Low | Rename or implement refinement phase |

### Finding 6: Journey Has Hidden Bugs

- BFS in ANCHOR phase traverses both forward AND reverse edges. Should be reverse-only (callers, not callees) to find the "most depended upon" anchor.
- DEEP DIVE has N+1 query pattern — fetches entities one at a time.
- Ego network has priority inversion: `continue` should be `break` for strict priority truncation when over budget.

### Finding 7: Contains Hierarchy Exists

BUG-3 was partially wrong — `contains` edges ARE wired in Phase 7 of ingestion. But edge resolution (BUG-1) still breaks them since they also use symbolic targets.

---

## Part 2: Strategic Analysis (Shreyas Doshi Simulations)

### Kill List: Permanently Rejected

| Idea | Why Rejected |
|------|-------------|
| **Datalog as query language** | CozoDB was used as a KV store, never leveraged Datalog. Turso migration was correct. Adding Datalog adds learning curve without product value. |
| **GraphBLAS / SuiteSparse** | Matrix multiplication for graph algorithms is overkill. Our graphs are <100K nodes. HashMap adjacency list is fine. GraphBLAS adds C FFI, unsafe blocks, and MATLAB-era complexity. |
| **Custom graph database in Rust** | Writing a graph DB is a multi-year, multi-team project. We build a code analysis tool, not a database. |
| **Shallow CPG (cfg + data_flow edges)** | Tree-sitter cannot extract control flow or data flow. Would require per-language AST interpreters. Cost/benefit ratio is terrible for 12 languages. |
| **CozoDB fork / DuckDB extension** | Fork maintenance burden. DuckDB extension requires C++ FFI. Neither adds product value over Turso + custom graph. |
| **petgraph migration** | Only 40% of algorithms replaceable (BFS, Tarjan, PageRank). Custom algorithms (k-core, Leiden, CK, SQALE, entropy) stay. NodeIndex<->String mapping adds overhead. Migration churn for marginal gain. |
| **Embeddings / vector search** | CPU-only guarantee. No GPU, no LLM-in-the-middle. Trigram + trie + RRF is sufficient. |

### Strategic Insight 1: Fix Data, Not Architecture

Every architecture discussion (petgraph, GraphBLAS, Datalog, CPG) is a distraction from the real problem: **edges don't resolve**. Fix BUG-1 and every existing algorithm, endpoint, and journey works on real data. The architecture is fine. The data pipeline is broken.

### Strategic Insight 2: "Graph Intelligence, Not Token Efficiency"

Context windows will grow to 1M+ tokens. "99% token reduction" is a shrinking moat. The durable positioning is: "We understand your codebase's dependency structure and can answer architectural questions." This survives context window growth because raw code dumps can't answer "where are the natural module seams?"

### Strategic Insight 3: Simulation Is The Differentiator (But Not Yet)

Surface 3 (dependency graph as architect's tool) is what makes Parseltongue different from "grep but for LLMs." But simulation endpoints on corrupted data are worthless. Ship simulation AFTER edges resolve correctly and algorithms are verified.

### Strategic Insight 4: Solo Developer Burnout Is Risk #1

Already happened once (2-month hiatus). The plan must be executable by one person in focused 2-week sprints. Reject any plan requiring >4 weeks of uninterrupted work before a shippable checkpoint.

### Strategic Insight 5: Kuzu Was The Missed Alternative

MIT-licensed embedded graph database with Cypher query support. If we were starting over, Kuzu > CozoDB. But we already migrated to Turso and it works. Not worth another migration. File under "if starting fresh."

### Strategic Insight 6: NetworkX Validation Recommended

~420 lines of Python to validate all 7 graph algorithms against a reference implementation. Build a small reference graph, run algorithms in both Rust and NetworkX, assert matching results. Should be done before any "algorithms are correct" claim.

### Strategic Insight 7: ripgrep as 5th Search Signal

Async subprocess with 25ms timeout. Content-level search that complements symbol trie (exact) and trigram (fuzzy). Adds recall for cases where entity names don't match but code content does. Low effort, high value.

---

## Part 3: The Recommendation

### Phase 0: Make The Graph Real (Week 1-2)

Fix BUG-1. This unblocks everything.

1. Add `resolve_edge_targets()` to `dependency_edge_extractor.rs` (~50 lines)
2. Add `build_name_to_pk_index()` to `import_resolution_mapper.rs` (~30 lines)
3. Wire 2-pass in `ingestion_pipeline_orchestrator.rs` (~20 lines)
4. Wire `git_recency` at server startup (1 line in `shared_server_app_state.rs`)
5. Gate pt09 behind feature flag
6. Fix PageRank dangling nodes (~10 lines)
7. Fix Tarjan iterative DFS (~40 lines)
8. Fix k-core guard (1 line)

**Acceptance**: `parseltongue ingest .` produces edges with real PKs (not `-1:-1`). `cargo test --all` passes.

### Phase 1: Ship The Journey (Week 3)

Make the 7-event journey work end-to-end on real data.

1. Remove `snippet` column from DB schema
2. Add disk-read function for DEEP DIVE (`read_lines(file, start, end)`)
3. Cap DEEP DIVE at 20K tokens
4. Fix ANCHOR BFS to reverse-only traversal
5. Fix ego network priority (break not continue)
6. Fix N+1 in DEEP DIVE (batch query)
7. Integration test: full journey with resolved edges

**Acceptance**: `/query?q=handle` returns clusters with real edges. Response includes source read from disk, not DB.

### Phase 2: Verify Everything (Week 4)

Before adding features, prove what exists is correct.

1. NetworkX validation harness (~420 lines Python)
2. Run all 7 algorithms on reference graph, compare Rust vs NetworkX
3. Fix any algorithm discrepancies found
4. Add ripgrep as 5th RRF signal (optional, low risk)
5. Contentless FTS5 for snippet tokens (indexed but not stored)

**Acceptance**: All 7 algorithms produce identical results to NetworkX on the reference graph.

### Phase 3: Simulation Layer (Week 5-6, IF Phase 0-2 are solid)

Only if the foundation is verified.

1. Shadow graph (clone + mutate + diff)
2. `/simulate-entity-move-blast-radius`
3. `/simulate-module-split-impact`
4. `/coupling-boundary-analysis`
5. Structured response format (blast_radius, breaking_changes, effort, risk, alternatives)

### Phase 4: Tauri App (Week 7-8, parallel-safe)

Can begin in parallel with Phase 2-3 since it's mostly UI wrapping HTTP.

1. Folder picker + workspace creation
2. Ingestion progress via Tauri events
3. Dashboard with FRESH/STALE badges
4. HTTP server sidecar management

### Deferred to v302

- rustc_private / MIR extraction (compiler-verified edges for Rust)
- Trait impl resolution from compiler
- Visibility extraction from compiler
- Additional simulation endpoints (insertion-point-finder, batch-refactoring)

---

## Comparison With Plan File Options

All 22 agents converged on: **Option A first (Fix-Forward), then Option B (Simulation) once the foundation is verified.** Nobody recommended jumping straight to B, C, or D.

| Plan File Option | Agent Consensus |
|-----------------|----------------|
| A: Fix-Forward | **Do this first.** Phases 0-2 above. |
| B: Simulation | Do this second, IF Phase 0-2 succeed. Phases 3-4 above. |
| C: Petgraph | Reject. Only 40% replaceable. Migration churn not worth it. |
| D: Compiler | Defer to v302. pt09 is 40% skeleton. MIR is 0%. |

The key shift from the plan file: **don't skip to simulation before verifying the graph is correct.** The plan file's Phase 0 (bug fixes) and Phase 2 (simulation) were too close together. Insert a verification phase (NetworkX validation) between them.

---

## Key Files

| File | What Happens |
|------|-------------|
| `parseltongue-core/src/edges/dependency_edge_extractor.rs:417` | BUG-1 lives here. Add `resolve_edge_targets()` |
| `parseltongue-core/src/edges/import_resolution_mapper.rs` | Exists but never called. Add `build_name_to_pk_index()` |
| `pt08-http-query-api-server/src/shared_server_app_state.rs:71` | BUG-2 lives here. Wire `git_recency` |
| `parseltongue-core/src/graph/centrality_pagerank_betweenness.rs` | PageRank dangling node fix |
| `parseltongue-core/src/graph/tarjan_scc_detection.rs` | Iterative DFS fix |
| `parseltongue-core/src/graph/kcore_decomposition_layering.rs` | Guard condition fix |
| `parseltongue-core/src/graph/leiden_community_clustering.rs` | Rename to Louvain or add refinement |
| `parseltongue-core/src/entity_type_definitions.rs` | Remove `snippet` field |
| `parseltongue-core/src/storage/schema_definition_tables.rs` | Remove snippet column |

---

## One-Line Summary

Fix the edges, verify the algorithms, then build on a foundation you trust.
