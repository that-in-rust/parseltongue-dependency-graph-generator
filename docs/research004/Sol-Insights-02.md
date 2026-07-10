<!-- markdownlint-disable MD013 MD024 MD060 -->

# Solution Insights 02: Visual Trust, Architecture Snapshots, and Agent Change Proof

- **Date:** 2026-07-10
- **Status:** Product-thesis synthesis after discussion and competitive research
- **Sequence:** Numbered continuation of [Sol-Observations-01.md](Sol-Observations-01.md) and [Sol-Insights-01.md](Sol-Insights-01.md)
- **Central question:** How can Parseltongue help a human trust what a coding agent changed without requiring the human to reconstruct the entire change from chat and raw diff text?

## 1. Governing Conclusion

Parseltongue should not try to differentiate primarily as the fastest code-search engine, the largest code graph, the graph system with the most algorithms, or another general AI pull-request reviewer.

The stronger product thesis is:

> Parseltongue should make coding-agent work visible, inspectable, and provable by turning a repository revision into a small, versioned architecture graph and turning a code change into a task-scoped visual Change Map.

The product should help a human answer five questions:

1. What did the agent say it would change?
2. What structural facts actually changed?
3. Which callers, callees, dependencies, public interfaces, and boundaries were affected?
4. What evidence verifies the change?
5. What remains unresolved, unindexed, stale, or otherwise unproven?

This is not merely a visualization feature. It is a trust system with a visual interface.

The minimum useful architecture vocabulary is deliberately small:

```text
CONTAINS
  repository -> module/package/crate -> file -> symbol

DEPENDS_ON
  module/file/symbol -> module/file/symbol

CALLS
  caller symbol -> callee symbol
```

Forward call and backward call are two queries over the same directed `CALLS` relationship:

```text
A --CALLS--> B

forward from A  = outgoing CALLS edges = callees
backward from B = incoming CALLS edges = callers
```

The product does not initially need PageRank, betweenness, Leiden, Louvain, Node2Vec, graph kernels, graph edit distance, or a large analytics catalog. Those may become optional consumers of the architecture format later. They are not required to establish trust in an agent-generated change.

## 2. How The Product Thesis Evolved

The discussion moved through several increasingly precise product hypotheses. Preserving this evolution matters because each rejected framing still identifies a useful capability, but not necessarily the product identity.

### Stage 1: Fast Code Search

The initial temptation was to compete on fast repository search and efficient LLM context retrieval.

That capability remains useful, but it is a weak identity because:

- mature search and indexing systems already exist;
- coding agents can already use repository maps, lexical search, semantic search, LSP, SCIP, MCP servers, and agent-native exploration;
- after search becomes interactive, further latency improvements often do not change the user's decision;
- search performance is relatively easy to benchmark and copy;
- incumbents already have distribution inside code hosts, editors, and agent products;
- the reviewer waits on understanding and confidence, not only query latency.

The durable performance target is not "fastest search." It is:

> Fastest trustworthy path from an agent-produced change to a confident human decision.

Search therefore remains infrastructure. It should be fast, fresh, local where possible, and available to agents, but it should not be the top-level promise.

### Stage 2: Code Reviewer

The next hypothesis was that code review is a better segment because the reviewer needs more than a token-efficient context packet. The reviewer needs to determine whether a Git diff is directionally correct.

This is directionally stronger because review has:

- a concrete trigger;
- a repeated workflow;
- an asymmetric cost of mistakes;
- a visible decision, merge or do not merge;
- evidence that can be compared with later outcomes.

However, "code reviewer" is still too broad. Generic AI review is highly competitive. Existing tools already claim full-repository context, code graphs, cross-repository reasoning, custom rules, PR history, automatic comments, suggested fixes, and architecture awareness.

The open surface is not "AI reviews code." It is narrower:

> A human needs a perceptible, evidence-linked account of what an agent did, why it matters structurally, and which parts remain unproven.

### Stage 3: Architectural Intent Verification

The discussion then sharpened around architectural direction.

A graph can show that a new dependency appeared. It cannot, by itself, determine whether that dependency is desirable.

A graph can show a large blast radius. Large impact can be intentional and correct.

A graph can show that a change stayed within one file. A one-line import can still violate a critical boundary.

Therefore, "right direction" requires a three-way comparison:

```text
declared change intent
        +
standing structural constraints
        +
observed baseline-to-candidate graph delta
        =
accounted, missing, unaccounted, violating, or unresolved change
```

This remains a valuable product capability, but it also exposes an adoption problem: most teams do not maintain complete architecture specifications. Requiring a universal architecture model before the first review would create too much setup friction.

### Stage 4: Visual Trust For Agent Work

The deeper observation is that people do not trust coding-agent output because they cannot perceive the work as a coherent system change.

Chat provides a narrative written by the same agent whose work is being evaluated.

A raw Git diff provides implementation detail, but not necessarily structure, impact, or proof.

Tests provide evidence about exercised behavior, but not a complete explanation of what changed.

A conventional repository graph is usually too large and abstract to answer the immediate question.

The missing object is a task-scoped visual account:

```text
INTENDED
  What the agent said it would change
        |
        v
OBSERVED
  What nodes, edges, signatures, and files changed
        |
        v
IMPACTED
  What callers, callees, dependencies, consumers, and tests are connected
        |
        v
VERIFIED
  Which builds, tests, policies, and graph checks ran
        |
        v
UNKNOWN
  What the system could not index, resolve, execute, or prove
```

The visual interface is not the source of truth. It is the perceptual surface over evidence.

### Stage 5: A Minimal Architecture Interchange Format

To render the Change Map consistently, Parseltongue needs a portable representation of repository structure.

The discussion therefore arrived at a final architectural question:

> Is there already a universal software architecture format that stores code-derived dependencies and calls at multiple levels and can be immediately consumed by a visualizer?

The answer is:

> Generic graph formats exist. Rich code intermediate representations exist. No broadly adopted format currently combines the exact semantics, minimality, versioning, confidence, multi-level projection, and change representation needed for this product.

Parseltongue should not attempt to standardize all software architecture. It should define a deliberately small Architecture Snapshot Format for this specific job.

## 3. Evidence And Confidence Boundaries

This document combines three kinds of material:

- **Repository evidence:** current Parseltongue APIs, previous research documents, local reference repositories, and historical failure reports.
- **Current external evidence:** official documentation and public repositories for code search, code review, graph interchange, code intelligence, and architecture systems.
- **Product inference:** judgments about differentiation, user segments, trust, product-market fit, and appropriate scope.

The external market observations are current as of 2026-07-10 but will continue changing.

The product conclusions are hypotheses. They should be tested against real reviews and real users rather than treated as proven because the reasoning is internally coherent.

Confidence by conclusion:

| Conclusion | Confidence | Reason |
|---|---|---|
| Fastest search is a weak primary identity | High | Mature search/indexing alternatives and diminishing user value after interactive latency |
| Generic AI code review is crowded | High | Multiple major products already provide repository context, automatic review, rules, and agent integration |
| Humans supervising agents have a trust and legibility problem | Medium-high | Strong workflow logic and widespread concern, but Parseltongue-specific demand is not yet measured |
| A visual Change Map can improve trust | Medium | Plausible and testable; a bad visualization could instead create false confidence |
| A minimal architecture snapshot format is technically feasible | High | Existing graph, SCIP, Kythe, LSP, and CPG precedents cover the necessary primitives |
| The format itself can become a defensible product moat | Low-medium | Standards and schemas are copyable; adoption, adapters, evaluation, and decision history matter more |
| The proposed segment will pay | Unknown | Requires buyer and usage validation |

## 4. Competitive Reality

### 4.1 Search And Context Are Mature Capabilities

The local Research002 corpus already identified strong agent-facing systems with job-shaped operations such as `context`, `impact`, `trace`, `detect_changes`, `api_impact`, `review_context`, `tests_for`, and `validate`.

Examples include:

- [GitNexus](https://github.com/abhigyanpatwari/GitNexus), which exposes context, impact, trace, change detection, API impact, routes, contracts, and cross-repository group facts;
- [codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp), which provides a persistent local code graph and broad agent integration;
- [code-review-graph](https://github.com/tirth8205/code-review-graph), which provides impact analysis, graph diff, surprise scoring, affected flows, suggested questions, and CI review comments;
- [Sourcegraph Code Navigation](https://sourcegraph.com/docs/code-navigation), which combines search-based navigation with optional compiler-derived precise navigation;
- [SCIP](https://github.com/scip-code/scip), which provides language-neutral symbols, occurrences, definitions, references, implementations, source ranges, and role metadata.

The conclusion is not that Parseltongue should abandon search. It is that Parseltongue should consume or match the best available search and semantic evidence while differentiating at the human decision layer.

### 4.2 Generic AI Review Is Also Competitive

Current review products document capabilities such as:

- full-project context gathering;
- codebase graphs;
- custom repository and organization instructions;
- issue and PR-history context;
- cross-repository dependency reasoning;
- automatic PR comments;
- suggested fixes;
- rule learning from reviewer feedback;
- architecture maps and intended dependency constraints;
- agent and MCP integration.

Examples include:

- [GitHub Copilot code review](https://docs.github.com/en/enterprise-cloud%40latest/copilot/concepts/agents/code-review);
- [Greptile](https://www.greptile.com/docs/introduction);
- [CodeRabbit](https://docs.coderabbit.ai/knowledge-base);
- [Qodo cross-repository review](https://docs.qodo.ai/governance/cross-repo-code-review);
- [SonarQube architecture management](https://www.sonarsource.com/blog/code-architecture-management-general-availability-in-sonarqube/).

Parseltongue should therefore avoid competing as another undifferentiated comment-generating bot.

### 4.3 Architecture Governance Is Not Empty

Architecture checking also has established precedents:

- ArchUnit-style tests declare allowed dependencies and layer rules;
- dependency-cruiser declares forbidden, allowed, and required module relationships;
- SonarQube compares current and intended component structures;
- Clarity shows file and module dependency shape, cycles, and range-specific structural impact;
- language-specific compatibility tools compare public API and schema contracts.

The relevant gap is at the junction of these systems:

```text
agent intent
  + code-derived graph delta
  + language-specific contract evidence
  + visual human review
  + explicit uncertainty
```

That junction is not empty, but it is less standardized and less consistently productized than search or generic review.

## 5. The Target User Segment

### 5.1 Broad Segment To Reject

"All code reviewers" is not a useful initial segment.

Many reviews are trivial. Many repositories are small. Many teams are satisfied with tests, linters, IDE navigation, and an existing AI reviewer. Some reviewers will never open a graph. A broad segment makes it impossible to define the urgency, required precision, and acceptable setup cost.

### 5.2 Recommended User

The stronger initial user is:

> A developer, maintainer, or tech lead supervising substantial coding-agent changes in a non-trivial repository who is accountable for whether those changes preserve public interfaces and structural boundaries.

Strong examples include:

- maintainers of shared internal libraries;
- SDK and API-platform teams;
- maintainers of public crates or packages;
- teams with cross-repository consumers;
- tech leads reviewing AI-generated refactors;
- engineers approving changes in mixed Rust, C++, TypeScript, Python, or service repositories;
- teams where architecture knowledge currently lives in reviewer memory.

### 5.3 High-Intensity Moment

The strongest initial moment is:

> The agent says it is finished, but the human does not yet feel able to merge.

The human is asking:

- Did the agent do what it claimed?
- Did it touch anything outside the expected area?
- Did it add a new dependency or call path?
- Did it alter a public signature?
- Which consumers or tests are now relevant?
- Is the graph current enough to trust?
- What has not been checked?

This is more precise than a generic review persona. It is an agent-supervision moment.

### 5.4 User, Buyer, And Distribution

The daily user may be a developer or reviewer.

The economic buyer is more likely to be:

- a platform engineering lead;
- a DevEx leader;
- an engineering manager responsible for AI adoption;
- an API governance or architecture group;
- a security or compliance group when structural evidence is auditable.

The distribution strategy should not require replacing the team's existing reviewer. Parseltongue should integrate with:

- local CLI workflows;
- GitHub or GitLab checks;
- SARIF-compatible review surfaces;
- MCP-compatible coding and reviewing agents;
- editor or desktop visualizations;
- CI artifacts.

## 6. A Product Model Of Trust

### 6.1 Trust Is Not The Same As Explanation

An LLM can generate a persuasive explanation that is wrong.

An automatically generated graph can be incomplete while looking authoritative.

A test suite can pass while missing the affected behavior.

A small diff can violate architecture.

Trust must therefore be composed from several independently inspectable qualities:

```text
Trustworthiness = legibility x evidence x control x calibration
```

This is not a numerical production formula. It is a product-design constraint. If any factor is effectively absent, trust degrades sharply.

### 6.2 Legibility

The human can form a coherent mental model of the change.

Required behaviors:

- show a bounded task-specific graph;
- separate expected from unexpected changes;
- organize information by architecture level;
- let the user expand from module to file to symbol;
- use concise labels instead of raw internal entity keys;
- preserve a direct path to source evidence.

### 6.3 Evidence

Every important claim has a traceable basis.

Required behaviors:

- link graph edges to source occurrences;
- identify the extractor or resolver that produced each fact;
- distinguish compiler evidence from syntax inference;
- report snapshot revision and freshness;
- attach test and build results to the change, not merely to the repository;
- expose skipped files, unsupported languages, and unresolved calls.

### 6.4 Control

The human can intervene.

Required behaviors:

- approve or amend the planned change envelope;
- pause on unexpected scope expansion;
- ask why two nodes are connected;
- mark a structural delta as expected or violating;
- request another verification step;
- compare checkpoints;
- reject or roll back the candidate change.

### 6.5 Calibration

The product demonstrates where it is reliable and where it is not.

Required behaviors:

- no success response for an empty result caused by index failure;
- no undifferentiated confidence score;
- no claim that Tree-sitter resolves every call exactly;
- no use of a low blast radius as proof of correctness;
- no use of community detection as authoritative module structure;
- historical comparison between predicted impact and actual review or test outcomes.

## 7. The Visual Product: A Change Map, Not A Giant Graph

### 7.1 Primary Object

The primary visual object should be a task-scoped **Change Map**.

It should not begin with the full repository graph. Large force-directed graphs are difficult to read, unstable between renders, and prone to becoming decorative rather than decisional.

The first view should answer:

```text
What changed?
What did it connect to?
Was that expected?
What was verified?
What is unknown?
```

### 7.2 Suggested First View

```text
+--------------------------------------------------------------+
| CHANGE: Add retry policy to the public client                |
| Base: main@abc123       Candidate: branch@def456              |
+--------------------------------------------------------------+
| EXPECTED            | OBSERVED             | STATUS           |
| New RetryPolicy     | RetryPolicy added    | matched          |
| Builder method      | method added         | matched          |
| Preserve send API   | signature modified   | unaccounted      |
+--------------------------------------------------------------+
| STRUCTURAL DELTA                                             |
| client -> retry                         added                 |
| public_api -> transport_internal        added, boundary alert |
| legacy_client -> client                 removed               |
+--------------------------------------------------------------+
| CALL IMPACT                                                  |
| 4 direct callers | 13 transitive callers | 2 unresolved      |
+--------------------------------------------------------------+
| VERIFICATION                                                 |
| build passed | 12/14 related tests passed | Python unchecked  |
+--------------------------------------------------------------+
```

The precise layout can evolve. The invariant is that the first viewport communicates decision status, not repository grandeur.

### 7.3 Progressive Disclosure

The user should be able to move through three levels:

```text
Level 1: Decision summary
  Expected, unexpected, violating, unresolved

Level 2: Structural map
  Modules, files, symbols, calls, dependencies

Level 3: Evidence
  Source ranges, signatures, resolver, test output, graph path
```

The user should never need to understand the storage backend or query language to investigate a finding.

### 7.4 Visual Semantics

The interface should distinguish change classes through labels, icons, shape, and color rather than color alone:

- added;
- removed;
- modified;
- moved;
- expected;
- unaccounted;
- constraint violation;
- unresolved;
- stale or skipped;
- verified.

Every derived visual edge should be selectable. Selecting it should answer:

- What does this edge mean?
- Which direction does it point?
- Where was it found?
- Which extractor produced it?
- How certain is it?
- Was it present in the baseline?

### 7.5 Visual Trust Failure Modes

The visual product fails if it:

- renders thousands of nodes without task scope;
- uses centrality size as an unexplained proxy for importance;
- hides unresolved edges;
- changes layout so drastically that baseline and candidate cannot be compared;
- presents an inferred community as an authoritative module;
- compresses multiple edge meanings into one generic line;
- shows a green overall score despite missing coverage;
- cannot navigate from a graph fact back to source;
- treats visual attractiveness as evidence quality.

## 8. Bidirectional Agent-Supervision Workflows

The visual product should support several loops rather than a one-way report.

### Workflow A: Intent To Proof

```text
human states intent
  -> agent proposes files, symbols, and interfaces likely to change
  -> Parseltongue renders proposed change envelope
  -> human approves or adjusts
  -> agent edits
  -> candidate snapshot is generated
  -> observed delta is compared with intent
  -> human reviews proof
```

### Workflow B: Unexpected Change Reconciliation

```text
new dependency appears
  -> Change Map marks it unaccounted
  -> agent explains why it was introduced
  -> graph provides the exact source and dependency path
  -> human accepts, redirects, or rejects
  -> decision is stored with provenance
```

### Workflow C: Calls And Impact

```text
public symbol changes
  -> outgoing calls reveal implementation dependencies
  -> incoming calls reveal consumers
  -> user expands selected callers only
  -> relevant tests are linked where evidence exists
  -> residual unresolved consumers remain visible
```

### Workflow D: Verification Feedback

```text
graph predicts impacted tests
  -> tests run
  -> results return to Change Map
  -> failed tests refine the impact hypothesis
  -> missing test relationships become graph-quality observations
```

### Workflow E: Learned Structural Intent

```text
reviewer accepts an unaccounted edge as intentional
  -> acceptance is recorded as a revision-scoped decision
  -> team may promote it to a standing rule or documented boundary
  -> future changes compare against the updated contract
```

Human decisions should never silently rewrite parser facts. They are annotations with author, timestamp, scope, and reason.

## 9. Does A Universal Architecture Format Already Exist?

### 9.1 The Important Distinction

There are two meanings of "universal format."

The first is a universal container for graph data. This exists.

The second is a universal semantic model of software architecture that every language tool can emit and every visualization can understand consistently. This does not exist in the required turnkey form.

### 9.2 Generic Graph Exchange Formats

[GraphML](https://graphml.graphdrawing.org/) supports directed and undirected graphs, hierarchical graphs, attributes, external references, and graphical metadata. [GEXF](https://docs.gephi.org/desktop/User_Manual/Import/GEXF_File_Format/) similarly transports graph structure, attributes, and visualization metadata.

These formats can be imported into graph tools immediately.

They do not standardize:

- what a software symbol is;
- what a stable symbol ID looks like;
- whether a dependency is an import, call, type use, runtime edge, or inferred relation;
- how repository, module, file, and symbol levels relate;
- how a baseline and candidate revision differ;
- how source evidence is attached;
- how uncertainty is represented.

They are suitable export formats, not sufficient canonical semantics.

### 9.3 SCIP

[SCIP](https://github.com/scip-code/scip) is a strong language-neutral semantic-index precedent.

It models:

- structured symbols;
- documents;
- occurrences;
- source ranges;
- symbol roles such as definition, import, read, write, generated, and test;
- relationships for references, implementations, type definitions, and definitions;
- external symbols and package descriptors.

SCIP is excellent for semantic identity and navigation.

It is not, by itself, a complete architecture snapshot and change format. In particular, a uniform persisted call graph and multi-level repository dependency projection are not its complete product contract.

The Research002 conclusion remains useful:

> Use SCIP as a semantic contract and evidence source, not necessarily as the whole engine.

See [tree-sitter-ref-202606.md](../research002/tree-sitter-ref-202606.md).

### 9.4 Language Server Protocol

The [Language Server Protocol 3.17 specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_prepareCallHierarchy) defines incoming and outgoing call-hierarchy requests.

This proves that callers and callees are widely recognized cross-language product concepts.

LSP is a live request protocol, not a portable versioned repository artifact. Implementations also vary by language server, project configuration, and build availability.

Parseltongue should be able to import LSP call evidence where available, but should not make a live language server the only storage model.

### 9.5 Kythe

[Kythe](https://kythe.io/docs/kythe-overview.html) is one of the closest conceptual matches.

It provides:

- a language-neutral graph storage format;
- an extensible graph schema;
- compiler and build metadata;
- definitions, usages, type information, and cross-language associations;
- a hub model for language indexers and client tools.

Kythe also explicitly rejects becoming a universal replacement for every specialized intermediate representation. It favors sharing useful subsets and emitting incomplete information rather than incorrect information.

That philosophy should be preserved.

### 9.6 Code Property Graph

The [Code Property Graph specification](https://cpg.joern.io/) defines a directed, edge-labeled, attributed multigraph with layers for files, namespaces, methods, calls, types, AST, control flow, data flow, and other analysis concepts.

It is a real language-neutral intermediate representation and probably the richest nearby standard.

It is also much larger than the initial Parseltongue product needs. Adopting its full AST, CFG, PDG, DFG, dominator, binding, and configuration model would make the architecture substrate dominate the product.

The right lesson is to borrow:

- attributed directed nodes and edges;
- source location;
- full names and signatures;
- external and unresolved entities;
- explicit dispatch type;
- versioned layers and overlays.

The wrong lesson is to implement the entire CPG before producing a useful Change Map.

### 9.7 C4 And Human Architecture Models

[The C4 model](https://c4model.com/) provides useful human abstractions: system, container, component, and code.

It helps people communicate architecture at multiple levels.

It is not a universal code-derived storage format. C4 elements are often manually authored and reflect intentional design rather than complete compiler facts.

Parseltongue can export or map selected architecture projections into C4-like views later. It should not require teams to maintain a complete C4 model before using the product.

### 9.8 Final Format Judgment

No existing option simultaneously gives Parseltongue all of the following:

- minimal code-architecture semantics;
- repository, module, file, and symbol hierarchy;
- dependencies and calls;
- stable source-linked identity;
- baseline and candidate revisions;
- exact graph delta;
- confidence and evidence provenance;
- immediate task-scoped visualization;
- broad language fallback;
- backend-neutral storage.

Therefore, a small Parseltongue format is justified, provided it is designed as a profile and integration layer rather than a claim to model all software architecture.

## 10. Architecture Snapshot Format

### 10.1 Design Goals

The Architecture Snapshot Format should be:

- small enough for a human to understand;
- deterministic enough for exact snapshot comparison;
- expressive enough for dependency and call visualization;
- hierarchical enough for multiple architecture levels;
- evidence-bearing enough for trust;
- open enough for multiple extractors and backends;
- versioned enough to evolve safely;
- portable enough for CLI, HTTP, MCP, CI, and visual clients;
- honest about incomplete resolution.

### 10.2 Non-Goals

Version 1 should not attempt to encode:

- every AST node;
- complete runtime behavior;
- full control-flow or data-flow graphs;
- product-domain semantics inferred by an LLM;
- deployment topology;
- every build-system concept;
- performance profiles;
- ownership and organizational structure as canonical code facts;
- graph embeddings;
- automatically discovered "true" modules;
- a universal architecture policy language.

These may be represented by extensions or other systems later.

### 10.3 Canonical Snapshot

```text
ArchitectureSnapshot {
  schema_version
  snapshot_id
  repository_identity
  source_revision
  generated_at
  generators[]
  coverage
  nodes[]
  edges[]
  diagnostics[]
}
```

### 10.4 Node Model

```text
ArchitectureNode {
  id
  kind
  display_name
  qualified_name
  language
  parent_id
  signature?
  visibility?
  source_location?
  external
  generated
  test
  evidence
  attributes
}
```

Initial node kinds:

```text
repository
module
package
crate
directory
file
type
interface
trait
function
method
external_symbol
unresolved_symbol
```

The format may expose fewer node kinds in its visual profile. Internally preserving distinctions is useful as long as the consumer can collapse them.

### 10.5 Edge Model

```text
ArchitectureEdge {
  id
  kind
  from
  to
  source_location?
  resolver
  confidence
  unresolved
  attributes
}
```

Canonical version-1 edge kinds:

```text
contains
depends_on
calls
```

Optional evidence-level subtypes may clarify how `depends_on` was obtained:

```text
imports
uses_type
implements
extends
exports
```

These do not need to become top-level product promises immediately. They can remain typed evidence contributing to a dependency projection.

### 10.6 Evidence Model

```text
Evidence {
  extractor
  extractor_version
  evidence_kind
  confidence
  source_revision
  source_path?
  source_range?
  explanation?
}
```

Suggested confidence classes:

```text
semantic_exact
semantic_environment
syntactic_strong
syntactic_weak
unknown_or_unresolved
```

Examples:

| Fact | Evidence | Confidence |
|---|---|---|
| Rust call resolved by rust-analyzer SCIP | compiler semantic index | `semantic_exact` |
| TypeScript symbol resolved by type checker | compiler API | `semantic_exact` |
| C++ call resolved under compile command | clang index | `semantic_exact` |
| Python package inferred from environment | environment resolver | `semantic_environment` |
| Import captured from Tree-sitter field | syntax query | `syntactic_strong` |
| Call target matched only by short name | name heuristic | `syntactic_weak` |
| Dynamic dispatch target unavailable | unresolved | `unknown_or_unresolved` |

### 10.7 Coverage Model

```text
CoverageReport {
  discovered_files
  indexed_files
  skipped_files[]
  supported_languages[]
  unsupported_languages[]
  parse_failures[]
  unresolved_imports
  unresolved_calls
  semantic_extractors_available[]
  syntax_only_languages[]
}
```

Coverage belongs in the snapshot header and every review receipt. It should not be hidden in logs.

### 10.8 JSON Example

```json
{
  "schema_version": "1.0",
  "snapshot_id": "sha256:example",
  "repository": "example/client-sdk",
  "revision": "def456",
  "coverage": {
    "indexed_files": 482,
    "skipped_files": 3,
    "unresolved_calls": 7,
    "semantic_languages": ["rust"],
    "syntax_only_languages": ["shell"]
  },
  "nodes": [
    {
      "id": "rust:crate:client:file:src/client.rs:fn:send",
      "kind": "function",
      "display_name": "Client::send",
      "qualified_name": "client::Client::send",
      "language": "rust",
      "parent_id": "file:src/client.rs",
      "signature": "fn send(&self, request: Request) -> Result<Response>",
      "visibility": "public",
      "source_location": {
        "path": "src/client.rs",
        "start_byte": 1200,
        "end_byte": 1840
      }
    }
  ],
  "edges": [
    {
      "id": "edge:caller:send",
      "kind": "calls",
      "from": "rust:crate:cli:file:src/main.rs:fn:run",
      "to": "rust:crate:client:file:src/client.rs:fn:send",
      "resolver": "rust-analyzer-scip",
      "confidence": "semantic_exact",
      "unresolved": false,
      "source_location": {
        "path": "src/main.rs",
        "start_byte": 900,
        "end_byte": 917
      }
    }
  ]
}
```

### 10.9 Canonical Encoding And Export Encodings

Recommended initial shape:

- canonical external representation: versioned JSON;
- internal Rust types: strongly typed structs and enums;
- durable facts: normalized SQLite/Turso tables;
- in-memory traversal: immutable Rust adjacency projection;
- large or binary interchange later: Protobuf if measured need appears;
- generic graph export: GraphML;
- lightweight documentation export: Mermaid and DOT;
- findings export: SARIF where appropriate.

JSON is not the most compact format. It is the easiest format for debugging, fixtures, agent consumption, API evolution, and early third-party experimentation.

## 11. Multiple Architecture Levels Through Containment

Architecture at multiple levels does not require several unrelated graph schemas.

It requires one canonical graph with hierarchical containment and deterministic projections.

### 11.1 Canonical Hierarchy

```text
repository
  -> workspace
    -> crate/package/module
      -> directory
        -> file
          -> type/function/method
```

Not every language uses every level. Consumers should tolerate missing intermediate levels.

### 11.2 Symbol-Level Projection

Use for:

- incoming calls;
- outgoing calls;
- public signature impact;
- exact source navigation;
- implementation review.

### 11.3 File-Level Projection

Aggregate symbol relationships by parent file:

```text
if symbol A in file X calls symbol B in file Y
then file X depends_on file Y
```

Use for:

- Clarity-like structural shape;
- changed-file context;
- manageable visual review;
- language-agnostic first views.

### 11.4 Module-Level Projection

Aggregate file dependencies by declared package, crate, workspace, or configured module.

Use for:

- boundary review;
- dependency direction;
- architecture overview;
- cross-repository contracts.

Declared modules should win over inferred communities. Community detection may suggest a candidate grouping only when no declared structure exists, and the result must remain labeled as inferred.

### 11.5 Stable Visual Comparison

Baseline and candidate should use the same containment anchors and a stable layout where practical. A node that did not change should not jump to an unrelated location merely because another node was added.

The visualizer should support:

- collapse to module;
- expand one module to files;
- expand one file to symbols;
- isolate changed nodes plus one-hop context;
- show inbound or outbound edges;
- show only added or removed edges;
- pin selected nodes between revisions.

## 12. Forward Calls, Backward Calls, And Dependencies

### 12.1 One Directed Call Fact

Store calls in execution direction:

```text
caller --CALLS--> callee
```

This avoids duplicating facts and eliminates directional ambiguity.

### 12.2 Forward Query

```text
callees(symbol, depth)
```

Answers:

- What does this function call?
- Which implementation dependencies does this method introduce?
- What work may occur downstream?

### 12.3 Backward Query

```text
callers(symbol, depth)
```

Answers:

- Who uses this function?
- Which consumers may be affected by a signature or behavior change?
- Where should review attention move next?

### 12.4 Dependency Query

```text
dependencies(node, direction, level, depth)
```

Answers:

- Which files or modules does this area depend on?
- Which files or modules depend on this area?
- Did a branch add a cross-boundary dependency?

### 12.5 Path Explanation

One additional primitive is worth retaining:

```text
path(from, to, edge_kinds, max_depth)
```

It answers:

> Why are these two areas connected?

This can use ordinary bounded BFS. It does not require sophisticated graph similarity or centrality.

### 12.6 Resolution Caveats

Calls are difficult across languages:

- virtual dispatch may have several possible targets;
- dynamic languages may resolve targets only at runtime;
- callbacks invert apparent control;
- macros alter source locations and declarations;
- reflection and string-based dispatch escape static resolution;
- generated bindings may not be present;
- build configuration changes semantic identity;
- external dependencies may lack source.

The format must preserve unresolved and multi-target call evidence rather than forcing one confident edge.

## 13. Snapshot Diff And Change Proof

### 13.1 Exact Delta Before Similarity

With stable identity, snapshot comparison should begin as deterministic set comparison:

```text
base nodes - candidate nodes       = removed nodes
candidate nodes - base nodes       = added nodes
base edges - candidate edges       = removed edges
candidate edges - base edges       = added edges
same ID, changed properties        = modified nodes or edges
```

This is more useful to a reviewer than a single distance score.

### 13.2 Change Kinds

```text
NodeAdded
NodeRemoved
NodeModified
NodeMoved
NodeVisibilityChanged
NodeSignatureChanged

EdgeAdded
EdgeRemoved
EdgeEvidenceChanged
EdgeConfidenceChanged
EdgeBecameResolved
EdgeBecameUnresolved
```

### 13.3 Identity Is The Hard Part

Exact diff requires identity that survives harmless movement.

Line-number-based IDs are insufficient because adding lines above a function changes its range.

Name-only IDs are insufficient because of overloads, duplicate names, local functions, and moves.

Potential identity ingredients include:

- language;
- package or crate identity;
- qualified symbol descriptor;
- owning module or type;
- signature;
- source path as a weaker component;
- compiler or SCIP symbol when available;
- content or structure fingerprints for move matching;
- birth identity in a temporal store.

Parseltongue should distinguish canonical identity from matching evidence. A move detector may propose that two nodes are the same entity; that proposal should retain its confidence.

### 13.4 Intent Accounting

The graph delta becomes a change proof only after comparison with intent:

```text
expected delta present       -> matched
expected delta absent        -> missing
observed delta not expected  -> unaccounted
delta violates standing rule -> violating
evidence insufficient        -> unresolved
```

Zero-configuration mode can infer a proposed intent from the PR description or agent plan, but a human should confirm it before the system treats it as authoritative.

### 13.5 Proof Receipt

```text
ChangeProofReceipt {
  base_snapshot_id
  candidate_snapshot_id
  intent_revision
  matched_changes[]
  missing_changes[]
  unaccounted_changes[]
  violations[]
  unresolved_changes[]
  affected_callers[]
  affected_callees[]
  dependency_delta[]
  verification_results[]
  coverage
  deterministic_receipt_hash
}
```

The receipt should be serializable and attachable to a PR, agent session, or CI artifact.

## 14. Do We Need Distance, Centrality, Or Community Algorithms?

### 14.1 Clarifying The Distance Question

The discussion referenced distance algorithms in general. This may include string distances such as Levenshtein or Hamming distance, AST edit distance, graph edit distance, shortest-path distance, or community algorithms such as Leiden and Louvain.

They solve different problems and should not be grouped into the core merely because all can operate on structured data.

### 14.2 Decision Table

| Capability | Version-1 judgment | Product reason |
|---|---|---|
| Direct incoming and outgoing edges | Keep | Fundamental structural truth |
| Bounded BFS/DFS reachability | Keep | Explains local impact without complex scoring |
| Shortest explanatory path | Keep, narrowly | Answers why two areas are connected |
| Strongly connected components / cycles | Optional early | Exact and understandable boundary signal |
| Exact snapshot node/edge diff | Keep | Core of visual change proof |
| Fuzzy string matching | Utility only | Helps resolve user-entered names; not architecture evidence |
| Levenshtein/Hamming distance | Defer | Search convenience, not core trust |
| AST edit distance | Defer | Clone and structural similarity problem |
| Graph edit distance | Reject for core | Expensive scalar that hides specific change evidence |
| PageRank | Defer | Ranking heuristic, not correctness or architectural intent |
| Betweenness centrality | Defer | Can identify bridges, but not required for change proof |
| k-core | Defer | Analytics, not a basic user decision |
| Leiden/Louvain communities | Defer | Inferred modules can conflict with human architecture |
| Node2Vec/graph embeddings | Defer | Hard to explain and unnecessary for exact change visualization |
| Weisfeiler-Lehman graph kernel | Defer | Cross-graph similarity is not an initial workflow |
| Spectral layouts or partitioning | Visualization implementation only | May help layout later, but should not define semantics |

### 14.3 Why Exact Facts Win Initially

The reviewer can understand:

```text
This branch added a dependency from public_api to transport_internal.
```

The reviewer cannot act as directly on:

```text
Architecture distance increased by 0.18.
```

The first statement is evidence. The second is an interpretation requiring explanation, calibration, and agreement about the metric.

### 14.4 When Advanced Algorithms Become Justified

Add an algorithm only when a repeated user decision cannot be served by direct graph facts.

Examples:

- PageRank may become useful for prioritizing a huge orientation map.
- Betweenness may become useful when selecting migration chokepoints.
- Leiden may become useful for suggesting modules in an unstructured legacy repository.
- graph edit distance may become useful for comparing architecture families across repositories.
- AST edit distance may become useful for clone detection.

None of these is necessary to answer whether a particular agent-generated change added, removed, or modified a dependency or call path.

### 14.5 Recommended Codebase Treatment

Existing algorithms do not need to be deleted merely because they are not part of the first product promise.

They should be:

- moved behind an optional analytics boundary;
- excluded from the canonical snapshot semantics;
- omitted from the default Change Map;
- retained only with explicit parameters and deterministic tests;
- activated when a product workflow demonstrates need.

## 15. Product API Surface

The existing low-level APIs can remain for compatibility. A smaller review-oriented surface should sit above them.

### 15.1 Snapshot APIs

```text
create_architecture_snapshot(repository, revision)
get_architecture_snapshot(snapshot_id, level, scope)
export_architecture_snapshot(snapshot_id, format)
get_snapshot_coverage(snapshot_id)
```

### 15.2 Graph APIs

```text
get_dependencies(node, direction, level, depth)
get_callers(symbol, depth)
get_callees(symbol, depth)
explain_path(from, to, edge_kinds, max_depth)
```

### 15.3 Diff APIs

```text
compare_architecture_snapshots(base, candidate, level, scope)
get_changed_nodes(diff_id)
get_changed_edges(diff_id)
get_change_neighborhood(diff_id, depth)
```

### 15.4 Trust APIs

```text
prepare_change(intent, base_revision)
verify_change(intent_id, candidate_revision)
explain_change_finding(finding_id)
acknowledge_change_finding(finding_id, decision, reason)
```

### 15.5 Agent Tool Surface

A coding or reviewing agent should need very few tools:

```text
architecture_snapshot
architecture_diff
callers
callees
explain_connection
verify_change
```

One dispatch tool with typed operations may be preferable to many nearly overlapping MCP definitions.

### 15.6 Backend-Neutral Contract

API behavior must not expose whether facts are stored in SQLite, Neo4j, memory, or another backend.

Normalized semantics must define:

- edge direction;
- ordering;
- depth behavior;
- duplicate aggregation;
- external nodes;
- unresolved nodes;
- truncation;
- snapshot consistency;
- source ranges;
- coverage and error envelopes.

## 16. Runtime And Storage Architecture

The backend-neutral recommendation from [Sol-Insights-01.md](Sol-Insights-01.md) remains compatible with this product thesis.

```text
language-specific extractors
        |
        v
canonical evidence facts
        |
        v
SQLite/Turso durable revisions
        |
        v
immutable Rust graph projection
        |
        +--------------------+
        |                    |
        v                    v
snapshot/diff service   source/evidence service
        |                    |
        +----------+---------+
                   v
          Change Map / MCP / CI
```

### 16.1 Durable Store

SQLite/Turso should own:

- repository and revision metadata;
- files and source hashes;
- nodes and stable identities;
- raw evidence records;
- canonical relationships;
- diagnostics and coverage;
- human annotations and decisions;
- snapshot manifests;
- schema migrations.

### 16.2 In-Memory Graph

The immutable Rust graph should own:

- adjacency indexes;
- callers and callees;
- dependencies and dependents;
- bounded reachability;
- shortest explanatory path;
- cycle detection if enabled;
- projection from symbol to file or module;
- exact set comparison support.

[`petgraph`](../../git-ref-repo/ignore-this-folder-repos/petgraph__petgraph/README.md) is the strongest first implementation candidate for this layer. It is a Rust graph library, not a database: SQLite/Turso remains the durable source of facts, while `petgraph` supplies directed graph structures, traversal, path-finding, and optional DOT export over one published snapshot. Because a published snapshot is immutable, a compact directed `Graph` may be sufficient; `StableGraph` is useful only if the candidate-building process needs deletion without invalidating internal indexes.

There are two important cautions:

1. `NodeIndex` is an implementation-local address, not a durable identity. Parseltongue must map canonical node IDs to and from graph indexes on every snapshot build and must never expose or persist a `NodeIndex` through the product API.
2. A graph library is mathematically accurate only relative to the graph it receives. Breadth-first search can return the exact callers represented in memory while the underlying call graph is still incomplete because dynamic dispatch, macros, generated code, reflection, or unresolved imports were not captured. `petgraph`, Neo4j, NetworkX, JGraphT, igraph, or a custom adjacency list cannot repair extraction errors merely by executing a correct traversal.

The resulting accuracy contract is therefore:

```text
answer correctness
  = extraction quality
  x identity and resolution quality
  x snapshot freshness
  x traversal correctness
  x honest coverage reporting
```

The library choice mainly affects traversal correctness, latency, memory, and implementation effort. It does not eliminate the need for language fixtures, compiler/LSP comparison where available, full-versus-incremental parity tests, unresolved-edge reporting, and coverage-aware responses. The local `petgraph` checkout also warns that its trunk is transitioning to a multi-crate architecture, so Parseltongue should pin a released API rather than accidentally designing against development-branch churn.

### 16.3 Candidate Publication

Incremental indexing should produce a candidate snapshot, validate it, then atomically publish it.

```text
file event or Git revision
  -> candidate facts
  -> parse and resolver diagnostics
  -> graph integrity checks
  -> full/incremental parity checks where applicable
  -> atomic snapshot publication
```

A failed candidate must not overwrite the last known-good snapshot.

### 16.4 Optional External Engines

Neo4j or another graph engine may be useful for:

- shared deployments;
- exploratory Cypher;
- large centralized graphs;
- advanced analytics;
- cross-repository organizational views.

It should not define the Architecture Snapshot Format or be required for local use.

## 17. Extraction And Semantic Accuracy

### 17.1 Tree-Sitter Baseline

Tree-sitter is valuable for:

- broad language coverage;
- incremental syntax parsing;
- function, type, module, and import extraction;
- source ranges;
- syntax-aware fallback;
- environments where builds cannot run.

Tree-sitter alone cannot reliably resolve every call across Rust, C++, TypeScript, Python, Ruby, reflection, macros, virtual dispatch, or generated code.

### 17.2 Semantic Enrichment

Use the strongest available source:

| Language or ecosystem | Preferred precision source |
|---|---|
| Rust | rust-analyzer SCIP, rustdoc JSON, compiler metadata where appropriate |
| TypeScript/JavaScript | TypeScript compiler and project references |
| Python | Pyright or another environment-aware semantic analyzer |
| C/C++ | Clang tooling plus compilation database |
| Java/Kotlin | compiler/LSP/SCIP index where available |
| Protobuf | Buf images and breaking-change rules |
| TypeScript public packages | API Extractor reports |
| OpenAPI/GraphQL/AsyncAPI | ecosystem-specific schema analyzers |

Parseltongue should wrap proven analyzers rather than imitate a compiler for every language.

### 17.3 Conflict Resolution

If several extractors disagree:

- preserve each evidence record;
- select a canonical fact through documented precedence;
- report the resolver used;
- make disagreement diagnosable;
- never silently upgrade a weak guess to exact evidence.

### 17.4 Full And Incremental Parity

For the same revision:

```text
full index canonical snapshot == incremental index canonical snapshot
```

This is an essential correctness property for a visual trust product. A stale or partially deleted edge can make the Change Map actively misleading.

## 18. Product Quality Model

### 18.1 Expected Quality

Failure here destroys trust:

- caller/callee direction is correct;
- dependencies have clear semantics;
- source links resolve;
- snapshot revision is visible;
- baseline and candidate are internally consistent;
- empty answers explain whether nothing exists or analysis failed;
- skipped files and languages are reported;
- deleted facts disappear;
- unsupported semantic precision is admitted;
- visual status never contradicts the receipt.

### 18.2 Performance Quality

This determines whether the product remains in the agent loop:

- local activation is simple;
- snapshot refresh is incremental;
- post-change proof arrives within the user's review rhythm;
- graph interaction is responsive;
- the first view is bounded;
- exact evidence can be loaded on demand;
- CI artifacts are deterministic and cacheable.

The important latency metric is time to useful proof, not only edge-query latency.

### 18.3 Delight Quality

This can differentiate after correctness:

- stable before/after visual transitions;
- one-click movement from architecture delta to source;
- a clear "why is this connected?" path;
- an agent plan rendered as a predicted change envelope;
- visible reconciliation between predicted and actual scope;
- a reusable proof receipt;
- a calibrated history showing how often predicted impact matched outcomes.

## 19. Product Requirements

### REQ-SNAPSHOT-001: Versioned Architecture Artifact

**WHEN** Parseltongue indexes a repository revision
**THEN** it SHALL produce a versioned architecture snapshot with repository identity, revision, nodes, edges, coverage, evidence, and diagnostics.

### REQ-HIERARCHY-002: Multiple Architecture Levels

**WHEN** a client requests module, file, or symbol level
**THEN** Parseltongue SHALL derive the requested projection from canonical containment relationships without changing edge direction or snapshot revision.

### REQ-CALL-003: Canonical Call Direction

**WHEN** Parseltongue stores a call
**THEN** the edge SHALL point from caller to callee
**AND** callers SHALL be queried through incoming edges
**AND** callees SHALL be queried through outgoing edges.

### REQ-DEPENDENCY-004: Explainable Dependencies

**WHEN** a dependency is returned
**THEN** the result SHALL include its level, source evidence, resolver, confidence, and aggregation basis where the dependency is projected from lower-level facts.

### REQ-DIFF-005: Exact Snapshot Delta

**WHEN** two compatible snapshots are compared
**THEN** Parseltongue SHALL report added, removed, modified, and moved nodes and edges
**AND** SHALL avoid substituting a distance score for the explicit delta.

### REQ-TRUST-006: Honest Unknowns

**WHEN** a file, language, import, or call cannot be resolved
**THEN** the Change Map SHALL display that uncertainty
**AND** SHALL not treat the missing evidence as an absence of impact.

### REQ-EVIDENCE-007: Source Traceability

**WHEN** a user selects a node, edge, or finding
**THEN** Parseltongue SHALL provide the source location and evidence lineage where available.

### REQ-INTENT-008: Intent Comparison

**WHEN** an approved change intent is available
**THEN** Parseltongue SHALL classify observed deltas as matched, missing, unaccounted, violating, or unresolved.

### REQ-VISUAL-009: Task-Scoped First View

**WHEN** a user opens a change proof
**THEN** the first view SHALL prioritize changed and affected structure
**AND** SHALL not render the entire repository graph by default.

### REQ-BACKEND-010: Behavioral Neutrality

**WHEN** different storage or graph engines implement the same service
**THEN** normalized snapshot, traversal, and diff semantics SHALL remain equivalent under backend parity fixtures.

### REQ-PARITY-011: Incremental Correctness

**WHEN** a revision is produced through incremental indexing
**THEN** its canonical snapshot SHALL match a clean full index of the same revision, subject only to explicitly documented nondeterministic metadata.

### REQ-HUMAN-012: Decision Provenance

**WHEN** a reviewer accepts, rejects, or reclassifies a structural delta
**THEN** the decision SHALL be stored as an annotation with actor, timestamp, revision scope, and reason
**AND** SHALL not mutate parser evidence.

## 20. Minimal Product Surface

### 20.1 First Release

The first useful release should do only this:

1. Index one repository revision.
2. Produce repository, module/file, and symbol nodes.
3. Produce `contains`, `depends_on`, and `calls` edges.
4. Answer direct callers, callees, dependencies, and dependents.
5. Compare a base and candidate snapshot.
6. Render a task-scoped visual Change Map.
7. Link every displayed fact to evidence or mark it unresolved.
8. Export the snapshot as JSON and GraphML.

### 20.2 Second Release

Add:

- approved change intent;
- matched, missing, and unaccounted classification;
- relevant test relationships;
- CI and PR proof receipts;
- one semantic precision adapter beyond Tree-sitter;
- reviewer annotations.

### 20.3 Later Releases

Consider only after usage evidence:

- cross-repository snapshots;
- public API and schema adapters;
- standing architecture constraints;
- live agent cockpit during execution;
- history and drift views;
- shared team service;
- advanced analytics;
- inferred module suggestions;
- optional Neo4j or large-graph adapters.

### 20.4 Explicitly Not First

- a universal architecture modeling language;
- a generic chatbot;
- a giant always-visible force graph;
- dozens of graph metrics;
- an LLM-generated architecture treated as truth;
- automatic merge blocking based on blast-radius count;
- all-language compiler precision;
- a required external graph server;
- a proprietary visual format with no export.

## 21. PMF And Evaluation Strategy

### 21.1 The PMF Hypothesis

The user will repeatedly use Parseltongue if the Change Map helps them reach a confident merge or request-changes decision faster than reading the raw diff and manually exploring the repository.

### 21.2 Historical Review Study

Select at least 30 non-trivial historical PRs from three repositories where human reviewers requested structural changes.

For each PR:

1. hide the original review comments;
2. build baseline and candidate snapshots;
3. provide only the PR description or linked issue as intent;
4. generate the Change Map and proof receipt;
5. compare findings with the original review and subsequent fixes;
6. compare against ordinary Git diff, Clarity, one graph-review tool, and one AI reviewer.

Measure:

- recall of known architecture-related concerns;
- precision of unaccounted and violating findings;
- time to the first correct review question;
- reviewer-rated usefulness;
- false-severity rate;
- coverage honesty;
- whether the output changes review order or decision;
- setup and analysis time.

### 21.3 Prospective Shadow Mode

Run Parseltongue on live PRs without blocking merges.

Ask reviewers:

- Did the Change Map show anything you would otherwise have missed?
- Did it reduce the amount of source you had to read?
- Did an unresolved warning prevent false confidence?
- Which part was visual noise?
- Would you request this artifact on the next risky agent PR?

Voluntary repeated use is a stronger signal than stated interest.

### 21.4 Trust Calibration Study

Inject controlled failure cases:

- stale snapshot;
- parser failure;
- unsupported language;
- unresolved dynamic call;
- deleted file with stale derived edge;
- generated code excluded;
- failed semantic indexer;
- truncated graph neighborhood.

The user should be able to identify why the result is incomplete from the Change Map without inspecting database internals.

### 21.5 Kill Or Narrow Criteria

The thesis should be killed or narrowed if:

- reviewers do not voluntarily reopen the Change Map after initial novelty;
- the visual artifact takes longer to understand than a focused diff;
- high-severity false positives destroy trust;
- Tree-sitter-only facts are too inaccurate for the target segment;
- users will not confirm even a lightweight change intent;
- existing reviewers plus repository instructions produce equivalent decisions;
- architecture-level findings rarely alter review behavior;
- maintaining stable identities across revisions proves unreliable;
- the chosen target repositories do not have meaningful dependency or call structure.

## 22. Strategic Differentiation

The format alone is not a moat. A JSON schema can be copied.

Potential durable advantages are:

### 22.1 Evidence Quality

The best available semantic adapter is selected per language, while fallback facts remain explicitly labeled.

### 22.2 Change Decision Dataset

Over time, accepted and rejected structural changes create a typed repository-specific record of what reviewers consider important.

### 22.3 Evaluation Corpus

A public or proprietary benchmark of real architecture-related review decisions can differentiate Parseltongue from systems evaluated only on token reduction or graph-derived ground truth.

### 22.4 Agent Neutrality

Parseltongue can serve Copilot, Codex, Claude Code, Cursor, Greptile, CodeRabbit, or another reviewer rather than forcing the user to replace them.

### 22.5 Visual Decision Design

The product can become excellent at presenting just enough graph structure for a specific decision, rather than treating the full graph as the interface.

### 22.6 Trust History

The system can show:

- how accurately agent plans predicted touched symbols;
- how often impact predictions matched failing tests;
- which edge types are frequently unresolved;
- which architectural warnings reviewers accept or reject;
- how coverage changes over time.

This calibration history is more defensible than another graph algorithm.

## 23. Major Risks

### Risk 1: False Visual Confidence

The graph may look complete when it is not.

Countermeasure: make coverage, evidence class, unresolved facts, and snapshot freshness first-class visual elements.

### Risk 2: No Universal Semantic Accuracy

Call resolution differs substantially across languages.

Countermeasure: use adapter architecture, typed confidence, semantic enrichment, and language-specific conformance fixtures.

### Risk 3: Architecture Means Too Many Things

Users may expect deployment, data, runtime, ownership, or domain architecture.

Countermeasure: call the initial model a code architecture or structural contract graph and publish explicit non-goals.

### Risk 4: Visualization Becomes A Demo

The graph may impress during onboarding but not enter daily work.

Countermeasure: attach it to the after-agent-before-merge moment and measure repeated use.

### Risk 5: Existing Reviewers Add The Same Feature

Incumbents have distribution and can add architecture visualizations.

Countermeasure: remain agent-neutral, evidence-first, open-format, local-first, and stronger at deterministic change proof than prose review.

### Risk 6: Intent Capture Adds Ceremony

Users may refuse to author architecture specifications or change manifests.

Countermeasure: begin with zero-configuration observed delta, derive proposed intent from existing issue/plan text, and ask the human to confirm a small change envelope rather than author a model from scratch.

### Risk 7: Stable Identity Fails

Moves, renames, overloads, generated code, and duplicate functions can produce misleading diffs.

Countermeasure: separate identity from location, use compiler symbols where available, retain matching evidence, and label uncertain move detection.

### Risk 8: Algorithmic Distraction Returns

Existing graph algorithms can pull the roadmap toward impressive but weakly demanded features.

Countermeasure: require every algorithm proposal to name the repeated user decision it improves and the direct-fact baseline it beats.

## 24. Decisions Reached

### Product Decisions

1. Search is infrastructure, not the primary differentiation.
2. Generic code review is too crowded as the product identity.
3. The target moment is human supervision of agent-generated change before merge.
4. The primary artifact is a visual Change Map backed by a deterministic proof receipt.
5. Trust requires legibility, evidence, control, and calibration.
6. The first view is task-scoped, not a full repository graph.
7. Intent improves directionality, but complete architecture authoring cannot be required for activation.
8. Existing reviewing agents should consume Parseltongue rather than necessarily be replaced by it.

### Format Decisions

1. No sufficient universal turnkey format currently exists for the exact job.
2. Define a minimal, versioned Architecture Snapshot Format.
3. Use containment to support repository, module, file, and symbol levels.
4. Canonical edge direction for calls is caller to callee.
5. Forward and backward calls are outgoing and incoming views over one edge.
6. Initial canonical edges are `contains`, `depends_on`, and `calls`.
7. Every fact carries evidence, source revision, and confidence.
8. Coverage and unresolved evidence are part of the product result.
9. JSON is the first canonical external representation.
10. GraphML, Mermaid, DOT, and SARIF are exports for specific consumers.

### Algorithm Decisions

1. Exact node and edge diff is core.
2. Direct traversal and bounded reachability are core.
3. Shortest explanatory path is useful.
4. Cycle detection may be an early optional exact check.
5. String, AST, and graph distance metrics are not core.
6. PageRank, betweenness, k-core, Leiden, Louvain, Node2Vec, and graph kernels are deferred analytics.
7. Inferred communities are never authoritative architecture facts.

### Architecture Decisions

1. SQLite/Turso remains the recommended durable fact store.
2. An immutable Rust adjacency graph remains the recommended reference traversal engine.
3. Tree-sitter provides broad syntax coverage.
4. SCIP, LSP, compiler, rustdoc, and schema tools provide optional semantic enrichment.
5. Backend behavior is defined by normalized service contracts and parity tests.
6. Neo4j and other graph engines remain optional adapters, not product identity.

## 25. Open Questions

The next decisions should be answered through prototypes and user evidence:

1. Is the first technical beachhead Rust libraries, mixed Rust/C++ systems, or TypeScript platform repositories?
2. Is the first user an individual maintainer or an internal platform team?
3. Should the first visual surface be post-completion only, or should it include live agent supervision?
4. How much intent confirmation will users tolerate?
5. Which public-interface adapter produces the highest early trust gain?
6. What is the smallest stable identity scheme that survives the target repositories?
7. Should file dependency be stored canonically, derived from symbol evidence, or both with explicit provenance?
8. Which change findings are advisory and which can eventually block a merge?
9. Can existing code-host review surfaces render enough of the Change Map, or is a dedicated local/web UI required?
10. Does visual review improve decisions after novelty wears off?
11. What architecture evidence is useful when there are no resolvable function calls?
12. How should cross-repository external symbols be versioned and refreshed?

## 26. Recommended Immediate Experiment

Do not begin by implementing the full universal format or redesigning every API.

Build one end-to-end vertical slice:

```text
Input:
  one Rust repository
  base Git revision
  candidate Git revision

Facts:
  repository, crate/module, file, function/method
  contains, depends_on, calls
  source spans and resolver confidence

Queries:
  direct callers
  direct callees
  file/module dependencies

Delta:
  added/removed/modified nodes
  added/removed edges

Output:
  Architecture Snapshot JSON
  static task-scoped Change Map
  clickable evidence links
  coverage and unresolved report
```

Use Tree-sitter and the existing graph as the baseline. Add rust-analyzer or SCIP evidence for the same repository and visually distinguish where semantic evidence changes the result.

Then put the artifact in front of maintainers reviewing real historical agent-sized changes.

The purpose of this experiment is not to prove that graphs can be drawn. It is to determine whether the visual artifact changes what a reviewer notices, investigates, or decides.

## 27. Source Map

### Local Sources

| Source | Relevance |
|---|---|
| [README.md](../../README.md) | Current endpoint surface, callers, callees, blast radius, edge types, centrality, and community endpoints |
| [Sol-Observations-01.md](Sol-Observations-01.md) | Broad product journeys, trust requirements, architecture patterns, failures, and algorithm cautions |
| [Sol-Insights-01.md](Sol-Insights-01.md) | Backend-neutral architecture, PMF ranking, pre/post edit loop, review workflows, and accuracy contract |
| [J003.md](../research002/J003.md) | Competitive repository graph and agent-tool landscape |
| [tree-sitter-ref-202606.md](../research002/tree-sitter-ref-202606.md) | Detailed SCIP, parser, semantic identity, environment, and agent-context research |
| [PRD-ARCH-UNIFIED.md](../research000/archive/archive-docs-v2/archive-p2/PRD-ARCH-UNIFIED.md) | Existing snapshot/entity/edge diff and blast-radius design |
| [Clarity README](../../git-ref-repo/ignore-this-folder-repos/LegacyCodeHQ__clarity-cli/README.md) | File/module structural impact, change review, cycles, and explicit API-contract limitations |
| [code-review-graph README](../../git-ref-repo/ignore-this-folder-repos/tirth8205__code-review-graph/README.md) | Existing local-first review graph, graph diff, surprise scoring, flows, test gaps, and CI product surface |

### External Sources

| Source | Relevance |
|---|---|
| [GraphML](https://graphml.graphdrawing.org/) | Generic graph exchange and hierarchy |
| [GEXF](https://docs.gephi.org/desktop/User_Manual/Import/GEXF_File_Format/) | Graph and visualization metadata exchange |
| [SCIP](https://github.com/scip-code/scip) | Language-neutral semantic symbols and occurrences |
| [LSP 3.17](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_prepareCallHierarchy) | Standard incoming and outgoing call hierarchy |
| [Kythe overview](https://kythe.io/docs/kythe-overview.html) | Language-neutral code graph and interoperability principles |
| [Code Property Graph specification](https://cpg.joern.io/) | Rich layered code graph model |
| [C4 model](https://c4model.com/) | Human architecture abstractions and multi-level communication |
| [SARIF](https://www.oasis-open.org/committees/sarif/) | Static-analysis result interchange |
| [Sourcegraph Code Navigation](https://sourcegraph.com/docs/code-navigation) | Search-based and precise code navigation distinction |
| [GitHub Copilot code review](https://docs.github.com/en/enterprise-cloud%40latest/copilot/concepts/agents/code-review) | Full-project agentic review and MCP/skill integration |
| [SonarQube architecture management](https://www.sonarsource.com/blog/code-architecture-management-general-availability-in-sonarqube/) | Current/intended architecture and dependency enforcement |
| [Qodo cross-repository review](https://docs.qodo.ai/governance/cross-repo-code-review) | Cross-repository signature and contract review |

## 28. Final Thesis

The project began from the premise that code graphs could give LLMs better context.

That remains true, but it is no longer sufficiently differentiated.

The more important problem is that humans cannot easily perceive what a coding agent changed as a system. Chat is a narrative. Git diff is implementation detail. Tests are partial evidence. A giant graph is overwhelming. None alone produces trust.

Parseltongue can connect them through one bounded artifact:

> A visual, source-linked, uncertainty-aware Change Map generated from versioned architecture snapshots.

The minimum representation is not all of software architecture. It is:

```text
containment
dependencies
calls
revision
evidence
coverage
delta
```

That is enough to show a reviewer:

- where the change lives;
- what new structural relationships appeared;
- what disappeared;
- what calls into the change;
- what the change calls out to;
- which architecture level is being viewed;
- what was expected;
- what was verified;
- what the system does not know.

The strategic discipline is to keep the core exact and legible. Advanced metrics can remain optional. The user should never need graph theory to decide whether an agent-generated change deserves trust.
