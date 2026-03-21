# v300 Architecture Truth Table

**Date**: 2026-03-21
**Purpose**: Honest audit of what exists, what works, what is fake.

---

## Persistence Layer

| Component | Status | File | Notes |
|-----------|--------|------|-------|
| libSQL/Turso local DB | REAL | `parseltongue-core/src/storage/turso_storage_client.rs` | Working CRUD |
| `codebases` table | REAL | `schema_definition_tables.rs` | id, root_path, name, timestamps |
| `indexed_files` table | REAL | `schema_definition_tables.rs` | file_path + hash for incremental |
| `entities` table | REAL | `schema_definition_tables.rs` | PK = (file_path, start_line, end_line, codebase_id) |
| `edges` table | REAL | `schema_definition_tables.rs` | No composite PK, no uniqueness constraint |
| Per-codebase FTS5 | REAL | `schema_definition_tables.rs:89-95` | Contentless, porter tokenizer |
| `snippet` in FTS | **NO** | — | Stored in entities table, NOT in FTS index |
| Graph database | **NO** | — | Never was. Not CozoDB, not petgraph, not Neo4j |

## FTS5 Indexed Fields

| Field | In FTS? | Weight logic |
|-------|---------|-------------|
| `name` | YES | — |
| `signature` | YES | — |
| `doc_comment` | YES | — |
| `file_path` | YES | — |
| `entity_type` | YES | — |
| `snippet` (body text) | **NO** | Stored but invisible to search |

FTS5 is contentless (`content=''`) with porter tokenizer. No field weights configured.

## In-Memory Graph

| Component | Status | File |
|-----------|--------|------|
| `AdjacencyListGraphRepresentation` | REAL | `graph/adjacency_list_graph.rs` |
| Structure | `HashMap<String, Vec<(String, String)>>` forward + reverse + `HashSet<String>` nodes |
| Built from | `edges` table rows at server startup |
| Node ID format | `"file_path:start_line:end_line"` |
| petgraph | **NOT USED** | Only in `docs/pre202602/` research references |

## Graph Algorithms (12 files)

| Algorithm | File | Status |
|-----------|------|--------|
| Tarjan SCC | `tarjan_scc_detection.rs` | REAL code, operates on custom graph |
| K-core decomposition | `kcore_decomposition_layering.rs` | REAL code |
| PageRank + Betweenness | `centrality_pagerank_betweenness.rs` | REAL code |
| Leiden community | `leiden_community_clustering.rs` | REAL code |
| Shannon entropy | `entropy_shannon_complexity.rs` | REAL code |
| CK metrics (CBO/LCOM/RFC/WMC) | `ck_metrics_coupling_cohesion.rs` | REAL code |
| SQALE tech debt | `sqale_technical_debt_scoring.rs` | REAL code |
| BFS anchor traversal | `bfs_anchor_public_traversal.rs` | REAL code |
| Ego network builder | `ego_network_cluster_builder.rs` | REAL code |
| Test fixtures | `test_fixture_reference_graphs.rs` | Test support only |

**Critical problem**: All algorithms run on a graph built from **unresolved edges**.
100% of call edges store symbolic target names (e.g., `"HashMap"`) with sentinel positions `(-1, -1)`.
Graph nodes like `"HashMap:-1:-1"` are phantom nodes that don't map to any real entity.

## Search Stack

| Signal | Status | Index | Searches over |
|--------|--------|-------|---------------|
| FTS5 | REAL | Per-codebase virtual table | name, sig, doc_comment, file_path, entity_type |
| LIKE fallback | REAL | None (full scan) | name, sig, doc_comment |
| Symbol trie | REAL | In-memory Patricia trie | entity names only |
| Trigram fuzzy | REAL | In-memory trigram index | entity names only |
| Git recency | **CODE EXISTS, NOT WIRED** | — | — |
| RRF combiner | REAL | `rank_fusion_combiner_rrf.rs` | Merges 4 signals |

### Git recency detail
- `build_git_recency_scores()` works — runs `git log`, computes exponential decay
- `SharedServerAppState` sets `git_recency: HashMap::new()` — **always empty at runtime**
- Test hardcodes path `/code/that-in-rust/parseltongue-rust-LLM-companion` — fails everywhere else
- Net effect: RRF is a 3-signal system, not 4

## Edge Extraction

| Component | Status | Notes |
|-----------|--------|-------|
| Tree-sitter call/import extraction | REAL | `dependency_edge_extractor.rs` |
| `from` side | RESOLVED | `(file_path, start_line, end_line)` from entity PK |
| `to` side | **UNRESOLVED** | `(target_name, -1, -1)` — symbolic name, no file/line |
| `contains` hierarchy edges | **NOT IMPLEMENTED** | No folder->file or file->entity containment |
| Cross-file resolution | **NOT IMPLEMENTED** | Would need name->PK lookup pass |
| Stdlib exclusion | **NOT IMPLEMENTED** | `new`, `iter`, `collect`, `unwrap` etc. create phantom nodes |

This is the single biggest architectural gap. The graph is structurally present but semantically empty.

## Compiler Enrichment (pt09)

| Component | Status | Notes |
|-----------|--------|-------|
| `rustc_driver` / `rustc_interface` deps | **NOT IN Cargo.toml** | Only parseltongue-core, anyhow, serde, tokio |
| `compiler_analysis_driver.rs` | COMPILES but INCOMPLETE | Uses `rustc_driver` in code, 4 TODOs for sig/visibility/mir/traits |
| `entity_enrichment_mapper.rs` | EXISTS | Defines `EnrichmentResult` struct |
| `mir_call_graph_extractor.rs` | **PLACEHOLDER** | Returns `vec![]` |
| Actual rustc_private integration | **NOT FUNCTIONAL** | Would need nightly toolchain + real dependencies |

## Tauri App (pt10)

Not audited here. Separate concern.

## Entity Model

| Layer | Status | Example PK |
|-------|--------|-----------|
| L0: Folder | **NOT IMPLEMENTED** | `src/auth/` |
| L1: File | REAL (via tree-sitter walker) | `src/auth/service.rs:0:0` |
| L2: Code span | REAL (via tree-sitter) | `src/auth/service.rs:8:25` |
| L3: Compiler enrichment | **COLUMNS EXIST, DATA EMPTY** | Same row, nullable columns |

Nullable L3 columns in schema: `rustc_scope`, `rustc_sig`, `visibility`, `mir_calls`, `trait_impls`.
All are NULL in practice.

## What actually works end-to-end today

1. Walk directory, skip .gitignore'd files
2. Tree-sitter parse 14 languages into entities with wc
3. Extract symbolic (unresolved) call/import edges
4. Store entities + edges in libSQL
5. Build FTS5 index on metadata fields
6. Build in-memory trie + trigram on entity names
7. Start HTTP server with 22 endpoint stubs
8. FTS search on metadata finds entities
9. Graph algorithms run (on polluted data)

## What does NOT work

1. Edge targets are unresolved — graph analysis operates on phantom nodes
2. Git recency is not wired into app state
3. Snippet body text is not searchable
4. No hierarchy (contains) edges exist
5. No stdlib exclusion — ~40% of phantom edges are stdlib noise
6. Compiler enrichment is placeholder code
7. Visibility is heuristic (`!name.starts_with('_')`) not from compiler
8. No cross-file or cross-crate edge resolution
9. FTS5 has no field weights (PRD says name=5.0, sig=3.0, doc=2.0)
10. `edges` table has no uniqueness constraint — duplicates possible

---

## The one thing that matters most

**Edge resolution**. Without it, the graph is noise. Every algorithm, every BFS anchor, every ego cluster, every hotspot ranking — all meaningless until `to_path: "HashMap", to_start: -1, to_end: -1` becomes `to_path: "src/collections/map.rs", to_start: 45, to_end: 120`.

Fix edges first. Everything else follows.
