# Tree-Sitter Patterns 3: Repository Scanning, Indexing, Symbols, Chunks, Graphs

This file focuses on how parsed syntax becomes repository-scale code intelligence: file walking, ignore handling, language detection, symbol records, chunks, graph nodes, graph edges, cache layers, and retrieval indexes.

## Phase 0 - Deconstruct and Clarify

Core objective for this slice: extract patterns from code-indexing and repository-analysis systems that can help Parseltongue turn Tree-sitter parse output into durable, queryable, agent-useful knowledge.

Premise is sound. Proceeding with optimized protocol.

Important evidence boundary: this slice includes patterns from direct source inspection and broad repository scans. It does not claim every one of the 609 repositories was manually audited file by file.

## Phase 1 - Cognitive Staging

Expert council used for this slice:

- Repository Indexing Architect: focuses on file discovery, ignore behavior, batching, caches, and persistence.
- Graph Semantics Engineer: focuses on entities, references, calls, imports, ownership, and edge correctness.
- Search/Retrieval Engineer: focuses on chunk schemas, filters, overfetching, and token-aware retrieval.
- Rust Storage Architect: translates Python and service patterns into Rust traits, structs, and database boundaries.
- Skeptical Systems Engineer: challenges stale caches, config bleed, orphan nodes, graph drift, over-indexing, and false semantic edges.

Knowledge scaffolding:

- Repository walking: `.gitignore`, custom excludes, hidden files, generated files, max file size, concurrency.
- Parse orchestration: language detection, parser creation, parse reports, skip diagnostics.
- Index models: file, document, chunk, symbol, reference, scope, edge, source span, content hash.
- Graph correctness: ownership, nested scopes, imports, call resolution, unresolved references, confidence.
- Retrieval: lexical tokens, structural filters, document/chunk schema, result overfetch, reranking, context budgets.

## Phase 2 - Multi-Perspective Synthesis

Conventional approach: scan source files, parse supported extensions, extract functions/classes/imports, store them in a database, and search with text or embeddings.

Alternative 1 - Cartography blend: treat a repository as terrain. File walkers survey land, parsers draw topographic contours, symbol extractors mark cities, references draw roads, and retrieval selects routes for the agent.

Alternative 2 - Legal evidence blend: treat every symbol and edge as an evidentiary claim. The record must include source, span, extraction method, confidence, and caveats; unresolved claims are admissible only as "unresolved."

Alternative 3 - Supply-chain logistics blend: treat parse outputs as packages moving through queues: discovered file -> parsed file -> extracted symbols -> chunks -> graph edges -> searchable documents -> rendered context. Each stage has batching, backpressure, retries, and failure reports.

Selected path: legal evidence plus supply-chain logistics. Parseltongue should manage repository analysis as a staged pipeline where every output is both durable data and an evidence-bearing claim.

Council debate summary:

- Repository Indexing Architect: start with deterministic file discovery and parse reports; indexing correctness begins before Tree-sitter sees a byte.
- Skeptical Systems Engineer: ignore behavior, config scoping, and deletes cause subtle corruption; graph data rots unless deletion and invalidation are tested.
- Graph Semantics Engineer response: make entity ownership and edge provenance first-class; never store a semantic edge without source evidence or confidence.
- Search/Retrieval Engineer response: chunk schemas need filters for language, path, source, and body tokens; retrieval should overfetch then rerank.
- Rust Storage Architect response: define narrow stage interfaces and make cache keys include file content, grammar, query, and extractor versions.

Core thesis: Parseltongue's repository layer should be a deterministic, staged evidence pipeline: discover files with explicit ignore policy, parse and extract source-spanned records, build conservative graph claims, persist versioned chunks and symbols, and expose retrieval with filters, budgets, and diagnostics.

## Phase 3 - Verification Anchors

Primary local evidence used:

- `chunkhound__chunkhound/src/lib.rs`
- `chunkhound__chunkhound/site/src/pages/docs/configuration.md`
- `Ataraxy-Labs__sem/crates/sem-core/src/model/entity.rs`
- `Ataraxy-Labs__sem/crates/sem-core/src/parser/scope_resolve.rs`
- `Ataraxy-Labs__sem/CHANGELOG.md`
- `Aider-AI__aider/aider/repomap.py`
- `TabbyML__tabby/crates/tabby-common/src/index/mod.rs`
- `TabbyML__tabby/crates/tabby-common/src/index/code/mod.rs`
- `TabbyML__tabby/crates/tabby/src/services/structured_doc/tantivy.rs`
- `CodeGraphContext__CodeGraphContext/CGC_E2E_BUG_REPORT.md`
- `CodeGraphContext__CodeGraphContext/CGC_GRAPH_INCONSISTENCIES.md`
- `docs/research003/repo-feature-summary.tsv`

Self-correction questions:

- Does local code show parallel repository walking with ignore behavior? Yes, `chunkhound__chunkhound/src/lib.rs` uses `ignore::WalkBuilder` and `build_parallel`.
- Does local code show a document/chunk search schema? Yes, Tabby's `IndexSchema` documents corpus -> document -> chunk and defines chunk attributes and tokens.
- Does local code show source-spanned semantic entities? Yes, Ataraxy's `SemanticEntity` includes line and byte spans, content hashes, and structural hash.
- Does local evidence show cache invalidation problems? Yes, CodeGraphContext's reports document config bleed, delete orphans, and golden drift.
- Does local code show repo-map graph ranking? Yes, Aider builds definitions/references and a NetworkX graph with personalization and weights.

## Pattern 1 - File Discovery Is a Policy Engine

Where found:

- Repository: `chunkhound__chunkhound`
- File: `src/lib.rs`
- Language: Rust with PyO3

Observed shape:

The scanner uses:

```rust
WalkBuilder::new(root)
    .git_ignore(true)
    .git_global(false)
    .git_exclude(false)
    .ignore(false)
    .hidden(false)
    .build_parallel()
```

It uses `GitignoreBuilder` for custom exclude patterns, skips configured directories with `WalkState::Skip`, filters by extension and exact filenames, and collects results through `Arc<Mutex<Vec<String>>>`.

Why this matters:

The index begins with a file selection policy. Bad file discovery causes parse noise, missed source, poor performance, and user distrust.

Why it matters for Parseltongue:

Parseltongue needs explicit, inspectable file selection:

- include patterns,
- exclude patterns,
- ignore mode,
- hidden-file policy,
- generated/vendor policy,
- max file size,
- exact filename support,
- language support table.

Rust translation:

```rust
pub struct FileDiscoveryPolicy {
    pub root: Utf8PathBuf,
    pub include_extensions: BTreeSet<FileExtension>,
    pub include_filenames: BTreeSet<FileName>,
    pub exclude_globs: Vec<GlobPattern>,
    pub skip_directories: BTreeSet<FileName>,
    pub ignore_mode: IgnoreMode,
    pub include_hidden: bool,
    pub max_file_size_bytes: Option<u64>,
}
```

When to use:

- Every repository indexing run.
- CLI "explain why this file was skipped."
- Watch mode bootstrapping.

When not to use:

- Do not bake ignore behavior into ad-hoc path filters in extractors.

Risks and caveats:

- `.gitignore` behavior differs from custom glob behavior.
- Hidden files may contain useful config or huge caches.
- Directory skipping must be deterministic in parallel walks.

Testing implications:

- Golden-test file discovery against fixture repositories.
- Include `.gitignore`, custom excludes, hidden files, exact names, extensionless files, symlinks, and nested skip dirs.

Agent guidance:

When generating new indexing code, create a `FileDiscoveryReport` with included/skipped counts and reasons.

## Pattern 2 - Ignore Modes Are User-Facing Configuration

Where found:

- Repository: `chunkhound__chunkhound`
- File: `site/src/pages/docs/configuration.md`
- Stack: documentation for CLI/config behavior

Observed options in local docs:

```text
exclude
include
exclude_mode
force_reindex
max_concurrent
cleanup
max_file_size_mb
config_file_size_threshold_kb
per_file_timeout_seconds
batch_size
db_batch_size
detect_embedded_sql
per_file_timeout_min_size_kb
```

Observed exclude modes:

```text
combined
config_only
gitignore_only
```

Why this matters:

Users need to control whether an analyzer follows `.gitignore`, project-specific config, or both. This is especially important for code intelligence because ignored generated files may still be relevant, and tracked large files may be expensive.

Why it matters for Parseltongue:

Parseltongue should make ignore policy explicit in config and reports:

- `combined`: respect project ignores plus Parseltongue config,
- `gitignore_only`: mimic source-control view,
- `config_only`: allow user override for analysis experiments,
- `none`: possible debug mode with warnings.

Rust translation:

```rust
pub enum IgnoreMode {
    Combined,
    GitignoreOnly,
    ConfigOnly,
    None,
}

pub struct FileDecision {
    pub path: Utf8PathBuf,
    pub decision: IncludeDecision,
    pub reasons: Vec<FileDecisionReason>,
}
```

When to use:

- Config files.
- CLI flags.
- Agent tools that index a repo.

When not to use:

- Do not silently change ignore policy between commands.

Risks and caveats:

- Different ignore modes can produce different graphs.
- Cache keys should include file discovery policy, or indexes should record the policy used.

Testing implications:

- Test same fixture under each ignore mode.
- Assert skipped reasons are stable and explainable.

Agent guidance:

When invoking Parseltongue from an agent, print or record the ignore mode so future steps understand coverage.

## Pattern 3 - Parse Reports Should Exist Per File

Where found:

- Repository: `Christoph__treesitter-mcp`
- File: `src/extraction/types.rs`
- Repository: `chunkhound__chunkhound`
- Config docs imply per-file timeouts and file-size thresholds.

Observed shape:

Christoph's extractor reads a file once, detects language, attempts extraction, and debug-logs read or parse errors while continuing.

Why this matters:

Repository indexing is partial by nature. A code-intelligence system should know which files were parsed, skipped, unsupported, too large, timed out, or failed.

Why it matters for Parseltongue:

Agentic assistants need confidence boundaries. "The repo map excludes generated files and three unsupported languages" is radically different from "the repo is fully indexed."

Rust translation:

```rust
pub struct RepositoryIndexReport {
    pub root: Utf8PathBuf,
    pub started_at: SystemTime,
    pub file_reports: Vec<FileParseReport>,
    pub totals: RepositoryIndexTotals,
}

pub struct FileParseReport {
    pub path: Utf8PathBuf,
    pub language: Option<LanguageId>,
    pub decision: FileIndexDecision,
    pub parse_duration: Option<Duration>,
    pub diagnostics: Vec<IndexDiagnostic>,
}
```

When to use:

- CLI indexing.
- Background watch mode.
- CI reports.
- Agent context selection.

When not to use:

- Do not require every tool response to include full per-file reports; expose summary plus optional detail.

Risks and caveats:

- Per-file reports can be large.
- Diagnostics should be structured and filterable.

Testing implications:

- Fixture with unsupported, unreadable, too-large, syntax-error, and valid files.
- Assert totals and diagnostics.

Agent guidance:

Generated Parseltongue commands should surface coverage summaries before making broad claims about a repo.

## Pattern 4 - Entity Model Needs Content, Structure, Span, Parentage

Where found:

- Repository: `Ataraxy-Labs__sem`
- File: `crates/sem-core/src/model/entity.rs`
- Language: Rust

Observed shape:

`SemanticEntity` includes:

```text
id
file_path
entity_type
name
parent_id
content
content_hash
structural_hash
start_line
end_line
start_byte
end_byte
metadata
```

Why this matters:

Symbols are not just names. Good indexing requires identity, hierarchy, source content, hashes, spans, and metadata.

Why it matters for Parseltongue:

LLM context and graph operations both need the same durable symbol model. A name-only tag is insufficient for refactoring, explanation, or graph traversal.

Rust translation:

```rust
pub struct SemanticEntity {
    pub id: EntityId,
    pub file_id: FileId,
    pub kind: EntityKind,
    pub name: SymbolName,
    pub parent_id: Option<EntityId>,
    pub span: SourceSpan,
    pub content_hash: ContentHash,
    pub structural_hash: Option<StructuralHash>,
    pub metadata: EntityMetadata,
}
```

When to use:

- Symbols.
- Types.
- Functions.
- Classes.
- Modules.
- Chunks that map to semantic entities.

When not to use:

- Do not store full content in every entity if a source-store plus spans can reconstruct it.

Risks and caveats:

- Entity IDs must handle overloads, same-name symbols, same-line declarations, and generated anonymous constructs.
- Parentage can change after refactors.

Testing implications:

- Test overloaded/same-name entities.
- Test parent-child IDs.
- Test exact span slicing.

Agent guidance:

When adding a new symbol kind, update identity, parentage, span, and metadata tests together.

## Pattern 5 - Entity IDs Need Disambiguators

Where found:

- Repository: `Ataraxy-Labs__sem`
- File: `crates/sem-core/src/model/entity.rs`
- Language: Rust

Observed shape:

The code provides `build_entity_id`, `build_entity_id_disambiguated`, and `build_entity_id_disambiguated_with_ordinal`, adding line number and same-line ordinal when needed.

Why this matters:

Real languages allow overloads, same-name functions, generated methods, nested scopes, and multiple declarations on one line.

Why it matters for Parseltongue:

Stable entity IDs are the backbone of graph edges, cache keys, summaries, and agent references. Without disambiguation, edges silently attach to the wrong symbol.

Rust translation:

```rust
pub struct EntityIdComponents {
    pub file_id: FileId,
    pub parent_id: Option<EntityId>,
    pub kind: EntityKind,
    pub name: SymbolName,
    pub start_line: usize,
    pub same_line_ordinal: usize,
}
```

When to use:

- Any symbol identity.
- Graph nodes.
- Persistent caches.

When not to use:

- Do not expose raw generated IDs as the only user-facing display. Keep a human display name.

Risks and caveats:

- Line-based IDs can shift after edits.
- Content-hash-based IDs can change after edits.
- Fully stable semantic IDs are hard across refactors.

Testing implications:

- Test duplicate names in same file.
- Test same-line declarations.
- Test nested same-name symbols.

Agent guidance:

Generate both stable-ish internal IDs and readable labels. Never key graph edges only by symbol name.

## Pattern 6 - Scope-Aware References Are Graph Edge Preconditions

Where found:

- Repository: `Ataraxy-Labs__sem`
- File: `crates/sem-core/src/parser/scope_resolve.rs`
- Language: Rust

Observed shape:

The resolver models scopes, local definitions, bindings, variable type bindings, unresolved call assignments, field-access assignments, and reference kinds such as bare calls, scoped calls, and method calls.

Why this matters:

Graph edges such as `CALLS`, `CONTAINS`, `IMPLEMENTS`, and `IMPORTS` can be false if references are owned by the wrong scope or resolved by name alone.

Why it matters for Parseltongue:

Parseltongue's value to LLM agents depends on relationship accuracy. A wrong call graph can guide an agent to modify the wrong code.

Rust translation:

```rust
pub struct ReferenceRecord {
    pub id: ReferenceId,
    pub source_entity: Option<EntityId>,
    pub target: ReferenceTarget,
    pub kind: ReferenceKind,
    pub span: SourceSpan,
    pub confidence: ResolutionConfidence,
}

pub enum ReferenceTarget {
    Resolved(EntityId),
    Unresolved(SymbolName),
    Dynamic(String),
}
```

When to use:

- Call graphs.
- Import graphs.
- Reference search.
- Blast-radius analysis.

When not to use:

- Do not claim compiler-grade resolution where only syntax-level evidence exists.

Risks and caveats:

- Dynamic dispatch, macros, reflection, generated code, decorators, and overloads complicate resolution.
- Cross-file resolution requires import/export understanding.

Testing implications:

- Use language-specific fixtures for scope and shadowing.
- Include unresolved-reference snapshots.
- Separate detection tests from resolution tests.

Agent guidance:

If resolution confidence is low, preserve unresolved evidence instead of inventing a target.

## Pattern 7 - Document -> Chunk Schema Separates Storage and Retrieval

Where found:

- Repository: `TabbyML__tabby`
- File: `crates/tabby-common/src/index/mod.rs`
- Language: Rust

Observed shape:

Tabby's schema comments describe:

```text
corpus -> document -> chunk
```

A document is a group of chunks, and chunk is the unit retrieved during search. The schema stores corpus, source ID, document ID, document attributes, failed chunk count, chunk ID, chunk attributes, and chunk tokens.

Why this matters:

Search systems need stable document identity and smaller retrieval units. Chunk-level retrieval without document context loses provenance; document-level retrieval is too coarse.

Why it matters for Parseltongue:

Parseltongue can model:

- repository as source,
- file as document,
- symbol/chunk as retrieval unit,
- parse diagnostics as document metadata,
- chunk spans and entity IDs as chunk metadata.

Rust translation:

```rust
pub struct IndexedDocument {
    pub source_id: SourceId,
    pub document_id: DocumentId,
    pub file_id: FileId,
    pub attributes: DocumentAttributes,
    pub failed_chunks_count: u64,
}

pub struct IndexedChunk {
    pub chunk_id: ChunkId,
    pub document_id: DocumentId,
    pub span: SourceSpan,
    pub attributes: ChunkAttributes,
    pub indexed_tokens: Vec<String>,
}
```

When to use:

- Code search.
- LLM context retrieval.
- Embedding indexes.
- Hybrid lexical/semantic search.

When not to use:

- Do not store huge full chunk bodies in every search index if a source store can reconstruct them.

Risks and caveats:

- Chunk boundaries must be stable enough for cache reuse.
- Chunk attributes need versioned schema.

Testing implications:

- Test file with multiple chunks.
- Test chunk source reconstruction.
- Test failed chunk counts.

Agent guidance:

When generating retrieval code, keep document-level and chunk-level metadata separate.

## Pattern 8 - Code Search Filters Should Not Pollute Score

Where found:

- Repository: `TabbyML__tabby`
- File: `crates/tabby-common/src/index/code/mod.rs`
- Language: Rust

Observed shape:

Code search builds Boolean queries over body tokens, corpus, source ID, language, and filepath. Language/source/path filters are wrapped in constant-score queries so they do not affect ranking.

Why this matters:

Filters constrain the candidate set; they should not boost irrelevant matches just because they match metadata.

Why it matters for Parseltongue:

Agent retrieval should allow:

- language filters,
- path filters,
- current-file exclusion,
- repository/source filters,
- symbol-kind filters,
- confidence filters.

But the ranking should primarily reflect relevance to the user query or agent task.

Rust translation:

```rust
pub struct CodeSearchFilter {
    pub language: Option<LanguageId>,
    pub include_paths: Vec<PathPattern>,
    pub exclude_paths: Vec<PathPattern>,
    pub symbol_kinds: Vec<EntityKind>,
    pub min_confidence: Option<ResolutionConfidence>,
}
```

When to use:

- Search.
- Context selection.
- Test discovery.
- Refactor planning.

When not to use:

- Do not make filters invisible. The user or agent should know when a search excluded the current file or language.

Risks and caveats:

- Over-filtering can hide relevant code.
- Path normalization must be consistent.

Testing implications:

- Test language normalization, especially JS/TS variants.
- Test current-file exclusion.
- Test query explain output.

Agent guidance:

When selecting context, log filters and provide an "expanded search" fallback when results are weak.

## Pattern 9 - Language Normalization in Search Filters

Where found:

- Repository: `TabbyML__tabby`
- File: `crates/tabby-common/src/index/code/mod.rs`
- Language: Rust

Observed shape:

Tabby maps `javascript`, `typescript`, `javascriptreact`, and `typescriptreact` to `javascript-typescript` in `language_query`.

Why this matters:

Some retrieval tasks care about language families more than exact file language. JavaScript, TypeScript, JSX, and TSX often share search semantics.

Why it matters for Parseltongue:

Language support needs both exact and family-level filters:

- exact: `typescriptreact`,
- family: `javascript-typescript`,
- ecosystem: `web`,
- normalized syntax: `c-like`.

Rust translation:

```rust
pub enum LanguageFilter {
    Exact(LanguageId),
    Family(LanguageFamily),
}

pub enum LanguageFamily {
    JavaScriptTypeScript,
    CFamily,
    Jvm,
    DotNet,
    Shell,
}
```

When to use:

- Search filters.
- Context retrieval.
- Cross-language summaries.

When not to use:

- Do not use language family normalization for parser selection.

Risks and caveats:

- Family filters can include semantically different languages.
- Users may expect exact language behavior.

Testing implications:

- Unit-test filter expansion.
- Test exact vs family search results.

Agent guidance:

Use exact language filters for parsing, family filters for retrieval and exploration.

## Pattern 10 - Repo Maps Build a Graph From Defs and Refs

Where found:

- Repository: `Aider-AI__aider`
- File: `aider/repomap.py`
- Language: Python

Observed shape:

Aider extracts tags, groups definitions and references, builds a `networkx.MultiDiGraph`, adds edges from referencers to definers, weights identifiers by mention, naming style, frequency, and whether the file is in chat, then uses ranking to build a repository map.

Why this matters:

A repo map is a ranked structural summary, not a flat file listing.

Why it matters for Parseltongue:

Parseltongue can build agent context by ranking:

- files already in the conversation,
- mentioned names,
- symbols referenced by those names,
- dependency edges,
- tests touching changed code,
- high-centrality modules.

Rust translation:

```rust
pub struct RepoMapRankInput {
    pub chat_files: BTreeSet<FileId>,
    pub mentioned_files: BTreeSet<FileId>,
    pub mentioned_symbols: BTreeSet<SymbolName>,
    pub definitions: Vec<SymbolRecord>,
    pub references: Vec<ReferenceRecord>,
}
```

When to use:

- LLM context construction.
- Code explanation.
- Change planning.
- Review assistance.

When not to use:

- Do not use repo-map ranking as proof of semantic dependency. It is a relevance heuristic.

Risks and caveats:

- Incomplete refs lead to poor ranking.
- Large monorepos require budget-aware graph algorithms.
- Dynamic language fallback refs can flood the graph.

Testing implications:

- Fixture repo where a mentioned symbol should pull in its definer.
- Fixture where high-frequency generic names should not dominate.
- Snapshot ranked files under known mentions.

Agent guidance:

Use repo maps to guide exploration, then inspect source evidence before modifying code.

## Pattern 11 - Cache by File Version and Parser/Query Version

Where found:

- Repository: `Aider-AI__aider`
- File: `aider/repomap.py`
- Repository: `Ataraxy-Labs__sem`
- File: `CHANGELOG.md`

Observed shape:

Aider caches tags by filename and mtime and bumps cache version for language-pack changes. Ataraxy's changelog describes file content storage plus byte spans, reducing repeated entity content storage in a large corpus.

Why this matters:

Index caches are performance features and correctness risks. They must invalidate when source, grammar, queries, or extractor logic changes.

Why it matters for Parseltongue:

Parseltongue should use stronger cache keys than path/mtime for durable indexes:

```text
file content hash
language id
grammar version
query bundle version
extractor version
file discovery policy
```

Rust translation:

```rust
pub struct SymbolCacheKey {
    pub file_content_hash: ContentHash,
    pub language: LanguageId,
    pub grammar_version: GrammarVersion,
    pub query_bundle_hash: QueryBundleHash,
    pub extractor_version: ExtractorVersion,
}
```

When to use:

- Parse trees.
- Captures.
- Symbol records.
- Chunks.
- Embeddings.

When not to use:

- Avoid relying only on modification time for long-lived caches.

Risks and caveats:

- Hashing all files has a cost.
- Cache migrations need tests.

Testing implications:

- Change query file, assert cache miss.
- Change extractor version, assert cache miss.
- Same content with different mtime should be a hit if content-hash keyed.

Agent guidance:

When generating cache code, list every input that can affect output and put it in the key.

## Pattern 12 - Store File Content Once, Slice by Byte Span

Where found:

- Repository: `Ataraxy-Labs__sem`
- File: `CHANGELOG.md`
- Language: Rust project documentation

Observed evidence:

The changelog reports a cache pattern using a file-contents table plus byte spans, with content stored once and entities slicing original bytes. It reports measured size changes on a 139K-entity corpus. These are repository-provided measurements and should be independently verified before treating them as universal benchmarks.

Why this matters:

Duplicating content across every entity and chunk inflates storage. Byte spans plus a content store can reduce duplication and preserve exact reconstruction.

Why it matters for Parseltongue:

Parseltongue will generate overlapping views: symbols, chunks, summaries, references, diagnostics. Many point into the same file. Store the file once.

Rust translation:

```rust
pub struct ContentAddressedFile {
    pub file_id: FileId,
    pub content_hash: ContentHash,
    pub bytes: Arc<[u8]>,
}

pub struct SpanBackedEntity {
    pub entity_id: EntityId,
    pub file_version: FileVersionId,
    pub span: ByteSpan,
}
```

When to use:

- Persistent index.
- LLM context reconstruction.
- Chunk storage.

When not to use:

- Very small ephemeral tools may copy snippets for simplicity.

Risks and caveats:

- Source content retention has disk and privacy implications.
- Byte spans need source versioning.

Testing implications:

- Reconstruct entity content from stored file plus span.
- Test deletion and cleanup of unreferenced content blobs.

Agent guidance:

Prefer file-content storage plus spans over storing large duplicated strings.

## Pattern 13 - Retrieval Should Overfetch Before Final Selection

Where found:

- Repository: `TabbyML__tabby`
- File: `crates/tabby/src/services/structured_doc/tantivy.rs`
- Evidence from local inspection: search uses a limit multiplied by two before final extraction/ranking.

Why this matters:

First-pass lexical search is noisy. Overfetching gives downstream ranking, filtering, or deduplication room to work.

Why it matters for Parseltongue:

For LLM context, the cost of missing the right code is often worse than initially retrieving too much, as long as final context is budgeted.

Rust translation:

```rust
pub struct RetrievalPolicy {
    pub requested_limit: usize,
    pub overfetch_multiplier: usize,
    pub final_token_budget: TokenBudget,
}
```

When to use:

- Search + rerank.
- Hybrid lexical + graph retrieval.
- Context construction.

When not to use:

- Avoid unbounded overfetch in huge repositories.

Risks and caveats:

- Overfetching increases latency and memory.
- Reranker quality determines final benefit.

Testing implications:

- Measure recall at K before and after overfetch.
- Test latency budgets.

Agent guidance:

Use overfetching when building context, but always trim with explicit token and item budgets.

## Pattern 14 - Graph Deletion Must Remove Orphans

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- File: `CGC_E2E_BUG_REPORT.md`

Observed issue:

`BUG-005` reports that delete leaves orphan nodes such as parameters and variables after deleting a repository.

Why this matters:

Indexes are not append-only. Deletes, reindexes, branch switches, and file moves must clean all derived data.

Why it matters for Parseltongue:

Agent worktrees change constantly. If stale symbols remain, agents may retrieve deleted code or hallucinate relationships from old state.

Rust translation:

```rust
pub trait RepositoryIndexStore {
    fn delete_repository_index(
        &self,
        repository_id: RepositoryId,
    ) -> Result<DeleteSummary, StoreError>;
}

pub struct DeleteSummary {
    pub documents_deleted: u64,
    pub chunks_deleted: u64,
    pub symbols_deleted: u64,
    pub references_deleted: u64,
    pub orphan_records_remaining: u64,
}
```

When to use:

- Repository delete.
- Forced reindex.
- Branch switch.
- Worktree cleanup.

When not to use:

- Never delete by path string alone if records have repository IDs.

Risks and caveats:

- Derived records without repository/path metadata are hard to clean.
- Graph databases may require transaction boundaries.

Testing implications:

- Index fixture, delete it, assert zero records by repository ID.
- Test nested derived nodes such as parameters and locals.
- Test repeated index/delete cycles.

Agent guidance:

Generate delete tests whenever adding a new derived record type.

## Pattern 15 - Config Isolation Is Part of Index Correctness

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- File: `CGC_E2E_BUG_REPORT.md`

Observed issue:

`BUG-001` reports that repo-local `.codegraphcontext/.env` can override isolated global config, redirecting indexing/querying to unexpected databases.

Why this matters:

Code intelligence results depend on configuration. If configuration scopes bleed, tests can pass against the wrong graph or users can query stale data.

Why it matters for Parseltongue:

Parseltongue should make workspace/root/config/database identity explicit in every index and tool response.

Rust translation:

```rust
pub struct IndexContext {
    pub workspace_root: Utf8PathBuf,
    pub config_source: ConfigSource,
    pub storage_namespace: StorageNamespace,
    pub repository_id: RepositoryId,
}
```

When to use:

- CLI commands.
- MCP tools.
- CI.
- Multi-repo indexing.

When not to use:

- Do not infer storage namespace only from current working directory without reporting it.

Risks and caveats:

- Per-repo and global modes can confuse users.
- Hidden config files can alter behavior.

Testing implications:

- Isolated HOME tests.
- CWD inside repo with local config tests.
- Explicit config precedence tests.

Agent guidance:

Every indexing or query command should report which workspace and storage namespace it used.

## Pattern 16 - Golden Drift Must Be Diagnosed by Node and Edge Class

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- Files: `CGC_E2E_BUG_REPORT.md`, `CGC_GRAPH_INCONSISTENCIES.md`

Observed issue classes:

- Node-count drift while edge counts match.
- Extra parameter and variable nodes.
- Missing enum members.
- Missing partial-class relationships.
- Missing implementation/protocol/companion edges.
- Published package behavior diverging from local editable behavior.

Why this matters:

A single "golden mismatch" number is not enough. Drift must be categorized by node kind, edge kind, language, and release artifact.

Why it matters for Parseltongue:

Parseltongue should produce diff reports that explain:

- added/removed symbols by kind,
- added/removed edges by kind,
- unresolved references,
- parser/query version changes,
- fixture language affected.

Rust translation:

```rust
pub struct GoldenDiffReport {
    pub language: LanguageId,
    pub symbol_kind_deltas: BTreeMap<EntityKind, CountDelta>,
    pub edge_kind_deltas: BTreeMap<EdgeKind, CountDelta>,
    pub examples: Vec<GoldenDiffExample>,
}
```

When to use:

- Query changes.
- Grammar upgrades.
- Extractor changes.
- Release validation.

When not to use:

- Do not accept broad tolerances without classifying the difference.

Risks and caveats:

- Goldens can encode old bugs.
- Perfect self-generated goldens can create circular validation.

Testing implications:

- Keep source-truth fixtures separate from generated graph snapshots.
- Review golden updates as semantic changes.

Agent guidance:

When a golden changes, summarize the semantic reason, not just the count delta.

## Pattern 17 - Similar Implementations Reveal a Canonical Pipeline

Where found:

- `chunkhound__chunkhound`: repository file discovery.
- `Christoph__treesitter-mcp`: language detection and type extraction.
- `Ataraxy-Labs__sem`: entity model and scope-aware references.
- `Aider-AI__aider`: tags -> repo map -> token budget.
- `TabbyML__tabby`: document/chunk indexing and code search filters.
- `CodeGraphContext__CodeGraphContext`: graph indexing and audit failures.

Canonical pipeline:

```text
discover files
-> detect language
-> parse source bytes
-> run query layers
-> refine captures by traversal
-> create source-spanned entities
-> create references and edges
-> build chunks
-> persist document/chunk/symbol/edge records
-> retrieve by lexical/graph/context signals
-> render agent context with provenance
```

Why this matters:

The same architecture appears in different forms across repo maps, code search tools, graph indexers, and MCP servers.

Why it matters for Parseltongue:

Parseltongue should implement this as explicit stages, not one monolithic parser pass.

Rust translation:

```rust
pub trait IndexStage<I, O> {
    fn run_stage(&self, input: I, diagnostics: &mut DiagnosticSink) -> Result<O, StageError>;
}
```

When to use:

- Initial architecture.
- Refactoring.
- Performance profiling.

When not to use:

- Do not over-engineer with async queues before a synchronous staged pipeline is correct.

Risks and caveats:

- Stage boundaries can add serialization overhead.
- Too many generic traits can obscure simple data flow.

Testing implications:

- Unit-test each stage.
- Integration-test full pipeline on fixture repos.
- Snapshot stage outputs for regression diagnosis.

Agent guidance:

Future agents should modify one stage at a time and verify downstream effects with golden diffs.

## Repository Indexing Checklist for Parseltongue

- File discovery policy is explicit and reported.
- Ignore mode is part of index metadata.
- Every file has a parse/index decision.
- Language detection is centralized.
- Parse output preserves source spans and file versions.
- Entities have IDs, parent IDs, kinds, names, spans, and hashes.
- References are source-spanned and can be unresolved.
- Graph edges carry evidence and confidence.
- Documents and chunks are separate.
- Chunk search filters are separate from score.
- Cache keys include source, grammar, query, and extractor versions.
- Deletes remove derived data and report orphan counts.
- Golden diffs classify node and edge changes by kind and language.

## Anti-Patterns Captured

- Treating file walking as incidental.
- Path/mtime-only durable cache keys.
- Name-only graph nodes.
- Edge records without source evidence.
- Silent partial indexing.
- Stale graph records after delete.
- Config bleed across repositories.
- Over-normalizing language semantics.
- Search filters that accidentally influence ranking.
- Golden reports that hide which construct changed.

## Transferable Design Principle

Tree-sitter parse trees become useful only after they are staged into evidence-bearing repository data. Parseltongue should make every stage explicit, every cache versioned, every derived record deletable, and every semantic claim traceable back to source bytes.
