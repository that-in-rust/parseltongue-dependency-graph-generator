# Can The Walk Graph Handle Control Flow? Rust MIR Says Yes

## Big Idea

The walk-graph architecture we designed is not locked to function-level edges.

Its storage shape — packed neighbor slices, CSR/CSC adjacency, sharded by node range — works for any directed graph.
That includes the control flow inside individual functions.

For most languages, building a real control flow graph is impossible from a syntax-only parser.
But for Rust specifically, the compiler already builds an explicit, typed control flow graph as part of normal compilation.
It is called MIR.

The story is: **the walk architecture fits perfectly, the blocker has always been the parser, and for Rust that blocker does not exist.**

---

## Why It Matters

The call graph tells you:
- which function calls which function
- what breaks if you change a function

The control flow graph tells you:
- which lines of code run after which other lines
- whether execution can even reach a given line
- which code is inside a loop
- which code runs when a panic happens

These are completely different questions.

The call graph is the city map.
The control flow graph is the street-level turn-by-turn directions inside one building.

Both are useful.
One is much harder to build — unless you have Rust.

---

## Core Ideas Made Simple

### 1. Why tree-sitter cannot give you a control flow graph

Tree-sitter is a syntax parser.
It reads the shape of the code — the brackets, keywords, and names.
It does not understand how the code executes.

To build a real control flow graph you need to know:
- where each `if` branch goes
- where each loop jumps back to
- what happens when a function panics
- whether a `match` arm is exhaustive

A syntax parser does not know any of this.
It sees `if foo { ... }` but cannot tell you which block runs.

So for languages other than Rust, the walk architecture can store a control flow graph, but building one from tree-sitter is not possible.
You would need a different parser for each language — something closer to a compiler.

### 2. What MIR is and why it matters

When you compile Rust code, `rustc` internally builds several representations of your program before producing machine code.

One of those representations is called **MIR** — Mid-level Intermediate Representation.

MIR is explicitly a control flow graph.
It is not a guess or an approximation.
It is the same structure the borrow checker and the optimiser use.

For every Rust function, MIR gives you:

```text
Function: login_handler
  BasicBlock BB0:
    statements: [let user = ..., ...]
    terminator: Call(load_user_record) -> [success: BB1, panic: BB4]

  BasicBlock BB1:
    statements: [...]
    terminator: Call(verify_password) -> [success: BB2, panic: BB4]

  BasicBlock BB2:
    statements: [...]
    terminator: Call(issue_session) -> [success: BB3, panic: BB4]

  BasicBlock BB3:
    statements: [return Ok(token)]
    terminator: Return

  BasicBlock BB4:
    terminator: Resume  (panic path)
```

Every node is a basic block — a straight-line sequence of code with no branching.
Every edge is a control flow transfer with an explicit type: `Goto`, `SwitchInt` (branch), `Call`, `Return`, `Drop`, `Assert`, `Resume` (panic).

This is a perfect graph.

### 3. How MIR maps to the walk architecture

The mapping is direct and requires almost no translation:

| Walk architecture concept | MIR concept |
|---|---|
| Node | `BasicBlock` (an integer index into a dense array) |
| Forward neighbor slice | Terminator's successor list |
| Backward neighbor slice | Reverse of successor list (CSC direction) |
| Shard | Group of functions and their basic block ranges |
| Dimension | Edge kind: `goto`, `branch_true`, `branch_false`, `call`, `unwind`, `return` |

The most important detail: `BasicBlock` in rustc is already an index into an `IndexVec`.
That means the node IDs are **already dense integers from zero**.
CSR/CSC construction requires no ID remapping step.
The raw data structure from the compiler is already in the right shape.

### 4. What new queries this unlocks

Questions the call graph cannot answer:

| Query | How |
|---|---|
| Can execution ever reach this line? | BFS forward from the entry basic block |
| What code always runs before this block? | Dominator walk (forward BFS ancestors) |
| What code always runs after? | Post-dominator walk (backward BFS from exit) |
| Which blocks are inside a loop? | SCC on the basic block graph |
| Can this panic path be triggered? | BFS following `unwind` / `Resume` edges |
| Is there dead code inside this function? | Unreachable blocks — no path from entry |
| What is the blast radius of changing this branch condition? | Backward walk on `branch_true` / `branch_false` edges |

These are richer and more precise than call-graph blast radius.
They are path-sensitive and intra-procedural.

### 5. The two-layer design

The walk artifact would have two node classes living together:

```text
Layer 1 — function-level (existing today)
  rust:fn:my_crate::auth::login_handler

Layer 2 — basic-block-level (new, Rust-only)
  rust:bb:my_crate::auth::login_handler::BB0
  rust:bb:my_crate::auth::login_handler::BB1
  rust:bb:my_crate::auth::login_handler::BB2
  ...
```

Edges connecting the layers:
- `contains`: `login_handler` → `BB0`, `BB1`, `BB2`, ...
- `calls` (inter-procedural): `BB2` (a Call terminator) → `verify_password::BB0`

The walk runtime does not need to change.
It just gains new node kinds and new edge dimensions.
The existing function-level call graph stays intact for all 12+ supported languages.

### 6. How to access MIR in practice

Tree-sitter reads files directly.
MIR requires a different approach: you hook into the Rust compiler as a driver.

Instead of:
```text
read file -> tree-sitter parse -> extract entities
```

You do:
```text
rustc compiles your code
    |
    v
your hook runs after analysis
    |
    v
tcx.optimized_mir(def_id) gives you the BasicBlock CFG
    |
    v
extract nodes and edges -> persist to walk artifact
```

The hook looks like implementing a `rustc_driver::Callbacks` trait.
The `after_analysis` method gives you a `TyCtxt` — the full compiler context.
From there, iterating every function and reading its MIR body is straightforward.

The main cost: this requires nightly Rust and the `#![feature(rustc_private)]` flag.
The rustc internal APIs are unstable and can change between compiler versions.
The mitigation is pinning to a specific compiler version in `rust-toolchain.toml`.

### 7. What Flowistry is and where it sits

Flowistry (github.com/willcrichton/flowistry) is a real, shipped tool that proves this path works.

It uses the exact same approach:
- rustc driver callbacks
- `after_analysis` hook
- `TyCtxt` + `BodyWithBorrowckFacts`
- MIR basic blocks as the underlying structure

But Flowistry is positioned **one layer above** what we described.

Think of it like this:

```text
Flowistry:          "does variable A influence variable B?"
                     (information flow analysis, program slicing)
                          |
Walk artifact layer: "what are the successors of BasicBlock 3?"
                     (raw CFG query — what we described)
                          |
rustc MIR:           the actual BasicBlock data structure
```

Flowistry answers higher-level questions by running a dataflow analysis on top of the CFG.
It does not persist the CFG as a queryable walk artifact.
It computes on demand — up to 15 seconds per function for large functions.

The walk architecture is the storage layer that Flowistry-style analysis could run on top of.

Three other important facts about Flowistry:

- It is **intra-procedural only** — it does not fully analyse what happens inside called functions, only approximates from type signatures.
  A call graph plus CFG walk artifact would be inter-procedural.

- It is **capped at Rust 1.73** — the README says this explicitly.
  This is the rustc API stability problem made real.
  Flowistry stopped tracking nightly.
  A pinned `rust-toolchain.toml` is the right mitigation for any tool using rustc internals.

- Its successor, **Paralegal** (Brown University), builds an inter-procedural Program Dependence Graph (PDG) from MIR.
  The PDG is a persistent graph combining CFG edges with data dependence edges across functions.
  That is the closest existing prior art to the inter-procedural walk artifact we described.

### 8. What the current tree-sitter path already handles

It is worth being clear that some control-flow-style questions already work today at the call-graph level:

| Question | Tool today |
|---|---|
| Can execution reach this function from the entry point? | BFS on `calls` dimension |
| What functions are always called before this one? | Backward walk on `calls` |
| What is the blast radius of changing this function? | Blast radius endpoint |
| Are there circular dependencies? | SCC on call graph |

These are control-flow questions answered at function granularity, not basic-block granularity.
For many practical questions, function granularity is enough.

Basic-block granularity only becomes necessary when the question is:
- *"Can execution skip the null check on line 47?"*
- *"Which branches are never reachable?"*
- *"Does the panic path access this variable?"*

---

## Tiny Example

A Rust function:

```rust
pub fn divide(x: f64, y: f64) -> Option<f64> {
    if y == 0.0 {
        return None;
    }
    Some(x / y)
}
```

MIR produces roughly:

```text
BB0: if y == 0.0 -> [true: BB1, false: BB2]
BB1: return None
BB2: return Some(x / y)
```

Walk artifact stores this as:

```text
nodes:  [BB0, BB1, BB2]
CSR fwd: BB0 -> [BB1, BB2]
         BB1 -> []
         BB2 -> []
CSC bwd: BB0 -> []
         BB1 -> [BB0]
         BB2 -> [BB0]
```

Queries this enables:

- "Can BB1 be reached from entry?" → BFS from BB0, yes via `branch_true` edge
- "What always runs before BB2?" → backward walk from BB2 → BB0 is the dominator
- "Is BB2 dead code?" → BFS from BB0 reaches BB2, so no

This is the same walk runtime.
Same CSR/CSC format.
Same shard structure.
Different node granularity.

---

## What To Remember

The walk-graph architecture is not a call-graph-only idea.
It is a general directed-graph storage shape.

For Rust specifically, rustc's MIR gives you a free, accurate, typed control flow graph as a compiler output — and its basic blocks are already dense integer IDs that map perfectly into CSR/CSC adjacency.

Flowistry proves the rustc driver path is viable in production.
But Flowistry is the analysis product on top.
The walk artifact is the storage layer below it.

**The compiler already built the control flow graph. The only question is whether you persist it in a queryable form or throw it away after each build.**

---

## Source Notes

This explainer captures the conversation and analysis from the following context:

- Prior conversation: *"60GB Graph On 16GB RAM: Walk Runtime + Rank Runtime"* — established the dual walk/rank architecture
- Prior conversation: *"Can this walk-graph architecture work for control flow?"* — identified tree-sitter as the blocker and Rust MIR as the exception
- Prior conversation: *"What if only built this for Rust using rustc APIs"* — worked through the MIR-to-walk-artifact mapping, scale analysis, and two-layer design
- Prior conversation: *"I think Flowistry exactly does that no?"* — placed Flowistry precisely in the stack, identified Paralegal as closer prior art
- `pensieve-dual-runtime-architecture-eli5.md` — base architecture this extends
- `walk-runtime-options-explainer.md` — walk runtime storage design
- `A-20260407213233-tree-sitter-edge-policy-options.md` — confirms the "tree-sitter only" parser constraint
- Flowistry paper: *"Modular Information Flow through Ownership"* (Crichton et al., PLDI 2022)
- Flowistry GitHub: github.com/willcrichton/flowistry (3,038 stars, last push Sep 2025, capped at Rust 1.73)
