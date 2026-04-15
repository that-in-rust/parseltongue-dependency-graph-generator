# Tree Sitter Edge Policy Options

## Premise Check

The current Parseltongue simulation note asks one artifact to do three jobs at once:

1. be a **MECE coverage ledger** for folders and files
2. be a **public/interface dependency graph**
3. be a **simulation substrate** for blast radius and structural diffs

That is possible, but only if the PRD is explicit about which layer is canonical and which layer is derived.

The biggest tension is this:

- "no nested entities"
- "one-level-below-file only"
- "comments/tests/imports separated out"
- "all entity `wc` sums must match file total"
- "entity graph should still be useful for calls, blast radius, and context"

Those goals do not naturally collapse into one perfect representation.

## Expert Lenses

- Parser lens: what Tree-sitter can actually provide reliably across languages
- Coverage/accounting lens: how to satisfy MECE `wc` invariants without lying
- Graph semantics lens: which entities and edges produce good dependency queries
- Skeptical lens: where the desire for one clean model creates hidden contradictions

## Candidate Approaches

### Option 1. Coverage First Flatgraph

This takes the current note literally.

- folder is an entity
- file is an entity
- every supported file is partitioned into one-level-only child entities
- unsupported files also become entities
- imports, comments, tests, and gaps are graph-visible entities
- every edge has a direction, even for sibling or shared-context relations

#### Parse model

- `folder`
- `file_supported`
- `file_opaque` for unsupported or intentionally unparsed files
- root-child fragment entities only
- no nested semantic extraction
- `wc(file) = sum(child fragment wc)`
- `wc(folder) = sum(child file wc)`

#### Edge model

- `contains`: parent -> child
- `calls`: caller -> callee
- `shared_parent`: deterministic canonical direction
- `shared_file_context`: deterministic canonical direction by source order

#### Strengths

- strongest accounting story
- easiest to verify mechanically
- simplest "every byte belongs to some entity" rule
- easiest to explain for storage and provenance

#### Weaknesses

- graph gets noisy because comments/imports/test fragments are first-class graph nodes
- OO languages lose method-level semantics if methods are not separate entities
- "shared context" edges become artificial and can distort algorithms if treated like causal edges
- unsupported files become graph residents even when they carry no real dependency semantics

#### Best fit

- coverage dashboards
- repository accounting
- folder/file/entity rollups
- UI cluster layout based on structural containment

### Option 2. Semantic First Flatgraph

This treats the graph primarily as a dependency and simulation substrate.

- folders and files are graph entities
- supported files contribute only semantically relevant entities
- unsupported files are recorded, but lightly
- comments/tests/imports are mostly metadata, flags, or side records
- siblinghood and same-file context are derived at query time instead of stored as durable edges

#### Parse model

- `folder`
- `file_supported`
- `file_non_eligible`
- code/public/interface entities only
- tests marked with flags, not separate semantic kinds
- comments excluded from primary graph identity
- imports may be side records that emit import edges without becoming nodes

#### Edge model

- `contains`: parent -> child
- `calls`: caller -> callee
- `imports`: importer -> imported target
- same-file and sibling relations derived from shared parent/file membership, not stored

#### Strengths

- cleanest graph for blast radius and simulation
- lower node count and less algorithm noise
- better fit for forward/backward dependency traversal
- easier to align with "public interface graph" framing

#### Weaknesses

- breaks the user's strongest accounting instinct unless a separate coverage ledger exists
- `wc` invariants no longer live entirely inside the graph
- comments and tests become second-class data even though they matter for accuracy reporting
- harder to claim "this graph exhaustively represents the file"

#### Best fit

- dependency traversal
- blast radius
- public boundary queries
- graph algorithms and simulation

### Option 3. Dual Layer Canonical Model

This separates **coverage truth** from **semantic graph truth** while keeping both derived from the same parse.

- Layer A is a one-level-below-file coverage partition
- Layer B is the graph used for dependency queries and simulation
- Layer A may remain strictly flat and MECE
- Layer B may be cleaner and, if needed later, semantically richer

This is the strongest design if we want both rigorous accounting and a high-quality graph.

#### Parse model

**Layer A: coverage partition**

- `folder`
- `file_supported`
- `file_opaque`
- root-child fragments only
- comments/imports/tests/docstrings/attributes/gaps counted here
- hard `wc` invariants enforced here

**Layer B: semantic graph**

- `folder`
- `file`
- code/interface entities
- imports can be nodes or edge-only records
- tests/comments may remain tags or fragment kinds, not graph-primary entities
- no requirement that every byte-sized fragment be graph-primary

#### Edge model

- `contains`: parent -> child
- `calls`: caller -> callee
- `imports`: importer -> imported target
- `shared_parent` / `shared_file_context` stored only if there is a clear product need
- if stored, symmetric relations use a deterministic canonical direction for persistence, but API semantics treat them as undirected context links

#### Strengths

- resolves the biggest contradiction cleanly
- preserves exact coverage/accounting
- preserves a cleaner graph for algorithms
- allows future richer semantic views without corrupting the coverage model
- makes accuracy reporting easier because parse failures and uncovered spans belong to Layer A explicitly

#### Weaknesses

- more design complexity
- requires the PRD to admit that "the graph" is not a single table of truth
- needs clear naming for the two layers so users do not confuse them

#### Best fit

- Parseltongue as both a coverage-aware ingestion system and a simulation engine

## Chosen Thesis

Option 3, the **Dual Layer Canonical Model**, is the best fit.

Why it wins:

- the current PRD asks for both strict coverage invariants and good dependency semantics
- local notes already show these concerns drifting apart
- Tree-sitter gives enough structure to build both layers from one parse
- this avoids polluting graph algorithms with bookkeeping-only nodes while preserving exactness

The single biggest recommendation is:

> make the one-level-below-file MECE partition the canonical **coverage layer**, and make the persisted dependency graph the canonical **semantic layer**.

## Evidence and Verification

### First Pass: Local Evidence

#### 1. The current note really does require a flat one-level partition

Your raw scope note states:

- only Tree-sitter
- file is broken into mutually exclusive cumulatively exhaustive entities
- only one level below file
- no nested entities
- `wc` totals must reconcile at file and folder level
- all edges must have direction

Source:
- [graph-PRD-simulations-20260407.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/graph-PRD-simulations-20260407.md)

#### 2. Older local notes already contained a stricter MECE coverage model

Earlier notes explicitly proposed:

- every entity has `wc`
- sum of entity `wc` equals file total
- root-child extraction only
- imports/comments/doc comments/attributes/whitespace as counted fragment kinds
- unsupported files as file-level structural entities

Source:
- [Notes20260317.md:121](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/archived-docs/Notes20260317.md#L121)
- [Notes20260317.md:128](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/archived-docs/Notes20260317.md#L128)

This strongly supports a coverage partition, but not necessarily a clean dependency graph.

#### 3. Older local notes also warned that comments and tests are classification problems

Local notes explicitly say:

- Tree-sitter does not distinguish doc comments from plain comments at the node type level
- Python docstrings are not comment nodes
- test detection is language-specific
- previous work allowed one-level split for large containers in some languages

Source:
- [Notes20260317.md:547](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/archived-docs/Notes20260317.md#L547)
- [Notes20260317.md:573](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/archived-docs/Notes20260317.md#L573)
- [Notes20260317.md:596](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/archived-docs/Notes20260317.md#L596)
- [Notes20260317.md:613](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/archived-docs/Notes20260317.md#L613)

This is direct evidence against pretending that comment/test treatment can be uniform and trivial.

#### 4. Local decision logs already separated non-eligible files and tests as first-class categories

Earlier decision logs used categories like:

- `non-eligible-text`
- `identifiable-tests`
- `code-graph`

and explicitly proposed same-file context as an inferred relation.

Source:
- [ES-V200-Decision-log-01.md:38](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/archived-docs/pre202602/ACTIVE-PRD/ES-V200-Decision-log-01.md#L38)

This supports the idea that unsupported files and test artifacts should be tracked explicitly, but it does not force them to be graph-primary entities.

#### 5. Earlier relation modeling used durable `from_key`, `to_key`, `edge_type`

That earlier shape is still good:

- a directional edge row is simple and durable
- semantic meaning belongs in `edge_type`

Source:
- [how-cozodb-connects-relations.md:9](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/archived-docs/journey-of-parseltongue/how-cozodb-connects-relations.md#L9)

### Second Pass: Web Evidence

#### 1. Tree-sitter does give precise positional ranges

The Rust bindings expose:

- `start_position`
- `end_position`
- `range`
- `start_byte`
- `end_byte`

Source:
- [tree-sitter Node docs](https://docs.rs/tree-sitter/latest/tree_sitter/struct.Node.html)

Implication:

- line ranges are available
- but for deterministic ordering and tie-breaks, **byte ranges** are stronger than line numbers alone

#### 2. Tree-sitter explicitly treats comments as `extra` nodes

The Node docs say:

- extra nodes represent things like comments
- they are not required by the grammar and can appear anywhere

Source:
- [tree-sitter Node docs](https://docs.rs/tree-sitter/latest/tree_sitter/struct.Node.html)

Implication:

- comments are real parse artifacts, but they are not structurally uniform like declarations
- forcing them into the same semantic graph role as declarations is a design choice, not a parser truth

#### 3. Tree-sitter is designed for incremental parsing, not just batch parsing

Tree-sitter describes itself as:

- an incremental parsing library
- able to efficiently update syntax trees as source is edited

Source:
- [tree-sitter GitHub README](https://github.com/tree-sitter/tree-sitter)

Implication:

- later incremental reindexing can be efficient
- but incremental parsing does not solve the semantic classification and edge-direction policy questions by itself

#### 4. Program-graph research supports multi-relation representations

Research like *Learning to Represent Programs with Graphs* argues that structured program graphs outperform less structured representations for downstream reasoning tasks.

Source:
- [Microsoft Research summary of the paper](https://www.microsoft.com/en-us/research/publication/learning-represent-programs-graphs/)

Implication:

- multiple edge families are a feature, not a problem
- but they should be semantically meaningful
- bookkeeping-only relations should not be allowed to contaminate dependency algorithms without filters

### Verification Questions

#### Q1. Can line numbers alone define a stable same-file ordering?

No.

Reason:

- Tree-sitter exposes byte offsets as well as line/column ranges
- multiple entities can start on the same line with different columns
- comments/import blocks/attributes can share line regions in ways that make line-only ordering ambiguous

Revision:

- use a total order of `(start_byte, end_byte, entity_kind_rank, stable_key)`

#### Q2. Can tests be modeled purely as a separate entity type?

Only with cost.

Reason:

- local notes previously preferred `is_test` flags
- separate `test_function`, `test_class`, `test_method` kinds explode taxonomy

Revision:

- use `entity_kind` for semantic kind
- use `entity_role = test | production | support`

#### Q3. Can comments be modeled uniformly across languages?

No.

Reason:

- local notes say doc comments require text inspection
- Python docstrings are not comment nodes
- Tree-sitter `extra` nodes are grammar-dependent

Revision:

- treat comments/doc-comments/docstrings as **coverage fragments first**
- only elevate them into semantic graph relevance where product value is proven

#### Q4. Is "no nested entities" compatible with good method-level call graphs?

Not always.

Reason:

- strict one-level extraction collapses methods into classes/impls in OO-heavy languages

Revision:

- either accept the coarse graph honestly
- or let the semantic layer derive deeper callable nodes later without changing the coverage layer

## Final Synthesis

The PRD should separate three ideas that are currently entangled:

1. **coverage partition**
2. **semantic graph**
3. **simulation edge policy**

Recommended design:

- `folder` is a graph entity
- `file_supported` and `file_opaque` are graph entities
- unsupported files should use `file_opaque` plus a reason enum such as:
  - `unsupported_extension`
  - `parser_unavailable`
  - `parse_error`
  - `binary_or_skipped`
- coverage fragments should be one-level-below-file and MECE
- semantic graph entities should be cleaner and more dependency-oriented
- tests should be a role/tag, not a parallel semantic taxonomy
- comments should be coverage-first, not graph-first
- sibling/shared-file context should only be stored if there is product value
- if stored, those symmetric relations should use deterministic persistence direction, not fake causal semantics

## Concrete Naming Recommendations

| concern | options | recommendation |
| --- | --- | --- |
| unsupported file entity | `file_unparsable`, `file_non_eligible`, `file_opaque` | `file_opaque` |
| test modeling | separate entity types, flag only, role enum | role enum |
| comment modeling | graph entity everywhere, side table only, coverage fragment | coverage fragment |
| symmetric relation persistence | mirrored dual edges, canonical single direction, derive only | derive only or canonical single direction |
| same-file ordering | line only, line+column, byte-range total order | byte-range total order |

## Open Questions

- Do we want imports to be graph nodes, edge emitters, or both?
- Is the no-nested rule truly non-negotiable, even if it weakens OO-language call graphs?
- Should the coverage layer and semantic layer share the same key namespace?
- Do we want `shared_file_context` persisted from day one, or derived only when clustering/context packing needs it?
- Should folder/file sibling relations exist as explicit edges, or should siblinghood be derived from shared parent containment?
