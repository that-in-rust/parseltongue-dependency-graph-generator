# Tree-Sitter Patterns 4: LLM Companion, Agentic Context, MCP Tools, Repo Maps

This file captures patterns for turning Tree-sitter-powered repository understanding into useful LLM and agent workflows.

## Phase 0 - Deconstruct and Clarify

Core objective for this slice: identify patterns that help Parseltongue serve agents, code assistants, and developers. The focus is context construction, repo maps, token budgeting, tool surfaces, query safety, summarization, provenance, and human-debuggable outputs.

Premise is sound. Proceeding with optimized protocol.

Important evidence boundary: this file uses direct local evidence from Aider, CodeGraphContext, Tabby, wrale's Tree-sitter MCP server, and supporting scans. It includes Rust-facing design recommendations inferred from that evidence; recommendations are labeled as translations rather than claims about existing Parseltongue implementation.

## Phase 1 - Cognitive Staging

Expert council used for this slice:

- Agent Context Architect: designs context windows, repo maps, ranking, and summary retention.
- MCP Tooling Engineer: studies tool surfaces, schemas, background jobs, query guards, and watch/update flows.
- Retrieval Systems Engineer: connects code chunks, source filters, lexical search, graph signals, and reranking.
- Rust Product Engineer: turns companion behavior into typed APIs, CLI ergonomics, and safe defaults.
- Skeptical Systems Engineer: challenges token waste, stale context, misleading graph answers, unsafe query tools, and false confidence.

Knowledge scaffolding:

- Repo maps: tags, definitions, references, graph ranking, file personalization, important files, token fitting.
- Context budgets: system messages, chat history, repo map, files, read-only files, max context window.
- Summarization: head/tail splitting, recursive compaction, reserved buffers, role-aware summaries.
- Tooling: MCP schemas, background jobs, watch directory, graph queries, read-only guards, path validation.
- Retrieval: document/chunk indexes, filters, source IDs, language filters, current-file exclusion, overfetch.
- Agent provenance: every context item should know why it was selected and where it came from.

## Phase 2 - Multi-Perspective Synthesis

Conventional approach: expose a `search_code` command, stuff matching snippets into the prompt, and let the LLM reason from there.

Alternative 1 - Detective work blend: every context item is a clue with chain of custody. The agent should know who found it, why it was selected, what source span it came from, and whether the evidence is complete or truncated.

Alternative 2 - Library science blend: repo maps are not summaries; they are catalogs. Tags, chunks, graph edges, and file metadata are index cards that help an agent request the right shelf, not the entire library.

Alternative 3 - Air traffic control blend: context construction is traffic management. System prompt, chat history, repo map, editable files, read-only files, tool results, and summaries compete for runway space under a hard token budget.

Selected path: detective work plus air traffic control. Parseltongue should return evidence-bearing context packages with explicit token budgets, provenance, confidence, and fallback routes.

Council debate summary:

- Agent Context Architect: repo maps should be ranked, token-fitted, and adjusted based on files already in chat.
- Skeptical Systems Engineer: repo maps can become stale or misleading; query tools can mutate graphs or leak paths if not guarded.
- MCP Tooling Engineer response: expose background jobs, read-only query enforcement, job status, and watch-directory updates as first-class tools.
- Retrieval Systems Engineer response: use structured filters and overfetch, then rank by graph and task signals.
- Rust Product Engineer response: make the context package typed and inspectable, with token accounting and source spans.

Core thesis: Parseltongue should behave like an evidence router for agents. It should parse and index code into source-spanned facts, rank those facts under a token budget, expose safe tools for retrieval and graph questions, and render context with enough provenance that an agent can verify before editing.

## Phase 3 - Verification Anchors

Primary local evidence used:

- `Aider-AI__aider/aider/repomap.py`
- `Aider-AI__aider/aider/models.py`
- `Aider-AI__aider/aider/history.py`
- `Aider-AI__aider/aider/commands.py`
- `CodeGraphContext__CodeGraphContext/src/codegraphcontext/tool_definitions.py`
- `CodeGraphContext__CodeGraphContext/tests/unit/utils/test_cypher_readonly.py`
- `CodeGraphContext__CodeGraphContext/tests/unit/core/test_cgcignore_core.py`
- `TabbyML__tabby/crates/tabby-common/src/index/mod.rs`
- `TabbyML__tabby/crates/tabby-common/src/index/code/mod.rs`
- `wrale__mcp-server-tree-sitter/src/mcp_server_tree_sitter/...`

Self-correction questions:

- Does local evidence show explicit repo-map token sizing? Yes, Aider's `get_repo_map_tokens` defaults to 1024, uses max input tokens divided by 8, caps at 4096, and floors at 1024.
- Does local evidence show context accounting by category? Yes, Aider's `/tokens` command reports system messages, chat history, repository map, editable files, and read-only files.
- Does local evidence show repo-map fitting under a token target? Yes, Aider's `get_ranked_tags_map_uncached` binary-searches the number of ranked tags to fit a map token budget.
- Does local evidence show MCP-like code graph tools? Yes, CodeGraphContext's `tool_definitions.py` defines indexing, job status, code search, relationship analysis, watch, read-only Cypher, package add, delete, and visualization tools.
- Does local evidence show read-only query safety tests? Yes, CodeGraphContext has `tests/unit/utils/test_cypher_readonly.py`.

## Pattern 1 - Repo Map Token Budget Should Scale With Model Window

Where found:

- Repository: `Aider-AI__aider`
- File: `aider/models.py`
- Language: Python

Observed shape:

`get_repo_map_tokens` defaults to 1024. If model max input tokens are known, repo-map tokens become `max_input_tokens / 8`, capped at 4096 and floored at 1024.

Why this matters:

Repo maps are useful only when they leave room for the user's task, chat history, editable files, and tool outputs.

Why it matters for Parseltongue:

Parseltongue should not use a fixed context size for all models. It should size repo summaries relative to available context and task type.

Rust translation:

```rust
pub struct ContextBudgetPolicy {
    pub default_repo_map_tokens: usize,
    pub repo_map_fraction: f32,
    pub repo_map_min_tokens: usize,
    pub repo_map_max_tokens: usize,
    pub reserved_response_tokens: usize,
}

impl ContextBudgetPolicy {
    pub fn repo_map_budget(&self, model_window: Option<usize>) -> usize {
        let Some(window) = model_window else {
            return self.default_repo_map_tokens;
        };
        ((window as f32 * self.repo_map_fraction) as usize)
            .clamp(self.repo_map_min_tokens, self.repo_map_max_tokens)
    }
}
```

When to use:

- Repo-map rendering.
- Code explanation.
- Search summaries.
- Agent planning context.

When not to use:

- Do not hard-code Aider's exact constants without measuring Parseltongue's context shape.

Risks and caveats:

- Larger context windows do not mean all extra space should become repo map.
- Context quality can degrade when too much low-value code is included.

Testing implications:

- Unit-test budget calculation for small, medium, and large model windows.
- Snapshot context packages under budgets.

Agent guidance:

When generating context, declare the token budget and how much each category consumed.

## Pattern 2 - Repo Maps Expand When No Files Are Already In Chat

Where found:

- Repository: `Aider-AI__aider`
- File: `aider/repomap.py`
- Language: Python

Observed shape:

When no chat files are present, Aider expands repo-map budget by multiplying map tokens, while still preserving a 4096-token padding against the context window.

Why this matters:

If the agent has no local anchor, a broader map helps orientation. Once files are already in context, the repo map can shrink and focus on relationships.

Why it matters for Parseltongue:

Context selection should adapt:

- cold start: broader repository orientation,
- focused edit: narrower dependency and test context,
- review: changed files plus callers/callees,
- bug hunt: error paths and relevant tests.

Rust translation:

```rust
pub enum ContextMode {
    ColdStart,
    FocusedEdit,
    CodeReview,
    BugInvestigation,
    RefactorPlan,
}

pub struct ContextRequest {
    pub mode: ContextMode,
    pub anchor_files: BTreeSet<FileId>,
    pub mentioned_symbols: BTreeSet<SymbolName>,
    pub max_tokens: usize,
}
```

When to use:

- Initial repository orientation.
- First agent turn in a new repo.
- Switching tasks.

When not to use:

- Do not keep broad orientation context once the task has narrowed.

Risks and caveats:

- Cold-start maps can include too much irrelevant code.
- Expanding context without ranking can hurt accuracy.

Testing implications:

- Compare cold-start and focused-edit context packages for the same repo.
- Assert focused mode includes anchors and direct relations first.

Agent guidance:

Ask for a broader map only at orientation time. After anchors are known, retrieve around anchors.

## Pattern 3 - Token Accounting Should Be User-Visible

Where found:

- Repository: `Aider-AI__aider`
- File: `aider/commands.py`
- Language: Python

Observed shape:

The `/tokens` command accounts for:

- system messages,
- chat history,
- repository map,
- editable files,
- read-only files,
- image files,
- total tokens and remaining context window.

Why this matters:

Context is a limited resource. Users and agents need to know what is consuming it.

Why it matters for Parseltongue:

Parseltongue context packages should include budget reports:

- selected chunks,
- dropped candidates,
- truncation,
- parse diagnostics,
- repo-map size,
- source snippets,
- graph evidence.

Rust translation:

```rust
pub struct ContextBudgetReport {
    pub model_window_tokens: Option<usize>,
    pub categories: Vec<ContextBudgetCategory>,
    pub total_estimated_tokens: usize,
    pub dropped: Vec<DroppedContextCandidate>,
}
```

When to use:

- Agent responses.
- CLI context previews.
- Debugging poor LLM answers.

When not to use:

- Do not require verbose token reports in every short command. Provide compact and verbose modes.

Risks and caveats:

- Token estimates differ by model/tokenizer.
- Counting source snippets after rendering is safer than estimating before formatting.

Testing implications:

- Snapshot rendered context and token report together.
- Test budget overflow paths.

Agent guidance:

When context is truncated, say what was dropped and why.

## Pattern 4 - Long Text Token Counts Can Be Estimated By Sampling

Where found:

- Repository: `Aider-AI__aider`
- File: `aider/repomap.py`
- Language: Python

Observed shape:

For text longer than 200 characters, Aider samples roughly 100 evenly spaced lines, tokenizes the sample, and scales up to estimate total tokens.

Why this matters:

Exact tokenization of large repo maps during ranking loops can be expensive.

Why it matters for Parseltongue:

Parseltongue may need fast estimates while selecting chunks, then exact counts for final rendered context.

Rust translation:

```rust
pub trait TokenEstimator {
    fn estimate_tokens_fast(&self, text: &str) -> usize;
    fn count_tokens_exact(&self, text: &str, model: ModelId) -> Result<usize, TokenError>;
}
```

When to use:

- Ranking loops.
- Binary search over repo-map size.
- Preliminary chunk packing.

When not to use:

- Do not rely on estimates for final hard context-limit enforcement.

Risks and caveats:

- Generated code, minified files, and long lines can defeat line sampling.
- Different tokenizers vary.

Testing implications:

- Compare estimate vs exact over representative source files.
- Include minified, Markdown, Rust, TypeScript, Python, and JSON fixtures.

Agent guidance:

Use fast estimates for search; exact-count the final context payload.

## Pattern 5 - Fit Repo Maps With Search, Not Guesswork

Where found:

- Repository: `Aider-AI__aider`
- File: `aider/repomap.py`
- Language: Python

Observed shape:

Aider ranks tags, then binary-searches how many tags to include so the rendered tree approaches the token budget within an error tolerance.

Why this matters:

The relationship between tag count and rendered tokens is nonlinear. File paths, indentation, context lines, and names vary in length.

Why it matters for Parseltongue:

Parseltongue should render candidate context packages during selection, not assume each symbol costs a fixed amount.

Rust translation:

```rust
pub fn fit_ranked_context_to_budget(
    ranked: &[ContextCandidate],
    budget: TokenBudget,
    renderer: &dyn ContextRenderer,
    estimator: &dyn TokenEstimator,
) -> RenderedContext {
    // Binary search or greedy packing with exact final count.
    todo!()
}
```

When to use:

- Repo map generation.
- Chunk packing.
- Symbol summaries.

When not to use:

- For tiny candidate lists, simple greedy packing is enough.

Risks and caveats:

- Binary search assumes ranking prefix is the only choice; knapsack may do better when item sizes vary.
- Rendered context may change with grouping.

Testing implications:

- Snapshot included item IDs under budget.
- Test near-boundary budgets.

Agent guidance:

Never blindly include "top N" without checking rendered token size.

## Pattern 6 - Summarization Should Preserve Recent Tail

Where found:

- Repository: `Aider-AI__aider`
- File: `aider/history.py`
- Language: Python

Observed shape:

The summarizer tokenizes messages, keeps a recent tail up to roughly half the budget, summarizes the head, reserves a buffer below model max input tokens, and recurses if summary plus tail still exceeds budget.

Why this matters:

Old context can be summarized; recent messages often contain active constraints and edits that should remain verbatim.

Why it matters for Parseltongue:

Long-running agent work on Parseltongue should preserve:

- current task objective,
- current file anchors,
- open diagnostics,
- recent tool results,
- pending verification steps.

Rust translation:

```rust
pub struct ContextMemoryPolicy {
    pub max_history_tokens: usize,
    pub tail_fraction: f32,
    pub reserved_model_buffer: usize,
    pub max_summary_depth: usize,
}
```

When to use:

- Long agent sessions.
- Thread handoffs.
- Background indexing summaries.

When not to use:

- Do not summarize source evidence that must remain exact; store it in files and link spans.

Risks and caveats:

- Summaries can drop constraints.
- Recursive summaries can become vague.

Testing implications:

- Golden-test summarization prompts and retention policy.
- Ensure exact source snippets are not only stored inside summaries.

Agent guidance:

Use summaries for conversation state, not as the sole copy of code evidence.

## Pattern 7 - Tool Surfaces Should Separate Indexing, Search, Analysis, Watch, and Admin

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- File: `src/codegraphcontext/tool_definitions.py`
- Language: Python

Observed tool families:

```text
add_code_to_graph
check_job_status
list_jobs
find_code
analyze_code_relationships
watch_directory
execute_cypher_query
add_package_to_graph
find_dead_code
calculate_cyclomatic_complexity
find_most_complex_functions
list_indexed_repositories
delete_repository
visualize_graph_query
list_watched_paths
unwatch_directory
```

Why this matters:

Agent tools should be small enough to understand and compose, but high-level enough to avoid raw database manipulation for common tasks.

Why it matters for Parseltongue:

Parseltongue should expose tools by intent:

- index repository,
- get job status,
- search code,
- explain symbol,
- find callers/callees,
- build context,
- watch repository,
- list coverage,
- delete index,
- debug query captures.

Rust translation:

```rust
pub enum ParseltongueTool {
    IndexRepository,
    CheckIndexJob,
    SearchCode,
    AnalyzeRelationships,
    BuildContext,
    WatchRepository,
    ListIndexedRepositories,
    DeleteRepositoryIndex,
    DebugTreeSitterQuery,
}
```

When to use:

- MCP server.
- CLI commands.
- Agent integrations.

When not to use:

- Do not expose raw graph write tools to agents by default.

Risks and caveats:

- Tool sprawl increases model confusion.
- Ambiguous tool names cause wrong calls.

Testing implications:

- Schema tests for every tool.
- Tool dry-run tests.
- Permission/safety tests.

Agent guidance:

Prefer task-level tools with structured outputs over asking agents to synthesize database queries.

## Pattern 8 - Graph Query Tools Must Be Read-Only By Default

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- Files: `tool_definitions.py`, `tests/unit/utils/test_cypher_readonly.py`
- Language: Python

Observed shape:

`execute_cypher_query` is described as a read-only Cypher query tool. Tests check that statements such as `CREATE`, `DELETE`, `SET`, `COPY`, `ALTER`, and dangerous `CALL` forms are not read-only, while simple `MATCH ... RETURN` queries are allowed.

Why this matters:

Agents should not mutate code indexes through generic query surfaces unless explicitly authorized.

Why it matters for Parseltongue:

If Parseltongue exposes graph or SQL query tools, default them to read-only and provide purpose-built write/admin tools with clear confirmations.

Rust translation:

```rust
pub enum QueryPermission {
    ReadOnly,
    Admin,
}

pub trait QueryGuard {
    fn validate_query(
        &self,
        query: &str,
        permission: QueryPermission,
    ) -> Result<(), QueryGuardError>;
}
```

When to use:

- Graph query tools.
- Debug query tools.
- User-provided search expressions.

When not to use:

- Do not treat string filters as safe simply because they are "internal."

Risks and caveats:

- Query guard parsing is hard; deny by default.
- Some read-only procedure calls may still access files or network.

Testing implications:

- Positive and negative query safety fixtures.
- Comment and string literal edge cases.
- Semicolon/multiple-statement tests.

Agent guidance:

Use structured relationship tools first. Raw query tools should be last resort and read-only.

## Pattern 9 - Background Jobs Need Status Tools

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- File: `src/codegraphcontext/tool_definitions.py`
- Language: Python

Observed shape:

`add_code_to_graph` returns a job ID for background processing; `check_job_status` and `list_jobs` expose progress.

Why this matters:

Repository indexing can exceed an interactive tool-call timeout. Agents need a way to continue productively while work runs.

Why it matters for Parseltongue:

Parseltongue indexing, embedding, graph building, and full-repo query compilation should be job-based for large repositories.

Rust translation:

```rust
pub struct IndexJobStatus {
    pub job_id: JobId,
    pub state: JobState,
    pub files_total: usize,
    pub files_done: usize,
    pub diagnostics_count: usize,
    pub started_at: SystemTime,
    pub updated_at: SystemTime,
}
```

When to use:

- Full repository indexing.
- Watch bootstrap.
- Embedding generation.
- Multi-language graph rebuilds.

When not to use:

- Small single-file commands can remain synchronous.

Risks and caveats:

- Job IDs and logs must be persisted or recoverable across process restarts.
- Cancelation semantics matter.

Testing implications:

- Job lifecycle tests: queued, running, succeeded, failed, canceled.
- Partial-progress reporting tests.

Agent guidance:

After starting a large index job, poll status and use partial summaries only if the tool reports partial outputs are valid.

## Pattern 10 - Watch Mode Is an Agent Superpower and a Correctness Trap

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- File: `tool_definitions.py`
- Related tests: watcher startup sync and ignore behavior found under tests.

Observed shape:

Tools include `watch_directory`, `list_watched_paths`, and `unwatch_directory`.

Why this matters:

Agents modify files. A live code index can stay current without full reindexing after every edit.

Why it matters for Parseltongue:

Watch mode should update parse trees, symbols, chunks, references, and graph edges incrementally. It must also report when it falls back to full reparse or full reindex.

Rust translation:

```rust
pub struct WatchUpdate {
    pub path: Utf8PathBuf,
    pub event: WatchEventKind,
    pub old_file_version: Option<FileVersionId>,
    pub new_file_version: Option<FileVersionId>,
    pub invalidated_records: Vec<RecordId>,
}
```

When to use:

- Editor integrations.
- Agent sessions with repeated edits.
- Background repo indexes.

When not to use:

- Avoid enabling watch mode before delete/invalidation semantics are tested.

Risks and caveats:

- File events can coalesce or arrive out of order.
- Incremental parse may be wrong if edit metadata is unavailable.
- Generated files can change in bursts.

Testing implications:

- Create/modify/delete/rename tests.
- Full reindex parity tests after a series of watch events.

Agent guidance:

When in doubt, request a full reindex verification after large refactors.

## Pattern 11 - Relationship Tools Should Name Query Type Explicitly

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- File: `tool_definitions.py`
- Language: Python

Observed shape:

`analyze_code_relationships` accepts `query_type` values such as:

```text
find_callers
find_callees
find_all_callers
find_all_callees
find_importers
who_modifies
class_hierarchy
overrides
dead_code
call_chain
module_deps
variable_scope
find_complexity
find_functions_by_argument
find_functions_by_decorator
```

Why this matters:

Relationship analysis is not one generic search problem. Each query has different graph semantics, source evidence, and confidence.

Why it matters for Parseltongue:

Parseltongue should expose relationship queries as typed requests:

- callers/callees,
- importers/imported,
- definitions/references,
- overrides/implements,
- tests for symbol,
- dependencies,
- changed blast radius.

Rust translation:

```rust
pub enum RelationshipQuery {
    FindCallers { target: SymbolSelector },
    FindCallees { target: SymbolSelector },
    FindImporters { module: ModuleSelector },
    CallChain { from: SymbolSelector, to: SymbolSelector, max_depth: usize },
    ClassHierarchy { target: SymbolSelector },
    DeadCode { policy: DeadCodePolicy },
}
```

When to use:

- Agent planning.
- Code review.
- Refactoring.
- Bug investigation.

When not to use:

- Do not ask LLMs to formulate raw graph queries when a typed relationship query exists.

Risks and caveats:

- Relationship outputs must include unresolved/low-confidence warnings.
- Dynamic languages need conservative caveats.

Testing implications:

- One fixture per relationship type.
- Negative tests for absent relationships.

Agent guidance:

Prefer typed relationship tools and inspect returned evidence before editing.

## Pattern 12 - MCP Tools Should Return Structured Evidence, Not Just Prose

Where found:

- Evidence across CodeGraphContext tool schemas, wrale Tree-sitter MCP server helpers, Tabby index schemas, and Aider repo-map records.

Why this matters:

Agents can use structured data for follow-up actions. Prose-only outputs are harder to verify, filter, rerank, or cite.

Why it matters for Parseltongue:

Every tool response should include machine-readable fields:

- file path,
- language,
- symbol ID,
- span,
- snippet or snippet token budget,
- relationship kind,
- confidence,
- diagnostics,
- truncation.

Rust translation:

```rust
pub struct ToolEvidence<T> {
    pub data: T,
    pub provenance: EvidenceProvenance,
    pub diagnostics: Vec<ToolDiagnostic>,
    pub truncation: Option<TruncationInfo>,
}
```

When to use:

- MCP server.
- JSON CLI output.
- Agent integration APIs.

When not to use:

- Human-only terminal commands can render prose, but should be backed by structured records.

Risks and caveats:

- Large structured outputs need pagination.
- Source snippets may leak secrets; redaction policy matters.

Testing implications:

- JSON schema tests.
- Snapshot structured output.
- Pagination/truncation tests.

Agent guidance:

Use structured evidence fields for subsequent actions. Treat prose as commentary, not source of truth.

## Pattern 13 - Context Packages Need Selection Reasons

Where found:

- Aider's repo map ranks by mentioned files, mentioned identifiers, definitions, references, and chat-file personalization.
- Tabby's search supports language/path/source filters.
- CodeGraphContext exposes relationship queries that imply graph-based reasons.

Why this matters:

An LLM should know why a snippet is present:

- direct user-selected file,
- caller of target,
- callee of target,
- import dependency,
- test file,
- mentioned symbol,
- high centrality,
- search match,
- fallback important file.

Why it matters for Parseltongue:

Selection reasons help agents decide what to trust and what to inspect next.

Rust translation:

```rust
pub enum ContextSelectionReason {
    UserAnchoredFile,
    MentionedSymbol,
    DirectCaller,
    DirectCallee,
    ImportDependency,
    TestRelated,
    SearchMatch,
    ImportantFile,
    RecentEdit,
}

pub struct ContextItem {
    pub span: SourceSpan,
    pub text: String,
    pub reasons: Vec<ContextSelectionReason>,
    pub estimated_tokens: usize,
}
```

When to use:

- LLM prompt construction.
- Debugging context quality.
- User-facing "why this context" views.

When not to use:

- Do not let reasons grow into long prose inside the prompt by default; use compact labels.

Risks and caveats:

- Multiple reasons can conflict.
- Reason weights need tuning.

Testing implications:

- Snapshot reasons for fixture context requests.
- Test ranking when reasons compete.

Agent guidance:

When a generated answer relies on context, cite the item reason and source span where possible.

## Pattern 14 - Current File Exclusion Prevents Echo-Search

Where found:

- Repository: `TabbyML__tabby`
- File: `crates/tabby-common/src/index/code/mod.rs`
- Language: Rust

Observed shape:

When filepath is present in code search query, Tabby excludes that file from search results.

Why this matters:

If the current file is already in context, search should often find related files, not return the same file again.

Why it matters for Parseltongue:

Context construction should avoid wasting budget on duplicate content.

Rust translation:

```rust
pub struct RetrievalContext {
    pub anchor_files: BTreeSet<FileId>,
    pub exclude_anchor_files: bool,
}
```

When to use:

- Expanding context around open files.
- Finding tests or callers.

When not to use:

- If the current file is not fully in context, include missing relevant spans.

Risks and caveats:

- Excluding current file can hide local definitions needed for understanding.

Testing implications:

- Test search with and without current-file exclusion.

Agent guidance:

If anchor file snippets are already included, search outward first.

## Pattern 15 - Debug Visualizations Should Link Parse, Query, Graph, and Context

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- Tool: `visualize_graph_query`
- Path: `website/public/wasm` contains web Tree-sitter parser assets.
- Related evidence: editor integrations with Tree-sitter query assets.

Why this matters:

Users and agents need to debug why a symbol, edge, or chunk exists.

Why it matters for Parseltongue:

A great debug view would show:

```text
source
syntax tree
query captures
extracted symbols
references
graph edges
chunks
selected context
token budget
```

Rust translation:

```rust
pub struct ContextDebugBundle {
    pub parse_debug: ParseDebugArtifact,
    pub graph_debug: GraphDebugArtifact,
    pub retrieval_debug: RetrievalDebugArtifact,
    pub rendered_context: RenderedContext,
}
```

When to use:

- Query development.
- Language onboarding.
- Explaining missed context.
- Regression triage.

When not to use:

- Do not require visual debugging for headless CI.

Risks and caveats:

- Debug bundles can be huge.
- Source display may expose secrets.

Testing implications:

- Snapshot compact debug JSON.
- Redaction tests.

Agent guidance:

When extraction behavior is surprising, generate a debug artifact before editing extractors.

## Pattern 16 - Safe Path and Ignore Handling Are Agent-Safety Features

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- Tests: `test_cgcignore_core.py`, path ignore tests, path traversal-related bug reports and tests.

Observed shape:

Tests cover `.cgcignore` parsing, comments/blanks, default patterns, nested discovery, non-root escape behavior, and path ignore fragments. Bug reports include bad-path behavior for delete tools.

Why this matters:

Agent tools often accept paths. Path validation and ignore scoping prevent accidental indexing/deleting/querying outside the intended repository.

Why it matters for Parseltongue:

Parseltongue should keep repository roots, path normalization, and path containment checks central.

Rust translation:

```rust
pub struct RepositoryPath {
    pub root: Utf8PathBuf,
}

impl RepositoryPath {
    pub fn resolve_inside(&self, user_path: &str) -> Result<Utf8PathBuf, PathSafetyError> {
        todo!()
    }
}
```

When to use:

- Tool inputs.
- CLI path arguments.
- Watch paths.
- Delete/index commands.

When not to use:

- Never trust a path string from an agent or user without normalization and containment checks.

Risks and caveats:

- Symlinks complicate containment.
- Case-insensitive filesystems can surprise path comparison.

Testing implications:

- Path traversal tests.
- Symlink tests where feasible.
- Non-repository parent ignore tests.

Agent guidance:

Always pass paths through repository-root validation before indexing or deleting.

## Pattern 17 - LLM Context Should Prefer Evidence Records Over Raw Dumps

Where found:

- Aider repo maps, Tabby chunk schema, Ataraxy source spans, CodeGraphContext graph relationships.

Why this matters:

Raw file dumps waste context and bury the relevant facts. Evidence records let the agent know what each snippet is and why it matters.

Why it matters for Parseltongue:

Rendered context should be compact but inspectable:

```text
File: src/foo.rs
Symbol: parse_repository_files_only
Kind: function
Span: bytes 1200..1880, lines 42..67
Reason: direct callee of requested function
Snippet:
...
```

Rust translation:

```rust
pub struct RenderedContextItem {
    pub header: ContextHeader,
    pub snippet: String,
    pub provenance: EvidenceProvenance,
}
```

When to use:

- Code explanation.
- Refactoring.
- Review.
- Bug investigation.

When not to use:

- If the user explicitly asks for full file content, render full files with token accounting.

Risks and caveats:

- Overly structured context can be verbose.
- Snippet boundaries must remain readable.

Testing implications:

- Snapshot rendered context for readability and stable structure.
- Test snippets do not cut mid-grapheme or invalid UTF-8.

Agent guidance:

Build context as compact evidence cards, not undifferentiated source blobs.

## Parseltongue Agent Tool Recommendation

Suggested tool set:

```text
index_repository
check_index_job
list_indexed_repositories
build_context
search_code
explain_symbol
find_relationships
debug_query_captures
watch_repository
delete_repository_index
```

Suggested `build_context` output:

```rust
pub struct BuildContextResponse {
    pub rendered_markdown: String,
    pub items: Vec<ContextItem>,
    pub budget_report: ContextBudgetReport,
    pub diagnostics: Vec<ContextDiagnostic>,
    pub coverage: IndexCoverageSummary,
}
```

Required invariants:

- Every context item has source provenance.
- Every context response has token accounting.
- Every truncated response says what was omitted.
- Every graph answer includes confidence and unresolved cases.
- Every raw query interface is read-only by default.
- Every indexing command reports workspace/root/config identity.

## Anti-Patterns Captured

- Raw file dumping without ranking or provenance.
- Fixed repo-map size across all model windows.
- Context packages with no token report.
- Stale repo maps after file edits.
- Graph query tools that can mutate by default.
- Prose-only tool responses.
- Relationship answers with no source evidence.
- Path arguments without containment checks.
- Watch mode without invalidation tests.
- Summary compaction as the only copy of important source evidence.

## Transferable Design Principle

An LLM companion should not merely parse code; it should route evidence. Parseltongue's agent-facing layer should turn Tree-sitter records into ranked, budgeted, source-spanned context packages and safe tools that help agents ask better follow-up questions before editing code.
