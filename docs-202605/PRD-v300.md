# Parseltongue v3.0

> Grep returns files. Parseltongue returns understanding.
> A minimalistic, well-verified proof of Rust craft.

---

# THE ANSWER

Parseltongue v3.0 is Codemogger's proven foundation — ported to Rust — plus graph intelligence,
compiler enrichment, and a native Mac GUI. Codemogger (by Turso team, ~2,100 lines TypeScript)
solved code indexing: file walking, tree-sitter chunking, Turso/libSQL storage, FTS5 search.
That covers ~30% of our scope. The remaining ~70% is what makes Parseltongue different:
graph analysis algorithms, BFS anchoring, ego network clustering, rustc_private enrichment,
a 7-event query journey, 22 HTTP endpoints, 100% word-count coverage, and a Tauri desktop app.

Build order: Codemogger foundation first (M1-M3), then Parseltongue-specific intelligence (M4-M8).
All within v3.0. 8 milestones. ~16,300 lines of Rust. One product.

## SCQA

**Situation:** LLMs and developers need to understand large codebases without reading every file.
Current tools (grep, ripgrep, ctags) return file matches, not understanding.

**Complication:** Raw code dumps cost 500K+ tokens. Embeddings add 97% indexing overhead.
Existing code search tools (Codemogger, Sourcegraph) offer flat search results without
graph-based navigation.

**Question:** How do we give LLMs structured understanding of a codebase in <20K tokens?

**Answer:** Parse everything with tree-sitter, build a dependency graph, use BFS anchoring
and ego-network clustering to navigate from a search hit to the right neighborhood.
CPU-only. No embeddings. The graph IS the intelligence.

---

# PART I: THE PRODUCT

Everything follows from the screens. The screens ARE the product.

---

## Screen 1: First Launch (Empty State)

User downloads .dmg or `brew install parseltongue`. Opens the app.
No login. No account. No server. Privacy-first.

```
+------------------------------------------+
|  Parseltongue                        [-] |
|------------------------------------------|
|                                          |
|  No codebases yet.                       |
|                                          |
|  [ Browse Folder ]  or drag & drop here  |
|                                          |
|  - - - - - - - - - - - - - - - - - - -   |
|  Parseltongue analyzes Rust codebases    |
|  so you (and your LLM) can understand    |
|  them without reading every file.        |
|                                          |
+------------------------------------------+
```

User clicks Browse Folder. Native macOS file dialog opens.
They pick ~/code/my-rust-project.

**What happens behind the screen:**
- Tauri native file dialog (researched in docs/tauri-research/)
- No database exists yet. Nothing to configure.
- Tauri research: docs/tauri-research/

---

## Screen 2: Ingestion Progress

```
+------------------------------------------+
|  Parseltongue                        [-] |
|------------------------------------------|
|                                          |
|  Analyzing: my-rust-project              |
|  /Users/dev/code/my-rust-project         |
|                                          |
|  [========..............] 42%            |
|                                          |
|  Tree-sitter parsing...     142 files    |
|  Rust compiler analysis...  in progress  |
|  Building graph...          pending      |
|                                          |
|  Found so far:                           |
|    387 entities  .  1,204 edges          |
|                                          |
+------------------------------------------+
```

Two passes:
1. Fast pass (tree-sitter): entities + basic edges for all languages. Seconds.
2. Deep pass (rustc_private): compiler-verified types, real call graph, trait impls. Rust only.

macOS notification when done: "my-rust-project analyzed. 387 entities, 1,204 edges."

**What happens behind the screen:**
- Walk folder tree -> create folder entities (Layer 0) and file entities (Layer 1)
- For each parsable file -> tree-sitter extracts line-range entities (Layer 2)
- For .rs files -> rustc_private adds compiler truth (Layer 3)
- All stored in Turso/libSQL database managed by the app
- Database location is invisible to the user

---

## Screen 3: Home Screen (With Data)

```
+------------------------------------------+
|  Parseltongue                        [-] |
|------------------------------------------|
|                                          |
|  YOUR CODEBASES                          |
|                                          |
|  +------------------------------------+  |
|  | my-rust-project           FRESH    |  |
|  | ~/code/my-rust-project             |  |
|  | 387 entities . 1,204 edges         |  |
|  | Last analyzed: 2 min ago           |  |
|  |                                    |  |
|  | HTTP: localhost:7777  [ Copy URL ] |  |
|  | [ Re-analyze ]  [ Open Terminal ]  |  |
|  +------------------------------------+  |
|                                          |
|  [ + Add Another Codebase ]              |
|                                          |
+------------------------------------------+
```

- FRESH / STALE badge -- compares DB timestamp to git HEAD or file mod times
- Copy URL -- one click to get the HTTP endpoint for LLM tools
- DB location is invisible -- Tauri manages it
- Each workspace = one codebase + one DB + one HTTP server instance

**What happens behind the screen:**
- Tauri spawns/manages an HTTP server per workspace
- Staleness check: compare stored file hashes to current files on disk
- The HTTP URL is the interface for LLMs and CLI users

---

## Screen 4: The HTTP Query (7-Event Journey begins)

User copies localhost:7777 into Claude Code / Cursor / terminal.

```
curl "http://localhost:7777/query?q=authentication+flow"
```

This is NOT a Tauri screen. This is an HTTP response.
The 7-event journey runs server-side:

**Event 1: QUERY** -- the ~7 words arrive

**Event 2: SEARCH** -- RRF fusion finds 4 candidates (<10ms)
  - FTS5 full-text search (name + signature + doc_comment, weighted)
  - Symbol trie (exact matches on identifiers)
  - Trigram index (fuzzy matches)
  - Git history (recent edits boosted)
  - RRF ref: docs/research-glommer/codemogger/src/search/rank.ts (k=60, fts=0.4, vec=0.6)
  - Query preprocessing ref: docs/research-glommer/codemogger/src/search/query.ts (stopwords)

**Event 3: ANCHOR** -- BFS upward to public API boundary (<50ms)
  - For private entities: walk callers until a public fn/trait is found
  - For public entities: anchor is itself

**Event 4: CLUSTER** -- ego network 1-hop for each anchor (<100ms)
  - Cluster = anchor + callers + callees + implementations
  - Each cluster compressed to ~3000 tokens

---

## Screen 5: The HTTP Response (Cluster Selection)

The HTTP response presents 4 candidate clusters (~200 tokens):

```json
{
  "query": "authentication flow",
  "clusters": [
    {
      "id": 1,
      "label": "API HANDLER",
      "anchor": "src/api/handlers.rs:45:78",
      "name": "login_route",
      "summary": "HTTP endpoint, calls auth::login",
      "entity_count": 5,
      "edge_count": 8
    },
    {
      "id": 2,
      "label": "AUTH TRAIT",
      "anchor": "src/auth/provider.rs:12:35",
      "name": "AuthProvider",
      "summary": "Abstraction, 2 impls: JWT, OAuth",
      "entity_count": 7,
      "edge_count": 12
    },
    {
      "id": 3,
      "label": "MODULE",
      "anchor": "src/auth/",
      "name": "authentication",
      "summary": "Folder with 12 files",
      "entity_count": 24,
      "edge_count": 45
    },
    {
      "id": 4,
      "label": "EXTERNAL",
      "anchor": "src/oauth/client.rs:8:42",
      "name": "oauth",
      "summary": "Third-party integration",
      "entity_count": 3,
      "edge_count": 5
    }
  ],
  "prompt": "Which cluster? [1] [2] [3] [4] [none]"
}
```

LLM or human picks one. Token cost to decide: ~200 tokens.

---

## Screen 6: The Deep Dive Response

User/LLM chose [1]. Full context returned (up to 20k tokens):

```json
{
  "cluster_id": 1,
  "anchor": {
    "location": "src/api/handlers.rs:45:78",
    "name": "login_route",
    "kind": "function",
    "signature": "pub async fn login_route(req: Request) -> Result<Response>",
    "visibility": "pub",
    "code": "pub async fn login_route(req: Request) -> Result<Response> {\n    ...\n}"
  },
  "callers": [ "..." ],
  "callees": [ "..." ],
  "type_signatures": { "..." },
  "control_flow": { "..." },
  "git_history": [ "..." ],
  "next_queries": [
    "blast_radius(src/api/handlers.rs:45:78)",
    "type_flow(src/api/handlers.rs:45:78)",
    "call_slice(src/api/handlers.rs:45:78)"
  ]
}
```

Note: anchors are referenced by physical location (file:line:line).
Rust entities get compiler-verified signatures, types, control flow.
Other languages get tree-sitter-level information.

---

## Screen 7: Coming Back Tomorrow

User opens Parseltongue the next day.

```
+------------------------------------------+
|  Parseltongue                        [-] |
|------------------------------------------|
|                                          |
|  YOUR CODEBASES                          |
|                                          |
|  +------------------------------------+  |
|  | my-rust-project           STALE    |  |
|  | ~/code/my-rust-project             |  |
|  | 387 entities . 1,204 edges         |  |
|  | Last analyzed: yesterday           |  |
|  | 3 commits behind                   |  |
|  |                                    |  |
|  | HTTP: stopped     [ Start Server ] |  |
|  | [ Re-analyze ]  [ Open Terminal ]  |  |
|  +------------------------------------+  |
|                                          |
+------------------------------------------+
```

One click to re-analyze. Server can restart with old data while re-ingestion runs.

---

## Token Economics

### The wc-to-Token Bridge

Every entity stores `wc` (word count). Tokens ~ wc * 1.3 (rough average for code).
This means we can compute exact token economics from the database itself:

    SELECT
      SUM(CASE WHEN entity_type IN ('function','method','struct',...) THEN wc END) as searchable_wc,
      SUM(CASE WHEN entity_type = 'doc_comment' THEN wc END) as doc_wc,
      SUM(CASE WHEN entity_type = 'import' THEN wc END) as import_wc,
      SUM(CASE WHEN entity_type IN ('comment','whitespace') THEN wc END) as overhead_wc,
      SUM(wc) as total_wc
    FROM entities WHERE file_path LIKE 'src/%';

    -- "Your codebase: 1.2M words (1.56M tokens).
    --  Searchable: 920K words (1.2M tokens). Overhead: 280K words.
    --  You queried with 7 words. We returned 200 tokens to decide,
    --  then 20K tokens of understanding. 99.7% token reduction."

### Per-Query Token Flow

| Screen | Tokens (Internal) | Tokens (to LLM) |
|--------|-------------------|-----------------|
| Screen 4: Query arrives | 0 | ~7 words |
| Event 2: Search | 30 | - |
| Event 3: Anchor | 100 | - |
| Event 4: Cluster | 12,000 | - |
| Screen 5: Cluster selection | - | ~200 |
| Screen 6: Deep dive | - | Up to 20,000 |

LLM pays ~200 tokens to choose, then up to 20k for ONE deep dive (not 80k for all 4).

---

# PART II: THE PLAN

## Codemogger: Our Foundation (What They Built)

Codemogger is a code indexing and search library by the Turso team.
Source: docs/research-glommer/codemogger/src/ (~2,100 lines across 14 TypeScript files)

| Component | Codemogger file | Lines | What it does |
|-----------|----------------|-------|-------------|
| File walker | src/scan/walker.ts | 122 | Walk dir, .gitignore, SHA-256, skip >1MB/empty/hidden |
| Tree-sitter chunker | src/chunk/treesitter.ts | 340 | 14 languages, processNode, splitLargeNode (>150 lines) |
| Language configs | src/chunk/languages.ts | 247 | Data-driven topLevelNodes + splitNodes per language |
| Chunk types | src/chunk/types.ts | 12 | CodeChunk interface, chunkKey = file:startLine:endLine |
| DB schema | src/db/schema.ts | 85 | 3 tables (codebases, chunks, indexed_files) + FTS5 |
| DB store | src/db/store.ts | 408 | CRUD, batch upsert, FTS lifecycle, vector search |
| Indexing pipeline | src/index.ts | 370 | 5-phase: scan -> hash -> chunk -> embed -> cleanup -> FTS |
| Search query | src/search/query.ts | 73 | Keyword extraction, stopword removal, 73 stopwords |
| Search rank | src/search/rank.ts | 49 | RRF merge: w_fts=0.4, w_vec=0.6, k=60 |
| Local embedding | src/embed/local.ts | 40 | all-MiniLM-L6-v2, q8 quantized, 384-dim (97% of time) |
| CLI | bin/codemogger.ts | 195 | Commands: index, search, list, mcp |
| MCP server | src/mcp.ts | 165 | 3 tools: codemogger_search, codemogger_index, codemogger_reindex |

### What Parseltongue Steals from Codemogger

| Pattern | Codemogger source | Our use |
|---------|------------------|---------|
| file:startLine:endLine as chunk key | src/chunk/types.ts | Our primary key (Big-Rock-02) |
| SHA-256 hash-based incremental indexing | src/index.ts | Skip unchanged files on re-analyze |
| Simplified .gitignore (directory names only) | src/scan/walker.ts | File walker |
| ALWAYS_IGNORE hardcoded set | src/scan/walker.ts | .git, node_modules, target, build, dist, etc. |
| Per-codebase FTS with weighted fields | src/db/schema.ts | name=5.0, signature=3.0 |
| Single .db file per codebase | src/db/schema.ts | .parseltongue/index.db |
| Data-driven LanguageConfig | src/chunk/languages.ts | Our LanguageConfigDefinition struct |
| topLevelNodes + splitNodes pattern | src/chunk/treesitter.ts | Our entity extraction walker |
| RRF fusion formula | src/search/rank.ts | Extended from 2 signals to 4 |
| Query preprocessing (stopwords) | src/search/query.ts | Strip agent filler before FTS |

### What Parseltongue Does NOT Steal

| Pattern | Why not |
|---------|---------|
| Embeddings (all-MiniLM-L6-v2) | CPU-only guarantee. 97% of indexing time. Sourcegraph removed embeddings from Cody Enterprise. |
| MCP as primary interface | HTTP-first (D4). MCP can be a thin wrapper later. |
| Bun/TypeScript runtime | We're Rust. |
| Vector search in DB | CPU-only. Use FTS5 + symbol trie + trigram + git instead. |

### What Parseltongue Adds Beyond Codemogger (~70% of total scope)

| Feature | Complexity | Codemogger has it? |
|---------|-----------|-------------------|
| Tauri Mac desktop app | Very High | No |
| rustc_private compiler enrichment | Very High | No |
| 22 HTTP REST endpoints | Very High | No (3 MCP tools) |
| Graph algorithms (SCC, k-core, PageRank, Leiden, SQALE, entropy, CK) | Very High | No |
| BFS anchoring (private -> public elevation) | High | No |
| Ego network clustering | High | No |
| 7-event journey pipeline | High | No |
| Symbol trie + trigram index + git history RRF | High | No (uses embeddings instead) |
| Folder/file structural entities (L0-L1) | Medium | No |
| 100% wc coverage model | Medium | No |
| File watcher + staleness detection | Medium | No |

---

## Crate Structure

```
crates/
  parseltongue-core/                    # M1-M4: types, walker, chunker, storage, search, graph
  pt01-codebase-ingestion-engine/       # M3: ingestion pipeline + CLI
  pt08-http-query-api-server/           # M5: Axum HTTP server (22 endpoints)
  pt09-rustc-compiler-enrichment/       # M7: rustc_private Layer 3 (nightly-only)
  pt10-tauri-workspace-manager/         # M8: Tauri Mac app
```

Dependency flow: `parseltongue` (root binary) -> `pt01`/`pt08`/`pt10` -> `parseltongue-core`

Starting point:
- `crates/` directory exists but is empty -- clean slate
- Workspace Cargo.toml already has all 14 tree-sitter grammar crates + tokio + clap + serde
- v1.6.1 graph algorithms (2,812 lines, 11 files) at toBeDeleted/archived-code/parseltongue-core/src/graph_analysis/ -- portable
- v1.6.1 HTTP server at toBeDeleted/archived-code/pt08-http-code-query-server/ -- reference only
- PRD: this file

New workspace dependencies needed:

    # Add to [workspace.dependencies] in Cargo.toml
    libsql = "0.6"                    # Turso/libSQL (replaces cozo/rocksdb)
    sha2 = "0.10"                     # SHA-256 file hashing
    walkdir = "2.5"                   # Directory traversal
    axum = "0.8"                      # HTTP server
    tower-http = "0.6"                # CORS middleware

---

## Milestone Map

```
M1 (Types + Storage)
 |
 v
M2 (Walker + Chunker) --------> M6 (Edge Extraction)
 |                                |
 v                                v
M3 (Pipeline + CLI) ----------> M4 (Search + Graph Algorithms)
                                  |
                                  v
                               M5 (HTTP Server)
                                  |
                             +----+----+
                             v         v
                          M7 (rustc) M8 (Tauri)
```

Critical path: M1 -> M2 -> M3 -> M4 -> M5
Parallel track: M6 can start after M2 (before M3 finishes)
Independent: M7 and M8 proceed in parallel after M5

| Milestone | Est. Lines | Cumulative | Deliverable |
|-----------|-----------|------------|-------------|
| M1: Types + Storage | ~1,200 | 1,200 | Entity model, libSQL schema, Store CRUD |
| M2: Walker + Chunker | ~1,800 | 3,000 | 14-language tree-sitter, 100% wc coverage |
| M3: Pipeline + CLI | ~800 | 3,800 | `parseltongue ingest .` works end-to-end |
| M4: Search + Graph | ~3,500 | 7,300 | 4-signal RRF search, 7+ graph algorithms |
| M5: HTTP Server | ~3,000 | 10,300 | 22+ endpoints, 7-event journey |
| M6: Edge Extraction | ~1,500 | 11,800 | Dependency graph from tree-sitter |
| M7: rustc_private | ~2,000 | 13,800 | Compiler-enriched entities for .rs files |
| M8: Tauri App | ~2,500 | 16,300 | Native Mac GUI, workspace management |

---

## M1: Foundation Types + Turso Storage (~1,200 lines)

**Port from:** codemogger src/db/schema.ts (85 lines) + src/db/store.ts (408 lines) + src/chunk/types.ts (12 lines)

### Files to create

    parseltongue-core/
      Cargo.toml
      src/
        lib.rs
        entity_type_definitions.rs     # EntityPrimaryKeyLocation, EntityTypeClassification (30 types), CodeEntityRecord, EdgeRecord
        storage/
          mod.rs
          schema_definition_tables.rs  # SQL DDL: entities, edges, codebases, indexed_files, per-codebase FTS5
          turso_storage_client.rs      # Store CRUD: open, upsert entities/edges, hash check, FTS rebuild, cleanup

### Key data structures

    // parseltongue-core/src/entity_type_definitions.rs

    /// Primary key: path:start_line:end_line
    /// Sentinels: -1:-1 = folder, 0:0 = file, N:M (N>=1) = code span
    pub struct EntityPrimaryKeyLocation {
        pub file_path: String,
        pub start_line: i32,   // -1 for folders, 0 for files, 1+ for code spans
        pub end_line: i32,
    }

    /// 30 entity types for 100% coverage
    pub enum EntityTypeClassification {
        // Structural (4)
        Folder, FileParsable, FileUnparsable, FileConfig,
        // Searchable code (18)
        Function, Method, Struct, Class, Enum, Trait, Interface, Impl,
        TypeAlias, Constant, Static, Macro, Module, Variable,
        Constructor, Namespace, Record, Object,
        // Non-code (5) — coverage only
        Import, DocComment, Comment, Attribute, Whitespace,
        // Rust config (3)
        Dependency, PackageMeta, ConfigSection,
    }

    /// The core entity stored in the database
    pub struct CodeEntityRecord {
        pub pk: EntityPrimaryKeyLocation,
        pub entity_type: EntityTypeClassification,
        pub language: String,
        pub name: String,
        pub signature: String,
        pub snippet: String,
        pub doc_comment: String,    // Folded from adjacent /// or /** */
        pub wc: u32,                // Word count of source text
        pub file_hash: String,
        pub is_test: bool,
        pub codebase_id: i64,
    }

### Database schema (Turso/libSQL — see Part III for full DDL)

4 tables + dynamic per-codebase FTS:

    codebases(id, root_path UNIQUE, name, indexed_at)
    indexed_files(codebase_id, file_path, file_hash, chunk_count, indexed_at)
    entities(file_path, start_line, end_line, codebase_id, entity_type, language, name,
             signature, snippet, doc_comment, wc, file_hash, is_test, indexed_at,
             rustc_scope?, rustc_sig?, visibility?, mir_calls?, trait_impls?)
    edges(from_path, from_start, from_end, to_path, to_start, to_end, edge_type, codebase_id)
    fts_{codebase_id}(entity_rowid, name, signature, doc_comment)  -- weights: 5.0, 3.0, 2.0

### Store API (4-word functions)

    pub struct TursoStorageClient { /* libsql::Connection */ }

    impl TursoStorageClient {
        pub async fn open_database_connection_path(db_path: &str) -> Result<Self>;
        pub async fn initialize_schema_tables_all() -> Result<()>;
        pub async fn get_or_create_codebase_entry(root_path: &str) -> Result<i64>;
        pub async fn get_stored_file_hash_value(codebase_id: i64, file_path: &str) -> Result<Option<String>>;
        pub async fn batch_upsert_entity_records(codebase_id: i64, entities: &[CodeEntityRecord]) -> Result<()>;
        pub async fn batch_upsert_edge_records(codebase_id: i64, edges: &[EdgeRecord]) -> Result<()>;
        pub async fn remove_stale_file_entities(codebase_id: i64, active_files: &HashSet<String>) -> Result<u32>;
        pub async fn rebuild_fts_index_table(codebase_id: i64) -> Result<()>;
    }

### Tests
- EntityPrimaryKeyLocation parse/format round-trip (folder, file, code span sentinel detection)
- EntityTypeClassification: all 30 variants serialize correctly
- In-memory libSQL: schema creation, entity CRUD, FTS rebuild + search
- File hash check: stored hash matches -> skip, differs -> process

### Risks
- libsql crate maturity: newer than Node.js driver. Verify FTS5 + fts_match/fts_score. Fallback: rusqlite with bundled SQLite.
- File locking: memelord research (docs/research-glommer/memelord/) warns about Turso embedded locking. Mitigation: short-lived connections, PRAGMA busy_timeout=5000.

---

## M2: File Walker + Tree-Sitter Chunker (~1,800 lines)

**Port from:** codemogger src/scan/walker.ts (122 lines) + src/chunk/treesitter.ts (340 lines) + src/chunk/languages.ts (247 lines)

### Files to create

    parseltongue-core/src/
      walker/
        mod.rs
        directory_tree_scanner.rs      # Walk dir, .gitignore, SHA-256, skip >1MB/empty/hidden
        language_config_registry.rs    # 14 LanguageConfig structs (top_level_nodes, split_nodes, node->entity_type)
        treesitter_entity_extractor.rs # processNode, splitLargeNode, extractName, extractSignature
        doc_comment_folding_logic.rs   # Fold /// and //! into adjacent entity's doc_comment field
        word_count_coverage_tracker.rs # wc per entity, whitespace gap computation, sum verification

### What changes from Codemogger

1. **100% wc coverage**: Every root node child becomes an entity (not just topLevelNodes). Comments, imports, whitespace gaps all get entity records.
2. **doc_comment folding**: `///` and `//!` merged into adjacent code entity's `doc_comment` field for FTS.
3. **Comment discrimination**: Inspect text -- `///`/`//!` = doc_comment, `//` = plain comment.
4. **is_test detection**: `#[test]` (Rust), `test_` prefix (Python), `@Test` (Java), etc.
5. **Folder entities (L0)**: Walker emits folder entity for every directory.
6. **File entities (L1)**: Every file gets file_parsable/file_unparsable/file_config with total_wc.
7. **Cargo.toml parsing**: TOML -> dependency/package_meta/config_section entities.

### Language configs (14 languages, data-driven)

Each LanguageConfigDefinition is a const static:

    pub struct LanguageConfigDefinition {
        pub name: &'static str,
        pub extensions: &'static [&'static str],
        pub top_level_nodes: &'static [&'static str],
        pub split_nodes: &'static [&'static str],
        pub tree_sitter_language: fn() -> tree_sitter::Language,
    }

Reference: docs/research-glommer/codemogger/src/chunk/languages.ts (14 configs)
Reference: Tree-Sitter Node Type Mapping tables below (complete node->entity_type per language)

### Key functions (4-word names)

    pub fn scan_directory_tree_recursive(root: &Path) -> Result<(Vec<ScannedFileRecord>, Vec<FolderEntityRecord>)>;
    pub fn detect_language_from_extension(file_path: &str) -> Option<&'static LanguageConfigDefinition>;
    pub fn extract_entities_from_source(file_path: &str, content: &str, config: &LanguageConfigDefinition) -> Result<Vec<CodeEntityRecord>>;
    fn classify_node_to_entity_type(node_kind: &str, lang: &str) -> EntityTypeClassification;
    fn extract_name_from_node(node: &tree_sitter::Node, source: &[u8]) -> String;
    fn fold_doc_comment_into_entity(entities: &mut [CodeEntityRecord]);
    fn compute_whitespace_gap_entities(file_entities: &[CodeEntityRecord], file_total_wc: u32) -> Vec<CodeEntityRecord>;

### Tests
- Per-language fixture: parse small file, verify entity count + types
- Doc comment folding: `///` merges into adjacent function's doc_comment field
- wc coverage: sum(entity.wc) == file.total_wc for fixtures in every language
- is_test detection per language
- Integration: scan Parseltongue repo itself, verify plausible entity counts

### Risks
- tree-sitter Rust crate (0.25) uses native bindings, not WASM. API differs slightly from codemogger's WASM usage. Well understood from v1.6.1 code.

---

## M3: Ingestion Pipeline + CLI (~800 lines)

**Port from:** codemogger src/index.ts (370 lines) + bin/codemogger.ts (195 lines)

### Files to create

    pt01-codebase-ingestion-engine/
      Cargo.toml
      src/
        lib.rs
        ingestion_pipeline_orchestrator.rs  # 6-phase pipeline
        cli_argument_parser_setup.rs        # Clap args

    parseltongue/                           # Root binary crate
      src/main.rs                           # Dispatch: ingest, serve, enrich

### Pipeline phases (NO embedding phase -- CPU-only)

1. **SCAN**: Walk directory, collect files + folders
2. **HASH CHECK**: Compare SHA-256 to stored hashes, skip unchanged
3. **CHUNK**: Tree-sitter parse changed files -> entities with wc + doc_comment
4. **WRITE**: Batch upsert to Turso (FILE_BATCH=200)
5. **CLEANUP**: Remove entities for deleted files
6. **FTS REBUILD**: Rebuild per-codebase FTS5 index
7. **VERIFY** (debug builds): Assert sum(entity.wc) == file.total_wc

### Key types

    pub struct IngestionPipelineOrchestrator { store: TursoStorageClient }

    pub struct IngestionResultSummary {
        pub files_processed: u32,
        pub entities_created: u32,
        pub files_skipped: u32,
        pub files_removed: u32,
        pub duration_ms: u64,
    }

### Tests
- E2E: temp dir with fixtures -> ingest -> verify entities in DB
- Incremental: ingest, modify one file, re-ingest -> only that file re-processed
- Stale removal: ingest, delete file, re-ingest -> entities removed
- FTS: ingest -> search by function name -> verify found

---

## M4: Search Engine + Graph Algorithms (~3,500 lines)

### M4a: Search Engine (~700 lines, NEW -- replaces Codemogger's embeddings)

**Port from:** codemogger src/search/query.ts (73 lines) + src/search/rank.ts (49 lines)

    parseltongue-core/src/
      search/
        mod.rs
        query_preprocessor_engine.rs    # Stopword removal, agent filler, keyword extraction
        symbol_trie_exact_index.rs      # In-memory Patricia trie over entity names (~200 lines)
        trigram_fuzzy_match_index.rs    # Trigram index for partial/fuzzy matching (~200 lines)
        git_history_recency_signal.rs   # `git log` -> recency scores per file (~100 lines)
        rank_fusion_combiner_rrf.rs     # RRF over 4 signals (~100 lines)

RRF formula: `score = w1/(k+rank_fts) + w2/(k+rank_trie) + w3/(k+rank_trigram) + w4/(k+rank_git)`
Default weights: w1=0.35 (FTS), w2=0.30 (trie), w3=0.20 (trigram), w4=0.15 (git), k=60

### M4b: Graph Algorithms (~2,800 lines, PORT from v1.6.1)

v1.6.1 source: toBeDeleted/archived-code/parseltongue-core/src/graph_analysis/ (11 files, 2,812 lines)
These algorithms operate on generic adjacency lists, not on specific DB types. Port is straightforward.

    parseltongue-core/src/
      graph/
        mod.rs
        adjacency_list_graph.rs            # from adjacency_list_graph_representation.rs (267 lines)
        tarjan_scc_detection.rs            # Tarjan's SCC (183 lines)
        kcore_decomposition_layering.rs    # k-core (215 lines)
        centrality_pagerank_betweenness.rs # PageRank + Betweenness (325 lines)
        leiden_community_clustering.rs     # Leiden (328 lines)
        entropy_shannon_complexity.rs      # Shannon entropy (186 lines)
        ck_metrics_coupling_cohesion.rs    # CBO/LCOM/RFC/WMC (402 lines)
        sqale_technical_debt_scoring.rs    # SQALE ISO 25010 (340 lines)
        bfs_anchor_public_traversal.rs     # NEW: BFS private -> public (~150 lines)
        ego_network_cluster_builder.rs     # NEW: 1-hop ego network (~200 lines)
        test_fixture_reference_graphs.rs   # from test_fixture_reference_graphs.rs (116 lines)

### New graph functions (4-word names)

    /// BFS from a private entity upward through callers to nearest public API boundary
    pub fn anchor_bfs_to_public_boundary(
        graph: &AdjacencyListGraphRepresentation,
        start_entity: &str,
        entity_visibility: &HashMap<String, bool>,
    ) -> Vec<String>;

    /// 1-hop ego network: anchor + all callers + callees + implementations
    pub struct EgoNetworkCluster {
        pub anchor: String,
        pub entities: Vec<String>,
        pub edges: Vec<(String, String, String)>,
        pub estimated_tokens: u32,
    }

    pub fn build_ego_network_cluster(
        graph: &AdjacencyListGraphRepresentation,
        anchor: &str,
        token_budget: u32,
    ) -> EgoNetworkCluster;

### Tests
- All v1.6.1 algorithm tests carry over (they test against reference graphs)
- BFS anchor: synthetic graph with public/private entities -> verify anchor found
- Ego network: verify 1-hop neighborhood correct
- Search RRF: fixture entities -> verify ranking with 4 signals
- Performance: symbol trie lookup < 1ms on 10K entities

### Risks
- Symbol trie memory: 40K entities -> Patricia trie must stay < 50MB (compresses well)
- Git history subprocess: `git log` via std::process::Command. Handle missing git, shallow clones.

---

## M5: HTTP REST API Server (~3,000 lines)

**Reference:** toBeDeleted/archived-code/pt08-http-code-query-server/ (9,478 lines -- v3.0 will be leaner)

### Files to create

    pt08-http-query-api-server/
      Cargo.toml
      src/
        lib.rs
        http_server_startup_runner.rs       # Axum router setup, bind port
        shared_server_app_state.rs          # Store + Graph + SymbolTrie + TrigramIndex + GitRecency
        route_definition_builder.rs         # All 22+ routes
        handlers/
          mod.rs
          health_check_endpoint.rs          # /server-health-check-status
          statistics_overview_endpoint.rs   # /codebase-statistics-overview-summary
          api_documentation_endpoint.rs     # /api-reference-documentation-help
          entities_list_endpoint.rs         # /code-entities-list-all
          entity_detail_endpoint.rs         # /code-entity-detail-view/:key
          search_fuzzy_endpoint.rs          # /code-entities-search-fuzzy?q=
          edges_list_endpoint.rs            # /dependency-edges-list-all
          callers_query_endpoint.rs         # /reverse-callers-query-graph?entity=
          callees_query_endpoint.rs         # /forward-callees-query-graph?entity=
          blast_radius_endpoint.rs          # /blast-radius-impact-analysis?entity=&hops=
          circular_deps_endpoint.rs         # /circular-dependency-detection-scan
          hotspots_ranking_endpoint.rs      # /complexity-hotspots-ranking-view?top=
          cluster_grouping_endpoint.rs      # /semantic-cluster-grouping-list
          smart_context_endpoint.rs         # /smart-context-token-budget?focus=&tokens=
          scc_analysis_endpoint.rs          # /strongly-connected-components-analysis
          sqale_scoring_endpoint.rs         # /technical-debt-sqale-scoring?entity=
          kcore_layering_endpoint.rs        # /kcore-decomposition-layering-analysis?k=
          centrality_ranking_endpoint.rs    # /centrality-measures-entity-ranking?method=
          entropy_scores_endpoint.rs        # /entropy-complexity-measurement-scores?entity=
          coupling_metrics_endpoint.rs      # /coupling-cohesion-metrics-suite?entity=
          leiden_clusters_endpoint.rs       # /leiden-community-detection-clusters
          coverage_report_endpoint.rs       # /ingestion-coverage-folder-report?depth=
          seven_event_journey_endpoint.rs   # /query?q=  (THE flagship endpoint)

### Server state (loaded at startup)

    pub struct SharedServerAppState {
        pub store: TursoStorageClient,
        pub graph: AdjacencyListGraphRepresentation,  // built from edges table
        pub symbol_trie: SymbolTrieIndex,
        pub trigram_index: TrigramFuzzyMatchIndex,
        pub git_recency: HashMap<String, f64>,
        pub codebase_id: i64,
    }

### The 7-event journey endpoint (/query?q=...)

Orchestrates the full journey:
1. QUERY: parse ~7 word input
2. SEARCH: RRF over 4 signals -> top 4 candidates (<10ms)
3. ANCHOR: BFS from each to public API boundary (<50ms)
4. CLUSTER: Ego network 1-hop per anchor (<100ms)
5. ASK: Return 4 cluster summaries (~200 tokens)
Then `/query?q=...&cluster=N` triggers DEEP DIVE (up to 20K tokens).

### Tests
- Integration: start server with test DB -> HTTP requests -> verify JSON responses
- Performance: /query < 200ms on 1000-entity DB
- Error handling: invalid entity keys, missing params -> proper error responses

---

## M6: Dependency Edge Extraction (~1,500 lines)

**Can start in parallel with M3** (needs M2's tree-sitter walker but not the full pipeline)

    parseltongue-core/src/
      edges/
        mod.rs
        dependency_edge_extractor.rs     # Extract call/import/implement edges from AST (~500 lines)
        import_resolution_mapper.rs      # Resolve import paths to target entities (~300 lines)
        hierarchy_edge_builder.rs        # folder->file, file->entity containment edges (~200 lines)

### Edge types

| Type | Description | Detection |
|------|-------------|-----------|
| `calls` | Function calls another function | call_expression, method_call nodes |
| `imports` | File imports from another module | use_declaration, import_statement nodes |
| `implements` | Type implements a trait/interface | impl_item target type, class parent |
| `type_refs` | Entity references a type | Type identifiers in signatures |
| `field_access` | Entity accesses a field | Field expressions |
| `contains` | Structural hierarchy | folder->file, file->entity |

v1.6.1 had 8 edge types (calls, uses, implements, type_refs, field_access, async_await, iterators, generics).
Ref: toBeDeleted/archived-code/parseltongue-core/src/dependency_queries/rust.scm (180 lines)

### Tests
- Per-language fixture: small file with known calls -> verify edges
- Hierarchy: folder->file->entity containment
- Import resolution: `use crate::auth::login` -> correct entity pk

### Risks
- Cross-file resolution is heuristic (tree-sitter is syntactic). Layer 3 (M7) fixes this for Rust.

---

## M7: Rust Compiler Enrichment (~2,000 lines)

    pt09-rustc-compiler-enrichment/
      Cargo.toml                         # requires nightly, #![feature(rustc_private)]
      src/
        lib.rs
        compiler_analysis_driver.rs      # Drive rustc on target project
        entity_enrichment_mapper.rs      # Match DefId -> (file, line) -> existing entity pk -> UPDATE
        mir_call_graph_extractor.rs      # Extract real call edges from MIR

### What gets enriched (UPDATE existing entity rows)

| Column | rustc API | Example value |
|--------|----------|---------------|
| rustc_scope | tcx.def_path_str() | "crate::auth::service::login" |
| rustc_sig | tcx.fn_sig() | "fn(&Credentials) -> Result<Token>" |
| visibility | tcx.visibility() | "pub(crate)" |
| mir_calls | tcx.optimized_mir() | ["crate::db::lookup", ...] |
| trait_impls | tcx.all_impls() | [...] |

Match strategy: rustc gives DefId -> Span -> (file, line). Match to entity by (file_path, start_line).

### Tests
- Unit: enrich small Rust crate, verify rustc_scope and rustc_sig populated
- Integration: verify MIR call edges match/superset tree-sitter edges
- Negative: run on non-Rust project, verify graceful no-op

### Risks
- Must pin exact nightly toolchain version. rustc_private API changes between nightlies.
- Target project must compile successfully for enrichment to work.
- Rationale: docs/v300/rustc_private_stability_rationale_202603091530.md

---

## M8: Tauri Mac Application (~2,500 lines)

    pt10-tauri-workspace-manager/
      Cargo.toml
      src-tauri/src/
        main.rs
        workspace_lifecycle_commands.rs   # add/remove workspace, start/stop server
        server_process_manager.rs        # Spawn HTTP server as Tokio task per workspace
        staleness_detection_checker.rs   # Compare file hashes -> FRESH/STALE
      src/
        App.tsx
        screens/
          WorkspacePicker.tsx            # Screen 1
          IngestionProgress.tsx          # Screen 2
          Dashboard.tsx                  # Screen 3
          QueryInterface.tsx             # Screens 4-7

### Tauri commands (4-word names)

    #[tauri::command]
    async fn add_workspace_folder_path(path: String) -> Result<WorkspaceInfo, String>;

    #[tauri::command]
    async fn start_ingestion_pipeline_run(workspace_id: String) -> Result<(), String>;

    #[tauri::command]
    async fn check_staleness_status_all() -> Result<Vec<WorkspaceStatus>, String>;

    #[tauri::command]
    async fn start_http_server_instance(workspace_id: String) -> Result<String, String>;

    #[tauri::command]
    async fn stop_http_server_instance(workspace_id: String) -> Result<(), String>;

### Tests
- Backend unit tests: workspace add/remove, staleness detection
- Frontend: manual testing (Tauri E2E testing is still immature)

### Risks
- Tauri v2 Mac-specific behaviors (notarization, file dialogs) need testing.
- Multi-workspace: multiple HTTP server Tokio tasks. Use task handles, not OS processes.
- Tauri research: docs/tauri-research/

---

## Verification Strategy

After each milestone:
- `cargo test --all` passes
- `cargo clippy --all` clean
- No TODO/STUB/PLACEHOLDER in committed code
- For M3+: `parseltongue ingest .` works on this repo
- For M5+: `curl http://localhost:7777/server-health-check-status` returns 200
- For M5+: 7-event journey `/query?q=handle` returns results in < 200ms

---

# PART III: THE SPECIFICATION

---

## Primary Key Format

Uniform for ALL entities: `path:start_line:end_line`

    src/auth/:-1:-1                    -> folder
    src/auth/service.rs:0:0            -> file
    src/auth/service.rs:8:25           -> code span (fn login)

Sentinels: `-1:-1` = folder, `0:0` = file, `N:M` (N >= 1) = code span.

---

## Coverage Model (grounded in apache/iggy: 2712 files, 775 dirs, 379K lines)
## (iggy cloned to: docs/research-glommer/iggy-sample/)

Every byte in every file is accounted for. Zero gaps. 100% coverage by construction.

### The Rule: Every Entity Has a `wc` (Word Count)

Every entity -- searchable or not -- stores a `wc` field (word count of its source text).
For any parsable file: `file.total_wc = sum(entity.wc for all entities in that file)`.
No gaps. No unaccounted words. This is how we track coverage and compute token economics.

Tree-sitter's root node children cover the entire file. We classify ALL of them (not just
code declarations). Whitespace gaps between root children are computed and accounted for.

### File Categories (what happens to each file after .gitignore)

    Category              iggy count    What Parseltongue does
    --------              ----------    ----------------------
    CODE (tree-sitter)    2143 files    Parse ALL root children -> entities with wc. Searchable.
      .rs                 1237            208K lines (Rust -- also gets Layer 3 enrichment)
      .java               320             36K lines
      .ts                 224             18K lines
      .cs                 220             25K lines
      .go                 127             17K lines
      .py                 11              2K lines
      .js                 4               177 lines
      .svelte             70              7K lines (if we add tree-sitter-svelte)

    RUST CONFIG           83 files      Parse as TOML -> dependency/package_meta/config_section entities
      Cargo.toml          83              5K lines
      (build.rs)          2               (counted in .rs above)

    DATA/CONFIG           132 files     File entity + hash + total_wc. NOT parsed.
      .toml (non-Cargo)   0
      .yml/.yaml          65              12K lines
      .json               63              14K lines
      .xml                3               238 lines
      .proto              1               41 lines

    DOCUMENTATION         56 files      File entity + hash + total_wc. NOT parsed.
      .md                 52              8K lines
      .txt                4               61 lines

    SCRIPTS/TOOLING       66 files      File entity + hash + total_wc. NOT parsed.
      .sh                 31              4K lines
      Dockerfile          8
      justfile            2
      .http               3               665 lines
      .editorconfig       3
      .gitignore          10
      .dockerignore       6
      other (no ext)      ~20

    BUILD SYSTEM          49 files      File entity + hash + total_wc. NOT parsed.
      .csproj/.sln/.props 23
      .kts                15
      .properties         7
      .bazel              2
      other               2

    BINARY/OPAQUE         56 files      File entity + hash. No wc (binary).
      .png                34
      .svg                12
      .pem                3
      .lock               7               (16K lines but generated, not useful)

### The Coverage Equation

For any parsable file, tree-sitter gives us ALL root node children. We classify every one:

    file.total_wc = sum(entity.wc) + whitespace_wc

    Where entities include ALL root children:
      code entities   (function, struct, impl, ...) -> searchable, snippet stored
      imports         (use_declaration, ...)         -> wc counted, drives edges
      doc_comments    (///, //!, /** */)             -> wc counted, folded into adjacent entity FTS
      comments        (// plain, /* block */)        -> wc counted only
      whitespace      (gaps between root children)   -> wc counted only

This gives us per-file, per-folder, and per-repo breakdowns:

    Per file:
      SELECT entity_type, SUM(wc) as words,
             ROUND(SUM(wc) * 100.0 / file.total_wc, 1) as pct
      FROM entities WHERE file = 'src/auth/service.rs'
      GROUP BY entity_type;

      -- function     340 words   56.7%    <- searchable
      -- struct       120 words   20.0%    <- searchable
      -- import        30 words    5.0%    <- graph edges
      -- doc_comment   50 words    8.3%    <- searchable via code spans
      -- comment       30 words    5.0%    <- coverage only
      -- whitespace    30 words    5.0%    <- coverage only
      -- TOTAL        600 words  100.0%

    Per repo (apache/iggy scale):
      Searchable code:    ~800K words (~1M tokens)    <- FTS indexes this
      Doc comments:       ~120K words (~156K tokens)   <- searchable via code spans
      Imports:             ~80K words                   <- drives graph edges only
      Plain comments:     ~100K words                   <- coverage only
      Whitespace:         ~100K words                   <- coverage only
      Total:             ~1.2M words                    <- 100% accounted for

---

## Searchability Rule

- SEARCHABLE: Code entities (function, struct, impl, ...) go into FTS. Name + signature + snippet + doc_comment.
- SEARCHABLE VIA CODE SPANS: Doc comments (///, //!, /** */) folded into adjacent code span's
  `doc_comment` field. Searchable through FTS but not separate blobs.
- NOT SEARCHABLE: Folders, files, imports, plain comments, whitespace -- graph/coverage only.
- NOT STORED: Full file content is NEVER stored. Only parsed snippets.
- ALL COUNTED: Every entity has `wc`. Sum of all entity wc = file total wc. Zero gaps.

---

## Entity Taxonomy (`entity_type` column -- 100% file coverage)

Every entity has: `pk` (path:start_line:end_line), `entity_type`, `wc` (word count).
For parsable files, ALL tree-sitter root children get an entity_type. No bytes left uncounted.
Only module-level declarations become entities. Nested items (closures, inner fns, structs
inside function bodies) are part of their parent entity's snippet -- not separate entities.

### A. Structural Entities (graph-only, not searchable)

    entity_type       Example PK                     Description
    -----------       ----------                     -----------
    folder            src/auth/:-1:-1                Every directory in the tree
    file_parsable     src/auth/service.rs:0:0        Tree-sitter can parse. Stores total_wc + hash.
    file_unparsable   README.md:0:0                  Can't parse. Stores total_wc + hash.
    file_config       Cargo.toml:0:0                 Rust config only. Parsed as TOML.

All files store: file_hash (SHA-256), total_wc.
For parsable files: total_wc = sum(child entity wc). Verified on save.
Other languages' config files (package.json, pyproject.toml) = file_unparsable.

### B. Code Entities (searchable, FTS-indexed: name + signature + doc_comment)

Module-level declarations extracted by tree-sitter. Each stores snippet + wc.

    entity_type   Example PK                     tree-sitter node types
    -----------   ----------                     ----------------------
    function      src/main.rs:10:25              function_item, function_definition, function_declaration
    method        src/auth.rs:30:45              (inside impl/class via splitNodes)
    struct        src/model.rs:5:15              struct_item, struct_specifier
    class         src/app.py:1:50                class_definition, class_declaration
    enum          src/status.rs:3:12             enum_item, enum_declaration, enum_specifier
    trait         src/auth.rs:1:20               trait_item, trait_definition, trait_declaration
    interface     src/api.ts:5:30                interface_declaration
    impl          src/auth.rs:22:60              impl_item
    type_alias    src/types.rs:3:3               type_item, type_alias_declaration, type_definition
    constant      src/config.rs:1:1              const_item, const_declaration
    static        src/global.rs:5:5              static_item
    macro         src/macros.rs:1:20             macro_definition, preproc_def, preproc_function_def
    module        src/lib.rs:1:1                 mod_item, module
    variable      src/app.js:1:1                 lexical_declaration, variable_declaration, val_definition
    constructor   src/App.java:10:20             constructor_declaration
    namespace     src/lib.cpp:1:50               namespace_definition, namespace_declaration
    record        src/User.java:1:10             record_declaration
    object        src/App.scala:1:20             object_definition

Tests: not a separate entity_type -- `is_test=true` flag on function/method entities.

### C. Non-Code Entities (not searchable, but counted for 100% wc coverage)

These are tree-sitter root children that are NOT code declarations.
They exist so that sum(entity.wc) = file.total_wc with zero gaps.

    entity_type     Example PK                     tree-sitter node types
    -----------     ----------                     ----------------------
    import          src/main.rs:1:3                use_declaration, import_statement, extern_crate_item
    doc_comment     src/auth.rs:7:9                line_comment (///), block_comment (/** */), inner docs (//!)
    comment         src/main.rs:1:1                line_comment (//), block_comment (/* */)
    attribute       src/auth.rs:6:6                attribute_item (#[...]), decorator (@...)
    whitespace      (computed, not a TS node)       gaps between root children

    import:       wc counted + drives dependency graph edges. Not FTS-indexed.
    doc_comment:  wc counted + text folded into adjacent code entity's `doc_comment` FTS field.
                  Module doc comments (//!) folded into file_parsable entity's `doc_comment`.
    comment:      wc counted only. Not indexed. Not stored as blob.
    attribute:    wc counted + attached to next code entity (for is_test detection, etc.)
    whitespace:   wc counted only. Computed as: file.total_wc - sum(all other entity wc).

### D. Rust Config Span Entities (Cargo.toml only)

    entity_type     Example PK                     Description
    -----------     ----------                     -----------
    dependency      Cargo.toml:5:5                 A crate dependency declaration
    package_meta    Cargo.toml:1:4                 Package name, version, edition
    config_section  Cargo.toml:10:15               Named section ([features], [workspace], etc.)

### E. Rust Compiler Enrichment (Layer 3 -- extra columns, not new entity_types)

For .rs code entities, same pk, same row, more columns filled in:

    rustc_scope     tcx.def_path_str()     "crate::auth::service::login"
    rustc_sig       tcx.fn_sig()           "fn(&Credentials) -> Result<Token>"
    visibility      tcx.visibility()       "pub(crate)"
    mir_calls       tcx.optimized_mir()    ["crate::db::lookup", ...]
    trait_impls     tcx.all_impls()        [...]

### Entity Type Summary

    Searchable (FTS):      18 code entity_types (function through object)
    Graph-only:            4 structural (folder, file_parsable, file_unparsable, file_config)
                           1 import (drives edges)
                           1 attribute (attached to next entity)
    Coverage-only:         3 (doc_comment, comment, whitespace)
    Rust config:           3 (dependency, package_meta, config_section)
    Rust enrichment:       0 new types (extra columns on existing code entities)
    -------
    Total distinct:        30 entity_types

---

## Database Schema (Turso/libSQL)

Full DDL for M1 storage layer. Replaces CozoDB.

    -- Table 1: Codebases (one per indexed project)
    CREATE TABLE IF NOT EXISTS codebases (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        root_path  TEXT NOT NULL UNIQUE,
        name       TEXT NOT NULL DEFAULT '',
        indexed_at INTEGER NOT NULL
    );

    -- Table 2: Indexed files (tracks file hashes for incremental indexing)
    CREATE TABLE IF NOT EXISTS indexed_files (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        codebase_id INTEGER NOT NULL REFERENCES codebases(id),
        file_path   TEXT NOT NULL,
        file_hash   TEXT NOT NULL,
        chunk_count INTEGER NOT NULL DEFAULT 0,
        indexed_at  INTEGER NOT NULL,
        UNIQUE(codebase_id, file_path)
    );

    -- Table 3: Entities (the core table -- all 30 entity types)
    CREATE TABLE IF NOT EXISTS entities (
        file_path    TEXT NOT NULL,
        start_line   INTEGER NOT NULL,
        end_line     INTEGER NOT NULL,
        codebase_id  INTEGER NOT NULL REFERENCES codebases(id),
        entity_type  TEXT NOT NULL,        -- one of 30 EntityTypeClassification values
        language     TEXT NOT NULL DEFAULT '',
        name         TEXT NOT NULL DEFAULT '',
        signature    TEXT NOT NULL DEFAULT '',
        snippet      TEXT NOT NULL DEFAULT '',
        doc_comment  TEXT NOT NULL DEFAULT '',
        wc           INTEGER NOT NULL DEFAULT 0,
        file_hash    TEXT NOT NULL DEFAULT '',
        is_test      INTEGER NOT NULL DEFAULT 0,
        indexed_at   INTEGER NOT NULL,
        -- Layer 3 (rustc_private) columns, NULL until enriched
        rustc_scope  TEXT,
        rustc_sig    TEXT,
        visibility   TEXT,
        mir_calls    TEXT,                 -- JSON array of call targets
        trait_impls  TEXT,                 -- JSON array of trait impls
        PRIMARY KEY (file_path, start_line, end_line)
    );

    -- Table 4: Edges (dependency graph)
    CREATE TABLE IF NOT EXISTS edges (
        from_path     TEXT NOT NULL,
        from_start    INTEGER NOT NULL,
        from_end      INTEGER NOT NULL,
        to_path       TEXT NOT NULL,
        to_start      INTEGER NOT NULL,
        to_end        INTEGER NOT NULL,
        edge_type     TEXT NOT NULL,       -- calls, imports, implements, type_refs, contains, field_access
        codebase_id   INTEGER NOT NULL,
        PRIMARY KEY (from_path, from_start, from_end, to_path, to_start, to_end, edge_type)
    );

    -- Per-codebase FTS5 (created dynamically after indexing)
    CREATE VIRTUAL TABLE IF NOT EXISTS fts_{codebase_id} USING fts5(
        name, signature, doc_comment,
        content='entities',
        content_rowid='rowid'
    );
    -- Weights applied at query time: name=5.0, signature=3.0, doc_comment=2.0

    -- Indexes
    CREATE INDEX IF NOT EXISTS idx_entities_codebase ON entities(codebase_id);
    CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(codebase_id, entity_type);
    CREATE INDEX IF NOT EXISTS idx_edges_from ON edges(from_path, from_start, from_end);
    CREATE INDEX IF NOT EXISTS idx_edges_to ON edges(to_path, to_start, to_end);
    CREATE INDEX IF NOT EXISTS idx_indexed_files_hash ON indexed_files(codebase_id, file_path);

---

## Search Engine (4-Signal RRF, CPU-Only)

Replaces codemogger's 2-signal search (FTS + vector) with 4 CPU-only signals:

| Signal | Source | Speed | What it catches |
|--------|--------|-------|-----------------|
| FTS5 | Turso built-in BM25 on name + signature + doc_comment | <5ms | Keyword matches with ranking |
| Symbol trie | In-memory Patricia trie over all entity names | <1ms | Exact identifier lookup (camelCase/snake_case) |
| Trigram index | In-memory trigram index | <3ms | Fuzzy/partial matches, typos |
| Git history | `git log` recency scores per file | cached | Recently edited files boosted |

RRF fusion: `score = 0.35/(60+rank_fts) + 0.30/(60+rank_trie) + 0.20/(60+rank_trigram) + 0.15/(60+rank_git)`

Query preprocessing (ported from codemogger src/search/query.ts):
- Lowercase, split on non-alphanumeric (except hyphens/underscores)
- Filter: 3-30 chars per term, max 12 terms
- Remove 73 English stopwords + agent filler ("help", "please", "file", "code", "find")
- Deduplicate

---

## Tree-Sitter API Reference

### Core Node API (what we get per node)

Every tree-sitter node exposes these properties. This is the raw material for entity extraction.
Source: tree-sitter 0.25 C/Rust API (verified via Context7 + cargo cache node-types.json).
Grammar crate versions: Cargo.toml lines 31-45.
node-types.json files: ~/.cargo/registry/src/index.crates.io-*/tree-sitter-{lang}-*/src/node-types.json

    Property/Method              Returns              Used for
    ---------------              -------              --------
    node.kind()                  &str                 entity_type classification
    node.start_byte()            u32                  wc = end_byte - start_byte (-> byte count)
    node.end_byte()              u32                  wc calculation
    node.start_position()        { row, column }      start_line (row + 1, 1-based)
    node.end_position()          { row, column }      end_line (row + 1, 1-based)
    node.child_by_field_name()   Option<Node>         extract "name", "type", "trait" fields
    node.children()              Iterator<Node>       walk all children
    node.named_children()        Iterator<Node>       skip anonymous nodes (punctuation)
    node.parent()                Option<Node>         walk upward
    node.next_sibling()          Option<Node>         find adjacent doc_comments
    node.prev_sibling()          Option<Node>         find adjacent doc_comments
    node.is_named()              bool                 skip anonymous (keywords, brackets)
    node.text                    &str (via bytes)     snippet extraction

### Language Enumeration API (runtime node type discovery)

We do NOT need to hardcode node types. The Language API lets us enumerate at runtime:

    language.node_kind_count()            -> total number of node kinds
    language.node_kind_for_id(id: u16)    -> name string for each id
    language.node_kind_is_named(id: u16)  -> skip anonymous nodes
    language.field_count()                -> number of named fields
    language.field_name_for_id(id: u16)   -> field name by id

This means: at build time or first-run, we can generate the complete mapping table
for every grammar version we ship. No manual maintenance.

### Root Node Names Per Language

    Language       Root node type         Grammar crate in Cargo.toml
    --------       --------------         ---------------------------
    Rust           source_file            tree-sitter-rust 0.23
    Python         module                 tree-sitter-python 0.25
    JavaScript     program                tree-sitter-javascript 0.25
    TypeScript     program                tree-sitter-typescript 0.23
    Java           program                tree-sitter-java 0.23
    Go             source_file            tree-sitter-go 0.25
    C              translation_unit       tree-sitter-c 0.24
    C++            translation_unit       tree-sitter-cpp 0.23
    C#             compilation_unit       tree-sitter-c-sharp 0.23
    Ruby           program                tree-sitter-ruby 0.23
    Scala          compilation_unit       tree-sitter-scala 0.24
    PHP            program                tree-sitter-php 0.24
    Swift          source_file            tree-sitter-swift 0.7
    Kotlin         source_file            tree-sitter-kotlin 0.3

---

## Tree-Sitter Node Type -> entity_type Mapping (per language)

Every concrete node type that can appear as a direct child of the root node,
mapped to our entity_type. Extracted from node-types.json files in cargo cache.
Codemogger reference: docs/research-glommer/codemogger/src/chunk/languages.ts (topLevelNodes per lang)
Codemogger walker:    docs/research-glommer/codemogger/src/chunk/treesitter.ts (processNode + splitLargeNode)

Key: S = searchable (FTS), G = graph edges, C = coverage only, A = attach to next entity.

### Rust (root: source_file)

    tree-sitter node type        entity_type    S/G/C/A   notes
    -------------------------    -----------    -------   -----
    function_item                function       S
    function_signature_item      function       S         trait fn signatures
    struct_item                  struct         S
    enum_item                    enum           S
    impl_item                   impl           S         splitNodes -> methods
    trait_item                   trait          S         splitNodes -> methods
    type_item                    type_alias     S
    const_item                   constant       S
    static_item                  static         S
    macro_definition             macro          S
    mod_item                     module         S         splitNodes
    union_item                   struct         S         treat as struct
    foreign_mod_item             module         S         extern blocks
    associated_type              type_alias     S
    macro_invocation             macro          S         top-level macro calls
    use_declaration              import         G         dependency edges
    extern_crate_declaration     import         G         dependency edges
    attribute_item               attribute      A         #[...], attach to next
    inner_attribute_item         attribute      A         #![...], attach to file
    line_comment (///)           doc_comment    C         fold into next entity FTS
    line_comment (//)            comment        C         wc only
    block_comment (/** */)       doc_comment    C         fold into next entity FTS
    block_comment (/* */)        comment        C         wc only
    empty_statement              whitespace     C
    expression_statement         variable       C         rare at top-level
    let_declaration              variable       C         rare at top-level
    shebang                      comment        C

    Comment detection: both /// and // are `line_comment` -- inspect first chars to classify.
    Doc markers: //! and /*! are module-level doc_comments -> fold into file entity.

### Python (root: module)

    tree-sitter node type        entity_type    S/G/C/A   notes
    -------------------------    -----------    -------   -----
    function_definition          function       S
    class_definition             class          S         splitNodes -> methods
    decorated_definition         function/class S         unwrap to inner definition
    type_alias_statement         type_alias     S
    expression_statement         variable       S         top-level X = 5 assignments
    import_statement             import         G
    import_from_statement        import         G
    future_import_statement      import         G
    if_statement                 comment        C         rare at module level
    for_statement                comment        C
    while_statement              comment        C
    try_statement                comment        C
    with_statement               comment        C
    match_statement              comment        C
    assert_statement             comment        C
    pass_statement               comment        C
    return_statement             comment        C
    break_statement              comment        C
    continue_statement           comment        C
    raise_statement              comment        C
    delete_statement             comment        C
    exec_statement               comment        C
    print_statement              comment        C
    global_statement             comment        C
    nonlocal_statement           comment        C
    comment                      comment/doc    C         # vs docstring position

    is_test detection: function name starts with test_ or file in tests/.

### JavaScript (root: program)

    tree-sitter node type            entity_type    S/G/C/A   notes
    -------------------------        -----------    -------   -----
    function_declaration             function       S
    generator_function_declaration   function       S
    class_declaration                class          S         splitNodes -> methods
    lexical_declaration              variable       S         const/let at top level
    variable_declaration             variable       S         var at top level
    export_statement                 (unwrap)       S         unwrap to inner declaration
    expression_statement             variable       S         module.exports = ...
    import_statement                 import         G
    if_statement                     comment        C
    for_statement                    comment        C
    for_in_statement                 comment        C
    while_statement                  comment        C
    do_statement                     comment        C
    switch_statement                 comment        C
    try_statement                    comment        C
    with_statement                   comment        C
    return_statement                 comment        C
    throw_statement                  comment        C
    break_statement                  comment        C
    continue_statement               comment        C
    debugger_statement               comment        C
    labeled_statement                comment        C
    statement_block                  comment        C
    empty_statement                  whitespace     C
    comment (/** */)                 doc_comment    C         JSDoc -> fold into next entity
    comment (//)                     comment        C
    hash_bang_line                   comment        C         #!/usr/bin/env node

    is_test: inside describe()/it()/test() blocks, or file matches *.test.* / *.spec.*.

### TypeScript (root: program)

    Same as JavaScript, plus:

    tree-sitter node type            entity_type    S/G/C/A   notes
    -------------------------        -----------    -------   -----
    interface_declaration            interface      S
    type_alias_declaration           type_alias     S
    enum_declaration                 enum           S
    abstract_class_declaration       class          S         splitNodes
    using_declaration                variable       S

### Java (root: program)

    tree-sitter node type                entity_type    S/G/C/A   notes
    -------------------------            -----------    -------   -----
    class_declaration                    class          S         splitNodes -> methods
    interface_declaration                interface      S         splitNodes
    enum_declaration                     enum           S         splitNodes
    record_declaration                   record         S
    annotation_interface_declaration     interface      S
    import_declaration                   import         G
    package_declaration                  module         S
    block_comment (/** */)               doc_comment    C         Javadoc -> fold into next
    block_comment (/* */)                comment        C
    line_comment                         comment        C

    is_test: @Test annotation on method.

### Go (root: source_file)

    tree-sitter node type        entity_type    S/G/C/A   notes
    -------------------------    -----------    -------   -----
    function_declaration         function       S
    method_declaration           method         S         receiver.Type.Name
    type_declaration             (inspect)      S         contains struct/interface/type_alias
    const_declaration            constant       S
    var_declaration              variable       S
    import_declaration           import         G
    package_clause               module         S
    comment                      comment/doc    C         // before func = doc_comment

    type_declaration unwrapping: inspect child type_spec to determine struct vs interface vs type_alias.
    is_test: function starts with Test in *_test.go files.

### C (root: translation_unit)

    tree-sitter node type        entity_type    S/G/C/A   notes
    -------------------------    -----------    -------   -----
    function_definition          function       S
    declaration                  variable       S         top-level vars, externs
    type_definition              type_alias     S         typedef
    struct_specifier             struct         S
    enum_specifier               enum           S
    union_specifier              struct         S         treat as struct
    preproc_def                  macro          S         #define VALUE
    preproc_function_def         macro          S         #define FUNC(x)
    preproc_include              import         G         #include
    preproc_if                   attribute      C
    preproc_ifdef                attribute      C
    preproc_call                 macro          C
    linkage_specification        module         S         extern "C" { }
    comment                      comment        C

### C++ (root: translation_unit)

    Same as C, plus:

    tree-sitter node type            entity_type    S/G/C/A   notes
    -------------------------        -----------    -------   -----
    class_specifier                  class          S         splitNodes -> methods
    namespace_definition             namespace      S         splitNodes
    template_declaration             (unwrap)       S         unwrap to inner class/fn
    using_declaration                import         G
    namespace_alias_definition       type_alias     S
    concept_definition               trait          S         C++20 concepts ~ traits
    alias_declaration                type_alias     S         using X = Y
    static_assert_declaration        comment        C

### C# (root: compilation_unit)

    tree-sitter node type                entity_type    S/G/C/A   notes
    -------------------------            -----------    -------   -----
    class_declaration                    class          S         splitNodes
    interface_declaration                interface      S
    struct_declaration                   struct         S
    enum_declaration                     enum           S
    record_declaration                   record         S
    namespace_declaration                namespace      S         splitNodes
    file_scoped_namespace_declaration    namespace      S
    delegate_declaration                 type_alias     S
    method_declaration                   method         S
    constructor_declaration              constructor    S
    destructor_declaration               method         S
    property_declaration                 variable       S
    field_declaration                    variable       S
    event_declaration                    variable       S
    event_field_declaration              variable       S
    indexer_declaration                  method         S
    operator_declaration                 method         S
    conversion_operator_declaration      method         S
    using_directive                      import         G
    extern_alias_directive               import         G
    global_attribute                     attribute      A
    comment                              comment        C

    is_test: [Test] or [TestMethod] attribute.

### Ruby (root: program)

    tree-sitter node type        entity_type    S/G/C/A   notes
    -------------------------    -----------    -------   -----
    module                       module         S         splitNodes
    class                        class          S         splitNodes
    method                       function       S
    singleton_method             function       S         self.method
    assignment                   variable       S         top-level CONST = ...
    alias                        type_alias     S
    call                         variable       C         top-level method calls (rare)
    begin_block                  comment        C
    end_block                    comment        C
    undef                        comment        C
    comment                      comment/doc    C         # comment (RDoc before def = doc)

### Scala (root: compilation_unit)

    tree-sitter node type        entity_type    S/G/C/A   notes
    -------------------------    -----------    -------   -----
    class_definition             class          S         splitNodes
    object_definition            object         S         splitNodes
    trait_definition             trait          S         splitNodes
    function_definition          function       S
    function_declaration         function       S
    val_definition               constant       S
    val_declaration              constant       S
    var_definition               variable       S
    var_declaration              variable       S
    type_definition              type_alias     S
    enum_definition              enum           S
    given_definition             impl           S         Scala 3 given ~ Rust impl
    extension_definition         impl           S         Scala 3 extension ~ Rust impl
    import_declaration           import         G
    export_declaration           import         G
    package_clause               module         S
    package_object               module         S
    block_comment                comment        C
    comment                      comment        C         Scaladoc (/** */) = doc_comment

---

## Tree-Sitter Implementation Notes

### 1. Comment Detection Requires Text Inspection

Tree-sitter does NOT distinguish doc comments from plain comments at the node type level.
Both `///` and `//` parse as `line_comment` in Rust. Both `/** */` and `/* */` parse as
`block_comment`. We must inspect the first characters of the comment text:

    Rust:    /// or //! -> doc_comment. // -> comment. /** */ or /*! */ -> doc_comment.
    Python:  Docstrings are expression_statement containing a string, not comment nodes.
    JS/TS:   /** */ -> JSDoc (doc_comment). // and /* */ -> comment.
    Java:    /** */ -> Javadoc (doc_comment). // and /* */ -> comment.
    Go:      // comment immediately before a declaration -> doc_comment (by convention).
    Ruby:    # comment before def/class -> RDoc (doc_comment, by convention).

### 2. Nodes That Need Unwrapping

Some root children wrap the real entity. We unwrap before classifying:

    export_statement (JS/TS)       -> inner is class/function/variable declaration
    decorated_definition (Python)  -> inner is function_definition or class_definition
    template_declaration (C++)     -> inner is class/function/struct
    type_declaration (Go)          -> inner type_spec reveals struct vs interface vs alias

Codemogger implements all four: docs/research-glommer/codemogger/src/chunk/treesitter.ts:36-52,228-280

### 3. splitNodes: One Level Deeper for Large Containers

When an entity exceeds ~150 lines, we split into sub-items (methods within class/impl).
Only one level deep -- never recurse into function bodies.

    Language    Split targets
    --------    -------------
    Rust        impl_item, trait_item, mod_item
    Python      class_definition
    JS          class_declaration
    TS          class_declaration, abstract_class_declaration, interface_declaration
    Java        class_declaration, interface_declaration, enum_declaration
    Go          (none -- Go has flat top-level declarations)
    C           (none)
    C++         class_specifier, struct_specifier, namespace_definition
    C#          class_declaration, interface_declaration, struct_declaration, namespace_declaration
    Ruby        module, class
    Scala       class_definition, object_definition, trait_definition

Body wrapper nodes to walk into: class_body, declaration_list, field_declaration_list,
body_statement, block (varies by language).
Ref: docs/research-glommer/codemogger/src/chunk/treesitter.ts:296-301 (bodyWrappers set)

### 4. Module-Level Only Rule

We only extract entities from root node children (+ one splitNodes level).
Anything nested inside a function body is an implementation detail:

    YES: top-level fn, struct, class, impl, trait, module
    YES: methods inside impl/class (via splitNodes -- one level deep)
    YES: items inside mod tests { } (module-level within test module)
    NO:  closure inside function body
    NO:  fn nested inside another fn
    NO:  struct/enum inside function body
    NO:  block expression items

This matches codemogger's approach: processNode() only walks tree.rootNode.children (line 334),
and splitLargeNode() only goes one level into body wrappers (line 304).
Ref: docs/research-glommer/codemogger/src/chunk/treesitter.ts:228-339

### 5. is_test Detection (per language)

    Rust:      #[test] or #[cfg(test)] attribute on function
    Python:    function name starts with test_ or file in tests/
    JS/TS:     inside describe()/it()/test(), or file matches *.test.* / *.spec.*
    Go:        function starts with Test in *_test.go files
    Java:      @Test annotation
    C#:        [Test] or [TestMethod] attribute
    Ruby:      method inside RSpec describe block, or file in spec/
    Scala:     extends FunSuite/FlatSpec, or method annotated with test

---

## Reindexing Speed (from codemogger benchmarks)

Codemogger benchmarks on Apple M2 (from docs/research-glommer/codemogger/README.md):

    Project         Files     Keyword search    Semantic search    ripgrep
    -------         -----     --------------    ---------------    -------
    Turso (Rust)    748       1 ms              35 ms              25 ms
    Bun (Zig)       9,255     2 ms              137 ms             166 ms
    TypeScript      39,298    4 ms              242 ms             1,500 ms
    Kubernetes (Go) 16,668    12 ms             617 ms             731 ms

Key insight: embedding is 97% of codemogger's indexing time. We skip embedding entirely.

    For Parseltongue (no embedding), single-file reindex estimate:
      SHA-256 hash         ~instant
      Tree-sitter parse    10-20ms
      Delete old entities  <5ms (single SQL DELETE)
      Insert new entities  <5ms (batch INSERT)
      Update FTS           <5ms (incremental)
      --------------------
      Total:               <50ms per changed file

    Full initial index (748-file Rust project like Turso):
      Without embedding:   ~5-15 seconds (tree-sitter only)
      With embedding:      ~60-120 seconds (97% embedding time)

---

# PART IV: THE EVIDENCE

---

## v1.6.1 Retrospective: What We Learned

### Two Approaches Compared

v1.6.1 used declarative `.scm` tree-sitter query files (12 languages, ~15 lines each).
Codemogger uses imperative AST walking with data-driven LanguageConfig (~587 lines total).

v1.6.1 source:  toBeDeleted/archived-code/parseltongue-core/src/query_extractor.rs
v1.6.1 queries: toBeDeleted/archived-code/parseltongue-core/src/entity_queries/*.scm (12 files)
v1.6.1 deps:    toBeDeleted/archived-code/parseltongue-core/src/dependency_queries/*.scm
v1.6.1 keys:    toBeDeleted/archived-code/parseltongue-core/src/isgl1_v2.rs
v1.6.1 types:   toBeDeleted/archived-code/parseltongue-core/src/entities.rs
v1.6.1 storage: toBeDeleted/archived-code/parseltongue-core/src/storage/cozo_client.rs
Codemogger:     docs/research-glommer/codemogger/src/chunk/treesitter.ts (340 lines)
Codemogger cfg: docs/research-glommer/codemogger/src/chunk/languages.ts (247 lines)
Codemogger idx: docs/research-glommer/codemogger/src/index.ts (370 lines)
TS query guide: docs/pre202602/ACTIVE-Reference/AR035-Prep-Tree-Sitter-Query-Patterns.md (68.7KB)

    Approach              v1.6.1 (.scm queries)       Codemogger (imperative walk)
    --------              ---------------------        ----------------------------
    Code per language     ~15 lines .scm + Rust glue  ~50 lines config (shared walker)
    Total code            12 .scm files + glue         587 lines for 14 languages
    splitNodes            NOT implemented              Built-in (>150 lines -> split)
    Comment detection     NOT implemented              Manual text inspection
    Export unwrapping     Nested .scm patterns         Explicit code
    Fuzzy node matching   Exact node type names        type.includes("function")
    Compile-time embed    include_str!() -> zero I/O   N/A (WASM runtime)
    Production tested     Internal only                Shipped by Turso team

### What v1.6.1 Got Right (steal these)

1. **FileWordCoverage schema** -- had source_word_count, entity_word_count, import_word_count,
   comment_word_count, raw_coverage_pct, effective_coverage_pct. Validates our wc model.
   Ref: toBeDeleted/archived-code/parseltongue-core/src/storage/cozo_client.rs
2. **8 dependency edge types** -- calls, uses, implements, type_refs, field_access,
   async_await, iterators, generics.
   Ref: toBeDeleted/archived-code/parseltongue-core/src/dependency_queries/rust.scm (180 lines)
3. **Deduplication** -- HashSet<(name, line_range)> to handle overlapping query matches.
   Ref: toBeDeleted/archived-code/parseltongue-core/src/query_extractor.rs:407-416
4. **include_str!() embedding** -- compile-time config embedding, zero runtime I/O.
   Ref: toBeDeleted/archived-code/parseltongue-core/src/query_extractor.rs (include_str! calls)
5. **Graph algorithms** -- 7 well-tested algorithms (2,812 lines) portable to v3.0.
   Ref: toBeDeleted/archived-code/parseltongue-core/src/graph_analysis/ (11 files)

### What v1.6.1 Got Wrong (avoid these)

1. **Key format** -- rust:fn:name:__path:T170... breaks on renames. Our path:line:line is stable.
2. **No splitNodes** -- large impl blocks became single giant entities.
3. **No doc comment handling** -- known gap, never addressed.
4. **CozoDB underutilized** -- stored data but didn't use graph engine.
5. **.scm queries are fragile** -- grammar updates break exact node type patterns.

---

## Embedding Research (2026-03-17)

Two independent research agents investigated whether Parseltongue should add embeddings.
Both reached the same conclusion: **stay CPU-only**.

### Key findings

1. **Sourcegraph removed embeddings from Cody Enterprise** (v7.0). Went back to BM25F keyword search. Cited security, scalability, and operational complexity.
2. **Codemogger's own data**: embedding is 97% of indexing time. Keyword search: 1-12ms. Semantic: 35-617ms.
3. **Claude Code uses only lexical search** and "achieves strong results."
4. **Aider (AST+PageRank, no embeddings)** had lowest token usage among all coding agents (Oct 2025 paper).
5. **GitHub Code Search**: billions of files, trigram index, no embeddings.
6. **Open-source code embedding models are poor**: CodeBERT MRR=0.117. Good models (Voyage Code-3) are API-only.
7. **Parseltongue's graph compensates**: BFS anchoring + ego clustering navigate from imprecise search hits to the right neighborhood. Embeddings can't do this.

### If embeddings are ever needed (v3.1+)

Model2Vec static embeddings via `model2vec-rs` crate:
- Pure Rust, no ONNX/C++ dependency
- <0.05ms per embedding (vs MiniLM's 15-25ms)
- ~30MB model, 256 dimensions
- Turso/libSQL has native vector_distance_cos() -- zero schema changes needed
- Add nullable `embedding F32_BLOB(256)` column now (costs nothing if NULL)

Sources: Sourcegraph blog (BM25F), GitHub blog (Blackbird), Continue.dev docs, Aider paper (Oct 2025), Anthropic (contextual retrieval).

---

## Decisions

### D1: Tauri App is Priority One
- Mac-first. Workspace management. File picker, ingestion status, settings, logs.
- Tauri does NOT do: graph algorithms, compiler analysis, database ops, search logic.
- Queries happen over HTTP. Tauri manages the lifecycle.

### D2: Rust Gets rustc_private Enrichment
- Pin nightly toolchain. Extract: resolved types, real call graphs, trait impls, visibility, MIR.
- Proven by: Miri, Flowistry, Aquascope, Prusti, Kani, Rudra.
- Rationale: docs/v300/rustc_private_stability_rationale_202603091530.md

### D3: Other Languages Get Basic Tree-Sitter Only
- Entity extraction + basic edges. No deep analysis.

### D4: HTTP-Only for LLM Integration
- Ship HTTP REST. MCP can be a thin wrapper added later.

### D5: Algorithm Breadth is Minimal
- Only what the 7-event journey needs: RRF, BFS, ego network, deep dive.
- Plus 7 ported graph algorithms from v1.6.1 (SCC, k-core, PageRank, Leiden, entropy, CK, SQALE).

### D6: Audience is Both Humans and LLMs
- OSS contributors + LLM coding agents. Same journey for both.

### D7: Database is Turso/libSQL
- Replacing CozoDB. Single file. FTS5 built-in.
- Full DDL: see Database Schema section in Part III.
- Codemogger schema ref: docs/research-glommer/codemogger/src/db/schema.ts
- Codemogger store ref: docs/research-glommer/codemogger/src/db/store.ts
- Memelord Turso warning: docs/research-glommer/memelord/ (file locking -- use short-lived connections)
- Cachebro FRESH/STALE: docs/research-glommer/cachebro/ (content-addressed file versioning)

### D8: Primary Key is Physical Location
- file_path + optional start_line:end_line. ISG_L1_V3 is derived, not identity.

### D9: Entity Taxonomy is 30 Types with `entity_type` Column (2026-03-17)
- Grounded in: codemogger (docs/research-glommer/codemogger/src/chunk/languages.ts)
- Grounded in: cargo cache node-types.json (see Tree-Sitter API Reference section above)
- v1.6.1 entity types: toBeDeleted/archived-code/parseltongue-core/src/entities.rs
- Every entity has: pk, entity_type, wc (word count).
- 4 structural (folder, file_parsable, file_unparsable, file_config)
- 18 searchable code entities (function, method, struct, class, enum, trait, interface, impl,
  type_alias, constant, static, macro, module, variable, constructor, namespace, record, object)
  + is_test flag on function/method
- 5 non-code entities (import, doc_comment, comment, attribute, whitespace) for 100% coverage
- 3 Rust config spans (dependency, package_meta, config_section)
- Doc comments (///, //!, /** */) folded into adjacent code entity's `doc_comment` FTS field.
- Module doc comments (//!) folded into file_parsable entity's `doc_comment` field.
- Plain comments (//), whitespace counted for coverage only.
- Imports drive dependency graph edges. Attributes attach to next code entity.
- Module-level only: nested items are part of parent entity's snippet, not separate entities.
- Only code entities are FTS-searchable. Everything else is graph/coverage only.

### D10: Coverage via Word Count (2026-03-17, grounded in apache/iggy)
- Validated by v1.6.1's FileWordCoverage: toBeDeleted/archived-code/parseltongue-core/src/storage/cozo_client.rs
- apache/iggy sample: docs/research-glommer/iggy-sample/ (gitignored)
- Every entity stores `wc` (word count). File stores `total_wc`.
- For parsable files: sum(entity.wc) = file.total_wc. Verified on save. Zero gaps.
- Coverage computable at file, folder, and repo level via SQL GROUP BY entity_type.
- Token economics derived from wc: tokens ~ wc * 1.3.
- Expected breakdown: ~65% searchable code, ~10% doc comments (also searchable), ~25% overhead.
- Lock files, binaries, generated files are file entities with total_wc but no child entities.
- .svelte could be added if tree-sitter-svelte grammar is included (adds 70 files in iggy).

### D11: Data-Driven Tree-Sitter Walker (2026-03-17, codemogger-validated)
- Codemogger walker: docs/research-glommer/codemogger/src/chunk/treesitter.ts
- Codemogger config: docs/research-glommer/codemogger/src/chunk/languages.ts
- v1.6.1 .scm files: toBeDeleted/archived-code/parseltongue-core/src/entity_queries/*.scm
- v1.6.1 query engine: toBeDeleted/archived-code/parseltongue-core/src/query_extractor.rs
- Architecture decisions: docs/v300/minimal_v200_architecture_decisions_202603091545.md
- Follow codemogger's imperative AST walking, not v1.6.1's .scm query files.
- LanguageConfig struct: name, extensions, top_level_nodes, split_nodes (const, compile-time).
- Shared walker for all languages. Classify via node.kind() -> entity_type.
- splitNodes for large containers (>150 lines -> extract methods).
- Comment detection via text inspection (/// vs // are same node type).
- Export/decorator/template unwrapping in shared code.
- v1.6.1's FileWordCoverage schema validates our wc model.
- v1.6.1's 8 dependency edge types are the right set to extract.
- Reindexing: <50ms per changed file (no embedding bottleneck).

### D12: CPU-Only Guarantee (2026-03-17, validated by two independent research agents)
- No GPU. No embedding model. No LLM in the middle.
- Symbol trie lookup, trigram index, graph traversal, rustc type info.
- Full transparency: logs show exactly why each result ranked.
- Sourcegraph removed embeddings from Cody Enterprise. Claude Code uses lexical search only.
- Parseltongue's graph (BFS anchoring + ego clustering) compensates for FTS semantic gap.
- Upgrade path: Model2Vec static embeddings via model2vec-rs if ever needed (v3.1+).
- Architecture-ready: nullable embedding column in schema costs nothing if NULL.

---

## Big Rocks

- Big-Rock-01: the scope and dependencies
    - language Rust 21
    - treesitter for C, C++, Javascript, Typescript, Python, Java, Go, Ruby, Scala, PHP, C#, Swift, Kotlin
    - rustcompiler enrichment for Rust code

- Big-Rock-02: the primary-key and entity_type
    - Uniform PK: `path:start_line:end_line` -- see Entity Taxonomy above
    - Sentinels: -1:-1 = folder, 0:0 = file, N:M = code span
    - ISG_L1_V3 (language|||kind|||scope|||name|||file_path|||discriminator) is DERIVED, not the key
    - Every entity has: pk, entity_type, wc (word count)
    - 30 entity_types for 100% file coverage -- see Entity Taxonomy above
    - Module-level only: nested items (closures, inner fns) are part of parent snippet
    - Validated by codemogger (docs/research-glommer/codemogger/src/chunk/types.ts -- same `file:line:line` chunk key)

- Big-Rock-03: code-graph-building
    - Graph DB thesis: docs/parseltongue-code-graph-db/
    - File walker ref: docs/research-glommer/codemogger/src/scan/walker.ts
    - .gitignore-driven walk (simplified: directory names only, no globs)
    - Hardcoded ALWAYS_IGNORE: .git, node_modules, target, build, dist, __pycache__, .venv, .cargo, .rustup
    - SHA-256 hash per file for incremental indexing (skip unchanged files on re-analyze)
    - Folder -> folder edges (parent/child)
    - File -> folder edges (belongs_to)
    - Code span -> file edges (part_of)
    - Code span -> code span edges (calls, imports, implements -- from tree-sitter + rustc)
    - Rust files (.rs) -> Layer 2 (tree-sitter) + Layer 3 (rustc_private enrichment)
    - Rust config (Cargo.toml) -> parsed as TOML, yields dependency/package_meta/config_section entities
    - Other parsable languages (py, js, ts, go, java, c, cpp) -> Layer 2 only
    - Unparsable files -> Layer 1 only (just the address + hash)
    - Tests -> same entities, flagged with is_test=true
