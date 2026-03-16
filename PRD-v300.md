# Parseltongue: Rust LLM Companion

> Grep returns files. Parseltongue returns understanding.
> A minimalistic, well-verified proof of Rust craft.

# Key Ideas

- Types of entities
    - folder is an entity
    - sub-folders are an entity with edge connections to upper folders and lower folders
    - 



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
    - PRIMARY KEY = physical location: file_path (+ start_line:end_line for code spans)
    - ISG_L1_V3 (language|||kind|||scope|||name|||file_path|||discriminator) is a DERIVED COLUMN, not the key
    - The key is the address. Everything else is metadata about that address.

    Entity hierarchy (everything is a physical location first):

    Layer 0: FOLDER
        src/auth/                              pk = "src/auth/"
        - every folder is an entity
        - subfolders connect to parent via edge

    Layer 1: FILE
        src/auth/service.rs                    pk = "src/auth/service.rs"
        - every file is an entity, connects to its folder via edge
        - file is either parsable or not parsable
        - unparsable file: just the address, that's the entity

    Layer 2: LINE RANGE (within a parsable file)
        src/auth/service.rs:1:3     (comments)     pk = "src/auth/service.rs:1:3"
        src/auth/service.rs:4:6     (imports)      pk = "src/auth/service.rs:4:6"
        src/auth/service.rs:8:25    (fn login)     pk = "src/auth/service.rs:8:25"
        src/auth/service.rs:27:40   (struct Auth)  pk = "src/auth/service.rs:27:40"

        Classifications for parsable line ranges:
        - comment
        - blank (not an entity)
        - import statement
        - statement (assignment, expression)
        - function / method
        - type definition (struct, class, enum)
        - trait / interface
        - impl block (Rust-specific)

        All classifications work for any language via tree-sitter.

    Layer 3: RUST COMPILER ENRICHMENT (extra columns, only for Rust files)
        src/auth/service.rs:8:25 gets additional columns:
        - rustc_scope:   "crate::auth::service::login"       from tcx.def_path_str()
        - rustc_sig:     "fn(&Credentials) -> Result<Token>"  from tcx.fn_sig()
        - visibility:    "pub(crate)"                          from tcx.visibility()
        - mir_calls:     ["crate::db::lookup", ...]            from tcx.optimized_mir()
        - trait_impls:   [...]                                 from tcx.all_impls()

        Rust enrichment is ADDITIVE. Same table, same primary key.
        Rust rows just have more columns filled in.

    The ISG_L1_V3 rich name:
        language|||kind|||scope|||name|||file_path|||discriminator
        rust|||fn|||auth::service|||login|||src/auth/service.rs|||sig_v3

        This is DERIVED from the layers above. Useful for display and search.
        NOT the primary key. The entity IS its location.

- Big-Rock-03: code-graph-building
    - parse folder names
    - folders become entities of type folder, connected via edges
    - subfolders connect to parent folders
    - files connect to their containing folder
    - rust-ecosystem files
        - rust code (.rs) → parsable, gets Layer 2 + Layer 3
        - rust config (Cargo.toml) → parsable as TOML, not as code
        - rust tests → parsable, marked with is_test metadata
    - non-rust files
        - parsable languages (py, js, ts, go, java, c, cpp) → Layer 2 only
        - unparsable files (README.md, .env, images) → Layer 1 only (just the file path)

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

---

# CPU-Only Guarantee

No GPU. No embedding model. No LLM in the middle.
Symbol trie lookup, trigram index, graph traversal, rustc type info.
Full transparency: logs show exactly why each result ranked.
