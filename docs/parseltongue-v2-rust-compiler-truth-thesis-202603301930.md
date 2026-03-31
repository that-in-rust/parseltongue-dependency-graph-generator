# Parseltongue v2: Rust Code Reading Companion — Compiler Truth Edition

**Constraint: Rust codebases only. Compiler-verified graphs. Zero ambiguity.**
**Core UX: Semantic Focus Lens. Moat: Variant Graph Overlays.**

---

## 1. Premise Check (Updated)

### What this product actually is:

A macOS desktop app where a developer drags in a Rust project folder, waits 10-60 seconds for
compiler-level indexing, and then has a persistent local environment for reading, browsing, and
learning that codebase. The app shows visual maps, source code with compiler-verified graph
annotations, and an embedded LLM that acts as a reading guide — not a chatbot.

Behind the scenes: SQLite stores entities and edges, `rustc_private` (pinned nightly) extracts
MIR-level control flow and call graphs, Python computes graph metrics, and a local HTTP API serves
structured packets to both the Tauri frontend and the LLM.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        WHAT CHANGED FROM v1                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  v1 (tree-sitter)              v2 (rustc_private + MIR)                 │
│  ─────────────────             ──────────────────────────                │
│  "foo() calls bar()"           "foo() calls THIS SPECIFIC bar()"        │
│  "3 possible targets"          "1 resolved target (compiler-verified)"  │
│  Syntax-level edges            Semantic-level edges                     │
│  Approximate ownership         Polonius-verified borrow facts           │
│  No control flow               Exact CFG (BasicBlock DAG)              │
│  No type information           Full type of every expression            │
│  Multi-language                 Rust-only (by design)                   │
│                                                                         │
│  The graph is no longer approximate. It is compiler truth.              │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Most important implications for human code reading:

1. **The graph IS the compiler's graph.** (Direct.) MIR basic blocks, call edges from
   `TerminatorKind::Call`, trait dispatch resolved via `Instance::try_resolve()`. Every edge in the
   graph is an edge the compiler verified. No heuristic matching. No "3 possible targets." One
   target. The right one.

2. **Control flow is visible as a first-class surface.** (New.) MIR gives us the actual CFG — basic
   blocks, branches, loops (via dominator tree back-edges), panic paths, drop glue calls. A
   newcomer can see "this function has 4 branches: happy path, error case, timeout, and panic" as
   a visual diagram, not by reading nested match arms.

3. **Ownership and borrowing are compiler-verified, not syntactic guesses.** (Upgraded.) Polonius
   facts give us: where borrows are created (`loan_issued_at`), where they're live
   (`origin_live_at`), where moves happen (`path_moved_at_base`), where drops occur
   (`var_dropped_at`). The ownership visualizer is no longer "approximate" — it shows what the
   borrow checker actually verified.

4. **Trait dispatch is resolved.** (New.) `Instance::try_resolve(tcx, typing_env, def_id, args)`
   resolves `trait_method()` to the concrete implementation. The Trait/Impl Browser shows not just
   "who implements this trait" but "which implementation is called HERE, in THIS call site, with
   THESE generic parameters."

5. **The app is Rust-only, and that's the point.** (Constraint.) By constraining to Rust, we get
   compiler-level truth that no multi-language tool can provide. Tree-sitter still handles search
   indexing (FTS5, trie, trigram), but the graph — the thing that powers every reading mode — comes
   from the compiler.

### Biggest constraint shaping the UX (UPDATED):

**The old constraint is gone.** "Edge quality caps explanation depth" was the v1 constraint. With
MIR, edge quality is 100% for all resolvable calls. The new constraint is:

**Dynamic dispatch (`dyn Trait`) cannot be fully resolved at compile time.** When the compiler sees
`&dyn MessageHandler`, it knows the trait but not the concrete type. This is the ONE remaining case
where the product must show uncertainty. Mitigation: show ALL implementations with a "dynamic
dispatch — runtime determines which" label. For everything else (static dispatch, generics,
closures), the graph is exact.

```
┌──────────────────────────────────────────────────────────────┐
│                    EDGE CONFIDENCE MODEL                     │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Edge Type                    Confidence    Source            │
│  ─────────────────────────    ──────────    ──────────────    │
│  Direct call (fn → fn)        100%          MIR terminator   │
│  Static dispatch (T: Trait)   100%          Instance::resolve │
│  Generic monomorphized        100%          MIR instance      │
│  Closure invocation           100%          MIR closure body  │
│  Dynamic dispatch (dyn)       N candidates  trait_impls_of   │
│  Drop glue                    100%          MIR Drop term     │
│  Async (.await)               100%          MIR generator     │
│                                                              │
│  Visual rule:                                                │
│  ────── solid line = 100% resolved                           │
│  ┄┄┄┄┄┄ dashed line = dynamic dispatch (N candidates)       │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 2. Product Thesis (Unchanged Core, Upgraded Foundation)

The 8 thesis points from v1 remain identical. The reading environment metaphor, the orientation →
comprehension → navigation micro-loop, concept-browsing over folder-browsing, the LLM as senior
engineer not chatbot, spatial progressive disclosure, reading history as first-class — all of this
holds.

What changes is the **foundation beneath them**:

```
v1 thesis: "The graph provides reading structure that file trees cannot."
v2 thesis: "The COMPILER'S graph provides reading structure that file trees cannot."

v1 thesis: "Show uncertainty — '3 possible targets.'"
v2 thesis: "Show certainty — '1 verified target.' Reserve uncertainty for dyn only."

v1 thesis: "The LLM fills gaps where tree-sitter can't see."
v2 thesis: "The LLM narrates what the compiler already knows."
```

**New thesis points (9, 10, 11):**

9. **The compiler is the source of truth, and the product never contradicts it.** Every edge, every
   type annotation, every ownership fact shown in the UI must trace back to a `rustc_private` query.
   If the compiler doesn't know it, the product doesn't claim it. The LLM can speculate ("this
   pattern is probably used for X"), but the graph never speculates.

10. **Zooming is changing levels of abstraction, not scaling the raw graph (Semantic Focus Lens).**
    You should not zoom the raw graph. You should zoom the representation. The product behaves like
    a focus lens: the selected thing is fully saturated, its 1-hop neighborhood is visible and
    ranked, its 2-hop neighborhood is faint, and unrelated areas are ghosted or hidden. Boundary
    nodes are shown as exits, not full clutter.

    "Importance" is always relative to a focus. The ranking stack:
    1. Local relevance: `Personalized PageRank` from the selected node
    2. Structural proximity: `BFS` distance
    3. Global importance: `PageRank`
    4. Edge semantics: calls, impls, type refs, public boundary edges

    This produces four zoom levels:
    - **Workspace level**: subsystems and communities
    - **Subsystem level**: modules, public APIs, representative files
    - **Entity level**: one function/type/trait and its ego network
    - **Flow level**: CFG, DDG, type-flow slices inside one chosen unit

```
┌──────────────────────────────────────────────────────────────────────┐
│                      SEMANTIC FOCUS LENS                             │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  When the user selects an entity:                                    │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │                                                             │     │
│  │     ░░░░░░░░░░░  ghosted  ░░░░░░░░░░░░░░░░░░░░            │     │
│  │   ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░           │     │
│  │   ░░░░░  ▒▒▒▒▒▒▒▒▒▒▒▒▒  faint (2-hop)  ░░░░░░           │     │
│  │   ░░░░░  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  ░░░░░           │     │
│  │   ░░░░░  ▒▒▒  ▓▓▓▓▓▓▓▓▓▓▓▓▓  visible    ░░░░░           │     │
│  │   ░░░░░  ▒▒▒  ▓▓▓▓▓▓▓▓▓▓▓▓▓  (1-hop)    ░░░░░           │     │
│  │   ░░░░░  ▒▒▒  ▓▓▓  ████████  ▓▓▓  ▒▒▒▒  ░░░░░           │     │
│  │   ░░░░░  ▒▒▒  ▓▓▓  █ FOCUS█  ▓▓▓  ▒▒▒▒  ░░░░░           │     │
│  │   ░░░░░  ▒▒▒  ▓▓▓  ████████  ▓▓▓  ▒▒▒▒  ░░░░░           │     │
│  │   ░░░░░  ▒▒▒  ▓▓▓▓▓▓▓▓▓▓▓▓▓  ▒▒▒  ▒▒▒▒  ░░░░░           │     │
│  │   ░░░░░  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  ▒▒▒▒  ░░░░░           │     │
│  │   ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░           │     │
│  │     ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░            │     │
│  │                                                             │     │
│  │  Boundary nodes shown as ◇ exit portals, not full entities  │     │
│  │  Click a boundary node → re-center focus there              │     │
│  └─────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  Ranking within each ring:                                           │
│    1. PPR score from focus (local relevance)                         │
│    2. BFS distance (structural proximity)                            │
│    3. Global PageRank (importance)                                   │
│    4. Edge kind weight (calls > impls > type_refs > contains)        │
│                                                                      │
│  Zoom levels:                                                        │
│    Workspace ──► Subsystem ──► Entity ──► Flow                       │
│    (communities)  (modules)   (ego net)  (CFG/DDG)                   │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

11. **Variant overlays give LLMs what they have never had: structured architectural reasoning at
    the public interface level (Variant Graph Overlays).**

    Today, when you ask an LLM "should I restructure this module?", it sees raw source and
    pattern-matches against training data. It has no concept of coupling metrics, community
    boundaries, or what happens to the dependency graph when you move a public API. It reasons
    about code as text, not as architecture.

    Variants change this. They give the LLM a **graph-level workspace** where it can:
    - Propose structural changes as typed operations (not prose)
    - See computed consequences (not hallucinated ones)
    - Compare options with real metrics (not vibes)
    - Reason at the public interface level — module boundaries, trait abstractions, dependency
      direction — the abstraction level where architecture decisions actually happen

    This is the level that LLMs currently cannot reach. They can refactor a function. They cannot
    reason about whether introducing a trait boundary between two modules reduces coupling enough
    to justify the abstraction cost — because they have no way to compute "coupling" or "cost."
    The consequence engine gives them that.

    **How it works:**

    An architecture option is not just "add edge." It is often: add edge, remove edge, reroute
    dependency, replace direct dependency with interface dependency, collapse or split a node.

    Representation:
    - **Base snapshot**: the current compiler-verified graph (truth)
    - **Variant A/B/C**: overlay deltas on the base

    Each delta contains typed, justified, clearly-marked-as-proposed operations:
    - `add_edge { src, dst, kind, rationale }`
    - `remove_edge { src, dst, kind, rationale }`
    - `change_edge_kind { src, dst, old_kind, new_kind, rationale }`

    Every API can be queried as `?variant=current` or `?variant=option-1`.

    The LLM receives **difference packets**, not raw graph dumps:
    - edges added/removed
    - new reachability changes
    - SCC changes (cycles introduced or broken)
    - PageRank delta (what got more/less important)
    - k-core delta (what moved to core/periphery)
    - Leiden community boundary changes
    - hotspot shifts
    - public boundary crossings changed

    This turns "graph dump comparison" into an **architectural consequence engine** — and it
    gives the LLM a language for architecture that it has never had before.

```
┌──────────────────────────────────────────────────────────────────────┐
│           WHY LLMs CANNOT DO THIS TODAY                              │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  What LLMs see today:           What variants give them:             │
│  ─────────────────────          ──────────────────────────           │
│  Raw source code                Public interface graph               │
│  "This function calls that"     "Module A couples to B via 8 edges"  │
│  Pattern matching on text       Computed coupling metrics             │
│  "I think you should..."        "Variant A reduces coupling by 26%"  │
│  No consequence computation     PageRank/SCC/k-core/Leiden diffs     │
│  Reasoning by analogy           Reasoning by measurement             │
│                                                                      │
│  The gap:                                                            │
│  LLMs reason about code at LINE level.                               │
│  Architecture decisions happen at MODULE BOUNDARY level.             │
│  Variants bridge this gap by giving LLMs a structured workspace     │
│  at the right abstraction level.                                     │
│                                                                      │
│  Example:                                                            │
│  ────────                                                            │
│  Human: "Should I add a trait boundary between Server and Consumer?" │
│                                                                      │
│  Without variants (today):                                           │
│  LLM: "It depends on your use case. Generally, trait boundaries      │
│  improve testability but add complexity..." (generic, unhelpful)     │
│                                                                      │
│  With variants:                                                      │
│  LLM creates variant → consequence engine computes →                │
│  LLM: "Adding ConsumerAPI trait between Server and Consumer:         │
│    - Reduces Server→Consumer coupling from 8 edges to 1              │
│    - Consumer's PageRank drops 26% (less central, more isolated)     │
│    - Creates a new community boundary (streaming splits from server) │
│    - No cycles introduced ✓                                         │
│    - Cost: 1 new trait + 1 impl block                               │
│    Worth it if you plan multiple Consumer implementations.           │
│    Not worth it if Consumer is the only implementation."             │
│                                                                      │
│  The LLM went from "it depends" to a specific, measured             │
│  recommendation — because it had the consequence data.              │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

    **Trust constraint**: proposed edges must be typed, variant-scoped, justified with rationale,
    and clearly marked as `proposed` — never `truth`. The base graph is always compiler-verified.
    Variants are always human- or LLM-proposed hypotheticals. The LLM reasons with real metrics
    but never asserts that a variant IS the codebase — only that it WOULD produce these
    consequences IF applied.

```
┌──────────────────────────────────────────────────────────────────────┐
│                     VARIANT GRAPH OVERLAYS                            │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────────────────────────────────┐                      │
│  │  BASE SNAPSHOT (compiler truth)            │                      │
│  │  ══════════════════════════════            │                      │
│  │  A ──calls──► B ──calls──► C              │                      │
│  │  A ──calls──► D                            │                      │
│  │  D ──impls──► TraitX                       │                      │
│  └────────────────────┬───────────────────────┘                      │
│                       │                                              │
│         ┌─────────────┼─────────────┐                                │
│         ▼             ▼             ▼                                │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐                  │
│  │  VARIANT A   │ │  VARIANT B   │ │  VARIANT C   │                  │
│  │  "Interface" │ │  "Direct"    │ │  "Merge"     │                  │
│  │ ──────────── │ │ ──────────── │ │ ──────────── │                  │
│  │ + A→I (new)  │ │ + A→C (new)  │ │ - D (remove) │                  │
│  │ + I→B (new)  │ │ - A→B (rem)  │ │ + A→B_D      │                  │
│  │ - A→B (rem)  │ │              │ │   (merged)   │                  │
│  │ + trait Iface│ │              │ │              │                  │
│  └──────┬───────┘ └──────┬───────┘ └──────┬───────┘                  │
│         │                │                │                          │
│         ▼                ▼                ▼                          │
│  ┌───────────────────────────────────────────────┐                   │
│  │  CONSEQUENCE ENGINE (per variant)              │                   │
│  │  ─────────────────────────────────             │                   │
│  │  • PageRank delta: B drops 0.04 → 0.02        │                   │
│  │  • New SCC: none / cycle introduced?           │                   │
│  │  • k-core shift: D moves periphery → core     │                   │
│  │  • Community boundary: A and B now split       │                   │
│  │  • Hotspot change: I becomes new hotspot       │                   │
│  │  • Public boundary: I now exposed              │                   │
│  └───────────────────────────────────────────────┘                   │
│                                                                      │
│  API: /communities?variant=option-1                                  │
│       /hotspots?variant=option-2                                     │
│       /variant/{id}/diff  ← the difference packet                   │
│                                                                      │
│  Trust rule: base = solid lines. Variant = dotted + "proposed" tag.  │
│              LLM narrates consequences, never asserts truth.         │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 2.5. Typed Boundary Aggregation Model

The dependency graph is just an edge table. Most architectural reasoning — "what's in this module,
what does it depend on, how coupled are these two crates" — is pure relational queries. No graph
algorithms required.

**But the original "three-level GROUP BY dirname()" model was wrong.** Validated against the iggy
codebase (1,313 Rust files, 23 crates, 338 directories), it revealed critical gaps:

1. Crate boundaries (Cargo.toml) are compiler-enforced HARD boundaries. Folder boundaries within
   a crate are SOFT. Treating them identically loses the most important architectural signal.
2. `server/binary/ → server/shard/` (intra-crate, can access pub(crate)) and
   `server/ → common/` (cross-crate, can only access pub) are fundamentally different kinds of
   coupling. An LLM must know which one it's looking at.
3. Counting `use` statements is not the same as counting imported items. `use X::{A, B, C}` is
   one statement importing three items. 48 files importing `IggyError` is 48 uses of one item.
4. Internal cohesion matters as much as external coupling.
5. Structural symmetry (tcp/, quic/, websocket/ all having identical dependency profiles) is
   architecturally significant and detectable with pure SQL.
6. Re-exports (facade crates) hide real dependencies.

### Converged Design: Three Tables, Typed Boundaries

```
┌──────────────────────────────────────────────────────────────────────┐
│              TYPED BOUNDARY AGGREGATION MODEL                        │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Three tables:                                                       │
│                                                                      │
│  1. entities       (unchanged — id, name, kind, file_path, etc.)     │
│  2. edges          (unchanged — src_id, dst_id, edge_kind)           │
│  3. boundaries     (NEW — the aggregation layer)                     │
│                                                                      │
│  A boundary is a named container with:                               │
│    - a path (crate root or module directory)                         │
│    - a type: "crate" | "module" | "folder"                          │
│    - a parent boundary (nesting)                                     │
│    - computed metrics (pub surface, cohesion, coupling)               │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Boundary Types

```
┌──────────────────────────────────────────────────────────────────────┐
│  THREE BOUNDARY TYPES                                                │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  "crate"    Has Cargo.toml. Compiler-enforced boundary.              │
│             Only pub items accessible from outside.                  │
│             Declared dependencies in [dependencies].                 │
│             Changing a cross-crate edge = changing a public API.     │
│             Cost of change: HIGH.                                    │
│                                                                      │
│  "module"   Has mod.rs or is declared in parent's mod tree.          │
│             pub(crate) items accessible within crate.                │
│             pub items accessible from outside crate.                 │
│             Changing an intra-crate edge = internal refactor.        │
│             Cost of change: LOW.                                     │
│                                                                      │
│  "folder"   Any directory containing .rs files with no mod           │
│             declaration. No compiler significance.                   │
│             Organizational only.                                     │
│             Cost of change: TRIVIAL.                                │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Real-World Boundary Tree (iggy)

Validated against the iggy streaming platform (23 crates, 1313 .rs files):

```
workspace (root)
├── server/           [crate]     ← Cargo.toml, 228 files
│   ├── binary/       [module]    ← 61 files, handles wire protocol
│   │   └── handlers/ [module]    ← one handler per command type
│   ├── shard/        [module]    ← 52 files, THE internal hub
│   ├── streaming/    [module]    ← 49 files, core domain (well-isolated)
│   ├── http/         [module]    ← 24 files, HTTP transport
│   ├── metadata/     [module]    ← 12 files
│   ├── tcp/          [module]    ← 6 files, TCP transport
│   ├── quic/         [module]    ← 4 files, QUIC transport
│   ├── websocket/    [module]    ← 5 files, WebSocket transport
│   ├── state/        [module]
│   ├── compat/       [module]
│   ├── io/           [module]
│   └── log/          [module]
├── common/           [crate]     ← 120 entities, THE hub crate
│   ├── commands/     [module]
│   ├── error/        [module]
│   ├── types/        [module]
│   └── traits/       [module]
├── binary_protocol/  [crate]     ← wire format, second hub
├── sdk/              [crate]     ← client library
├── cli/              [crate]     ← CLI tool
├── partitions/       [crate]
├── metadata/         [crate]
├── shard/            [crate]
├── consensus/        [crate]
├── journal/          [crate]
├── message_bus/      [crate]
├── clock/            [crate]
├── configs/          [crate]
└── ...               [crate × 23 total]
```

### Edge Classification by Boundary Crossing

```
┌──────────────────────────────────────────────────────────────────────┐
│  EDGES CARRY THEIR CROSSING TYPE                                     │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Edge                                    Crossing type               │
│  ────────────────────────────────────    ─────────────────────────   │
│  server/binary/ → server/shard/          INTRA-CRATE (module→module) │
│  server/ → common/                       CROSS-CRATE (crate→crate)   │
│  server/shard/tasks/ → server/shard/     INTRA-MODULE (child→parent) │
│                                                                      │
│  Why this matters for LLMs:                                          │
│                                                                      │
│  An LLM saying "decouple A from B" means very different things       │
│  depending on whether A→B crosses a crate boundary or not:           │
│                                                                      │
│  CROSS-CRATE: "Change the public API of the dependency crate,       │
│    update Cargo.toml, potentially break all downstream consumers."   │
│    → The LLM should warn about blast radius.                        │
│                                                                      │
│  INTRA-CRATE: "Move some use crate:: imports. Internal refactor."   │
│    → The LLM can recommend this freely.                             │
│                                                                      │
│  INTRA-MODULE: "Rearrange code within one module."                  │
│    → Almost trivial. The LLM shouldn't even mention cost.           │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Boundary Metrics (all GROUP BY, no graph algorithms)

```
┌──────────────────────────────────────────────────────────────────────┐
│  PER-BOUNDARY METRICS                                                │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Metric            How computed                What it tells you     │
│  ────────────────  ──────────────────────────  ────────────────────  │
│  entity_count      COUNT entities in boundary  Size                  │
│  pub_surface       COUNT WHERE vis = pub       Interface width       │
│  internal_edges    COUNT WHERE src+dst inside  Internal wiring       │
│  outgoing_edges    COUNT WHERE src in, dst out Dependency load       │
│  incoming_edges    COUNT WHERE dst in, src out Dependent load        │
│  cohesion          internal / entity_count     How well-connected    │
│  coupling_out      outgoing / entity_count     How dependent         │
│  coupling_in       incoming / entity_count     How depended upon     │
│  fan_in            COUNT DISTINCT src bounds   How many consumers    │
│  fan_out           COUNT DISTINCT dst bounds   How many dependencies │
│  import_breadth    DISTINCT items / pub_surf   Width of coupling     │
│                    of the target               (what % of API used)  │
│  import_spread     DISTINCT importing files /  Spread of coupling    │
│                    total files in consumer     (how pervasive)       │
│  is_facade         >60% of items are pub use   Facade/re-export crate│
│                                                                      │
│  ALL of these are GROUP BY + COUNT. No graph algorithms.             │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Real Metrics on iggy (validated)

```
┌──────────────────────────────────────────────────────────────────────┐
│  IGGY BOUNDARY METRICS — REAL DATA                                   │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Boundary: common/ [crate]                                           │
│    entity_count:   ~120                                              │
│    pub_surface:    ~80 (most things are pub — it's a library)        │
│    outgoing_edges: 0 (depends on nothing internal)                   │
│    incoming_edges: ~900 (everyone depends on it)                     │
│    coupling_in:    7.50 (900/120) ← extremely depended upon         │
│    fan_in:         14 (14 crates depend on it)                       │
│    fan_out:        0 (leaf dependency)                               │
│    Insight: Hub crate. Pure library. No outgoing dependencies.       │
│                                                                      │
│  Boundary: server/ [crate]                                           │
│    entity_count:   ~800                                              │
│    pub_surface:    ~40 (most things are pub(crate))                  │
│    outgoing_edges: ~445 (330 to common, 115 to binary_protocol)      │
│    incoming_edges: ~20 (cli and integration tests)                   │
│    cohesion:       0.50 (400/800)                                    │
│    coupling_out:   0.56 (445/800) ← high external dependency        │
│    fan_out:        2 (depends on 2 crates)                           │
│    fan_in:         2 (cli + integration depend on it)                │
│    Insight: Big consumer. Most deps are on common/.                  │
│                                                                      │
│  Boundary: server/streaming/ [module]                                │
│    entity_count:   ~80                                               │
│    internal_edges: ~50                                               │
│    outgoing_edges: 2 (only to server/shard/)                         │
│    incoming_edges: ~120 (binary, http, shard, metadata, all call it) │
│    cohesion:       0.63 ← HIGH, well-structured module              │
│    coupling_out:   0.025 ← almost no outgoing deps = well isolated  │
│    coupling_in:    1.50 ← everyone depends on it                    │
│    Insight: Core domain. High cohesion, low outgoing coupling.       │
│    This is the best-designed module in server/.                      │
│                                                                      │
│  Boundary: server/shard/ [module]                                    │
│    entity_count:   ~90                                               │
│    outgoing_edges: ~43 (streaming=34, metadata=5, http=2, ...)       │
│    incoming_edges: ~147 (binary=98, http=25, tcp=9, websocket=9,...) │
│    cohesion:       0.45                                              │
│    coupling_out:   0.48 ← moderate                                  │
│    coupling_in:    1.63 ← THE internal hub                          │
│    fan_in:         8 (8 modules depend on it)                        │
│    Insight: Orchestrator. Everything routes through shard/.          │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Structural Symmetry Detection (pure SQL)

```
┌──────────────────────────────────────────────────────────────────────┐
│  SYMMETRY: BOUNDARIES WITH IDENTICAL DEPENDENCY PROFILES             │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Query: group boundaries by their sorted list of outgoing targets.   │
│  Any group with >1 member = structural symmetry.                    │
│                                                                      │
│  SELECT                                                              │
│    GROUP_CONCAT(src_boundary) AS symmetric_set,                      │
│    dep_profile                                                       │
│  FROM (                                                              │
│    SELECT src_boundary,                                              │
│      GROUP_CONCAT(dst_boundary ORDER BY dst_boundary) AS dep_profile │
│    FROM boundary_edges                                               │
│    WHERE crossing_type = 'INTRA-CRATE'                              │
│    GROUP BY src_boundary                                             │
│  )                                                                   │
│  GROUP BY dep_profile HAVING COUNT(*) > 1                            │
│                                                                      │
│  Iggy result:                                                        │
│  ─────────────                                                       │
│  symmetric_set:  "tcp/, quic/, websocket/"                           │
│  dep_profile:    "binary/, shard/, streaming/"                        │
│                                                                      │
│  → "tcp/, quic/, and websocket/ are structurally interchangeable.    │
│     They all depend on the same 3 modules. They are transport        │
│     layer implementations with identical architectural roles."       │
│                                                                      │
│  This is a GROUP BY + HAVING query. No graph algorithm.              │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Facade Crate Detection

```
┌──────────────────────────────────────────────────────────────────────┐
│  RE-EXPORTS: DETECTING FACADE CRATES                                 │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Problem: cli/ uses `iggy::Client` but iggy/ re-exports from sdk/.   │
│  The REAL dependency is cli/ → sdk/, not cli/ → iggy/.               │
│                                                                      │
│  Strategy 1 (tree-sitter, Phase 0):                                  │
│    Count `pub use` vs total items per crate.                        │
│    IF > 60% of a crate's pub items are `pub use` re-exports         │
│    THEN mark is_facade = true.                                      │
│    Show: "cli/ depends on iggy/ (facade for sdk/ + common/)."       │
│    Good enough for LLM reasoning.                                   │
│                                                                      │
│  Strategy 2 (MIR, Phase 4):                                         │
│    Instance::try_resolve() traces through re-exports.                │
│    Show both declared AND resolved dependency.                      │
│    "cli/ imports from iggy/ which re-exports from sdk/."            │
│                                                                      │
│  Decision: Start with Strategy 1. Upgrade with MIR later.           │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### How Boundaries Are Discovered (at index time)

```
┌──────────────────────────────────────────────────────────────────────┐
│  BOUNDARY DISCOVERY PIPELINE                                         │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Step 1: Scan for Cargo.toml files                                   │
│    → Create "crate" boundary for each                               │
│    → Parse [dependencies] for declared crate-to-crate deps          │
│                                                                      │
│  Step 2: Scan for mod.rs / mod declarations                          │
│    → Create "module" boundary for each                              │
│    → Parse visibility: pub mod, pub(crate) mod                      │
│                                                                      │
│  Step 3: Any remaining directory with .rs files                      │
│    → Create "folder" boundary (no compiler significance)            │
│                                                                      │
│  Step 4: Set parent_id by path containment                           │
│    → server/shard/ parent = server/                                 │
│    → server/ parent = workspace                                     │
│                                                                      │
│  Step 5: Compute boundary_edges by classifying entity edges          │
│    → For each entity edge (src→dst):                                │
│      Find src boundary and dst boundary                             │
│      If same boundary: internal_edge++                               │
│      If different boundary, same crate: INTRA-CRATE crossing        │
│      If different crate: CROSS-CRATE crossing                       │
│    → GROUP BY (src_boundary, dst_boundary)                          │
│                                                                      │
│  Step 6: Compute per-boundary metrics                                │
│    → entity_count, pub_surface, cohesion, coupling, fan_in/out      │
│    → All GROUP BY. No graph algorithms.                             │
│                                                                      │
│  Total time: <1 second for iggy (1313 files).                       │
│  These are materialized tables, recomputed on re-index.              │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### What the LLM Gets (before vs after)

```
┌──────────────────────────────────────────────────────────────────────┐
│  LLM CONTEXT PACKET: BEFORE vs AFTER                                 │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  BEFORE (flat folder GROUP BY):                                      │
│    "{crate_a}/ depends on {crate_b}/ ({N} uses)"                    │
│                                                                      │
│  AFTER (typed boundary with metrics):                                │
│    "{crate_a}/ [crate, {file_count} files, pub_surface={M}]         │
│     depends on:                                                      │
│      {crate_b}/ [crate, leaf, pub_surface={P}] — {N} edges,        │
│        CROSS-CRATE, imports {K} of {P} pub items                    │
│        ({breadth}% breadth) from {F} files ({spread}% spread)       │
│                                                                      │
│    Internal structure:                                               │
│      {module_x}/ [module, cohesion={C}] — internal hub              │
│        (fan_in={I})                                                  │
│      {module_y}/ [module, cohesion={C}] — well-isolated core        │
│        (coupling_out={D})                                            │
│      {mod_a}/, {mod_b}/, {mod_c}/ — SYMMETRIC                      │
│        (identical dep profile: {shared_deps})                        │
│                                                                      │
│  The second version is what the LLM needs for architectural         │
│  reasoning. It's all GROUP BY. No PageRank, no Leiden."             │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

**Example (iggy codebase):**

```
  BEFORE: "server/ depends on common/ (330 uses)"

  AFTER:  "server/ [crate, 228 files, pub_surface=40] depends on:
            common/ [crate, leaf, pub_surface=80] — 330 edges,
              CROSS-CRATE, imports 12 of 80 pub items (15% breadth)
              from 48 files (21% spread)
            binary_protocol/ [crate] — 115 edges, CROSS-CRATE

          Internal structure:
            shard/ [module, cohesion=0.45] — internal hub (fan_in=8)
            streaming/ [module, cohesion=0.63] — well-isolated core
              (coupling_out=0.025)
            tcp/, quic/, websocket/ — SYMMETRIC
              (identical dep profile: binary+shard+streaming)

          Architectural insight: streaming/ is the cleanest module.
          shard/ is the orchestrator. The transport layers are
          interchangeable."
```

### Relationship to the Semantic Focus Lens

The boundary tree maps directly to the focus lens zoom levels:

```
  Focus Lens Zoom Level     Boundary Level               What's shown
  ─────────────────────     ──────────────────────────   ─────────────
  Workspace                 Crate boundaries (depth=1)   crate-to-crate
  Subsystem                 Module boundaries within     module-to-module
                            focused crate (depth=2)      within one crate
  Entity                    Entities within focused      entity-to-entity
                            module                       raw edges
  Flow                      CFG/borrows within one fn    sub-entity

  When the user zooms from workspace to subsystem, the query switches
  from boundary_edges WHERE boundary_type='crate' to boundary_edges
  WHERE parent='server/' AND boundary_type='module'. Same table,
  different filter.
```

### Relationship to Variants

Variant deltas operate on entity edges (L1). Boundary metrics are RECOMPUTED from the modified
edge table — same GROUP BY, different input:

```
  /boundary/{id}?variant={variant_id}
    → Metrics recomputed on base_edges + variant deltas
    → Crossing types and coupling counts reflect the hypothetical graph
    → New boundaries may appear if a variant introduces a new crate/module

  The consequence engine now speaks in boundary-level changes,
  not just entity-level edge diffs.
```

### Relationship to tree-sitter vs MIR

The boundary model works identically regardless of edge source. The boundary discovery
(Cargo.toml scanning, mod detection) is independent of how entity edges are extracted:

```
  Edge source        Entity edges (L1)   Boundary metrics
  ──────────────     ──────────────────  ──────────────────────────
  tree-sitter        approximate edges   approximate coupling counts
                     ("3 possible bar")  (may overcount if ambiguous)

  MIR (rustc)        exact edges         exact coupling counts
                     ("this specific     (every edge is real)
                      bar, resolved")

  Same boundary tables. Same queries. Different data quality.
  Start with tree-sitter. Upgrade to MIR. The interface doesn't change.
```

---

## 3. Reading Modes (17 Modes — 12 Upgraded + 3 Compiler + 2 Foundational)

### Mode 1: Architecture Overview (Upgraded)

**What changed:** Community clustering now runs on the compiler-verified call graph, not tree-sitter
approximations. Leiden communities are dramatically more accurate because every edge is real.

**What the user sees:** Same as v1 (8-15 Leiden clusters), but with two new badges per community:
- **Edge resolution**: "100% static" or "23 dynamic dispatch edges"
- **Type density**: "14 structs, 8 traits, 42 functions"

The modularity gate (suppress if < 0.3) is more likely to pass because the graph is cleaner.

---

### Mode 2: Module Deep Read (Upgraded)

**What changed:** Entity list now includes compiler-derived type information. Each entity shows its
full resolved signature, not just the name.

**What the user sees:** Same three-panel layout, but the entity list shows:
```
┌──────────────────────────────────────────────────────────────┐
│  Module: "Streaming Pipeline" (Leiden community #3)          │
├──────────────────────────────────────────────────────────────┤
│  #  Entity                    Type        PageRank  Callers  │
│  ── ─────────────────────     ────────    ────────  ───────  │
│  1  StreamProcessor::run      async fn    0.042     8        │
│  2  Consumer::poll             fn(&mut)   0.038     5        │
│  3  MessageHandler (trait)     trait      0.031     —        │
│  4  ConsumerGroup::rebalance   fn(&mut)   0.027     4        │
│  5  TopicPartition::read       fn(&self)  0.019     3        │
│     ...                                                      │
│                                                              │
│  New: fn(&mut) vs fn(&self) comes from MIR fn_sig(),         │
│  not tree-sitter pattern matching.                           │
└──────────────────────────────────────────────────────────────┘
```

---

### Mode 3: Call Chain Explorer (MAJOR UPGRADE)

**What changed:** Call chains are no longer BFS on approximate edges. They are extracted directly
from MIR `TerminatorKind::Call` terminators with fully resolved targets.

**v1 problem:** "process_message() calls bar() — but which bar()?"
**v2 answer:** "process_message() calls Consumer::bar() via static dispatch. Resolved."

**What the user sees:** Same vertical lane visualization, but:
- Every edge is solid (resolved) or dashed (dynamic dispatch only)
- Each card shows the resolved concrete type, not just the function name
- Branching is based on actual MIR branches (if/match arms), not heuristic call-site detection
- Error paths (`?` operator) are shown as a distinct branch color
- Drop/cleanup paths are shown as faded side-lanes

```
    ┌────────────────────────────────┐
    │  main()                        │
    │  src/main.rs:12:45             │
    └───────────┬────────────────────┘
                │ TerminatorKind::Call
                ▼
    ┌────────────────────────────────┐
    │  Server::start()               │
    │  src/server.rs:28:92           │
    │  async fn(&self) -> Result<()> │
    └──────┬──────────────┬──────────┘
           │              │
      ┌────▼────┐    ┌────▼────────────────┐
      │ Ok path │    │ Err path (? on L41) │
      │         │    │                     │
      ▼         │    ▼
    ┌───────────┤  ┌─────────────────────┐
    │ listen()  │  │ cleanup_resources() │
    │ ...       │  │ DROP path           │
    └───────────┘  └─────────────────────┘
```

---

### Mode 4: Trait/Impl Browser (MAJOR UPGRADE)

**What changed:** `Instance::try_resolve()` gives us the ability to show not just "who implements
this trait" but "which implementation is called at each call site."

**What the user sees:** Same split view (trait left, impls right), but with a new third column:

```
┌─────────────────┐  ┌──────────────────────┐  ┌──────────────────────────┐
│  trait Stream    │  │  Implementations     │  │  Call Sites (WHERE)      │
│  ─────────────   │  │  ──────────────────   │  │  ─────────────────────   │
│  fn poll_next()  │  │  impl Stream for     │  │  consumer_loop.rs:42     │
│  fn size_hint()  │  │    Consumer     ✓ 5x │  │    → Consumer::poll_next │
│                  │  │    BatchReader  ✓ 2x │  │  batch.rs:18             │
│  associated:     │  │    FileStream  ✓ 1x │  │    → BatchReader::poll   │
│  type Item       │  │                      │  │  test_stream.rs:7        │
│                  │  │  5x = called 5 times │  │    → FileStream::poll    │
│                  │  │  via static dispatch │  │                          │
│                  │  │                      │  │  3 dyn dispatch sites:   │
│                  │  │                      │  │    handler.rs:29 ┄┄┄     │
│                  │  │                      │  │    router.rs:55  ┄┄┄     │
│                  │  │                      │  │    proxy.rs:12   ┄┄┄     │
└─────────────────┘  └──────────────────────┘  └──────────────────────────┘
```

The "5x" count is exact — from MIR call site analysis, not heuristic grep.

---

### Mode 5: Guided Tour (Upgraded)

**What changed:** Tour generation uses compiler-verified importance (resolved call count, not
approximate edge count). Tours can now include a "Type System Tour" that follows generic
instantiations and trait hierarchies — impossible with tree-sitter.

**New tour types available:**
- "Architecture Tour" (breadth-first, Leiden communities) — same as v1
- "Message Flow Tour" (depth-first, MIR call chains) — upgraded, exact paths
- "Ownership Patterns Tour" (thematic, Polonius facts) — NEW, follows borrow lifecycle
- "Type System Tour" (trait hierarchy → impls → generic instantiations) — NEW, compiler-only
- "Error Handling Tour" (follows Result/? propagation through MIR) — NEW, exact error paths

---

### Mode 6: "You Are Here" Navigator (Same)

No change needed. Breadcrumbs + ego network work identically with compiler edges.

---

### Mode 7: "What Should I Read Next?" Recommender (Upgraded)

**What changed:** PPR now runs on the compiler-verified graph. Suggestions are better because the
graph has no false edges and no missing edges. The "why" reasoning is also richer:

```
v1: "Read this next because it calls the function you just read."
v2: "Read this next because it calls Consumer::poll via the Stream trait —
     that's the same trait you just saw implemented on line 42."
```

---

### Mode 8: Dependency Ladder (Upgraded)

**What changed:** Dependencies now include type dependencies (not just call dependencies). If
`struct Foo` contains a `Bar`, that's a dependency edge from MIR `type_of()`. Crate-level
dependencies from `extern crate` resolution.

---

### Mode 9: Hotspot Heatmap (Same algorithm, better data)

No algorithm change. But the composite score (PageRank + in-degree + k-core) is computed on a
cleaner graph, so hotspots are more meaningful.

---

### Mode 10: Reading History & Bookmarks (Same)

No change. Pure SQLite, no graph computation.

---

### Mode 11: Side-by-Side Comparison (Upgraded)

**What changed:** Comparison now includes type-level differences:

```
┌────────────────────────┐     ┌────────────────────────┐
│  TopicConsumer          │     │  ConsumerGroup         │
│  ──────────────         │     │  ─────────────         │
│  Shared traits:         │     │  Shared traits:        │
│    MessageHandler ✓     │     │    MessageHandler ✓    │
│                         │     │                        │
│  Unique traits:         │     │  Unique traits:        │
│    (none)               │     │    Rebalancer           │
│                         │     │                        │
│  Fields:                │     │  Fields:               │
│    topic: Topic         │     │    consumers: Vec<TC>  │
│    offset: u64          │     │    partition_map: HashMap│
│                         │     │    offset_store: Arc<> │
│  Ownership:             │     │  Ownership:            │
│    Owned, not shared    │     │    Arc<Mutex<>> shared  │
│                         │     │                        │
│  Generic params: none   │     │  Generic params:       │
│                         │     │    <H: MessageHandler> │
└────────────────────────┘     └────────────────────────┘
```

The field types, generic parameters, and ownership model come from `type_of()`, `generics_of()`,
and `fn_sig()` — compiler truth.

---

### Mode 12: Ownership & Borrow Visualizer (COMPLETELY REBUILT)

**What changed:** This is no longer "syntactic only." Polonius facts give us the actual borrow
checker's view.

**v1 label:** "Ownership Patterns (syntactic)" — honest about tree-sitter limits.
**v2 label:** "Ownership & Borrow Analysis (compiler-verified)"

**What the user sees:**

```
┌──────────────────────────────────────────────────────────────────────┐
│  fn process(&mut self, msg: Message) -> Result<(), Error> {         │
│  ────────────────────────────────────────────────────────────        │
│       ▲ &mut self                                                    │
│       │ LOAN L1: mutable borrow of self                             │
│       │ LIVE: lines 12-45                                            │
│       │                                                              │
│  12   │  let data = msg.payload;  ◄── MOVE: msg.payload moved here  │
│       │                               msg partially moved            │
│       │                                                              │
│  18   │  self.buffer.push(data);  ◄── MUTATION: why &mut required   │
│       │                               L1 used here (write)          │
│       │                                                              │
│  25   │  let view = &self.buffer; ◄── LOAN L2: shared borrow        │
│       │                               L2 LIVE: lines 25-30          │
│       │                               ⚠ L1 and L2 coexist           │
│       │                               (ok: L2 is shared, no write)  │
│       │                                                              │
│  30   │  drop(view);              ◄── L2 ENDS here                  │
│       │                                                              │
│  35   │  self.buffer.clear();     ◄── L1 write (ok: L2 is dead)    │
│       │                                                              │
│  45   }  ◄── L1 ENDS, &mut self released                           │
│                                                                      │
│  Legend: ■ owned  ━ &mut borrow  ─ & borrow  ⟶ move  ⚠ overlap     │
└──────────────────────────────────────────────────────────────────────┘
```

**Data source:** Polonius facts:
- `loan_issued_at(origin, loan, point)` → where borrows start
- `origin_live_at(origin, point)` → borrow liveness range
- `path_moved_at_base(path, point)` → move locations
- `var_defined_at`, `var_used_at`, `var_dropped_at` → variable lifecycle

---

### Mode 13: Control Flow Graph Visualizer (NEW)

**Problem it solves:** "I can read the source, but I can't see the branching structure."

**Who it's for:** Anyone trying to understand complex match/if/loop logic.

**What the user sees:** A visual CFG derived directly from MIR `BasicBlock` structure:

```
┌───────────────────────────────────────────────────────────────┐
│  fn handle_message(&mut self, msg: Message) -> Result<()>     │
│                                                               │
│  ┌─────────┐                                                  │
│  │  bb0     │  let kind = msg.kind();                         │
│  │  ENTRY   │                                                 │
│  └────┬─────┘                                                 │
│       │ SwitchInt(kind)                                       │
│       ├──────────────────┬──────────────────┐                 │
│       ▼                  ▼                  ▼                 │
│  ┌─────────┐       ┌─────────┐       ┌─────────┐             │
│  │  bb1     │       │  bb2     │       │  bb3     │            │
│  │ Data(d)  │       │ Control │       │ Error(e) │            │
│  └────┬─────┘       └────┬─────┘       └────┬─────┘           │
│       │                  │                  │                 │
│       ▼                  ▼                  ▼                 │
│  ┌─────────┐       ┌─────────┐       ┌─────────┐             │
│  │  bb4     │       │  bb5     │       │  bb6     │            │
│  │ process()│       │ handle()│       │ return   │            │
│  │ ──►call  │       │ ──►call │       │ Err(e)   │            │
│  └────┬─────┘       └────┬─────┘       └─────────┘            │
│       │                  │                                    │
│       └────────┬─────────┘                                    │
│                ▼                                              │
│           ┌─────────┐                                         │
│           │  bb7     │                                        │
│           │ return   │                                        │
│           │ Ok(())   │                                        │
│           └─────────┘                                         │
│                                                               │
│  Nodes: 8 basic blocks                                        │
│  Back-edges (loops): none                                     │
│  Unreachable blocks: none                                     │
│  Dominator tree depth: 3                                      │
└───────────────────────────────────────────────────────────────┘
```

**APIs:** `tcx.optimized_mir(def_id)` → `body.basic_blocks` → successors/predecessors/dominators.

**Why it matters:** No tool shows this. VS Code shows source. Parseltongue shows the execution
structure the compiler actually built. A function with 15 match arms looks like a flat list in
source but reveals its true branching structure as a CFG.

---

### Mode 14: Data Flow Explorer (NEW)

**Problem it solves:** "Where does this value come from? Where does it go?"

**Who it's for:** Anyone tracing how data moves through a function or across call boundaries.

**What the user sees:** Select a variable or expression. The app highlights:
- **Definitions:** Where the value is created (assignments, function params, return values)
- **Uses:** Where the value is read
- **Moves:** Where ownership transfers (arrows)
- **Drops:** Where the value is destroyed

```
┌──────────────────────────────────────────────────────────────┐
│  Tracing: `config` through StreamProcessor::new()            │
│                                                              │
│  DEFINITION ──► let config = Config::from_file(path)?;       │
│                 src/processor.rs:14                           │
│                      │                                       │
│                      │ MOVE                                  │
│                      ▼                                       │
│  USE ──────────► let pool = ConnectionPool::new(&config);    │
│                  src/processor.rs:18                          │
│                  (shared borrow, config still owned)         │
│                      │                                       │
│                      │ MOVE                                  │
│                      ▼                                       │
│  MOVE ─────────► StreamProcessor { config, pool, ... }       │
│                  src/processor.rs:25                          │
│                  (config moved into struct, no longer local)  │
│                      │                                       │
│                      │ STORED IN                             │
│                      ▼                                       │
│  FIELD ────────► self.config (owned by StreamProcessor)      │
│                  accessible via &self / &mut self             │
│                                                              │
│  Source: MIR Place + Projection analysis                     │
│          Operand::Move vs Operand::Copy                      │
│          Rvalue assignments                                   │
└──────────────────────────────────────────────────────────────┘
```

**Data source:** MIR `Place`, `Projection`, `Operand::Move`/`Copy`, `Rvalue`, plus
`MaybeInitializedPlaces` and `MaybeLiveLocals` dataflow analyses.

---

### Mode 15: Lifetime & Borrow Scope Visualizer (NEW)

**Problem it solves:** "Why does the borrow checker reject this? What outlives what?"

**Who it's for:** Rust newcomers fighting the borrow checker.

**What the user sees:** A timeline view showing when each borrow is live, when they overlap, and
why the compiler accepts or rejects the code:

```
┌────────────────────────────────────────────────────────────────┐
│  Lifetime Analysis: process_batch()                            │
│                                                                │
│  Line   Code                          Borrows Active           │
│  ────   ────────────────────────────   ─────────────────────    │
│   10    let items = &self.queue;       ┃ L1: &self.queue        │
│   11    let count = items.len();       ┃                        │
│   12    for item in items.iter() {     ┃ L2: &item (from L1)   │
│   13      self.process(item);          ┃ ┃  ⚠ ERROR:           │
│         ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─    ┃ ┃  &self (L1) active  │
│         Compiler says: cannot borrow   ┃ ┃  &mut self needed    │
│         `self` as mutable because it   ┃ ┃  by process()        │
│         is also borrowed as immutable  ┃ ┃                      │
│   14    }                              ┃ ┗━ L2 ends             │
│   15    // L1 still active here        ┃                        │
│   16    drop(items);                   ┗━━━ L1 ends             │
│   17    self.process_remaining();      ✓ OK: no active borrows  │
│                                                                │
│  Fix suggestion (from compiler):                               │
│  Collect items first: let items: Vec<_> = self.queue.iter()    │
│  .cloned().collect(); — then self is free for &mut             │
│                                                                │
│  Source: Polonius subset_base(origin1, origin2, point)         │
│          + loan_issued_at + origin_live_at                     │
└────────────────────────────────────────────────────────────────┘
```

---

### Mode 16: Semantic Focus Lens (NEW — FOUNDATIONAL)

**Problem it solves:** "I clicked a function and now I see the entire 500-node graph. Everything is
equally visible. I can't tell what matters."

**Who it's for:** Everyone, every time. This is the default rendering mode, not a feature toggle.

**Why it's foundational:** Every other mode benefits from focus-relative rendering. Architecture
Overview uses it at workspace level. Module Deep Read uses it at subsystem level. Call Chain
Explorer uses it at entity level. CFG Visualizer uses it at flow level.

**What the user sees:**

```
┌────────────────────────────────────────────────────────────────────┐
│  Focus: Consumer::poll()                                          │
│  Zoom level: Entity                                                │
│                                                                    │
│  ┌─ 1-hop (visible, ranked by PPR) ──────────────────────────┐    │
│  │                                                            │    │
│  │  ← callers                    callees →                    │    │
│  │  ┌──────────────────┐        ┌──────────────────┐          │    │
│  │  │ consumer_loop    │        │ msg_queue        │          │    │
│  │  │ ::run()          │───────►│ ::dequeue()      │          │    │
│  │  │ PPR: 0.18        │        │ PPR: 0.15        │          │    │
│  │  └──────────────────┘        └──────────────────┘          │    │
│  │  ┌──────────────────┐        ┌──────────────────┐          │    │
│  │  │ batch_processor  │        │ offset_tracker   │          │    │
│  │  │ ::flush()        │───────►│ ::advance()      │          │    │
│  │  │ PPR: 0.09        │        │ PPR: 0.12        │          │    │
│  │  └──────────────────┘        └──────────────────┘          │    │
│  │                                                            │    │
│  │  siblings (same impl block)                                │    │
│  │  ┌──────────────────┐  ┌──────────────────┐                │    │
│  │  │ Consumer::new()  │  │ Consumer::stop() │                │    │
│  │  │ PPR: 0.06        │  │ PPR: 0.04        │                │    │
│  │  └──────────────────┘  └──────────────────┘                │    │
│  └────────────────────────────────────────────────────────────┘    │
│                                                                    │
│  ┌─ 2-hop (faint) ───────────────────────────────────────────┐    │
│  │  ▒ Server::start() ▒  ▒ TopicPartition::read() ▒          │    │
│  │  ▒ MessageHandler::process() ▒                             │    │
│  └────────────────────────────────────────────────────────────┘    │
│                                                                    │
│  Boundary exits: ◇ "Streaming Pipeline" community                 │
│                  ◇ "Storage" community                             │
│                  ◇ "Server" community                              │
│                                                                    │
│  Everything else: ghosted ░░░░░░                                   │
└────────────────────────────────────────────────────────────────────┘
```

**Zoom level transitions:**
- **Workspace → Subsystem**: Click a community. Community entities become 1-hop. Other communities
  become boundary exits.
- **Subsystem → Entity**: Click an entity. Entity becomes focus. Community peers become 1-hop.
  Other communities ghost.
- **Entity → Flow**: Click "CFG" or "Borrows." The entity's internal structure (basic blocks, borrow
  scopes) fills the view. Callers/callees become boundary exits.
- **Any level → Up**: Click breadcrumb or press Escape. Re-center at parent zoom level.

**Key design rule:** The lens is not a filter. It doesn't hide things — it dims them. The user can
always see ghosted nodes and click them to re-center. The lens controls salience, not visibility.

---

### Mode 17: Architecture Variant Explorer (NEW — MOAT)

**Problem it solves:** "What would happen if we introduced an interface between A and B? How would
that change the architecture?"

**Who it's for:** Tech leads, architects, anyone making structural decisions.

**Why it's the moat:** No other code reading tool lets you explore architectural alternatives as
structured, comparable graph variants with computed consequences. This is the feature that makes
Parseltongue irreplaceable once adopted.

**What the user sees:**

```
┌──────────────────────────────────────────────────────────────────────┐
│  Architecture Variants                                               │
│                                                                      │
│  ┌─ BASE (current) ────────────────────────────────────────────┐    │
│  │  Server ──calls──► Consumer ──calls──► Storage              │    │
│  │  Server ──calls──► BatchProcessor                            │    │
│  │  BatchProcessor ──impls──► MessageHandler                    │    │
│  │                                                              │    │
│  │  Communities: 3 | Hotspot: Consumer | Cycles: 0              │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌─ VARIANT A: "Add Interface Layer" ──────────────────────────┐    │
│  │  + Server ──calls──► ConsumerAPI (new trait)    [proposed]   │    │
│  │  + ConsumerAPI ──calls──► Consumer              [proposed]   │    │
│  │  - Server ──calls──► Consumer                   [removed]    │    │
│  │                                                              │    │
│  │  Consequences:                                               │    │
│  │  • Consumer PageRank: 0.042 → 0.031 (▼ 26%)                │    │
│  │  • ConsumerAPI becomes new hotspot (PageRank: 0.038)        │    │
│  │  • Communities: 3 → 4 (ConsumerAPI creates boundary)        │    │
│  │  • Coupling: Server↔Consumer reduced by 1 direct edge       │    │
│  │  • No new cycles introduced ✓                               │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  ┌─ VARIANT B: "Merge Batch into Consumer" ────────────────────┐    │
│  │  - BatchProcessor (removed as separate entity)   [proposed] │    │
│  │  + Consumer ──impls──► MessageHandler             [proposed] │    │
│  │                                                              │    │
│  │  Consequences:                                               │    │
│  │  • Consumer PageRank: 0.042 → 0.058 (▲ 38% — more central) │    │
│  │  • k-core: Consumer moves from shell 6 → shell 8           │    │
│  │  • ⚠ Risk: Consumer becomes god-object (out-degree: 15→23) │    │
│  │  • Communities: 3 → 2 (batch absorbed into streaming)       │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  [Compare A vs B]  [Apply variant to map overlay]  [Ask LLM]        │
│                                                                      │
│  LLM: "Variant A decouples Server from Consumer, which is good      │
│  for testability but adds a new abstraction layer. Variant B         │
│  simplifies the dependency graph but risks making Consumer a         │
│  god-object. I'd recommend A if you plan to add more consumer        │
│  types, B if Consumer and BatchProcessor are always changed           │
│  together."                                                          │
└──────────────────────────────────────────────────────────────────────┘
```

**Trust rules:**
- Base graph edges = solid lines, labeled "truth" (compiler-verified)
- Variant edges = dotted lines, labeled "proposed"
- Every proposed edge requires: `kind` (typed), `rationale` (justified), `variant_id` (scoped)
- The LLM can suggest variants, but they are always marked as proposed, never merged into truth
- The consequence engine computes metrics on the hypothetical graph — it does not guess

---

## 4. Workflow Catalog (40 Workflows — 28 Upgraded + 7 Compiler + 5 Focus/Variant)

### Original 28 workflows carry forward with these upgrades:

**Workflows #4-5 (callers/callees):** Now use MIR-resolved edges. No ambiguity.
**Workflow #9-10 (trait impls):** Now show resolved dispatch sites per implementation.
**Workflow #11 (blast radius):** Exact impact because edges are exact.
**Workflow #16 (dead code):** MIR unreachable block detection supplements BFS reachability.
**Workflow #24 (data flow):** Upgraded from "call-chain level" to actual MIR data flow.
**Workflow #27 (error handling):** Now traces exact `?` propagation through MIR terminators.

### 7 New Workflows:

```
────────────────────────────────────────
#: 29
Workflow: View function control flow graph
Human Action/Prompt: Clicks "CFG" button on a function entity
LLM Role: Explains the branching structure — "This has 3 paths: happy,
  error, and timeout"
Parseltongue API Role: /entity/{id}/cfg
Journey Stage: DEEP DIVE
Visual/Textual: Visual (CFG diagram)
Required Data: MIR basic_blocks, successors, dominators
Why It Matters: Makes invisible branching structure visible. No other
  reading tool shows this.
────────────────────────────────────────
#: 30
Workflow: Trace a value through the code
Human Action/Prompt: Clicks a variable, selects "Trace data flow"
LLM Role: Narrates the journey — "This value is created on line 14,
  moved into the struct on line 25, and accessed via &self thereafter"
Parseltongue API Role: /entity/{id}/dataflow?variable={name}
Journey Stage: DEEP DIVE
Visual/Textual: Visual (highlighted path through source)
Required Data: MIR Place + Projection, Operand analysis
Why It Matters: Answers "where does this come from?" without reading
  every line.
────────────────────────────────────────
#: 31
Workflow: View borrow lifetimes for a function
Human Action/Prompt: Clicks "Borrows" button on a function
LLM Role: Explains why borrows overlap or don't — specific to THIS code
Parseltongue API Role: /entity/{id}/borrows
Journey Stage: DEEP DIVE
Visual/Textual: Visual (timeline of borrow scopes)
Required Data: Polonius loan_issued_at, origin_live_at, subset_base
Why It Matters: Makes the borrow checker's reasoning visible. #1 pain
  point for Rust newcomers.
────────────────────────────────────────
#: 32
Workflow: "Which implementation is called here?"
Human Action/Prompt: Clicks a trait method call in source pane
LLM Role: Explains the dispatch — "This calls Consumer::poll because
  the receiver is Consumer, resolved at compile time"
Parseltongue API Role: /resolve-dispatch?call_site={location}
Journey Stage: DEEP DIVE
Visual/Textual: Text (resolved target + explanation)
Required Data: Instance::try_resolve from MIR
Why It Matters: Eliminates the #1 source of confusion in trait-heavy
  Rust code.
────────────────────────────────────────
#: 33
Workflow: View async execution flow
Human Action/Prompt: Clicks an async function, selects "Async flow"
LLM Role: Explains await points and what happens between them
Parseltongue API Role: /entity/{id}/async-flow
Journey Stage: DEEP DIVE
Visual/Textual: Visual (await-point segmented CFG)
Required Data: MIR generator/async analysis, Yield terminators
Why It Matters: Async Rust is notoriously hard to reason about. This
  shows where the function suspends and resumes.
────────────────────────────────────────
#: 34
Workflow: View generic instantiations
Human Action/Prompt: Clicks a generic function, selects "Instantiations"
LLM Role: Explains which concrete types are used
Parseltongue API Role: /entity/{id}/monomorphizations
Journey Stage: DEEP DIVE
Visual/Textual: Visual (list of concrete instantiations with call sites)
Required Data: MIR instance_mir, monomorphization info
Why It Matters: "This function is called with 4 different type
  parameters" — shows the actual usage, not the abstract definition.
────────────────────────────────────────
#: 35
Workflow: "Why does this need unsafe?"
Human Action/Prompt: Clicks an unsafe block, asks LLM
LLM Role: Explains which specific operation requires unsafe and what
  invariant the programmer is asserting
Parseltongue API Role: /entity/{id}/unsafe-analysis
Journey Stage: DEEP DIVE
Visual/Textual: Text (specific unsafe operation + explanation)
Required Data: MIR unsafe block analysis, UnsafetyCheckResult
Why It Matters: Unsafe Rust is critical to understand in systems code.
  This shows exactly which line needs unsafe and why.
```

---

### 5 New Focus Lens + Variant Workflows:

```
────────────────────────────────────────
#: 36
Workflow: Focus on an entity (Semantic Focus Lens)
Human Action/Prompt: Clicks any entity on any surface
LLM Role: Silent — visual transition
Parseltongue API Role: /focus?entity={id}&depth=2
Journey Stage: ALL (this IS the navigation model)
Visual/Textual: Visual (focus lens re-render)
Required Data: PPR from selected node, BFS distance, global PageRank
Why It Matters: This is the foundational UX. Every click triggers a
  focus transition. Without it, the graph is noise.
────────────────────────────────────────
#: 37
Workflow: Zoom between abstraction levels
Human Action/Prompt: Clicks breadcrumb level or scrolls zoom control
LLM Role: Silent at workspace/subsystem/entity. Active at flow level
  ("This CFG has 3 branches...")
Parseltongue API Role: /focus?entity={id}&level=workspace|subsystem|entity|flow
Journey Stage: ALL
Visual/Textual: Visual (level transition animation)
Required Data: Containment hierarchy + PPR at each level
Why It Matters: Zoom = changing abstraction, not scaling pixels. The
  user moves between "big picture" and "inside one function" fluidly.
────────────────────────────────────────
#: 38
Workflow: Create an architecture variant
Human Action/Prompt: Clicks "New variant" → adds/removes edges via UI
  or asks LLM to propose one
LLM Role: Can propose a variant: "If you want to decouple Server from
  Consumer, I'd suggest adding a ConsumerAPI trait between them"
Parseltongue API Role: POST /variant { name, deltas: [{op, src, dst, kind, rationale}] }
Journey Stage: POST-DIVE (architecture analysis)
Visual/Textual: Visual (variant panel with delta list)
Required Data: Base graph + proposed delta operations
Why It Matters: Turns Parseltongue from "reading tool" to
  "architectural reasoning tool." The moat feature.
────────────────────────────────────────
#: 39
Workflow: Compare architecture variants
Human Action/Prompt: Selects 2 variants, clicks "Compare"
LLM Role: Narrates consequences — "Variant A reduces coupling but adds
  a new abstraction. Variant B simplifies but creates a god-object."
Parseltongue API Role: GET /variant/{id}/diff, GET /variant/compare?a={id}&b={id}
Journey Stage: POST-DIVE
Visual/Textual: Visual (side-by-side consequence tables) + Text (LLM)
Required Data: Consequence engine: PageRank delta, SCC changes, k-core
  delta, Leiden boundary changes, hotspot shifts
Why It Matters: "Which architecture option is better?" answered with
  data, not opinions. No other tool does this.
────────────────────────────────────────
#: 40
Workflow: Apply variant overlay to map
Human Action/Prompt: Toggles a variant on the architecture map
LLM Role: Silent
Parseltongue API Role: GET /communities?variant={id}&layout=true
Journey Stage: POST-DIVE
Visual/Textual: Visual (map with proposed edges shown as dotted lines,
  removed edges shown as strikethrough, consequence badges on affected
  entities)
Required Data: Base layout + variant delta overlay
Why It Matters: See the architectural change ON the map, not in a
  table. Visual impact assessment.
```

---

## 5. Top 10 UX Flows (Re-Ranked for v2)

1. **Semantic Focus Lens → Architecture Overview → Community Zoom → Read** (Focus lens is now THE
   navigation model. Every click triggers a focus transition. The architecture map is the first
   focus context. This is not a feature — it is the product.)
2. **Guided Tour** (upgraded with "Type System Tour" and "Ownership Tour")
3. **Call Chain Explorer** (MAJOR upgrade — exact chains, error path branches)
4. **Trait/Impl Browser with Dispatch Resolution** (NEW capability, killer Rust feature)
5. **"What Should I Read Next?"** (same algorithm, better graph, now uses PPR-relative ranking
   from the focus lens — suggestions are relative to WHERE you are, not globally)
6. **Ownership & Borrow Visualizer** (REBUILT — compiler-verified, not syntactic)
7. **"You Are Here" Navigator** (same — the focus lens subsumes this as the default state)
8. **Control Flow Graph Visualizer** (NEW — no other tool has this)
9. **"Explain This Module To Me"** (same, but LLM gets richer compiler context)
10. **Architecture Variant Explorer** (MOAT — compare architectural alternatives with computed
    consequences. Not top-5 because it requires a trustworthy base graph and focus lens first.)

**What moved up:** Focus Lens (new, #1) — it's the foundational UX, not a feature.
Call Chain Explorer (#8 → #3) because exact chains are transformative.
Trait/Impl Browser (#5 → #4) because dispatch resolution is a killer feature.
Ownership Visualizer (was "Later" → #6) because Polonius makes it real.

**What's new in top 10:** Semantic Focus Lens (#1), Architecture Variant Explorer (#10).

**Priority order for the two new ideas:**
1. Build the focus/zoom model first (it IS the navigation)
2. Build variant overlays second (it IS the moat)
3. Build "compare architecture options" views on top

---

## 6. Three-Level Interaction Model (Updated Layer 3)

Layers 1 (Human) and 2 (LLM) are unchanged. Layer 3 (APIs) gets compiler enrichment:

```
┌─────────────────────────────────────────────────────────────────┐
│  LAYER 3: APIs + Compiler Engine (UPDATED)                      │
│                                                                 │
│  ┌──────────┐  ┌────────────┐  ┌───────────┐  ┌─────────────┐  │
│  │  SQLite   │  │  rustc_    │  │   Graph   │  │   Source     │  │
│  │  Store    │  │  private   │  │ Algorithms│  │  from Disk   │  │
│  │          │  │  + MIR     │  │ (igraph)  │  │             │  │
│  │ entities │  │ call graph │  │ Leiden    │  │ always fresh │  │
│  │ edges    │  │ CFG        │  │ PageRank  │  │             │  │
│  │ history  │  │ types      │  │ PPR       │  │             │  │
│  │ tours    │  │ Polonius   │  │ k-core    │  │             │  │
│  └──────────┘  └────────────┘  └───────────┘  └─────────────┘  │
│       ▲              │              ▲              ▲            │
│       │              │              │              │            │
│       │    ┌─────────▼──────────┐   │              │            │
│       │    │  Indexing Pipeline  │   │              │            │
│       │    │  (runs once, ~30s)  │───┘              │            │
│       │    │                    │                   │            │
│       │    │  1. rustc_driver   │                   │            │
│       │    │  2. extract MIR    │                   │            │
│       │    │  3. build edges    │                   │            │
│       │    │  4. run Leiden     │                   │            │
│       │    │  5. compute ranks  │                   │            │
│       │    │  6. store in SQLite│                   │            │
│       │    └────────────────────┘                   │            │
│       │                                            │            │
│       └──── query-time ◄───────────────────────────┘            │
│              (reads from SQLite + disk)                          │
└─────────────────────────────────────────────────────────────────┘
```

**Indexing pipeline detail:**

```
Rust source (.rs files)
        │
        ▼
┌───────────────────────────────────────┐
│  rustc_driver::run_compiler()         │
│  with CodeGraphCallbacks              │
│  (pinned nightly: 2025-03-01)         │
├───────────────────────────────────────┤
│                                       │
│  Phase 1: after_analysis(tcx)         │
│  ├─ tcx.hir_body_owners()            │  ← iterate all functions
│  ├─ tcx.fn_sig(def_id)               │  ← signatures
│  ├─ tcx.visibility(def_id)           │  ← pub/crate/private
│  ├─ tcx.type_of(def_id)              │  ← types
│  ├─ tcx.generics_of(def_id)          │  ← generic params
│  ├─ tcx.trait_impls_of(def_id)       │  ← all impls of a trait
│  └─ tcx.def_path_str(def_id)         │  ← qualified name
│                                       │
│  Phase 2: MIR extraction              │
│  ├─ tcx.optimized_mir(def_id)        │  ← get CFG
│  ├─ body.basic_blocks                │  ← all blocks
│  ├─ TerminatorKind::Call { func, .. }│  ← extract calls
│  ├─ Instance::try_resolve()          │  ← resolve dispatch
│  ├─ BasicBlocks::dominators()        │  ← loop detection
│  └─ TerminatorKind::Drop             │  ← drop glue
│                                       │
│  Phase 3: Polonius (optional)         │
│  ├─ loan_issued_at                   │  ← borrow origins
│  ├─ origin_live_at                   │  ← liveness
│  ├─ path_moved_at_base              │  ← moves
│  └─ subset_base                      │  ← lifetime outlives
│                                       │
└───────┬───────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────┐
│  SQLite: entities + edges + CFG       │
│  + Leiden communities + PageRank      │
│  + Polonius facts (per-function)      │
└───────────────────────────────────────┘
```

---

## 7. Visual System (14 Surfaces — 10 Upgraded + 2 Compiler + 2 Foundational)

Surfaces 1-10 from v1 carry forward. Two new surfaces:

### Surface 11: Control Flow Graph Panel

**What it shows:** The MIR CFG for the currently-viewed function, rendered as a node-edge diagram.
Basic blocks are boxes. Edges are typed (branch, call, drop, return, panic). The dominator tree is
available as an overlay. Loop back-edges are highlighted.

**When it appears:** When the user clicks "CFG" on any function entity.

**APIs:** `/entity/{id}/cfg` returns `{ blocks: [{id, statements, terminator_kind}], edges: [{from, to, kind}], dominators: [{block, idom}] }`.

**Useful vs decorative:** Useful for functions with complex branching (>3 match arms, nested
if/else, loops). Decorative for simple linear functions. Mitigation: Only show the CFG button for
functions with >3 basic blocks.

### Surface 12: Borrow Timeline Panel

**What it shows:** A vertical timeline showing all active borrows at each point in a function.
Color-coded: blue for shared (`&`), red for mutable (`&mut`), with overlap warnings.

**When it appears:** When the user clicks "Borrows" on any function entity.

**APIs:** `/entity/{id}/borrows` returns `{ loans: [{id, kind, start_point, end_point, place}], conflicts: [{loan_a, loan_b, point, reason}] }`.

**Useful vs decorative:** Extremely useful for functions with borrow checker errors or complex
lifetime patterns. Less useful for simple functions. Mitigation: Show a "complexity badge" on
entities where Polonius detects >2 overlapping loans.

---

### Surface 13: Focus Lens Renderer (NEW — powers all surfaces)

**What it shows:** Not a separate panel — this is the rendering model applied to every visual
surface. When the user selects any entity, the renderer applies the focus lens: selected = full
saturation, 1-hop = visible + ranked by PPR, 2-hop = faint, unrelated = ghosted, boundary nodes
= exit portals.

**When it appears:** Always. This is the default rendering behavior.

**APIs:** `/focus?entity={id}&depth=2` returns `{ focus: {id, metrics}, ring_1: [{id, name, ppr_score, bfs_distance, edge_kind}], ring_2: [{id, name, ppr_score}], boundaries: [{id, name, community}], ghosted_count: int }`.

**Useful vs decorative:** Definitionally useful — without it, every graph surface is an
unreadable hairball. The focus lens is what makes the graph legible.

### Surface 14: Variant Overlay Panel

**What it shows:** A split or tabbed view showing the base graph alongside one or more variant
overlays. Proposed edges are dotted lines. Removed edges are struck through. Affected entities show
consequence badges (PageRank delta, k-core shift, new/broken cycles).

**When it appears:** When the user creates or selects a variant from the Architecture Variant
Explorer (Mode 17).

**APIs:** `/variant/{id}/diff` returns `{ added_edges: [...], removed_edges: [...], changed_edges: [...], consequences: { pagerank_delta: [...], scc_changes: [...], kcore_delta: [...], community_changes: [...], hotspot_shifts: [...] } }`.

**Useful vs decorative:** Extremely useful for architecture decision-making. Decorative if the user
never makes architectural changes. Mitigation: This is a v1.3 feature — by then the user trusts
the base graph and wants to explore alternatives.

---

## 8. API Consequences (Updated)

All v1 APIs remain. New endpoints:

### COMPILER-ENRICHED

```
GET /entity/{id}/cfg
  Returns: { blocks: [{id, stmts: [string], terminator: {kind, targets}}],
             edges: [{from, to, kind: "branch"|"call"|"drop"|"return"|"panic"}],
             dominators: [{block, idom}],
             back_edges: [{from, to}] }
  Source: tcx.optimized_mir(def_id).basic_blocks
  Precomputed at index time. ~200-2000 tokens depending on function complexity.

GET /entity/{id}/borrows
  Returns: { loans: [{id, kind: "&"|"&mut", place, start_line, end_line}],
             moves: [{place, line}],
             drops: [{place, line}],
             conflicts: [{loan_a, loan_b, line, reason}] }
  Source: Polonius facts. Precomputed at index time. ~100-500 tokens.

GET /entity/{id}/dataflow?variable={name}
  Returns: { definitions: [{line, kind}], uses: [{line, kind}],
             moves: [{line, target}], drops: [{line}] }
  Source: MIR Place + Projection analysis. Query-time. ~50-200 tokens.

GET /resolve-dispatch?call_site={file}:{line}
  Returns: { trait_name, method_name, resolved_impl: {type, def_id, location},
             dispatch_kind: "static"|"virtual"|"closure",
             confidence: "exact"|"dynamic_N_candidates",
             all_candidates: [{type, location}] }
  Source: Instance::try_resolve(). Precomputed for all call sites. ~80 tokens.

GET /entity/{id}/async-flow
  Returns: { await_points: [{line, resumed_at}],
             state_machine: [{state, entry_block, yield_block}] }
  Source: MIR generator analysis. Precomputed. ~100-300 tokens.

GET /entity/{id}/monomorphizations
  Returns: [{ concrete_types: {T: "Consumer", U: "String"},
              call_sites: [{file, line}],
              count: 5 }]
  Source: MIR instance enumeration. Precomputed. ~50 tokens per instantiation.

GET /entity/{id}/unsafe-analysis
  Returns: { unsafe_blocks: [{start_line, end_line,
              operations: [{kind: "raw_ptr_deref"|"ffi_call"|"union_access",
                            line, explanation}] }] }
  Source: MIR UnsafetyCheckResult. Precomputed. ~50-200 tokens.
```

### BOUNDARY QUERIES (typed aggregation)

```
GET /boundary/{id}
  Returns: { boundary: {id, name, type: "crate"|"module"|"folder",
                         parent_id, depth, path},
             metrics: {entity_count, pub_surface, internal_edges,
                       outgoing_edges, incoming_edges, cohesion,
                       coupling_out, coupling_in, fan_in, fan_out,
                       is_facade},
             depends_on: [
               { boundary: string, type: "crate"|"module"|"folder",
                 crossing: "CROSS-CRATE"|"INTRA-CRATE"|"INTRA-MODULE",
                 edges: int, file_pairs: int,
                 distinct_items: int, import_breadth: float,
                 kinds: [string] }
             ],
             depended_on_by: [
               { boundary: string, type: string,
                 crossing: string,
                 edges: int, file_pairs: int }
             ],
             public_surface: [
               { entity: string, kind: string, callers_outside: int }
             ] }
  Source: boundaries + boundary_edges tables. Precomputed. ~300-1000 tokens.
  THIS is the packet the LLM needs for architectural reasoning.

GET /boundary/{id}?expand=true
  Returns: same as above, plus:
    children: [
      { id: string, type: "module"|"folder",
        entity_count: int, cohesion: float,
        coupling_in: float, coupling_out: float }
    ]
  Shows the internal structure of a boundary.

GET /boundary/{id}?variant={variant_id}
  Same as above, metrics recomputed on base_edges + variant deltas.

GET /boundary/coupling?a={id}&b={id}
  Returns: { a_to_b: { crossing: "CROSS-CRATE"|"INTRA-CRATE",
                        edges: int, file_pairs: int,
                        distinct_items: int, import_breadth: float,
                        kinds: [string],
                        entities: [{src: string, dst: string, kind: string}] },
             b_to_a: { ... },
             total_coupling: int,
             shared_traits: [string],
             cost_of_change: "HIGH"|"LOW"|"TRIVIAL" }
  cost_of_change derived from crossing type:
    CROSS-CRATE = HIGH, INTRA-CRATE = LOW, INTRA-MODULE = TRIVIAL.
  Query-time. ~100-400 tokens.

GET /boundary/coupling?a={id}&b={id}&variant={variant_id}
  Same coupling query on the variant graph.

GET /boundary/symmetry?parent={id}
  Returns: [{ group: [string],
              shared_deps: [string],
              dep_profile_hash: string }]
  Detects child boundaries with identical dependency profiles.
  Pure SQL GROUP BY + HAVING. ~50-200 tokens.

GET /boundary/tree
  Returns: full boundary hierarchy as a nested tree.
  { id: string, type: "workspace", children: [
    { id: string, type: "crate", children: [
      { id: string, type: "module", children: [...] },
      ...
    ]},
    ...
  ]}
  Powers the breadcrumb trail and zoom level navigation.
```

### ENTITY QUERIES (unchanged, kept for completeness)

```
GET /deps/entity?id={id}
  Returns: { entity: {id, name, kind, file_path, visibility},
             calls: [{id, name, file_path, dispatch_kind}],
             called_by: [{id, name, file_path, dispatch_kind}],
             impls: [{id, name}], traits: [{id, name}],
             boundary: {id, type, parent_id} }
  Source: filter edges by src_id/dst_id. Query-time. ~100-500 tokens.
  Now includes which boundary this entity belongs to.
```

### FOCUS LENS

```
GET /focus?entity={id}&depth=2
  Returns: { focus: {id, name, metrics},
             ring_1: [{id, name, ppr_score, bfs_distance, edge_kind,
                        global_pagerank}],
             ring_2: [{id, name, ppr_score, bfs_distance}],
             boundaries: [{id, name, community_name, exit_edge_kind}],
             ghosted_count: int,
             zoom_level: "workspace"|"subsystem"|"entity"|"flow" }
  Source: PPR from focus node, BFS distance, global PageRank, edge type
  weights. Query-time PPR + precomputed PageRank. ~100-500 tokens.

  Ranking within each ring:
    1. ppr_score (local relevance — highest weight)
    2. bfs_distance (structural proximity — tiebreaker)
    3. global_pagerank (importance — second tiebreaker)
    4. edge_kind weight: calls=1.0, impls=0.9, type_refs=0.7,
       contains=0.5, public_boundary=0.8

GET /focus?entity={id}&level=flow
  Returns: { focus: {id, name},
             cfg: { blocks: [...], edges: [...] },
             borrows: { loans: [...], conflicts: [...] },
             exit_boundaries: [{id, name, edge_kind}] }
  The "flow level" — zooms INTO the entity's internal structure (CFG,
  borrows) and treats callers/callees as boundary exits.
```

### VARIANT GRAPH OVERLAYS

```
POST /variant
  Body: { name: "Add Interface Layer",
          deltas: [
            { op: "add_edge", src: "server::handle", dst: "consumer_api::process",
              kind: "calls", rationale: "Decouple Server from Consumer" },
            { op: "remove_edge", src: "server::handle", dst: "consumer::poll",
              kind: "calls", rationale: "Replace direct call with interface" },
            { op: "add_edge", src: "consumer_api::process", dst: "consumer::poll",
              kind: "calls", rationale: "Interface delegates to impl" }
          ] }
  Returns: { variant_id, name, delta_count, status: "created" }

GET /variant/{id}
  Returns: { id, name, deltas: [...], created_at, status }

GET /variant/{id}/diff
  Returns: { added_edges: [{src, dst, kind, rationale}],
             removed_edges: [{src, dst, kind, rationale}],
             changed_edges: [{src, dst, old_kind, new_kind, rationale}],
             consequences: {
               pagerank_delta: [{entity, old, new, change_pct}],
               scc_changes: { new_cycles: [...], broken_cycles: [...] },
               kcore_delta: [{entity, old_shell, new_shell}],
               community_changes: [{entity, old_community, new_community}],
               hotspot_shifts: [{entity, old_rank, new_rank}],
               public_boundary_changes: [{entity, old_visibility, new_exposure}]
             } }
  Source: Recompute Leiden, PageRank, k-core, SCC on base_graph + delta.
  Cached per variant. ~500-2000 tokens.

GET /variant/compare?a={id}&b={id}
  Returns: { variant_a: {name, consequences_summary},
             variant_b: {name, consequences_summary},
             comparative: {
               which_adds_more_coupling: "a"|"b",
               which_creates_cycles: "a"|"b"|"neither",
               which_changes_hotspots_more: "a"|"b",
               which_splits_communities: "a"|"b"|"neither"
             } }

DELETE /variant/{id}
  Removes a variant and its cached consequences.

# Querying any existing API with a variant overlay:
GET /communities?variant={id}&layout=true
GET /hotspots?variant={id}&top=50
GET /entity/{id}/context?variant={id}
  All existing APIs accept an optional ?variant= parameter.
  When present, they compute results on base_graph + variant_delta.
  Proposed edges are tagged with { proposed: true, variant_id }.
```

### UPDATED ENDPOINTS

```
GET /entity/{id}/annotations (UPDATED)
  Returns: { callers: [{id, name, location, dispatch_kind, confidence}],
             callees: [{id, name, location, dispatch_kind, confidence}],
             impls: [{id, name, dispatch_sites: int}],
             traits: [{id, name}],
             type_info: {signature, generics, visibility} }
  Now includes dispatch_kind and confidence per edge.
  Source: MIR + Instance::try_resolve. Precomputed.

GET /entity/{id}/context (UPDATED)
  Returns: { entity: {..., type_info, cfg_complexity},
             callers: [..., with dispatch info],
             callees: [..., with dispatch info],
             ownership_summary: {borrows: int, moves: int, unsafe_blocks: int},
             community: {...},
             containment: [...],
             metrics: {...} }
  Adds type_info, cfg_complexity, and ownership_summary to LLM context.
```

---

## 9. Algorithm-to-Experience Mapping (Updated)

All v1 algorithms remain. New compiler-derived analyses:

```
────────────────────────────────────────
Analysis: MIR CFG extraction
Reading/Browsing Experience: Control flow graph visualization, loop detection,
  unreachable code identification, async await-point mapping
Compute Mode: Precomputed (at index time via rustc_private)
Library: rustc_private (optimized_mir, basic_blocks, dominators)
Expected Data Shape: BasicBlock DAG + Terminator types
Bad UX If Misused: Showing CFG for a 2-line function is noise. Gate: only show
  for functions with >3 basic blocks. For functions with >30 blocks, collapse
  non-branching chains into single nodes.
────────────────────────────────────────
Analysis: Instance::try_resolve (dispatch resolution)
Reading/Browsing Experience: Exact call targets in call chains, trait/impl
  browser call-site column, "which implementation?" workflow
Compute Mode: Precomputed (at index time)
Library: rustc_private (Instance::try_resolve)
Expected Data Shape: Call site → resolved DefId + dispatch kind
Bad UX If Misused: Dynamic dispatch (dyn Trait) returns Virtual — show all
  candidates, don't pretend it's resolved. Label clearly.
────────────────────────────────────────
Analysis: Polonius borrow facts
Reading/Browsing Experience: Ownership visualizer, borrow timeline, lifetime
  scope viewer, conflict explanation
Compute Mode: Precomputed (at index time, optional — adds ~5s to indexing)
Library: rustc_private (Polonius facts via -Znll-facts or in-process)
Expected Data Shape: (origin, loan, point) tuples
Bad UX If Misused: Raw Polonius facts are incomprehensible to users. MUST
  translate to line ranges and plain-English conflict descriptions. Never show
  raw point/origin IDs.
────────────────────────────────────────
Analysis: MIR dataflow (initialization + liveness)
Reading/Browsing Experience: "Where does this value come from?" trace, move
  detection, "why is this value unavailable after line X?"
Compute Mode: Query-time (fast — linear in function size)
Library: rustc_private (MaybeInitializedPlaces, MaybeLiveLocals)
Expected Data Shape: Bitsets per BasicBlock (initialized/live locals)
Bad UX If Misused: Dataflow results are per-basic-block, not per-line. Must
  interpolate to source lines. Show at statement granularity, not block.
────────────────────────────────────────
Analysis: Type query (fn_sig, type_of, generics_of)
Reading/Browsing Experience: Resolved signatures in entity lists, type
  comparison in side-by-side mode, generic instantiation viewer
Compute Mode: Precomputed (at index time)
Library: rustc_private (TyCtxt queries)
Expected Data Shape: Ty<'tcx> serialized to human-readable strings
Bad UX If Misused: Fully-qualified type names can be very long
  (std::collections::HashMap<String, Vec<Arc<Mutex<Consumer<T>>>>>). Truncate
  to last segment with tooltip for full path.
```

---

```
────────────────────────────────────────
Analysis: Focus-Relative Ranking (Semantic Focus Lens)
Reading/Browsing Experience: Every visual surface. Determines what is
  saturated, visible, faint, or ghosted at every zoom level.
Compute Mode: Query-time (PPR is the bottleneck, ~10-50ms per focus change)
Library: python-igraph (personalized_pagerank) + precomputed global PageRank
Expected Data Shape: PPR vector from focus node + BFS distance + global
  PageRank + edge type weights
Bad UX If Misused: If PPR is too slow, the focus transition feels laggy.
  Pre-cache PPR for the top-20 entities per community. For cold entities,
  compute on click and show a 50ms transition animation to mask latency.
  If ranking treats all edge types equally, type_ref edges drown out call
  edges. Weight: calls=1.0, impls=0.9, public_boundary=0.8, type_refs=0.7,
  contains=0.5.
────────────────────────────────────────
Analysis: Variant Consequence Engine (Graph Overlays)
Reading/Browsing Experience: Architecture variant comparison, "what-if"
  analysis, consequence tables, variant overlay on maps
Compute Mode: On-demand (recompute when variant is created or modified)
Library: python-igraph (Leiden, PageRank, SCC, k-core on modified graph)
Expected Data Shape: Base graph + delta operations → modified graph →
  recomputed metrics → diff against base metrics
Bad UX If Misused: If consequence computation is slow (>5s), the variant
  explorer feels broken. Mitigation: cache consequences per variant. Only
  recompute when the variant is modified. For large graphs (>50K entities),
  approximate PageRank delta by only recomputing in the affected subgraph.
  CRITICAL: never show variant consequences as truth. Always label as
  "projected" or "hypothetical."
```

---

## 10. LLM-Guided Reading Patterns (Updated)

All 6 v1 patterns carry forward. The LLM now receives richer context:

### Updated LLM Context Packet

```
Entity: stream_processor::poll_next
Location: src/streaming/processor.rs:142:198
Signature: fn poll_next(&mut self) -> Poll<Option<Message>>   ← FROM COMPILER
Type info: &mut self = &mut StreamProcessor                   ← FROM COMPILER

Callers (RESOLVED):
  - consumer_loop::run (STATIC dispatch, called 3x)           ← EXACT
  - batch_processor::flush (STATIC dispatch, called 1x)       ← EXACT

Callees (RESOLVED):
  - message_queue::dequeue (STATIC, line 155)                  ← EXACT
  - offset_tracker::advance (STATIC, line 168)                 ← EXACT
  - MessageHandler::process (DYNAMIC dyn dispatch, line 172)   ← 3 candidates

CFG: 6 basic blocks, 1 loop (back-edge bb4→bb2), 1 error path (bb3→bb5)
Borrows: 2 loans active (L1: &mut self lines 142-198, L2: &msg lines 155-170)
Community: "Streaming Pipeline" (12 entities)
```

The LLM can now say: "This function has a loop (the poll retry on line 160) and an error branch
(the `?` on line 155 jumps to cleanup). The `&msg` borrow on line 155 prevents you from calling
`self.advance()` until after line 170 — that's why the offset update is at the bottom."

### New Pattern 7: Ownership Narrator

**Behavior:** When the user views the borrow timeline, the LLM explains WHY borrows overlap or
conflict, specific to this code.

```
LLM: "The borrow checker rejects line 13 because `self.queue` is borrowed
immutably on line 10 (for iteration), but `self.process()` on line 13
needs `&mut self`. These two borrows overlap on lines 10-14.

Fix: collect the items first with `.cloned().collect::<Vec<_>>()` on line 10,
then the shared borrow ends and you're free to call `&mut self` methods."
```

### New Pattern 8: Dispatch Explainer

**Behavior:** When the user clicks a trait method call, the LLM explains the dispatch mechanism.

```
LLM: "This calls `Consumer::poll_next` — resolved by the compiler because
the receiver type is `Consumer` (not `dyn Stream`). The compiler
monomorphized this call, so there's zero runtime overhead.

If you see a dashed line on a different call, that means dynamic dispatch
via a trait object — the compiler can't resolve it at compile time."
```

---

### New Pattern 9: Architecture Advisor (the pattern that makes variants a moat)

**Behavior:** The LLM operates as an architectural reasoning agent — not by pattern-matching
against training data, but by proposing structured graph variants and reading the consequence
engine's output. This is the pattern that gives LLMs what they've never had: the ability to reason
about public interface boundaries with real metrics.

**The workflow:**
1. Human asks an architectural question ("Should I decouple Server from Consumer?")
2. LLM proposes 2-3 variants via `POST /variant` (structured deltas, not prose)
3. Consequence engine computes metrics on each variant
4. LLM receives difference packets (PageRank delta, coupling changes, community shifts)
5. LLM narrates the trade-offs using **measured consequences**, not analogies

```
Human: "Should I add a trait boundary between Server and Consumer?"

LLM (internally): Creates two variants via API:
  Variant A: + Server→ConsumerAPI, + ConsumerAPI→Consumer, - Server→Consumer
  Variant B: keep current, but + Consumer: pub(crate) → pub

Consequence engine returns:
  Variant A: coupling -26%, new community boundary, 0 cycles, +1 abstraction
  Variant B: coupling unchanged, public surface +3 methods, 0 new dependencies

LLM: "I created two variants to compare:

Variant A (add ConsumerAPI trait):
  - Server→Consumer coupling drops from 8 edges to 1
  - Consumer's PageRank drops 26% (more isolated, less central)
  - Creates a new community boundary between server and streaming
  - Cost: 1 new trait, 1 impl block

Variant B (widen Consumer's visibility):
  - No structural change — just visibility
  - Public surface grows by 3 methods
  - Coupling unchanged

Recommendation: A if you plan multiple Consumer implementations or want
to test Server independently. B if Consumer is the only implementation
and you just need cross-module access."
```

**Why this is different from what LLMs do today:** Today, the LLM says "it depends on your use
case" because it has no way to compute coupling, centrality, or community structure. With variants,
it goes from *reasoning by analogy* to *reasoning by measurement*. The consequence data is not
hallucinated — it's computed on the actual graph.

### New Pattern 10: Focus Context Narrator

**Behavior:** When the focus lens transitions, the LLM receives the focus context (what's in
ring 1, what's in ring 2, what's at the boundary) and generates a 1-sentence orientation.

```
LLM: "You're looking at Consumer::poll. Its most important neighbor is
consumer_loop::run (calls it 3x). The boundary to the Storage community
is through offset_tracker::advance. You've already read the caller —
read the offset tracker next to understand how progress is persisted."
```

This combines the focus lens ranking with reading history to produce navigation guidance that
is **relative to where you are**, not globally generic.

---

## 11. Risks and Failure Modes (Updated)

### Eliminated risks:

**"Wrong edges" — ELIMINATED.** MIR edges are compiler-verified. The only remaining ambiguity is
`dyn Trait` dynamic dispatch, which is honestly labeled.

### Updated risks:

```
────────────────────────────────────────
Risk: Slow indexing
What Goes Wrong: rustc_private compilation takes 60+ seconds for large
  codebases (100K+ lines). User thinks the app is frozen.
How It Manifests: User gives up during initial indexing.
Mitigation: Progress bar showing "Compiling... Extracting MIR... Building
  graph... Computing communities..." with estimated time. Background indexing
  with partial results available early. Show file count and function count as
  they're discovered.
────────────────────────────────────────
Risk: Nightly Rust version mismatch
What Goes Wrong: User's project requires a different nightly than the pinned
  one. Compilation fails.
How It Manifests: "Compilation failed" error on drag-and-drop.
Mitigation: Pin a widely-compatible nightly. If compilation fails, fall back
  to tree-sitter-only mode with a banner: "Compiler analysis unavailable.
  Using syntax-only mode. Edges may be approximate." Tree-sitter is the
  degraded-but-functional fallback.
────────────────────────────────────────
Risk: Polonius facts overwhelm the UI
What Goes Wrong: A complex function has 50 active borrows. The borrow timeline
  is unreadable.
How It Manifests: User sees a wall of colored lines and gives up.
Mitigation: Collapse borrows by scope. Default view shows top-5 borrows by
  lifetime length. Expand on click. Only show conflicts by default — borrows
  that don't conflict are dimmed.
────────────────────────────────────────
Risk: CFG is too complex
What Goes Wrong: A function with 40 basic blocks produces an unreadable graph.
How It Manifests: User sees spaghetti and closes the panel.
Mitigation: Collapse non-branching chains (bb1→bb2→bb3 where each has one
  successor) into single nodes. Default max display: 15 nodes. Expand on click.
  For functions with >50 blocks, show dominator tree instead of full CFG.
────────────────────────────────────────
Risk: Rust-only limits market
What Goes Wrong: Users want to analyze Python/Go/JS projects.
How It Manifests: Feature requests for other languages.
Mitigation: This is a feature, not a bug. The product thesis is "compiler truth
  for Rust." Tree-sitter multi-language is a different product. Own the niche.
  "The best Rust code reading tool" > "a mediocre multi-language tool."
```

```
────────────────────────────────────────
Risk: Focus lens feels like tunnel vision
What Goes Wrong: User can only see the focused entity's neighborhood.
  They lose awareness of the broader codebase.
How It Manifests: User complains "I can't see the big picture anymore."
Mitigation: Ghosted nodes are always visible and clickable. Breadcrumbs
  show the containment hierarchy. A "zoom out" button (or Escape key)
  immediately re-centers at the parent level. The lens controls salience,
  not visibility — nothing is truly hidden, just dimmed.
────────────────────────────────────────
Risk: PPR latency breaks focus transitions
What Goes Wrong: Computing PPR from a cold node takes >200ms. The focus
  transition feels sluggish.
How It Manifests: User clicks an entity and sees a lag before the
  neighborhood renders.
Mitigation: Pre-cache PPR for the top-20 entities per community (~80%
  of clicks). For cold entities, show the entity + its 1-hop BFS neighbors
  immediately (from precomputed edge list, <10ms), then replace with
  PPR-ranked ordering when it arrives (~50ms later). The user sees
  content instantly; ranking refines after a beat.
────────────────────────────────────────
Risk: Variant overlays become fiction
What Goes Wrong: User (or LLM) creates variants with nonsensical edges.
  The consequence engine dutifully computes metrics on garbage input.
How It Manifests: User sees "removing all edges makes everything a
  periphery node" — technically correct, useless.
Mitigation: Variants must use typed edges (calls, impls, type_refs —
  not "relates_to"). Each delta requires a rationale field. The UI shows
  a "validity score" based on whether proposed edges connect entities that
  are in the same or adjacent communities. LLM-proposed variants are
  reviewed before creation. Clearly mark everything as "proposed."
────────────────────────────────────────
Risk: Too many variants create confusion
What Goes Wrong: User creates 10 variants and can't remember what each
  one represents.
How It Manifests: Variant panel becomes a junk drawer.
Mitigation: Limit to 5 active variants per workspace. Each variant has
  a required name and description. Archive (don't delete) old variants.
  Show a 1-line summary of each variant's key consequence on the list
  view.
```

All v1 risks (too much text, fake visual novelty, stale snapshot, bad tours, overwhelming
newcomers, LLM over-explaining, meaningless hotspots, slow LLM, reading pressure) remain
unchanged.

---

## 12. Recommended Build Order (Updated)

### Alternative Build Order: Interface-and-Variant-First

This build order prioritizes getting the dependency graph queryable for LLMs before building
the full visual experience. It starts with tree-sitter edges (available now) and upgrades to
MIR later — same schema, same queries, better data.

```
┌──────────────────────────────────────────────────────────────────────┐
│  INTERFACE-AND-VARIANT-FIRST BUILD ORDER                             │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Phase 0: Get the edge table (foundation)                            │
│  ─────────────────────────────────────────                           │
│  • tree-sitter extracts entities + edges (already exists in pt01)    │
│  • Store as SQLite (entities table + edges table)                    │
│  • Enrich with file_path, dirname for L2/L3 aggregation             │
│  • This is the foundation. Everything else queries these tables.     │
│  • Later: swap tree-sitter edges for MIR edges (same schema!)       │
│                                                                      │
│  Phase 1: Dependency query interface for LLMs                        │
│  ────────────────────────────────────────────                        │
│  • HTTP API wrapping Polars/SQL queries on the two tables            │
│  • /deps/folder?path= → entities + incoming/outgoing at L3          │
│  • /deps/file?path= → file-level dependencies at L2                 │
│  • /deps/coupling?a=&b= → edge count between two paths              │
│  • /deps/entity?id= → callers + callees at L1                       │
│  • No graph algorithms. Just filter/join/group_by.                   │
│  • The LLM can now query: "What does {crate}/ depend on?"          │
│    and get a structured JSON answer.                                 │
│                                                                      │
│  Phase 2: Variant overlays                                           │
│  ─────────────────────────                                           │
│  • POST /variant → create a named delta (add/remove edges)          │
│  • All /deps/ queries accept ?variant= parameter                    │
│  • GET /variant/{id}/diff → what changed at L3 level                │
│  • LLM can now: propose variant, query consequences, compare        │
│  • Still no graph algorithms. Just relational queries on modified    │
│    edge tables.                                                      │
│                                                                      │
│  Phase 3: Graph algorithms for ranking + visualization               │
│  ─────────────────────────────────────────────────────               │
│  • Leiden communities (for the architecture map)                     │
│  • PageRank (for "most important" ranking)                           │
│  • PPR (for the semantic focus lens)                                 │
│  • SCC (for cycle detection — "did this variant create a loop?")    │
│  • These ENRICH the interface. They don't replace it.               │
│                                                                      │
│  Phase 4: Swap tree-sitter → MIR (when ready)                       │
│  ─────────────────────────────────────────────                       │
│  • Same schema. Same queries. Better edges.                          │
│  • dispatch_kind and confidence columns populated by compiler       │
│  • The interface doesn't change. The data quality does.              │
│                                                                      │
│  Phase 5: Full visual experience (Tauri + focus lens + surfaces)     │
│  ────────────────────────────────────────────────────────────        │
│  • The v1/v1.1/v1.2/v1.3 milestones below, built on the now-proven │
│    query interface and variant system.                               │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

---

### Original Build Order (Visual-First)

The following build order assumes the full Tauri visual experience is the priority.
The Interface-and-Variant-First order above can be used instead if LLM-native querying
is prioritized.

### v1 — "I Can Browse This Codebase" (compiler-powered from day 1)

User feels: "I opened a Rust codebase and within 30 seconds I can see its structure with
compiler-verified accuracy."

```
┌──────────────────────────────────────────────────────┬─────────────────────────┐
│                    What ships                        │   Experience enabled    │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Tauri app shell with workspace drag-and-drop         │ Entry point             │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ rustc_private indexing pipeline (MIR extraction)     │ Compiler truth          │
│ with tree-sitter fallback if compilation fails       │ foundation              │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ SQLite schema: entities, edges (with dispatch_kind), │ Storage                 │
│ CFG blocks, files, snapshots                         │                         │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Architecture overview map (Leiden on compiler graph)  │ First-open orientation  │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Community zoom → entity list (PageRank, resolved     │ Browse by concept       │
│ signatures from fn_sig)                              │                         │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Source pane with RESOLVED caller/callee chips        │ Clickable code, zero    │
│ (solid = static, dashed = dynamic)                   │ ambiguity navigation    │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Breadcrumb trail (containment chain)                 │ "You are here"          │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Semantic Focus Lens renderer (PPR + BFS + PageRank   │ THE navigation model.   │
│ ranking, 1-hop visible, 2-hop faint, ghosted rest)   │ Every click triggers it.│
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ 4-level zoom: workspace → subsystem → entity → flow │ Abstraction-level       │
│                                                      │ navigation              │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Neighborhood mini-map (ego network, PPR)             │ Spatial context         │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Search bar (RRF: FTS5 + trie + trigram)              │ Find things by name     │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Dispatch resolution on click ("calls Consumer::poll  │ Rust-specific killer    │
│ via Stream trait, static dispatch")                   │ feature from day 1      │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Hotspot overlay toggle                               │ Where to focus reading  │
└──────────────────────────────────────────────────────┴─────────────────────────┘
```

v1 endpoints: 14 (v1 original 10 + /resolve-dispatch + /entity/{id}/cfg + /focus + /focus?level=flow)

### v1.1 — "The App Guides Me" (same as v1 original)

Same as v1 original v1.1. LLM companion, guided tours, suggestions, history, bookmarks.
+7 endpoints.

### v1.2 — "I Can See the Compiler's View" (NEW — replaces original v1.2)

User feels: "I finally understand ownership, control flow, and trait dispatch in this codebase."

```
┌──────────────────────────────────────────────────────┬─────────────────────────┐
│                    What ships                        │   Experience enabled    │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Trait/impl browser with dispatch resolution column   │ Rust polymorphism       │
│                                                      │ made concrete           │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Call chain explorer (MIR-exact, error path branches)  │ Execution flow with    │
│                                                      │ zero ambiguity          │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Control flow graph visualizer (BasicBlock CFG)       │ Branching structure     │
│                                                      │ made visible            │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Ownership & borrow visualizer (Polonius-verified)    │ Borrow checker          │
│                                                      │ reasoning visible       │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Side-by-side comparison (with type + ownership diff) │ Structural comparison   │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Data flow explorer (MIR Place analysis)              │ Value provenance        │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Async execution flow visualizer                      │ Async Rust clarity      │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Cycle detection + dead code + k-core overlays        │ Architecture health     │
└──────────────────────────────────────────────────────┴─────────────────────────┘
```

v1.2 adds: 9 endpoints (/impls, /traits, /call-chain, /compare, /cycles, /dead-code,
/core-periphery, /entity/{id}/borrows, /entity/{id}/dataflow, /entity/{id}/async-flow)

### v1.3 — "I Can Reason About Architecture" (THE MOAT)

User feels: "I can explore what-if scenarios and compare architectural options with real data."

```
┌──────────────────────────────────────────────────────┬─────────────────────────┐
│                    What ships                        │   Experience enabled    │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Variant Graph Overlays (create, diff, compare)       │ Architectural           │
│                                                      │ what-if analysis        │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Consequence Engine (PageRank delta, SCC changes,     │ Data-driven             │
│ k-core delta, community boundary shifts, hotspot     │ architecture decisions  │
│ changes)                                             │                         │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Variant overlay on architecture map (dotted proposed │ Visual impact           │
│ edges, strikethrough removed, consequence badges)    │ assessment              │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ LLM Architecture Advisor pattern (narrates trade-    │ Guided architectural    │
│ offs from consequence data)                          │ reasoning               │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ ?variant= parameter on all existing APIs             │ Query any view in       │
│                                                      │ hypothetical mode       │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Generic instantiation viewer                         │ Monomorphization        │
│                                                      │ clarity                 │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Unsafe analysis visualizer                           │ Safety boundary         │
│                                                      │ understanding           │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Lifetime scope visualizer (Polonius subset_base)     │ Borrow conflict         │
│                                                      │ resolution              │
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Dependency ladder (with type dependencies)           │ Module boundary         │
│                                                      │ understanding           │
└──────────────────────────────────────────────────────┴─────────────────────────┘
```

v1.3 adds: 8 endpoints (POST /variant, GET /variant/{id}, GET /variant/{id}/diff,
GET /variant/compare, DELETE /variant/{id}, + ?variant= parameter on existing APIs,
/entity/{id}/monomorphizations, /entity/{id}/unsafe-analysis)

---

## 13. Final Synthesis (Updated)

The single best newcomer workflow: **Focus Lens → Architecture Overview → Community Zoom → Read
the Most Important Entity → Click a Call → See the Resolved Target → View CFG.** In 60 seconds,
the user goes from "I have no idea what this codebase is" to reading the most critical function
AND seeing its exact control flow graph with compiler-verified call targets. The focus lens makes
every step legible — the selected thing is bright, its neighborhood is visible, everything else
fades. No other tool provides this path.

The core UX problem solved: **The Semantic Focus Lens.** You don't zoom the graph — you zoom the
representation. Four levels of abstraction (workspace → subsystem → entity → flow), importance
always relative to where you're standing (PPR → BFS → PageRank → edge semantics), boundary nodes
as exit portals not clutter. This is not a feature. This is how the product navigates.

The moat: **Variant Graph Overlays as an LLM architectural reasoning workspace.** Today, LLMs
reason about code at the line level. Architecture decisions happen at the module boundary level.
Variants bridge this gap — they give LLMs a structured workspace at the public interface level
where they can propose changes as typed operations, receive computed consequences (coupling deltas,
community shifts, cycle creation), and reason by measurement instead of analogy. No other tool
gives LLMs this capability. An LLM with Parseltongue variants goes from "it depends on your use
case" to "Variant A reduces coupling 26% but adds one abstraction — worth it if you plan multiple
implementations." That's the moat.

The best visual workflow: **The architecture map with focus lens and community zoom** — built on
a graph with zero false edges, rendered with focus-relative salience.

The best Rust-specific workflow: **Trait/Impl Browser with dispatch resolution.** Click a trait
method call, see exactly which implementation the compiler chose, see all call sites grouped by
concrete type. No IDE, no tool, nothing else shows this.

The best LLM-guided workflow: **The guided tour** — same as v1, but now the "Ownership Patterns
Tour" is real (Polonius-verified), not syntactic.

The most dangerous gimmick to avoid: **Variant overlays without trust constraints.** If the LLM
can insert arbitrary edges with no structure, the system becomes fiction fast. Every proposed edge
must be typed, variant-scoped, justified with rationale, and clearly marked as "proposed" — never
"truth."

The biggest constraint eliminated: **"Edge quality caps explanation depth."** No longer true. The
compiler's edges ARE the truth. The only remaining uncertainty is dynamic dispatch, which is
honestly labeled. Everything else is exact.

**Priority order for the two foundational ideas:**
1. Build the Semantic Focus Lens first — it IS the navigation
2. Build Variant Graph Overlays second — it IS the moat
3. Build "compare architecture options" views on top

The one-sentence product thesis (updated):

**Parseltongue is a reading environment that uses the Rust compiler's own graph — exact call
targets, verified borrow scopes, real control flow — viewed through a semantic focus lens that
makes the local neighborhood legible and the rest fade away, with variant overlays that give
LLMs something they have never had: the ability to reason about architecture at the public
interface level with computed consequences instead of pattern-matched analogies.**

---

## Appendix A: rustc_private Stability Guarantee

```
┌──────────────────────────────────────────────────────────────┐
│                   VERSION PINNING STRATEGY                    │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  rust-toolchain.toml:                                        │
│    [toolchain]                                               │
│    channel = "nightly-2025-03-01"                            │
│                                                              │
│  API Category          Change Frequency Between Nightlies    │
│  ────────────────────  ────────────────────────────────────   │
│  Core queries           0% (fn_sig, visibility, type_of)     │
│  MIR structure           1% (BasicBlock layout)               │
│  HIR structure           3% (minor field renames)             │
│  Helper methods          5% (convenience wrappers)            │
│  Error types             5% (diagnostic changes)             │
│                                                              │
│  Upgrade path: Pin → work 100% → upgrade 6 months later →   │
│  95-98% works without changes → fix 2-5% in 30 minutes      │
│                                                              │
│  Production tools using this strategy:                       │
│  Miri, Flowistry, Aquascope, Prusti, Creusot, Rudra,        │
│  Kani, Charon — all pinned, all stable, all working.        │
│                                                              │
│  Fallback: If compilation fails, degrade to tree-sitter.     │
│  Show banner: "Syntax-only mode. Edges may be approximate."  │
└──────────────────────────────────────────────────────────────┘
```

## Appendix B: Semantic Focus Lens — Ranking Formula

```
┌────────────────────────────────────────────────────────────────┐
│                    FOCUS-RELATIVE RANKING                       │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  For a focus node F, each entity E gets a composite score:     │
│                                                                │
│  score(E) = w1 * PPR(F→E)                                     │
│           + w2 * (1 / BFS_dist(F, E))                          │
│           + w3 * GlobalPageRank(E)                              │
│           + w4 * EdgeKindWeight(best_edge(F, E))               │
│                                                                │
│  Default weights:                                              │
│    w1 = 0.50  (local relevance dominates)                      │
│    w2 = 0.25  (proximity matters)                              │
│    w3 = 0.15  (global importance is a tiebreaker)              │
│    w4 = 0.10  (edge semantics refine)                          │
│                                                                │
│  Edge kind weights:                                            │
│    calls          = 1.0                                        │
│    impls          = 0.9                                        │
│    public_boundary = 0.8                                       │
│    type_refs      = 0.7                                        │
│    contains       = 0.5                                        │
│                                                                │
│  Ring assignment:                                              │
│    Ring 0 (focus): E == F                                       │
│    Ring 1 (visible): BFS_dist ≤ 1 AND score > threshold_1      │
│    Ring 2 (faint): BFS_dist ≤ 2 AND score > threshold_2        │
│    Boundary: BFS_dist == max_depth AND crosses community       │
│    Ghosted: everything else                                    │
│                                                                │
│  Max ring 1 size: 15 entities (ranked by score, truncated)     │
│  Max ring 2 size: 30 entities (ranked by score, truncated)     │
│                                                                │
│  Performance:                                                  │
│    PPR computation: ~10-50ms (igraph, sparse graph)             │
│    BFS 2-hop: <5ms                                              │
│    Total focus transition: <100ms target                        │
│    Pre-cached for top-20 entities per community: <10ms          │
└────────────────────────────────────────────────────────────────┘
```

## Appendix C: Variant Graph Overlay Schema

```
┌────────────────────────────────────────────────────────────────┐
│                    VARIANT OVERLAY SCHEMA                       │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  SQLite table: variants                                        │
│  ──────────────────────────────                                │
│  variant_id    TEXT PRIMARY KEY                                 │
│  name          TEXT NOT NULL                                    │
│  description   TEXT                                             │
│  created_at    TIMESTAMP                                       │
│  workspace_id  TEXT NOT NULL (FK → workspaces)                 │
│                                                                │
│  SQLite table: variant_deltas                                  │
│  ──────────────────────────────                                │
│  delta_id      INTEGER PRIMARY KEY                             │
│  variant_id    TEXT NOT NULL (FK → variants)                   │
│  op            TEXT NOT NULL ("add_edge"|"remove_edge"|         │
│                               "change_edge_kind")              │
│  src_entity    TEXT NOT NULL                                    │
│  dst_entity    TEXT NOT NULL                                    │
│  edge_kind     TEXT NOT NULL ("calls"|"impls"|"type_ref"|      │
│                               "contains"|"public_boundary")    │
│  old_edge_kind TEXT (only for change_edge_kind)                │
│  rationale     TEXT NOT NULL                                    │
│  proposed_by   TEXT ("human"|"llm")                             │
│                                                                │
│  SQLite table: variant_consequences (cached)                   │
│  ──────────────────────────────────────────                    │
│  variant_id    TEXT NOT NULL (FK → variants)                   │
│  metric        TEXT NOT NULL ("pagerank"|"kcore"|"scc"|         │
│                               "leiden"|"hotspot")              │
│  entity_id     TEXT                                             │
│  old_value     REAL                                             │
│  new_value     REAL                                             │
│  change_pct    REAL                                             │
│  computed_at   TIMESTAMP                                       │
│                                                                │
│  Trust constraints (enforced at API level):                    │
│  • op must be one of the 3 allowed operations                  │
│  • edge_kind must be a known type (no "relates_to")            │
│  • rationale must be non-empty                                 │
│  • Max 5 active variants per workspace                         │
│  • Max 20 deltas per variant                                   │
│  • All proposed edges render as dotted lines, never solid      │
│  • Base graph edges NEVER modified by variants                 │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

## Appendix D: Indexing Time Budget (unchanged from Appendix B)

```
┌────────────────────────────────────────────────────────────────┐
│                    INDEXING PIPELINE TIMING                     │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  Phase                    Small (5K LOC)  Large (100K LOC)     │
│  ──────────────────────   ──────────────  ───────────────      │
│  rustc compilation         3s              25s                  │
│  MIR extraction            1s               5s                  │
│  Edge building             <1s              3s                  │
│  Polonius facts            2s              10s (optional)       │
│  Leiden + PageRank         <1s              2s                  │
│  SQLite writes             <1s              3s                  │
│  ──────────────────────   ──────────────  ───────────────      │
│  Total (without Polonius)  ~5s             ~38s                 │
│  Total (with Polonius)     ~7s             ~48s                 │
│                                                                │
│  Polonius is opt-in. Default: MIR-only indexing.               │
│  User can click "Enable borrow analysis" to re-index with     │
│  Polonius. This is a one-time cost per workspace.              │
└────────────────────────────────────────────────────────────────┘
```

## Appendix E: Typed Boundary Table Schemas

```
┌────────────────────────────────────────────────────────────────┐
│              TYPED BOUNDARY TABLE SCHEMAS                       │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  These tables are materialized at index time from the base     │
│  entities + edges tables. Refreshed on re-index. Queryable     │
│  instantly. Replaces the earlier flat folder_deps design.      │
│                                                                │
│  ─────────────────────────────────────────────────             │
│  Table: boundaries                                             │
│  ─────────────────────────────────────────────────             │
│  boundary_id     TEXT PRIMARY KEY  (path-based, e.g. "server/")│
│  name            TEXT NOT NULL     (leaf name, e.g. "server")  │
│  boundary_type   TEXT NOT NULL     ("crate"|"module"|"folder") │
│  parent_id       TEXT              (FK → boundaries, nullable) │
│  path            TEXT NOT NULL     (filesystem path)           │
│  depth           INT NOT NULL      (nesting from workspace)   │
│                                                                │
│  -- Computed at index time (materialized):                     │
│  entity_count    INT                                           │
│  pub_surface     INT                                           │
│  internal_edges  INT                                           │
│  outgoing_edges  INT                                           │
│  incoming_edges  INT                                           │
│  cohesion        REAL   (internal_edges / entity_count)        │
│  coupling_out    REAL   (outgoing_edges / entity_count)        │
│  coupling_in     REAL   (incoming_edges / entity_count)        │
│  fan_in          INT    (distinct src boundaries incoming)     │
│  fan_out         INT    (distinct dst boundaries outgoing)     │
│  is_facade       BOOL   (>60% pub items are pub use re-export)│
│                                                                │
│  ─────────────────────────────────────────────────             │
│  Table: boundary_edges                                         │
│  ─────────────────────────────────────────────────             │
│  src_boundary    TEXT NOT NULL  (FK → boundaries)              │
│  dst_boundary    TEXT NOT NULL  (FK → boundaries)              │
│  crossing_type   TEXT NOT NULL  ("CROSS-CRATE"|"INTRA-CRATE"| │
│                                  "INTRA-MODULE")               │
│  edge_count      INT                                           │
│  file_pairs      INT                                           │
│  distinct_items  INT   (how many distinct dst items imported)  │
│  distinct_files  INT   (how many distinct src files import)   │
│  kinds           TEXT  (comma-separated edge_kinds)             │
│  PRIMARY KEY (src_boundary, dst_boundary)                      │
│                                                                │
│  ─────────────────────────────────────────────────             │
│  Table: file_deps (kept for file-level queries)                │
│  ─────────────────────────────────────────────────             │
│  src_file      TEXT                                            │
│  dst_file      TEXT                                            │
│  edge_count    INT                                             │
│  kinds         TEXT                                            │
│  PRIMARY KEY (src_file, dst_file)                              │
│                                                                │
│  Query examples:                                               │
│  • Most coupled boundary pair:                                 │
│    SELECT * FROM boundary_edges                                │
│    ORDER BY edge_count DESC LIMIT 5                            │
│  • Most depended-upon boundary:                                │
│    SELECT dst_boundary, SUM(edge_count)                        │
│    FROM boundary_edges GROUP BY 1 ORDER BY 2 DESC              │
│  • Highest cohesion modules:                                   │
│    SELECT * FROM boundaries                                    │
│    WHERE boundary_type = 'module'                              │
│    ORDER BY cohesion DESC LIMIT 10                             │
│  • Cross-crate coupling only:                                  │
│    SELECT * FROM boundary_edges                                │
│    WHERE crossing_type = 'CROSS-CRATE'                         │
│    ORDER BY edge_count DESC                                    │
│  • Symmetric boundaries (same dep profile):                    │
│    SELECT GROUP_CONCAT(src_boundary), dep_profile              │
│    FROM (SELECT src_boundary,                                  │
│      GROUP_CONCAT(dst_boundary ORDER BY dst_boundary)          │
│      AS dep_profile FROM boundary_edges                        │
│      GROUP BY src_boundary)                                    │
│    GROUP BY dep_profile HAVING COUNT(*) > 1                    │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```
