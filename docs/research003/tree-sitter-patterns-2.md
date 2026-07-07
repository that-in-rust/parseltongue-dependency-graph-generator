# Tree-Sitter Patterns 2: Grammar Integration, Query Assets, Multi-Language Normalization

This file captures grammar-pack, query-asset, and cross-language normalization patterns for Parseltongue.

## Phase 0 - Deconstruct and Clarify

Core objective for this slice: discover how real repositories organize Tree-sitter grammars, `.scm` queries, language packs, editor query assets, tags queries, and language-specific extraction differences. The target is not Rust-only. Rust is the implementation language for Parseltongue, but the reusable evidence comes from Python, Rust, Swift, TypeScript, Neovim ecosystems, editor integrations, and code-assist tools.

Premise is sound. Proceeding with optimized protocol.

Important evidence boundary: the all-repo scan found 609 Git repositories and 6267 `.scm` query files. Direct inspection focused on high-signal repositories and representative files. Therefore, the counts are broad evidence; the patterns are curated evidence.

## Phase 1 - Cognitive Staging

Expert council used for this slice:

- Grammar Cartographer: maps grammar repositories, query families, parser assets, and language-pack conventions.
- Multi-Language Semantics Engineer: normalizes syntax differences into stable concepts such as definition, reference, import, call, type, scope, and comment.
- Rust Packaging Architect: translates language packs and query assets into Rust crate/module boundaries.
- Skeptical Systems Engineer: challenges grammar drift, capture-name inconsistency, generated-file hygiene, and parser ABI compatibility.
- Agentic Retrieval Designer: asks how query assets can produce context that helps future coding agents.

Knowledge scaffolding:

- Tree-sitter grammar shape: `grammar.js`, `src/node-types.json`, generated C/C++ parser, optional scanners, bindings, queries, corpus tests.
- Query families: `highlights.scm`, `injections.scm`, `folds.scm`, `indents.scm`, `locals.scm`, `textobjects.scm`, `tags.scm`.
- Language-pack behavior: extension maps, parser loading, language listing, bundled grammars, version pinning.
- Cross-language concepts: definitions, references, calls, imports, exports, declarations, containers, comments, docstrings, and scopes.
- Rust asset strategy: embedded query assets, per-language modules, typed capture taxonomy, query linting, fixture corpus.

## Phase 2 - Multi-Perspective Synthesis

Conventional approach: bundle a handful of language crates, write a tags query per language, and normalize captures into a simple symbol enum.

Alternative 1 - Library science blend: treat every query family as a cataloging schema. Highlights, locals, injections, folds, textobjects, and tags are different catalog cards for the same source artifact. This teaches Parseltongue not to overfit to tags queries only.

Alternative 2 - Biological taxonomy blend: treat languages as species with homologous organs. A Rust `trait`, TypeScript `interface`, Swift `protocol`, and Go `interface` are not identical, but they occupy related taxonomic slots in a code-intelligence ontology.

Alternative 3 - Cartography blend: treat each grammar query as a map layer. Highlights are terrain, locals are political boundaries, tags are landmarks, injections are hidden tunnels, folds are elevation contours, and textobjects are navigable regions.

Selected path: a library-science and cartography hybrid. Parseltongue should maintain a catalog of query layers per language and compose them into semantic map layers, rather than assuming one query file can answer every code-intelligence question.

Council debate summary:

- Grammar Cartographer: the `.scm` corpus proves that Tree-sitter ecosystems organize knowledge by query purpose, not just by language.
- Skeptical Systems Engineer: query files are often editor-facing, not code-intelligence-facing; borrowing them blindly can import UI assumptions.
- Multi-Language Semantics Engineer response: use editor query assets as evidence and test data, but define a Parseltongue-specific capture taxonomy.
- Rust Packaging Architect response: embed curated query files with version metadata and add a query-lint tool.
- Agentic Retrieval Designer response: preserve the original query layer so agents know whether a capture came from tags, locals, injections, or a custom relation query.

Core thesis: Parseltongue should treat grammar integration as a versioned atlas: every language has multiple query layers, each layer has typed capture roles, and normalization happens through an explicit semantic ontology rather than through accidental capture strings.

## Phase 3 - Verification Anchors

Primary local evidence used:

- Broad `.scm` scan under `git-ref-repo/ignore-this-folder-repos`: 6267 files.
- Top query filenames found: `highlights.scm` 1632, `injections.scm` 1163, `folds.scm` 768, `indents.scm` 618, `locals.scm` 602, `textobjects.scm` 358, `tags.scm` 138.
- Top query-heavy repositories found: `romus204__tree-sitter-manager.nvim`, `arborist-ts__arborist.nvim`, `nvim-treesitter__nvim-treesitter`, `meain__evil-textobj-tree-sitter`, `bearcove__arborium`, `CodeEditApp__CodeEditLanguages`, `zed-industries__zed`, `Aider-AI__aider`, `Tomatio13__repo-map-skill`.
- `wrale__mcp-server-tree-sitter/src/mcp_server_tree_sitter/language/registry.py`
- `Aider-AI__aider/aider/repomap.py`
- `Aider-AI__aider/aider/queries/...`
- `Tomatio13__repo-map-skill/assets/queries/tree-sitter-languages/*.scm`
- `Christoph__treesitter-mcp/src/extraction/types.rs`
- `Wilfred__difftastic/Cargo.toml`
- `CodeGraphContext__CodeGraphContext/website/public/wasm`

Self-correction questions:

- Are `highlights.scm` and `injections.scm` actually the most common `.scm` files in the local corpus? Yes, the local scan reported 1632 and 1163 respectively.
- Is there local evidence for tags-query capture normalization? Yes, Aider consumes `name.definition.*` and `name.reference.*`.
- Is there local evidence for broad language extension maps? Yes, the wrale registry maps many extensions to language identifiers.
- Is there local evidence that grammar-specific extraction diverges by language? Yes, Christoph's extractor branches for Rust, TypeScript/JavaScript, Python, Java, C#, and Go.
- Is every query asset directly suitable for Parseltongue? No. Editor query assets are evidence and inspiration, not automatic production-ready code-intelligence rules.

## Pattern 1 - Query Asset Families Are Separate Knowledge Layers

Where found:

- Repositories: `nvim-treesitter__nvim-treesitter`, `arborist-ts__arborist.nvim`, `romus204__tree-sitter-manager.nvim`, `zed-industries__zed`, `CodeEditApp__CodeEditLanguages`, and others.
- Evidence: 6267 `.scm` files across the local repository set.
- Language/framework stack: Tree-sitter editor integrations, Neovim, Zed, CodeEdit, Emacs-style packages.

Observed families:

```text
highlights.scm      syntax categories for presentation
injections.scm      embedded-language discovery
folds.scm           collapsible structural regions
indents.scm         indentation behavior
locals.scm          scopes, definitions, references, locals
textobjects.scm     navigable semantic regions
tags.scm            def/ref extraction for repo maps
```

Why this matters:

Different query families encode different forms of syntax understanding. A code-intelligence system should not ignore `locals.scm` and `injections.scm` just because `tags.scm` is the obvious repo-map file.

Why it matters for Parseltongue:

Parseltongue can build richer context if it distinguishes:

- visible definitions from lexical locals,
- embedded languages from host-language syntax,
- foldable regions from chunk boundaries,
- textobjects from edit/refactor targets,
- highlights from semantic categories that may help display and debugging.

Rust translation:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryLayer {
    Tags,
    Locals,
    Injections,
    Highlights,
    Folds,
    TextObjects,
    CustomRelations,
}

pub struct QueryAsset {
    pub language: LanguageId,
    pub layer: QueryLayer,
    pub source: &'static str,
    pub version: QueryAssetVersion,
}
```

When to use:

- Query organization.
- Query linting.
- Debug views.
- Language onboarding.
- Chunking and context rendering.

When not to use:

- Do not combine all query files into one mega-query. Layer meaning matters.

Risks and caveats:

- Editor query assets may prioritize UI behavior, not indexing correctness.
- Some captures overlap and conflict across layers.
- Injection queries can recursively expand parsing scope and cost.

Testing implications:

- Compile every query asset.
- Snapshot capture distributions per layer.
- Test that context-building code only consumes allowed layers.

Agent guidance:

When adding a language, create an asset manifest listing query layers. Do not paste query strings inline without layer metadata.

## Pattern 2 - Tags Capture Taxonomy as a Portable Contract

Where found:

- Repository: `Aider-AI__aider`
- File: `aider/repomap.py`
- Repository: `Tomatio13__repo-map-skill`
- Files: `assets/queries/tree-sitter-languages/*-tags.scm`

Observed shape:

Aider maps captures beginning with `name.definition.` to `def` tags and captures beginning with `name.reference.` to `ref` tags.

Representative query shape from the tags-query ecosystem:

```scheme
(function_declarator
  declarator: (qualified_identifier
    scope: (namespace_identifier) @scope
    name: (identifier) @name.definition.method)) @definition.method
```

Why this matters:

The capture name carries semantic intent across languages. The node kind varies, but `name.definition.method` is a portable concept.

Why it matters for Parseltongue:

Parseltongue should use a richer but similarly consistent taxonomy:

```text
name.definition.function
name.definition.method
name.definition.type
name.definition.module
name.reference.call
name.reference.type
name.reference.import
scope.container
scope.namespace
doc.comment
```

Rust translation:

```rust
pub enum SymbolKind {
    Function,
    Method,
    Type,
    Class,
    Struct,
    Enum,
    Trait,
    Interface,
    Module,
    Constant,
    Variable,
}

pub enum CaptureMeaning {
    DefinitionName(SymbolKind),
    ReferenceName(ReferenceKind),
    ScopeName(ScopeKind),
    DefinitionNode(SymbolKind),
}
```

When to use:

- Def/ref extraction.
- Language query linting.
- Repository maps.
- Graph edges.

When not to use:

- Do not force every language construct into a false common category. Preserve original node kind and language-specific subtype.

Risks and caveats:

- Some languages lack clean syntactic references in tags queries.
- A capture may name the identifier while its parent capture names the definition node.
- Third-party query assets can use incompatible capture names.

Testing implications:

- Add a query validator that checks allowed capture prefixes.
- Golden-test one fixture per capture role.
- Include roundtrip from capture to `SymbolKind`.

Agent guidance:

When generating query files, use capture names that map to typed enums. If a new semantic role is needed, add the enum variant first.

## Pattern 3 - Language Registry and Extension Map as Product Surface

Where found:

- Repository: `wrale__mcp-server-tree-sitter`
- File: `src/mcp_server_tree_sitter/language/registry.py`
- Language: Python

Observed shape:

The registry maps extensions to language names, exposes available language listing, checks availability, preloads configured languages, and obtains parsers from `tree_sitter_language_pack`.

Why this matters:

Language support is user-visible. It affects CLI help, error messages, indexing coverage, and agent expectations.

Why it matters for Parseltongue:

Parseltongue should have a single language-support manifest with:

- file extensions,
- known ambiguous extensions,
- parser crate or backend,
- query layers present,
- extraction features present,
- fixture coverage,
- maturity level.

Rust translation:

```rust
pub struct LanguageSupport {
    pub id: LanguageId,
    pub extensions: &'static [&'static str],
    pub filenames: &'static [&'static str],
    pub query_layers: &'static [QueryLayer],
    pub supports_incremental: bool,
    pub supports_injections: bool,
    pub maturity: LanguageMaturity,
}
```

When to use:

- CLI `languages list`.
- Index diagnostics.
- CI coverage matrix.
- Tool output when files are skipped.

When not to use:

- Do not let language support emerge from scattered `match extension` expressions.

Risks and caveats:

- Extension maps do not handle shebangs, editor modelines, generated sources, or embedded code.
- Multiple languages share extensions.

Testing implications:

- Snapshot language-support table.
- Test extension, filename, and shebang detection separately.
- Assert every supported language has at least one fixture and one query compilation test.

Agent guidance:

When adding a language, update the support manifest and tests before implementing extraction details.

## Pattern 4 - Grammar Repositories Have a Reusable Physical Shape

Where found:

- Many local grammar repositories under `git-ref-repo/ignore-this-folder-repos`, including `tree-sitter__tree-sitter` and numerous `tree-sitter-*` grammar repos.
- Evidence from broad file scan: grammar assets such as `grammar.js`, `src/node-types.json`, generated parser sources, query folders, bindings, corpus tests.

Common shape:

```text
grammar.js
src/parser.c
src/scanner.c or src/scanner.cc
src/node-types.json
queries/highlights.scm
queries/injections.scm
test/corpus/*.txt
bindings/rust/
bindings/node/
bindings/python/
```

Why this matters:

Grammar repositories already contain the source of truth for node kinds, fields, corpus examples, and editor query assets.

Why it matters for Parseltongue:

Parseltongue can use grammar repo shape to automate language onboarding:

- discover node types,
- compile bundled queries,
- generate fixture cases,
- detect external scanner presence,
- document supported constructs.

Rust translation:

```rust
pub struct GrammarPackageAudit {
    pub has_grammar_js: bool,
    pub has_node_types_json: bool,
    pub has_external_scanner: bool,
    pub query_layers: BTreeSet<QueryLayer>,
    pub corpus_tests: usize,
    pub bindings: BTreeSet<BindingKind>,
}
```

When to use:

- Evaluating candidate language grammars.
- Creating automated support reports.
- Detecting generated-file drift.

When not to use:

- Do not assume all grammar packages follow the same layout exactly.

Risks and caveats:

- Generated parser files can be stale.
- External scanners carry extra build and safety complexity.
- Node type JSON can change across grammar versions.

Testing implications:

- Validate that checked-in generated parser matches grammar version where possible.
- Run corpus tests for grammars that Parseltongue depends on.
- Snapshot node type fields used by extractors.

Agent guidance:

Future agents should inspect `node-types.json` before writing field-name extraction code for an unfamiliar grammar.

## Pattern 5 - Grammar-Specific Divergence Belongs Behind a Common Trait

Where found:

- Repository: `Christoph__treesitter-mcp`
- File: `src/extraction/types.rs`
- Language: Rust

Observed shape:

The extractor detects a supported language, then dispatches to language-specific functions:

```text
Rust -> extract_rust_types
TypeScript -> extract_typescript_types(..., true)
JavaScript -> extract_typescript_types(..., false)
Python -> extract_python_types
Java -> extract_java_types
CSharp -> extract_csharp_types
Go -> extract_go_types
```

Why this matters:

Multi-language extraction needs shared outputs and language-specific internals. Trying to erase all grammar differences too early causes brittle abstractions.

Why it matters for Parseltongue:

Parseltongue should expose a uniform `LanguageExtractor` trait while letting each implementation use its own queries, node kinds, field names, and fallback rules.

Rust translation:

```rust
pub trait LanguageExtractor: Send + Sync {
    fn language_id(&self) -> LanguageId;
    fn extract_symbols(
        &self,
        file: &ParsedFile<'_>,
        sink: &mut dyn SymbolSink,
    ) -> Result<(), ExtractError>;

    fn extract_references(
        &self,
        file: &ParsedFile<'_>,
        sink: &mut dyn ReferenceSink,
    ) -> Result<(), ExtractError>;
}
```

When to use:

- Type extraction.
- Function extraction.
- Reference extraction.
- Import extraction.
- Chunk boundary selection.

When not to use:

- Do not create a per-language trait hierarchy so deep that cross-language features require boilerplate in every language.

Risks and caveats:

- Shared output models can become too generic.
- Language-specific subtypes need escape hatches.

Testing implications:

- Shared conformance tests for all language extractors.
- Language-specific golden tests for tricky grammar constructs.
- Coverage table for which features each extractor supports.

Agent guidance:

Generate common trait implementations with explicit `UnsupportedFeature` records for missing capabilities.

## Pattern 6 - Type-Like Constructs Need a Cross-Language Ontology

Where found:

- Repository: `Christoph__treesitter-mcp`
- File: `src/extraction/types.rs`
- Language: Rust

Observed shape:

`TypeKind` includes `Struct`, `Class`, `Enum`, `Trait`, `Interface`, `Protocol`, `TypeAlias`, `Record`, `TypedDict`, and `NamedTuple`.

Why this matters:

Languages use different names for related type-level concepts. A code-intelligence system needs both normalized kind and original language-specific kind.

Why it matters for Parseltongue:

This ontology lets agents ask cross-language questions:

- "Show all public types."
- "Find interface-like contracts."
- "Summarize data shapes."
- "Compare trait/protocol/interface usage."

Rust translation:

```rust
pub enum NormalizedTypeKind {
    NominalStruct,
    Class,
    Enum,
    InterfaceLike,
    TraitLike,
    ProtocolLike,
    Alias,
    RecordLike,
    TupleLike,
}

pub struct TypeSymbol {
    pub normalized_kind: NormalizedTypeKind,
    pub language_kind: String,
    pub name: SymbolName,
    pub span: SourceSpan,
}
```

When to use:

- Multi-language type extraction.
- Graph schemas.
- LLM summaries.
- API surface analysis.

When not to use:

- Do not discard the exact language construct. A Rust trait and TypeScript interface have different semantics.

Risks and caveats:

- Over-normalization can produce false equivalence.
- Some constructs are context-dependent.

Testing implications:

- Golden-test normalized and original kind together.
- Include language-specific edge cases such as Rust impl blocks, Swift extensions, C# partial classes, TypeScript declaration merging.

Agent guidance:

When generating user-facing summaries, use normalized kind for grouping and original kind for precision.

## Pattern 7 - Injections Are Embedded-Language Infrastructure

Where found:

- Broad `.scm` corpus: 1163 `injections.scm` files.
- Repositories include Neovim Tree-sitter ecosystems, Zed, CodeEdit, and other editor integrations.

Observed shape:

Injection queries tell Tree-sitter-aware tools where a host language contains embedded code: SQL strings, regex literals, Markdown code fences, template languages, CSS in HTML, JavaScript in HTML, and similar cases.

Why this matters:

LLM code intelligence often fails on embedded languages. SQL-in-strings, shell snippets, HTML templates, and Markdown fences can carry important behavior.

Why it matters for Parseltongue:

Parseltongue can later parse embedded snippets as child documents with parent spans:

```text
host file span -> embedded language -> embedded parse tree -> embedded symbols or diagnostics
```

Rust translation:

```rust
pub struct InjectionRegion {
    pub host_file: FileId,
    pub host_span: SourceSpan,
    pub injected_language: LanguageId,
    pub content_span: SourceSpan,
    pub extraction_policy: InjectionExtractionPolicy,
}
```

When to use:

- Markdown code fences.
- SQL strings.
- HTML/CSS/JS template files.
- Regex analysis.
- Documentation examples.

When not to use:

- Avoid recursively parsing every string literal by default. Require explicit injection rules and budgets.

Risks and caveats:

- Injection parsing can explode cost.
- Embedded snippets may not be standalone valid programs.
- Source mapping back to host file is tricky.

Testing implications:

- Test host-to-injection span mapping.
- Test invalid embedded snippets.
- Test token budget limits for recursive injection parsing.

Agent guidance:

Treat injections as opt-in query layers with budget accounting and provenance to the host source span.

## Pattern 8 - Locals Queries Inform Scope and Chunk Boundaries

Where found:

- Broad `.scm` corpus: 602 `locals.scm` files.
- Editor integrations and Tree-sitter language packages.

Observed shape:

Locals queries typically encode scopes, definitions, references, and local binding structure for editor features.

Why this matters:

Locals are closer to semantic structure than highlights. They can help identify lexical boundaries, symbol visibility, and where references should resolve.

Why it matters for Parseltongue:

Parseltongue can use locals queries as one input to:

- build scope trees,
- prevent child-scope references from being attributed to parent symbols,
- choose chunk boundaries,
- detect local definitions vs exported API.

Rust translation:

```rust
pub struct LocalScopeCapture {
    pub scope_span: SourceSpan,
    pub bindings: Vec<LocalBindingCapture>,
    pub references: Vec<LocalReferenceCapture>,
}
```

When to use:

- Scope-aware reference resolution.
- Chunking inside large files.
- Ranking local context for a selected symbol.

When not to use:

- Do not assume editor locals queries are enough for compiler-grade name resolution.

Risks and caveats:

- Locals query conventions vary across languages.
- Some grammars have incomplete locals queries.

Testing implications:

- Compare locals-based scope boundaries with extractor-owned scope boundaries.
- Test nested functions/classes/blocks.

Agent guidance:

Use locals queries as evidence, not as final truth. Merge them with language-specific scope extraction.

## Pattern 9 - Textobjects and Folds Are Chunk Boundary Hints

Where found:

- Broad `.scm` corpus: 358 `textobjects.scm`, 768 `folds.scm`.
- Repositories: Neovim Tree-sitter ecosystem, textobject plugins, editor integrations.

Observed shape:

Textobjects identify user-navigable regions such as functions, classes, loops, parameters, blocks, comments, and statements. Folds identify collapsible regions.

Why this matters:

LLM chunks should align with human-meaningful code regions. Textobjects and folds are already community-curated hints about those regions.

Why it matters for Parseltongue:

Chunkers can combine:

- symbol spans,
- textobject spans,
- fold spans,
- comment/doc spans,
- import regions,
- dependency edges.

Rust translation:

```rust
pub enum ChunkBoundaryEvidence {
    Symbol(SymbolId),
    TextObject { role: TextObjectRole, span: SourceSpan },
    Fold { span: SourceSpan },
    ImportBlock { span: SourceSpan },
    CommentBlock { span: SourceSpan },
}
```

When to use:

- Function-level chunks.
- Class/module-level chunks.
- Long file summarization.
- Refactor target selection.

When not to use:

- Do not rely on folds alone for semantic chunks. Folds can reflect editor convenience.

Risks and caveats:

- Textobject query naming conventions differ.
- Some files have nested chunks that exceed token budgets.

Testing implications:

- Snapshot chunk boundaries for representative files.
- Test long functions and nested classes.
- Test chunk stability after formatting changes.

Agent guidance:

When generating chunking logic, accept multiple boundary signals and record why a boundary was chosen.

## Pattern 10 - Language Packs Need Versioned Cache Keys

Where found:

- Repository: `Aider-AI__aider`
- File: `aider/repomap.py`
- Repository: `wrale__mcp-server-tree-sitter`
- File: language registry
- Repository: `Wilfred__difftastic`
- File: `Cargo.toml`

Observed shape:

Aider changes `CACHE_VERSION` when using `tree-sitter-language-pack`. Difftastic pins many Tree-sitter parser crates in Cargo dependencies. Wrale loads languages through a language pack and caches language objects.

Why this matters:

The same source file parsed under different grammar versions can produce different node kinds and captures. Caches must know the parser/query version.

Why it matters for Parseltongue:

Persistent symbols, chunks, summaries, embeddings, and graph edges are invalid if grammar or query versions change.

Rust translation:

```rust
pub struct ParseCacheKey {
    pub file_id: FileId,
    pub file_content_hash: ContentHash,
    pub language_id: LanguageId,
    pub grammar_version: GrammarVersion,
    pub query_bundle_version: QueryBundleVersion,
    pub extractor_version: ExtractorVersion,
}
```

When to use:

- Parse caches.
- Tag caches.
- Chunk caches.
- Embedding caches derived from parse output.

When not to use:

- Do not key parse caches only by file path and mtime for durable persisted indexes.

Risks and caveats:

- Version extraction from grammar crates may require manual metadata.
- Query changes can be as important as parser changes.

Testing implications:

- Bump query version and assert stale cache is ignored.
- Change grammar version in a fixture test and assert cache invalidation.

Agent guidance:

If generated code adds a query, parser, or extractor behavior change, update a cache-version constant or content-derived query hash.

## Pattern 11 - Browser/WASM Parser Bundles Enable Visual Debugging

Where found:

- Repository: `CodeGraphContext__CodeGraphContext`
- Path: `website/public/wasm`
- Evidence: `web-tree-sitter.*` and many `tree-sitter-*.wasm` assets found locally.

Why this matters:

Tree-sitter parsing is easier to debug when users can inspect trees in a browser or UI. WASM parser bundles make language support portable across local docs and web views.

Why it matters for Parseltongue:

Parseltongue could expose a debug UI that displays:

- source text,
- syntax tree,
- query captures,
- symbol records,
- chunk boundaries,
- graph edges,
- LLM-selected context.

Rust translation:

```rust
pub struct ParseDebugArtifact {
    pub file: FileId,
    pub language: LanguageId,
    pub tree_sexp: String,
    pub captures: Vec<CaptureDebugRecord>,
    pub symbols: Vec<SymbolRecord>,
    pub chunks: Vec<ChunkRecord>,
}
```

When to use:

- Query development.
- Language onboarding.
- Debugging missed captures.
- Explaining index output to users.

When not to use:

- WASM debug support should not be required for core CLI indexing.

Risks and caveats:

- Native and WASM parsers must be version-aligned.
- Large files can overwhelm browser visualizers.

Testing implications:

- Debug artifacts should be deterministic and snapshot-friendly.
- Do not snapshot giant full trees by default; allow focused ranges.

Agent guidance:

Future agents should create debug artifacts whenever extraction behavior changes significantly.

## Pattern 12 - Generated Grammar Assets Need Hygiene Rules

Where found:

- Broad grammar repository shapes.
- Difftastic packaging references to parser and query assets.
- Tree-sitter grammar repositories with generated parser sources.

Why this matters:

Grammar-generated files can drift from grammar definitions. Generated parser code and query assets are not ordinary handwritten code.

Why it matters for Parseltongue:

If Parseltongue vendors grammar assets or generated query metadata, it needs clear rules:

- what is generated,
- what is edited by humans,
- what must be regenerated on version bumps,
- what is included in packages,
- what is tested.

Rust translation:

```text
crates/parseltongue-languages/
  build.rs
  src/registry.rs
  queries/
    rust/tags.scm
    rust/locals.scm
    typescript/tags.scm
  fixtures/
    rust/basic.rs
    typescript/basic.ts
  generated/
    node_types/
```

When to use:

- Bundled query assets.
- Generated node-type enums.
- Grammar metadata reports.

When not to use:

- Do not hand-edit generated files without documenting it.

Risks and caveats:

- Generated artifacts can create noisy diffs.
- Updating grammar crates can require query updates.

Testing implications:

- CI should verify generated assets are fresh if generation is part of the workflow.
- Query compilation tests should run after dependency updates.

Agent guidance:

When modifying generated grammar metadata, run the generator and include only intentional diff artifacts.

## Pattern 13 - Multi-Language Normalization Must Preserve Language-Specific Escape Hatches

Where found:

- `Christoph__treesitter-mcp` type ontology.
- `Ataraxy-Labs__sem` language-specific scope resolver config.
- `CodeGraphContext__CodeGraphContext/CGC_GRAPH_INCONSISTENCIES.md` cross-language audit gaps.

Observed problem classes from local evidence:

- Rust traits, impls, functions, methods, macros.
- Go interface implementation edges.
- C enum members and callback references.
- C# partial classes.
- Kotlin companion objects.
- Swift protocols and extensions.
- Python decorators, metaclasses, dynamic dispatch, nested calls.
- TypeScript decorators and dynamic imports.

Why this matters:

Cross-language systems fail when they pretend every language has the same semantic model.

Why it matters for Parseltongue:

Parseltongue should store:

- normalized kind,
- language-specific kind,
- original node kind,
- capture role,
- confidence,
- unresolved reasons.

Rust translation:

```rust
pub struct NormalizedSymbol {
    pub normalized_kind: SymbolKind,
    pub language_kind: String,
    pub tree_sitter_node_kind: String,
    pub confidence: ExtractionConfidence,
    pub caveats: Vec<ExtractionCaveat>,
}
```

When to use:

- Public graph schema.
- LLM summaries.
- Cross-language search filters.

When not to use:

- Do not collapse a language-specific feature into a generic kind if the distinction matters for behavior.

Risks and caveats:

- Too many escape hatches can make the model hard to query.
- Too few escape hatches create false facts.

Testing implications:

- Cross-language golden matrix.
- Per-language tricky construct fixtures.
- Regression tests for every previously observed inconsistency.

Agent guidance:

When unsure, generate conservative records with caveats instead of confident false equivalences.

## Parseltongue Query Bundle Recommendation

Suggested directory shape:

```text
crates/parseltongue-query-assets/
  src/lib.rs
  assets/
    rust/
      tags.scm
      locals.scm
      chunks.scm
      injections.scm
    typescript/
      tags.scm
      locals.scm
      chunks.scm
    python/
      tags.scm
      locals.scm
      chunks.scm
  fixtures/
    rust/
    typescript/
    python/
  tests/
    compile_all_queries.rs
    capture_contract.rs
    fixture_captures.rs
```

Recommended invariants:

- Every query asset compiles.
- Every capture name maps to a typed role.
- Every supported language has at least one fixture.
- Every fixture has expected capture snapshots.
- Every query layer has a documented consumer.
- Every parser/query/extractor version participates in cache keys.

## Anti-Patterns Captured

- Treating editor highlight queries as semantic truth.
- Using third-party capture names without a compatibility layer.
- Scattering extension maps across code.
- Adding parser crates without fixture coverage.
- Assuming `tags.scm` is enough for code intelligence.
- Ignoring injections and embedded languages.
- Collapsing language-specific constructs into over-broad categories.
- Cache keys that omit grammar/query/extractor versions.

## Transferable Design Principle

Tree-sitter language support is an atlas, not a lookup table. Parseltongue should maintain layered maps per language: tags for landmarks, locals for borders, injections for embedded regions, textobjects and folds for chunk geometry, and custom relation queries for code intelligence. The atlas must be versioned, validated, and translated through a typed semantic ontology.
