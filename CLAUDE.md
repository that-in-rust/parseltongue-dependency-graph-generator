# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Parseltongue v3.0 is a code analysis toolkit that parses codebases into a graph database for LLM-optimized querying. Core value: 99% token reduction (2-5K tokens vs 500K raw dumps), 31x faster than grep.

**Version**: 3.0.0 (libsql storage, workspace architecture, 20 HTTP endpoints)
**Languages Supported**: Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, Ruby, PHP, C#, Swift, Kotlin, Scala
**Storage**: libsql (no CozoDB, no rusqlite)

## Workspace Structure

```
parseltongue/                            # Root binary – CLI dispatcher
crates/
├── parseltongue-core/                   # Shared types, traits, storage, tree-sitter parsing
├── pt01-codebase-ingestion-engine/      # Crate 1: Ingest codebase -> libsql
├── pt08-http-query-api-server/          # Crate 8: HTTP REST API server (Axum)
├── pt09-rustc-compiler-enrichment/      # Crate 9: rustc_private enrichment (nightly only)
└── pt10-tauri-workspace-manager/        # Crate 10: Tauri desktop workspace manager
```

**Dependency Flow**: `parseltongue` (binary) -> `pt01`/`pt08` (tools) -> `parseltongue-core` (shared)

## Build and Test Commands

```bash
# Build (stable toolchain – excludes pt09 which requires nightly)
cargo build --workspace --exclude pt09-rustc-compiler-enrichment

# Check pt09 (requires nightly for rustc_private)
cargo +nightly check -p pt09-rustc-compiler-enrichment

# Run all tests (stable toolchain)
cargo test --workspace --exclude pt09-rustc-compiler-enrichment

# Run tests for a specific crate
cargo test -p parseltongue-core
cargo test -p pt01-codebase-ingestion-engine
cargo test -p pt08-http-query-api-server
```

## CLI Usage

```bash
# Ingest a codebase (creates .parseltongue/index.db by default)
parseltongue ingest <path>
parseltongue ingest <path> --db custom/path.db

# Start HTTP query server (default port 8080)
parseltongue serve --db <path-to-db> --port 8080

# Rustc enrichment (nightly only)
parseltongue enrich <path> --db <path-to-db>
```

## HTTP Server Endpoints (20 Total)

| # | Endpoint | Description |
|---|----------|-------------|
| 1 | `/server-health-check-status` | Health check |
| 2 | `/codebase-statistics-overview-summary` | Stats summary |
| 3 | `/api-reference-documentation-help` | API docs |
| 4 | `/code-entities-list-all` | All entities |
| 5 | `/code-entities-search-fuzzy?q=pattern` | Fuzzy search |
| 6 | `/code-entity-detail-view/{pk}` | Entity detail by primary key |
| 7 | `/dependency-edges-list-all` | All dependency edges |
| 8 | `/strongly-connected-components-analysis` | Tarjan SCC detection |
| 9 | `/leiden-community-detection-clusters` | Leiden community clustering |
| 10 | `/centrality-measures-entity-ranking?method=pagerank` | PageRank/Betweenness centrality |
| 11 | `/kcore-decomposition-layering-analysis?k=N` | K-core graph layering |
| 12 | `/entropy-complexity-measurement-scores?entity=X` | Shannon entropy |
| 13 | `/coupling-cohesion-metrics-suite?entity=X` | CK metrics (CBO/LCOM/RFC/WMC) |
| 14 | `/technical-debt-sqale-scoring?entity=X` | SQALE tech debt (ISO 25010) |
| 15 | `/ingestion-coverage-folder-report?depth=N` | Ingestion coverage |
| 16 | `/smart-context-token-budget?focus=X&tokens=N` | LLM context budget |
| 17 | `/complexity-hotspots-ranking-view?top=N` | Coupling hotspots |
| 18 | `/circular-dependency-detection-scan` | Cycle detection |
| 19 | `/semantic-cluster-grouping-list` | Module clusters |
| 20 | `/query?q=...` | Natural-language journey query |

## Key Technical Decisions

- **Storage**: libsql only — no CozoDB, no rusqlite
- **Primary key format**: `path:start:end` (file path, start line, end line)
- **Search**: 4-signal Reciprocal Rank Fusion (RRF)
- **CPU-only**: no embeddings, no GPU dependencies
- **Nightly isolation**: `pt09-rustc-compiler-enrichment` requires `rustc_private` and is always excluded from stable builds

## Naming Conventions

**FOUR-WORD NAMING**: All function/crate/command names must be exactly 4 words.

```rust
// Functions: underscore-separated
filter_implementation_entities_only()    // Good
render_box_with_title_unicode()          // Good
filter_entities()                        // Bad – too short

// Crates: hyphen-separated
pt01-codebase-ingestion-engine           // Good
pt08-http-query-api-server               // Good
```

**Pattern**: `verb_constraint_target_qualifier()`

## Error Handling

- **Libraries** (`parseltongue-core`): Use `thiserror` for structured errors
- **Applications** (CLI/tools): Use `anyhow` for context

## TDD Workflow

Follow STUB -> RED -> GREEN -> REFACTOR cycle:
1. Write failing test first
2. Run test, verify failure
3. Minimal implementation to pass
4. Refactor without breaking tests

## Test Fixtures

- `test-fixtures-preV200/` — pre-v2.0 test fixture files (per-language, per-pattern)
- `tests-preV200/e2e_workspace/` — end-to-end workspace integration tests

## Version Increment Rules

- Each version = ONE complete feature, end-to-end working
- Zero TODOs/stubs in commits
- All tests passing before commit
