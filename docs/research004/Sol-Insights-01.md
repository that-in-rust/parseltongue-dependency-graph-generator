<!-- markdownlint-disable MD013 MD024 MD060 -->

# Solution Insights 01: PMF Opportunities and Backend-Neutral Product Direction

- **Date:** 2026-07-10
- **Input:** [Sol-Observations-01.md](Sol-Observations-01.md) and the repository `docs/` corpus
- **Mode:** Product judgment using Shreyas-style problem, quality, strategy, and execution lenses

## 1. Executive Answer

Parseltongue should not lead with "a graph database for code," "26 graph endpoints," or "more graph algorithms." Its strongest product wedge is:

> Before an agent edits code, Parseltongue returns the smallest trustworthy edit context and impact boundary. After the edit, it proves the structural change, identifies relevant tests, and reports residual uncertainty.

This is a repeated, urgent, measurable workflow. It also makes the product's existing graph capabilities legible to users without asking them to understand graph terminology.

The architecture should be local-first and backend-neutral:

```text
Tree-sitter baseline + optional SCIP/compiler enrichment
                         |
                         v
              Canonical evidence records
                         |
                         v
                SQLite/Turso storage
                         |
          +--------------+---------------+
          |                              |
          v                              v
immutable Rust graph snapshot      lexical/source retrieval
          |                              |
          +--------------+---------------+
                         v
           job-shaped product workflows
```

Neo4j, GraphBLAS, GPU algorithms, or another graph engine can later be optional execution adapters. They should never define the product's HTTP semantics.

## 2. Shreyas-Style Product Frame

### 2.1 Target user

The best initial segment is not every software developer. It is:

- an AI-first developer or maintainer;
- working in a non-trivial repository;
- delegating or reviewing code changes frequently;
- unable to trust file search alone for dependency impact;
- willing to run a local tool because a wrong edit has a visible cost.

Secondary segments are coding-agent builders, open-source maintainers, and platform teams reviewing AI-generated changes.

### 2.2 The high-intensity moment

The user has already decided to change code. They are asking:

- "What should the agent read before touching this?"
- "What could this break?"
- "Which tests and contracts matter?"
- "Did the patch change only what we intended?"

The user's urgency exists before Parseltongue enters the scene. That is a good wedge. General architecture exploration often requires the product to manufacture urgency.

### 2.3 Current alternatives

- `rg`, file search, and manual reading;
- language-server find references;
- broad repository context dumps;
- coding-agent native exploration;
- test-all and CI;
- existing code-graph or review tools;
- intuition and reviewer experience.

Parseltongue wins only if it combines these into a faster and more trustworthy decision. A graph answer that is slower, stale, or semantically weaker than language-server references has no advantage.

### 2.4 Differentiation

Potential differentiation is not the existence of Tree-sitter, SQLite, petgraph, or PageRank. Those are available to competitors.

The defensible combination is:

1. broad local parsing with explicit evidence quality;
2. stable identities and change-aware indexing;
3. deterministic, job-shaped graph workflows;
4. token-budgeted context packages;
5. bidirectional pre-edit and post-edit proof;
6. backend and parser enrichment without changing the user contract;
7. unusually strong diagnostics when the answer cannot be trusted.

### 2.5 Product risk versus execution risk

- **Product risk:** Will developers actually invoke a pre-edit/post-edit workflow, or will it feel like ceremony?
- **Quality risk:** Are calls, references, tests, and source spans accurate enough for the word "safe"?
- **Activation risk:** Can a user obtain value within one task and less than ten minutes of setup?
- **Strategy risk:** Will native coding agents or language servers absorb the workflow?
- **Execution risk:** Can incremental indexing and backend abstraction remain correct across languages and operating systems?

The next phase should attack product and quality risk before expanding algorithm breadth.

## 3. Product Quality Model

### Expected quality: failure here destroys trust

| Expected behavior | Required product response |
|---|---|
| Index reflects current code | Report source revision, snapshot ID, watcher lag, and last successful update |
| Symbol identity remains stable | Separate durable identity from current source span and matching evidence |
| Caller/callee answers are directionally correct | Use canonical semantics and backend parity fixtures |
| Empty means empty | Distinguish none, missing, skipped, stale, unresolved, truncated, and failed |
| Results lead back to code | Include source spans and relationship evidence |
| Deletes disappear completely | Remove entities, edges, chunks, caches, and search records |
| Language support is honest | Publish a capability and coverage matrix, not a yes/no badge |
| Output is reproducible | Sort exact results and disclose stochastic parameters |

These are not PMF features. They are the minimum quality level required for any PMF feature to survive.

### Performance quality: this makes the workflow preferable

| Performance dimension | Product bar |
|---|---|
| Time to first useful evidence | Faster than manual repo orientation for the same task |
| Incremental freshness | Fast enough to remain in the edit loop without user polling |
| Context efficiency | Fewer irrelevant tokens while retaining task-critical evidence |
| Query ergonomics | One job-shaped request instead of composing many graph primitives |
| Diagnostic recovery | One bounded path from suspicious answer to root cause |
| Local operation | No external server required for the default workflow |

### Delight quality: differentiation after trust

- The agent pauses before a risky edit with a precise reason.
- Post-edit proof catches an unexpected contract or caller.
- Context explains why each item was selected.
- An empty result explains itself.
- A failed candidate index preserves a known-good snapshot automatically.
- The same workflow can use compiler evidence when available without reconfiguration of the client.

## 4. PMF Opportunity Table

Judgments are qualitative hypotheses, not market-validation scores.

| Rank | Opportunity | User moment | Pain / frequency | Architecture fit | PMF judgment | Why | Fastest falsification experiment |
|---:|---|---|---|---|---|---|---|
| 1 | **Pre-edit context plus post-edit structural proof** | Agent is about to change code, then has changed it | High pain, daily for AI-first developers | Local facts, immutable snapshots, `prepare_edit` and `verify_change` orchestration | **Best wedge** | Urgent, repeated, measurable, and uses the graph where file search is weakest | Run 20 real edits in shadow mode; measure missed-file rate, unexpected edges caught, latency, and voluntary reuse |
| 2 | **Relevant-test selection with residual-risk report** | Patch is ready for validation | High pain, per edit or PR | Test entities retained in canonical facts; graph projection plus runner adapter | **Core companion workflow** | Converts graph impact into an immediate action and closes the verification loop | On fixture and real PRs, compare selected tests with tests that fail in full suite; track recall before optimizing reduction |
| 3 | **Branch / PR impact review** | Maintainer decides where to spend review attention | High pain, per PR; strongest for busy maintainers | Baseline/candidate artifacts, semantic diff, headless CLI/HTTP | **Strong expansion and possible paid team feature** | Bounded workflow with clear buyer and team value | Analyze 30 historical PRs blind; ask reviewers whether top questions and impacted flows would have changed review behavior |
| 4 | **Index trust, coverage, and explain-empty diagnostics** | Any result looks wrong or incomplete | Infrequent when quality is good, existential when it occurs | Per-stage diagnostics ledger and query receipts | **Mandatory foundation, not standalone wedge** | Historical failures show PMF claims collapse without it | Seed missing parser, stale snapshot, excluded file, unresolved edge, and backend failure; users must diagnose each in under two minutes |
| 5 | **Incident and failing-test investigation** | Developer needs a root-cause hypothesis quickly | High urgency, episodic frequency | Search plus reverse paths plus optional SCIP/runtime traces | **Strong adjacent workflow** | Static graph narrows search space; bidirectional evidence loop suits agent reasoning | Time paired investigations on known bugs with and without the workflow; score hypothesis quality and time to falsifiable experiment |
| 6 | **Token-budgeted evidence packages** | Agent task spans many related symbols | High frequency for agents; pain varies by model/context | Separate planner, tokenizer, source slicer, graph retrieval | **Differentiator inside core workflows** | Valuable when it improves task success, not merely token count | Compare fixed-budget completion on 20 tasks against raw file reads and simple repo maps; record correctness and tokens |
| 7 | **Compiler/SCIP precision tier** | Tree-sitter result contains ambiguous or unresolved relationships | High value in supported languages, setup-sensitive | Optional fact importer merged with baseline evidence | **Moat enabler, not initial product** | Raises trust without sacrificing broad fallback coverage | For Rust/TS/Java fixtures, compare edge precision/recall and setup time with Tree-sitter baseline |
| 8 | **Architecture refactoring and migration planning** | Staff engineer plans a costly structural change | High willingness to pay, low frequency | Analytics projection, seeded algorithms, explainable boundaries | **High-value expansion** | Expensive decisions justify richer analysis, but communities are hypotheses rather than truth | Use three completed refactors; test whether output predicts actual boundary conflicts and migration order |
| 9 | **First-ten-minute repository orientation** | User enters unfamiliar code | Frequent across tasks, lower urgency than edit risk | Local embedded index, progressive map, guided context | **Useful activation surface** | Broad top-of-funnel value and natural entry into safe-edit loop | Give unfamiliar developers ten-minute tasks; compare first correct source location and next action against normal tools |
| 10 | **Shared CI governance and architecture policy** | Platform team wants consistent review checks | Team-level value, less frequent buying decision | Shared service or portable artifacts, policy engine | **Later enterprise expansion** | Requires proven local behavior first; otherwise centralizes false confidence | Pilot one repository in non-blocking mode; measure useful comments versus noisy flags before enabling gates |
| 11 | **Data lineage and API-contract impact** | Schema, route, event, or DTO changes | High pain in specific stacks, moderate frequency | Typed relation adapters, cross-language evidence, optional CPG-like schema | **Promising vertical bet** | Clear value but needs deeper semantics than generic call edges | Pick one supported stack and one contract type; measure downstream-consumer recall on historical migrations |
| 12 | **Language-pack SDK and verification harness** | Maintainer adds or repairs language support | Ecosystem pain, not end-user daily pain | Query registry, typed captures, fixtures, golden tests | **Strategic platform investment** | Improves coverage and contribution velocity; unlikely to be the commercial wedge | Have an external contributor add one language construct using only the SDK and docs; record time and defects |
| 13 | **Focused visual graph views** | Human needs to inspect a path, diff, or boundary | Moderate pain, supportive frequency | Read-only projection of existing workflows | **Supporting interface** | Useful when attached to review, incident, or refactor decisions; weak as a universal graph canvas | Prototype only blast-radius and diff views; test whether users find evidence faster than tables/source links |
| 14 | **General graph analytics buffet** | User browses metrics without a specific decision | Low demonstrated urgency | Any graph engine | **Do not lead with this** | Algorithm count is easy to copy and can hide weak graph construction | Remove algorithm names from the pitch; if no user job remains, do not prioritize the feature |
| 15 | **Generic natural-language code chatbot** | User asks open-ended repo questions | Crowded, frequent, weak differentiation | LLM plus retrieval | **Avoid as primary identity** | Native agents already do this; Parseltongue should provide evidence to them | Test whether job-shaped workflows outperform open chat on correctness and next-action clarity |

## 5. Recommended Product Surface

Keep the existing low-level APIs for compatibility, but place a small job-shaped surface above them.

| Product workflow | User question | Composed evidence | Required response contract |
|---|---|---|---|
| `prepare_edit` | What must be understood before changing X? | exact symbol, source, callers, callees, contracts, tests, bounded impact | baseline revision, evidence, confidence, unresolved cases, context budget |
| `verify_change` | Did this patch do only what was intended? | graph diff, changed contracts, relevant tests, coverage delta | candidate revision, expected/unexpected changes, residual risk |
| `review_change` | Where should review attention go? | branch diff, public API impact, flows, owners, tests | evidence-linked questions, not generic risk prose |
| `investigate_failure` | What path should I test next? | search, reverse paths, error handlers, tests, optional traces | ranked hypotheses with contradictory evidence and next experiment |
| `build_context` | What evidence fits this task and budget? | ranked graph and source items | actual tokens, reasons, omissions, continuation |
| `explain_index_gap` | Why is this answer empty or suspicious? | file decisions, parser diagnostics, capture and edge lineage | precise stage failure and repair action |

### Why orchestration instead of more endpoints

The current API already exposes many useful primitives. Adding more primitives increases agent planning burden. Product workflows should compose existing capabilities, enforce revision consistency, and return a single evidence receipt.

The low-level API remains valuable for debugging, power users, and backend parity testing.

## 6. Bidirectional Product Loop

The recommended loop has three planes:

```text
Reasoning plane
  User intent <-> LLM hypothesis and interpretation
           |                         ^
           v                         |
Evidence plane
  deterministic facts -> graph queries -> source-spanned receipt
           |                         ^
           v                         |
Action and proof plane
  edit / test / review -> candidate revision -> deterministic verification
```

### Rules for the loop

1. LLM interpretation may rank or label evidence but cannot silently rewrite canonical facts.
2. Every LLM claim used for an action should be rechecked against deterministic evidence where possible.
3. Failed verification returns to reasoning with a precise contradiction.
4. Unresolved evidence remains unresolved until a deterministic resolver confirms it.
5. Human decisions and accepted semantic labels are stored as annotations with provenance.
6. The loop ends with a proof receipt and residual uncertainty, not a confidence-themed paragraph.

### Highest-value loops to ship together

- Intent -> `prepare_edit` -> patch -> `verify_change`.
- Change -> test candidates -> test results -> residual-risk update.
- Error -> graph trace -> hypothesis -> narrower graph trace.
- Branch diff -> review questions -> evidence verification.
- Task budget -> context package -> gap request -> continuation package.

## 7. Architecture Decision

### Recommended: SQLite/Turso facts plus immutable Rust graph snapshots

| Criterion | Assessment |
|---|---|
| Local activation | Strong: one embedded store and no database service |
| Backend neutrality | Strong if handlers depend on product services and typed ports |
| Direct traversal | Strong with adjacency indexes in RAM |
| Advanced analytics | Adequate through pluggable algorithm modules |
| Durability | Strong with transactions, revisions, and migrations |
| Debuggability | Strong because canonical facts are inspectable with SQL |
| Incremental updates | Strong with candidate revisions and snapshot swap |
| Team scale | Requires later shared-service option |
| Very large graph scale | Requires profiling, compressed projection, or accelerator |

SQLite/Turso should persist facts, not emulate every graph algorithm through recursive SQL. The RAM projection should perform graph work. This is also conceptually similar to graph systems that project durable data into specialized in-memory analytical structures.

### Neo4j: optional adapter, not default identity

Neo4j can be useful for shared deployments, Cypher exploration, and mature analytics. It should be introduced only after:

- normalized endpoint semantics exist;
- a backend parity suite exists;
- the local reference engine is correct;
- a user segment demonstrates need for a shared graph service.

Neo4j is not an embedded Rust database and should not be required for local activation.

### Pure recursive SQL: use selectively

SQL is appropriate for direct lookups, filtering, aggregation, revision metadata, and bounded data loading. It is a poor single abstraction for the full mix of traversal, SCC, centrality, communities, source retrieval, and context planning.

### Compiler-only architecture: too narrow

Compiler indexes offer stronger semantics but uneven language and build support. Use them as enrichment. Do not make Parseltongue unavailable when a project cannot produce a compiler index.

### Multi-backend framework immediately: over-architecture risk

Backend neutrality should begin as behavior and one small set of ports, not five speculative adapters. Implement the reference backend first. Add a second adapter specifically to test whether the abstraction is real.

## 8. Reference Architecture Boundaries

```rust
pub trait CodeIntelligenceService {
    fn prepare_edit(&self, request: PrepareEditRequest) -> Result<EditContextReceipt, ServiceError>;
    fn verify_change(&self, request: VerifyChangeRequest) -> Result<ChangeProofReceipt, ServiceError>;
    fn review_change(&self, request: ReviewChangeRequest) -> Result<ReviewEvidenceReceipt, ServiceError>;
    fn investigate_failure(&self, request: InvestigationRequest) -> Result<InvestigationReceipt, ServiceError>;
    fn build_context(&self, request: ContextRequest) -> Result<ContextPackage, ServiceError>;
}
```

Implementation components:

| Component | Owns | Must not own |
|---|---|---|
| `LanguageRegistry` | grammar, extensions, capability metadata | graph semantics |
| `QueryRegistry` | versioned query assets and typed captures | HTTP rendering |
| `RepositoryIndexer` | staged fact production and diagnostics | product workflow prose |
| `CodeFactStore` | canonical revisions, files, entities, edges | traversal algorithms |
| `GraphProjectionBuilder` | stable ID to internal index mapping | durable identity |
| `ReferenceGraphEngine` | exact traversal semantics | storage migrations |
| `AnalyticsEngine` | parameterized advanced algorithms | canonical relationships |
| `ContextPlanner` | relevance, source slices, token budgets | parser fact mutation |
| `CodeIntelligenceService` | workflow orchestration and consistency | SQL, Cypher, Datalog leakage |
| HTTP/CLI/tool adapters | transport and DTO mapping | product semantics |

## 9. Accuracy Contract

### Exact operations

Direct callers, callees, bounded reachability, SCC membership, and k-core should compare normalized sets or partitions exactly against fixtures and independent oracles.

### Numeric operations

PageRank and betweenness require documented directedness, edge aggregation, weights, damping, endpoints, normalization, convergence tolerance, and iteration limits. Compare within declared numeric tolerances.

### Stochastic operations

Leiden and other community methods do not have one universally correct partition. Require fixed seed where supported, valid partition coverage, connectedness guarantees where applicable, quality score, parameters, and stability checks.

### The larger accuracy risk

Algorithm implementation is not the dominant risk. The dominant risks are:

- wrong caller/callee direction;
- duplicate call-site edges interpreted inconsistently;
- unresolved names promoted to real calls;
- incorrect scope filtering order;
- stale snapshots after durable commits;
- unstable or reused internal indexes;
- tests and generated code excluded without disclosure;
- language grammar drift.

## 10. Sequencing Recommendation

### Now: prove the wedge

1. Define evidence receipt, freshness, coverage, and honest-empty contracts.
2. Build one trusted local backend with SQLite/Turso plus RAM graph.
3. Compose existing primitives into `prepare_edit` and `verify_change`.
4. Retain test entities and implement relevant-test evidence.
5. Dogfood on real edits in shadow mode before claiming safety.

### Next: expand the loop

1. Add `review_change` for branches and pull requests.
2. Add token-budgeted `build_context` with selection reasons.
3. Add `investigate_failure` using the same evidence model.
4. Import SCIP indexes for one or two languages and expose precision tier.
5. Add focused diff and blast-radius visual views.

### Later: team and architecture expansion

1. Shared service and CI policy.
2. Architecture migration analytics.
3. Contract/data-lineage adapters for one chosen stack.
4. External Neo4j or high-performance analytics adapter if scale evidence demands it.
5. Language-pack contributor SDK.

### Not now

- a universal graph visualization product;
- a large catalog of unvalidated graph metrics;
- generic natural-language repo chat;
- GPU or distributed execution before local profiling;
- five storage backends before one behavior suite is trustworthy;
- automatic semantic-fact mutation by an LLM;
- hard CI blocking based only on blast-radius counts.

## 11. PMF Experiments

### Experiment 1: Safe-edit shadow study

- Select 20 real, non-trivial coding tasks.
- Let the developer or agent work normally.
- Run `prepare_edit` and `verify_change` in shadow mode.
- Record dependencies missed by normal exploration, unexpected graph changes, false alarms, added latency, and whether the user would invoke it next time.
- Success is voluntary repeated use and material catches, not token savings alone.

### Experiment 2: Relevant-test recall

- Use historical commits with known full-suite outcomes.
- Predict test candidates from the pre-change graph.
- Primary metric: recall of tests that actually fail.
- Secondary metric: reduction versus full suite.
- Do not optimize reduction until recall reaches the agreed safety threshold.

### Experiment 3: Trust under failure

- Inject stale index, parser error, unsupported language, excluded path, unresolved call, truncated context, and backend outage.
- Ask users to explain the empty or suspicious answer.
- Success: correct diagnosis and recovery in under two minutes without inspecting database internals.

### Experiment 4: Competitor baseline

- Compare Parseltongue with `rg`, language-server references, a repo map, and one established code-graph tool.
- Use identical tasks and token budgets.
- Measure task correctness, time to confident next action, irrelevant context, and setup burden.

### Experiment 5: Local architecture limit

- Benchmark representative small, medium, and large repositories.
- Measure durable store size, graph projection memory, cold projection time, incremental publication latency, and p95 query latency.
- Use results to decide whether compressed CSR, a shared service, or an external engine is actually needed.

## 12. Decision Table

| Decision | Recommendation | Confidence | What would change it |
|---|---|---|---|
| Product identity | Evidence router for safe agent edits | Medium-high | Users do not voluntarily reuse pre/post edit workflow |
| Initial user | AI-first developer and maintainer | Medium | Agent builders or platform teams show materially stronger pull |
| Core workflow | `prepare_edit` plus `verify_change` | High from corpus convergence, unvalidated in market | Shadow study shows ceremony without useful catches |
| Durable storage | SQLite/Turso canonical facts | Medium-high | Measured write, portability, or migration failures |
| Graph execution | Immutable Rust in-memory reference snapshot | High | Repository scale exceeds laptop memory or rebuild budget in target segment |
| Semantic source | Tree-sitter baseline plus optional SCIP/compiler evidence | High | A narrower single-language segment proves substantially stronger PMF |
| Neo4j | Optional later adapter | High | Immediate target requires shared Cypher/GDS operations and accepts server setup |
| Advanced analytics | Expansion only, tied to decisions | High | A specific algorithm becomes a repeated must-have workflow |
| Visualization | Focused supporting views | High | Human visual exploration becomes the primary repeated entry point |
| LLM role | Interpret, rank, request, and explain; do not silently mutate facts | High | A deterministic verification mechanism can safely promote specific annotations |

## 13. Final Thesis

Parseltongue can earn product-market fit by becoming the trust layer around agentic code changes, not by becoming a broader graph database.

The product should own six things exceptionally well:

1. stable, source-spanned evidence;
2. honest freshness and coverage;
3. minimal context before an edit;
4. structural proof after an edit;
5. relevant tests and review questions;
6. backend-neutral behavior with explicit semantic quality.

SQLite/Turso plus an immutable Rust graph projection is the strongest first architecture because it preserves local activation, inspectability, and future optionality. Neo4j and other engines become useful only after the behavior contract and PMF workflow are proven.

The strategic filter for every future feature is:

> Does this help the user reach a safer next action with less uncertainty, and can Parseltongue prove why the answer should be trusted?
