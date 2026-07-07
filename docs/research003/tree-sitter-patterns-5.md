# Tree-Sitter Patterns 5: Testing, Benchmarks, CI, Debugging, Anti-Patterns

This file captures the hardening layer for Parseltongue: test strategy, fixture design, incremental parsing verification, graph golden tests, performance budgets, CI gates, debugging artifacts, and anti-patterns.

## Phase 0 - Deconstruct and Clarify

Core objective for this slice: extract patterns that help a Rust Tree-sitter code-intelligence system prove its behavior and resist regressions across languages, grammars, parser APIs, file systems, caches, and agent workflows.

Premise is sound. Proceeding with optimized protocol.

Important evidence boundary: this file includes local evidence from tests, docs, bug reports, and source scans. Some recommendations translate observed practices into a proposed Parseltongue verification architecture; those are recommendations, not claims that the current repository already implements them.

## Phase 1 - Cognitive Staging

Expert council used for this slice:

- Verification Architect: designs unit, integration, golden, corpus, fuzz, and benchmark layers.
- Tree-sitter Binding Engineer: studies parser/query/cursor/incremental tests and runtime footguns.
- Graph QA Engineer: focuses on node/edge parity, delete tests, config isolation, and cross-language fixture matrices.
- Performance Engineer: focuses on file-size limits, timeouts, concurrency, batching, cache size, and benchmark realism.
- Skeptical Systems Engineer: challenges circular goldens, brittle snapshots, hidden config, stale caches, and misleading pass rates.

Knowledge scaffolding:

- Tree-sitter grammar testing: `test/corpus/*.txt`, expected S-expressions, targeted corpus includes.
- Fuzz testing: random edits, incremental parse versus full reparse, replayable failing snapshots.
- Binding tests: parser creation, callbacks, cancellation, timeouts, included ranges, query cursor, tree cursor, input edits.
- Code-intelligence goldens: nodes, edges, metadata, fixtures, cross-language matrices.
- CI gates: compile, query compile, fixture parse, golden diff, performance budget, delete cleanup, config isolation.

## Phase 2 - Multi-Perspective Synthesis

Conventional approach: add unit tests for extractors and run them in CI.

Alternative 1 - Compiler construction blend: every extractor change is a language semantics change. Treat fixtures and expected graphs like compiler test suites, with explicit positive and negative examples.

Alternative 2 - Forensics blend: every bug report becomes a preserved crime scene. Save the source, expected captures, actual captures, graph diff, and reproduction command.

Alternative 3 - Scientific instrumentation blend: Parseltongue needs experiments with controls. Compare incremental parse to full parse, query change to previous capture distribution, cache hit to cold run, and graph output to source-truth fixtures.

Selected path: compiler-test rigor plus forensic preservation. Parseltongue should keep small focused tests for runtime units, multi-language golden fixtures for semantic output, and replay artifacts for hard parser/index regressions.

Council debate summary:

- Verification Architect: create a layered pyramid: query compile tests, fixture capture tests, extractor unit tests, full-index integration tests, golden graph tests, fuzz and benchmarks.
- Skeptical Systems Engineer: goldens can become circular; passing against generated snapshots is not proof of correctness.
- Graph QA Engineer response: separate source-truth expected cases from generated graph snapshots; classify diffs by node and edge kind.
- Performance Engineer response: add limits and budgets to tests, not just correctness assertions.
- Tree-sitter Binding Engineer response: incremental parsing must be verified against full reparse and replayable fuzz snapshots.

Core thesis: Parseltongue should harden Tree-sitter intelligence with layered, source-grounded verification: every query compiles, every capture has fixtures, every extractor has semantic expectations, every graph edge has evidence, every cache invalidates under version changes, every performance claim has a benchmark, and every known failure becomes a regression test.

## Phase 3 - Verification Anchors

Primary local evidence used:

- `viktorstrate__swift-tree-sitter/Tests/SwiftTreeSitterTests/ParserTests.swift`
- `viktorstrate__swift-tree-sitter/Tests/SwiftTreeSitterTests/QueryCursorTests.swift`
- `viktorstrate__swift-tree-sitter/Tests/SwiftTreeSitterTests/TreeCursorTests.swift`
- `viktorstrate__swift-tree-sitter/Sources/SwiftTreeSitter/STSParser.swift`
- `viktorstrate__swift-tree-sitter/Sources/SwiftTreeSitter/STSTree.swift`
- `viktorstrate__swift-tree-sitter/Sources/SwiftTreeSitter/STSInputEdit.swift`
- `Idorobots__tree-sitter-org/AGENTS.md`
- `Idorobots__tree-sitter-org/fuzz.py`
- `wharflab__tree-sitter-batch/CLAUDE.md`
- `CodeGraphContext__CodeGraphContext/tests/fixtures/goldens/...`
- `CodeGraphContext__CodeGraphContext/CGC_E2E_BUG_REPORT.md`
- `CodeGraphContext__CodeGraphContext/CGC_GRAPH_INCONSISTENCIES.md`
- `CodeGraphContext__CodeGraphContext/tests/unit/...`
- `chunkhound__chunkhound/site/src/pages/docs/configuration.md`
- `Artemarius__Engram/CLAUDE.md` and `README.md` benchmark references

Self-correction questions:

- Does local evidence show parser binding tests for cancellation, timeout, included ranges, query cursor, and tree cursor? Yes, the Swift binding tests and sources cover these areas.
- Does local evidence show Tree-sitter corpus/fuzz workflow? Yes, `Idorobots__tree-sitter-org/AGENTS.md` documents `tree-sitter test`, targeted corpus runs, `tree-sitter fuzz`, and replay through `fuzz.py`.
- Does local evidence show multi-language golden graph fixtures? Yes, CodeGraphContext has `tests/fixtures/goldens/sample_project_*` with nodes, edges, and metadata across many languages.
- Does local evidence show performance limits in config? Yes, ChunkHound docs list max file size, per-file timeout, concurrency, batch size, DB batch size, and embedded SQL detection.
- Does local evidence show benchmark harnesses for chunk/search systems? Yes, Engram references chunking, embedding, and query latency benchmarks.

## Pattern 1 - Query Compilation Tests Are the First Gate

Where found:

- Broad `.scm` corpus across 6267 query files.
- Aider and wrale both compile Tree-sitter queries dynamically.
- Editor integrations bundle many query families.

Why this matters:

If a query does not compile, no downstream symbol or context logic matters.

Why it matters for Parseltongue:

Every bundled query asset should compile against the exact grammar runtime used by the extractor.

Rust translation:

```rust
#[test]
fn compile_all_bundled_queries() {
    for asset in QueryAssetRegistry::all() {
        let language = LanguageRegistry::test().language(asset.language).unwrap();
        tree_sitter::Query::new(&language.tree_sitter_language, asset.source)
            .unwrap_or_else(|err| panic!("query failed: {:?}: {}", asset.id, err));
    }
}
```

When to use:

- Every CI run.
- Every grammar crate bump.
- Every query asset change.

When not to use:

- Query compilation alone does not prove captures are correct.

Risks and caveats:

- A query can compile and still capture nothing.
- Wrong capture names can compile.

Testing implications:

- Pair query compile tests with fixture capture tests and capture-name linting.

Agent guidance:

When generating or editing `.scm`, add or update fixture capture tests in the same change.

## Pattern 2 - Capture Contract Tests Prevent Semantic Drift

Where found:

- Aider's tag capture model.
- Tomatio repo-map query assets.
- CodeGraphContext graph golden diffs showing drift after extractor changes.

Why this matters:

Capture names are the contract between language-specific queries and language-neutral code intelligence.

Why it matters for Parseltongue:

Parseltongue needs tests that assert:

- allowed capture names,
- expected capture roles,
- expected node kinds,
- expected byte ranges,
- expected source text.

Rust translation:

```rust
pub struct ExpectedCapture {
    pub capture_role: CaptureRole,
    pub node_kind: &'static str,
    pub source_text: &'static str,
    pub start_line: usize,
}
```

When to use:

- Every language query.
- Every new capture role.

When not to use:

- Do not snapshot entire parse trees for every query if focused capture snapshots are clearer.

Risks and caveats:

- Source formatting changes can churn snapshots.
- Byte ranges depend on fixture text exactly.

Testing implications:

- Use minimal fixtures for each semantic role.
- Include negative examples where similar syntax should not capture.

Agent guidance:

Future agents should preserve focused fixture text and expected captures in one place.

## Pattern 3 - Tree-Sitter Corpus Tests Are the Grammar-Level Safety Net

Where found:

- Repository: `Idorobots__tree-sitter-org`
- File: `AGENTS.md`
- Repository: `wharflab__tree-sitter-batch`
- File: `CLAUDE.md`

Observed shape:

Local instructions mention:

```text
tree-sitter test
tree-sitter test --file-name test/corpus/headings.txt
tree-sitter test --file-name test/corpus/headings.txt --include "Level 1 heading"
```

and the corpus format under `test/corpus/*.txt` with delimited sections.

Why this matters:

Grammar-level tests validate parse-tree shape before code-intelligence extraction begins.

Why it matters for Parseltongue:

Parseltongue may not own every grammar, but when it vendors or develops a grammar/query adapter, it should run grammar corpus tests where available.

Rust translation:

```text
Verification command policy:
1. Run narrow corpus test for changed grammar area.
2. Run full grammar corpus.
3. Run Parseltongue capture/extractor tests.
4. Run full multi-language index fixture.
```

When to use:

- Grammar changes.
- External scanner changes.
- Grammar version upgrades.

When not to use:

- If using third-party published parser crates only, Parseltongue may not run upstream corpus tests in normal CI, but should still keep fixture capture tests.

Risks and caveats:

- Corpus tests assert parse-tree shape, not semantic extraction.
- Updating expected S-expressions can hide regressions.

Testing implications:

- Add failing corpus test before grammar fix when owning grammar.
- Keep corpus changes small and reviewable.

Agent guidance:

When grammar shape changes, update grammar corpus first, then update extraction fixtures.

## Pattern 4 - Fuzz Replay Validates Incremental Parsing

Where found:

- Repository: `Idorobots__tree-sitter-org`
- Files: `AGENTS.md`, `fuzz.py`
- Language: Python plus Tree-sitter CLI

Observed shape:

Instructions mention:

```text
tree-sitter fuzz --iterations 10 --edits 3
tree-sitter fuzz --log-graphs --iterations 50 --edits 5 --include "heading"
python3 fuzz.py < fuzz_input.log
```

`fuzz.py` extracts snapshots, computes contiguous replacement edits, replays edits incrementally, and compares against full parses.

Why this matters:

Incremental parsing correctness cannot be assumed. Fuzzing finds edit sequences humans do not write by hand.

Why it matters for Parseltongue:

Watch mode and editor integrations should not trust incremental parse output until it matches full reparse output across random and captured edits.

Rust translation:

```rust
pub struct IncrementalReplayCase {
    pub language: LanguageId,
    pub snapshots: Vec<Vec<u8>>,
}

pub fn verify_incremental_equals_full_reparse(case: IncrementalReplayCase) {
    // For each edit: edit old tree, incremental parse, full parse, compare selected invariants.
}
```

When to use:

- Incremental parsing implementation.
- External scanner changes.
- Watch mode changes.
- Parser timeout/cancellation changes.

When not to use:

- Fuzzing should complement, not replace, deterministic fixtures.

Risks and caveats:

- Full tree equality can be too strict across error recovery; define comparison policy.
- Fuzz failures need replay artifacts.

Testing implications:

- Save failing snapshots.
- Add reduced deterministic regression tests.
- Compare not just root sexp but also captures and symbol records.

Agent guidance:

If incremental parse behavior changes, generate replay tests before trusting performance improvements.

## Pattern 5 - Binding-Level Tests Expose Runtime Footguns

Where found:

- Repository: `viktorstrate__swift-tree-sitter`
- Files: `ParserTests.swift`, `QueryCursorTests.swift`, `TreeCursorTests.swift`
- Language: Swift

Observed test areas:

- parser language assignment,
- parsing strings,
- parsing through callbacks,
- cancellation flag,
- parser reset,
- timeout micros,
- included ranges,
- query cursor captures/matches,
- tree cursor navigation.

Why this matters:

Tree-sitter bindings have runtime settings that can persist between parses and cause subtle bugs.

Why it matters for Parseltongue:

Rust wrappers should test parser state transitions even if the raw crate is trusted.

Rust translation:

```rust
#[test]
fn parser_pool_resets_timeout_and_included_ranges() {
    // Acquire parser, set non-default settings, return to pool.
    // Reacquire parser and assert defaults.
}
```

When to use:

- Parser factory.
- Parser pool.
- Query cursor wrapper.
- Included ranges.
- Cancellation/timeouts.

When not to use:

- Do not wrap every Tree-sitter API with a custom abstraction unless the wrapper adds policy or safety.

Risks and caveats:

- Parser settings can persist.
- Included ranges can silently limit parse scope.
- Cancellation can leave parser state that requires reset.

Testing implications:

- Test parser reuse across files.
- Test parser reuse across languages.
- Test cancellation followed by normal parse.

Agent guidance:

When adding parser pooling, add reset tests first.

## Pattern 6 - Changed Ranges Need Full-Reparse Parity Tests

Where found:

- Repository: `viktorstrate__swift-tree-sitter`
- Files: `STSTree.swift`, `STSInputEdit.swift`
- Repository: `wrale__mcp-server-tree-sitter`
- File: `utils/tree_sitter_helpers.py`

Observed shape:

Swift binding exposes tree edit and `changedRanges(oldTree:newTree:)`. Wrale's Python helper notes a simplified changed-range implementation where binding support may be missing.

Why this matters:

Changed ranges are often used to invalidate caches. If ranges are wrong or too broad/narrow, derived records become stale.

Why it matters for Parseltongue:

Cache invalidation for symbols, chunks, embeddings, and graph edges should be validated against full reindex output.

Rust translation:

```rust
#[test]
fn changed_ranges_invalidate_equivalent_to_full_reindex() {
    // Edit fixture, incremental update derived records, full reindex fixture,
    // compare final symbols/chunks/references.
}
```

When to use:

- Watch mode.
- Incremental chunk updates.
- Incremental graph updates.

When not to use:

- Do not rely on changed ranges for first release if full-file invalidation is simpler and correct.

Risks and caveats:

- Parent symbols may need invalidation even when only child range changed.
- Imports can affect other files.

Testing implications:

- Edits inside function body.
- Edits to signature.
- Edits to imports.
- Edits that rename a symbol.
- Edits that move a symbol.

Agent guidance:

Prefer conservative invalidation until parity tests prove narrower invalidation is correct.

## Pattern 7 - Multi-Language Golden Fixtures Catch Cross-Language Blind Spots

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- Path: `tests/fixtures/goldens/sample_project_*`
- Languages observed in fixture names: Python, C, C++, C#, Dart, Elixir, Go, Haskell, Java, JavaScript, Kotlin, Lua, Perl, PHP, Ruby, Rust, Scala, Swift, TypeScript, and misc.

Observed shape:

Each golden fixture includes files such as:

```text
nodes.jsonl
edges.jsonl
metadata.json
nodes_have.jsonl
edges_have.jsonl
```

Why this matters:

Cross-language systems regress unevenly. A change that improves Rust extraction can break Swift, Lua, or C#.

Why it matters for Parseltongue:

Parseltongue needs a multi-language fixture suite even if Rust is the implementation language.

Rust translation:

```rust
pub struct GoldenFixture {
    pub language: LanguageId,
    pub source_root: Utf8PathBuf,
    pub expected_symbols: Vec<ExpectedSymbol>,
    pub expected_edges: Vec<ExpectedEdge>,
}
```

When to use:

- Extractor changes.
- Query changes.
- Graph resolver changes.
- Parser crate upgrades.

When not to use:

- Avoid one giant golden that obscures which feature failed.

Risks and caveats:

- Goldens can encode incorrect behavior.
- Generated goldens need source-truth review.

Testing implications:

- Keep small hand-authored "must have" expectations.
- Keep generated full snapshots separately.
- Diff by node/edge kind and language.

Agent guidance:

When a golden changes, explain the semantic reason and update source-truth expectations, not just snapshots.

## Pattern 8 - Avoid Circular Golden Validation

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- File: `CGC_GRAPH_INCONSISTENCIES.md`

Observed issue:

The report warns about a "circular perfection gate" where export is compared against itself or a misleading perfection report conflicts with audit results.

Why this matters:

A generated graph matching a generated graph is not proof that the graph matches source truth.

Why it matters for Parseltongue:

Parseltongue should distinguish:

- source-truth expectations written by humans,
- generated snapshots for regression detection,
- broad statistical tolerances,
- semantic audit reports.

Rust translation:

```rust
pub enum GoldenExpectationKind {
    HumanAuthoredMustHave,
    GeneratedSnapshot,
    StatisticalSummary,
}
```

When to use:

- Golden infrastructure.
- CI reporting.

When not to use:

- Do not claim "100 percent correct" based on generated snapshot parity.

Risks and caveats:

- Human-authored expectations are slower to maintain.
- Generated snapshots are still useful for drift detection.

Testing implications:

- Report which expectation kind failed.
- Require review for human-authored expected changes.

Agent guidance:

If a test is snapshot-only, say it protects against drift, not that it proves semantic truth.

## Pattern 9 - Graph Drift Reports Should Classify Failures

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- Files: `CGC_E2E_BUG_REPORT.md`, `CGC_GRAPH_INCONSISTENCIES.md`

Observed issue classes:

- missing CALLS edges,
- missing enum members,
- missing partial-class edges,
- missing implements/protocol/companion relationships,
- node-count drift from extra Parameter/Variable nodes,
- published package behavior diverging from local editable behavior.

Why this matters:

The fix for "missing enum member" differs from "node-count inflation" or "package release drift."

Why it matters for Parseltongue:

Failure reports should identify:

- language,
- fixture,
- expected kind,
- actual kind,
- node/edge category,
- parser/query/extractor version,
- example source span.

Rust translation:

```rust
pub struct GraphFailure {
    pub language: LanguageId,
    pub fixture: String,
    pub category: GraphFailureCategory,
    pub expected: String,
    pub actual: String,
    pub source_span: Option<SourceSpan>,
}
```

When to use:

- CI golden reports.
- Regression triage.
- Release notes.

When not to use:

- Do not collapse all mismatches into a single count.

Risks and caveats:

- Detailed diffs can be large.
- Classification logic itself needs tests.

Testing implications:

- Snapshot diff reports for known failures.

Agent guidance:

When fixing graph failures, select the smallest category and add a regression fixture for it.

## Pattern 10 - Delete and Cleanup Tests Are Required for Every Derived Record

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- File: `CGC_E2E_BUG_REPORT.md`
- Related local tests for delete commands and graph transactions.

Observed issue:

Delete left orphan nodes after removing a repository.

Why this matters:

Indexers generate many derived records: files, symbols, references, locals, parameters, chunks, embeddings, graph edges, diagnostics. Every derived type must be deletable.

Why it matters for Parseltongue:

Agents will reindex, switch branches, delete worktrees, and rewrite files. Stale derived data creates false context.

Rust translation:

```rust
#[test]
fn repository_delete_removes_all_derived_records() {
    // Index fixture.
    // Delete repository.
    // Assert zero records by repository id across every table/index.
}
```

When to use:

- New derived record type.
- Store schema changes.
- Cleanup commands.

When not to use:

- Never rely on manual DB cleanup as normal operation.

Risks and caveats:

- Records without repository ID are hard to delete.
- Graph and search indexes can diverge.

Testing implications:

- Index/delete/reindex cycles.
- Branch switch simulations.
- Orphan count reports.

Agent guidance:

Adding a table or graph node type requires adding delete coverage.

## Pattern 11 - Config Isolation Tests Prevent False Results

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- File: `CGC_E2E_BUG_REPORT.md`
- Tests: config and ignore-related unit tests.

Observed issue:

Repo-local config could override isolated global config and direct queries to the wrong database.

Why this matters:

False context from the wrong repository or database can be worse than no context.

Why it matters for Parseltongue:

Parseltongue should test:

- isolated home directory,
- workspace-local config,
- explicit config path,
- current working directory inside a repo,
- global vs per-repo storage namespaces.

Rust translation:

```rust
#[test]
fn local_config_does_not_override_explicit_global_context() {
    // Create temp home and repo-local config.
    // Run config resolution.
    // Assert configured precedence.
}
```

When to use:

- Config manager.
- CLI commands.
- MCP server startup.

When not to use:

- Do not let tests run against developer machine global config.

Risks and caveats:

- Environment variables can also bleed into tests.

Testing implications:

- Use temp directories.
- Clear relevant environment variables.
- Assert config source in output.

Agent guidance:

When debugging odd index results, inspect context/config source before parser code.

## Pattern 12 - Read-Only Query Guards Need Negative Tests

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- File: `tests/unit/utils/test_cypher_readonly.py`

Observed tested cases:

- Accept simple `MATCH ... RETURN`.
- Reject `CREATE`, `DELETE`, `SET`, `COPY`, `ALTER`.
- Reject dangerous `CALL` forms.
- Reject multiple-statement attempts after semicolon.
- Allow comments containing forbidden words when the actual query is read-only.

Why this matters:

Agents will use query tools. Query guards need adversarial tests, not just happy paths.

Why it matters for Parseltongue:

Any raw graph/SQL/debug query interface must be denied-by-default with tested exceptions.

Rust translation:

```rust
#[test]
fn graph_query_guard_rejects_mutations_and_procedures() {
    for query in mutation_queries() {
        assert!(guard.validate_read_only(query).is_err());
    }
}
```

When to use:

- Raw graph query tool.
- Debug SQL tool.
- User-defined filters.

When not to use:

- Prefer typed query APIs for common operations.

Risks and caveats:

- Regex-only guards can be bypassed.
- Database-specific procedures can mutate state.

Testing implications:

- Add negative tests for every newly supported backend.

Agent guidance:

Default to typed tools; raw queries should be read-only and guarded.

## Pattern 13 - Per-File Timeout and File-Size Budgets Are Product Requirements

Where found:

- Repository: `chunkhound__chunkhound`
- File: `site/src/pages/docs/configuration.md`

Observed config:

```text
max_concurrent
cleanup
max_file_size_mb
per_file_timeout_seconds
batch_size
db_batch_size
detect_embedded_sql
per_file_timeout_min_size_kb
```

Why this matters:

Indexing unbounded files can hang agents and CI. Parser performance must be controlled at product boundaries.

Why it matters for Parseltongue:

Parseltongue should include:

- max file size,
- parse timeout,
- timeout only above size threshold,
- concurrency limit,
- batch sizes,
- cleanup toggle,
- embedded-language detection toggle.

Rust translation:

```rust
pub struct IndexPerformancePolicy {
    pub max_concurrent_parsers: usize,
    pub max_file_size_bytes: u64,
    pub per_file_timeout: Option<Duration>,
    pub timeout_min_size_bytes: u64,
    pub parse_batch_size: usize,
    pub write_batch_size: usize,
}
```

When to use:

- Full repository indexing.
- CI.
- MCP background jobs.

When not to use:

- Do not silently skip large files without reporting them.

Risks and caveats:

- Timeouts can produce partial indexes.
- Too-low file size limits miss generated but relevant code.

Testing implications:

- Large file skip tests.
- Timeout simulation tests.
- Concurrency determinism tests.

Agent guidance:

If important files are skipped for size/time, surface that before answering architecture questions.

## Pattern 14 - Benchmarks Should Match the Pipeline, Not Just Parser Speed

Where found:

- Repository: `Artemarius__Engram`
- Files: `CLAUDE.md`, `README.md`
- Evidence: benchmark executable for chunking, embedding, and query latency.
- Repository: `Ataraxy-Labs__sem`
- File: `CHANGELOG.md`
- Evidence: cache-size and warm-load measurements for a large entity corpus.

Why this matters:

Parser speed is only one part of the user experience. Chunking, embedding, indexing, graph resolution, cache load, and retrieval all matter.

Why it matters for Parseltongue:

Benchmark stages separately:

- file discovery,
- parsing,
- query capture,
- symbol extraction,
- reference resolution,
- chunk construction,
- storage write,
- search,
- context rendering.

Rust translation:

```rust
pub struct IndexBenchmarkResult {
    pub files_per_second: f64,
    pub parse_ms_p50: f64,
    pub parse_ms_p95: f64,
    pub extraction_ms_p95: f64,
    pub index_write_ms_p95: f64,
    pub context_render_ms_p95: f64,
    pub peak_memory_bytes: u64,
}
```

When to use:

- Performance claims.
- Parser pool changes.
- Cache changes.
- Query changes.

When not to use:

- Do not generalize one repository benchmark to all workloads.

Risks and caveats:

- Benchmarks can be noisy.
- Warm cache and cold cache must be separated.

Testing implications:

- Keep smoke benchmarks in CI if cheap.
- Keep full benchmarks manual or scheduled.
- Record fixture size and machine context.

Agent guidance:

When claiming speed or memory improvement, point to a benchmark command and result artifact.

## Pattern 15 - Parser Debug Graphs and Visual Artifacts Help Triage Hard Bugs

Where found:

- Repository: `viktorstrate__swift-tree-sitter`
- File: `STSParser.swift`
- Evidence: parser debug graph printing through `ts_parser_print_dot_graphs`.
- Repository: `CodeGraphContext__CodeGraphContext`
- Tool: graph visualization.

Why this matters:

Hard parser bugs need visual evidence: tree shape, parser transitions, query captures, graph edges.

Why it matters for Parseltongue:

Parseltongue should be able to emit debug bundles:

- parse tree S-expression,
- query captures,
- symbol records,
- reference records,
- graph diff,
- context selection reasons.

Rust translation:

```rust
pub struct DebugBundle {
    pub source_fixture: Utf8PathBuf,
    pub tree_sexp: Option<String>,
    pub captures_json: serde_json::Value,
    pub symbols_json: serde_json::Value,
    pub graph_diff_json: Option<serde_json::Value>,
}
```

When to use:

- Query failures.
- Grammar upgrades.
- Unknown graph drift.
- Context-selection surprises.

When not to use:

- Do not emit large debug artifacts in normal CLI output.

Risks and caveats:

- Debug artifacts may include source secrets.
- DOT graphs can be huge.

Testing implications:

- Redaction policy tests.
- Snapshot compact debug output.

Agent guidance:

When a parser/extractor bug is unclear, produce a debug bundle before modifying logic.

## Pattern 16 - Missing and Error Nodes Are Indexing Signals

Where found:

- Repository: `viktorstrate__swift-tree-sitter`
- File: `STSNode.swift`
- Evidence from search: node API exposes missing-node checks.
- Tree-sitter corpus and grammar workflows generally care about `ERROR` and `MISSING` nodes.
- `wharflab__tree-sitter-batch/CLAUDE.md` references examples checked for zero `ERROR`/`MISSING` nodes.

Why this matters:

Tree-sitter can produce a tree even for invalid code. Missing/error nodes are useful but should influence confidence.

Why it matters for Parseltongue:

Agent workflows often inspect broken code. Parseltongue should index partially parseable files but mark confidence and diagnostics.

Rust translation:

```rust
pub struct ParseQuality {
    pub has_error: bool,
    pub has_missing: bool,
    pub error_node_count: usize,
    pub missing_node_count: usize,
}
```

When to use:

- Parse reports.
- Symbol confidence.
- Context rendering.
- Watch mode during edits.

When not to use:

- Do not discard all parse output solely because `ERROR` exists unless policy requires it.

Risks and caveats:

- Error nodes can hide important constructs.
- Generated partial code may be valid enough for context.

Testing implications:

- Syntax-error fixtures.
- Partial edit fixtures.
- Confidence downgrade tests.

Agent guidance:

Tell the LLM when context came from a file with parse errors.

## Pattern 17 - Security and Sanitization Belong in Code-Intelligence Stores

Where found:

- Repository: `cmillstead__codesight-mcp`
- Files found by scan: `src/codesight_mcp/security.py`, `src/codesight_mcp/storage/index_store.py`
- Evidence from search: sanitization of signatures, list fields, hashes, summary injection phrases, and field validation.

Why this matters:

Code indexes can store untrusted repository text. Tool outputs can become prompts. Prompt-injection and malformed data concerns are real for agentic code assistants.

Why it matters for Parseltongue:

Parseltongue should treat indexed comments, docstrings, strings, and summaries as untrusted data.

Rust translation:

```rust
pub struct SanitizedContextText {
    pub text: String,
    pub redactions: Vec<Redaction>,
    pub trust_level: TrustLevel,
}
```

When to use:

- Rendering comments/docstrings.
- Including README or issue text.
- Storing LLM-generated summaries.

When not to use:

- Do not sanitize by destroying source evidence. Preserve raw source in the store but render safely.

Risks and caveats:

- Over-sanitization can remove useful code.
- Under-sanitization can expose secrets or instructions.

Testing implications:

- Prompt-injection fixture comments.
- Secret-looking strings.
- Malformed JSON/schema inputs.

Agent guidance:

Mark repository text as data, not instructions, in rendered context.

## Pattern 18 - Narrow Tests First, Then Broaden

Where found:

- Repository: `Idorobots__tree-sitter-org`
- File: `AGENTS.md`

Observed instruction:

Run the narrowest relevant test first, then broaden to corpus file, then full corpus.

Why this matters:

Parser and indexer test suites can be expensive. Narrow red/green loops are faster and easier to diagnose.

Why it matters for Parseltongue:

Use a verification ladder:

```text
one query fixture
-> one extractor unit
-> one language fixture
-> one multi-language fixture
-> full repository scan
-> benchmark
```

Rust translation:

```text
cargo test query_contract_rust_function
cargo test extractor_rust
cargo test golden_rust_fixture
cargo test golden_all_languages
cargo bench index_pipeline
```

When to use:

- Development loop.
- Agent modifications.
- CI sharding.

When not to use:

- Do not skip the broad tests before release.

Risks and caveats:

- Narrow tests can create local optimum fixes.

Testing implications:

- Document the ladder in contributor instructions.

Agent guidance:

Start with the narrow failing fixture, then run the wider verification set before declaring completion.

## Pattern 19 - Release Parity Must Test Installed Artifacts

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- File: `CGC_E2E_BUG_REPORT.md`

Observed issue:

The bug report distinguishes PyPI package behavior from local editable behavior; some issues were fixed locally but not released.

Why this matters:

Users run installed packages, not source trees. Generated assets, package data, and dependency versions can differ.

Why it matters for Parseltongue:

Parseltongue should test:

- local workspace,
- packaged crate,
- CLI installed from package,
- bundled query assets present,
- grammar assets present,
- generated files included.

Rust translation:

```text
cargo test
cargo package --allow-dirty
cargo install --path .
parseltongue --version
parseltongue index tests/fixtures/sample_project_rust
```

When to use:

- Release preparation.
- Query asset packaging changes.
- Build script changes.

When not to use:

- Daily local loops can skip package-install tests until release gates.

Risks and caveats:

- Packaged assets are often missed by normal unit tests.

Testing implications:

- CI release job should install the packaged artifact and run smoke indexing.

Agent guidance:

If a feature depends on bundled assets, verify packaged artifact behavior.

## Pattern 20 - Known Footgun Registry

This section consolidates anti-patterns observed across the evidence set.

### Footgun: Query compiles but captures wrong semantic role

Countermeasure:

- Capture contract tests.
- Typed capture role linter.

### Footgun: Incremental parse used without exact edit validation

Countermeasure:

- Full reparse parity tests.
- Fuzz replay snapshots.

### Footgun: Parser reused after timeout/cancellation without reset

Countermeasure:

- Parser pool reset tests.
- Cancellation followed by normal parse tests.

### Footgun: File skipped silently because of size, ignore, or unsupported language

Countermeasure:

- File parse reports.
- Coverage summaries.

### Footgun: Graph delete leaves orphan derived records

Countermeasure:

- Delete all records by repository ID.
- Orphan count tests.

### Footgun: Golden snapshot validates itself

Countermeasure:

- Separate human-authored must-have expectations from generated snapshots.

### Footgun: Config from another repo changes current indexing target

Countermeasure:

- Config source reporting.
- Isolated HOME tests.
- Workspace-root containment checks.

### Footgun: Token context hides truncation

Countermeasure:

- Context budget reports.
- Explicit omitted-item summaries.

### Footgun: Language-specific feature forced into false normalized category

Countermeasure:

- Store normalized kind, language-specific kind, original node kind, and caveats.

### Footgun: Raw query tool mutates index

Countermeasure:

- Read-only guard.
- Typed relationship tools.

## Parseltongue Verification Matrix

Suggested gates:

```text
1. Query compile tests
2. Capture-name lint tests
3. Fixture capture snapshots
4. Extractor unit tests
5. Source-span roundtrip tests
6. Parse quality tests with ERROR/MISSING nodes
7. Incremental parse full-reparse parity tests
8. File discovery and ignore-policy tests
9. Multi-language golden symbol tests
10. Multi-language golden edge tests
11. Cache invalidation tests
12. Delete/orphan cleanup tests
13. Config isolation tests
14. Tool schema and query guard tests
15. Context budget and truncation tests
16. Benchmarks for parse, extraction, storage, retrieval, rendering
17. Packaged artifact smoke tests
```

## Transferable Design Principle

The strongest Tree-sitter systems do not trust a parser, a query, a cache, or a graph edge merely because it exists. They test every boundary where meaning can drift: grammar shape, query captures, source spans, incremental edits, symbol identity, relationship resolution, storage cleanup, token rendering, and packaged release behavior. Parseltongue should make those boundaries visible, versioned, and testable.
