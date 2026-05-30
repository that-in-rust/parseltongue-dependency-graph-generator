# PRD: v180 Phase 1 - Primary Store Shift

**Date**: 2026-03-23
**Status**: Draft
**Scope**: Primary key model, CozoDB removal, Turso/libsql durable storage, exhaustive file-span truth, RAM-native runtime graph

---

## 1. One-Sentence Summary

Phase 1 moves Parseltongue off CozoDB and onto a RAM-native graph runtime backed by a local Turso/libsql durable store, with exhaustive file-span truth, line-span-based public keys, and integer database primary keys.

---

## 2. Why This Phase Exists

The current system already computes graph algorithms in RAM after loading entities and edges from storage.

Today the flow is effectively:

```text
storage -> load entities/edges -> build RAM graph -> run algorithms -> return JSON
```

This means the graph runtime is already separate from the database. Phase 1 formalizes that split and removes CozoDB from the architecture.

This phase also resolves the identity direction for the live graph:

- exact line spans are first-class
- file changes invalidate the whole file graph
- reparsing the full changed file is acceptable
- current-snapshot truth matters more than pretending entity keys are stable across edits

---

## 3. Product Goal

Keep the HTTP API surface stable while replacing the internal storage/runtime model with a cleaner and more rigorous architecture.

### Primary outcomes

1. Remove CozoDB from ingestion, storage, and serving.
2. Keep Turso/libsql as the only durable local store.
3. Make the live graph RAM-native.
4. Make line spans first-class in entity identity.
5. Persist exhaustive file-local span coverage for the current snapshot.
6. Define a clean primary key model that separates:
   - database row identity
   - exact span identity
   - optional semantic continuity metadata

---

## 4. Non-Goals

Phase 1 does **not** include:

1. A rich multi-bucket coverage taxonomy beyond exact span truth.
2. New graph algorithms.
3. Semantic context edges such as `shared_context` or `public_module_context`.
4. Quality improvements to extractor coverage beyond what is needed for storage migration.
5. Architecture simulation overlays.
6. A new graph database.
7. Full folder-as-entity modeling.
8. Public HTTP endpoint path redesign.

Those belong to later phases.

---

## 5. Core Decisions

### D1. CozoDB is removed

CozoDB is removed from:

- ingestion path
- durable storage
- HTTP serving path
- graph query implementation

No new code in Phase 1 may depend on CozoDB types, Datalog queries, or Cozo-backed schemas.

### D2. Turso/libsql is the durable store

A local Turso/libsql-compatible database is the only persistent backing store.

Its jobs are:

- persist codebase metadata
- persist file hashes
- persist entity rows
- persist edge rows
- persist FTS/index support
- support reload after process restart

It is **not** the graph algorithm engine.

### D3. RAM is the runtime graph

All query-time graph computation runs on an in-memory graph representation built from the durable store.

The RAM graph is the source of truth for:

- callers
- callees
- blast radius
- SCC
- centrality
- clustering
- token-budget traversal

### D4. File change invalidates the whole file

If a file changes, Parseltongue treats all entities and edges originating from that file as stale.

The update model is:

1. file hash changes
2. delete prior entities/edges for that file
3. reparse the full file
4. insert the new entities/edges for that file

We do not attempt fine-grained intra-file key preservation.

### D5. Line spans are first-class and exhaustive per file snapshot

Line spans are canonical for the live graph view.

Every entity must have:

- `file_path`
- `start_line`
- `end_line`

Public API consumers may use those spans to zoom, inspect, and diff the current snapshot.

For each successfully indexed file, Phase 1 SHALL also persist a complete file-span partition for the current snapshot.

That means:

- every source line in the indexed file belongs to a persisted span segment
- span segments do not overlap within a file
- the partition is rebuilt whenever the file is reparsed

Phase 1 cares about exact span truth.
It does **not** require a large or final taxonomy vocabulary yet.

### D6. One identity is not enough

Phase 1 uses three identity lanes:

1. **Database primary key**
   - `entity_id INTEGER PRIMARY KEY`
   - internal only

2. **Span key**
   - human/API key for the current snapshot
   - exact location identity
   - line-span-based

3. **Semantic locator**
   - non-primary metadata for continuity and later matching
   - may include language, kind, scope, name, discriminator

The database primary key exists for storage efficiency.
The span key exists for current-truth addressing.
The semantic locator exists so we can add continuity logic later without redesigning storage.

---

## 6. Primary Key Model

### 6.1 Entity row identity

Each entity row SHALL have an integer database primary key.

```sql
entity_id INTEGER PRIMARY KEY AUTOINCREMENT
```

This is the durable storage identity and the edge foreign-key target.

### 6.2 Public span key

Each entity SHALL expose a public key derived from exact location.

**Recommended format:**

```text
{language}|||{kind}|||{display_name}|||{file_path}|||{start_line}|||{end_line}
```

Example:

```text
rust|||fn|||handle_auth|||src/auth.rs|||15|||42
```

This key is:

- readable
- exact
- snapshot-local
- safe for file reading and zooming

This key is **allowed to change** if the file changes.

### 6.3 Semantic locator

Each entity SHALL also store semantic continuity metadata.

Minimum fields:

- `language`
- `entity_kind`
- `scope_text`
- `display_name`
- `discriminator_text`

This lane is not the public primary key in Phase 1.
It exists for:

- future diffing
- future continuity matching
- future rename/move reasoning

### 6.4 Edge identity

Edges SHALL reference entity rows by integer IDs.

Minimum fields:

- `from_entity_id`
- `to_entity_id`
- `edge_type`
- `codebase_id`

Optional denormalized fields for debugging/export may be added later, but RAM graph build should not depend on string parsing.

---

## 7. Durable Schema Requirements

### REQ-PH1-001.0: Replace Cozo with libsql-only storage

**WHEN** Parseltongue ingests or serves a codebase
**THEN** the system SHALL use Turso/libsql as the only durable store
**AND** SHALL not require any CozoDB process, schema, or query runtime
**SHALL** compile without CozoDB-linked storage paths in the Phase 1 target path

### REQ-PH1-002.0: Integer row primary keys

**WHEN** entity rows are persisted
**THEN** the system SHALL assign each persisted entity an integer `entity_id`
**AND** SHALL use that integer as the durable row primary key
**SHALL** store edges using integer references instead of span-string parsing

### REQ-PH1-003.0: Span key visibility

**WHEN** an entity is returned from HTTP APIs
**THEN** the response SHALL include an exact span-addressable key
**AND** the key SHALL include `file_path`, `start_line`, and `end_line`
**SHALL** remain sufficient for a caller to re-open the exact source region in the current snapshot

### REQ-PH1-010.0: Exhaustive file span coverage

**WHEN** a file is successfully indexed
**THEN** the system SHALL persist a complete non-overlapping line-span partition for that file
**AND** every line in the current indexed snapshot SHALL belong to exactly one persisted span segment
**SHALL** rebuild the full partition whenever the file is invalidated and reparsed

### REQ-PH1-004.0: Full-file invalidation

**WHEN** a file hash changes
**THEN** the system SHALL invalidate all prior entities and edges sourced from that file
**AND** SHALL reparse the full file before re-inserting entities and edges
**SHALL NOT** attempt partial entity preservation within the changed file

### REQ-PH1-005.0: Atomic file replacement

**WHEN** the system replaces the graph for one file
**THEN** delete-plus-insert for that file SHALL happen transactionally
**AND** readers SHALL never observe a half-deleted or half-inserted file graph
**SHALL** either see the old file graph or the new file graph

### REQ-PH1-006.0: RAM-native query graph

**WHEN** the HTTP server starts
**THEN** it SHALL load persisted entities and edges from Turso/libsql
**AND** SHALL build the in-memory adjacency graph from those rows
**SHALL** execute graph algorithms against the RAM graph rather than against SQL joins

### REQ-PH1-007.0: Semantic locator persistence

**WHEN** an entity row is written
**THEN** the system SHALL persist semantic locator metadata separate from span identity
**AND** SHALL not require that semantic locator be unique in Phase 1
**SHALL** preserve enough fields to support future continuity matching

### REQ-PH1-008.0: HTTP path stability

**WHEN** Phase 1 ships
**THEN** existing top-level HTTP endpoint paths SHALL remain unchanged
**AND** existing endpoint categories SHALL remain recognizable to current users and agents
**SHALL** limit response-shape changes to key-field semantics and storage-backed details only

### REQ-PH1-009.0: No Cozo-specific strings in public docs

**WHEN** Phase 1 documentation and examples are updated
**THEN** setup and run instructions SHALL reference the new Turso/libsql-backed flow
**AND** SHALL not instruct users to create or query CozoDB-backed workspaces
**SHALL** remove stale references to `rocksdb:`/Cozo-only guidance where no longer valid

---

## 8. Proposed Phase 1 Data Model

### 8.1 Codebases

```sql
codebases(
  codebase_id INTEGER PRIMARY KEY,
  root_path TEXT UNIQUE,
  name TEXT,
  created_at INTEGER,
  last_indexed_at INTEGER
)
```

### 8.2 Indexed files

```sql
indexed_files(
  file_id INTEGER PRIMARY KEY,
  codebase_id INTEGER,
  file_path TEXT,
  file_hash TEXT,
  indexed_at INTEGER,
  UNIQUE(codebase_id, file_path)
)
```

### 8.3 File spans

```sql
file_spans(
  span_id INTEGER PRIMARY KEY,
  codebase_id INTEGER,
  file_id INTEGER,
  file_path TEXT,
  start_line INTEGER,
  end_line INTEGER,
  span_kind TEXT,
  linked_entity_id INTEGER,
  indexed_at INTEGER,
  UNIQUE(codebase_id, file_path, start_line, end_line)
)
```

`file_spans` is the exhaustive truth layer for the current snapshot.
It exists even when a span is not a first-class code entity.

### 8.4 Entities

```sql
entities(
  entity_id INTEGER PRIMARY KEY,
  codebase_id INTEGER,
  file_id INTEGER,
  language TEXT,
  entity_kind TEXT,
  display_name TEXT,
  scope_text TEXT,
  discriminator_text TEXT,
  file_path TEXT,
  start_line INTEGER,
  end_line INTEGER,
  span_key TEXT,
  signature TEXT,
  snippet TEXT,
  doc_comment TEXT,
  wc INTEGER,
  visibility TEXT,
  is_test INTEGER,
  indexed_at INTEGER,
  UNIQUE(codebase_id, span_key)
)
```

### 8.5 Edges

```sql
edges(
  edge_id INTEGER PRIMARY KEY,
  codebase_id INTEGER,
  from_entity_id INTEGER,
  to_entity_id INTEGER,
  edge_type TEXT
)
```

### 8.6 Search support

FTS/index support remains in libsql/SQLite-compatible form, but Phase 1 does not redesign the retrieval strategy beyond adapting it to the new entity schema.

---

## 9. Runtime Architecture

```text
filesystem
  -> tree-sitter parse
  -> file-local span partition build
  -> file-local entity extraction
  -> file-local edge extraction
  -> libsql transaction write
  -> server loads rows
  -> RAM adjacency graph build
  -> HTTP queries and algorithms
```

### What lives in RAM

- entity map by `entity_id`
- span-key lookup map
- forward adjacency
- reverse adjacency
- visibility map
- word-count map
- search indexes needed by handlers

### What lives durably in libsql

- codebase registry
- file hashes
- file spans
- entities
- edges
- FTS tables
- reloadable metadata

---

## 10. Endpoint Compatibility Rules

Phase 1 must preserve user trust by keeping the recognizable API intact.

### Compatibility target

- same endpoint paths
- same major endpoint categories
- same high-level meanings

### Allowed internal changes

- `key` format can change
- detail payloads can reflect new storage fields
- implementation can stop using Cozo-specific assumptions entirely

### Not allowed in Phase 1

- deleting major endpoint categories
- renaming the public HTTP surface as part of the storage migration
- mixing Phase 2 semantic-edge work into the Phase 1 contract

---

## 11. Test Matrix

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-PH1-001.0 | TEST-PH1-001 | integration | ingest + serve works with libsql only and no Cozo runtime path | storage migration |
| REQ-PH1-002.0 | TEST-PH1-002 | unit | entity rows receive integer primary keys and edges store integer refs | schema layer |
| REQ-PH1-003.0 | TEST-PH1-003 | integration | returned entity keys include file path and exact line span | HTTP detail/search |
| REQ-PH1-004.0 | TEST-PH1-004 | integration | changed file causes full-file replacement, not partial retention | reindex path |
| REQ-PH1-005.0 | TEST-PH1-005 | integration | readers never observe half-replaced file graph | transaction boundary |
| REQ-PH1-006.0 | TEST-PH1-006 | integration | server builds RAM graph from libsql rows and algorithms run on RAM graph | app state |
| REQ-PH1-007.0 | TEST-PH1-007 | unit | semantic locator fields persist independently of span key | entity persistence |
| REQ-PH1-008.0 | TEST-PH1-008 | API regression | endpoint paths remain callable with prior workflow names | HTTP contract |
| REQ-PH1-009.0 | TEST-PH1-009 | docs check | docs and examples no longer require CozoDB paths | docs |

---

## 12. TDD Plan

### STUB

1. Add failing schema tests for integer entity/edge primary keys.
2. Add failing tests for span-key serialization.
3. Add failing tests for `replace_file_graph_transactionally()` behavior.
4. Add failing app-state test proving the server can boot from libsql-only rows.
5. Add API regression tests for unchanged endpoint paths.

### RED

1. Run targeted storage tests.
2. Confirm current implementation still assumes old storage layout.
3. Capture failures for:
   - missing integer IDs
   - string-derived graph IDs
   - Cozo-only startup assumptions

### GREEN

1. Introduce the new libsql schema.
2. Write entity/edge adapters using integer IDs.
3. Implement span-key generation.
4. Implement transactional file graph replacement.
5. Update app-state boot to build graph from new rows.
6. Remove Cozo-backed hot path from the Phase 1 target.

### REFACTOR

1. Remove legacy storage glue no longer needed.
2. collapse duplicate key-format helpers.
3. simplify graph build path around integer IDs.
4. keep compatibility wrappers only where needed for HTTP behavior.

### VERIFY

1. Run storage tests.
2. Run HTTP server tests.
3. Re-ingest self-repo and validate key/address lookups.
4. Verify no CozoDB-linked user workflow remains in Phase 1 docs.

---

## 13. Quality Gates

Before Phase 1 is complete, verify:

1. `cargo test` passes for storage and server crates.
2. No new `TODO`, `STUB`, or `FIXME` markers are introduced.
3. Every `REQ-PH1-*` has at least one mapped test.
4. No Phase 1 code path depends on CozoDB.
5. Every persisted edge references integer entity IDs.
6. Every returned entity key is line-span-addressable.
7. File replacement is transactional.
8. Public HTTP endpoint paths remain stable.

---

## 14. Open Questions

1. Should `span_key` include `display_name`, or should it be purely `language/kind/file/start/end`?
2. Should `entity_id` be regenerated on full re-ingest, or do we want optional semantic matching to preserve IDs between runs later?
3. Do we want the Phase 1 server to rebuild the RAM graph at startup only, or also support hot in-process file replacement before Phase 2?
4. Which existing HTTP responses expose `key` most heavily and therefore need the most compatibility care?
5. Should legacy key formats be translated by a compatibility parser during the transition, or do we require a clean reindex and new keys only?

---

## 15. Phase 1 Exit Criteria

Phase 1 is complete when:

1. Parseltongue can ingest and serve without CozoDB.
2. Turso/libsql is the only durable backing store.
3. The server builds its query graph in RAM from persisted rows.
4. Exact line-span keys are returned publicly.
5. File replacement is full-file and transactional.
6. Existing HTTP endpoint paths still work.
7. The codebase is ready for Phase 2 graph-truth improvements without another storage rewrite.
