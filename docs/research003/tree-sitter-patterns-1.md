# Tree-Sitter Patterns 1: Parser Runtime, Queries, Traversal, Source Mapping

This file is part of the `docs/research003` corpus for Parseltongue, a Rust LLM companion centered on Tree-sitter powered code understanding.

## Phase 0 - Deconstruct and Clarify

Core objective: extract reusable runtime and parser-facing patterns from the repositories under:

`/Users/amuldotexe/Desktop/personal-repos-lane/parseltongue-rust-LLM-companion/git-ref-repo`

Desired output: a long-lived, repo-grounded reference corpus for designing, implementing, testing, debugging, benchmarking, and evolving a robust Rust Tree-sitter code-intelligence layer.

Premise is sound. Proceeding with optimized protocol.

Important caveat: the requested `codegraphcontext-evidence-reader` workflow was attempted against representative repositories, but the CGC scan processes terminated with exit 143 before usable graph listings or stats were produced. The corpus therefore uses local repository forensics, direct file inspection, and broad `rg` scans as the primary evidence. Claims below are grounded in local files unless explicitly marked as an inference.

## Phase 1 - Cognitive Staging

Expert council used for this slice:

- Tree-sitter Runtime Engineer: focuses on parser setup, query API drift, tree editing, cursor traversal, and source ranges.
- Rust Systems Architect: translates patterns into ownership-safe Rust modules, traits, error types, and cache boundaries.
- Code Intelligence Engineer: maps syntax captures into definitions, references, symbols, and repository context.
- Skeptical Systems Engineer: challenges query fragility, parser lifetime risks, incremental parsing correctness, memory blowups, and byte/UTF-8 mistakes.
- Agentic Context Designer: keeps every pattern usable by future coding agents generating Parseltongue changes.

Knowledge scaffolding:

- Tree-sitter parser lifecycle: `Parser`, `Language`, `Tree`, `Node`, `TreeCursor`, `Query`, `QueryCursor`.
- Incremental parsing: old tree reuse, `Tree::edit`, changed ranges, byte and point ranges.
- Query model: compiled query assets, capture names, language-specific grammars, compatibility shims.
- Traversal model: field-name navigation, named children, byte ranges, parent/child/sibling traversal.
- Rust idioms: newtypes for spans, zero-copy source slicing, explicit errors, `Arc`, `RwLock`, parser pools, Send/Sync boundaries.
- Agent workflows: parse once, preserve spans, report truncation, keep exact source evidence available for later context construction.

## Phase 2 - Multi-Perspective Synthesis

Conventional approach: create one parser per language, run language-specific `.scm` queries, traverse captured nodes into symbols, and store file path plus line numbers.

Alternative 1 - Linguistics blend: treat grammars as dialects and captures as a normalized interlingua. This pushes Parseltongue to keep a stable semantic vocabulary such as `definition`, `reference`, `call`, `import`, and `type`, while allowing each language query to diverge locally.

Alternative 2 - Archaeology blend: treat every parsed file as a stratified site. Byte spans, line spans, structural hashes, and source slices become provenance records. This helps preserve exact evidence for LLMs and makes later refactors auditable.

Alternative 3 - Operating-system kernel blend: treat parser instances, language objects, source buffers, and trees as kernel resources with lifetimes, capabilities, quotas, and invalidation rules. This exposes the need for parser pools, query registries, memory budgets, and cache eviction.

Selected path: a hybrid of linguistics plus archaeology plus OS-kernel resource discipline. Parseltongue should normalize across language dialects, preserve exact byte provenance, and manage parser/query/tree resources explicitly.

Council debate summary:

- Runtime Engineer: compile queries once, parse source bytes, and never collapse byte spans into only line spans.
- Skeptical Systems Engineer: query captures are fragile under grammar drift; incremental parsing silently lies if edits are wrong; API shims can hide unsupported behavior.
- Rust Systems Architect response: make grammar version, parser API version, and query version explicit in cache keys; require tests for edit roundtrips and source-slice equality.
- Code Intelligence Engineer response: use queries only for coarse candidates, then verify with field-name traversal and source-span ownership rules.
- Agentic Context Designer response: every extracted symbol should carry enough provenance for a later agent to inspect exact bytes and explain why it was selected.

Core thesis: Parseltongue's parser runtime should be built as a provenance-preserving resource manager: parse source bytes through versioned language adapters, run compiled query registries for coarse discovery, refine with grammar-aware traversal, and persist exact file, byte, point, and truncation metadata so downstream LLM context is explainable and correctable.

## Phase 3 - Verification Anchors

Primary local evidence used:

- `Christoph__treesitter-mcp/src/extraction/types.rs`
- `wrale__mcp-server-tree-sitter/src/mcp_server_tree_sitter/utils/tree_sitter_helpers.py`
- `wrale__mcp-server-tree-sitter/src/mcp_server_tree_sitter/language/registry.py`
- `Ataraxy-Labs__sem/crates/sem-core/src/model/entity.rs`
- `Ataraxy-Labs__sem/crates/sem-core/src/parser/scope_resolve.rs`
- `viktorstrate__swift-tree-sitter/Sources/SwiftTreeSitter/*.swift` and tests found by repository search
- `docs/research003/repo-feature-summary.tsv`
- `/tmp/parseltongue-ts-rg.txt`
- `/tmp/parseltongue-api-evidence.txt`

Fact-check questions asked during self-correction:

- Does local Rust evidence show direct Tree-sitter parser/query/cursor usage? Yes: `Christoph__treesitter-mcp/src/extraction/types.rs` imports `Node`, `Parser`, `Query`, and `QueryCursor`.
- Does local evidence show py-tree-sitter API drift handling? Yes: `wrale__mcp-server-tree-sitter` and Aider both branch between old `query.captures` and newer `QueryCursor`.
- Does local evidence show byte spans as durable entity data? Yes: `Ataraxy-Labs__sem/crates/sem-core/src/model/entity.rs` stores `start_byte` and `end_byte` and documents them as Tree-sitter node byte offsets.
- Does local evidence show incremental edit semantics? Yes: `wrale__mcp-server-tree-sitter` exposes `edit_tree` and `parse_source_incremental`; Swift bindings expose parser old-tree parsing and input edits.
- Does local evidence justify claims about every repository? Broad path and text scans covered 609 Git repositories, but direct source reading covered selected high-signal repositories. This file should not be read as a complete per-repo audit.

## Pattern 1 - Version-Compatible Query Execution

Where found:

- Repository: `wrale__mcp-server-tree-sitter`
- File: `src/mcp_server_tree_sitter/utils/tree_sitter_helpers.py`
- Language: Python
- Related repository: `Aider-AI__aider`
- File: `aider/repomap.py`

Observed shape:

```python
try:
    from tree_sitter import Query
    return Query(language, query_string)
except (ImportError, TypeError):
    return language.query(query_string)
```

and:

```python
if hasattr(query, "captures"):
    return query.captures(node)
from tree_sitter import QueryCursor
cursor = QueryCursor(query)
return cursor.captures(node)
```

Why this matters:

Tree-sitter bindings evolve. In py-tree-sitter, query construction and capture execution moved between APIs. Systems that index arbitrary user repositories cannot assume a single binding surface unless they fully vendor or pin it.

Why it matters for Parseltongue:

Parseltongue's target implementation is Rust, where the binding API is more stable than Python in this evidence, but the same concept applies to language grammar crates, Tree-sitter runtime versions, generated parser ABI versions, and optional WASM/native backends.

Rust translation:

```rust
pub struct CompiledQuery {
    pub language_id: LanguageId,
    pub query_kind: QueryKind,
    pub runtime_version: TreeSitterRuntimeVersion,
    pub grammar_version: GrammarVersion,
    pub query: tree_sitter::Query,
}

pub trait QueryCompiler {
    fn compile_query_checked(
        &self,
        language: LanguageId,
        kind: QueryKind,
        source: &str,
    ) -> Result<CompiledQuery, QueryCompileError>;
}
```

When to use:

- Any time query files are loaded from disk or bundled assets.
- Any time Parseltongue supports grammar crates that may move independently.
- Any time capture names form part of a public indexing contract.

When not to use:

- Do not hide compile failures with silent fallback queries. A missing capture should be a structured warning or error.

Risks and caveats:

- Compatibility wrappers can mask real regressions.
- A query may compile but no longer capture expected node kinds after grammar drift.
- Capture output shapes can differ by runtime binding.

Testing implications:

- Golden-test every query file against fixture source.
- Include one fixture per major grammar construct and one failure case per language.
- Snapshot expected capture names and node byte ranges, not just symbol names.

Agent guidance:

When generating Parseltongue query code, create a typed query registry with explicit compile errors, capture-name validation, and grammar-version cache keys.

## Pattern 2 - Thread-Safe Language Registry

Where found:

- Repository: `wrale__mcp-server-tree-sitter`
- File: `src/mcp_server_tree_sitter/language/registry.py`
- Language: Python

Observed shape:

The registry keeps a lock, a language cache, an extension map, preloading, availability checks, and parser loading through `tree_sitter_language_pack`.

Representative extension map:

```text
rs -> rust
ts -> typescript
tsx -> typescript
js -> javascript
jsx -> javascript
py -> python
go -> go
java -> java
kt -> kotlin
swift -> swift
sql -> sql
sh -> bash
```

Why this matters:

Language detection and language-object loading are not incidental utilities. They are central boundaries that determine what can be parsed, cached, skipped, tested, and reported.

Why it matters for Parseltongue:

Parseltongue needs an explicit registry rather than scattered file-extension switches. Without a registry, grammar availability, query availability, parser reuse, and diagnostic reporting will drift apart.

Rust translation:

```rust
pub struct LanguageRegistry {
    languages: dashmap::DashMap<LanguageId, Arc<LanguageHandle>>,
    extension_map: ExtensionLanguageMap,
    query_registry: Arc<QueryRegistry>,
}

pub struct LanguageHandle {
    pub id: LanguageId,
    pub tree_sitter_language: tree_sitter::Language,
    pub grammar_version: GrammarVersion,
    pub supported_queries: BTreeSet<QueryKind>,
}
```

When to use:

- Repository scanning.
- CLI language listing.
- Query compilation.
- Parser pool initialization.
- Error reporting when a file extension is known but unsupported.

When not to use:

- Do not make extension mapping the only detection mechanism for ambiguous files such as headers, `.m`, `.h`, `.inc`, extensionless scripts, Markdown code fences, or generated files.

Risks and caveats:

- Extension mapping is lossy.
- Header files can be C, C++, Objective-C, or mixed.
- Markdown, notebooks, HTML, and template files may require embedded-language parsing.

Testing implications:

- Test extension-to-language mapping with ambiguous cases.
- Test language availability separately from parser availability.
- Test that missing languages produce actionable errors.

Agent guidance:

Future Parseltongue agents should add languages by touching a registry entry, query assets, fixture files, and golden capture tests together.

## Pattern 3 - Parser Creation with API Fallbacks

Where found:

- Repository: `wrale__mcp-server-tree-sitter`
- File: `src/mcp_server_tree_sitter/utils/tree_sitter_helpers.py`
- Language: Python

Observed shape:

```python
parser = Parser()
try:
    parser.set_language(safe_language)
except AttributeError:
    parser.language = safe_language
```

Why this matters:

The parser construction boundary is where dynamic language objects become operational. Incompatibility here should be caught early and reported with language and version context.

Why it matters for Parseltongue:

Rust should not need the same Python fallback, but it should still isolate parser construction so that parser reuse, pooling, timeout/cancellation, included ranges, and language switching can be tested.

Rust translation:

```rust
pub struct ParserFactory {
    registry: Arc<LanguageRegistry>,
}

impl ParserFactory {
    pub fn create_parser_for_language(
        &self,
        language_id: LanguageId,
    ) -> Result<tree_sitter::Parser, ParserCreateError> {
        let language = self.registry.language(language_id)?;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language.tree_sitter_language)?;
        Ok(parser)
    }
}
```

When to use:

- When each parse task needs its own parser.
- When parser pooling is introduced.
- When language switching may happen across parse jobs.

When not to use:

- Do not share a mutable parser freely across concurrent tasks. Treat parser access as exclusive unless the API and wrapper prove otherwise.

Risks and caveats:

- Parser reuse can leak included ranges, timeout settings, cancellation flags, or stale language settings if not reset.
- Parser creation cost may matter in repository-wide indexing.

Testing implications:

- Test parser reuse across different languages.
- Test that parser settings are reset between jobs.
- Test parse errors include language and file path.

Agent guidance:

Generate a `ParserPool` only after measuring parser creation cost. Until then, keep a `ParserFactory` boundary so pooling can be added without changing extractors.

## Pattern 4 - Incremental Parse Requires Exact Edit Metadata

Where found:

- Repository: `wrale__mcp-server-tree-sitter`
- File: `src/mcp_server_tree_sitter/utils/tree_sitter_helpers.py`
- Language: Python
- Repository: `viktorstrate__swift-tree-sitter`
- Files: `Sources/SwiftTreeSitter/STSParser.swift`, `STSInputEdit.swift`, tests found by search
- Language: Swift

Observed shape:

`wrale__mcp-server-tree-sitter` exposes `parse_source_incremental(source, old_tree, parser)` and `edit_tree(...)`. Swift bindings expose `parse(... oldTree: tree)` and input edits with start and old/new byte and point ranges.

Why this matters:

Incremental parsing is only correct when the old tree is edited with exactly the same byte and point delta as the source buffer. If byte offsets and points disagree, later changed ranges and node reuse can become misleading.

Why it matters for Parseltongue:

Parseltongue will likely want watch-mode indexing. Incremental parsing is tempting, but incorrect edit tracking can poison symbol indexes, call graphs, and LLM context.

Rust translation:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceEdit {
    pub start_byte: ByteOffset,
    pub old_end_byte: ByteOffset,
    pub new_end_byte: ByteOffset,
    pub start_point: SourcePoint,
    pub old_end_point: SourcePoint,
    pub new_end_point: SourcePoint,
}

pub fn apply_tree_edit_checked(
    tree: &mut tree_sitter::Tree,
    edit: SourceEdit,
) -> Result<(), EditValidationError> {
    edit.validate_monotonic()?;
    tree.edit(&edit.into_tree_sitter_input_edit());
    Ok(())
}
```

When to use:

- Watch mode.
- Editor integration.
- Local agent sessions that repeatedly modify files.
- Large repositories where full reparsing is too expensive.

When not to use:

- Avoid incremental parsing in the first implementation unless there are correctness tests against full reparse output.
- Avoid using changed ranges as the only invalidation signal for higher-level graph edges before testing nested references.

Risks and caveats:

- Byte offsets must be UTF-8 byte offsets, not Unicode scalar or grapheme positions.
- Points are row and column coordinates with binding-specific conventions.
- A single edit can invalidate ancestor nodes that own symbol context.
- The helper in `wrale__mcp-server-tree-sitter` defaults missing edit values to zero in the non-dict branch; that is a useful footgun warning, not a pattern to copy.

Testing implications:

- Apply edit, incremental parse, full reparse, compare root sexp or selected captures.
- Test ASCII, multi-byte UTF-8, line insertion, line deletion, and edits at file boundaries.
- Test cache invalidation for symbols whose spans move but content remains the same.

Agent guidance:

Do not generate incremental parsing code without an `EditValidationError`, a full-reparse parity test, and cache invalidation tests.

## Pattern 5 - Query for Coarse Candidates, Traverse for Details

Where found:

- Repository: `Christoph__treesitter-mcp`
- File: `src/extraction/types.rs`
- Language: Rust

Observed shape:

The Rust extractor creates a parser, sets `tree_sitter_rust::LANGUAGE`, parses source, compiles an inline query for type-level constructs, and then uses field-name traversal such as `child_by_field_name("body")` to collect fields, variants, and members.

Why this matters:

Queries are excellent for finding candidate nodes, but detailed extraction often needs grammar-aware traversal. A single giant query for every nested detail becomes brittle and hard to debug.

Why it matters for Parseltongue:

Parseltongue should use queries as discovery gates, not as the only semantic interpreter. This is especially important for Rust traits, impls, associated types, Java annotations, Python decorators, TypeScript type aliases, Go methods, and Swift extensions.

Rust translation:

```rust
pub trait SymbolExtractor {
    fn discover_symbol_candidates(
        &self,
        tree: &tree_sitter::Tree,
        source: SourceText<'_>,
    ) -> Result<Vec<CandidateNode>, ExtractError>;

    fn refine_candidate_node(
        &self,
        candidate: CandidateNode,
        source: SourceText<'_>,
    ) -> Result<SymbolRecord, ExtractError>;
}
```

When to use:

- Definitions where the outer construct is easy to capture but inner metadata varies.
- Types/classes/enums/traits/interfaces with nested bodies.
- Function signatures where parameters, return types, attributes, decorators, and modifiers need separate handling.

When not to use:

- Do not traverse every node recursively if a focused query can reduce the search space cheaply.

Risks and caveats:

- Field names are grammar-specific and can change.
- Node kind strings are also grammar-specific.
- Some languages encode similar constructs differently: TypeScript classes and interfaces, Rust traits and impls, Python classes and functions, C# partial classes.

Testing implications:

- Test both the candidate query and the refinement traversal separately.
- Golden-test field extraction, variant extraction, and signature extraction.
- Include tests where a candidate is captured but later rejected.

Agent guidance:

When adding a new extractor, generate a small candidate query first, then write named helper functions for field traversal. Avoid one opaque mega-query.

## Pattern 6 - Capture Names as a Cross-Language Contract

Where found:

- Repository: `Aider-AI__aider`
- File: `aider/repomap.py`
- Query assets under `aider/queries/...`
- Repository: `Tomatio13__repo-map-skill`
- Assets: `assets/queries/tree-sitter-languages/*.scm`

Observed shape:

Aider expects capture names such as `name.definition.*` and `name.reference.*`, then turns them into `Tag(..., kind="def")` or `Tag(..., kind="ref")`.

Why this matters:

Capture names are not just query internals. They are the semantic API between grammar-specific query files and language-neutral indexing.

Why it matters for Parseltongue:

Parseltongue needs a capture taxonomy that every language query maps into:

- `name.definition.function`
- `name.definition.type`
- `name.definition.method`
- `name.reference.call`
- `name.reference.import`
- `scope`
- `doc.comment`
- `module.name`

Rust translation:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaptureRole {
    Definition(SymbolKind),
    Reference(ReferenceKind),
    Scope,
    Documentation,
    Modifier,
    Unknown,
}

impl TryFrom<&str> for CaptureRole {
    type Error = CaptureRoleError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Parse "name.definition.function" style capture names.
        todo!()
    }
}
```

When to use:

- Multi-language symbol extraction.
- Repo maps.
- LLM context chunks.
- Query asset validation.

When not to use:

- Do not treat capture taxonomy as a substitute for semantic resolution. A `name.reference` capture is still only a syntactic reference.

Risks and caveats:

- Some query files only define definitions and no references.
- Some references need fallback tokenization, as Aider does with Pygments when references are absent.
- Capture names can be inconsistent across third-party query assets.

Testing implications:

- Lint query assets for allowed capture names.
- Fail CI if a query emits an unknown capture role unless explicitly allowed.
- Snapshot capture role distribution by language.

Agent guidance:

Future codegen should never introduce ad-hoc capture strings. Add roles to a typed enum and update query-lint fixtures.

## Pattern 7 - Exact Byte Spans as Durable Provenance

Where found:

- Repository: `Ataraxy-Labs__sem`
- File: `crates/sem-core/src/model/entity.rs`
- Language: Rust

Observed shape:

`SemanticEntity` stores `start_line`, `end_line`, optional `start_byte`, and optional `end_byte`. The file comments state the byte offsets match Tree-sitter node start and end bytes and allow exact original bytes to be sliced from the source file.

Why this matters:

Line numbers are useful for humans, but byte spans are the durable bridge between syntax trees, source text, hashes, chunks, graph nodes, diagnostics, and edits.

Why it matters for Parseltongue:

LLM companion output must be inspectable. If an agent claims a function was selected for context, it should be possible to slice the exact bytes used and verify the claim.

Rust translation:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub file_id: FileId,
    pub byte_span: ByteSpan,
    pub start_point: SourcePoint,
    pub end_point: SourcePoint,
}
```

When to use:

- Every symbol record.
- Every extracted reference.
- Every chunk.
- Every graph edge with source evidence.
- Every diagnostic or query capture record.

When not to use:

- Do not store byte spans without also knowing which source version they refer to.

Risks and caveats:

- Byte spans become invalid after edits unless tied to a file content hash or version.
- UTF-8 slicing must be checked; not every arbitrary byte span is a valid `&str` boundary.
- Generated code and non-UTF-8 files require separate policies.

Testing implications:

- Roundtrip: extracted span slices exactly equal extracted content.
- Test multi-byte identifiers and comments.
- Test span validity after CRLF normalization decisions.

Agent guidance:

Every generated Parseltongue symbol/chunk/entity model should include a file content identity plus byte span. Do not settle for line-only provenance.

## Pattern 8 - Source Versioning and Structural Hashes

Where found:

- Repository: `Ataraxy-Labs__sem`
- File: `crates/sem-core/src/model/entity.rs`
- Language: Rust

Observed shape:

`SemanticEntity` stores `content_hash` and an optional `structural_hash` described as stripping comments and normalizing whitespace.

Why this matters:

Source text changes frequently. Some changes are semantic, some are formatting. Separating content identity from structural identity enables smarter cache reuse and more meaningful diff analysis.

Why it matters for Parseltongue:

Parseltongue can avoid regenerating expensive symbol summaries or embeddings when only formatting changes, while still preserving exact byte provenance for the current file version.

Rust translation:

```rust
pub struct EntityIdentity {
    pub stable_id: EntityId,
    pub content_hash: ContentHash,
    pub structural_hash: Option<StructuralHash>,
    pub source_version: SourceVersion,
}
```

When to use:

- Symbol cache invalidation.
- LLM summary reuse.
- Chunk identity.
- Diff-aware context selection.

When not to use:

- Do not use structural hash as the only identity for overloaded functions or same-name declarations.

Risks and caveats:

- Structural hashing must be language-aware.
- Removing comments may remove doc semantics.
- Same structure can still mean different behavior after identifier renames.

Testing implications:

- Formatting-only changes should preserve structural hash where intended.
- Semantic changes should change structural hash.
- Doc-comment changes should have a policy: either structural change or separate documentation hash.

Agent guidance:

If generating a cache model, include both exact content hash and optional structural hash. Make the structural hash explicitly best-effort.

## Pattern 9 - Scope-Aware Ownership Beats Bag-of-Words

Where found:

- Repository: `Ataraxy-Labs__sem`
- File: `crates/sem-core/src/parser/scope_resolve.rs`
- Language: Rust

Observed shape:

The module documents a move from bag-of-words tokenization to AST-aware references. It models scopes, definitions, bindings, variable types, call references, scoped calls, and method calls. `ref_owned_by_entity` checks byte ranges and excludes references that fall inside child reference scopes.

Why this matters:

Definition/reference extraction is not enough. A reference belongs to some owning symbol. If ownership is wrong, call graphs and context chunks become misleading.

Why it matters for Parseltongue:

LLM companions need reliable explanations such as "function A calls function B." That requires ownership, scope, and sometimes type inference, not just text search.

Rust translation:

```rust
pub struct ScopeFrame {
    pub parent: Option<ScopeId>,
    pub owner: Option<EntityId>,
    pub definitions: FxHashMap<SymbolName, EntityId>,
    pub local_bindings: FxHashSet<SymbolName>,
}

pub struct AstReference {
    pub kind: ReferenceKind,
    pub owner: Option<EntityId>,
    pub span: SourceSpan,
}
```

When to use:

- Call graph extraction.
- Reference graph extraction.
- Blast-radius analysis.
- LLM explanations that cite relationships.

When not to use:

- Do not attempt perfect language-server-grade resolution before basic syntactic extraction is stable. Ship in tiers.

Risks and caveats:

- Dynamic dispatch, macros, decorators, metaclasses, reflection, and generated code require conservative representation.
- Scope resolution varies dramatically by language.
- Partial classes, extensions, impl blocks, and companion objects complicate ownership.

Testing implications:

- Golden-test nested functions, nested classes, method calls, qualified calls, imports, overloads, and same-name shadowing.
- Include negative tests where text search would produce a false edge.

Agent guidance:

When generating relationship code, separate syntactic reference detection from semantic resolution. Store unresolved references rather than inventing edges.

## Pattern 10 - Hard Limits with Explicit Truncation State

Where found:

- Repository: `Christoph__treesitter-mcp`
- File: `src/extraction/types.rs`
- Language: Rust

Observed shape:

`HARD_TYPE_LIMIT` is set to 1000. `TypeExtractionResult` stores `total_types`, `types_included`, `limit_hit`, and `truncated`.

Why this matters:

Repository-scale extraction can produce more data than a tool response, token budget, or memory budget can safely carry. Silent truncation makes downstream LLM behavior dangerous.

Why it matters for Parseltongue:

Parseltongue must report when context is partial. An agent should know whether it saw every type in a repository or only the first N.

Rust translation:

```rust
pub struct ExtractionResult<T> {
    pub items: Vec<T>,
    pub total_seen: usize,
    pub included: usize,
    pub limit_hit: Option<ExtractionLimitHit>,
}

pub enum ExtractionLimitHit {
    ItemLimit { limit: usize },
    TokenLimit { limit: usize },
    ByteLimit { limit: usize },
    TimeLimit { millis: u64 },
}
```

When to use:

- Tool outputs.
- CLI summaries.
- Repo maps.
- Symbol listings.
- LLM context construction.

When not to use:

- Do not use a hard limit without deterministic ordering. Otherwise repeated runs can hide different symbols.

Risks and caveats:

- Limit order biases results.
- File traversal order must be stable for reproducibility.
- Truncation should be surfaced in both human output and machine-readable output.

Testing implications:

- Force limits in fixtures and assert `limit_hit`.
- Test deterministic ordering under parallel scanning.
- Test that summaries mention truncation.

Agent guidance:

Generated tools should never return a bare list for repository-scale data. Return list plus counts plus truncation reason.

## Pattern 11 - Skip Bad Files, But Preserve Diagnostics

Where found:

- Repository: `Christoph__treesitter-mcp`
- File: `src/extraction/types.rs`
- Language: Rust

Observed shape:

The extractor reads files once, detects language by path, and logs debug messages when a file cannot be read or parsed, continuing repository processing.

Why this matters:

Real repositories contain generated files, binary files, broken files, partial edits, bad encodings, vendored code, and syntax errors. A repository analyzer that fails all work on one bad file is brittle.

Why it matters for Parseltongue:

Agentic coding workflows often run while the worktree is in a broken intermediate state. Parseltongue should still index what it can and explain what it skipped.

Rust translation:

```rust
pub struct FileParseReport {
    pub file_id: FileId,
    pub language: Option<LanguageId>,
    pub status: FileParseStatus,
    pub diagnostics: Vec<ParseDiagnostic>,
}

pub enum FileParseStatus {
    Parsed,
    SkippedUnsupportedLanguage,
    SkippedTooLarge,
    ReadError,
    ParseError,
}
```

When to use:

- Repository sweeps.
- Watch mode.
- CLI indexing.
- Background indexing tasks.

When not to use:

- Do not suppress diagnostics from user-facing modes. Debug-only logging is not enough for a long-lived index.

Risks and caveats:

- Too much tolerance can hide systemic parser failures.
- Unsupported language counts should be visible.

Testing implications:

- Include unreadable file simulation where possible.
- Include syntax-error fixtures.
- Assert index succeeds with partial diagnostics.

Agent guidance:

When generating Parseltongue indexing code, produce a parse report table, not just a success/failure boolean.

## Pattern 12 - Source Slice Access Should Be Zero-Copy Until It Cannot Be

Where found:

- Repository: `Ataraxy-Labs__sem`
- File: `crates/sem-core/src/model/entity.rs`
- Repository: `Christoph__treesitter-mcp`
- File: `src/extraction/types.rs`

Observed shape:

Tree-sitter nodes expose byte spans and `utf8_text(source_bytes)`. `SemanticEntity` stores byte spans so consumers can slice exact original bytes later.

Why this matters:

Copying full source text into every entity, chunk, and edge can be expensive. But not storing enough provenance prevents exact reconstruction.

Why it matters for Parseltongue:

Parseltongue should parse source once, keep source text in a file-content store, and let symbols/chunks reference byte ranges. This keeps LLM context generation exact while avoiding redundant storage.

Rust translation:

```rust
pub struct SourceStore {
    files: DashMap<FileVersionId, Arc<str>>,
}

pub struct SymbolRecord {
    pub id: SymbolId,
    pub span: SourceSpan,
    pub signature_span: Option<SourceSpan>,
    pub body_span: Option<SourceSpan>,
}
```

When to use:

- Large repositories.
- Repeated context construction.
- Embedding pipelines.
- Symbol graph persistence.

When not to use:

- If source may disappear or change, persist a content-addressed copy or cache layer.

Risks and caveats:

- `Arc<str>` implies valid UTF-8. Some source files may require byte storage.
- Byte spans must be checked before slicing.
- Holding many source files in memory can exceed budgets.

Testing implications:

- Test memory usage on fixture repositories.
- Test source eviction and reload.
- Test non-UTF-8 behavior policy.

Agent guidance:

Prefer `SourceText<'a>` views and `SourceSpan` references over cloning snippets into every record. Clone only at API boundaries.

## Pattern 13 - Field-Name Access Is Stronger Than Positional Child Access

Where found:

- Repository: `Christoph__treesitter-mcp`
- File: `src/extraction/types.rs`
- Language: Rust

Observed shape:

Extraction helpers use field-name navigation, for example to locate a body node, rather than assuming child indexes.

Why this matters:

Tree-sitter grammars expose field names for semantically important relationships. Child indexes are more brittle across optional modifiers, attributes, comments, and grammar changes.

Why it matters for Parseltongue:

Symbol extractors should prefer field names such as `name`, `body`, `parameters`, `type`, `declarator`, `value`, and `argument` when grammars provide them.

Rust translation:

```rust
fn required_child_by_field<'tree>(
    node: Node<'tree>,
    field: &'static str,
) -> Result<Node<'tree>, NodeShapeError> {
    node.child_by_field_name(field)
        .ok_or(NodeShapeError::MissingField { field, kind: node.kind().into() })
}
```

When to use:

- Names, bodies, parameter lists, return types, attributes.
- Grammar-specific extractors.

When not to use:

- Not all grammars expose all useful relationships as fields.

Risks and caveats:

- Field names are still grammar contracts, not universal Tree-sitter contracts.
- A missing field may mean grammar drift or a legitimate construct variant.

Testing implications:

- Test every field access with a fixture and an error case.
- Include optional and malformed syntax.

Agent guidance:

Generate named helper functions around field access. Do not scatter string field names across extraction code.

## Pattern 14 - Keep Parser Output and Agent Output Separate

Where found:

- Repository: `Christoph__treesitter-mcp`
- File: `src/extraction/types.rs`
- Repository: `Ataraxy-Labs__sem`
- Files: model and resolver modules

Observed shape:

Runtime parser operations produce domain models such as `TypeDefinition`, `Field`, `Variant`, `Member`, `SemanticEntity`, `AstRef`, and `Scope`; they do not directly produce prose.

Why this matters:

LLM companion systems fail when parsing, indexing, ranking, and summarization are fused into one string-producing pass.

Why it matters for Parseltongue:

Parseltongue should first build durable intermediate records, then create LLM-oriented views from those records.

Rust translation:

```rust
pub struct ParseLayerOutput {
    pub files: Vec<FileParseReport>,
    pub symbols: Vec<SymbolRecord>,
    pub references: Vec<ReferenceRecord>,
}

pub trait ContextRenderer {
    fn render_context(
        &self,
        index: &CodeIndex,
        request: ContextRequest,
    ) -> Result<RenderedContext, RenderError>;
}
```

When to use:

- Any agent-facing command.
- Search, explain, review, refactor, summarize, and test generation flows.

When not to use:

- Tiny one-off CLI prototypes can print directly, but migrate early.

Risks and caveats:

- Intermediate models can become overgrown. Keep layers clear.

Testing implications:

- Unit-test parse output separately from context rendering.
- Snapshot context rendering after stable parse fixtures.

Agent guidance:

Future agents should produce typed records first and prose second.

## Runtime Checklist for Parseltongue

- Use a central `LanguageRegistry`.
- Compile query assets through a `QueryRegistry`.
- Store parser and grammar versions in cache keys.
- Parse source bytes, not lossy text abstractions.
- Preserve `SourceSpan` with file version, byte span, and point span.
- Use queries for coarse candidates and traversal for refinement.
- Prefer field-name access over positional child indexes.
- Report truncation, skipped files, unsupported languages, and parse errors.
- Validate incremental edits against full reparse before enabling watch-mode fast paths.
- Keep parser runtime models separate from LLM context rendering.

## Anti-Patterns Captured

- Silent query fallback that hides grammar drift.
- Line-only symbol provenance.
- Incremental parsing without exact edit metadata.
- Unbounded extraction result lists.
- One giant multi-language query abstraction that ignores grammar-specific shape.
- Bag-of-words call graphs presented as semantic truth.
- Parser reuse without resetting included ranges, timeout, cancellation, or language.
- Treating capture names as ad-hoc strings rather than a typed semantic contract.

## Transferable Design Principle

Tree-sitter gives Parseltongue concrete syntax, not finished understanding. The reliable architecture is a chain:

`source bytes -> parser runtime -> query candidates -> traversal refinement -> source-spanned records -> indexed relationships -> LLM context views`

Each arrow should be typed, tested, versioned, and allowed to fail with diagnostics.
