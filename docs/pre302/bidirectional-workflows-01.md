# Principal Architect Review: Parseltongue Bi-Directional Workflow Analysis

---

## 1. Premise Check

**What this system actually is, in blunt terms:**

A macOS desktop app that indexes a local codebase into SQLite, builds an in-memory graph in Python (igraph/NetworKit), and exposes a localhost HTTP API that returns small, structured JSON packets. LLMs and humans query it. No cloud. No GPU. No embeddings. No compiler in the loop (tree-sitter only for most languages).

**Most important implications of this stack:**

1. **Process boundary is real.** Tauri is Rust. Graph compute is Python. Every query-time graph operation crosses an IPC boundary. This means: precompute aggressively, keep the Python process warm as a sidecar, never invoke Python per-request as a cold subprocess. Latency budget for query-time graph ops is ~50-200ms, not seconds.

2. **Edge quality is the ceiling.** Tree-sitter gives you AST structure, not semantic resolution. Edges are symbolic name-based, not type-resolved. Dynamic dispatch, trait resolution, generic monomorphization — all invisible. Every workflow's quality is capped by edge extraction quality. This is not a weakness to apologize for — it is the defining constraint.

3. **The graph is a snapshot, not a live mirror.** The user indexes, then queries. Between indexes, the graph is stale. File watching can help, but the fundamental model is snapshot-based. Workflows that assume continuous freshness will lie.

4. **LLM consumption is the primary interface, not human eyeballs.** The HTTP API must return packets that an LLM can parse in <500 tokens to decide, <5000 tokens for a deep dive. If a response requires a human to visually scan a graph, it has failed.

5. **Python libraries are trusted for correctness, but integration is where bugs live.** igraph's PageRank is correct. Your node ID remapping from SQLite integer keys to igraph vertex indices is where you'll ship bugs. The risk surface is data shaping, not algorithm implementation.

**Two constraints that shape product truth:**

- **Constraint 1: No type resolution across files.** Without a compiler, "who calls `process()`" can match the wrong `process()` in a different module. This limits multi-hop traversals to ~2-3 hops before noise overwhelms signal. (Direct consequence of tree-sitter-only.)

- **Constraint 2: Precomputation latency vs freshness tradeoff.** Betweenness centrality on 50K nodes takes seconds. Leiden community detection takes seconds. You cannot run these per-query. But if they're stale by 1 commit, the hotspot ranking is wrong. The product must be honest about freshness timestamps on every cached metric.

---

## 2. Hard Thesis

1. **This product is a structural navigation engine, not a code intelligence platform.** It knows shapes and connections. It does not know types, semantics, or intent. Every workflow should stay within what structural graph traversal can actually prove.

2. **The core value loop is: search → converge → contextualize.** Not "analyze everything." Not "find all bugs." The product reduces a vague question to a precise subgraph, then packs that subgraph into tokens. That is the job.

3. **Bi-directionality is not a feature — it is the graph's native advantage over grep.** Grep can search forward. Only a graph can answer "who depends on this?" and "what is this reachable from?" in the same data structure. If a workflow doesn't exploit both directions, it doesn't justify the graph.

4. **The strongest moat is not algorithms — it is the progressive disclosure loop.** Leiden, PageRank, PPR are commodities. What is not a commodity: a deterministic, token-budgeted, graph-guided narrowing from 50K entities to 5 entities in 3 hops. That workflow is hard to replicate by dumping files into a 1M context window.

5. **Precomputation is the product.** The user waits during indexing, not during querying. Every metric that can be precomputed must be. Query-time graph operations must be BFS/PPR-class (milliseconds), never Betweenness-class (seconds).

6. **This product should refuse to optimize for:** real-time type-checked refactoring (use an IDE), security vulnerability scanning (needs taint analysis the stack can't do), code generation (not the job), and anything that requires runtime profiling data.

7. **The honest positioning is: "We know your codebase's shape. We don't know its meaning."** Shape is enough for navigation, impact analysis, and architecture review. Meaning is the LLM's job. The separation of labor is: graph computes structure, LLM interprets consequence.

8. **The biggest risk is not missing features — it is shipping confident answers from bad edges.** A wrong blast radius is worse than no blast radius. The product must surface confidence signals (edge resolution rate, unresolved references, snapshot age) alongside every answer.

---

## 3. Workflow Universe

| # | Workflow | Forward Question | Reverse Question | Primary Actor | Journey Stage(s) | Required Algorithms/Primitives | Precomputed or Query-Time | Product Value | Judgment |
|---|---------|-----------------|-------------------|---------------|-------------------|-------------------------------|--------------------------|---------------|----------|
| 1 | **Call graph forward/reverse** | What does X call? | Who calls X? | LLM, Human | DEEP DIVE | BFS on call edges | Query-time | Navigation foundation | **Must have** |
| 2 | **Public boundary resolution** | Which public API exposes this private helper? | What internals are beneath this public API? | LLM | ANCHOR | BFS reverse on call+contains edges with visibility filter | Query-time | Core journey step | **Must have** |
| 3 | **Blast radius** | What breaks if I change X? | What upstream change could explain this breakage? | LLM, Human | POST-DIVE | BFS forward (hops), In-degree for severity ranking | Query-time BFS + precomputed degree | Impact planning | **Must have** |
| 4 | **Dependency chain** | What does X transitively depend on? | What transitively depends on X? | LLM | DEEP DIVE, POST-DIVE | BFS/DFS forward/reverse on dependency edges | Query-time | Onboarding, understanding | **Must have** |
| 5 | **Ego network context** | What is the immediate neighborhood of X? | What entity has X in its neighborhood? | LLM | CLUSTER | PPR from seed X, ego network extraction, token budget packing | Query-time | Core journey step | **Must have** |
| 6 | **Hotspot ranking** | What are the most critical/coupled entities? | What is connected to this hotspot? | Human, LLM | POST-DIVE | PageRank + In-degree + k-core composite | Precomputed | Architecture review | **Strong** |
| 7 | **Dead code detection** | What has zero incoming call edges from public entry points? | What would become dead if I removed X? | Human | POST-DIVE | In-degree = 0 filter + BFS reachability from public roots | Precomputed (dead set) + query-time (removal simulation) | Cleanup, tech debt | **Strong** |
| 8 | **Cycle membership** | Is X part of a circular dependency? | What are all the members of X's cycle? | Human, LLM | POST-DIVE | SCC (igraph strongly_connected_components) | Precomputed | Architecture health | **Strong** |
| 9 | **Community/module boundary** | What natural cluster does X belong to? | What entities define this cluster? | LLM | CLUSTER, POST-DIVE | Leiden | Precomputed | Architecture map | **Strong** |
| 10 | **Cross-module coupling** | How tightly are modules A and B coupled? | What are the specific coupling edges? | Human | POST-DIVE | Cross-community edge count + enumeration | Precomputed count + query-time edges | Refactoring planning | **Strong** |
| 11 | **Root cause trace** | This entity is broken — trace backward to likely root | If X has a bug, what downstream entities are affected? | LLM | POST-DIVE | Reverse BFS + PageRank-weighted path ranking | Query-time BFS + precomputed PR | Debugging | **Strong** |
| 12 | **Interface surface map** | What is the public surface of module M? | What internal implementation supports this surface? | LLM | ANCHOR, DEEP DIVE | Visibility filter + contains edges + BFS inward | Query-time | Onboarding | **Strong** |
| 13 | **Change impact on tests** | Which test files are reachable from changed entity X? | What code path could have caused this test failure? | LLM, CI | POST-DIVE | BFS forward to test-tagged entities / reverse BFS from test | Query-time | CI integration | **Strong** |
| 14 | **Extraction candidate scoring** | Is module M loosely coupled enough to extract? | What would break if I extracted M? | Human | POST-DIVE | Leiden boundary + cross-boundary edge ratio + blast radius | Precomputed community + query-time blast | Architecture planning | **Strong** |
| 15 | **Bridge/chokepoint detection** | What entities are bridges between subsystems? | What alternative paths exist around this bridge? | Human | POST-DIVE | Betweenness centrality | Precomputed | Architecture review | **Strong** |
| 16 | **Core/periphery layering** | What is the dense architectural core? | What entities are peripheral? | Human | POST-DIVE | k-core decomposition | Precomputed | Architecture review | **Strong** |
| 17 | **Trait/interface dispatch** | Which impls satisfy this trait? | Which traits does this type implement? | LLM | DEEP DIVE | Edge-type filter on implements edges | Query-time | Navigation | **Strong** (limited by tree-sitter edge quality) |
| 18 | **Safe change boundary** | What is the minimal change set that doesn't leak past boundary B? | What is the maximal safe set within B? | LLM | POST-DIVE | Subgraph extraction with boundary constraint + blast radius | Query-time | Change planning | **Strong** |
| 19 | **Import/dependency listing** | What does this file import? | Who imports this file? | LLM | DEEP DIVE | Edge-type filter on import edges | Query-time | Table stakes navigation | **Must have** (but table stakes) |
| 20 | **Containment hierarchy** | What module contains X? | What does module M contain? | LLM | ANCHOR | Contains edges, trivial lookup | Query-time | Table stakes | **Must have** (but table stakes) |
| 21 | **Architecture simulation** | If I add/remove this edge, what structural changes result? | What mutation would fix this structural problem? | Human | POST-DIVE | Shadow graph clone + BFS/SCC/community diff | Query-time (expensive) | Differentiation play | **Strong** (but v1.2+) |
| 22 | **Type flow tracing** | Where can values of type T flow? | Where did this dangerous sink come from? | LLM | DEEP DIVE | DDG edges (data-flow) | Query-time | Understanding | **Weak** (tree-sitter can't reliably extract this) |
| 23 | **Temporal architecture drift** | How has X's connectivity changed since last snapshot? | What was X connected to N commits ago? | Human | POST-DIVE | Snapshot diff (requires dual-snapshot storage) | Precomputed diff | Long-term tracking | **Weak** (v1.2+ at earliest) |
| 24 | **Similar entity discovery** | What entities have similar graph structure to X? | Why are X and Y structurally similar? | Human | POST-DIVE | Graph kernel or structural similarity | Precomputed | Research-grade, low repeat use | **Kill** |
| 25 | **Semantic module naming** | What should this cluster be called? | What entities define the semantics of this name? | Human | POST-DIVE | Leiden + entity name frequency analysis | Precomputed + heuristic | Nice-to-have naming | **Weak** |
| 26 | **God function decomposition** | What functions have too many outgoing edges? | What clean split would reduce fan-out? | Human | POST-DIVE | Out-degree + community sub-partitioning | Precomputed degree + query-time | Refactoring suggestion | **Weak** (opinion-heavy, low confidence) |
| 27 | **Orphan detection** | What entities are disconnected from the main graph? | What is the nearest reachable entity from this orphan? | Human | POST-DIVE | Degree = 0 filter + nearest-connected BFS | Precomputed | Usually indicates bad extraction | **Weak** |
| 28 | **Dependency depth ranking** | What is the longest dependency chain? | What is on the critical path? | Human | POST-DIVE | Longest path in DAG (topological sort) | Precomputed | Moderate architecture insight | **Weak** |
| 29 | **Multi-language boundary** | Where do Rust and TypeScript interact? | What TypeScript depends on this Rust function? | LLM | DEEP DIVE | Cross-language edge filtering | Query-time | Polyglot value | **Strong** (if edge extraction supports it) |
| 30 | **Fan-in / fan-out heatmap** | What are the highest fan-in entities? | What are the highest fan-out entities? | Human | POST-DIVE | In-degree / Out-degree sorted | Precomputed | Quick architecture read | **Must have** (cheap, high signal) |

---

## 4. Kill List

### 1. Similar Entity Discovery (#24)
Sounds clever: "find structurally similar functions." In practice, structural similarity without semantic understanding produces noise. Two functions with identical fan-out patterns but completely different purposes will match. No user will trust this more than once. Graph kernels (WL, etc.) are research tools, not product features. The compute cost is high, the signal is low, and the repeat usage is zero.

### 2. God Function Decomposition (#26)
"What clean split would reduce fan-out?" — the system has no idea what a clean split is. It can count edges. It cannot reason about responsibilities. Suggesting decomposition from out-degree alone will produce wrong answers (a well-designed orchestrator has high fan-out by design). This is opinion territory dressed as analysis. Ship the raw metric (fan-out ranking); do not ship the suggestion.

### 3. Semantic Module Naming (#25)
"What should this cluster be called?" is an LLM job, not a graph job. The graph can identify the cluster. Naming it from entity name frequency is cute in demos and wrong in production (the most common token in a cluster is often `impl` or `new` or `test`). Let the LLM name things. The graph's job is grouping, not labeling.

### 4. Orphan Detection (#27)
Disconnected entities usually mean your edge extraction is broken, not that the code is dead. Shipping "orphan detection" as a feature will train users to distrust the graph when the extraction misses edges. Fix extraction quality instead. If you must expose it, expose it as a data quality diagnostic, not an analysis feature.

### 5. Dependency Depth Ranking (#28)
"What is the longest dependency chain?" sounds architectural but rarely drives action. The user can't do anything with "your deepest chain is 14 hops." Contrast with blast radius, which answers "if I change X, these 7 things break." Depth is trivia. Impact is actionable. The topological sort is trivially correct, but the product value is trivially low.

### 6. Type Flow Tracing (#22) — partial kill
Tree-sitter cannot reliably extract data-flow edges. It can see variable assignments within a function, not across function boundaries. Shipping "data flow" as a feature when the underlying edges are intra-function-only will disappoint anyone who tries to trace a value across a call chain. **If you build this, scope it honestly as "intra-function data flow" and do not promise cross-function tracing.**

---

## 5. Top 10 Workflows (Ranked)

### 1. Call Graph Forward/Reverse (#1)

**Why users come back:** Every debugging session, every code review, every "I need to understand this function" starts here. "What does it call?" and "Who calls it?" are the two questions developers ask 50x per day.

**Why the graph is necessary:** Grep finds string matches. The graph knows actual call relationships with direction. `process` as a grep hit gives you 200 matches. `process` in the call graph gives you 3 callers and 5 callees, correctly.

**Why this stack works:** BFS on igraph adjacency is sub-millisecond. No precomputation needed. The call edge extraction from tree-sitter is the most reliable edge type (function calls are syntactically unambiguous in most languages).

### 2. Public Boundary Resolution (#2)

**Why users come back:** LLMs search for implementation details and find private helpers. The user doesn't care about the helper — they care about the entry point. "You searched for `validate_token` — that's exposed through `login_handler`" is worth the entire product.

**Why the graph is necessary:** No flat search can answer "walk callers upward until you hit a public function." This requires reverse traversal with a visibility predicate. The graph is the only data structure that supports this.

**Why this stack works:** BFS reverse with a filter is trivial in igraph. The limiting factor is visibility extraction from tree-sitter (Rust `pub` is syntactically visible; Python/JS visibility is heuristic). Honest confidence signals per language are needed.

### 3. Blast Radius (#3)

**Why users come back:** Before every change. Before every PR review. "If I touch this, what else is affected?" is the single most repeated architectural question in software engineering.

**Why the graph is necessary:** Blast radius is multi-hop forward reachability. Grep cannot compute it. File-level dependency lists undercount it. Only the graph can say "changing `auth::validate` affects 3 direct callers, which affect 11 transitive dependents, 2 of which are in test files."

**Why this stack works:** BFS forward with hop limit. Sub-millisecond. In-degree precomputed as severity ranking within the blast set. Token budget can cap the response at exactly what the LLM can consume.

### 4. Ego Network Context Packing (#5)

**Why users come back:** This is the CLUSTER step — the moment the product proves it's smarter than grep. Instead of "here are 50 search results," the user gets "here is the 3000-token neighborhood of the entity you care about, ranked by relevance to that entity."

**Why the graph is necessary:** PPR from a seed node produces a relevance-ranked neighborhood that a keyword search cannot replicate. The graph knows which neighbors matter most to X, not just which neighbors contain similar text.

**Why this stack works:** igraph's `personalized_pagerank()` with a seed vector is fast and correct. Token budget packing is a second pass: sort PPR scores descending, pack entities into the budget greedily.

### 5. Hotspot Ranking (#6)

**Why users come back:** New developer onboarding, architecture review, tech debt triage. "What are the 10 most critical entities in this codebase?" is a question with high repeat value for team leads and LLM agents doing codebase-level analysis.

**Why the graph is necessary:** A function that appears in many files (grep frequency) is not the same as a function that sits at the center of the dependency graph (PageRank). `HashMap::new` appears everywhere but is not a hotspot. `UserService::authenticate` has moderate frequency but maximum centrality.

**Why this stack works:** Composite of precomputed PageRank + In-degree + k-core shell. All precomputed, all cached. The HTTP API serves this from SQLite in <10ms.

### 6. Dead Code Detection (#7)

**Why users come back:** Every quarter, someone asks "what can we delete?" This is the only tool that gives a graph-verified answer instead of a guess.

**Why the graph is necessary:** Dead code = unreachable from any public entry point. This is a reachability problem. Grep cannot solve it (a function that is referenced in a comment is not alive). The graph can compute: run BFS from all public entry points, anything not visited is dead.

**Why this stack works:** Precompute the reachable set from tagged public entry points. Store the dead set. The query-time reverse ("what would become dead if I removed X?") is a BFS minus operation.

### 7. Cycle Membership (#8)

**Why users come back:** Circular dependencies are the #1 architecture smell that teams actively hunt. "Show me all cycles" is a common CI gate. "Is my entity in a cycle?" is a common question during refactoring.

**Why the graph is necessary:** Cycle detection requires SCC computation. Grep cannot find cycles. Even file-level dependency tools often miss transitive cycles.

**Why this stack works:** igraph `clusters(mode="strong")` is correct and fast. Precompute SCC membership per snapshot. Query-time lookup is O(1).

### 8. Community/Module Boundary (#9)

**Why users come back:** "What are the natural modules in this codebase?" is the starting question for architecture review. Leiden communities often reveal module structures that don't match folder structure — that's the insight.

**Why the graph is necessary:** Folder structure is a human convention. The graph reveals actual coupling structure. When Leiden says "these 15 entities from 4 different folders form one community," that's a finding you can't get from `tree`.

**Why this stack works:** igraph `community_leiden()` is the reference implementation. Precompute per snapshot. Store community assignments in SQLite. Serve as metadata on every entity response.

### 9. Root Cause Trace (#11)

**Why users come back:** "This test is failing. Why?" — trace backward through the call graph from the failing assertion, ranked by PageRank (most important upstream entities first). This is the reverse of blast radius, and equally useful.

**Why the graph is necessary:** The reverse dependency chain IS the root cause candidate list. PageRank ranking within that chain prioritizes the most likely culprits (high-centrality entities that changed recently).

**Why this stack works:** Reverse BFS is query-time. PageRank ranking is precomputed. Combine at serve time. If git recency is wired, it further sharpens the ranking.

### 10. Cross-Module Coupling Analysis (#10)

**Why users come back:** "Should we split this service?" starts with "how coupled are these two modules?" The coupling metric (cross-community edge count / total edge count) is a number that directly drives architectural decisions.

**Why the graph is necessary:** Coupling is not "how many imports." It's "how many actual dependency edges cross this boundary, relative to the total edge budget." The graph computes this exactly.

**Why this stack works:** Precomputed Leiden boundaries + edge table query in Polars. The specific coupling edges are query-time (filter edge table by source community ≠ target community).

---

## 6. API Consequences

### SEARCH

```
POST /search
  Body: { "q": "authentication", "top_k": 10 }
  Returns: ranked entity list (id, name, location, score, match_source)
```

One endpoint. RRF fuses all signals server-side. The client never sees individual retrieval channels. Return **compact**: entity ID, display name, file:line:line, RRF score, which signal contributed most. ~30-50 tokens per result. **Cached:** FTS5 index + symbol trie + trigram index are all prebuilt at index time.

### ANCHOR

```
GET /anchor?entity={id}
  Returns: resolved public boundary entity + path from query to boundary
```

One endpoint. Takes any entity, returns the public API boundary it belongs to. If the entity is already public, returns itself. Returns the traversal path (chain of callers) so the LLM understands how they're connected. ~50-100 tokens. **Never cached** — this is a BFS per query, but it's sub-millisecond.

### CLUSTER

```
GET /cluster?entity={id}&budget={tokens}
  Returns: ego network under token budget, ranked by PPR
```

One endpoint. The `budget` parameter is critical — it forces the response to fit within what the LLM can consume. Returns entity summaries (name, kind, signature, location, PPR score), not full source code. ~200-500 tokens. **Query-time** PPR from seed, **precomputed** PageRank as global prior for tie-breaking.

### DEEP DIVE

```
GET /deep-dive?entity={id}&depth={hops}&budget={tokens}
  Returns: full context packet (source spans, callers, callees, type sigs, community, metrics)
```

One endpoint. This is the big payload — up to 5K-20K tokens. Source code is read from disk at serve time (not stored in DB). Includes callers, callees, implementations, containing module, community membership, centrality scores, git recency. **Hard call:** source is read from disk, not DB. Graph context is from precomputed cache + query-time BFS.

```
GET /deep-dive/source?entity={id}
  Returns: raw source span for the entity
```

Separate endpoint for source-only. Sometimes the LLM just wants the code, not the graph context.

### POST-DIVE

These are separate endpoints, each with a clear single purpose:

```
GET /blast-radius?entity={id}&hops={n}
  Returns: affected entities, ranked by severity (in-degree), grouped by hop distance

GET /upstream-trace?entity={id}&hops={n}
  Returns: reverse dependency chain, ranked by PageRank

GET /hotspots?top={n}&metric={pagerank|degree|kcore|composite}
  Returns: ranked entity list with metric scores

GET /dead-code?scope={all|module_id}
  Returns: entities unreachable from public entry points

GET /cycles?entity={id}
  Returns: SCC membership for entity, or all non-trivial SCCs if no entity

GET /communities
  Returns: Leiden community assignments, modularity score, cross-boundary edge counts

GET /coupling?community_a={id}&community_b={id}
  Returns: coupling metric + specific crossing edges

GET /core-periphery
  Returns: k-core shell assignments for all entities
```

**Hard calls:**

- Blast radius and upstream trace are **separate endpoints**, not one parameterized endpoint. The mental models are different (forward impact vs reverse diagnosis).
- Hotspots, dead code, cycles, communities, core-periphery are all **precomputed and cached**. These endpoints read from SQLite, not from igraph.
- Coupling is **mixed**: community assignments are precomputed, specific edges are query-time filtered.
- **No endpoint should return >100 entities without explicit pagination.** Default limits everywhere.

### META

```
GET /health
  Returns: snapshot age, entity count, edge count, edge resolution rate, stale files count

GET /stats
  Returns: per-language entity counts, edge type distribution, community count, SCC count
```

The `/health` endpoint is critical for trust. It must report **snapshot age** and **edge resolution rate** (what % of edges resolved to real entity PKs vs symbolic names). An LLM that knows the graph is 3 hours stale can adjust its confidence.

---

## 7. Algorithm Accountability

| Algorithm/Primitive | Why It Exists | Workflows That Justify It | Compute Mode | Library | Expected Data Shape | Risk If Misused |
|---|---|---|---|---|---|---|
| **BFS** | Forward/reverse graph traversal | #1 call graph, #2 anchor, #3 blast radius, #4 dependency chain, #11 root cause | Query-time | python-igraph | Directed edge list, integer vertex IDs | Low risk. Cap hop depth to prevent full-graph walks. Max 4 hops for query-time. |
| **PageRank** | Global importance ranking | #5 ego network tie-breaking, #6 hotspots, #11 root cause ranking | Precomputed | python-igraph | Directed weighted adjacency | Must handle dangling nodes (no outgoing edges). Add damping correction. Recompute per snapshot only. |
| **Personalized PageRank** | Local relevance from a seed | #5 ego network, CLUSTER packing | Query-time (or cached per hot anchor) | python-igraph | Directed adjacency + seed vector | Seed vector must be single-node. Multi-seed PPR is ambiguous. Cache top-20 hot anchors. |
| **In-Degree** | Fan-in signal, importance proxy | #3 blast radius severity, #6 hotspots, #7 dead code (degree=0), #30 heatmap | Precomputed | Polars grouped count | Edge table `dst_id` column | Trivially correct. Zero risk. |
| **Out-Degree** | Fan-out signal, complexity proxy | #6 hotspots, #30 heatmap, god function detection | Precomputed | Polars grouped count | Edge table `src_id` column | Trivially correct. Zero risk. High out-degree is not always bad (orchestrators). |
| **Betweenness** | Bridge/chokepoint detection | #15 bridge detection, architecture review | Precomputed | NetworKit | Undirected or directed graph object | **Expensive.** O(VE) on large graphs. Must precompute, never query-time. On 50K+ nodes, consider approximate betweenness (NetworKit `ApproxBetweenness`). |
| **k-core** | Core/periphery layering | #16 core-periphery, hotspot composite | Precomputed | NetworKit | Graph object, unweighted | Low risk. Ensure the graph is undirected for k-core (or use directed variant explicitly). |
| **Leiden** | Community detection | #9 module boundaries, #10 coupling, #14 extraction scoring | Precomputed | python-igraph | Weighted edge list, resolution parameter | **Resolution parameter is critical.** Default resolution=1.0 is rarely optimal for code graphs. Test multiple resolutions, pick the one maximizing modularity. Expose resolution as a config knob. |
| **RRF** | Multi-signal search fusion | SEARCH phase | Query-time | Custom Python | Ranked lists per signal: `[(entity_id, rank)]` | k=60 is standard. Risk: if one signal is garbage (e.g., trigram on very short queries), it contaminates the fusion. Weight signals by query length heuristic. |
| **Symbol trie** | Exact/prefix symbol lookup | SEARCH phase | Precomputed (built at index time) | `marisa-trie` or Rust `fst` | Static string vocabulary → entity ID mapping | Must rebuild on re-index. Stale trie after code change is a silent correctness bug. |
| **Trigram retrieval** | Fuzzy/partial name matching | SEARCH phase | Precomputed (index) + query-time (match) | `RapidFuzz` | Symbol strings | Garbage on queries <3 characters. Gate: skip trigram signal for single-word queries shorter than 4 chars. |
| **Ego network extraction** | Neighborhood packing | CLUSTER | Query-time | python-igraph (neighborhood + PPR) | Directed adjacency + seed node | Budget overflow risk: ego network at depth=2 can be 500+ entities on hub nodes. Always enforce token budget truncation. |
| **Token budget packing** | Constrain output size | CLUSTER, DEEP DIVE | Query-time | Custom Python | Sorted entity list + per-entity token cost | Must use actual word count (from DB), not estimated. Greedy packing by PPR score. Do not approximate. |
| **DFS** | Deep path trace, reachability | DEEP DIVE path drilling | Query-time | python-igraph | Directed adjacency | Stack overflow risk on deep recursive graphs. Use iterative DFS with explicit stack. Cap depth. |
| **SCC (Tarjan)** | Cycle detection | #8 cycle membership | Precomputed | python-igraph `clusters(mode="strong")` | Directed edge list | Correct out of the box. Only risk: SCC of size 1 is not a cycle — filter to size >= 2. |

---

## 8. Moat Analysis

### Table Stakes (anyone can build these)

| Workflow | Why It's Table Stakes |
|---|---|
| Call graph forward/reverse (#1) | Every IDE, every code intelligence tool, every LSP server does this. The graph makes it more reliable for cross-file, but the UX is not novel. |
| Import/dependency listing (#19) | `go mod graph`, `cargo tree`, `pydeps` — commodity. |
| Containment hierarchy (#20) | File browser. Folder tree. Every editor has this. |
| Fan-in / fan-out heatmap (#30) | A better `wc -l` for dependencies. Useful, not differentiating. |

### Strong Value (worth the product, not a moat)

| Workflow | Why It's Strong But Not A Moat |
|---|---|
| Blast radius (#3) | Very useful. But GitHub Copilot will eventually compute this from context. The graph is better today; context windows are catching up. |
| Dead code detection (#7) | Useful, but tree-shaking tools, IDE warnings, and linters compete. The graph-verified version is better, but the workflow is established. |
| Cycle membership (#8) | Useful, but `cargo clippy`, `pylint` circular import detection exist. Graph version is more complete, but the workflow is established. |
| Hotspot ranking (#6) | Useful for leads, but code climate / CodeScene already do this with git churn. The graph version is structurally better but competes with established tools. |
| Community detection (#9) | Strong insight, but a Leiden partition on its own is "here are some clusters." The value comes from what you DO with the clusters. |

### Actual Moat Candidates

| Workflow | Why It's A Moat |
|---|---|
| **Public boundary resolution (#2)** | No other tool answers "what public API exposes this private implementation?" This requires reverse graph traversal with visibility predicates. IDEs can "find usages" but cannot walk callers to the boundary. Grep certainly cannot. |
| **Ego network context packing (#5)** | Token-budgeted, PPR-ranked neighborhood packing is not a feature any existing tool offers. It's the core of the progressive disclosure loop. Dumping files into context windows is the competitor, and this is structurally better. |
| **Architecture simulation (#21)** | "What if I moved this module?" with exact structural consequences (new cycles, broken reachability, changed centrality) does not exist anywhere. It requires a mutable shadow graph. This is the endgame moat. |
| **Root cause trace with graph ranking (#11)** | Reverse BFS + PageRank ranking produces a structurally-ranked "suspect list" for a bug. No other debugging tool uses graph centrality to rank root cause candidates. |
| **Cross-module coupling quantification (#10)** | "Modules A and B have 14 crossing edges out of 200 total, coupling ratio 0.07" — this exact number does not come from any current tool. Code climate gives heuristics. This gives graph truth. |
| **Progressive disclosure journey (SEARCH → ANCHOR → CLUSTER → DIVE)** | The entire journey is the moat. Each step narrows from 50K entities to 5, using graph structure at every step. No tool does this. Context window dumping is the only alternative, and it scales worse. |

**Honest callout:** Some "graph" workflows (blast radius, dead code, cycles) are really just better packaging of existing concepts. They are strong product features but not moats. The moat is in workflows that require graph-guided progressive narrowing — things that fundamentally cannot be replicated by throwing more tokens at the problem.

---

## 9. Failure Modes

| Failure Mode | What Goes Wrong | How It Manifests | Mitigation |
|---|---|---|---|
| **Stale snapshot** | User edits 5 files, queries without re-indexing. Graph doesn't reflect current code. | Blast radius misses newly added call. Dead code includes a function that was just connected. Anchor resolves to a deleted function. | File watcher marks files as dirty. Every API response includes `snapshot_age_seconds` and `stale_file_count`. LLM/human can decide to re-index or proceed with caveat. |
| **Unresolved edges** | Tree-sitter extracts `call("process")` but can't resolve which `process`. Edge points to wrong entity or to a phantom node. | BFS traverses through a wrong connection. Blast radius includes unrelated code. Cycle detection finds phantom cycles. | 2-pass ingestion with name resolution (same-file > same-dir > first-match > drop). Report `edge_resolution_rate` on `/health`. Warn on responses when resolution is <80%. |
| **Bad anchor resolution** | BFS upward from private function finds a test helper before the production API boundary. | "This private function is exposed via `test_auth_helper`" — wrong and confusing. | Tag test files/functions during extraction. BFS anchor should skip test-tagged entities. Allow manual boundary tagging via config. |
| **Meaningless communities** | Leiden on a sparse graph with poor edges produces random-looking communities that don't match any human intuition about modules. | "These 30 entities from 12 folders are one community" — user loses trust. | Compute modularity score. If modularity < 0.3, suppress community features and fall back to folder-based grouping. Surface modularity in `/communities` response. |
| **Expensive analytics in hot path** | Betweenness centrality recomputed on every query. Leiden run per request. | 2-5 second latency on what should be <200ms endpoints. | Strict rule: Betweenness, k-core, Leiden, PageRank, SCC are ONLY precomputed. Never query-time. Store in SQLite `cached_metrics` table. Serve from cache. |
| **LLM-unfriendly payloads** | Deep dive returns 500 entities with full source code. Token budget ignored. | LLM context overflows. LLM hallucinates from noisy context. User sees degraded LLM performance. | Enforce `budget` parameter on CLUSTER and DEEP DIVE. Default budget of 4000 tokens. Hard cap at 20K. Return `truncated: true` if budget was hit. |
| **Hub node explosion** | Entity with 200 callers produces ego network of 2000 entities at depth 2. | CLUSTER response is enormous. PPR is slow on dense subgraph. | Cap ego network to depth=1 for entities with degree > 50. Use PPR top-k truncation (return top 20 neighbors by PPR score, not all neighbors). |
| **Node ID remapping bugs** | SQLite entity IDs don't match igraph vertex indices after graph rebuild. Cached metrics point to wrong entities. | PageRank score for entity 42 is actually the score for entity 71. Silent data corruption. | Single source of truth for ID mapping. Build mapping table once at graph construction. Validate: `assert len(mapping) == igraph.vcount()`. Store mapping version in cache metadata. Invalidate all caches on re-index. |

---

## 10. Build Order

### v1 — "The Journey Works"

**User feels:** "I searched, it showed me the right neighborhood, I got useful context."

| What ships | Workflows enabled |
|---|---|
| Tree-sitter entity + edge extraction (2-pass with name resolution) | Foundation for everything |
| SQLite schema: entities, edges, indexed_files, snapshots | Durable storage |
| RRF search (FTS5 + symbol trie + trigram) | SEARCH (#3 workflow) |
| BFS anchor resolution with visibility filter | ANCHOR (#2) |
| PPR ego network + token budget packing | CLUSTER (#5) |
| Deep dive: source from disk + callers + callees + signatures | DEEP DIVE (#1, #4) |
| Precomputed PageRank + In-degree + Out-degree | Ranking signals for all above |
| Blast radius (forward BFS + degree-ranked) | POST-DIVE (#3) |
| File-level invalidation (SHA-256 hash comparison) | Incremental re-index |
| `/health` with snapshot age + edge resolution rate | Trust signal |

**v1 endpoint count:** 7 (`/search`, `/anchor`, `/cluster`, `/deep-dive`, `/deep-dive/source`, `/blast-radius`, `/health`)

### v1.1 — "The Architecture View"

**User feels:** "I can see the architecture, find the problems, and understand the shape."

| What ships | Workflows enabled |
|---|---|
| Precomputed SCC (cycle detection) | Cycle membership (#8) |
| Precomputed Leiden communities + modularity score | Community/module boundaries (#9) |
| Precomputed k-core decomposition | Core-periphery layering (#16) |
| Upstream trace (reverse BFS + PageRank ranking) | Root cause trace (#11) |
| Dead code detection (reachability from public entry points) | Dead code (#7) |
| Hotspot composite metric (PR + degree + k-core) | Hotspot ranking (#6) |
| Cross-module coupling metric | Coupling analysis (#10) |

**v1.1 adds:** 7 endpoints (`/upstream-trace`, `/cycles`, `/communities`, `/dead-code`, `/hotspots`, `/core-periphery`, `/coupling`)

### v1.2 — "The Planning Tool"

**User feels:** "I can plan changes, evaluate extraction candidates, and simulate mutations."

| What ships | Workflows enabled |
|---|---|
| Precomputed Betweenness centrality (approximate for large graphs) | Bridge detection (#15) |
| Change impact on tests (BFS to test-tagged entities) | Test impact (#13) |
| Extraction candidate scoring (community boundary + coupling ratio) | Extraction scoring (#14) |
| Safe change boundary computation | Safe change planning (#18) |
| Interface surface map (visibility filter + contains) | Interface surface (#12) |
| Architecture simulation (shadow graph clone, mutate, diff) | Simulation (#21) |

**v1.2 adds:** 4 endpoints (`/bridge-points`, `/test-impact`, `/extraction-candidates`, `/simulate`)

### Later

| What ships | Why later |
|---|---|
| Temporal snapshot comparison | Requires dual-snapshot storage, low repeat usage until the product has traction |
| Multi-language boundary detection | Requires cross-language edge extraction quality that tree-sitter alone can't guarantee |
| Spectral clustering, Infomap, Walktrap | P2 algorithms with diminishing returns over Leiden |
| Closeness, Harmonic, Eigenvector, Katz, HITS | P2 centrality variants — PageRank + Betweenness cover 90% of use cases |

---

## 11. Final Verdict

**The single best workflow category:** Forward/reverse navigation with progressive disclosure (SEARCH → ANCHOR → CLUSTER → DIVE). This is the daily-use loop. Every other workflow is built on top of this.

**The most overrated workflow category:** Advanced centrality variants (Closeness, Harmonic, Eigenvector, Katz, HITS). They all answer slightly different flavors of "how important is this node" and PageRank + Betweenness already cover the product-relevant cases. The remaining variants are academic distinctions that no user will ever ask for by name.

**The best moat workflow:** Public boundary resolution (anchor detection). It is the single workflow that most clearly demonstrates why the graph exists and that cannot be replicated by dumping files into context. It transforms a search hit into an architectural understanding in one step.

**The best workflow for LLM consumption:** Ego network context packing under token budget. It gives the LLM exactly the right amount of context, ranked by graph-computed relevance, fitting within a specified token constraint. This is what makes graph-guided context fundamentally better than "here are the top 10 files that matched your query."

**The one-sentence product thesis:**

Parseltongue is a local-first structural navigation engine that uses graph traversal to progressively narrow 50,000 entities to the 5 that matter, delivering deterministic, token-budgeted context packets that no context-window dump can replicate.
