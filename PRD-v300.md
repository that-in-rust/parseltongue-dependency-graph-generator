# User Segment x Diferentiation

- Rust Open Source Library
    - Maintainers
    - Contributors

# This will be a minimalistic well verified proof of my Rust craft




# Parseltongue: Replace Grep for Code

> **Grep returns files. Parseltongue returns understanding.**

| Tool | Returns |
|------|---------|
| grep | Lines matching pattern |
| IDE search | Files containing text |
| Embedding search | Similar code chunks (guessed) |
| **Parseltongue** | Compiler-verified clusters + deep context |

---

# The 7-Event User Journey

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 1: QUERY                                                             │
│                                                                             │
│  LLM sends a short query (~7 words)                                         │
│                                                                             │
│  Example: "authentication flow in this codebase"                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 2: SEARCH                                                            │
│                                                                             │
│  Parseltongue finds 4 candidate entities using RRF fusion                   │
│                                                                             │
│  Retrievers:                                                                │
│    - Symbol trie (exact matches)                                            │
│    - Trigram index (fuzzy matches)                                          │
│    - Git history (recent edits)                                             │
│                                                                             │
│  Output: [auth::login, AuthProvider, authentication module, oauth]          │
│                                                                             │
│  Token cost: ~30 tokens                                                     │
│  Time: <10ms (pure CPU)                                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 3: ANCHOR                                                            │
│                                                                             │
│  For each candidate, find the public API boundary                           │
│                                                                             │
│  Algorithm: BFS upward until public function/trait found                    │
│                                                                             │
│  For auth::login (private):                                                 │
│    auth::login → auth::session::create → api::handlers::login_route (PUB)  │
│                                                                             │
│  For AuthProvider (public trait):                                           │
│    Already public → anchor is itself                                        │
│                                                                             │
│  Output: Public interface + module path + immediate neighbors               │
│                                                                             │
│  Token cost: ~100 tokens                                                    │
│  Time: <50ms (graph traversal in CozoDB)                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 4: CLUSTER                                                           │
│                                                                             │
│  Build ego network (distance=1) for each anchored entity                    │
│                                                                             │
│  Cluster = anchor + callers + callees + implementations                     │
│                                                                             │
│  Each cluster compressed to max 3000 tokens                                 │
│                                                                             │
│  Token cost: ~3000 tokens per cluster (4 clusters = 12000 tokens internal)  │
│  Time: <100ms                                                               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 5: ASK                                                               │
│                                                                             │
│  Present 4 candidate-info-clusters to LLM                                   │
│                                                                             │
│  "I found 4 clusters for 'authentication':                                  │
│                                                                             │
│   [1] API HANDLER - login_route (HTTP endpoint, calls auth::login)          │
│   [2] AUTH TRAIT - AuthProvider (abstraction, 2 impls: JWT, OAuth)          │
│   [3] MODULE - authentication (folder with 12 files)                        │
│   [4] EXTERNAL - oauth (third-party integration)                            │
│                                                                             │
│   Which cluster? [1] [2] [3] [4] [none]"                                    │
│                                                                             │
│  Token cost: ~200 tokens for LLM to read                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
                        LLM chooses: [1]
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 6: CHOICE                                                            │
│                                                                             │
│  LLM responds with:                                                         │
│    - A number [1-4] → proceed to deep dive                                  │
│    - "none" → no relevant cluster, try different query                      │
│    - Quit → end session                                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    ↓
                        If [1-4] chosen:
                                    ↓
┌─────────────────────────────────────────────────────────────────────────────┐
│  EVENT 7: DEEP DIVE                                                         │
│                                                                             │
│  Return full context for chosen cluster (up to 20k tokens)                  │
│                                                                             │
│  Includes:                                                                  │
│    - Complete code for anchor + ego network                                 │
│    - Control flow graph (branching, loops, error paths)                     │
│    - Data flow (where data comes from, where it goes)                       │
│    - Type signatures (compiler-verified)                                    │
│    - Git history (what changes with this code)                              │
│                                                                             │
│  Plus, suggest non-traditional queries for next step:                       │
│    - blast_radius() → who will break if I change this?                      │
│    - complexity() → how hard is this code?                                  │
│    - test_coverage() → what's untested?                                     │
│    - type_flow() → trace data from input to output                          │
│    - call_slice() → minimal executable path                                 │
│                                                                             │
│  Token cost: Up to 20,000 tokens                                            │
│  Time: <500ms (all pre-computed via rust-analyzer)                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Why This Wins

### For LLMs

| Need | How Parseltongue Helps |
|------|------------------------|
| Fast | All CPU, milliseconds not seconds |
| Accurate | Compiler-verified, zero hallucination |
| Efficient | Only pay tokens for what you choose |
| Transparent | Logs show exactly why results bubbled up |
| Flexible | Can always ask for more depth |

### For Humans

| Need | How Parseltongue Helps |
|------|------------------------|
| Simple | Single endpoint, no upfront questions |
| Trust | See the reasoning, not a black box |
| Control | You pick the cluster, not the system |
| Iterative | Drill down step by step |

---

## Token Economics

| Stage | Tokens (Internal) | Tokens (to LLM) |
|-------|-------------------|-----------------|
| Event 1: Query | 0 | 7 words |
| Event 2: Search | 30 | - |
| Event 3: Anchor | 100 | - |
| Event 4: Cluster | 12,000 | - |
| Event 5: Ask | - | 200 |
| Event 7: Deep Dive | - | Up to 20,000 |

**Key insight**: LLM only sees ~200 tokens before making choice, then pays 20k for deep dive on ONE cluster (not 80k for all 4).

---

## CPU-Only Guarantee

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   NO GPU                                                        │
│   NO EMBEDDING MODEL                                            │
│   NO LLM IN THE MIDDLE                                          │
│                                                                 │
│   Everything is:                                                │
│     - Symbol trie lookup (O(k) where k = query length)          │
│     - Trigram index scan (O(n) but highly optimized)            │
│     - Graph traversal in CozoDB (Datalog, compiled queries)     │
│     - rust-analyzer type information (pre-computed)             │
│                                                                 │
│   Transparency: Full logs of why each result ranked             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

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

It is extremely difficult to track which codebase was ingested where and when.
The Tauri app (Mac-first) is the single place to manage this.

- Tauri does: workspace management, file picker, ingestion status, settings, logs
- Tauri does NOT do: graph algorithms, compiler analysis, database ops, search logic
- Queries happen over HTTP. Tauri manages the lifecycle.

## D2: Rust Gets rustc_private Enrichment

Tree-sitter gives syntax. The Rust compiler gives truth.

- Pin nightly toolchain in rust-toolchain.toml
- Extract: resolved types, real call graphs, trait impls, visibility, MIR
- Core APIs (type_of, fn_sig, visibility) have 0% churn rate
- Proven by: Miri, Flowistry, Aquascope, Prusti, Kani, Rudra

## D3: Other Languages Get Basic Tree-Sitter Only

- Entity extraction (functions, structs, classes, traits)
- Basic edge detection (calls, imports, uses)
- No deep analysis, no compiler integration
- Keep what already works from v1.6 tree-sitter grammars

## D4: HTTP-Only for LLM Integration

Research (March 2026):
- MCP has massive adoption (97M downloads) but real production pain (auth, security, context bloat)
- Perplexity CTO publicly moved away from MCP toward plain APIs/CLIs
- HTTP is the universal foundation that never dies

Decision: Ship HTTP REST. MCP can be a thin wrapper added later.
It's a distribution decision, not an architecture decision.

## D5: Algorithm Breadth is Minimal

Only what the 7-event journey needs:
- RRF fusion (Event 2)
- BFS upward (Event 3)
- Ego network 1-hop (Event 4)
- Deep dive support (Event 7)

Pre-computed algorithms (PageRank, Leiden, k-core, etc.) are NOT required for v3 MVP.

## D6: Audience is Both Humans and LLMs

- OSS contributors landing on a Rust repo for the first time
- LLM coding agents (Claude Code, Cursor, Copilot) needing structured context
- The 7-event journey works identically for both

---

# Open Questions (Must Resolve Before Building)

## OQ1: What Data Do We Store vs. Point To?

This is the biggest unresolved question. What goes in the database vs. what gets read live from source files?

## OQ2: Database Choice

Prior v300 docs decided libSQL/SQLite (replacing CozoDB). Still the call?

## OQ3: Tauri App Concrete Design

What's on the home screen? How does ingestion work in the UI?

## OQ4: Crate Architecture for v3

How do we restructure crates for Tauri + rustc_private + 7-event journey?

## OQ5: Testing Strategy

How do we test rustc_private? Validate algorithms? Acceptance tests for the journey?

## OQ6: ISG_L1_V3 Key Format Validation

Does the primary key format work for both rustc_private and tree-sitter languages?


 ---
  The Real User Journey: What Actually Happens                                                                                                                                                                    
                                                                                                                                                                                                                
  Moment 0: Discovery

  Someone sees "Parseltongue - Rust LLM Companion" on GitHub/crates.io. They think: "I want to understand this Rust codebase I'm contributing to."

  They install:
  brew install parseltongue
  Or download the .dmg from GitHub releases.

  Moment 1: First Launch

  The Tauri app opens. No login. No account. No server. Privacy-first (your research already nails this).

  The home screen is essentially empty:

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

  The native macOS file dialog opens (your research has this pattern ready). They pick ~/code/my-rust-project.

  Moment 2: Ingestion

  A progress view appears. This is where Parseltongue does its work:

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

  Two passes:
  1. Fast pass (tree-sitter): entities + basic edges for all languages — seconds
  2. Deep pass (rustc_private): compiler-verified types, real call graph, trait impls for Rust — longer but the value-add

  macOS notification when done: "my-rust-project analyzed. 387 entities, 1,204 edges."

  Moment 3: The Home Screen (With Data)

  Now the app shows what it knows:

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

  Key details:
  - FRESH / STALE badge — compares DB timestamp to git HEAD or file mod times
  - Copy URL — one click to get the HTTP endpoint for your LLM tool
  - The DB location is invisible — Tauri manages it. User never thinks about parseltongue20251201/analysis.db

  Moment 4: Using It (The 7-Event Journey)

  The user copies localhost:7777 into Claude Code / Cursor / their terminal. Then:

  curl "http://localhost:7777/query?q=authentication+flow"

  And the 7-event journey runs. Search → Anchor → Cluster → present 4 options → choose → deep dive.

  Or an LLM agent calls it programmatically. Same endpoints, same journey.

  Moment 5: Coming Back Tomorrow

  They open Parseltongue again. The home screen shows:

    my-rust-project           STALE (3 commits behind)
    [ Re-analyze ]

  One click to re-ingest. Or it could auto-detect changes via file watching.

  ---
  What This Tells Us About the Open Questions

  This journey sketch actually answers some of the open questions:

  ┌──────────────────────────┬─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
  │      Open Question       │                                                                      What the Journey Implies                                                                       │
  ├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ OQ1: What data to store? │ The DB needs to be self-contained enough that the HTTP server works without re-parsing. But "STALE" detection means we track file hashes and compare to live files. │
  ├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ OQ3: Tauri design        │ It's a workspace list. Each workspace = one codebase + its DB + its HTTP server instance.                                                                           │
  ├──────────────────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
  │ OQ4: Crate architecture  │ Tauri app needs to spawn/manage HTTP server processes per workspace.                                                                                                │
  └──────────────────────────┴─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘              
                
⏺ User answered Claude's questions:      
  ⎿  · Does this user journey feel right to you? What would you change about how someone first experiences Parseltongue? → This feels right
                                                                                                                                                                                                                  
⏺ Good. Now let's tackle the question that's blocking everything — OQ1: What data do we store?
                                                                                                                                                                                                                  
  The user journey tells us some constraints:                                                                                                                                                                     
  - The DB must be self-contained enough that the HTTP server can answer queries without re-parsing source files                                                                                                  
  - But we need file hashes to detect staleness (FRESH vs STALE)                                                                                                                                                  
  - The deep dive (Event 7) needs actual source code — do we store it or read it live?                                                          