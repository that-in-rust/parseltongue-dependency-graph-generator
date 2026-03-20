# Executable Spec v2: Complete Parseltongue v300

**Date:** 2026-03-20
**Source PRD:** [PRD-v300.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/PRD-v300.md)
**Revision:** v2 — after rubber-duck analysis of v1 against actual codebase state
**Scope:** Fix what's broken, wire what's disconnected, defer what doesn't exist yet.

---

## Rubber Duck Findings (v1 Problems)

The notes01-agent read every `.rs` file in `crates/` and found:

### What's Real (substantial implementations with tests)
- **parseltongue-core**: entity types (30 variants), storage (libSQL DDL + 14-method CRUD), all 11 graph algorithms, 4-signal search (query preprocessor, symbol trie, trigram, RRF), walker (14 languages, tree-sitter, doc folding, wc coverage), hierarchy edge builder
- **pt01**: 7-phase ingestion pipeline, end-to-end working
- **pt08**: 20 HTTP routes including `/query`, journey handler, search handler, all core handlers
- **pt10**: workspace lifecycle (add/list/remove with JSON persistence), server process manager (tokio task spawn/stop)

### 4 Critical Bugs Found

**BUG-1: Edges store symbol names, not resolved PKs**
`dependency_edge_extractor.rs:417-426` stores `to_path: "HashMap"`, `to_start: -1`, `to_end: -1`. The graph algorithms expect node IDs in `"path:start:end"` format. Result: the adjacency list in `shared_server_app_state.rs` creates phantom nodes like `"HashMap:-1:-1"` that match nothing. The call/import graph is structurally valid but semantically empty. BFS anchor traversal and ego clustering have no real edges to walk.

**BUG-2: Git recency signal never wired into server**
`shared_server_app_state.rs:71` hardcodes `git_recency: HashMap::new()`. The function `build_git_recency_scores()` exists and works correctly, but is never called from `build_server_app_state()`. The RRF combiner always receives an empty map for its 4th signal.

**BUG-3: Visibility heuristic makes BFS trivial**
`shared_server_app_state.rs:55-58` sets visibility as `!e.name.starts_with('_')`. Nearly all entities are "public" by this heuristic, so BFS anchor traversal terminates immediately at the start node without traversing anything. The `visibility` column from the database (which pt09 would populate) is ignored.

**BUG-4: pt09 cannot compile**
- `compiler_analysis_driver.rs` uses `extern crate rustc_driver/rustc_interface/rustc_middle/rustc_hir/rustc_span` but `pt09/Cargo.toml` has no corresponding dependencies
- No `rust-toolchain.toml` in the repo, no nightly pin
- 4 explicit `// TODO` comments for `rustc_sig`, `visibility`, `mir_calls`, `trait_impls`
- `mir_call_graph_extractor.rs` is a placeholder that returns `vec![]`
- pt09 is unconditionally included in workspace members, blocking `cargo build` on stable

### Additional Issues
- `git_history_recency_signal.rs` test has hardcoded path `/code/that-in-rust/parseltongue-rust-LLM-companion` — fails on any other machine
- `staleness_detection_checker.rs` compares file count to entity count with 20% tolerance — logically broken (50 files can produce 400 entities)
- pt10 is a plain Rust library, not a Tauri app — no tauri dependency, no GUI, no frontend
- `FileParsable` entity has `wc: 0` in ingestion pipeline (PRD says every entity has meaningful wc)

### v1 Spec Contradictions
- REQ-015 (no placeholders) directly contradicts pt09's 4 TODOs
- REQ-008/009 (enrichment produces real facts) impossible given pt09 can't compile
- REQ-010/011 (Tauri lifecycle) assumes Tauri exists — it doesn't
- REQ-014 (E2E smoke test) conflates 4 distinct scopes into one gate
- Open Question 2 already answered in code (`/query?q=...&cluster=N`)
- Several TDD "RED" tests would pass immediately (REQ-003, REQ-006) or can't compile (REQ-008/009)

---

## v2 Design Principles

1. **Fix what's broken before building what's missing.** The wiring bugs (BUG-1 through BUG-3) are the highest-value work. They make existing code actually work.
2. **Three scopes, three iterations.** Iteration A: wiring fixes + pt09 gating. Iteration B: pt09 real enrichment. Iteration C: Tauri GUI. This spec covers Iteration A only.
3. **Tests that can actually fail.** No tests for code that doesn't exist. No tests that pass vacuously.
4. **Honest about what "done" means.** Iteration A makes the product work end-to-end on stable Rust without compiler enrichment or GUI. That is a shippable milestone.

---

## Iteration A Scope

Fix the wiring. Gate the unfinished. Make the 7-event journey work with real graph data.

### Out of Scope (Iteration B/C)
- REQ-008, REQ-009: Rust compiler enrichment (pt09 needs nightly toolchain + real implementation)
- REQ-010, REQ-011: Tauri workspace lifecycle (pt10 needs Tauri dependency + frontend)
- REQ-014 as written: full E2E with Tauri (replaced with CLI-level E2E in this iteration)

---

## Actors And Boundaries

1. **LLM or CLI caller** — calls HTTP endpoints, uses `/query?q=...` for the 7-event journey
2. **Developer running `parseltongue ingest` + `parseltongue serve`** — ingests codebase, starts server
3. **Boundaries** — v300-A is CPU-only, stable Rust, no Tauri, no compiler enrichment, storage is libSQL

---

## Failure Modes (Iteration A)

1. `/query` route exists but graph traversal returns zero edges (BFS/ego useless) — **this is the current state**
2. git-recency signal always empty, degrading RRF ranking quality — **this is the current state**
3. BFS anchor always returns start node because everything is "public" — **this is the current state**
4. pt09 included in workspace members, breaking `cargo build` on stable — **this is the current state**
5. git recency test hardcoded to author's machine path — **this is the current state**
6. ingestion stores `wc: 0` for FileParsable entities, violating coverage invariant

---

## Performance And Reliability Limits

1. `GET /server-health-check-status` SHALL respond in `< 100 ms` on warm server
2. `GET /query?q=handle` phase-1 SHALL complete in `< 200 ms` on this repo, warm DB
3. `GET /query?q=handle&cluster=0` phase-2 SHALL complete in `< 500 ms` on warm DB (tightened from v1's overly generous 800ms)
4. ingestion of this repo SHALL complete without panic
5. all workspace crates except pt09 SHALL pass `cargo test` on stable Rust
6. pt09 SHALL be excluded from default workspace members

---

# Executable Requirements

### REQ-A-001: Exclude pt09 From Stable Workspace Build

**WHEN** a contributor runs `cargo build` or `cargo test --workspace` on stable Rust
**THEN** the system SHALL NOT attempt to compile `pt09-rustc-compiler-enrichment`
**AND** SHALL pass all tests for `parseltongue-core`, `pt01-codebase-ingestion-engine`, `pt08-http-query-api-server`, `pt10-tauri-workspace-manager`

**Implementation:** Remove pt09 from `[workspace].members` in root Cargo.toml. Add a comment noting it requires nightly + `rustc-dev`. Optionally add a `rust-toolchain.toml` inside `crates/pt09-rustc-compiler-enrichment/` with the pinned nightly channel.

**Current state:** pt09 is in `members = ["crates/*"]` glob. `cargo build` fails on stable.

### REQ-A-002: Wire Git Recency Signal Into Server Startup

**WHEN** the HTTP server starts via `build_server_app_state(db_path)`
**THEN** the system SHALL call `build_git_recency_scores()` with the codebase root path
**AND** SHALL populate `SharedServerAppState.git_recency` with the result
**AND** SHALL return scores in `(0.0, 1.0]` for tracked files
**SHALL** return an empty map without panic when the codebase root is not a git repository

**Implementation:** `shared_server_app_state.rs` needs to:
1. Look up the codebase root path from the database (codebases table has `root_path`)
2. Call `build_git_recency_scores(&root_path)`
3. Assign result to `git_recency` field instead of `HashMap::new()`

**Current state:** `git_recency: HashMap::new()` hardcoded at `shared_server_app_state.rs:71`.

### REQ-A-003: Fix Git Recency Test To Use Dynamic Path

**WHEN** git recency tests run on any machine
**THEN** the test SHALL discover the repo root dynamically (e.g., via `CARGO_MANIFEST_DIR` or `git rev-parse --show-toplevel`)
**AND** SHALL NOT contain hardcoded filesystem paths

**Current state:** `git_history_recency_signal.rs` test has hardcoded `/code/that-in-rust/parseltongue-rust-LLM-companion`.

### REQ-A-004: Resolve Edge Targets To Entity PKs

**WHEN** `extract_edges_from_source` produces call/import/type-ref/implements edges
**THEN** the `to_path`, `to_start`, `to_end` fields SHALL contain resolved entity PKs where a matching entity exists in the same codebase
**AND** SHALL retain the symbolic name with sentinel values (`to_start = -1, to_end = -1`) only when no matching entity can be found
**AND** the resolution SHALL be performed after all files in a codebase have been chunked (so the full entity index is available)

**Implementation:** Two-phase approach:
1. Phase 1 (existing): `extract_edges_from_source` stores symbolic target names with sentinel `-1:-1` — no change needed here
2. Phase 2 (NEW): After all entities are ingested, run a resolution pass that:
   - Builds a `HashMap<String, EntityPrimaryKeyLocation>` mapping entity names to their PKs
   - For each edge where `to_start == -1`, looks up `to_path` (the symbolic name) in the map
   - If found: updates `to_path`, `to_start`, `to_end` to the resolved entity's PK
   - If ambiguous (multiple entities with same name): prefer same-file, then same-directory, then any match
   - If not found: leave as-is (symbolic edge, will create phantom graph nodes)

The resolution pass runs inside the ingestion pipeline, after Phase 4 (WRITE entities) and before Phase 6 (FTS REBUILD). It should be a new `resolve_symbolic_edge_targets()` function in `import_resolution_mapper.rs` (which already exists but is unused).

**Current state:** `dependency_edge_extractor.rs:417-426` stores `to_path: target_name` (e.g., `"HashMap"`, `"add"`) with `to_start: -1, to_end: -1`. Graph is populated with phantom nodes.

### REQ-A-005: Journey Endpoint Is Routable And Returns Structured JSON

**WHEN** `GET /query?q=<term>` is called
**THEN** the system SHALL return a JSON response with `phase`, `query`, `clusters` (array), and `total_candidates`
**AND** SHALL return a typed validation error when `q` is missing or empty

**WHEN** `GET /query?q=<term>&cluster=<n>` is called with a valid cluster index
**THEN** the system SHALL return entity snippets for the selected cluster
**SHALL** return a typed range error when the cluster index is out of bounds

**Current state:** Route exists and works. This requirement validates existing behavior and adds test coverage.

### REQ-A-006: BFS Anchor Uses Database Visibility When Available

**WHEN** the journey handler looks up entity visibility for BFS anchor traversal
**THEN** the system SHALL prefer the `visibility` column from the entity record if non-null and non-empty
**AND** SHALL fall back to the name heuristic (`!name.starts_with('_')`) only when `visibility` is null/empty
**AND** SHALL treat `"pub"` and `"pub(crate)"` as public, all other values as private

**Implementation:** Change `shared_server_app_state.rs:55-58` from:
```rust
(e.pk.to_string(), !e.name.starts_with('_'))
```
to something like:
```rust
(e.pk.to_string(), match e.visibility.as_deref() {
    Some("pub") | Some("pub(crate)") => true,
    Some(_) => false,
    None => !e.name.starts_with('_'),
})
```
Note: `visibility` may not be on `CodeEntityRecord` yet — the entity type definition may need a field added or the storage query updated.

**Current state:** All entities default to public via name heuristic. BFS is trivially deterministic.

### REQ-A-007: Ego Cluster Token Budget Is Enforced

**WHEN** a cluster is built around an anchor with a token budget
**THEN** the system SHALL always include the anchor
**AND** SHALL prefer callers before callees when truncating to budget
**SHALL** keep reported `estimated_tokens` ≤ requested budget (except mandatory anchor)

**Current state:** `ego_network_cluster_builder.rs` implements this correctly. This requirement adds test coverage for the server's journey handler integration (which uses its own budget math at `journey_handler.rs`).

### REQ-A-008: Ingestion Pipeline Completes On This Repo

**WHEN** `parseltongue ingest .` runs on this repository
**THEN** the system SHALL finish without panic
**AND** SHALL persist entities and edges into libSQL storage
**AND** SHALL make the resulting codebase queryable by pt08
**SHALL** record ingestion errors (individual file parse failures) without aborting

**Current state:** Pipeline works. This requirement adds E2E validation.

### REQ-A-009: Coverage Reporting Distinguishes File Categories

**WHEN** coverage statistics are generated for an indexed codebase
**THEN** the system SHALL report: parsed file count, unparsable file count (binary/too-large), ignored file count (.gitignore), and failed file count (parse error)
**AND** SHALL expose the ratio of searchable-entity words to total words per file

**Current state:** Coverage handler counts entities per folder but doesn't distinguish categories. Needs enhancement.

### REQ-A-010: FileParsable Entity wc Is Non-Zero

**WHEN** the ingestion pipeline creates a `FileParsable` entity for a parsed source file
**THEN** the `wc` field SHALL equal the total word count of the file
**AND** the sum of all child entity wc values (code spans + whitespace gaps) SHALL equal this total

**Current state:** `ingestion_pipeline_orchestrator.rs` stores `wc: 0` for file entities.

### REQ-A-011: Staleness Detection Uses Real Data

**WHEN** `check_workspace_staleness()` evaluates a workspace
**THEN** the system SHALL compare current file hashes against stored hashes in the database
**AND** SHALL NOT compare file count against entity count (which has no meaningful ratio)

**Current state:** `staleness_detection_checker.rs` compares file count vs cached entity count with 20% tolerance. This is logically broken.

### REQ-A-012: No Placeholder Debt In Shipping Crates

**WHEN** the iteration-A branch is prepared for merge
**THEN** `parseltongue-core`, `pt01-codebase-ingestion-engine`, `pt08-http-query-api-server`, and `pt10-tauri-workspace-manager` SHALL contain no `TODO`, `STUB`, `PLACEHOLDER`, `todo!()`, or `unimplemented!()` in non-test code
**AND** pt09 is explicitly excluded from this gate (deferred to Iteration B)

**Current state:** Need to grep. pt09 has 4 TODOs which are now explicitly out of scope.

### REQ-A-013: CLI E2E Smoke Test Passes

**WHEN** a contributor follows the CLI happy path on this repository
**THEN** the system SHALL support:
1. `parseltongue ingest .` (creates database)
2. `parseltongue serve --db <path>` (starts HTTP server)
3. `curl /server-health-check-status` (returns 200)
4. `curl /codebase-statistics-overview-summary` (returns non-zero counts)
5. `curl "/query?q=handle"` (returns phase-1 with clusters)
6. `curl "/query?q=handle&cluster=0"` (returns phase-2 with snippets)
**AND** SHALL complete this flow with no manual database surgery

**Current state:** Steps 1-4 likely work. Steps 5-6 return results but graph traversal (BFS/ego) operates on an effectively empty graph due to BUG-1.

---

# Test Matrix

| req_id | test_id | type | assertion | notes |
|--------|---------|------|-----------|-------|
| REQ-A-001 | TEST-BUILD-001 | build | `cargo test --workspace` passes on stable excluding pt09 | Verify Cargo.toml change |
| REQ-A-002 | TEST-UNIT-001 | unit | `build_server_app_state` populates git_recency with non-empty map | Mock or use real repo |
| REQ-A-002 | TEST-UNIT-002 | unit | git_recency returns empty map on non-repo path without panic | Error handling |
| REQ-A-003 | TEST-UNIT-003 | unit | git recency test uses dynamic path, passes on any machine | Fix existing test |
| REQ-A-004 | TEST-UNIT-004 | unit | after resolution pass, intra-file call edges have real PKs | Core correctness |
| REQ-A-004 | TEST-UNIT-005 | unit | unresolvable edges retain symbolic name with sentinel -1:-1 | Graceful fallback |
| REQ-A-004 | TEST-UNIT-006 | unit | ambiguous names prefer same-file match | Resolution priority |
| REQ-A-005 | TEST-INTEG-001 | integration | `/query?q=handle` returns 200 with clusters array | Route validation |
| REQ-A-005 | TEST-INTEG-002 | integration | `/query` without q returns typed error | Validation |
| REQ-A-005 | TEST-INTEG-003 | integration | `/query?q=handle&cluster=99` returns range error | Boundary |
| REQ-A-006 | TEST-UNIT-007 | unit | BFS uses visibility column when present, falls back to name heuristic | Visibility logic |
| REQ-A-006 | TEST-UNIT-008 | unit | entity with `visibility: "pub(crate)"` is treated as public | Specific case |
| REQ-A-007 | TEST-UNIT-009 | unit | ego cluster anchor is always included regardless of budget | Budget logic |
| REQ-A-007 | TEST-UNIT-010 | unit | cluster prefers callers over callees when truncating | Priority |
| REQ-A-008 | TEST-INTEG-004 | integration | ingest this repo → non-zero entity and edge count in DB | Pipeline E2E |
| REQ-A-009 | TEST-INTEG-005 | integration | coverage report distinguishes parsed/unparsable/ignored/failed | Reporting |
| REQ-A-010 | TEST-UNIT-011 | unit | FileParsable entity wc equals file total word count | wc invariant |
| REQ-A-011 | TEST-UNIT-012 | unit | staleness uses file hash comparison, not count ratio | Logic fix |
| REQ-A-012 | TEST-VERIFY-001 | verification | grep shipping crates for placeholder markers returns zero | Release hygiene |
| REQ-A-013 | TEST-E2E-001 | e2e | CLI happy path 6 steps succeed on this repo | Product readiness |

**Total: 20 tests.** All can be written and run. None require nightly. None test code that doesn't exist.

---

# TDD Plan

## 1. STUB — Write Failing Tests

**Round 1 (can write immediately — APIs exist):**
- TEST-UNIT-003: Fix git recency test to use dynamic path (currently hardcoded → will fail if path doesn't exist)
- TEST-INTEG-001/002/003: `/query` route tests (route exists, tests are additive)
- TEST-UNIT-009/010: ego cluster budget tests (algorithm exists, tests are additive)
- TEST-BUILD-001: Verify `cargo test --workspace` after removing pt09 from members
- TEST-VERIFY-001: grep for placeholders

**Round 2 (requires new code to have compilation targets):**
- TEST-UNIT-004/005/006: edge resolution tests (needs `resolve_symbolic_edge_targets()` function signature)
- TEST-UNIT-001/002: server git recency wiring tests (needs `build_server_app_state` to accept repo root)
- TEST-UNIT-007/008: visibility column tests (needs `CodeEntityRecord.visibility` field or DB column read)
- TEST-UNIT-011: FileParsable wc test (needs pipeline change)
- TEST-UNIT-012: staleness hash comparison test (needs staleness rewrite)

**Round 3 (integration, after GREEN on units):**
- TEST-INTEG-004: ingest this repo
- TEST-INTEG-005: coverage report categories
- TEST-E2E-001: full CLI happy path

## 2. RED — Verify Failures

Run `cargo test --workspace` (after pt09 exclusion). Expected failures:
- git recency test (hardcoded path)
- edge resolution tests (function doesn't exist yet)
- server git recency wiring (HashMap::new() hardcoded)
- visibility column tests (field doesn't exist on entity record)
- FileParsable wc (hardcoded to 0)
- staleness (wrong comparison logic)

Tests that will pass immediately (and that's fine — they validate existing working code):
- `/query` route tests
- ego cluster budget tests
- placeholder grep (if pt09 is excluded)

## 3. GREEN — Fix Each Bug

**Fix order (by dependency):**

1. **Remove pt09 from workspace members** (unblocks all other work — `cargo build` must work first)
2. **Fix git recency hardcoded test path** (quick win, TEST-UNIT-003)
3. **Wire git recency into server startup** (REQ-A-002, needs codebase root_path from DB)
4. **Fix FileParsable wc: 0** (REQ-A-010, one-line change in pipeline)
5. **Add `resolve_symbolic_edge_targets()` to `import_resolution_mapper.rs`** (REQ-A-004, the biggest piece of work)
6. **Wire resolution pass into ingestion pipeline after entity write phase** (REQ-A-004 integration)
7. **Add visibility field to entity record / read from DB** (REQ-A-006)
8. **Update visibility map builder in shared_server_app_state** (REQ-A-006)
9. **Rewrite staleness detection to use hash comparison** (REQ-A-011)
10. **Enhance coverage reporting** (REQ-A-009)

## 4. REFACTOR

- Remove dead code in `mir_call_graph_extractor.rs` placeholder (it's in pt09, out of scope, but note it)
- Ensure `import_resolution_mapper.rs` is clean (currently exists but unused — now it will be used)
- No new abstractions unless the resolution pass genuinely needs one

## 5. VERIFY

```bash
# 1. Stable build
cargo test --workspace  # pt09 excluded from members

# 2. Placeholder check
grep -r "TODO\|STUB\|PLACEHOLDER\|todo!()\|unimplemented!()" \
  --include="*.rs" \
  crates/parseltongue-core/ \
  crates/pt01-codebase-ingestion-engine/ \
  crates/pt08-http-query-api-server/ \
  crates/pt10-tauri-workspace-manager/

# 3. E2E smoke
cargo run -- ingest .
cargo run -- serve --db <printed-path> &
sleep 2
curl -s http://localhost:7777/server-health-check-status | jq .
curl -s http://localhost:7777/codebase-statistics-overview-summary | jq .
curl -s "http://localhost:7777/query?q=handle" | jq .clusters
curl -s "http://localhost:7777/query?q=handle&cluster=0" | jq .entities
```

---

# Quality Gates

## Pre-Commit (Iteration A)

- [ ] pt09 excluded from workspace members
- [ ] `cargo test --workspace` passes on stable
- [ ] `cargo clippy --workspace` clean
- [ ] git recency test uses dynamic path
- [ ] edge resolution produces real PKs for intra-codebase calls
- [ ] `/query?q=handle` returns non-empty clusters with real graph traversal
- [ ] no TODO/STUB/PLACEHOLDER in shipping crates (excluding pt09)
- [ ] FileParsable wc matches file total word count

## Release Gate (Iteration A)

- [ ] CLI E2E smoke test (6 steps) passes on this repository
- [ ] BFS anchor actually traverses at least one edge in the smoke test query
- [ ] git recency scores are non-empty in server state during smoke test

---

# Open Questions (Resolved)

| # | Question | v2 Answer |
|---|----------|-----------|
| 1 | pt09 default member or gated? | **Excluded from workspace members.** Separate build command for nightly. Iteration B. |
| 2 | Single /query or separate endpoints? | **Already decided in code.** Single `/query?q=...&cluster=N`. Not an open question. |
| 3 | pt10 updates counts or PT08 is source of truth? | **PT08 is source of truth.** pt10 should query the DB or PT08 for counts, not cache stale numbers. Fix staleness to use hash comparison. |
| 4 | Coverage threshold? | **"Zero unaccounted words" is already achieved** by whitespace gap entities. The real metric is searchable-entity word density, which the coverage report should expose. No PRD threshold change needed — just honest reporting. |

---

# What Iteration B and C Will Cover

## Iteration B: Rust Compiler Enrichment
- Add `rust-toolchain.toml` with pinned nightly
- Add `rustc-dev` component dependency resolution
- Implement the 4 TODOs in `compiler_analysis_driver.rs`
- Implement real MIR call extraction (replace placeholder)
- Wire enrichment into ingestion pipeline as optional post-pass
- REQ-V300-008, REQ-V300-009 from v1 spec

## Iteration C: Tauri Desktop App
- Add `tauri` dependency to pt10
- Create frontend (React/TypeScript)
- Wire workspace lifecycle commands as Tauri commands
- Build `.dmg` distribution
- REQ-V300-010, REQ-V300-011, REQ-V300-014 from v1 spec
