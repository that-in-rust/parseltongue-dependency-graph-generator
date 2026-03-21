# Executable Spec v3: Complete Parseltongue v300 — Iteration A

**Date:** 2026-03-20
**Source PRD:** [PRD-v300.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/PRD-v300.md)
**Revision:** v3 — synthesized from 4 parallel research agents studying all docs/ + crate implementations
**Scope:** Iteration A only. Fix wiring bugs, resolve edges, gate pt09, make the 7-event journey work with real graph data on stable Rust.

---

## What Changed From v2

v2 identified 4 critical bugs and wrote 13 requirements with 20 tests. v3 adds 8 new requirements and 15 new tests based on findings from:

| Source | Key Finding |
|--------|-------------|
| Agent 1 (PRD/plans) | `contains` edges must always resolve; BFS must walk reverse (caller) edges; edge resolution needs codebase_id scoping; splitNodes/is_test/doc_comment_folding have no tests; wc algorithm undefined |
| Agent 2 (Glommer) | FTS rebuild is always drop+create; query preprocessing needs stopwords; libSQL needs short-lived connections + busy_timeout=5000 + retry with jitter; verifySearchable guard at startup; codemogger has zero edges (our edge work is novel) |
| Agent 3 (History) | v1.7.2 had 51.67% unresolved references — entity resolution IS the bottleneck; slug-based port management; serve old data during re-ingestion; four-word naming convention is binding; all diagnostics via tracing not eprintln |
| Agent 4 (Crate code) | 2 missing HTTP endpoints (callers/callees); Phase 2 DEEP DIVE has no 20k token cap; graph-DB thesis says no homegrown algorithms (but we keep them for now); 14 grammars in Cargo.toml but only 12 registered |

---

## Rubber Duck Findings (carried from v2)

### What's Real
- **parseltongue-core**: 30 entity types, libSQL storage (14-method CRUD), 11 graph algorithms, 4-signal search, 14-language walker, doc folding, wc coverage, hierarchy edge builder
- **pt01**: 7-phase ingestion pipeline, end-to-end working
- **pt08**: 20 HTTP routes including `/query`, journey handler, search + graph handlers
- **pt10**: workspace lifecycle (add/list/remove + JSON persistence), server process manager

### 4 Critical Bugs (unchanged)
1. **BUG-1: Edges store symbol names, not resolved PKs** — `dependency_edge_extractor.rs:417-426`
2. **BUG-2: Git recency never wired** — `shared_server_app_state.rs:71` hardcodes `HashMap::new()`
3. **BUG-3: Visibility heuristic makes BFS trivial** — `shared_server_app_state.rs:55-58`
4. **BUG-4: pt09 cannot compile** — missing Cargo.toml deps, no nightly pin, 4 TODOs

### v3 Additional Findings
5. **BUG-5: `contains` edges silently unresolved** — hierarchy_edge_builder creates folder→file→entity edges but these should always resolve (both endpoints are our own entities). No distinction between "graceful fallback" and "contains bug."
6. **BUG-6: BFS walks wrong direction in journey handler** — `journey_handler.rs` has inline `find_public_anchor` that traverses both forward and reverse edges. PRD Event 3 says "walk callers until a public fn/trait is found" — direction must be reverse (caller) edges only.
7. **BUG-7: FTS query has no preprocessing** — codemogger strips stopwords, agent filler words, <3 char tokens, >30 char tokens, deduplicates, caps at 12 terms. Our FTS queries pass raw user input.
8. **BUG-8: No verifySearchable guard** — if DB file exists but FTS index is corrupt or WAL-locked, the server returns empty results silently instead of an actionable error.
9. **BUG-9: 2 missing HTTP endpoints** — `/reverse-callers-query-graph` and `/forward-callees-query-graph` are in CLAUDE.md's 22-endpoint table but absent from route_definition_builder.rs.
10. **BUG-10: Phase 2 DEEP DIVE has no token budget** — all entity snippets returned without the 20k token cap from PRD Event 7.
11. **BUG-11: libSQL connections are long-lived** — memelord research proves concurrent access (Tauri + HTTP server + watcher) needs short-lived connections with `PRAGMA busy_timeout = 5000` and retry with jitter. Current code opens one connection at server startup and holds it.

---

## Design Principles (v3)

1. **Entity resolution is the bottleneck, not more algorithms.** v1.7.2 had 51.67% unresolved references. Fix resolution first.
2. **Three iterations.** A: wiring + resolution + correctness. B: compiler enrichment. C: Tauri GUI.
3. **Tests that can actually fail.** No tests for nonexistent code. No vacuous passes.
4. **Codemogger had zero edges.** Our edge resolution work is novel — no reference implementation to steal. Extra care needed.
5. **Homegrown algorithms stay for now.** Graph-DB thesis says "use petgraph" — but porting to petgraph is Iteration B work. Current algorithms have unit tests and work correctly on correct data.

---

## Iteration A Scope

Fix wiring. Resolve edges. Gate pt09. Add missing correctness tests. Make the 7-event journey work with real graph data.

### Out of Scope (Iteration B/C)
- pt09 compiler enrichment (nightly + real implementation)
- Tauri GUI (pt10 needs tauri dependency + frontend)
- petgraph migration (works fine with homegrown if data is correct)
- Slug-based multi-project port management (Iteration C, requires Tauri)
- MCP server (explicitly deferred per V210 backlog)

---

## Actors And Boundaries

1. **LLM or CLI caller** — calls HTTP endpoints, uses `/query?q=...` for the 7-event journey
2. **Developer** — runs `parseltongue ingest .` + `parseltongue serve --db <path>`
3. **Boundaries** — CPU-only, stable Rust, no Tauri, no compiler enrichment, libSQL storage, short-lived DB connections

---

## Failure Modes

| # | Failure | Current State |
|---|---------|--------------|
| F1 | `/query` graph traversal returns zero real edges | **Active** — BUG-1 |
| F2 | git-recency signal always empty | **Active** — BUG-2 |
| F3 | BFS anchor always returns start node | **Active** — BUG-3 |
| F4 | `cargo build` fails on stable | **Active** — BUG-4 |
| F5 | FTS receives raw unprocessed queries | **Active** — BUG-7 |
| F6 | Server silently returns empty on corrupt DB | **Active** — BUG-8 |
| F7 | Phase 2 returns unbounded token payload | **Active** — BUG-10 |
| F8 | DB file locking under concurrent access | **Latent** — BUG-11 |

---

## Performance And Reliability Limits

| Target | Endpoint / Phase | Budget |
|--------|-----------------|--------|
| P1 | `GET /server-health-check-status` | `< 100 ms` |
| P2 | SEARCH phase (RRF fusion) | `< 10 ms` |
| P3 | ANCHOR phase (BFS) | `< 50 ms` |
| P4 | CLUSTER phase (ego network) | `< 100 ms` |
| P5 | `GET /query?q=handle` (P2+P3+P4 combined) | `< 200 ms` |
| P6 | `GET /query?q=handle&cluster=0` (phase-2) | `< 500 ms` |
| P7 | Ingestion of this repo | No panic |
| P8 | All crates except pt09 | Pass `cargo test` on stable |

---

# Executable Requirements

## Category 1: Build and Infrastructure

### REQ-A-001: Exclude pt09 From Stable Workspace Build

**WHEN** a contributor runs `cargo build` or `cargo test --workspace` on stable Rust
**THEN** the system SHALL NOT attempt to compile `pt09-rustc-compiler-enrichment`
**AND** SHALL pass all tests for the remaining workspace crates

**Implementation:** Replace `members = ["crates/*"]` glob with explicit member list excluding pt09. Add `rust-toolchain.toml` inside `crates/pt09-rustc-compiler-enrichment/` pinning the required nightly.

### REQ-A-002: LibSQL Connection Management

**WHEN** the HTTP server or ingestion pipeline opens a database connection
**THEN** the system SHALL set `PRAGMA busy_timeout = 5000` on every new connection
**AND** SHALL retry on `SQLITE_BUSY` / `SQLITE_LOCKED` with exponential backoff + jitter (base 50ms, max 10 retries)
**AND** SHALL prefer short-lived connections per logical operation over long-lived session connections

**Source:** memelord `withDb<T>` pattern (docs/research-glommer/memelord/). Validated against concurrent access (Tauri + HTTP server + watcher touching same DB).

### REQ-A-003: Searchability Guard At Server Startup

**WHEN** `build_server_app_state(db_path)` opens the database
**THEN** the system SHALL verify the database is queryable (FTS index exists, at least one codebase has entities)
**AND** SHALL return an actionable error message if the DB file exists but has zero indexed entities (possible WAL lock or missing ingestion)
**SHALL NOT** silently serve empty results from an uninitialized database

**Source:** codemogger `verifySearchable` pattern (docs/research-glommer/codemogger/).

---

## Category 2: Edge Resolution (THE Priority — v1.7.2 was 51.67% unresolved)

### REQ-A-004: Resolve Symbolic Edge Targets To Entity PKs

**WHEN** the ingestion pipeline completes entity extraction for all files in a codebase
**THEN** the system SHALL run a resolution pass that maps symbolic edge targets to entity PKs
**AND** the resolution SHALL be scoped by `codebase_id` (entity names are not globally unique)
**AND** SHALL use this priority for ambiguous names: (1) same-file, (2) same-directory, (3) any match in codebase
**AND** SHALL retain symbolic names with sentinel values (`to_start = -1, to_end = -1`) only when no match exists

**Implementation:** New `resolve_symbolic_edge_targets(store, codebase_id)` in `import_resolution_mapper.rs`. Runs after Phase 4 (WRITE) and before Phase 6 (FTS REBUILD) in the ingestion pipeline.

**Key constraint:** The name→PK lookup map must be built from the `entities` table filtered by `codebase_id`. Cross-codebase edge leakage is a correctness bug.

### REQ-A-005: Contains Edges Must Always Resolve

**WHEN** the hierarchy edge builder creates folder→file or file→entity `contains` edges
**THEN** both the `from` and `to` endpoints SHALL be entities that exist in the database
**AND** a `contains` edge with `to_start = -1, to_end = -1` SHALL be treated as a bug, not a graceful fallback
**SHALL** fail the ingestion pipeline test if any `contains` edge is unresolved

**Rationale:** Unlike call/import edges (which reference external symbols), `contains` edges reference entities WE created. If the hierarchy builder produces an unresolved contains edge, the ingestion pipeline has an ordering or naming bug.

### REQ-A-006: Edge Type Scope For Iteration A

**WHEN** edges are extracted and resolved
**THEN** the following edge types SHALL be extracted: `calls`, `imports`, `implements`, `type_refs`, `field_access`, `contains`
**AND** `async_await`, `iterators`, `generics` edge types are deferred to Iteration B (require deeper AST analysis)

**Source:** v1.6.1 had 8 edge types. PRD M6 specifies 6 for tree-sitter level. The 3 deferred types need compiler-level analysis or complex pattern matching.

---

## Category 3: Search and Journey

### REQ-A-007: Wire Git Recency Signal Into Server Startup

**WHEN** the HTTP server starts via `build_server_app_state(db_path)`
**THEN** the system SHALL look up the codebase root path from the `codebases` table
**AND** SHALL call `build_git_recency_scores(&root_path)`
**AND** SHALL populate `SharedServerAppState.git_recency` with the result (not `HashMap::new()`)

### REQ-A-008: Fix Git Recency Test To Use Dynamic Path

**WHEN** git recency tests run on any machine
**THEN** the test SHALL discover the repo root dynamically via `CARGO_MANIFEST_DIR` traversed to git root
**AND** SHALL NOT contain hardcoded filesystem paths

### REQ-A-009: FTS Query Preprocessing

**WHEN** a user query enters the search pipeline (via `/query?q=...` or `/code-entities-search-fuzzy?q=...`)
**THEN** the system SHALL preprocess the query before FTS:
1. Lowercase
2. Tokenize preserving compound identifiers (`snake_case`, `dot.paths`)
3. Remove English stopwords AND agent filler words (`help`, `please`, `can`, `could`, `would`, `should`, `file`, `files`, `code`, `use`, `using`, `make`, `way`, `thing`, `something`)
4. Remove tokens shorter than 3 chars or longer than 30 chars
5. Deduplicate
6. Cap at 12 tokens
**AND** SHALL pass the preprocessed query to FTS, not the raw user input

**Source:** codemogger `preprocessQuery` (docs/research-glommer/codemogger/src/search/query.ts). Our `query_preprocessor_engine.rs` may already do some of this — verify and fill gaps.

### REQ-A-010: RRF Weights Are Correct

**WHEN** the rank fusion combiner scores a candidate entity
**THEN** the system SHALL use these weights: `fts=0.35, trie=0.30, trigram=0.20, git_recency=0.15, k=60`
**AND** the formula SHALL be: `score = w/(k+rank+1)` for each signal
**AND** a test SHALL verify the formula with all 4 signals populated

### REQ-A-011: BFS Anchor Walks Reverse (Caller) Edges Only

**WHEN** the journey handler finds a public anchor for a search hit
**THEN** BFS SHALL traverse reverse edges (callers) only — "who calls this?"
**AND** SHALL NOT traverse forward edges (callees) — "what does this call?"
**AND** SHALL return the original entity when no public ancestor is reachable via callers
**AND** traversal SHALL be deterministic for the same graph state

**Source:** PRD Event 3 (line 168): "walk callers until a public fn/trait is found." The current `journey_handler.rs` inline `find_public_anchor` traverses both directions — this is wrong.

### REQ-A-012: BFS Visibility Uses Database Column

**WHEN** the journey handler builds the entity visibility map
**THEN** the system SHALL read the `visibility` column from the entity record
**AND** SHALL treat `"pub"` and `"pub(crate)"` as public
**AND** SHALL treat `"pub(super)"`, `"pub(in ...)"`, `"pub(self)"`, `""`, and any other value as private
**AND** SHALL fall back to name heuristic (`!name.starts_with('_')`) ONLY when `visibility` is null

### REQ-A-013: Ego Cluster Includes Implements Edges

**WHEN** the ego network cluster builder gathers 1-hop neighbors for an anchor
**THEN** the cluster SHALL include entities connected via `implements` edges (trait implementations)
**AND** SHALL include callers, callees, and implementations in that priority order when truncating to budget
**AND** the anchor SHALL always be included regardless of budget

**Source:** PRD Event 4 (line 174): "Cluster = anchor + callers + callees + implementations."

### REQ-A-014: Phase 2 DEEP DIVE Enforces Token Budget

**WHEN** `GET /query?q=<term>&cluster=<n>` returns entity snippets
**THEN** the total `estimated_tokens` (sum of `wc * 1.3` for all returned entities) SHALL NOT exceed 20,000 tokens
**AND** SHALL truncate by dropping the lowest-priority entities if the cluster exceeds budget
**SHALL** always include the anchor entity even if its tokens alone approach the budget

**Source:** PRD Event 7: "up to 20k tokens."

### REQ-A-015: Journey Endpoint Returns Structured JSON

**WHEN** `GET /query?q=<term>` is called
**THEN** the system SHALL return JSON with `phase`, `query`, `clusters` array, and `total_candidates`

**WHEN** `GET /query?q=<term>&cluster=<n>` is called with valid index
**THEN** the system SHALL return entity snippets for the selected cluster

**SHALL** return typed errors for: missing `q`, empty `q`, out-of-bounds cluster index

**WHEN** `GET /query?q=<term>` is called on an empty database (no entities)
**THEN** the system SHALL return `phase: "no_results"` with an empty clusters array
**SHALL NOT** panic or return 500

---

## Category 4: Ingestion Correctness

### REQ-A-016: wc Computation Algorithm Is Defined

**WHEN** the ingestion pipeline computes `wc` for an entity
**THEN** `wc` SHALL equal `source_text.split_whitespace().count()` for the entity's line range
**AND** for a `FileParsable` entity, `wc` SHALL equal the total word count of the entire file content
**AND** `sum(child_entity.wc) == file_entity.wc` for every parsable file (zero-gap invariant)

**Current state:** `FileParsable` entity has `wc: 0`. Must be fixed to file total.

### REQ-A-017: Doc Comment Folding Discriminates /// From //

**WHEN** the doc comment folding logic processes comments
**THEN** `///` and `//!` text SHALL be folded into the adjacent code entity's `doc_comment` field (for FTS)
**AND** `//` plain comments SHALL NOT be folded into `doc_comment` — they become standalone `comment` entities
**AND** a test SHALL verify that `///` merges into the next function's doc_comment while `//` does not

**Source:** PRD lines 1582-1595: "Tree-sitter does NOT distinguish doc comments from plain comments at the node type level."

### REQ-A-018: splitNodes Splits Containers Over 150 Lines

**WHEN** the tree-sitter entity extractor encounters an `impl_item`, `trait_item`, `mod_item`, `class_definition`, or `class_declaration` node exceeding 150 lines
**THEN** the system SHALL extract child methods/functions as separate entities
**AND** SHALL descend exactly one level into body wrappers (`declaration_list`, `class_body`, `block`)
**AND** a test SHALL verify that a 200-line impl block produces method-level entities

### REQ-A-019: is_test Detection Per Language

**WHEN** the entity extractor processes a code entity
**THEN** the system SHALL set `is_test = true` for:
- Rust: entity has `#[test]` or `#[cfg(test)]` attribute
- Python: function name starts with `test_`
- Go: function name starts with `Test`
- JavaScript/TypeScript: entity is inside `describe()` or `it()` block, or file matches `*.test.*` / `*.spec.*`
**AND** a test SHALL verify Rust `#[test]` detection specifically (attribute attachment to next entity)

### REQ-A-020: ALWAYS_IGNORE Directories Are Skipped

**WHEN** the directory scanner walks a codebase
**THEN** the system SHALL skip these directories unconditionally: `.git`, `node_modules`, `target`, `build`, `dist`, `.next`, `__pycache__`, `.tox`, `.venv`, `venv`, `.mypy_cache`, `.cargo`, `.rustup`
**AND** a test SHALL verify that ingesting a directory containing `target/` produces zero entities from `target/`

### REQ-A-021: FTS Rebuild Is Always Drop+Create

**WHEN** the ingestion pipeline rebuilds the FTS index (Phase 6)
**THEN** the system SHALL drop the existing FTS table, recreate it, populate from entities, and optimize
**AND** SHALL rebuild even on incremental runs (not just full reindex)
**AND** FTS weights SHALL be: `name=5.0, signature=3.0, doc_comment=2.0`

**Source:** codemogger always does full drop+create+populate+optimize. No incremental FTS update.

---

## Category 5: Server Correctness

### REQ-A-022: Missing Callers/Callees Endpoints Are Added

**WHEN** the HTTP server starts
**THEN** the system SHALL register these 2 currently-missing endpoints:
- `GET /reverse-callers-query-graph?entity=<pk>` — returns entities that call the given entity
- `GET /forward-callees-query-graph?entity=<pk>` — returns entities called by the given entity
**AND** these endpoints SHALL return data from the resolved edge graph (not phantom symbolic nodes)

**Source:** CLAUDE.md lists 22 endpoints. Route builder has 20. These 2 are missing.

### REQ-A-023: Coverage Reporting Distinguishes File Categories

**WHEN** coverage statistics are generated
**THEN** the system SHALL report: parsed file count, unparsable file count, ignored file count, failed file count
**AND** SHALL report ratio of searchable-entity words to total words per file
**AND** SHALL map to PRD entity_type categories: `file_parsable`, `file_unparsable`, `file_config`

### REQ-A-024: Staleness Detection Uses Hash Comparison

**WHEN** `check_workspace_staleness()` evaluates a workspace
**THEN** the system SHALL compare current file SHA-256 hashes against stored hashes in `indexed_files` table
**AND** SHALL NOT compare file count against entity count (logically broken)
**AND** SHALL report how many files have changed since last indexing

### REQ-A-025: No Placeholder Debt In Shipping Crates

**WHEN** the iteration-A branch is prepared for merge
**THEN** `parseltongue-core`, `pt01`, `pt08`, `pt10` SHALL contain no `TODO`, `STUB`, `PLACEHOLDER`, `todo!()`, or `unimplemented!()` in non-test code
**AND** pt09 is explicitly excluded (deferred to Iteration B)
**AND** no `eprintln!` anywhere — all diagnostics via `tracing`

---

# Test Matrix

## Build (2 tests)

| req_id | test_id | assertion |
|--------|---------|-----------|
| REQ-A-001 | TEST-BUILD-001 | `cargo test --workspace` passes on stable excluding pt09 |
| REQ-A-025 | TEST-VERIFY-001 | grep shipping crates for placeholder/eprintln markers returns zero |

## Edge Resolution (5 tests)

| req_id | test_id | assertion |
|--------|---------|-----------|
| REQ-A-004 | TEST-UNIT-001 | after resolution pass, intra-file call edges have real PKs (scoped by codebase_id) |
| REQ-A-004 | TEST-UNIT-002 | unresolvable edges retain symbolic name with sentinel -1:-1 |
| REQ-A-004 | TEST-UNIT-003 | ambiguous names prefer same-file, then same-directory |
| REQ-A-005 | TEST-UNIT-004 | all `contains` edges resolve — zero sentinels for hierarchy edges |
| REQ-A-004 | TEST-UNIT-005 | resolution does NOT leak edges across codebase_id boundaries |

## Search and Journey (14 tests)

| req_id | test_id | assertion |
|--------|---------|-----------|
| REQ-A-007 | TEST-UNIT-006 | `build_server_app_state` populates git_recency with non-empty map |
| REQ-A-007 | TEST-UNIT-007 | git_recency returns empty map on non-repo path without panic |
| REQ-A-008 | TEST-UNIT-008 | git recency test uses dynamic path, passes on any machine |
| REQ-A-009 | TEST-UNIT-009 | query preprocessor strips stopwords + agent filler, caps at 12 tokens |
| REQ-A-010 | TEST-UNIT-010 | RRF formula uses correct 4-signal weights (0.35, 0.30, 0.20, 0.15) |
| REQ-A-011 | TEST-UNIT-011 | BFS walks reverse (caller) edges only, not forward |
| REQ-A-011 | TEST-UNIT-012 | BFS returns original entity when no public ancestor via callers |
| REQ-A-012 | TEST-UNIT-013 | visibility uses DB column; `pub(crate)` = public, `pub(super)` = private |
| REQ-A-013 | TEST-UNIT-014 | ego cluster includes `implements` edges in neighbor set |
| REQ-A-013 | TEST-UNIT-015 | cluster truncation priority: callers > callees > implements |
| REQ-A-014 | TEST-UNIT-016 | Phase 2 response total tokens ≤ 20,000 |
| REQ-A-015 | TEST-INTEG-001 | `/query?q=handle` returns 200 with clusters array |
| REQ-A-015 | TEST-INTEG-002 | `/query` without q returns typed error |
| REQ-A-015 | TEST-INTEG-003 | `/query?q=anything` on empty DB returns `phase: "no_results"`, no panic |

## Ingestion Correctness (7 tests)

| req_id | test_id | assertion |
|--------|---------|-----------|
| REQ-A-016 | TEST-UNIT-017 | FileParsable wc equals `content.split_whitespace().count()` |
| REQ-A-016 | TEST-UNIT-018 | sum(child wc) == file wc for every parsable fixture file |
| REQ-A-017 | TEST-UNIT-019 | `///` merges into next fn's doc_comment; `//` does not |
| REQ-A-018 | TEST-UNIT-020 | 200-line impl block produces method-level child entities |
| REQ-A-019 | TEST-UNIT-021 | Rust `#[test]` fn has `is_test = true` |
| REQ-A-020 | TEST-UNIT-022 | ingesting dir with `target/` subfolder produces zero entities from `target/` |
| REQ-A-021 | TEST-UNIT-023 | FTS rebuild uses drop+create (not incremental), weights are 5.0/3.0/2.0 |

## Server and Infrastructure (5 tests)

| req_id | test_id | assertion |
|--------|---------|-----------|
| REQ-A-002 | TEST-UNIT-024 | DB connection sets `PRAGMA busy_timeout = 5000` |
| REQ-A-003 | TEST-UNIT-025 | server startup on empty DB returns actionable error, not silent empty results |
| REQ-A-022 | TEST-INTEG-004 | `/reverse-callers-query-graph?entity=<pk>` returns callers from resolved graph |
| REQ-A-022 | TEST-INTEG-005 | `/forward-callees-query-graph?entity=<pk>` returns callees from resolved graph |
| REQ-A-024 | TEST-UNIT-026 | staleness detection compares file hashes, not count ratio |

## Integration and E2E (3 tests)

| req_id | test_id | assertion |
|--------|---------|-----------|
| REQ-A-023 | TEST-INTEG-006 | coverage report distinguishes parsed/unparsable/ignored/failed |
| REQ-A-008+ | TEST-INTEG-007 | ingest this repo → non-zero entity count AND non-zero resolved edge count |
| REQ-A-ALL | TEST-E2E-001 | CLI happy path: ingest → serve → health → stats → /query phase-1 → /query phase-2 |

**Total: 35 tests.** All runnable on stable Rust. None test nonexistent code. None pass vacuously.

---

# TDD Plan

## Round 1: STUB (can write immediately — APIs exist)

Tests against existing working code (will pass, but establish baseline):
- TEST-BUILD-001, TEST-VERIFY-001
- TEST-INTEG-001, TEST-INTEG-002
- TEST-UNIT-008 (fix existing hardcoded test)
- TEST-UNIT-017, TEST-UNIT-018 (wc invariant — currently fails due to wc:0)
- TEST-UNIT-019, TEST-UNIT-020, TEST-UNIT-021, TEST-UNIT-022 (ingestion correctness)
- TEST-UNIT-023 (FTS rebuild)

## Round 2: STUB (requires new function signatures)

- TEST-UNIT-001 through 005 (edge resolution — needs `resolve_symbolic_edge_targets()`)
- TEST-UNIT-006, 007 (server git recency wiring — needs `build_server_app_state` to accept root path)
- TEST-UNIT-009 (query preprocessing — verify existing preprocessor, add missing stopwords)
- TEST-UNIT-010 (RRF weights)
- TEST-UNIT-011 through 015 (BFS direction, visibility, ego cluster)
- TEST-UNIT-016 (Phase 2 token budget)
- TEST-UNIT-024 (busy_timeout pragma)
- TEST-UNIT-025 (searchability guard)
- TEST-UNIT-026 (staleness hash comparison)

## Round 3: Integration and E2E (after GREEN on units)

- TEST-INTEG-003 through 007
- TEST-E2E-001

## GREEN — Fix Order (by dependency)

| Step | Fix | Blocks |
|------|-----|--------|
| 1 | Remove pt09 from workspace members | Everything (cargo build must work) |
| 2 | Fix git recency hardcoded test path | TEST-UNIT-008 |
| 3 | Fix FileParsable wc: 0 | TEST-UNIT-017/018 |
| 4 | Add query preprocessing stopwords/caps | TEST-UNIT-009 |
| 5 | Wire git recency into server startup | TEST-UNIT-006/007 |
| 6 | Add `resolve_symbolic_edge_targets()` | TEST-UNIT-001-005 (biggest work item) |
| 7 | Wire resolution into ingestion pipeline | TEST-INTEG-007 |
| 8 | Fix BFS to walk reverse edges only | TEST-UNIT-011/012 |
| 9 | Add visibility column read to entity record | TEST-UNIT-013 |
| 10 | Update visibility map in server state | REQ-A-012 |
| 11 | Add `implements` to ego cluster neighbor gathering | TEST-UNIT-014/015 |
| 12 | Add Phase 2 token budget enforcement | TEST-UNIT-016 |
| 13 | Add callers/callees endpoints | TEST-INTEG-004/005 |
| 14 | Add searchability guard | TEST-UNIT-025 |
| 15 | Add busy_timeout pragma | TEST-UNIT-024 |
| 16 | Rewrite staleness detection | TEST-UNIT-026 |
| 17 | Enhance coverage reporting | TEST-INTEG-006 |

## REFACTOR

- Ensure `import_resolution_mapper.rs` is the single location for resolution logic
- Remove `find_public_anchor` inline BFS from journey_handler.rs — delegate to `bfs_anchor_public_traversal.rs`
- All diagnostics via `tracing` — grep and remove any `eprintln!`
- Four-word naming convention on all new functions

## VERIFY

```bash
# 1. Stable build
cargo test --workspace

# 2. Clippy
cargo clippy --workspace -- -D warnings

# 3. Placeholder check
grep -rn "TODO\|STUB\|PLACEHOLDER\|todo!()\|unimplemented!()\|eprintln!" \
  --include="*.rs" \
  crates/parseltongue-core/ crates/pt01-*/ crates/pt08-*/ crates/pt10-*/

# 4. E2E smoke
cargo run -- ingest .
cargo run -- serve --db <path> &
sleep 2
curl -sf http://localhost:7777/server-health-check-status | jq .status
curl -sf http://localhost:7777/codebase-statistics-overview-summary | jq .entity_count
curl -sf "http://localhost:7777/query?q=handle" | jq '.clusters | length'
curl -sf "http://localhost:7777/query?q=handle&cluster=0" | jq '.entities | length'
curl -sf "http://localhost:7777/reverse-callers-query-graph?entity=src/main.rs:1:10" | jq .
```

---

# Quality Gates

## Pre-Commit (Iteration A)

- [ ] pt09 excluded from workspace members
- [ ] `cargo test --workspace` passes on stable (35 tests)
- [ ] `cargo clippy --workspace` clean
- [ ] edge resolution produces real PKs for intra-codebase calls (>50% resolution rate on this repo)
- [ ] all `contains` edges resolve (0% sentinel rate for hierarchy)
- [ ] BFS anchor traverses at least one real edge in smoke test
- [ ] git recency scores are non-empty in server state
- [ ] FTS queries are preprocessed (stopwords stripped)
- [ ] Phase 2 enforces 20k token budget
- [ ] no TODO/STUB/PLACEHOLDER/eprintln in shipping crates
- [ ] callers and callees endpoints return data from resolved graph

## Release Gate (Iteration A)

- [ ] TEST-E2E-001 CLI happy path passes on this repository
- [ ] `/query?q=handle` returns clusters with real (non-phantom) entity PKs
- [ ] ingestion + query cycle completes without manual database surgery
- [ ] 22 HTTP endpoints registered (20 existing + 2 new callers/callees)

---

# Open Questions (All Resolved)

| # | Question | v3 Answer |
|---|----------|-----------|
| 1 | pt09 default member or gated? | **Excluded from workspace members.** Iteration B. |
| 2 | Single /query or separate endpoints? | **Already decided in code.** `/query?q=...&cluster=N`. |
| 3 | pt10 updates counts or PT08 is truth? | **DB is source of truth.** Staleness uses hash comparison. |
| 4 | Coverage threshold? | **Zero unaccounted words** (achieved). Report searchable-entity density. |
| 5 | Homegrown algorithms or petgraph? | **Keep homegrown for Iteration A.** They work correctly on correct data. Petgraph migration is Iteration B. |
| 6 | Token formula: `wc * 1.3` or `chars * 0.75`? | **`wc * 1.3`** per PRD D10. Codemogger uses `chars * 0.75` — roughly equivalent. Use wc-based for consistency with our schema. |
| 7 | ISG_L1_V3 or physical PK for edges? | **Physical PK** (`path:start:end`). The minimal_v200 ISG_L1_V3 schema is superseded by PRD D8. |

---

# Iteration B and C Preview

## Iteration B: Compiler Enrichment + Algorithm Validation
- `rust-toolchain.toml` with pinned nightly
- Implement 4 TODOs in `compiler_analysis_driver.rs`
- Real MIR call extraction
- Wire enrichment into pipeline as optional post-pass
- Migrate to petgraph for BFS/SCC/PageRank (validate against NetworkX)
- `async_await`, `iterators`, `generics` edge types

## Iteration C: Tauri Desktop App
- Add `tauri` v2 dependency (capability-based, not v1 allowlist)
- Svelte or React frontend (research says Svelte, current stubs are React — decide)
- Wire workspace lifecycle commands as `#[tauri::command]`
- Slug-based multi-project port management
- FRESH/STALE badge with hash comparison
- macOS notification on ingest completion
- `.dmg` distribution
