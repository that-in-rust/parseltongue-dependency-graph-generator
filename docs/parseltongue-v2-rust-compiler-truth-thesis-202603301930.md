# Parseltongue v2: Rust Code Reading Companion — Compiler Truth Edition

**Constraint: Rust codebases only. Compiler-verified graphs. Zero ambiguity.**

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

**New thesis point (9):**

9. **The compiler is the source of truth, and the product never contradicts it.** Every edge, every
   type annotation, every ownership fact shown in the UI must trace back to a `rustc_private` query.
   If the compiler doesn't know it, the product doesn't claim it. The LLM can speculate ("this
   pattern is probably used for X"), but the graph never speculates.

---

## 3. Reading Modes (15 Modes — 12 Upgraded + 3 New)

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

## 4. Workflow Catalog (35 Workflows — 28 Upgraded + 7 New)

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

## 5. Top 10 UX Flows (Re-Ranked for v2)

1. **Architecture Overview → Community Zoom → Read** (same as v1, but better data)
2. **Guided Tour** (upgraded with "Type System Tour" and "Ownership Tour")
3. **Call Chain Explorer** (MAJOR upgrade — exact chains, error path branches)
4. **Trait/Impl Browser with Dispatch Resolution** (NEW capability, killer Rust feature)
5. **"What Should I Read Next?"** (same algorithm, better graph)
6. **Ownership & Borrow Visualizer** (REBUILT — compiler-verified, not syntactic)
7. **"You Are Here" Navigator** (same)
8. **Control Flow Graph Visualizer** (NEW — no other tool has this)
9. **"Explain This Module To Me"** (same, but LLM gets richer compiler context)
10. **Reading History & Resume** (same)

**What moved up:** Call Chain Explorer (#8 → #3) because exact chains are transformative.
Trait/Impl Browser (#5 → #4) because dispatch resolution is a killer feature.
Ownership Visualizer (was "Later" → #6) because Polonius makes it real, not approximate.

**What's new in top 10:** Control Flow Graph (#8) — unique capability no other tool provides.

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

## 7. Visual System (12 Surfaces — 10 Upgraded + 2 New)

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

All v1 risks (too much text, fake visual novelty, stale snapshot, bad tours, overwhelming
newcomers, LLM over-explaining, meaningless hotspots, slow LLM, reading pressure) remain
unchanged.

---

## 12. Recommended Build Order (Updated)

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

v1 endpoints: 12 (v1 original 10 + /resolve-dispatch + /entity/{id}/cfg)

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

### v1.3 — "I Can Understand Everything" (power features)

```
┌──────────────────────────────────────────────────────┬─────────────────────────┐
│                    What ships                        │   Experience enabled    │
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
├──────────────────────────────────────────────────────┼─────────────────────────┤
│ Architecture simulation (mutate → diff → explain)    │ What-if analysis        │
└──────────────────────────────────────────────────────┴─────────────────────────┘
```

---

## 13. Final Synthesis (Updated)

The single best newcomer workflow: **Architecture Overview → Community Zoom → Read the Most
Important Entity → Click a Call → See the Resolved Target → View CFG.** In 60 seconds, the user
goes from "I have no idea what this codebase is" to reading the most critical function AND seeing
its exact control flow graph with compiler-verified call targets. No other tool provides this path.

The best visual workflow: **The architecture map with community zoom** — same as v1, but built on
a graph with zero false edges.

The best Rust-specific workflow: **Trait/Impl Browser with dispatch resolution.** Click a trait
method call, see exactly which implementation the compiler chose, see all call sites grouped by
concrete type. No IDE, no tool, nothing else shows this.

The best LLM-guided workflow: **The guided tour** — same as v1, but now the "Ownership Patterns
Tour" is real (Polonius-verified), not syntactic.

The most dangerous gimmick to avoid: **Showing CFG for simple functions.** A 3-line function
doesn't need a control flow graph. Gate complex visualizations behind complexity thresholds.

The biggest constraint eliminated: **"Edge quality caps explanation depth."** No longer true. The
compiler's edges ARE the truth. The only remaining uncertainty is dynamic dispatch, which is
honestly labeled. Everything else is exact.

The one-sentence product thesis (updated):

**Parseltongue is a reading environment that uses the Rust compiler's own graph — exact call
targets, verified borrow scopes, real control flow — to turn a large Rust codebase into something
that feels like an explorable, guided, structured book where every edge is true, every ownership
annotation is compiler-verified, and a companion is ready to explain exactly what the borrow
checker sees when you can't.**

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

## Appendix B: Indexing Time Budget

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
