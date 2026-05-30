# v313 Walk Runtime Benchmark ELI5

## Big Idea

The walk runtime looks good because it stores the graph in the shape of the question.

That is the whole trick.

Instead of asking the machine to keep searching a table for matching rows, we give it a stored map of neighbors and let it jump straight to the right slice.

This note is based on:

- the `v313` PRD in [A-20260415152053-v313-PRD-L2.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/A-20260415152053-v313-PRD-L2.md)
- the walk-runtime thesis in [A-20260408115716-pensieve-walk-runtime-thesis.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/strategic-research/A-20260408115716-pensieve-walk-runtime-thesis.md)
- the options explainer in [A-20260408140806-walk-runtime-options-explainer.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/strategic-research/A-20260408140806-walk-runtime-options-explainer.md)
- the current harness benchmark report in [benchmark_report.json](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/test-harness/rust-test-001/bench_artifacts/benchmark_report.json)

## Why It Matters

We kept separating two jobs:

- build time
- walk time

Build time is allowed to be heavy.
It can parse code, resolve names, build hash maps, assign dense IDs, and write files.

Walk time should be boring.
It should mostly do:

- find the node
- find the offsets
- read the neighbor slice
- keep walking

That matters because `v313` is trying to power things like:

- one-hop impact
- two-hop impact
- Interface Dependency Map walking
- `FFF` plus cheap follow-up traversal

If walk time keeps scanning tables, the product feels slow.
If walk time can jump straight to slices, it feels more like a map.

## Core Ideas Made Simple

### 1. This does not depend on Rust

Rust can build it.
Python can read it.
Any language can read it if it understands the file format.

That is because the important part is not "Rust logic in RAM."
The important part is the persisted shape on disk.

For the benchmark, we used Python to prove exactly that:

- raw CSV scan backend
- embedded `libsql` backend
- walk snapshot backend using `offsets + peers` binary files with `mmap`

So the real product idea is:

- the graph lives on disk
- the runtime reads slices from disk
- the language is just the driver

### 2. The `v313` split still makes sense

The current design idea is:

- SQL for metadata and key lookup
- walk snapshot for actual graph walking

That means:

- `by_key` = resolve a name first, then walk
- `by_id` = dense ID already known, so just walk

This is like using a reception desk and a building map.

- SQL is the reception desk
- the walk snapshot is the building map

If you already know the room number, you do not need to ask the receptionist again.

### 3. The walk runtime wins where it should win

The strongest signal is the hot `by_id` path.

That is the clean structural question:

- no fuzzy search
- no label lookup
- no table scan
- just graph walking

From the current harness report:

- hot `forward_one by_id`
  - CSV: about `84,957 ns`
  - libSQL: about `10,592 ns`
  - walk: about `2,104 ns`
- hot `reverse_two by_id`
  - CSV: about `196,612 ns`
  - libSQL: about `25,800 ns`
  - walk: about `3,102 ns`

So on this small persisted benchmark:

- walk is about `5x` faster than libSQL for hot forward walking
- walk is about `8x` faster than libSQL for hot reverse-two-hop walking
- walk is about `40x` to `60x` faster than raw CSV scanning

These are measured facts from the current report, not universal guarantees.

Also, this report is a small verification run on `rust-test-001`, using:

- `warmup_passes = 1`
- `measure_passes = 2`
- `loops_per_pass = 5`

So the result should be read as **directional evidence for the design**, not a final production-grade benchmark campaign.

### 4. The walk runtime is not trying to win every metric

The cold-open numbers are not the hero story here.

Why?

Because cold-open deliberately includes reopening persisted things over and over.
That is useful to measure, but it is not the main product promise.

The main promise is:

- once the snapshot is there
- and once we are walking repeatedly
- structural traversal should be cheap

That is exactly where the walk runtime looks best.

### 5. `by_key` is supposed to be less dramatic

When we use `by_key`, the walk backend still pays for metadata lookup first.

That is not a bug.
That is the architecture.

So the expected shape is:

- `by_key` = closer to libSQL
- `by_id` = where the walk snapshot should really pull away

That is what the harness showed.

So the benchmark did not just give numbers.
It matched the theory.

### 6. We learned two practical runtime lessons

While wiring the benchmark, two small bugs taught us something useful.

First:

- reverse two-hop parity should compare the neighbor set, not the incidental order

Second:

- `by_id` should stay structural
- it should not accidentally fall back to "treat this ID like a string key"

In plain words:

- graph correctness is about the right reachable nodes
- `by_id` should remain the fastest, simplest path

That lines up with the walk-runtime thesis very well.

## Tiny Example

Imagine you want to know:

- who is affected if `save_session_record_now` changes?

There are three ways to answer that.

### Raw CSV way

You keep reopening a sheet of edge rows and searching for matches.

That is like reading every road entry in a city spreadsheet just to answer one local question.

### SQL way

You ask indexed tables for matching rows.

That is much better.
Now you have a proper receptionist with a filing cabinet.

### Walk runtime way

You already know the room number.
You open the map, jump to that room's incoming slice, and read the callers directly.

That is why the walk runtime is faster:

- less searching
- more jumping

## What To Remember

The benchmark says the walk runtime is good at the exact job it was built for:

- persisted
- repeated
- structural
- graph walking

It is not replacing SQL.
It is giving SQL a partner.

The clean architecture story now looks like this:

- SQL helps find the node
- the walk snapshot helps walk the graph

The current harness is still tiny, so this is not the final proof for a big repo.
But it is strong evidence that the `v313` direction is technically sane.

Sticky sentence:

**Use SQL to find the place, then use the walk snapshot to move through it.**
