<!-- markdownlint-disable MD013 MD024 MD060 -->

# Solution Observations 01: Product Journeys, Bidirectional Loops, and Architecture Evidence

- **Date:** 2026-07-10
- **Mode:** High-recall observation before prioritization
- **Product:** Parseltongue, a backend-neutral code-intelligence companion for developers and coding agents
- **Companion judgment document:** [Sol-Insights-01.md](Sol-Insights-01.md)

## 1. Purpose

This document preserves the broad opportunity space found across the `docs/` corpus. It intentionally records more ideas than should be built. The goal is to keep useful product requirements, user journeys, failure evidence, architecture patterns, and implementation shapes visible before applying product judgment.

The companion insights document performs the harder filtering. An idea appearing here does not mean it has product-market fit, is technically correct, or belongs on the roadmap.

## 2. Corpus Reading Method

The repository contains 278 files under `docs/`, totaling roughly 216,000 lines. Content hashing found 252 unique file contents; 49 files occur in duplicate-content sets, mostly because material was copied into several archive trees.

The reading pass used four layers:

1. Current product surface: root `README.md`, current research summaries, current endpoint descriptions, and current known limitations.
2. Canonical synthesis: feature master tables, user-journey summaries, architecture overviews, and recent agent-tool research.
3. Failure evidence: live-server investigations, incremental-indexing RCAs, file-watcher failures, stable-identity work, backend stalls, and coverage reports.
4. Detailed pattern corpus: the five Research003 Tree-sitter pattern files covering parser runtime, language packs, indexing, agent context, and verification.

### Evidence Labels

- **Observed-current:** described by the current README or recent repository evidence.
- **Observed-historical:** verified at the time by code inspection, live-server output, or a test report, but may have since changed.
- **Designed:** a detailed proposal, not proof of shipped behavior.
- **Speculative:** an idea, forecast, benchmark estimate, or PMF score without durable validation evidence.

Historical documents contain exact claims such as 99 percent token reduction, 31x speedups, 90 percent fewer bugs, and numerical PMF scores. These are preserved as document claims, not treated as independently verified facts.

## 3. Premise Observations

### 3.1 The user is not shopping for a graph database

The recurring job is not "run graph algorithms." The recurring job is:

> Give a developer or coding agent the smallest trustworthy evidence needed to take the next action safely.

The graph is an implementation advantage. The product is a set of decisions:

- What should I inspect first?
- Is this symbol or relationship real?
- What calls this?
- What does this call?
- What could break if this changes?
- Which tests and flows are relevant?
- What changed since the baseline?
- Is the index current and complete enough to trust?
- What evidence was omitted because of scope, parser failure, or token budget?

### 3.2 The highest-value moment surrounds an edit

Several older journey documents independently converge on the same habit:

1. Before an edit, ask for minimal context and impact.
2. Make the edit.
3. After the edit, ask for graph diff, affected tests, and unresolved evidence.

This moment has urgency, frequency, a concrete cost of failure, and measurable success. It is a stronger wedge than a general-purpose architecture dashboard.

### 3.3 Trust is part of the result schema

The historical failures show that a successful HTTP response can still be a failed product outcome:

- The watcher reported running while reindexing was disabled.
- Smart context returned `success: true` with no entities.
- entity keys were null or unstable.
- search could not find a main function.
- hotspots were polluted by unresolved or external symbols.
- files could be skipped without a complete accounting.
- persistent backends behaved differently across operating systems.

Therefore, every meaningful answer needs freshness, coverage, provenance, confidence, unresolved cases, and truncation metadata. These are user-visible product requirements, not debugging extras.

### 3.4 Backend neutrality is a product constraint

The documents moved through RocksDB, Sled, SQLite, in-memory backup, and proposed PostgreSQL. Earlier work tied behavior to CozoDB and Datalog. The repeated migrations suggest that the durable product contract must sit above storage and graph execution.

The same HTTP request should mean the same thing whether the implementation uses:

- SQLite or Turso for durable facts;
- a Rust in-memory graph projection;
- a compiler-produced SCIP index;
- an external Neo4j deployment;
- a future GraphBLAS or GPU accelerator.

Differences in supported capabilities must be explicit. Backend failures must never become plausible-looking empty results.

## 4. High-Recall Opportunity Inventory

This section records concepts before ranking them.

### 4.1 Orientation and Discovery

- Repository statistics with freshness and coverage.
- Folder and workspace maps.
- Entry-point discovery.
- Symbol search by exact name, fuzzy name, type, path, or language.
- Guided architecture tours.
- Capability discovery: "where is authentication implemented?"
- Source-spanned entity explanations.
- Progressive disclosure from repository map to symbol to source slice.
- Cross-repository and monorepo orientation.
- Public API and route inventories.

### 4.2 Edit Safety and Change Understanding

- Minimal edit context.
- Direct callers and callees.
- Bounded blast radius with hop-by-hop explanations.
- Dependency paths between two symbols.
- Branch or snapshot graph diff.
- Added, removed, modified, relocated, and unresolved entities.
- API contract impact.
- Suggested review questions derived from impacted flows.
- Safe rename planning.
- Test selection based on changed graph regions.
- Post-edit validation that claimed symbols and relationships exist.

### 4.3 Debugging and Incident Response

- Error-message-to-symbol search.
- Reverse call-chain reconstruction.
- Failure cascade mapping.
- Exception and error-handling path analysis.
- Root-cause candidate ranking with evidence.
- Compare failing and healthy snapshots.
- Runtime trace or profiling overlays when external telemetry exists.
- Missing guard, missing caller, and orphan-handler detection.
- Reproduction-context packages for an agent.

### 4.4 Refactoring and Architecture Work

- Circular dependency and SCC reports.
- Coupling and cohesion metrics.
- centrality, k-core, PageRank, betweenness, and hotspot analysis.
- Module and community detection.
- Architecture rule enforcement.
- Service or crate boundary candidates.
- Dead-code candidates with confidence tiers.
- Temporal architecture drift.
- Churn correlated with graph centrality.
- Dependency structure matrix and focused diagrams.
- Migration order and blocker detection.

### 4.5 Agent Context and Token Economics

- Context packages rather than raw file dumps.
- Explicit token budget reports.
- Preview, pointer, and full-detail levels.
- Reasons for every selected context item.
- Reasons for every omitted or truncated category.
- Overfetch followed by final graph-aware selection.
- Query plan explanation before execution.
- Stateful exploration cursors.
- Session hot-path caches.
- Context continuation tokens.
- Current-file exclusion to avoid echo-search.
- Recent-tail preservation during summary compaction.

### 4.6 Live Indexing and Freshness

- File watching with service-lifetime ownership.
- Debounced and coalesced file events.
- File-level coarse invalidation as a reliable baseline.
- Optional entity-level incremental parsing after parity is proven.
- Immutable graph snapshots with monotonic revisions.
- Atomic publication after durable storage commits.
- Delete and rename handling for all derived records.
- Index-job status and failure inspection.
- Full-reparse parity checks.
- Config isolation per repository.

### 4.7 Multi-Language Understanding

- Central language and extension registry.
- Versioned query bundles per language.
- Typed capture roles.
- tags, locals, chunks, folds, injections, and relation queries as separate layers.
- Language-specific escape hatches in the common ontology.
- Embedded-language support.
- Optional SCIP or LSP enrichment.
- Unresolved references retained as unresolved evidence.
- Per-language coverage and drift reports.
- Fixture repositories and golden symbol/edge snapshots.

### 4.8 Team, CI, and Governance

- Headless indexing in CI.
- Baseline artifact plus branch artifact.
- Pull-request risk summary.
- Impacted tests and owners.
- Architecture-policy checks.
- Review context and evidence links.
- SARIF, JSONL, Markdown, Mermaid, DOT, and machine-readable exports.
- Reproducible installed-artifact smoke tests.
- Backend parity certification.
- Audit logs for index revision and query plan.

### 4.9 Visual and Human Interfaces

- Focused blast-radius view.
- Caller/callee path view.
- Graph diff view.
- Architecture health view.
- Dependency structure matrix.
- Guided onboarding tour.
- Data-lineage view.
- Migration ordering view.
- Test-gap view.
- Debug visualization connecting parse node, query capture, graph edge, and rendered context.

The old visualization corpus includes 24 journeys and many metaphors. The reusable insight is role-specific views tied to decisions. A universal animated graph is not itself a workflow.

## 5. Canonical Architecture Observations

### 5.1 Staged indexing pipeline

The recent Tree-sitter corpus finds the same pipeline across code search, repo maps, graph indexers, and agent tools:

```text
discover files
-> apply ignore and containment policy
-> detect language
-> parse source bytes
-> run versioned query layers
-> refine captures by traversal
-> construct source-spanned entities
-> resolve references with confidence
-> construct evidence-bearing edges
-> build document and chunk records
-> commit canonical facts
-> publish immutable graph snapshot
-> retrieve and render agent context
```

Each stage needs independent diagnostics, timing, tests, and cache keys.

### 5.2 Durable facts plus disposable projections

The strongest backend-neutral shape is:

```text
SQLite/Turso durable facts
        |
        +-> lexical/search indexes
        |
        +-> immutable in-memory graph snapshot
        |
        +-> optional external graph projection
        |
        +-> context renderer and token planner
```

The durable store owns repositories, files, source versions, entities, references, edges, diagnostics, and snapshot metadata. Internal graph node indexes are projection details and must never become API identity.

### 5.3 Reference engine plus accelerators

A small Rust reference engine should define behavior for:

- direct callers and callees;
- bounded traversal and blast radius;
- SCC and cycle representation;
- degrees and simple hotspots;
- scope filtering;
- deterministic ordering;
- unresolved-symbol treatment.

Advanced algorithms can be supplied by a Rust library, an FFI library, Neo4j GDS, GraphBLAS, or another accelerator. They should pass parity tests against fixtures and declare algorithm parameters. An accelerator cannot redefine endpoint semantics.

### 5.4 Compiler enrichment without compiler dependence

Tree-sitter provides concrete syntax, not full semantic resolution. The product can support two evidence tiers:

1. Tree-sitter baseline: fast, local, broad language coverage, with confidence and unresolved references.
2. Compiler/SCIP enrichment: stronger definitions, references, calls, type hierarchies, and implementations when an indexer is available.

The API should expose evidence quality without forcing every user through a build-system integration.

### 5.5 Immutable snapshot publication

Watch mode and concurrent queries become easier to reason about when queries read an immutable `GraphSnapshot`:

```rust
pub struct GraphSnapshot {
    pub snapshot_id: SnapshotId,
    pub repository_id: RepositoryId,
    pub source_revision: SourceRevision,
    pub created_at: SystemTime,
    pub graph: CodeGraph,
    pub coverage: CoverageSummary,
}
```

An update builds a replacement snapshot, validates it, and atomically swaps the shared `Arc<GraphSnapshot>`. Requests should report the snapshot ID they used.

### 5.6 Typed, evidence-bearing domain records

Useful boundary types suggested by the docs include:

```rust
pub struct SourceSpan {
    pub file_id: FileId,
    pub file_version: FileVersion,
    pub start_byte: u32,
    pub end_byte: u32,
    pub start_point: SourcePoint,
    pub end_point: SourcePoint,
}

pub struct RelationshipEvidence {
    pub source_span: SourceSpan,
    pub resolver: ResolverKind,
    pub confidence: Confidence,
    pub unresolved_reason: Option<UnresolvedReason>,
}

pub struct ContextItem {
    pub entity_id: EntityId,
    pub source_span: SourceSpan,
    pub selection_reason: SelectionReason,
    pub estimated_tokens: u32,
    pub confidence: Confidence,
}
```

These records make uncertainty and provenance serializable instead of leaving them in logs.

### 5.7 Backend ports should follow user behavior

A broad database abstraction tends to leak. Smaller ports match product capabilities:

```rust
pub trait CodeFactStore {
    fn commit_index_revision(&self, revision: IndexRevision) -> Result<CommitReceipt, StoreError>;
    fn load_graph_facts(&self, repository: RepositoryId) -> Result<GraphFacts, StoreError>;
    fn load_source_slice(&self, span: &SourceSpan) -> Result<SourceSlice, StoreError>;
}

pub trait GraphQueryEngine {
    fn callers(&self, request: CallersRequest) -> Result<CallersResponse, QueryError>;
    fn blast_radius(&self, request: BlastRadiusRequest) -> Result<BlastRadiusResponse, QueryError>;
    fn strongly_connected(&self, request: SccRequest) -> Result<SccResponse, QueryError>;
}

pub trait ContextPlanner {
    fn build_context(&self, request: ContextRequest) -> Result<ContextPackage, ContextError>;
}
```

HTTP handlers should call a product service, not SQL, Datalog, Cypher, or petgraph directly.

## 6. Bidirectional Workflows

The documents repeatedly advocate LLM-CPU collaboration. The safer interpretation is not "let the LLM fix graph facts." It is a loop in which deterministic tooling supplies evidence, the LLM interprets intent or ambiguity, and deterministic tooling verifies the next claim.

### BW-01: Intent -> Minimal Evidence -> Edit -> Structural Proof

```text
User intent
-> LLM identifies target behavior and candidate symbols
-> Parseltongue returns minimal edit context, callers, contracts, and confidence
-> LLM proposes or performs edit
-> Parseltongue indexes the changed revision and returns graph diff
-> LLM explains whether the edit stayed inside the intended boundary
-> tests and Parseltongue validate the claim
```

Requirements:

- Pre-edit and post-edit requests must identify the same repository and baseline revision.
- The response must distinguish known affected nodes from unresolved evidence.
- Post-edit proof must not rely on the LLM's narrative alone.
- A stale index must stop or visibly downgrade the workflow.

### BW-02: Error -> Hypothesis -> Graph Trace -> Revised Hypothesis

```text
Error, log, or failing test
-> LLM extracts symbols and possible subsystem
-> Parseltongue searches exact and fuzzy entities
-> LLM selects candidate path
-> Parseltongue traces callers, callees, and failure-adjacent tests
-> LLM revises root-cause hypothesis
-> Parseltongue packages evidence for the chosen path
```

Requirements:

- Each hypothesis must cite source-spanned entities and relationships.
- No-callers and no-results must distinguish "none" from "not indexed" and "unresolved."
- The user should see the shortest next experiment, not a giant graph dump.

### BW-03: Branch Diff -> Impacted Flows -> Review Questions -> Evidence Check

```text
Git branch diff
-> Parseltongue maps changed symbols and graph impact
-> LLM generates targeted review questions
-> reviewer or agent investigates selected questions
-> Parseltongue verifies symbols, paths, tests, and API consumers
-> LLM produces a bounded review summary with residual uncertainty
```

Requirements:

- Review questions must be traceable to changed evidence.
- Risk categories cannot be based on node count alone.
- Public API, tests, generated code, and unresolved dynamic calls require separate treatment.

### BW-04: Change Set -> Test Candidates -> Test Results -> Graph Reassessment

```text
Changed entities
-> Parseltongue selects directly and transitively related tests
-> test runner executes bounded candidate set
-> LLM interprets failures and coverage gaps
-> Parseltongue checks whether failing tests share graph paths with the change
-> LLM recommends additional tests or declares residual risk
```

Requirements:

- Test exclusion during ingestion would invalidate this workflow.
- Test edges need evidence and language-specific semantics.
- The result must say which relevant tests could not be mapped.

### BW-05: Domain Concepts -> Structural Communities -> Human Labels -> Rule Validation

```text
User or LLM supplies domain concepts
-> graph engine computes structural communities without semantic claims
-> LLM proposes names and business interpretations
-> Parseltongue checks cohesion, cross-boundary edges, and known folder/module evidence
-> human accepts, edits, or rejects proposed labels
-> accepted labels remain annotations, not canonical parser facts
```

Requirements:

- Random seeds and algorithm parameters must be recorded.
- Community labels must not be presented as objectively correct.
- Structural clusters and semantic labels are different data layers.

### BW-06: Task Intent -> Context Plan -> LLM Gap Report -> Follow-Up Package

```text
Task intent and token budget
-> LLM names focus symbols and desired operation
-> ContextPlanner returns ranked evidence and budget report
-> LLM identifies missing evidence or ambiguity
-> planner expands a specific path, relation, or source slice
-> LLM proceeds with a bounded, cited context package
```

Requirements:

- Token estimates and actual rendered tokens should both be recorded.
- Truncation is a state, not a silent implementation detail.
- Every context item needs a selection reason.

### BW-07: File Event -> Incremental Candidate -> Full-Parity Guard -> Agent Notification

```text
File change
-> watcher creates normalized change event
-> indexer builds candidate revision
-> parity guard compares incremental result with full reparse on sampled or protected paths
-> snapshot is published or rejected
-> agent is notified of current revision and affected symbols
-> agent asks follow-up impact query
```

Requirements:

- The service owning watcher resources must live as long as the server.
- Delete, rename, and configuration changes must be first-class events.
- A failed candidate revision must not replace the last known-good snapshot.

### BW-08: Unresolved Reference -> LLM Classification -> Deterministic Recheck

```text
Parser emits unresolved reference with source evidence
-> LLM proposes possible category or target using nearby context
-> resolver checks symbols, imports, SCIP/LSP data, and repository boundaries
-> confirmed target becomes a new derived fact
-> unconfirmed proposal remains an annotation with confidence
```

Requirements:

- The LLM cannot mutate canonical references directly.
- Confirmation policy must be deterministic and testable.
- Unresolved references remain queryable and visible in coverage.

## 7. Journey x Architecture Timelines

The time horizons below describe plausible adoption and system evolution. They are not delivery estimates.

## Timeline 1: The First Ten Minutes of Repository Orientation

### PRD frame

- **User:** Developer or coding agent entering an unfamiliar repository.
- **Trigger:** A bug, feature request, review, or handoff begins before the user knows the code layout.
- **Current workaround:** README, file tree, ripgrep, broad file reads, and improvised summaries.
- **Job:** Reach a defensible first action without reading the whole repository.
- **Desired outcome:** Identify entry points, relevant symbols, module shape, and index limitations in one short sequence.

### Architecture pairing

Local embedded facts in SQLite/Turso, a fast immutable RAM projection, lexical symbol indexes, and a guided query surface:

```text
index_repository -> check_index_job -> repository_overview
-> search_code -> explain_symbol -> build_context
```

### Timeline

- **Minute 0:** User points Parseltongue at a repository. The product discovers files and immediately reports root, ignore source, detected languages, and pending index job.
- **Minute 2:** Partial orientation becomes available, explicitly marked partial. The user can inspect folder and language coverage while deeper relationships build.
- **Minute 5:** Entry points, public symbols, major modules, and unresolved-reference counts appear. Results cite snapshot ID and source revision.
- **Minute 10:** The user selects a task focus and receives a small context package with source slices and next-query suggestions.
- **Week 1:** Repeated usage teaches the agent to orient before searching broadly. The user notices fewer irrelevant file reads.
- **Month 1:** Repository-specific hot paths and common tasks can be cached, but cache state remains subordinate to snapshot revision.
- **Quarter 1:** Optional SCIP indexes improve exact navigation without changing the orientation API.

### Behavioral requirements

- Partial index results must look visibly partial.
- Unsupported and skipped files must be counted and inspectable.
- Orientation must work without embeddings or an external database.
- The experience must beat `rg` plus README on time to first useful source location.

### Failure modes

- A beautiful map that cannot lead to source.
- Startup ceremony longer than the task itself.
- Treating folder names as architecture truth.
- Returning external-library symbols as repository hotspots.

### Evidence roots

[D01 User Journeys](../research000/archive/archive-docs-v2/archive-p2/D01_UserJourneys.md), [Research002 J002](../research002/J002.md), [Tree-sitter Patterns 3](../research003/tree-sitter-patterns-3.md), and the current [README](../../README.md).

## Timeline 2: Safe Context Before an Agent Edit

### PRD frame

- **User:** Developer delegating a change to a coding agent.
- **Trigger:** The agent is about to modify a function, type, route, or configuration contract.
- **Fear:** The model edits a locally plausible implementation while missing a caller, test, or contract.
- **Job:** Obtain the smallest sufficient edit context and impact boundary before changing code.
- **Desired outcome:** Agent either proceeds with bounded evidence or pauses with a precise uncertainty.

### Architecture pairing

Job-shaped `prepare_edit` orchestration over a backend-neutral query service. The orchestration combines exact symbol lookup, callers/callees, bounded impact, relevant tests, source slices, and context planning.

### Timeline

- **Opening move:** Agent sends intent, focus symbol, repository revision, and token budget.
- **Seconds later:** Parseltongue resolves stable identity and returns direct relationships, evidence confidence, and relevant source spans.
- **Before editing:** Agent states intended files and expected graph effect. High-risk unresolved edges cause a pause rather than fabricated certainty.
- **After editing:** The new snapshot is compared with the baseline. Unexpected changed relationships become explicit review items.
- **Week 1:** Users learn a consistent pre-edit ritual. Adoption depends on latency staying below the cost of manual search.
- **Month 1:** The product learns task templates, not hidden semantic facts: rename, bug fix, API change, and refactor preparation.
- **Quarter 1:** Teams can require an edit-preparation receipt for high-risk paths.

### Behavioral requirements

- Stable entity IDs cannot depend on current line numbers.
- Caller/callee direction must be defined once and parity-tested.
- The package must include relevant tests or say why tests are unavailable.
- Internal `NodeIndex` values must never escape.
- Results are deterministically ordered by stable keys.

### Bidirectional loop

This is the primary use of BW-01 and BW-06.

### Evidence roots

[A02 Shreyas Ideation](../research000/archive/archive-docs-v2/archive-p2/A02ShreyasIdeation20260124.md), [Research002 J003](../research002/J003.md), and [Tree-sitter Patterns 4](../research003/tree-sitter-patterns-4.md).

## Timeline 3: Post-Edit Proof and Test Selection

### PRD frame

- **User:** Coding agent and developer immediately after a patch.
- **Trigger:** Files changed and the user needs evidence that the patch stayed within intent.
- **Current workaround:** Run broad tests, inspect `git diff`, and hope semantic effects are obvious.
- **Job:** Show structural change, affected tests, changed contracts, and residual unknowns.
- **Desired outcome:** A short proof package that can support continuation, rollback, or review.

### Architecture pairing

Immutable base and candidate graph snapshots, semantic diff, test graph, and a verification orchestrator. Durable storage retains snapshot metadata; graph projections are disposable.

### Timeline

- **Immediately after save:** Watcher builds a candidate index revision without mutating the currently published snapshot.
- **Within the feedback budget:** Candidate passes parser, source-span, graph-integrity, and deletion cleanup checks.
- **Publication:** Snapshot swaps atomically and reports changed entities, edges, coverage, and unresolved deltas.
- **Test selection:** Related tests are ranked by direct references, impacted flow, and historical metadata when available.
- **After tests:** Results flow back into the agent, which explains pass/fail evidence and remaining gaps.
- **Month 1:** Branch snapshots and baseline snapshots support review and CI using the same semantics.
- **Quarter 1:** Teams can compare predicted impact with actual failing tests to calibrate selection quality.

### Behavioral requirements

- Full ingestion and incremental ingestion must produce the same canonical graph.
- Failed indexing preserves last known-good data and returns a failed revision receipt.
- Deleted files remove all derived entities, chunks, edges, and caches.
- The product must distinguish structural proof from runtime behavioral proof.

### Bidirectional loop

This combines BW-01, BW-04, and BW-07.

### Evidence roots

[Unified Diff PRD](../research000/archive/archive-docs-v2/archive-p2/PRD-ARCH-UNIFIED.md), [Incremental Indexing RCA](../research001/unclassified/RCA-Incremental-Indexing-Failure.md), and [Tree-sitter Patterns 5](../research003/tree-sitter-patterns-5.md).

## Timeline 4: Pull Request Triage and Review

### PRD frame

- **User:** Maintainer, tech lead, or platform engineer handling many changes.
- **Trigger:** A branch or pull request needs review prioritization.
- **Current workaround:** Read every diff in roughly the same order or trust line counts.
- **Job:** Identify the few changed flows that deserve deep review and produce evidence-linked questions.
- **Desired outcome:** Faster review without pretending low graph impact means correct code.

### Architecture pairing

Headless CLI or HTTP service, baseline index artifact, branch index artifact, semantic diff engine, policy layer, and portable JSON/SARIF/Markdown output. Storage may be local SQLite in CI or a shared service adapter.

### Timeline

- **PR opened:** CI indexes changed files or restores the baseline and applies branch deltas.
- **Analysis:** The service returns changed symbols, public contract changes, impacted paths, relevant tests, and coverage degradation.
- **LLM pass:** An agent converts evidence into targeted review questions, each with source links.
- **Reviewer pass:** Human accepts, rejects, or investigates questions. The graph verifies claimed paths.
- **Merge decision:** Policy can require acknowledgment of high-confidence contract breaks, but should not block solely on a generic blast-radius threshold.
- **Month 1:** Teams compare flagged risks with actual review findings and production escapes.
- **Quarter 1:** Repository-specific policies emerge around public APIs, migrations, security boundaries, and generated code.

### Behavioral requirements

- The same request produces equivalent normalized results on local and shared backends.
- Changed-line count is never substituted for semantic impact.
- Policy is repository-owned configuration with provenance.
- The report includes what could not be analyzed.

### Bidirectional loop

This implements BW-03 and BW-04.

### Evidence roots

[D01 User Journeys](../research000/archive/archive-docs-v2/archive-p2/D01_UserJourneys.md), [Research002 J002](../research002/J002.md), and [Feature Master Table](../research000/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md).

## Timeline 5: Incident Root-Cause Investigation

### PRD frame

- **User:** Developer or incident responder under time pressure.
- **Trigger:** Failing test, exception, production symptom, or regression.
- **Current workaround:** Search logs, grep symbols, and manually reconstruct call paths.
- **Job:** Collapse the search space to plausible code paths and the next falsifiable experiment.
- **Desired outcome:** Reduce time to a testable root-cause hypothesis, not produce an authoritative diagnosis from static structure alone.

### Architecture pairing

Tree-sitter baseline enriched by SCIP/LSP when available, source-spanned call/reference edges, reverse path queries, optional runtime trace imports, and a context package builder.

### Timeline

- **Minute 0:** User submits error text, failing test, or symbol.
- **Minute 1:** Search finds exact and fuzzy candidates, explains match reasons, and reports index freshness.
- **Minute 3:** User or LLM selects a candidate; graph traces reverse callers, relevant error handlers, and related tests.
- **Minute 5:** LLM proposes a hypothesis and asks for one narrower graph path or source slice.
- **Minute 8:** Parseltongue returns evidence and contradictory facts. The LLM revises or rejects the hypothesis.
- **After fix:** Post-edit diff and tests close the loop.
- **Quarter 1:** Optional runtime traces can calibrate which static paths are actually exercised.

### Behavioral requirements

- Static reachability cannot be labeled runtime causality.
- Paths include the relationship evidence used at each hop.
- Search empty, graph empty, unresolved, and stale are separate states.
- Latency and output size should match incident urgency.

### Bidirectional loop

This is BW-02 followed by BW-01.

### Evidence roots

[Feature Master Table](../research000/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md), [Visualization Journey V2](../research000/archive/archive-docs-v2/web-ui/EIGHT_NEW_USER_JOURNEYS_VERSION_TWO.md), and [Tree-sitter Patterns 4](../research003/tree-sitter-patterns-4.md).

## Timeline 6: Live Refactoring with a Trustworthy Index

### PRD frame

- **User:** Developer performing a multi-step refactor.
- **Trigger:** The code graph changes repeatedly over minutes or hours.
- **Historical pain:** The watcher claimed health while losing events or failing to reindex.
- **Job:** Keep analysis aligned with the code while preserving a last known-good state.
- **Desired outcome:** Each refactor step has a current revision, visible diff, and recoverable failure state.

### Architecture pairing

Long-lived watcher ownership, bounded event queue, coarse file invalidation first, candidate index revisions, immutable snapshot publication, and full-reparse parity sampling.

### Timeline

- **First change:** Watcher reports event receipt, normalization, debounce decision, and candidate revision ID.
- **Reindex:** Changed file is reparsed; old derived records are replaced atomically by repository/file revision.
- **Validation:** Candidate graph checks source spans, dangling internal IDs, orphan derived rows, and coverage changes.
- **Publish:** Queries move to the new immutable snapshot only after validation.
- **Failure:** Last known-good snapshot remains active and the failed candidate is inspectable.
- **Week 1:** File-level invalidation proves reliability across create, modify, delete, and rename events.
- **Quarter 1:** Tree-sitter incremental edits are enabled only for languages and operations with full-reparse parity evidence.

### Behavioral requirements

- `watcher_running` means event delivery and revision publication are healthy, not merely that startup returned `Ok`.
- Health exposes last event time, last successful revision, queue depth, failed revisions, and lag.
- Parser instances are reset after timeout or cancellation.
- Config changes invalidate the correct caches.

### Bidirectional loop

This is BW-07, with the agent receiving current revision evidence before each follow-up query.

### Evidence roots

[File Watcher Debug Report](../research001/unclassified/File-Watcher-Debug-20260202.md), [Incremental Architecture](../research000/archive/archive-docs-v2/archive-p2/D04_Incremental_Indexing_Architecture.md), and [Tree-sitter Patterns 5](../research003/tree-sitter-patterns-5.md).

## Timeline 7: Adding or Repairing a Language Pack

### PRD frame

- **User:** Parseltongue maintainer or ecosystem contributor.
- **Trigger:** A new language is added, grammar updates, or captures drift.
- **Current workaround:** Add a parser crate and broad query, then discover silent semantic errors later.
- **Job:** Ship language support with measured symbol, edge, chunk, and source-span quality.
- **Desired outcome:** Language support is a versioned product capability, not a boolean in an extension map.

### Architecture pairing

Central `LanguageRegistry`, per-language query assets, typed capture ontology, fixture repositories, golden expectations, optional compiler index adapter, and cache keys containing grammar/query/extractor versions.

### Timeline

- **Day 0:** Contributor registers extensions, grammar version, capabilities, and unsupported constructs.
- **Day 1:** All query assets compile and capture-name lint passes.
- **Day 2:** Human-authored fixtures establish expected definitions, references, calls, imports, chunks, comments, and errors.
- **Week 1:** Full repository fixtures compare node and edge classes; unresolved cases are reported, not hidden.
- **Release:** Packaged binary smoke test proves grammar assets actually ship.
- **Month 1:** Drift telemetry shows which constructs fail in real repositories without uploading source.
- **Quarter 1:** SCIP/compiler enrichment can raise confidence for supported projects while preserving Tree-sitter fallback.

### Behavioral requirements

- A language can declare partial capabilities.
- Query compilation is necessary but insufficient.
- Language-specific node kinds are retained beside normalized kinds.
- Embedded languages and injections have explicit ownership.
- A grammar upgrade cannot reuse stale cached facts.

### Bidirectional loop

BW-08 can assist unresolved-reference triage, but only deterministic checks can promote a proposal to a fact.

### Evidence roots

[Tree-sitter Patterns 1](../research003/tree-sitter-patterns-1.md), [Tree-sitter Patterns 2](../research003/tree-sitter-patterns-2.md), and [Tree-sitter Patterns 5](../research003/tree-sitter-patterns-5.md).

## Timeline 8: Architecture Refactoring and Module Extraction

### PRD frame

- **User:** Tech lead, staff engineer, or architect.
- **Trigger:** A monolith, crate, or service has become difficult to change.
- **Current workaround:** Folder diagrams, intuition, metrics dashboards, and workshops.
- **Job:** Identify candidate boundaries, violations, and a reversible migration order.
- **Desired outcome:** Produce hypotheses and experiments, not claim that an algorithm discovered the one true architecture.

### Architecture pairing

Canonical code facts projected into an analytics graph. Exact structural operations use the reference engine; PageRank, betweenness, k-core, SCC, and seeded community detection may use pluggable algorithm engines. Semantic labels remain annotations.

### Timeline

- **Week 0:** Architect defines scope, relation types, and exclusions. Baseline graph quality is reviewed first.
- **Week 1:** SCCs, cross-boundary edges, public contracts, and hotspots identify constraints.
- **Week 2:** Seeded communities and coupling metrics generate candidate boundaries with caveats.
- **Week 3:** LLM proposes domain labels and migration stories; graph checks cohesion and impacted paths.
- **Month 2:** Team runs a small extraction and compares predicted with actual changes.
- **Quarter 2:** Calibrated rules support later migrations and architecture-policy checks.

### Behavioral requirements

- Every algorithm records directedness, weights, seed, tolerance, and projection filters.
- Community labels are never canonical parser facts.
- Approximate algorithms are labeled approximate.
- Users can inspect the edges responsible for a metric or boundary.
- Results are reproducible enough for before/after comparison.

### Bidirectional loop

This uses BW-05 and BW-03: semantic interpretation moves back and forth with deterministic structure and change evidence.

### Evidence roots

[Visualization Journey V3](../research000/archive/archive-docs-v2/web-ui/EIGHT_NEW_USER_JOURNEYS_VERSION_THREE.md), [Feature Master Table](../research000/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md), and [Tree-sitter Patterns 3](../research003/tree-sitter-patterns-3.md).

## Timeline 9: Token-Budgeted Context for a Long Agent Task

### PRD frame

- **User:** Coding agent operating under a finite context window.
- **Trigger:** A task spans multiple files and relationships.
- **Current workaround:** Read whole files, compress conversation, and repeatedly rediscover context.
- **Job:** Spend tokens on evidence that changes the next decision.
- **Desired outcome:** Bounded context packages that can expand deliberately without hiding omissions.

### Architecture pairing

Separate search, graph retrieval, source slicing, tokenization, ranking, and rendering stages. Durable facts store source once; context items reference byte spans. A planner explains its selection and supports continuation.

### Timeline

- **Task start:** Agent supplies goal, current files, focus symbols, and available budget.
- **Plan:** Context planner estimates candidates and returns a dry-run plan.
- **Package 1:** Direct definitions, callers, callees, tests, and contracts consume the first budget slice.
- **Reasoning:** Agent reports a specific evidence gap.
- **Package 2:** Planner expands one path or relation without repeating current-file content.
- **Long session:** Important evidence is persisted by stable IDs and source revisions, while summaries retain pointers.
- **Task close:** Final explanation lists used evidence, omitted categories, and stale revisions.

### Behavioral requirements

- Token accounting is based on the actual renderer/tokenizer where possible.
- The planner cannot silently reduce hop depth or scope.
- Selection reasons are machine-readable.
- Pointer retrieval checks source revision and detects stale spans.
- Context quality is measured by task outcomes, not token reduction alone.

### Bidirectional loop

This is BW-06 repeated throughout the task.

### Evidence roots

[Quick Reference 7 PRDs](../research001/PRD-research-20260131v1/QUICK_REFERENCE_7_PRDS.md), [Tree-sitter Patterns 4](../research003/tree-sitter-patterns-4.md), and [Research002 J002](../research002/J002.md).

## Timeline 10: Trust Recovery After an Empty or Suspicious Answer

### PRD frame

- **User:** Any developer or agent receiving zero results, surprising hotspots, or missing relationships.
- **Trigger:** An answer conflicts with visible code or prior knowledge.
- **Historical pain:** The API returned successful but empty smart context and hid incomplete indexing.
- **Job:** Determine whether the answer means none, unsupported, skipped, stale, unresolved, or failed.
- **Desired outcome:** Repair confidence without dropping into database internals.

### Architecture pairing

An evidence and diagnostics ledger attached to every index revision and query response. Coverage is queryable by repository, folder, language, file, entity kind, relationship kind, and failure reason.

### Timeline

- **Suspicious answer:** Response includes snapshot ID, coverage summary, unresolved count, truncation state, and diagnostic links.
- **One click/query:** User sees whether the focus file was eligible, parsed, captured, indexed, and projected.
- **Drill-down:** Debug view connects source bytes -> parse node -> query capture -> entity -> relationship -> context item.
- **Repair:** User changes config, language pack, or scope and starts a new candidate revision.
- **Verification:** New result is compared with the previous revision by node and edge class.
- **Month 1:** Frequent diagnostic classes guide language-pack and product-quality work.

### Behavioral requirements

- Empty is a valid result only when coverage and execution prove it.
- Errors are typed and actionable.
- Debug tools are read-only by default.
- Diagnostics never expose source outside repository containment policy.
- Installed artifacts receive the same smoke tests as development builds.

### Bidirectional loop

The user or agent challenges a result; the system returns stage-specific evidence; the next query verifies the repair.

### Evidence roots

[Critical Bugs Analysis](../research001/unclassified/CRITICAL-BUGS-ANALYSIS-v143.md), [Tree-sitter Patterns 5](../research003/tree-sitter-patterns-5.md), and [Research003 Completion Audit](../research003/completion-audit.md).

## Timeline 11: Data Lineage and API Contract Change

### PRD frame

- **User:** Data engineer, API maintainer, or migration owner.
- **Trigger:** A schema, DTO, event, route, or database contract changes.
- **Current workaround:** Search names across layers and manually infer data movement.
- **Job:** Trace producers, transformations, consumers, and tests across language boundaries.
- **Desired outcome:** A bounded contract-impact report with explicit gaps where static evidence is insufficient.

### Architecture pairing

Typed relation ontology inspired by code-property graphs, SCIP facts where available, Tree-sitter extraction for broad coverage, and configurable adapters for routes, schemas, migrations, and infrastructure files.

### Timeline

- **Change proposed:** User identifies contract symbol or source span.
- **Structural pass:** Parseltongue finds definitions, imports, calls, route bindings, and schema references.
- **Interpretation pass:** LLM proposes producer/transformer/consumer roles.
- **Verification pass:** Deterministic rules confirm supported roles and retain the rest as annotations.
- **Migration plan:** Impacted services, files, owners, tests, and unresolved dynamic consumers are reported.
- **After rollout:** Actual changed artifacts and failures calibrate adapter quality.

### Behavioral requirements

- "Data flow" is not inferred from a generic call edge alone.
- Relation types carry resolver and confidence.
- Cross-language boundaries preserve source evidence on both sides.
- Unknown runtime consumers remain visible.

### Evidence roots

[Visualization Journey V3](../research000/archive/archive-docs-v2/web-ui/EIGHT_NEW_USER_JOURNEYS_VERSION_THREE.md), [Code-intelligence tool research](../research002/J002.md), and [Tree-sitter Patterns 2](../research003/tree-sitter-patterns-2.md).

## 8. Cross-Timeline Architecture Options

| Architecture | Durable store | Graph execution | Semantic quality | Operational shape | Best-fit timelines | Main risk |
|---|---|---|---|---|---|---|
| Local embedded reference | SQLite/Turso | Rust in RAM | Tree-sitter baseline | One binary, local-first | 1, 2, 3, 6, 9, 10 | Memory and projection rebuild cost at very large scale |
| Compiler-enriched local | SQLite/Turso plus SCIP import | Rust in RAM | Tree-sitter plus SCIP/LSP | Local indexers when available | 2, 3, 5, 7, 11 | Build integration and uneven language support |
| Shared service | PostgreSQL or shared SQL | Service-owned RAM snapshots | Mixed indexers | Team daemon/API | 4, 8, 11 | Operations, tenancy, stale shared state |
| External graph adapter | SQL facts plus Neo4j projection | Neo4j GDS/Cypher | Depends on fact quality | Separate graph service | 8 and large monorepos | Backend semantics leaking into product APIs |
| Ephemeral analysis | In-memory ingest plus SQLite snapshot | Rust in RAM | Tree-sitter baseline | Fast disposable CI jobs | 3, 4, 10 | Persistence and crash recovery |
| Accelerator path | SQLite/Turso facts | GraphBLAS, C/C++, GPU, or service | Same canonical facts | Optional capability | 8 at extreme scale | Parameter and result parity drift |

### Working hybrid

The most reversible architecture is:

1. SQLite/Turso as canonical durable facts.
2. A versioned immutable Rust graph snapshot as the reference engine.
3. Optional SCIP/LSP imports for stronger semantics.
4. Separate lexical or full-text search.
5. Optional analytics adapters behind capability and parity contracts.
6. HTTP/CLI/tool schemas defined by product behavior, not backend query languages.

## 9. Cross-Cutting Product Requirements

### PR-TRUST-001: Honest empty results

When a query returns no entities or relationships, the response shall distinguish:

- proven no match;
- focus entity not found;
- file not indexed;
- language unsupported;
- relation unresolved;
- scope excluded;
- result truncated;
- backend or projection failure.

### PR-IDENTITY-002: Stable public identity

Entity identity shall survive unrelated line shifts. File revision and source span may change without changing durable entity identity. Ambiguous matches shall be reported rather than silently merged.

### PR-FRESHNESS-003: Revision-visible queries

Every query response shall identify repository, source revision, graph snapshot, index completion state, and last successful update.

### PR-PROVENANCE-004: Evidence-bearing relationships

Every relationship returned to an agent shall have source evidence, resolver kind, confidence, and unresolved status where applicable.

### PR-BUDGET-005: Explicit context budgeting

Every context response shall report requested budget, estimated and actual tokens, included items, omitted categories, and truncation.

### PR-PARITY-006: Backend-neutral behavior

All supported backends shall pass the same normalized contract suite for exact operations. Numeric and stochastic algorithms shall declare tolerances, seeds, and parameters.

### PR-INCREMENTAL-007: Full/incremental equivalence

Indexing a repository from scratch and reaching the same content through incremental updates shall produce equivalent canonical facts after normalization.

### PR-DELETE-008: Derived-data cleanup

Deleting or renaming a file shall remove or relocate every derived entity, edge, chunk, cache entry, and search record associated with the previous revision.

### PR-SAFETY-009: Read-only agent queries

Agent-facing graph and debug queries shall be read-only by default. Proposed semantic annotations require a separate, auditable write path.

### PR-DETERMINISM-010: Reproducible output

Exact query output shall use deterministic ordering. Stochastic analytics shall expose seed and quality metadata.

## 10. Architecture and Code Patterns Worth Preserving

### Pattern: Query coarse candidates, traverse for detail

Tree-sitter queries should locate broad constructs. Field-name access and explicit traversal should refine names, bodies, parameters, and relationships. A single giant query becomes brittle across grammar versions.

### Pattern: Source content stored once, records store spans

Documents own bytes. Entities, references, and chunks point to versioned byte spans. Context rendering slices source at the final boundary, enabling low-copy processing and consistent provenance.

### Pattern: File discovery is policy

Ignore rules, symlink policy, hidden files, generated files, size limits, repository containment, and supported extensions affect product truth. They belong in index metadata and coverage reports.

### Pattern: Bounded pipelines and backpressure

Discovery, parsing, resolution, persistence, projection, and rendering have different resource profiles. Bounded queues prevent file discovery or parsing from outrunning storage. A single writer can be correct if the queue exposes lag and flush semantics.

### Pattern: Candidate revision and atomic publish

Never expose half-applied graph updates. Build, validate, and publish a complete candidate revision. Preserve the last known-good snapshot on failure.

### Pattern: Reference implementation before optimization

The simplest correct BFS/SCC/scope implementation becomes the semantic oracle. Parallel, database-native, or GPU implementations must match it on fixtures before serving requests.

### Pattern: Capability matrix instead of false uniformity

Each language pack and backend declares what it can do: definitions, references, calls, imports, inheritance, tests, incremental parsing, analytics, full-text search, or persistence. Unsupported capabilities return explicit typed errors.

### Pattern: Golden drift by semantic class

Golden tests should report definitions added, calls removed, imports unresolved, spans changed, and language-specific differences. A single giant snapshot diff is difficult to diagnose.

## 11. Contradictions and Unresolved Tensions

### 11.1 MCP was both promoted and rejected

Some documents position MCP as the universal interface. A later feature synthesis says MCP was dropped because of token overhead. Recent competitor research again finds agent-callable surfaces valuable.

Observation: transport is not the product. The useful question is whether a small job-shaped tool surface reduces total tokens and improves decisions compared with CLI or HTTP. This needs measurement using the same workflows.

### 11.2 Stable identity is still conceptually unsettled

Historical approaches include stripping line ranges, birth timestamps, semantic paths, content hashes, and matching heuristics. Each handles different cases. File renames, duplicate identical functions, overloads, and moved symbols remain difficult.

Observation: public identity, matching evidence, and current location should be separate concepts.

### 11.3 Tree-sitter breadth conflicts with semantic precision

Tree-sitter enables broad local support. Exact cross-file calls and type-directed references often require compiler knowledge. Presenting name matches as calls would undermine the safe-edit promise.

Observation: confidence and resolver provenance are mandatory, and SCIP/compiler enrichment is an optional precision tier.

### 11.4 Test exclusion conflicts with test selection

Some ingestion designs intentionally exclude tests from the production graph. Several high-value journeys depend on identifying relevant tests.

Observation: tests may need a separate entity class and projection policy, not exclusion from canonical facts.

### 11.5 Algorithm abundance can hide product weakness

The corpus proposes SCC, k-core, entropy, CK metrics, PageRank, betweenness, Leiden, spectral partitioning, Node2Vec, UMAP, graph kernels, and many visualizations.

Observation: an algorithm is useful only when attached to a user decision and an explainable quality bar. More algorithms do not compensate for inaccurate edges or stale indexes.

### 11.6 Local simplicity conflicts with team scale

SQLite/Turso plus RAM projection is attractive for one developer and CI. Shared repositories, many simultaneous users, and cross-repository graphs may eventually need a service.

Observation: a backend-neutral product service preserves the option to add shared storage or Neo4j without forcing it into initial activation.

### 11.7 Historical metrics are not a PMF study

Many documents assign PMF scores and precise benefits without interviews, cohorts, or benchmark artifacts.

Observation: these scores are useful hypotheses. They should not determine sequencing without workflow experiments.

## 12. Source Map

| Source | Main evidence extracted | Evidence character |
|---|---|---|
| [Current README](../../README.md) | 26 endpoints, current limitations, current workflow | Observed-current description |
| [Feature Master Table](../research000/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md) | Broad inventory of 218 proposed and shipped features | Designed and speculative synthesis |
| [D01 User Journeys](../research000/archive/archive-docs-v2/archive-p2/D01_UserJourneys.md) | Safe edits, CI gates, review, maintainer workflows | Designed journeys |
| [A02 Shreyas Ideation](../research000/archive/archive-docs-v2/archive-p2/A02ShreyasIdeation20260124.md) | User segments, interface failure, Rust niche, graph-first thinking | Strategic notes |
| [Quick Reference 7 PRDs](../research001/PRD-research-20260131v1/QUICK_REFERENCE_7_PRDS.md) | preview/pointer, budget estimation, cursors, export, cache, pipelines, planner | Designed agent-memory features |
| [Critical Bugs Analysis](../research001/unclassified/CRITICAL-BUGS-ANALYSIS-v143.md) | false-positive health, empty context, null keys, missing search | Observed-historical failures |
| [Incremental RCA](../research001/unclassified/RCA-Incremental-Indexing-Failure.md) | orphan route, batch replacement, unstable IDs | Observed-historical and designed fix |
| [File Watcher Debug](../research001/unclassified/File-Watcher-Debug-20260202.md) | service lifetime, event silence, lock failures | Observed-historical failure |
| [Windows Storage Specs](../research000/SPEC-v172-windows-mem-backup.md) | RocksDB -> Sled -> memory/SQLite evolution | Historical architecture evidence |
| [Research002 J002](../research002/J002.md) | job-shaped agent tools and evidence-router thesis | Recent comparative research |
| [Research002 J003](../research002/J003.md) | pre-edit context, impact, trace, tests, review surface | Recent comparative research |
| [Tree-sitter Patterns 1](../research003/tree-sitter-patterns-1.md) | parser runtime, spans, traversal, incremental safeguards | Recent repo-grounded patterns |
| [Tree-sitter Patterns 2](../research003/tree-sitter-patterns-2.md) | language packs, query layers, typed ontology | Recent repo-grounded patterns |
| [Tree-sitter Patterns 3](../research003/tree-sitter-patterns-3.md) | canonical indexing pipeline, evidence-bearing graph | Recent repo-grounded patterns |
| [Tree-sitter Patterns 4](../research003/tree-sitter-patterns-4.md) | token budgets, agent tools, context packages | Recent repo-grounded patterns |
| [Tree-sitter Patterns 5](../research003/tree-sitter-patterns-5.md) | verification matrix and footgun registry | Recent repo-grounded patterns |

## 13. Questions to Preserve for Product Discovery

1. At what exact moment would a developer invoke Parseltongue without being reminded?
2. Does pre-edit context reduce missed files or merely add ceremony?
3. Can post-edit graph diff predict relevant tests better than changed-path heuristics?
4. What level of unresolved references makes blast radius misleading?
5. Which languages can meet a safe-edit quality bar with Tree-sitter alone?
6. Is local indexing fast enough that a user will tolerate it for a five-minute task?
7. Which evidence fields make users trust or reject an answer?
8. Does a grouped agent tool surface outperform raw HTTP endpoint exposure?
9. What is the smallest repository fixture that catches direction, duplicate-edge, scope, delete, and identity bugs?
10. When does a team need a shared service instead of local artifacts?
11. Which advanced algorithm changes a real decision often enough to justify its complexity?
12. Can SQLite/Turso durable facts plus RAM graph meet monorepo memory and latency targets?
13. Should tests be a separate graph projection while remaining canonical indexed facts?
14. How should the product distinguish static possibility from runtime likelihood?
15. Which user segment has both frequent pain and willingness to change workflow?

## 14. Observation Summary

The corpus contains many possible products: parser platform, graph analytics suite, architecture dashboard, visualization IDE, CI gate, code-search engine, and agent memory layer. The repeated evidence points most strongly toward a narrower center:

> Parseltongue is a local-first evidence router that helps an agent or developer inspect the right code before an edit and prove the structural consequences afterward.

Everything else can be evaluated by whether it strengthens that loop, expands it into an adjacent high-value journey, or distracts from trust and activation.
