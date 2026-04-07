# Global Public Interface Thesis

## Premise Check

Parseltongue should not try to become the universal raw-code archive for open source.

That is the wrong layer.

The stronger ambition is to become the canonical way to derive, store, version, and simulate a **Public Interface Relationship Graph** from open-source repositories across languages.

That makes Parseltongue:

- a derived semantic layer, not a raw archive
- a cross-language interface graph standard, not a compiler-specific truth engine
- a simulation substrate for architecture reasoning, not a general-purpose source host

## Core Thesis

If Parseltongue can define a stable, cross-language Public Interface Relationship Graph and make it:

- cheap to build from tree-sitter
- durable to store
- easy to diff
- safe to version
- useful for retrieval and simulation

then it could become valuable training data for coding models and a useful systems layer for tooling built on top of open-source code.

## Why This Could Matter

Raw code is noisy.

A public-interface graph is smaller, more stable, and closer to the architectural signals that matter for:

- API understanding
- dependency reasoning
- repository-level retrieval
- migration planning
- architecture-aware agents
- model post-training and evaluation

This does not replace raw source corpora.

It complements them with a canonical structural layer.

## What We Should Not Claim

With tree-sitter only, Parseltongue should not claim:

- compiler-truth semantics
- exact runtime behavior
- exact dispatch resolution in dynamic languages
- refactor safety in the compiler sense

The honest claim is narrower and stronger:

Parseltongue models **public interface relationships** and simulates structural consequences over that graph.

## The Wedge

The smallest credible wedge is:

1. Build a universal Public Interface Relationship Graph for a focused set of languages.
2. Version it per repository and commit.
3. Support exact export, diff, and simulation over that graph.
4. Prove it helps:
   - coding agents
   - retrieval systems
   - repository understanding
   - model post-training or evaluation

## Why This Is Better Than "Store All OSS"

"Store all OSS" sounds large but is strategically weak because it competes with archive and dataset incumbents.

"Define the canonical public-interface graph layer" is more differentiated because it creates a new semantic artifact:

- one repo can have many source files
- but it should have one canonical public-interface graph per revision

That gives us a product and standards wedge.

## Product Implication

Parseltongue should be framed as:

> a tree-sitter-based Public Interface Relationship Graph engine that can export, diff, and simulate architecture-level consequences across languages.

This keeps the scope ambitious but honest:

- tree-sitter only
- no `rustc`
- no compiler lock-in
- graph truth is interface-structural truth
- simulation operates on the graph artifact, not on the full program runtime

## Data Product Implication

If this works, the durable assets are:

- the schema
- the versioned graph snapshots
- the scenario mutation history
- the provenance model
- the benchmark showing downstream value

Those matter more than any one binary format.

## What Must Be Proven

Before calling this a platform, Parseltongue needs proof in four areas:

- schema stability across languages
- trustworthy provenance and licensing boundaries
- reproducible graph extraction
- measurable lift for downstream model or tooling tasks

## Recommended Direction

Build toward a world where Parseltongue is:

- not the raw archive of open source
- not a code host
- not a compiler-truth engine

but the canonical **public interface graph layer** for repositories, versions, and architecture simulation.

## Related Research

- [Iggy Storage Meta Patterns](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/strategic-research/iggy-storage-meta-patterns.md)
- [Iggy Storage Pattern Research](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/strategic-research/iggy-storage-pattern-research.md)
- [Tree Sitter Simulation Scope](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/strategic-research/tree-sitter-simulation-scope.md)
