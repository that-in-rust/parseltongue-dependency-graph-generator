# Parseltongue: Rust LLM Companion

> Grep returns files. Parseltongue returns understanding.
> A minimalistic, well-verified proof of Rust craft.

# Primary Key Format

Uniform for ALL entities: `path:start_line:end_line`

    src/auth/:-1:-1                    → folder
    src/auth/service.rs:0:0            → file
    src/auth/service.rs:8:25           → code span (fn login)

Sentinels: `-1:-1` = folder, `0:0` = file, `N:M` (N >= 1) = code span.

# Searchability Rule

- SEARCHABLE: Only code spans (N:M) go into FTS. They have name + signature + snippet.
- NOT SEARCHABLE: Folders and files are graph-only. Connectivity + staleness checks only.
- NOT STORED: Full file content is NEVER stored. Only parsed snippets.

# Entity Taxonomy (27 kinds)

## A. Structural Entities (graph-only, not searchable, no content stored)

    folder            src/auth/:-1:-1             Every directory in the tree
    file_parsable     src/auth/service.rs:0:0     Tree-sitter can parse it. Has child code spans. Stores file_hash only.
    file_unparsable   README.md:0:0               Can't parse. Just an address. Stores file_hash only.
    file_config       Cargo.toml:0:0              Rust config only. Parsed as TOML, not code. Other langs' configs = file_unparsable.

## B. Code Span Entities (searchable, extracted by tree-sitter)

Each is a contiguous line range within a parsable file. FTS indexes name + signature.

    Kind          Example PK                     Languages
    ----          ----------                     ---------
    function      src/main.rs:10:25              All
    method        src/auth.rs:30:45              All with classes/impls
    struct        src/model.rs:5:15              Rust, C, C++, Go
    class         src/app.py:1:50                Python, JS/TS, Java, C++, Ruby, PHP, C#
    enum          src/status.rs:3:12             Rust, Java, TS, C, C++, C#
    trait         src/auth.rs:1:20               Rust
    interface     src/api.ts:5:30                TS, Java, Go, C#, PHP
    impl          src/auth.rs:22:60              Rust
    type_alias    src/types.rs:3:3               Rust, TS, Go, C
    constant      src/config.rs:1:1              All
    static        src/global.rs:5:5              Rust, C, C++
    macro         src/macros.rs:1:20             Rust, C, C++
    module        src/lib.rs:1:1                 Rust, Python, Ruby, JS/TS
    import        src/main.rs:1:3                All (drives dependency edges)
    variable      src/app.js:1:1                 JS/TS, Python, Go
    constructor   src/App.java:10:20             Java, C++, TS, PHP, C#
    namespace     src/lib.cpp:1:50               C++, C#, PHP
    record        src/User.java:1:10             Java, C#
    object        src/App.scala:1:20             Scala

Comments are NOT entities. Blanks are NOT entities.
Tests are not a separate kind — `is_test=true` flag on function/method entities.

## C. Rust Config Span Entities (Cargo.toml only)

    dependency      Cargo.toml:5:5               A crate dependency declaration
    package_meta    Cargo.toml:1:4               Package name, version, edition
    config_section  Cargo.toml:10:15             Named section ([features], [workspace], etc.)

## D. Rust Compiler Enrichment (Layer 3 — extra columns, not new entities)

For .rs code spans, same pk, same row, more columns filled in:

    rustc_scope     tcx.def_path_str()     "crate::auth::service::login"
    rustc_sig       tcx.fn_sig()           "fn(&Credentials) -> Result<Token>"
    visibility      tcx.visibility()       "pub(crate)"
    mir_calls       tcx.optimized_mir()    ["crate::db::lookup", ...]
    trait_impls     tcx.all_impls()        [...]

---

# The Screens

Everything follows from the screens. The screens ARE the product.

---

## Screen 1: First Launch (Empty State)

User downloads .dmg or `brew install parseltongue`. Opens the app.
No login. No account. No server. Privacy-first.

```
┌──────────────────────────────────────────┐
│  Parseltongue                        [—] │
│──────────────────────────────────────────│
│                                          │
│  No codebases yet.                       │
│                                          │
│  [ Browse Folder ]  or drag & drop here  │
│                                          │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│  Parseltongue analyzes Rust codebases    │
│  so you (and your LLM) can understand    │
│  them without reading every file.        │
│                                          │
└──────────────────────────────────────────┘
```

User clicks Browse Folder. Native macOS file dialog opens.
They pick ~/code/my-rust-project.

**What happens behind the screen:**
- Tauri native file dialog (already researched in docs/tauri-research)
- No database exists yet. Nothing to configure.

---

## Screen 2: Ingestion Progress

```
┌──────────────────────────────────────────┐
│  Parseltongue                        [—] │
│──────────────────────────────────────────│
│                                          │
│  Analyzing: my-rust-project              │
│  /Users/dev/code/my-rust-project         │
│                                          │
│  [████████░░░░░░░░░░] 42%               │
│                                          │
│  Tree-sitter parsing...     142 files    │
│  Rust compiler analysis...  in progress  │
│  Building graph...          pending      │
│                                          │
│  Found so far:                           │
│    387 entities  ·  1,204 edges          │
│                                          │
└──────────────────────────────────────────┘
```

Two passes:
1. Fast pass (tree-sitter): entities + basic edges for all languages. Seconds.
2. Deep pass (rustc_private): compiler-verified types, real call graph, trait impls. Rust only.

macOS notification when done: "my-rust-project analyzed. 387 entities, 1,204 edges."

**What happens behind the screen:**
- Walk folder tree → create folder entities (Layer 0) and file entities (Layer 1)
- For each parsable file → tree-sitter extracts line-range entities (Layer 2)
- For .rs files → rustc_private adds compiler truth (Layer 3)
- All stored in Turso/libSQL database managed by the app
- Database location is invisible to the user

---

## Screen 3: Home Screen (With Data)

```
┌──────────────────────────────────────────┐
│  Parseltongue                        [—] │
│──────────────────────────────────────────│
│                                          │
│  YOUR CODEBASES                          │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │ my-rust-project           FRESH    │  │
│  │ ~/code/my-rust-project             │  │
│  │ 387 entities · 1,204 edges         │  │
│  │ Last analyzed: 2 min ago           │  │
│  │                                    │  │
│  │ HTTP: localhost:7777  [ Copy URL ] │  │
│  │ [ Re-analyze ]  [ Open Terminal ]  │  │
│  └────────────────────────────────────┘  │
│                                          │
│  [ + Add Another Codebase ]              │
│                                          │
└──────────────────────────────────────────┘
```

- FRESH / STALE badge — compares DB timestamp to git HEAD or file mod times
- Copy URL — one click to get the HTTP endpoint for LLM tools
- DB location is invisible — Tauri manages it
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

**Event 1: QUERY** — the ~7 words arrive

**Event 2: SEARCH** — RRF fusion finds 4 candidates (<10ms)
  - Symbol trie (exact matches)
  - Trigram index (fuzzy matches)
  - Git history (recent edits)

**Event 3: ANCHOR** — BFS upward to public API boundary (<50ms)
  - For private entities: walk callers until a public fn/trait is found
  - For public entities: anchor is itself

**Event 4: CLUSTER** — ego network 1-hop for each anchor (<100ms)
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
  "callers": [ ... ],
  "callees": [ ... ],
  "type_signatures": { ... },
  "control_flow": { ... },
  "git_history": [ ... ],
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
┌──────────────────────────────────────────┐
│  Parseltongue                        [—] │
│──────────────────────────────────────────│
│                                          │
│  YOUR CODEBASES                          │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │ my-rust-project           STALE    │  │
│  │ ~/code/my-rust-project             │  │
│  │ 387 entities · 1,204 edges         │  │
│  │ Last analyzed: yesterday           │  │
│  │ 3 commits behind                   │  │
│  │                                    │  │
│  │ HTTP: stopped     [ Start Server ] │  │
│  │ [ Re-analyze ]  [ Open Terminal ]  │  │
│  └────────────────────────────────────┘  │
│                                          │
└──────────────────────────────────────────┘
```

One click to re-analyze. Server can restart with old data while re-ingestion runs.

---

# Token Economics

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

# Big Rocks

- Big-Rock-01: the scope and dependencies
    - language Rust 21
    - treesitter for
        - C C++ Javascript Typescript Python Java Go
    - rustcompiler enrichment for Rust code

- Big-Rock-02: the primary-key
    - Uniform PK: `path:start_line:end_line` — see Entity Taxonomy above
    - Sentinels: -1:-1 = folder, 0:0 = file, N:M = code span
    - ISG_L1_V3 (language|||kind|||scope|||name|||file_path|||discriminator) is DERIVED, not the key
    - 27 entity kinds across 4 layers — see Entity Taxonomy above
    - Validated by codemogger (uses same `file:line:line` chunk key)

- Big-Rock-03: code-graph-building
    - .gitignore-driven walk (simplified: directory names only, no globs)
    - Hardcoded ALWAYS_IGNORE: .git, node_modules, target, build, dist, __pycache__, .venv, .cargo, .rustup
    - SHA-256 hash per file for incremental indexing (skip unchanged files on re-analyze)
    - Folder → folder edges (parent/child)
    - File → folder edges (belongs_to)
    - Code span → file edges (part_of)
    - Code span → code span edges (calls, imports, implements — from tree-sitter + rustc)
    - Rust files (.rs) → Layer 2 (tree-sitter) + Layer 3 (rustc_private enrichment)
    - Rust config (Cargo.toml) → parsed as TOML, yields dependency/package_meta/config_section entities
    - Other parsable languages (py, js, ts, go, java, c, cpp) → Layer 2 only
    - Unparsable files → Layer 1 only (just the address + hash)
    - Tests → same entities, flagged with is_test=true

---

# Decisions (2026-03-16 brainstorm session)

## D1: Tauri App is Priority One
- Mac-first. Workspace management. File picker, ingestion status, settings, logs.
- Tauri does NOT do: graph algorithms, compiler analysis, database ops, search logic.
- Queries happen over HTTP. Tauri manages the lifecycle.

## D2: Rust Gets rustc_private Enrichment
- Pin nightly toolchain. Extract: resolved types, real call graphs, trait impls, visibility, MIR.
- Proven by: Miri, Flowistry, Aquascope, Prusti, Kani, Rudra.

## D3: Other Languages Get Basic Tree-Sitter Only
- Entity extraction + basic edges. No deep analysis.

## D4: HTTP-Only for LLM Integration
- Ship HTTP REST. MCP can be a thin wrapper added later.

## D5: Algorithm Breadth is Minimal
- Only what the 7-event journey needs: RRF, BFS, ego network, deep dive.

## D6: Audience is Both Humans and LLMs
- OSS contributors + LLM coding agents. Same journey for both.

## D7: Database is Turso/libSQL
- Replacing CozoDB. Single file. FTS5 built-in.

## D8: Primary Key is Physical Location
- file_path + optional start_line:end_line. ISG_L1_V3 is derived, not identity.

## D9: Entity Taxonomy is 27 Kinds (2026-03-16)
- 4 structural (folder, file_parsable, file_unparsable, file_config)
- 19 code spans (function, method, struct, class, enum, trait, interface, impl, type_alias, constant, static, macro, module, import, variable, constructor, namespace, record, object) + is_test flag
- 3 Rust config spans (dependency, package_meta, config_section)
- Comments are NOT entities. Blanks are NOT entities.
- Imports ARE entities (drive dependency edges).
- Rust config (Cargo.toml) IS parsed. Other languages' configs are file_unparsable.
- Only code spans are searchable (FTS). Folders and files are graph-only.

---

# CPU-Only Guarantee

No GPU. No embedding model. No LLM in the middle.
Symbol trie lookup, trigram index, graph traversal, rustc type info.
Full transparency: logs show exactly why each result ranked.
