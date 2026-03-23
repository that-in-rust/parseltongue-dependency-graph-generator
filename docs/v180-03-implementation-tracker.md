# v180-03 Implementation Tracker

**Date**: 2026-03-23
**Status**: Active
**Purpose**: Execution truth for v180

---

## 1. Tracker Rules

This document tracks implementation status only.

It does not redefine scope.
If scope changes, update [v180-01-prd.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/v180-01-prd.md).
If design changes, update [v180-02-architecture-simulation.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/v180-02-architecture-simulation.md).

---

## 2. Workstreams

### WS1. Remove CozoDB from active path
Status: `not started`

Targets:
- remove active Cozo-backed assumptions from code paths
- remove stale Cozo-specific docs and run instructions
- confirm v180 target path builds and runs without Cozo runtime dependency

### WS2. Normalize Turso/libsql durable schema
Status: `not started`

Targets:
- add integer row identity where needed
- add durable exhaustive file-span layer
- reshape edges toward integer references
- preserve searchable entity metadata

### WS3. Preserve exact span identity
Status: `not started`

Targets:
- keep `file_path`, `start_line`, `end_line` first-class
- define `span_key` generation and usage
- keep exact HTTP-facing current-snapshot identity clean

### WS4. Implement file-level invalidation
Status: `not started`

Targets:
- detect file hash changes
- invalidate prior file rows
- reparse full file
- rewrite spans/entities/edges transactionally

### WS5. Upgrade RAM graph build
Status: `not started`

Targets:
- load durable rows into RAM cleanly
- move toward integer-keyed adjacency
- preserve handler compatibility
- verify traversal and analysis still operate correctly

### WS6. Preserve HTTP compatibility
Status: `not started`

Targets:
- keep route surface recognizable
- adjust handlers only as needed for storage/runtime cleanup
- keep entity/detail/search/analysis endpoints functioning

### WS7. Documentation cleanup
Status: `not started`

Targets:
- align README and agent docs with live architecture
- remove stale `rocksdb:` / Cozo-only instructions where invalid
- point future work to the canonical v180 docs

---

## 3. Initial File Map

### Likely core files

#### Storage
- [schema_definition_tables.rs](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/parseltongue-core/src/storage/schema_definition_tables.rs)
- [turso_storage_client.rs](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/parseltongue-core/src/storage/turso_storage_client.rs)

#### Core types
- [entity_type_definitions.rs](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/parseltongue-core/src/entity_type_definitions.rs)

#### Graph runtime
- [adjacency_list_graph.rs](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/parseltongue-core/src/graph/adjacency_list_graph.rs)
- [shared_server_app_state.rs](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/pt08-http-query-api-server/src/shared_server_app_state.rs)

#### Ingestion
- [ingestion_pipeline_orchestrator.rs](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/pt01-codebase-ingestion-engine/src/ingestion_pipeline_orchestrator.rs)

#### HTTP layer
- [route_definition_builder.rs](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/pt08-http-query-api-server/src/route_definition_builder.rs)
- `crates/pt08-http-query-api-server/src/handlers/*`

#### Stale docs likely needing cleanup
- [README.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/README.md)
- [AGENTS.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/AGENTS.md)
- [CLAUDE.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/CLAUDE.md)

---

## 4. Immediate Next Slice

### Slice A. Canonical docs established
Status: `done`

Deliverables:
- [v180-01-prd.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/v180-01-prd.md)
- [v180-02-architecture-simulation.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/v180-02-architecture-simulation.md)
- [v180-03-implementation-tracker.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/v180-03-implementation-tracker.md)

### Slice B. Confirm current-vs-target schema diff
Status: `not started`

Deliverables:
- current Turso schema table/column inventory
- target v180 schema table/column inventory
- migration delta list

### Slice C. Confirm current-vs-target identity model
Status: `not started`

Deliverables:
- current composite span PK model
- target integer row ID + span key model
- handler impact notes

### Slice D. Confirm delete/replace plan
Status: `not started`

Deliverables:
- list of Cozo-era assumptions to remove
- list of string-key graph assumptions to replace
- list of docs to update after code changes land

---

## 5. Risks To Watch

1. Schema migration may leak into handler behavior more than expected.
2. Integer-edge migration may require wider graph algorithm touch points than planned.
3. Exact span truth can bloat storage if implemented carelessly.
4. Old docs can easily drift back into being treated as truth.
5. File invalidation needs atomicity to avoid half-updated graph state.

---

## 6. Done Log

### 2026-03-23
- agreed canonical three-doc structure
- created `v180-01`, `v180-02`, `v180-03`
- kept dated notes intact as open reference material
