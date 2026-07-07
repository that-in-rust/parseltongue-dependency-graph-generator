# Tree-sitter Reference For Parceltongue - 2026-06

This is a living, repo-by-repo reference for Tree-sitter and similar parser/indexer patterns found under `git-ref-repo/`.

The purpose is not to admire Tree-sitter in the abstract. The purpose is to build a practical encyclopedia for Parceltongue: when Parceltongue needs to parse code, extract symbols, build dependency graphs, answer agent queries, or pack low-token context, this file should contain the patterns and source-backed examples worth copying or adapting.

## Scope And Method

- Reference root inspected: `git-ref-repo/`
- Cloned repos discovered so far: `608`
- Broad signal scan: `57,472` files matched Tree-sitter, parser, code graph, MCP, dependency graph, call graph, or adjacent code-intelligence terms.
- `codebase-memory-mcp` evidence pass started with `Christoph__treesitter-mcp`.
  - Indexed nodes: `2,498`
  - Indexed edges: `12,113`
  - Index output: `/tmp/codex-code-intel/codebase-memory/Christoph__treesitter-mcp-20260706-210718`
- Exact compiler implementations are excluded as compiler design references. Language grammar repos are still useful when they show grammar, query, binding, incremental parsing, or AST traversal patterns.

This file is written concept by concept. Each section includes:

- what the concept is
- where it was seen
- why it matters for Parceltongue
- concrete code or pseudocode patterns to steal
- traps and design cautions

## Concept 1: Treat Tree-sitter As A Parse Service Boundary

### The Big Idea

Do not scatter `Parser::new()`, `set_language()`, and ad hoc AST walking throughout Parceltongue.

The reusable architecture pattern is:

```text
file path or language id
  -> LanguageRegistry
  -> ParserPool or ParserFactory
  -> ParseArtifact
  -> Extractor(s)
  -> Agent-facing facts and graph edges
```

Tree-sitter should be hidden behind a small internal boundary. The rest of Parceltongue should consume stable domain objects: symbols, ranges, edges, imports, call sites, tests, route handlers, chunks, and graph relationships.

That boundary matters because agents do not want raw syntax trees. Agents want compact answers:

- "What symbol is at this line?"
- "What calls this function?"
- "What should I read next?"
- "What code moves if I edit this node?"
- "Which files are probably relevant under a 4K token budget?"

Tree-sitter is the parse engine. It is not the product API.

### Evidence From Repos

| Repo | Source | Pattern Seen | Why Parceltongue Should Care |
|---|---|---|---|
| `Christoph__treesitter-mcp` | `src/parser/mod.rs:10-64` | `Language` enum maps supported languages to Tree-sitter grammars. | Central registry prevents language support from being spread across tools. |
| `Christoph__treesitter-mcp` | `src/parser/mod.rs:109-135` | Extension-based language detection returns typed `Language` or clear error. | Parceltongue needs explicit unsupported-language behavior instead of silent empty results. |
| `Christoph__treesitter-mcp` | `src/parser/mod.rs:167-187` | `parse_code(source, language)` creates parser, sets language, parses, warns on syntax errors, returns `Tree`. | Good minimal facade, but parser creation per call is probably too expensive for a large codebase indexer. |
| `sdsrss__code-graph-mcp` | `src/parser/treesitter.rs:27-54` | Thread-local `PARSER_CACHE` keyed by language; parser timeout set before parsing; parser reset on parse failure. | Stronger service pattern for long-running indexing and agent tools. |
| `sdsrss__code-graph-mcp` | `src/parser/languages.rs:3-27` | `get_language(name) -> Option<tree_sitter::Language>` centralizes grammar lookup for many languages. | Good registry shape for Parceltongue's multi-language CLI and MCP tools. |
| `sdsrss__code-graph-mcp` | `src/parser/lang_config.rs:1-14` | Per-language config explains which behavior cannot be solved by config alone, especially call extraction. | Important humility pattern: some languages require dedicated relation extraction, not one universal string table. |
| `Christoph__treesitter-mcp` | `src/analysis/file_shape.rs:83-100`, `161-180` | Query compilation plus `QueryCursor::matches` extracts functions, classes, imports. | Good for high-level shape extraction, fast summaries, and token-efficient file overview. |
| `tree-sitter__py-tree-sitter` | `examples/usage.py:119-139` | Tree edit plus reparsing with old tree, then `changed_ranges`. | Foundation for incremental Parceltongue indexing after edits. |
| `tree-sitter__py-tree-sitter` | `examples/usage.py:142-167` | Query object, capture names, and match groups over one root node. | Useful distinction: captures are good for flat symbol lists; matches are better for structured facts. |
| `tree-sitter__node-tree-sitter` | `src/parser.cc:169-195`, `209-244` | Included ranges, old-tree reparsing, parse options/progress callback. | Useful for embedded languages, partial parsing, cancellation, and editor-scale responsiveness. |
| `smacker__go-tree-sitter` | `README.md:94-136` | Query predicates like `#match?` and predicate filtering over captures. | Valuable for naming conventions and filtering noisy symbol candidates before they become graph facts. |

### Concrete Pattern To Steal

Parceltongue should define an internal parse service that owns parser lifecycle and returns stable parse artifacts.

```rust
pub struct ParseArtifact {
    pub language: SupportedLanguage,
    pub tree: tree_sitter::Tree,
    pub source: String,
    pub had_error_nodes: bool,
}

pub trait ParseService {
    fn parse_source_text(
        &self,
        source: &str,
        language: SupportedLanguage,
    ) -> anyhow::Result<ParseArtifact>;
}
```

Then every extractor should consume `ParseArtifact` instead of creating its own parser:

```rust
pub trait SymbolExtractor {
    fn extract_symbols_from_artifact(
        &self,
        artifact: &ParseArtifact,
    ) -> anyhow::Result<Vec<CodeSymbol>>;
}
```

The main point is not the exact trait names. The point is ownership:

- `ParseService` owns parser creation, cache, timeout, old-tree reuse, and language setup.
- `Extractor` owns language-specific syntax knowledge.
- `GraphBuilder` owns relationships between extracted facts.
- `ContextTool` owns what gets exposed to the LLM under a token budget.

### Parser Lifecycle Pattern

`Christoph__treesitter-mcp` shows the smallest working shape:

```rust
let mut parser = Parser::new();
parser.set_language(&language.tree_sitter_language())?;
let tree = parser.parse(source, None).ok_or_else(...)?;
```

That is fine for a CLI command that parses one file. For Parceltongue, prefer the `sdsrss__code-graph-mcp` style:

```rust
thread_local! {
    static PARSER_CACHE: RefCell<HashMap<String, tree_sitter::Parser>> =
        RefCell::new(HashMap::new());
}
```

Why this is better for agentic code assist:

- indexing a repo means parsing thousands of files
- MCP tools may parse repeatedly during one session
- parser creation and grammar setup should not be repeated needlessly
- timeouts must be enforced centrally
- failed parses should reset parser state centrally

Recommended Parceltongue design:

```text
ParserPool
  key: SupportedLanguage
  value: tree_sitter::Parser
  policy:
    - set language once
    - set parse timeout once
    - parse with optional old tree
    - reset on failure
    - report syntax-error presence separately from hard parse failure
```

### Language Registry Pattern

Two good registry forms appear in the repos:

1. `Christoph__treesitter-mcp` uses a typed enum:

```rust
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Html,
    Css,
    Swift,
    CSharp,
    Java,
    Go,
}
```

2. `sdsrss__code-graph-mcp` uses string lookup:

```rust
pub fn get_language(name: &str) -> Option<Language> {
    match name {
        "typescript" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "javascript" | "jsx" => Some(tree_sitter_javascript::LANGUAGE.into()),
        // ...
        _ => None,
    }
}
```

For Parceltongue, the best form is probably both:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Rust,
    Python,
    TypeScript,
    Tsx,
    JavaScript,
    Go,
    Java,
    C,
    Cpp,
}
```

Then provide adapters:

```text
file extension -> SupportedLanguage
string id       -> SupportedLanguage
SupportedLanguage -> tree_sitter::Language
SupportedLanguage -> LanguageExtractionConfig
```

This gives the CLI/MCP layer string ergonomics while keeping the core typed.

### Query Execution Pattern

`Christoph__treesitter-mcp` shows the classic high-level extraction idiom:

```rust
let query = Query::new(&tree_sitter_rust::LANGUAGE.into(), query_src)?;
let mut cursor = QueryCursor::new();
let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

while let Some(match_) = matches.next() {
    for capture in match_.captures {
        match query.capture_names()[capture.index as usize] {
            "func.name" => { /* extract name and line */ }
            "struct.name" | "enum.name" | "trait.name" => { /* extract type */ }
            "use" => { /* extract import */ }
            _ => {}
        }
    }
}
```

Parceltongue should make query usage explicit by purpose:

```text
queries/
  rust/
    symbols.scm
    imports.scm
    calls.scm
    tests.scm
  python/
    symbols.scm
    imports.scm
    calls.scm
  typescript/
    symbols.scm
    imports.scm
    calls.scm
    routes.scm
```

Do not mix every extraction concern into one query. Agent-facing tools need composable answers:

- shape-only query for cheap orientation
- symbol query for search
- import query for dependencies
- call query for graph edges
- test query for likely verification path
- edit-scope query for minimal context

### Captures Versus Matches

`tree-sitter__py-tree-sitter/examples/usage.py` demonstrates both captures and matches.

Use captures when the output is a flat list:

```text
all function names
all import statements
all identifiers matching a predicate
```

Use matches when the output is a structured relation:

```text
function name plus function body
call target plus argument list
route method plus route path plus handler node
class name plus method declarations
```

This matters for graph correctness. If Parceltongue extracts a call edge, it should preserve the match-level grouping that proved source and target belong together. Flat captures can accidentally combine unrelated captures from different matched patterns.

### Incremental Parsing And Changed Ranges

The official Python binding example shows the ideal incremental edit sequence:

```text
old tree
  -> tree.edit(...)
  -> parser.parse(new_source, old_tree)
  -> old_tree.changed_ranges(new_tree)
```

The Node binding source also exposes the production concerns around this:

- parser can receive an old tree
- parse can be restricted to included ranges
- parse can use progress/cancellation options

For Parceltongue, this is the path to fast agent feedback:

```text
on file edit:
  1. apply byte/point edit to old tree
  2. parse new source with old tree
  3. compute changed ranges
  4. invalidate only symbols/edges overlapping changed ranges
  5. rerun extractors only for affected regions when possible
```

This should eventually power "what changed semantically?" and "what context must the agent reread?" tools.

### Included Ranges Pattern

`tree-sitter__node-tree-sitter/src/parser.cc` validates included ranges before parsing and rejects overlapping ranges.

This is important for:

- markdown with fenced code blocks
- MDX
- Vue/Svelte/Astro style mixed-language files
- comments or doc blocks containing code examples
- partial parsing after a narrow edit

Parceltongue should represent included ranges as a first-class parse request field:

```rust
pub struct ParseRequest<'a> {
    pub source: &'a str,
    pub language: SupportedLanguage,
    pub old_tree: Option<&'a tree_sitter::Tree>,
    pub included_ranges: Option<Vec<tree_sitter::Range>>,
}
```

The service should validate:

- ranges are sorted
- ranges do not overlap
- range byte offsets and points agree with source
- empty range lists are rejected or explicitly mean "parse all"

### Query Predicate Pattern

`smacker__go-tree-sitter` documents predicate filtering with `#match?`, for example matching identifiers by regex.

For Parceltongue, predicates are useful when extracting agent-grade facts:

```scheme
((identifier) @constant
 (#match? @constant "^[A-Z][A-Z_]+$"))
```

Possible uses:

- detect exported constants
- detect test names
- detect framework route functions
- filter generated-looking identifiers
- enforce naming conventions
- find symbols with AI-native naming signals

But predicates should not become business logic soup. Put reusable semantic filters in Rust after query extraction when they need cross-node context, file path context, or graph context.

### Parceltongue Architecture Recommendation

Build these layers:

```text
SupportedLanguage
  - enum plus extension and string-id adapters

LanguageRegistry
  - maps SupportedLanguage to tree_sitter::Language
  - maps SupportedLanguage to extraction config
  - exposes supported extensions for CLI/MCP discovery

ParserPool
  - one parser per language per worker/thread
  - central timeout
  - optional old-tree parse
  - optional included ranges
  - reset on hard failure

ParseArtifact
  - tree
  - source handle
  - language
  - file id/path
  - syntax error summary
  - parse timing

ExtractorRegistry
  - symbol extractor
  - import extractor
  - call extractor
  - test extractor
  - route/framework extractor
  - doc/comment extractor

GraphBuilder
  - creates nodes and edges from extracted facts
  - keeps relation provenance: file, byte range, line range, query/extractor id

AgentContextTools
  - answer narrow questions from graph and parse artifacts
  - return dependency-aware context under token budgets
```

### Design Cautions

1. Do not make `LanguageConfig` pretend every language is the same.
   - `sdsrss__code-graph-mcp/src/parser/lang_config.rs:5-14` explicitly says call-expression handling cannot be reduced to one config field.
   - Parceltongue should use config for flags and dedicated extractor modules for language-specific relation logic.

2. Do not treat syntax errors as total parse failure.
   - Tree-sitter often returns a tree with error nodes.
   - Parceltongue should store `had_error_nodes` and still extract safe facts when possible.

3. Do not lose source byte ranges.
   - Every symbol and edge should carry byte range and line range.
   - Without provenance, agent tools cannot show exact context or invalidate incrementally.

4. Do not expose raw ASTs as the main agent API.
   - Raw AST is too verbose.
   - Agent APIs should expose shape, path, relation, and minimal snippets.

5. Do not compile queries repeatedly in hot paths.
   - Query compilation should be cached by `(language, query_kind, query_version)`.

### First Parceltongue API Sketch

```rust
pub enum QueryKind {
    FileShape,
    Symbols,
    Imports,
    Calls,
    Tests,
    Routes,
}

pub struct ExtractedFact {
    pub kind: String,
    pub name: String,
    pub qualified_name: Option<String>,
    pub file_path: String,
    pub byte_range: std::ops::Range<usize>,
    pub line_range: std::ops::Range<u32>,
    pub source_excerpt: Option<String>,
    pub extractor_id: String,
}

pub trait CodeFactExtractor {
    fn query_kind(&self) -> QueryKind;
    fn extract_facts(&self, artifact: &ParseArtifact) -> anyhow::Result<Vec<ExtractedFact>>;
}
```

This is the core adapter between Tree-sitter and Parceltongue's graph. If this boundary is good, Parceltongue can support more languages, more agent tools, and more graph relationships without rewriting the parser layer.

### Repos Already Touched For This Concept

| Repo | Status | Notes |
|---|---|---|
| `Christoph__treesitter-mcp` | Source-read plus `codebase-memory-mcp` indexed | Best first example of Tree-sitter exposed as MCP-ish code-intelligence tools. |
| `sdsrss__code-graph-mcp` | Source-read | Strong parser cache, timeout, language config, and extraction pipeline patterns. |
| `tree-sitter__py-tree-sitter` | Source-read | Clear official examples for edit/reparse/changed-ranges and captures vs matches. |
| `tree-sitter__node-tree-sitter` | Source-read | Included ranges, old-tree parsing, parse options/progress callback. |
| `smacker__go-tree-sitter` | Source-read | Query predicates and Go binding usage pattern. |

## Concept 2: Query Packs Are Product Surfaces, Not Just Parser Plumbing

### The Big Idea

Tree-sitter query files are not just editor highlighting config. They are reusable semantic contracts.

The important pattern from editor-grade projects is that query files are split by purpose:

```text
queries/<language>/
  highlights.scm
  injections.scm
  locals.scm
  folds.scm
  indents.scm
  textobjects.scm
  outline.scm
  runnables.scm
  redactions.scm
```

For Parceltongue, the same pattern should become:

```text
queries/<language>/
  shape.scm
  symbols.scm
  imports.scm
  calls.scm
  tests.scm
  routes.scm
  injections.scm
  redactions.scm
  minimal_context.scm
```

Each query group should answer one agent question. This keeps context retrieval token-efficient and makes graph provenance much clearer.

### Evidence From Repos

| Repo | Source | Pattern Seen | Why Parceltongue Should Care |
|---|---|---|---|
| `nvim-treesitter__nvim-treesitter` | `CONTRIBUTING.md:72-94` | Query kinds are documented separately: highlights, injections, folds, locals, indents; query lint/check/format workflow exists. | Parceltongue should version and test query packs like code, not bury them as strings. |
| `nvim-treesitter__nvim-treesitter` | `CONTRIBUTING.md:96-111` | Query inheritance via `; inherits:` and standardized formatting. | Useful for TypeScript inheriting JavaScript patterns, TSX inheriting JSX/HTML-ish patterns, C++ inheriting C-ish patterns. |
| `nvim-treesitter__nvim-treesitter` | `README.md:185-187` | Query lookup under `queries/<language>` with precedence and `; extends`. | Parceltongue can support built-in queries plus project/user overrides. |
| `nvim-treesitter__nvim-treesitter` | `CONTRIBUTING.md:300-333` | Predicate performance order and priority/order guidance. | Query predicates can be expensive; prefer cheap literal predicates and pattern ordering before regex. |
| `nvim-treesitter__nvim-treesitter` | `runtime/queries/rust/highlights.scm:44-89` | Rust function definitions and calls are captured separately. | Captures like `@function` vs `@function.call` are directly useful for symbol and call extraction. |
| `nvim-treesitter__nvim-treesitter` | `runtime/queries/rust/locals.scm:1-98` | Imports, definitions, references, scopes are captured as `@local.definition.*`, `@local.reference`, `@local.scope`. | This is the closest editor query pattern to dependency graph construction. |
| `nvim-treesitter__nvim-treesitter` | `runtime/queries/rust/injections.scm:1-88` | Macro bodies, regex strings, comments, and token trees are captured as injected languages. | Parceltongue can parse embedded code and doc examples instead of treating them as inert strings. |
| `nvim-treesitter__nvim-treesitter-textobjects` | `queries/rust/textobjects.scm:1-220` | Functions, classes, calls, loops, blocks, comments, and parameters expose inner/outer regions. | Excellent source for minimal edit context and "give the agent just this enclosing unit." |
| `nvim-treesitter__nvim-treesitter-textobjects` | `README.md:75-91`, `125-142` | Consumers pass capture name plus query group, and can reuse captures from `locals.scm` or `folds.scm`. | Parceltongue tools should accept semantic capture groups, not raw AST node kinds. |
| `CodeEditApp__CodeEditLanguages` | query inventory | 178 `.scm` resources: highlights, injections, folds, locals, indents, tags, structure. | Shows a packaged language bundle model suitable for a Rust desktop/code-assist tool. |
| `zed-industries__zed` | query inventory plus Rust files | 149 `.scm` files including `outline.scm`, `runnables.scm`, `redactions.scm`, `textobjects.scm`. | These are directly agentic concepts: outline for orientation, runnables for verification, redactions for safe context. |
| `zed-industries__zed` | `crates/grammars/src/rust/outline.scm:5-81` | Captures outline `@item`, `@name`, `@context`, `@open`, `@close`. | A compact file outline query can drive "what should I read first?" tools. |
| `zed-industries__zed` | `crates/grammars/src/rust/runnables.scm:1-75` | Captures Rust tests, doc tests, `main`, and tags them with `#set! tag ...`. | Parceltongue should expose likely verification commands/tests beside code facts. |
| `zed-industries__zed` | `crates/grammars/src/json/redactions.scm:1-10` | Captures JSON string/number values as `@redact`. | Redaction queries can protect secrets before context is sent to an LLM. |
| `bearcove__arborium` | `ADDING_GRAMMARS.md:80-88` | Grammar, queries, and samples are bundled together. | Every query pack should have samples that prove captures still match. |
| `bearcove__arborium` | `crates/arborium-plugin-runtime/src/lib.rs:130-179` | Concatenates injections, locals, highlights into one query, tracks pattern index offsets, records injection capture indices. | Useful if Parceltongue wants one compiled query per language while still preserving group identity. |
| `bearcove__arborium` | `crates/arborium-highlight/src/render.rs:45-61`, `185-205` | Later query patterns win during dedupe. | Query order matters. Parceltongue must treat query file order as part of semantics. |
| `Wilfred__difftastic` | `src/parse/tree_sitter_parser.rs:120-180`, `1368-1395` | Each language has a Tree-sitter config with highlight query and capture-name prefix handling. | Capture namespaces like `constant.builtin` should collapse into broader semantic buckets when needed. |

### Query Inventory From The Current Sweep

These are not all the repos with queries, but they are high-signal examples already inspected:

| Repo | `.scm` Count | Dominant Query Names |
|---|---:|---|
| `nvim-treesitter__nvim-treesitter` | 1,182 | `highlights.scm`, `injections.scm`, `folds.scm`, `indents.scm`, `locals.scm` |
| `nvim-treesitter__nvim-treesitter-textobjects` | 79 | `textobjects.scm` |
| `CodeEditApp__CodeEditLanguages` | 178 | `highlights.scm`, `injections.scm`, `folds.scm`, `locals.scm`, `indents.scm`, `tags.scm`, `structure.scm` |
| `zed-industries__zed` | 149 | `highlights.scm`, `injections.scm`, `brackets.scm`, `indents.scm`, `outline.scm`, `textobjects.scm`, `runnables.scm`, `redactions.scm` |
| `bearcove__arborium` | 204 | `highlights.scm`, `injections.scm`, `locals.scm`, inherited query files, generated query constants |
| `Wilfred__difftastic` | 32 | vendored `highlights.scm`, `locals.scm`, `injections.scm`, `folds.scm`, and per-language highlight query constants |

### Capture Names Are The Contract

A capture name is a semantic API.

Examples from the inspected repos:

```scheme
@function
@function.call
@local.definition.function
@local.definition.import
@local.definition.type
@local.reference
@local.scope
@function.outer
@function.inner
@call.outer
@call.inner
@injection.content
@injection.language
@run
@redact
```

This gives Parceltongue a strong design rule:

```text
AST node kind = grammar implementation detail
capture name  = Parceltongue semantic contract
```

For example, a Rust function might be a `function_item`, a Python function might be a `function_definition`, and a JavaScript function might be a `function_declaration`, `function_expression`, or `arrow_function`. The agent-facing fact should still be:

```text
kind: function
name: ...
range: ...
source_language: ...
extractor: symbols.scm
```

### Recommended Parceltongue Query Pack Layout

```text
crates/parseltongue-tree-sitter/
  queries/
    rust/
      shape.scm
      symbols.scm
      imports.scm
      calls.scm
      tests.scm
      runnables.scm
      injections.scm
      redactions.scm
      minimal_context.scm
    python/
      shape.scm
      symbols.scm
      imports.scm
      calls.scm
      tests.scm
      injections.scm
      redactions.scm
      minimal_context.scm
    typescript/
      shape.scm
      symbols.scm
      imports.scm
      calls.scm
      tests.scm
      routes.scm
      injections.scm
      redactions.scm
      minimal_context.scm
```

The query file names should map to product questions:

| Query Group | Agent Question | Example Captures |
|---|---|---|
| `shape.scm` | "What is in this file?" | `@item`, `@name`, `@context`, `@open`, `@close` |
| `symbols.scm` | "What symbols are defined here?" | `@definition.function`, `@definition.type`, `@definition.method` |
| `imports.scm` | "What does this file depend on?" | `@import.path`, `@import.alias`, `@import.item` |
| `calls.scm` | "What code does this call?" | `@call.function`, `@call.receiver`, `@call.arguments` |
| `tests.scm` | "What tests exist or cover this?" | `@test.name`, `@test.body`, `@test.attribute` |
| `runnables.scm` | "What can the agent run?" | `@run`, `@run.name`, `tag=rust-test` |
| `routes.scm` | "What external API endpoints exist?" | `@route.method`, `@route.path`, `@route.handler` |
| `injections.scm` | "What embedded language should be parsed too?" | `@injection.content`, `@injection.language` |
| `redactions.scm` | "What should never be sent raw to an LLM?" | `@redact`, `@secret.key`, `@secret.value` |
| `minimal_context.scm` | "What is the smallest enclosing editable unit?" | `@function.outer`, `@function.inner`, `@class.outer`, `@call.outer` |

### Query Registry Pattern

Do not hardcode query strings in random extractor functions. Make query packs load through a registry.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryGroup {
    Shape,
    Symbols,
    Imports,
    Calls,
    Tests,
    Runnables,
    Routes,
    Injections,
    Redactions,
    MinimalContext,
}

pub struct CompiledQuerySpec {
    pub language: SupportedLanguage,
    pub group: QueryGroup,
    pub source: &'static str,
    pub query: tree_sitter::Query,
    pub capture_names: Vec<String>,
}
```

Cache key:

```text
(SupportedLanguage, QueryGroup, query_version)
```

The version can be a content hash of the `.scm` source. That makes query changes observable in incremental indexing and test snapshots.

### Query Groups Should Stay Separate

Arborium concatenates injections, locals, and highlights into one query and then tracks pattern index offsets. That is clever for a highlighter runtime because it wants one query execution pipeline.

For Parceltongue, start with separate compiled queries by group:

```text
symbols query   -> CodeSymbol facts
imports query   -> Import facts
calls query     -> CallEdge candidates
tests query     -> Test facts
redactions query -> ContextRedaction spans
```

Later, if profiling says query execution overhead is high, use the Arborium pattern:

```text
concatenate query sources
remember byte/pattern offsets for each group
execute one Query
dispatch matches by pattern_index range
```

But do not do that first. Group separation makes correctness easier and evidence clearer.

### Query Inheritance And Overrides

`nvim-treesitter` supports inheritance and extension:

```scheme
; inherits: javascript
```

and extension:

```scheme
;; extends
```

Parceltongue should steal this idea for language families:

```text
typescript imports base javascript queries
tsx imports typescript plus jsx/html-ish queries
cpp imports some c patterns but overrides function declarators
python framework query packs extend base python
```

Possible query pack resolution order:

```text
1. built-in base language queries
2. built-in language-family extensions
3. framework packs, e.g. react, fastapi, axum, rails
4. project-local `.parseltongue/queries/<language>/*.scm`
5. user-local override queries
```

This would let a solo power user teach Parceltongue project-specific framework patterns without recompiling the whole tool.

### Predicate Discipline

The inspected repos use predicates heavily:

```scheme
((identifier) @constant
 (#lua-match? @constant "^[A-Z][A-Z%d_]*$"))

((call_expression
  function: (scoped_identifier
    path: (identifier) @_regex
    (#any-of? @_regex "Regex" "RegexBuilder")
    name: (identifier) @_new
    (#eq? @_new "new"))
  arguments: (arguments
    (raw_string_literal
      (string_content) @injection.content)))
 (#set! injection.language "regex"))
```

Parceltongue rule:

```text
Use query predicates for cheap, local node-text filtering.
Use Rust code for cross-node, cross-file, framework, graph, or path-aware decisions.
```

Good predicate uses:

- `#eq?` for exact function/macro names
- `#any-of?` for small known lists
- `#match?` for naming conventions
- `#set!` for static metadata like `tag rust-test` or `injection.language regex`

Bad predicate uses:

- encoding a full import resolver in query predicates
- encoding framework semantics that require file/package context
- huge regexes that run across thousands of captures
- query patterns that become unreadable test substitutes

### Pattern Order Is Semantic

Arborium documents and implements "later pattern wins" when deduplicating exact ranges. `nvim-treesitter` also advises trying pattern order before explicit priority.

For Parceltongue:

- preserve query file order in compiled query behavior
- snapshot test query output when order changes
- document any intentional override at the query line
- avoid large catch-all patterns near the end unless they are meant to override

This matters for graph facts too. A broad `identifier @reference` pattern can swallow more specific captures if order and filtering are careless.

### Minimal Context From Textobjects

Textobjects are extremely relevant for agentic code assist.

From `nvim-treesitter-textobjects` and Zed:

```scheme
(function_item) @function.outer
(function_item
  body: (block
    "{" _+ @function.inner "}"))

(call_expression) @call.outer
(call_expression
  arguments: (arguments
    "(" _+ @call.inner ")"))
```

For Parceltongue, this powers:

- "give me the smallest function containing this line"
- "give me only the call expression I am editing"
- "give me the class/impl/module around this symbol"
- "give me the function body without imports and neighbor definitions"
- "expand context one ring at a time"

This is the token-saving surface. If an agent asks about one call, do not dump the file. Use textobject-style queries to locate the smallest semantically useful region, then grow to caller/callee context only when needed.

### Runnables As Agent Verification Hints

Zed's Rust `runnables.scm` captures:

- test modules
- test functions
- doc tests
- main functions

It also sets tags like:

```scheme
(#set! tag rust-test)
(#set! tag rust-doc-test)
(#set! tag rust-main)
```

For Parceltongue, this is a direct product idea:

```text
When an agent edits a symbol:
  1. find nearest runnable test or module
  2. find tests that mention/import/call the symbol
  3. suggest the cheapest verification command first
```

That is better than always running the full test suite.

### Redactions As Context Safety

Zed has `redactions.scm` for JSON, capturing string and number values as `@redact`.

Parceltongue should add redaction queries early. Agent context tools should not blindly ship:

- `.env` values
- tokens in JSON/YAML/TOML
- private keys
- password literals
- secrets embedded in test fixtures

Tree-sitter can produce precise byte ranges for redaction while preserving surrounding structure.

Example product behavior:

```text
raw:
  "OPENAI_API_KEY": "sk-..."

context output:
  "OPENAI_API_KEY": "<redacted:string>"
```

The graph can still know that the key exists without leaking the value.

### Query Testing Pattern

Steal from `nvim-treesitter` and Arborium:

```text
query file
  -> sample source file
  -> query validation/linting
  -> expected captures snapshot
```

For every Parceltongue query group:

```text
queries/rust/symbols.scm
fixtures/rust/symbols/sample.rs
snapshots/rust/symbols.expected.json
```

Snapshot output should include:

```json
{
  "capture": "definition.function",
  "text": "parse_source_text",
  "start_line": 12,
  "end_line": 20
}
```

This prevents silent breakage when grammar node names change.

### Parceltongue Query Naming Convention

Use a stable domain vocabulary.

Recommended capture prefixes:

```text
definition.*
reference.*
import.*
call.*
test.*
route.*
run.*
context.*
redact.*
injection.*
```

Avoid exposing UI-only capture names as graph facts:

```text
@keyword
@operator
@punctuation.bracket
@variable.builtin
```

They are useful for visual tools, but they are not enough for agent reasoning. If a UI capture is useful semantically, translate it into a Parceltongue fact type.

### Repos Already Touched For This Concept

| Repo | Status | Notes |
|---|---|---|
| `nvim-treesitter__nvim-treesitter` | Source-read | Best query-pack convention source: highlights, injections, folds, locals, indents, inheritance, lint/check workflow. |
| `nvim-treesitter__nvim-treesitter-textobjects` | Source-read | Best minimal-context/query-as-navigation source. |
| `CodeEditApp__CodeEditLanguages` | Inventory-read | Good packaged language-resource model with tags and structure queries. |
| `zed-industries__zed` | Source-read | Best examples of outline, runnables, redactions, and textobjects as product features. |
| `bearcove__arborium` | Source-read | Strong query packaging, generated constants, capture-name/runtime handling, pattern-index semantics. |
| `Wilfred__difftastic` | Source-read | Good capture namespace normalization for syntax-aware diffing. |

## Concept 3: Use Recursive AST Walkers When Queries Are Too Weak

### The Big Idea

Queries are excellent for finding candidate syntax nodes. They are not always enough to create reliable agent facts.

For Parceltongue, the high-value graph facts often need context that a single query match does not naturally carry:

- enclosing class, impl, trait, module, or test block
- whether a function is production code or test code
- whether a C++ function is free, in-class, out-of-class, or a gtest macro
- whether a Python class decorator should be part of the symbol extent
- whether a JavaScript arrow function is an inline route handler
- whether a TypeScript type member is a field, method, enum member, or interface member
- whether a method's qualified name should be `Class.method`, `Receiver.method`, or bare `function`

The reusable architecture pattern is hybrid:

```text
Tree-sitter parse tree
  -> cheap query or root traversal finds candidates
  -> recursive AST walker carries context
  -> language adapter resolves weird cases
  -> normalized Parceltongue fact
```

The slogan:

```text
queries find nodes;
walkers understand nodes.
```

### Evidence From Repos

| Repo | Source | Pattern Seen | Why Parceltongue Should Care |
|---|---|---|---|
| `sdsrss__code-graph-mcp` | `src/parser/treesitter.rs:88-170` | Recursive `extract_nodes` carries `parent_class`, `depth`, `in_test_context`, and `LanguageConfig`. | This is the core AST-walker shape Parceltongue should copy. |
| `sdsrss__code-graph-mcp` | `src/parser/treesitter.rs:120-152` | JS/TS test framework calls propagate `in_test_context` into nested callback functions. | Test detection needs contextual traversal, not only a node-kind query. |
| `sdsrss__code-graph-mcp` | `src/parser/treesitter.rs:176-216` | C/C++ `function_definition` handling descends into declarators, detects gtest macros, and builds qualified method names. | C/C++ extraction is a perfect example where universal queries are too weak. |
| `sdsrss__code-graph-mcp` | `src/parser/treesitter.rs:245-278` | Class extraction distinguishes Kotlin interface, Swift struct/enum keywords, and Python decorated extents. | A normalized `class` fact often needs language-specific AST interpretation. |
| `sdsrss__code-graph-mcp` | `src/parser/treesitter.rs:599-643` | Shared child traversal and `make_simple_node` produce normalized nodes with line ranges and truncated source. | Parceltongue needs consistent output structs no matter how messy the AST is. |
| `sdsrss__code-graph-mcp` | `src/parser/treesitter.rs:741-760` | Signature extraction normalizes TS/JS return type punctuation into consistent values. | Agent-facing signatures should not leak grammar-specific punctuation quirks. |
| `sdsrss__code-graph-mcp` | `src/parser/treesitter.rs:1120-1208` | Regression tests cover JS/TSX test context, C++ gtest detection, and C++ method qualification. | Parser support needs behavior tests, not only compile checks. |
| `Christoph__treesitter-mcp` | `src/extraction/types.rs:407-518` | After finding Rust types, it walks struct/enum/trait bodies to collect fields, variants, and members. | Good hybrid pattern: query/find top-level type, then walk body for rich type facts. |
| `Christoph__treesitter-mcp` | `src/extraction/types.rs:561-585` | Impl blocks are merged back into existing type definitions as members. | Dependency/type maps need multi-pass enrichment, not one pass per file. |
| `Christoph__treesitter-mcp` | `src/extraction/types.rs:708-805` | TypeScript fields, members, and enum variants are collected by walking body children. | TypeScript class/interface extraction benefits from targeted child walks. |
| `probelabs__probe` | `docs/reference/architecture.md:60-118` | Search pipeline: ripgrep scan, parse AST, rank, extract blocks, apply limits; language module owns Tree-sitter support. | Parceltongue should combine text search speed with AST precision instead of parsing everything first for every query. |
| `probelabs__probe` | `src/language/language_trait.rs:3-56` | `LanguageImpl` trait exposes language parser, acceptable parent logic, test detection, signatures, symbol-node logic, receiver type extraction. | Strong adapter boundary for language-specific behavior. |
| `probelabs__probe` | `src/language/parser_pool.rs:25-35`, `220-228` | Parser pool keyed by extension with preconfigured parsers. | Confirms parser lifecycle should be centralized when walking many files. |
| `probelabs__probe` | `src/language/rust.rs:28-96`, `98-130` | Rust adapter marks acceptable parent nodes and detects tests by attributes/name. | Per-language adapters keep walkers simpler and safer. |
| `probelabs__probe` | `src/extract/symbols.rs:266-315` | Child symbol collection tries body/member fields, known body node kinds, then falls back to direct child collection. | Useful fallback ladder for real-world grammars with inconsistent field names. |
| `probelabs__probe` | `src/extract/symbols.rs:317-399` | Symbol name extraction tries fields, impl-specific names, constructor/fallback special cases, then node kind fallback. | Symbol naming must be defensive and layered. |
| `probelabs__probe` | `src/extract/symbols.rs:402-448` | C-like declarator recursion extracts identifiers through nested declarators while skipping parameter lists. | Critical for C/C++/systems-language support. |
| `probelabs__probe` | `src/extract/symbols.rs:679-705` | Tree-sitter node kinds are normalized to user-friendly labels. | Parceltongue facts need stable domain kinds, not grammar-specific kind strings. |

`codebase-memory-mcp` also indexed `sdsrss__code-graph-mcp` for this concept:

- Index output: `/tmp/codex-code-intel/codebase-memory/sdsrss__code-graph-mcp-20260706-211343`
- Indexed nodes: `4,934`
- Indexed edges: `22,678`
- Excluded dirs: `docs`, `.git`, `vendor`

### The Walker Shape To Steal

The most useful function signature from the sweep is conceptually this:

```rust
fn extract_nodes(
    node: tree_sitter::Node,
    source: &str,
    language: SupportedLanguage,
    config: &LanguageExtractionConfig,
    parent_scope: Option<&str>,
    results: &mut Vec<ParsedNode>,
    depth: usize,
    in_test_context: bool,
) {
    if depth > MAX_AST_DEPTH {
        return;
    }

    let node_is_test = in_test_context || config.node_has_test_marker(node, source);

    match node.kind() {
        "function_item" => push_function(node, source, parent_scope, node_is_test, results),
        "class_declaration" => {
            let class_name = extract_name(node, source);
            push_class(node, source, &class_name, node_is_test, results);
            walk_children(node, source, language, config, Some(&class_name), results, depth + 1, node_is_test);
        }
        _ => walk_children(node, source, language, config, parent_scope, results, depth + 1, node_is_test),
    }
}
```

The important fields are:

- `parent_scope`: builds qualified names
- `depth`: prevents pathological recursion
- `in_test_context`: marks nested test helpers correctly
- `config`: carries language-specific flags
- `results`: accumulates normalized facts

### Normalized Output Shape

`sdsrss__code-graph-mcp` has a good `ParsedNode` shape:

```rust
pub struct ParsedNode {
    pub node_type: String,
    pub name: String,
    pub qualified_name: Option<String>,
    pub start_line: u32,
    pub end_line: u32,
    pub code_content: String,
    pub signature: Option<String>,
    pub doc_comment: Option<String>,
    pub return_type: Option<String>,
    pub param_types: Option<String>,
    pub is_test: bool,
}
```

Parceltongue should add byte ranges and extraction provenance:

```rust
pub struct CodeSymbolFact {
    pub symbol_kind: SymbolKind,
    pub name: String,
    pub qualified_name: Option<String>,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub start_byte: usize,
    pub end_byte: usize,
    pub signature: Option<String>,
    pub doc_comment: Option<String>,
    pub return_type: Option<String>,
    pub parameter_text: Option<String>,
    pub is_test: bool,
    pub parent_scope: Option<String>,
    pub extractor_id: String,
}
```

Byte ranges matter because graph edges, redactions, incremental invalidation, and minimal-context extraction all depend on exact spans.

### When Queries Become Too Weak

Use a recursive walker instead of only queries when:

1. You must carry context downward.
   - Example: functions inside `describe()` should be marked as test helpers.

2. You must build qualified names.
   - Example: `Calculator::multiply` should become `Calculator.multiply`.

3. You must interpret child relationships differently by language.
   - Example: C/C++ declarators, Go receiver methods, TypeScript class fields, Swift structs/classes/enums.

4. You must merge facts across constructs.
   - Example: type definition plus later `impl` members.

5. You must normalize signatures.
   - Example: TS return type text starts with `:`, while Rust/Go/Python do not.

6. You must preserve decorators/attributes as part of the semantic extent.
   - Example: Python decorated classes or functions.

7. You must apply fallback ladders.
   - Example: name field, type field, declarator recursion, constructor special case, fallback to node kind.

### Language Adapter Pattern

`probe` gives the clearest adapter interface:

```rust
pub trait LanguageImpl {
    fn get_tree_sitter_language(&self) -> TSLanguage;
    fn is_acceptable_parent(&self, node: &Node) -> bool;
    fn is_test_node(&self, node: &Node, source: &[u8]) -> bool;
    fn get_symbol_signature(&self, node: &Node, source: &[u8]) -> Option<String>;
    fn is_symbol_node(&self, node: &Node) -> bool;
    fn get_receiver_type(&self, node: &Node, source: &[u8]) -> Option<String>;
}
```

Parceltongue version:

```rust
pub trait LanguageExtractor {
    fn language(&self) -> SupportedLanguage;
    fn is_symbol_candidate(&self, node: tree_sitter::Node) -> bool;
    fn classify_symbol_kind(&self, node: tree_sitter::Node, source: &str) -> Option<SymbolKind>;
    fn extract_symbol_name(&self, node: tree_sitter::Node, source: &str) -> Option<String>;
    fn extract_signature(&self, node: tree_sitter::Node, source: &str) -> Option<String>;
    fn detect_test_context(&self, node: tree_sitter::Node, source: &str) -> bool;
    fn receiver_or_parent_scope(&self, node: tree_sitter::Node, source: &str) -> Option<String>;
}
```

Do not put all language behavior into one giant match forever. Start with matches if needed, but create a trait boundary early so Rust/C++/Python/TypeScript can diverge safely.

### Hybrid Query Plus Walker Pattern

Best practical pipeline:

```text
1. parse once
2. run cheap query for candidate top-level nodes
3. for each candidate, walk children with language adapter
4. normalize facts
5. attach provenance
6. persist facts and graph edges
```

Why not just recursively walk everything?

- Queries are faster and clearer for many candidate sets.
- Queries make intent visible and snapshot-testable.
- Walkers are better after the candidate has been narrowed.

Why not just queries?

- Real code facts require context, normalization, and fallback behavior.
- Queries can become unreadable when forced to encode every language-specific edge case.

### Multi-Pass Enrichment Pattern

`Christoph__treesitter-mcp` shows a useful type-map pattern:

```text
pass 1:
  collect type definitions
  collect fields, variants, trait members from bodies

pass 2:
  find impl blocks
  attach methods back to the existing type definition
```

This is important for Parceltongue because graph facts often need multiple passes:

```text
pass 1: symbols
pass 2: imports
pass 3: call sites
pass 4: resolve calls/imports to symbols
pass 5: enrich nodes with tests/routes/docs
pass 6: compute read-next and blast-radius surfaces
```

One parse tree can feed several passes. Do not reparse for each pass.

### Defensive Traversal Pattern

Use these guardrails:

```text
MAX_AST_DEPTH
safe source slicing by byte_range
UTF-8 boundary safe truncation
skip empty names
fallback ladders for names
normalize grammar-specific kinds
test regression for every language edge case
store extractor id and source range
```

The `sdsrss__code-graph-mcp` truncation pattern is worth stealing:

```rust
if content.len() <= max_code_content_len() {
    Cow::Borrowed(content)
} else {
    let mut end = max_code_content_len();
    while end > 0 && !content.is_char_boundary(end) {
        end -= 1;
    }
    let mut truncated = content[..end].to_string();
    truncated.push_str("...");
    Cow::Owned(truncated)
}
```

This sounds small, but it matters for agent tools. A panic while trimming context is an embarrassing failure mode.

### Normalizing Node Kinds

`probe` normalizes many grammar-specific kinds into friendly labels:

```text
function_item, function_declaration, function_definition, arrow_function -> function
method_declaration, method_definition, singleton_method -> method
struct_item, struct_type, struct_declaration -> struct
enum_item, enum_declaration -> enum
mod_item, module_declaration, namespace_declaration -> module
class_declaration, class_definition -> class
```

Parceltongue should have a formal `SymbolKind` enum:

```rust
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Trait,
    Interface,
    Module,
    Constant,
    Variable,
    Field,
    Route,
    Test,
    Unknown,
}
```

Graph storage should use normalized kind plus original node kind:

```text
kind = Function
original_node_kind = "function_item"
```

This preserves both agent ergonomics and debug detail.

### Regression Tests Are Mandatory

The `sdsrss__code-graph-mcp` tests are the right kind of tests:

- JS function outside `describe()` is not test
- JS helper inside `describe()` is test
- TSX did not silently become `unknown`
- C++ `TEST(MathSuite, Addition)` becomes `MathSuite.Addition` and is test
- C++ `Calculator::multiply` becomes `Calculator.multiply`

Parceltongue should keep one fixture per semantic edge case:

```text
fixtures/rust/tests.rs
fixtures/typescript/jest-describe.ts
fixtures/cpp/gtest.cpp
fixtures/python/decorators.py
fixtures/go/receiver_methods.go
fixtures/tsx/react-routes.tsx
```

Expected output should be JSON facts, not just "no panic."

### Parceltongue Product Translation

This concept powers these user journeys:

```text
Agent asks: "What symbol is under this line?"
  -> walker finds enclosing function/class/test context

Agent asks: "What should I run?"
  -> walker/query identifies nearest test/runnable

Agent asks: "What calls what?"
  -> walker extracts call sites with enclosing scope

Agent asks: "What is the blast radius?"
  -> graph uses normalized symbols and qualified names

Agent asks: "Give me minimum context."
  -> walker returns enclosing semantic unit plus dependency edges
```

### Repos Already Touched For This Concept

| Repo | Status | Notes |
|---|---|---|
| `sdsrss__code-graph-mcp` | Source-read plus `codebase-memory-mcp` indexed | Best AST walker example with context propagation, parser cache, tests, and normalized parsed nodes. |
| `Christoph__treesitter-mcp` | Source-read | Good hybrid type extraction: query top-level type, then walk bodies and impls for fields/members. |
| `probelabs__probe` | Source-read | Good language trait, parser pool, symbol extraction fallback ladders, and node-kind normalization. |

## Concept 4: Model Dependencies As Provenance-Rich Edges, Not Just Names

### Big Idea

For Parceltongue, the dependency graph should not be:

```text
function A calls function B
```

It should be:

```text
At file.rs:41:9, inside symbol A, extractor rust-call-v2 observed a call-shaped
Tree-sitter node whose target text was B. The resolver matched it to symbol B
in the same language with confidence extracted. The edge metadata says whether
this was a direct call, method call, route registration, import binding, type
reference, macro invocation, or ambiguous bare name.
```

That sounds more verbose, but it is the difference between a toy graph and an
agent-trust graph.

The user journey is not merely "show me all calls." The actual agent journey is:

```text
Agent wants to edit a symbol
  -> tool identifies symbol under cursor
  -> tool returns direct callers, direct callees, imports, tests, routes
  -> tool labels each edge with confidence and evidence
  -> agent chooses the next smallest file/function to inspect
  -> agent edits with a clear blast radius instead of wandering the repo
```

The important thing is that every edge must be explainable. If an LLM is going
to spend or save tokens based on a graph answer, the answer must carry enough
provenance for the LLM to know whether to trust it, ignore it, or ask for more
context.

### Evidence From The Reference Repos

| Repo | Source | Pattern | Parceltongue Takeaway |
|---|---|---|---|
| `sdsrss__code-graph-mcp` | `src/parser/relations/mod.rs:1-21` | Relation extraction is a separate public surface with `ParsedRelation`, `extract_relations`, and `extract_relations_from_tree`. Internals are split into imports, inherits, exports, routes, Rust, TS, Python, Go, Java, Dart, and C++. | Keep relation extraction distinct from symbol extraction. Symbols answer "what exists"; relations answer "what points at what." |
| `sdsrss__code-graph-mcp` | `src/parser/relations/mod.rs:75-110` | `ParsedRelation` stores source name, target name, relation, metadata, and source language. Language is stamped after walking so the resolver can enforce same-language equality. | Edge candidates need `source_language`. Without it, a Python `foo()` can accidentally match a C `foo()`. |
| `sdsrss__code-graph-mcp` | `src/parser/relations/mod.rs:259-560` | One recursive dispatcher carries `current_scope`, `current_class`, and Rust impl context while applying language-specific relation extractors. | Edge extraction needs lexical state. It cannot be just a flat query over all `call_expression` nodes. |
| `sdsrss__code-graph-mcp` | `src/parser/relations/imports.rs:1-241` | Imports preserve module metadata: JS/TS module names, Python modules, aliases, wildcard imports, and module-import flags. | Import edges should not collapse into bare `IMPORTS`. Preserve module/import-kind metadata for later resolution. |
| `sdsrss__code-graph-mcp` | `src/parser/relations/routes.rs:1-194` | Express, Go, Flask, and FastAPI routes become relations. Inline handlers get synthetic names with method, path, and line range. | Routes are dependency edges too. Agentic code assist needs route-entrypoint edges, not only function-call edges. |
| `sdsrss__code-graph-mcp` | `src/storage/schema.rs:72-118` | Edges include relation, metadata, and confidence. A `pending_unresolved_calls` table keeps calls that could not yet be matched. | Do not drop unresolved edges. Buffer them, re-sweep them after more files are indexed, and mark confidence honestly. |
| `sdsrss__code-graph-mcp` | `src/storage/queries/edges.rs:1-260` | Storage has edge records, pending call rows, incoming references, confidence floor filters, and batch target fetches. | Agent graph queries must avoid N+1 behavior. A good code graph is a database product, not only a parser product. |
| `sdsrss__code-graph-mcp` | `src/graph/impact.rs:1-240` | Impact classification partitions production callers, route callers, tests, affected files, risk level, and non-function unknown-risk warnings. | Blast radius is not "count all callers." It is a decision surface: prod/test/route/type confidence matters. |
| `Christoph__treesitter-mcp` | `src/analysis/call_graph.rs:21-23` | The MCP call graph has explicit edge rows: `direction|symbol|file|line|scope|depth`, with max depth and token budgets. | The agent-facing output should be compact rows with stable headers, not large nested JSON by default. |
| `Christoph__treesitter-mcp` | `src/analysis/call_graph.rs:266-399` | Caller/callee traversal uses BFS depth, definition resolution, containing-symbol lookup, and deduping. | The basic traversal shape is useful: queue, visited set, depth cap, resolve to enclosing definition. |
| `Christoph__treesitter-mcp` | `src/analysis/call_graph.rs:401-560` | Call extraction is best-effort: language-specific call-node kinds, `call_name`, row budget enforcement, and truncation markers. | When graph answers are heuristic, mark them compactly and enforce token budgets in the final response. |
| `Christoph__treesitter-mcp` | `src/analysis/find_usages.rs:31-43` | Usage rows include file, line, column, type, context, scope, confidence, and owner hint. | `find_usages` and graph edges should share the same mental model: evidence rows with confidence and ownership. |
| `Christoph__treesitter-mcp` | `src/analysis/find_usages.rs:123-148` | Tool output is compact JSON with `sym`, header `h`, rows `u`, and truncation metadata. | Parceltongue should have an ultra-compact mode optimized for LLM consumption. |
| `n24q02m__better-code-review-graph` | `src/better_code_review_graph/graph.py:1-6` | Graph store explicitly models nodes and edges: `CALLS`, `IMPORTS_FROM`, `INHERITS`, `IMPLEMENTS`, `CONTAINS`, `TESTED_BY`, `DEPENDS_ON`. | Start with a small relation vocabulary. Expand carefully, because every new relation becomes a product contract. |
| `n24q02m__better-code-review-graph` | `src/better_code_review_graph/graph.py:119-168` | SQLite schema stores nodes, edges, node summaries, source hashes, source text, and indexes by source/target/kind/file. | Graph storage should support both traversal and context materialization. Symbol rows need enough text to answer "show me the relevant body." |
| `n24q02m__better-code-review-graph` | `src/better_code_review_graph/graph.py:940-1064` | Edge lookups support optional kind filters, temporal filters, bare-name fallback, batch target lookup, and exact unqualified target search. | Reverse call tracing must handle both qualified and unqualified targets. Fallbacks should be explicit, not hidden. |
| `n24q02m__better-code-review-graph` | `src/better_code_review_graph/graph.py:1131-1251` | Impact radius runs BFS from changed files through forward and reverse edges, caps max nodes, batch-resolves nodes, returns impacted files and connecting edges. | For agents, impact radius should return the minimal connected subgraph, not just a list of filenames. |
| `tirth8205__code-review-graph` | `code_review_graph/graph.py:32-79` | SQLite graph schema includes edges with `confidence` and `confidence_tier`, plus source/target/kind indexes. | Confidence should be a first-class column. Do not bury it in prose. |
| `tirth8205__code-review-graph` | `code_review_graph/graph.py:223-287` | Edge upsert preserves multiple call sites by including line in the uniqueness check, and file updates atomically replace nodes and edges. | Multiple call sites between the same symbols are different evidence. Preserve line-level provenance. |
| `tirth8205__code-review-graph` | `code_review_graph/graph.py:360-388` | Reverse call tracing searches unqualified target names when fully qualified lookup misses, and transitive test discovery caps frontier growth. | Fallback and frontier caps are essential for large codebases. Otherwise the graph either misses obvious callers or explodes. |
| `tirth8205__code-review-graph` | `code_review_graph/graph.py:700-748` | Impact radius can be computed in SQL with a recursive CTE rather than only through an in-memory graph. | Parceltongue should not require loading the whole graph into memory for every agent query. |
| `abhigyanpatwari__GitNexus` | `gitnexus/src/core/ingestion/model/scope-resolution-indexes.ts:1-41` | Scope-resolution indexes are produced once after parsing and consumed later for call-resolution without re-walking ASTs. | Build durable indexes, then resolve. Do not re-walk the world for every question. |
| `abhigyanpatwari__GitNexus` | `gitnexus/src/core/ingestion/model/scope-resolution-indexes.ts:58-141` | The index bundle includes scope tree, defs, qualified names, module scopes, method dispatch, imports, bindings, reference sites, SCCs, and stats. | Real dependency graphs need lexical scope, import bindings, dispatch tables, and SCC information, not just symbol names. |
| `abhigyanpatwari__GitNexus` | `gitnexus/src/core/ingestion/resolve-references.ts:1-39` | Reference sites are resolved through registries into a `ReferenceIndex`; the module explicitly does no AST walks and no language switches. | Separate "extract possible reference sites" from "resolve them against indexes." This is the cleanest architecture in the set. |
| `abhigyanpatwari__GitNexus` | `gitnexus/src/core/ingestion/resolve-references.ts:90-160` | Resolution loops over reference sites, chooses a registry, emits references, and counts unresolved sites. | Resolution metrics are product metrics. Parceltongue should report unresolved counts. |
| `abhigyanpatwari__GitNexus` | `gitnexus/src/core/ingestion/resolve-references.ts:165-249` | Reference kinds choose registries: call, inherits, type-reference, read/write, import-use, macro. Results preserve confidence and evidence. | Calls, reads, writes, type refs, imports, and macros should not all be one "usage" blob. |
| `abhigyanpatwari__GitNexus` | `gitnexus/src/core/ingestion/emit-references.ts:1-46` | Resolved references are drained into graph edges with confidence, reason, evidence, and caller attribution from scope walking. | Graph edges should be materialized from resolved references, not from raw AST nodes directly. |
| `abhigyanpatwari__GitNexus` | `gitnexus/src/core/ingestion/emit-references.ts:245-305` | Reference kinds map to edge types: call -> `CALLS`, read/write -> `ACCESSES`, inherits -> `INHERITS`, type/import/macro -> `USES`. | A compact edge taxonomy can still retain detail through metadata and evidence. |
| `abhigyanpatwari__GitNexus` | `gitnexus/src/core/ingestion/scope-resolution/graph-bridge/references-to-edges.ts:1-19` | The graph bridge is a language-agnostic canonical path from `ReferenceIndex` to graph edges. | Parceltongue should have one graph bridge, with language-specific extraction upstream. |
| `abhigyanpatwari__GitNexus` | `gitnexus/src/core/ingestion/scope-resolution/graph-bridge/imports-to-edges.ts:1-57` | File-to-file import edges are deduped by source/target file and emitted through a single helper. | Import edges are a separate graph plane from symbol-call edges. They power file-level orientation. |
| `abhigyanpatwari__GitNexus` | `gitnexus/src/core/ingestion/call-processor.ts:1-16` | Call processing was narrowed to route/fetch edge emission and exported-type maps after call resolution moved into the registry pipeline. | Mature systems move away from ad hoc call processors toward registry-first resolution. |
| `abhigyanpatwari__GitNexus` | `gitnexus/src/core/ingestion/call-processor.ts:82-88` | Route-to-controller edges use an explicit confidence constant. | Framework edges are often high-value but not always as certain as lexical calls. They need confidence. |
| `abhigyanpatwari__GitNexus` | `gitnexus/src/core/ingestion/call-processor.ts:157-247` | Laravel routes are resolved qualified-first, then short-name fallback, skip ambiguity, and emit `CALLS` with route reason. | Framework-specific dependency extraction should be conservative. Skip ambiguous route edges rather than lying. |

### The Edge Pipeline Parceltongue Wants

A robust Parceltongue dependency graph should use a multi-phase pipeline:

```text
Source file
  -> ParseArtifact
  -> Symbol facts
  -> Relation candidates
  -> Scope/import/type indexes
  -> Resolution sweep
  -> Provenance-rich graph edges
  -> Agent-facing context queries
```

Each phase has a different job:

| Phase | Job | Output | Should It Guess? |
|---|---|---|---|
| Parse | Turn source into a Tree-sitter tree safely. | `ParseArtifact` | No. Parse or fail. |
| Symbol extraction | Find definitions and semantic containers. | `SymbolFact` rows | Lightly, with original node kind. |
| Relation extraction | Find raw references, calls, imports, routes, type refs, reads/writes. | `RelationCandidate` rows | Yes, but keep source text and extractor id. |
| Index finalization | Build scope tree, imports, bindings, qualified names, dispatch hints. | `ResolutionIndexes` | No. Deterministic indexes. |
| Resolution | Match candidates to known symbols or files. | `GraphEdge` or pending unresolved row | Yes, with confidence. |
| Query | Return the smallest useful graph slice. | Compact agent response | No hidden guessing. Return confidence and truncation. |

The key distinction:

```text
Relation candidate = "I saw text that appears to refer to X."
Graph edge         = "I resolved that observed reference to this known node."
```

Mixing these two is where bad tools become misleading. They see `foo()` and
immediately say "this calls repo/path/foo.ts::foo." Sometimes that is right.
Sometimes `foo` is an imported alias, a method on a receiver, a local callback,
a macro, a builtin, a re-export, or a same-name symbol in another language.

### Proposed Parceltongue Edge Candidate Shape

The candidate row should be lossless enough to debug later:

```rust
pub struct RelationCandidate {
    pub source_symbol: SymbolKey,
    pub source_file: FileId,
    pub source_language: LanguageId,
    pub source_span: TextSpan,
    pub relation: RelationKind,
    pub target_text: String,
    pub target_hint: TargetHint,
    pub metadata: RelationMetadata,
    pub extractor_id: &'static str,
    pub evidence: Vec<EdgeEvidence>,
}

pub enum TargetHint {
    BareName {
        name: String,
    },
    QualifiedPath {
        segments: Vec<String>,
    },
    ReceiverCall {
        receiver_text: String,
        method_name: String,
    },
    ImportedBinding {
        module: String,
        imported_name: String,
        local_name: String,
    },
    RouteHandler {
        method: String,
        path: String,
        handler_name: String,
    },
    ExternalUnknown {
        raw: String,
    },
}

pub enum RelationKind {
    Calls,
    Imports,
    References,
    Reads,
    Writes,
    UsesType,
    Inherits,
    Implements,
    Contains,
    Tests,
    RoutesTo,
    Fetches,
}

pub struct EdgeEvidence {
    pub kind: &'static str,
    pub weight: f32,
    pub note: String,
    pub span: Option<TextSpan>,
}
```

This is deliberately closer to GitNexus than to a quick AST grep. The raw AST
site is not the final truth. It is a piece of evidence that a resolver later
interprets against scope, imports, class ownership, module paths, and known
definitions.

### Proposed Parceltongue Graph Edge Shape

The resolved edge should be smaller, but still explainable:

```rust
pub struct GraphEdge {
    pub source_id: SymbolId,
    pub target_id: GraphTarget,
    pub relation: RelationKind,
    pub confidence: EdgeConfidence,
    pub metadata: EdgeMetadata,
    pub provenance: EdgeProvenance,
}

pub enum GraphTarget {
    Symbol(SymbolId),
    File(FileId),
    ExternalModule(String),
    UnresolvedName(String),
}

pub enum EdgeConfidence {
    Extracted,
    Inferred,
    Ambiguous,
}

pub struct EdgeProvenance {
    pub source_file: FileId,
    pub source_span: TextSpan,
    pub extractor_id: &'static str,
    pub resolver_id: &'static str,
}
```

Why keep unresolved targets in the type system? Because unresolved references
are still useful to agents:

```text
Agent asks: "Who calls parse_config?"
Tool says:
  - 5 extracted callers
  - 2 inferred same-language callers
  - 3 unresolved bare-name references called parse_config

Agent behavior:
  - inspect extracted callers first
  - inspect inferred callers if the edit is risky
  - maybe ignore unresolved rows unless debugging a missing edge
```

This is much better than pretending all 10 rows are equally true.

### Relation Vocabulary Should Stay Small

A practical first-pass taxonomy:

| Relation | Meaning | Typical Source |
|---|---|---|
| `CONTAINS` | File/class/module owns symbol. | Symbol extraction. |
| `CALLS` | Runtime-ish function/method/constructor dispatch. | Call expression, route handler, framework dispatch. |
| `IMPORTS` | File/scope imports module/file/symbol. | Import/use/include statements. |
| `REFERENCES` | Generic symbol reference when finer kind is unknown. | Identifier/value reference. |
| `READS` | Reads field/variable/member. | Field/value reference. |
| `WRITES` | Writes field/variable/member. | Assignment/update expression. |
| `USES_TYPE` | Type annotation, type parameter, superclass reference. | Type identifiers, annotations. |
| `INHERITS` | Class extends superclass. | Class declarations. |
| `IMPLEMENTS` | Class/type implements interface/trait. | Implements clauses, Rust impl trait. |
| `TESTS` | Test covers target. | Naming convention, test framework metadata, explicit edge. |
| `ROUTES_TO` | HTTP route maps to handler. | Express, Go net/http, Flask/FastAPI, Laravel, Next.js. |
| `FETCHES` | Consumer calls an HTTP route endpoint. | Fetch/Axios/request analysis. |

Do not start with 50 relation kinds. The product needs stable, teachable
answers. Add detail through metadata:

```json
{
  "relation": "CALLS",
  "metadata": {
    "call_kind": "method_call",
    "receiver": "client",
    "arity": 2,
    "qualifier": "self",
    "framework": null
  }
}
```

For route edges:

```json
{
  "relation": "ROUTES_TO",
  "metadata": {
    "framework": "express",
    "method": "GET",
    "path": "/users/:id",
    "inline_handler": true,
    "handler_start_line": 42,
    "handler_end_line": 57
  }
}
```

For import edges:

```json
{
  "relation": "IMPORTS",
  "metadata": {
    "module": "./user-service",
    "imported_name": "UserService",
    "local_name": "Service",
    "is_wildcard": false,
    "is_type_only": false
  }
}
```

### The Public Interface Dependency Graph

A useful agent graph is not always the full internal graph. For large codebases,
the agent often needs a filtered "public interface graph":

```text
All graph edges
  -> public/exported symbols
  -> route handlers
  -> CLI commands
  -> RPC handlers
  -> test entrypoints
  -> package/module boundaries
  -> external API calls
```

This answers questions like:

```text
If I change this function, does any public route change?
If I rename this type, does any exported package API change?
If I alter this struct field, what tests and consumers are closest?
If I touch this internal helper, can I stay inside one subsystem?
```

The public interface graph can be represented as a projection, not a separate
database:

```text
edge.is_public_surface = source.is_public || target.is_public || edge.relation in {ROUTES_TO, FETCHES}
edge.surface_kind = route | cli | exported_symbol | test | package_boundary | external_api
```

This matters for Parceltongue because an LLM does not need every private helper
edge first. It needs the boundaries that determine whether an edit is local,
subsystem-wide, or public-surface risky.

### Resolution Should Be Conservative And Observable

Resolution should have a ranked strategy:

```text
1. Same symbol id from scope-aware extraction.
2. Same-file exact qualified symbol.
3. Imported binding exact symbol.
4. Receiver/type-dispatch exact member.
5. Same-language unique symbol by qualified name.
6. Same-language unique symbol by bare name.
7. External module or builtin.
8. Pending unresolved.
9. Ambiguous, do not emit as normal edge unless caller asked for it.
```

The confidence can map like this:

| Resolution Result | Confidence | Agent Meaning |
|---|---|---|
| Scope-aware direct definition | `extracted` | Safe to follow by default. |
| Imported binding to exact symbol | `extracted` | Safe to follow by default. |
| Receiver/type dispatch with unique target | `inferred` | Usually useful; include in impact with clear label. |
| Same-language unique bare-name match | `inferred` | Include after exact edges; do not over-rank. |
| Same bare name with multiple candidates | `ambiguous` | Hide by default, expose with `include_ambiguous=true`. |
| Missing target during incremental indexing | `pending` | Store for later sweep. |
| External/builtin/module dependency | `external` | Useful for orientation, not local blast radius. |

The rule from the evidence is simple:

```text
False negatives are tolerable for an LLM-facing tool.
Confident false positives are poison.
```

This is exactly why the Ruby bare-call pass in `sdsrss__code-graph-mcp` is
biased toward false negatives, and why GitNexus skips ambiguous route controller
matches instead of emitting a wrong edge.

### Pending Unresolved Calls Are A First-Class Feature

Incremental indexing makes unresolved calls normal:

```text
Time 1:
  index file B
  B calls foo()
  foo is not indexed yet
  store pending_unresolved_call(source=B::bar, target_name=foo, language=ts)

Time 2:
  index file A
  A defines foo()
  sweep pending calls for target_name=foo and language=ts
  materialize edge B::bar CALLS A::foo
```

This matters for large repos where agents index only a slice. If Parceltongue
drops unresolved calls, it will quietly understate blast radius whenever the
callee appears later.

Proposed table:

```sql
CREATE TABLE pending_unresolved_relations (
    id INTEGER PRIMARY KEY,
    source_symbol_id INTEGER NOT NULL,
    source_file_id INTEGER NOT NULL,
    source_language TEXT NOT NULL,
    relation TEXT NOT NULL,
    target_text TEXT NOT NULL,
    metadata TEXT,
    extractor_id TEXT NOT NULL,
    source_start_line INTEGER NOT NULL,
    source_start_col INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);
```

The sweep should run:

```text
after indexing a file
after indexing a batch
after changing language config
after changing resolver version
after changing import/module resolver behavior
```

### File-Level Edges And Symbol-Level Edges Are Different Products

Agents need both:

```text
File-level graph:
  src/api/users.ts IMPORTS src/services/users.ts
  src/routes/users.ts ROUTES_TO src/controllers/users.ts

Symbol-level graph:
  getUserHandler CALLS fetchUserById
  UserController.get CALLS UserService.get
  UserService.get USES_TYPE User
```

File-level edges are best for orientation:

```text
"What files should I read before editing this file?"
"Which modules import this module?"
"Is this change isolated to one package?"
```

Symbol-level edges are best for precision:

```text
"Who calls this exact function?"
"Which tests cover this function?"
"What public route reaches this handler?"
```

Do not force one graph to answer both without a projection layer.

A clean product API:

```text
GET /graph/file-neighbors?file=src/foo.ts&depth=1
GET /graph/symbol-neighbors?symbol=Foo.bar&relations=CALLS,TESTS
GET /graph/public-surface?symbol=Foo.bar
GET /graph/impact?symbol=Foo.bar&change_type=signature
```

### Agent-Facing Query Shapes

The graph should expose small, predictable workflows.

#### 1. Callers Of Symbol

```text
Input:
  symbol = parse_config
  file = src/config.rs
  max_depth = 2
  min_confidence = inferred
  max_tokens = 2000

Output rows:
  direction|symbol|file|line|scope|depth|relation|confidence|reason
  caller|load_app_config|src/app.rs|44|App|1|CALLS|extracted|same-file import binding
  caller|main|src/main.rs|18|module|2|CALLS|inferred|same-language unique bare name
```

This gives the LLM enough to decide:

```text
Read load_app_config first.
Maybe read main if the signature changes.
Do not inspect every file containing "parse_config".
```

#### 2. Callees Of Symbol

```text
Input:
  symbol = handle_request
  relations = CALLS,ROUTES_TO,FETCHES

Output:
  callee rows sorted by directness, confidence, public-surface risk
```

Useful for:

```text
Before editing this handler, which services and validators does it depend on?
```

#### 3. Tests For Symbol

```text
Input:
  symbol = calculate_invoice_total

Output:
  direct TESTS edges
  naming-convention test candidates
  transitive tests through callees up to depth 1
```

The `tirth8205__code-review-graph` transitive test query is a good warning:
cap the frontier. Test discovery can explode if every utility function reaches
half the test suite.

#### 4. Impact Radius

```text
Input:
  symbol = UserService.update
  change_type = signature
  depth = 3
  max_nodes = 500

Output:
  prod_callers
  test_callers
  route_callers
  affected_files
  public_surface_edges
  risk_level
  truncated
```

This is more useful than raw BFS. `sdsrss__code-graph-mcp` is right to partition
production and test callers; an edit with 40 tests and 0 production callers is a
different decision from 2 public route callers and 0 tests.

#### 5. Explain Edge

A critical debug tool:

```text
GET /graph/explain-edge?source=UserController.get&target=UserService.get

Response:
  relation: CALLS
  confidence: extracted
  source_span: src/controllers/user.ts:44:11
  extractor: ts-call-expression-v3
  resolver: imported-binding-v2
  evidence:
    - call_expression target text "service.get"
    - receiver "service" bound from constructor parameter
    - imported type UserService from ../services/user
```

Without `explain-edge`, the LLM cannot debug the graph. It can only trust or
ignore it. That is too brittle for a personal mega-agent workflow.

### Context Selection From Edges

The graph should directly produce a read-next bundle:

```text
Query:
  I need to edit Foo.bar with token budget 6000.

Algorithm:
  1. Include Foo.bar body.
  2. Include direct extracted callers.
  3. Include direct extracted callees whose signatures are needed.
  4. Include tests with direct TESTS edges.
  5. Include public-surface routes reaching Foo.bar.
  6. Include imports/types only as signatures unless body needed.
  7. Include inferred/ambiguous edges only if budget remains or risk is high.
```

This can be represented as scoring:

| Edge Type | Base Score | Notes |
|---|---:|---|
| Direct exact caller | 1.00 | Highest value for behavior and signature edits. |
| Direct exact callee | 0.90 | Needed to understand dependencies. |
| Public route caller | 0.95 | Product surface risk. |
| Direct test | 0.85 | Verification path. |
| Importing file | 0.55 | Useful orientation, often not enough alone. |
| Type reference | 0.50 | Important for Rust/TS/C++ signature work. |
| Inferred bare-name caller | 0.45 | Useful but lower trust. |
| Ambiguous match | 0.10 | Only include on explicit request. |

Then adjust by distance:

```text
score = base_score * confidence_multiplier * depth_decay * recency_multiplier
```

Suggested multipliers:

```text
confidence:
  extracted = 1.00
  inferred = 0.65
  ambiguous = 0.25

depth:
  depth 1 = 1.00
  depth 2 = 0.65
  depth 3 = 0.40
```

This turns the dependency graph into a token allocator. That is the Parceltongue
product, not merely a diagram.

### Routes Are First-Class Edges

Routes are where code graphs become product-aware.

Examples:

```text
Express:
  app.get("/users/:id", getUser)
  route GET /users/:id ROUTES_TO getUser

Flask:
  @app.route("/users/<id>", methods=["GET"])
  def get_user(id): ...
  route GET /users/<id> ROUTES_TO get_user

Go:
  http.HandleFunc("/users", handleUsers)
  route ANY /users ROUTES_TO handleUsers

Laravel:
  Route::get("/orders", [OrderController::class, "index"])
  route GET /orders CALLS OrderController.index
```

Why this matters:

```text
Agent asks: "Is this helper reachable from public API?"
Graph answers:
  yes, route GET /users/:id reaches handler getUser, which calls helper.
```

That is a far better answer than:

```text
grep found helper in 37 files
```

### Imports Are Binding Evidence, Not Just File Links

A shallow file dependency graph says:

```text
foo.ts imports bar.ts
```

A useful agent graph says:

```text
foo.ts imports `UserService` from `./bar` as local binding `Service`.
Inside handleUser, `Service.get()` resolves to bar.ts::UserService.get.
```

The import edge alone is not enough. It is a bridge into reference resolution.

Parceltongue should store both:

```text
File import edge:
  foo.ts IMPORTS bar.ts

Binding fact:
  in module scope foo.ts, local name Service binds to bar.ts::UserService

Call edge:
  foo.ts::handleUser CALLS bar.ts::UserService.get
```

This mirrors GitNexus's `ScopeResolutionIndexes`: imports, bindings, module
scopes, defs, and reference sites are separate, composable indexes.

### Macro And Type References Must Not Become Fake Calls

This is a subtle but important point from GitNexus:

```text
log!("hello")
```

That should not automatically be a `CALLS` edge to a function named `log`.
Macros and functions are different namespaces in Rust. Similarly:

```text
User
```

inside a type annotation is not a runtime call to `User`.

Bad graph:

```text
create_user CALLS User
```

Better graph:

```text
create_user USES_TYPE User
create_user USES macro log
```

This matters because the agent's next move changes:

```text
CALLS      -> read callee behavior
USES_TYPE  -> read type definition/signature
USES macro -> read macro definition only if expansion affects edit
```

### SQL Storage Is A Product Decision

Several repos converge on SQLite-backed graph storage. That is not accidental.
SQLite gives an agent tool:

```text
fast local queries
portable cache file
indexes over source/target/kind
recursive CTEs for impact
FTS over symbol names and context strings
incremental update transactions
easy debugging with sqlite3
```

For a personal Codex-app companion, SQLite is more attractive than a server
database unless the graph becomes huge enough to need a specialized backend.

Minimum schema:

```sql
CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    language TEXT,
    blake3_hash TEXT NOT NULL,
    indexed_at INTEGER NOT NULL
);

CREATE TABLE symbols (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    qualified_name TEXT,
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    signature TEXT,
    context_string TEXT,
    is_test INTEGER NOT NULL DEFAULT 0,
    is_public INTEGER NOT NULL DEFAULT 0,
    metadata TEXT
);

CREATE TABLE edges (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL,
    target_id INTEGER,
    target_external TEXT,
    relation TEXT NOT NULL,
    confidence TEXT NOT NULL,
    metadata TEXT,
    extractor_id TEXT NOT NULL,
    resolver_id TEXT NOT NULL,
    source_line INTEGER NOT NULL,
    source_col INTEGER NOT NULL
);

CREATE INDEX idx_edges_source_relation ON edges(source_id, relation);
CREATE INDEX idx_edges_target_relation ON edges(target_id, relation);
CREATE INDEX idx_edges_relation_confidence ON edges(relation, confidence);
```

For agent search, add FTS:

```sql
CREATE VIRTUAL TABLE symbol_fts USING fts5(
    name,
    qualified_name,
    signature,
    context_string,
    metadata
);
```

### Recommended Parceltongue Resolver Architecture

The strongest architecture from these repos is:

```text
language extractors
  -> SymbolFact[]
  -> ReferenceSite[]
  -> ImportFact[]
  -> RouteFact[]

finalize indexes
  -> ScopeTree
  -> DefIndex
  -> QualifiedNameIndex
  -> ModuleScopeIndex
  -> ImportBindingIndex
  -> MethodDispatchIndex
  -> RouteIndex

resolution
  -> ReferenceIndex
  -> EdgeEmitter
  -> GraphStore
```

This is the GitNexus lesson in Parceltongue terms:

```text
Do not let every language extractor write graph edges directly.
Let extractors produce facts.
Let one resolver create graph edges.
```

Why?

```text
1. You can improve resolution without rewriting every extractor.
2. You can measure unresolved sites centrally.
3. You can add confidence/evidence uniformly.
4. You can support new agent queries without reparsing.
5. You can test each layer independently.
```

### Regression Fixtures Parceltongue Needs For Edges

Minimum edge fixtures:

```text
fixtures/rust/
  same_file_call.rs
  module_use_call.rs
  trait_impl_call.rs
  macro_invocation_not_call.rs
  type_reference_not_call.rs

fixtures/typescript/
  named_import_call.ts
  alias_import_call.ts
  method_receiver_call.ts
  type_only_import.ts
  express_route_inline.ts

fixtures/python/
  import_module_call.py
  from_import_alias_call.py
  flask_route.py
  local_function_shadow.py

fixtures/go/
  package_import.go
  receiver_method.go
  http_handle_func.go

fixtures/cpp/
  namespace_call.cpp
  method_call.cpp
  inheritance.cpp
  macro_not_call.cpp
```

Expected output should be graph facts:

```json
{
  "symbols": [
    {"qualified_name": "src/foo.rs::parse_config", "kind": "Function"}
  ],
  "relations": [
    {
      "source": "src/main.rs::main",
      "target_text": "parse_config",
      "relation": "CALLS",
      "confidence": "extracted",
      "source_line": 12
    }
  ],
  "edges": [
    {
      "source": "src/main.rs::main",
      "target": "src/foo.rs::parse_config",
      "relation": "CALLS",
      "confidence": "extracted"
    }
  ],
  "pending": []
}
```

This lets tests distinguish:

```text
extraction succeeded but resolution failed
resolution succeeded but confidence is wrong
edge exists but source span is wrong
macro incorrectly became CALLS
route edge was emitted but handler was not materialized
```

### Product Translation For A Solo Codex Power User

For the user's stated workflow:

```text
Solo agent power user
All languages
CRUD apps plus Rust/C/C++ systems programming
Not a product for others
Help Codex navigate large codebases faster and reliably
```

Concept 4 becomes these products:

| Product/Feature | What It Does | Why It Helps Codex |
|---|---|---|
| `callers_of` | Returns direct and transitive callers with confidence. | Codex can inspect impact before editing. |
| `callees_of` | Returns dependencies of the current symbol. | Codex can read only the functions that explain behavior. |
| `tests_for` | Returns direct, inferred, and transitive tests. | Codex can choose verification commands faster. |
| `routes_to` | Maps HTTP routes/RPC handlers to code. | Codex understands CRUD app entrypoints. |
| `public_surface` | Shows exported APIs, routes, CLI commands, and package boundaries touched by a symbol. | Codex knows whether a change is local or externally visible. |
| `why_edge` | Explains a dependency edge with source span and resolver evidence. | Codex can debug the graph instead of blindly trusting it. |
| `read_next` | Produces a token-budgeted context bundle from graph edges. | Codex spends fewer tokens wandering. |
| `impact_radius` | Returns prod callers, test callers, affected files, route callers, risk. | Codex can plan edits and verification with less guesswork. |

This is exactly the "LLM asks query to tool -> tool responds with relevant
context" loop:

```text
LLM:
  I need to edit `UserService.update`.

Tool:
  Here is the current symbol, 3 extracted callers, 2 callees, 1 route path,
  4 tests, and 1 inferred ambiguous caller hidden by default. Read these next.

LLM:
  Reads only those nodes, edits, then asks impact/test query again.
```

### Failure Modes To Avoid

| Failure | Why It Is Bad | Fix |
|---|---|---|
| Treating every identifier as a call. | Creates false blast radius. | Separate call/type/read/write/import/macro reference kinds. |
| Dropping unresolved calls. | Understates impact during partial/incremental indexing. | Store pending unresolved relations and sweep later. |
| No source span on edges. | Agent cannot verify why an edge exists. | Store line/column and extractor id. |
| No confidence. | Agent cannot rank trust. | Store `extracted`, `inferred`, `ambiguous`, `external`, `pending`. |
| No route edges. | CRUD apps lose their most important entrypoints. | Add route extractors and route metadata. |
| One graph level only. | File orientation and symbol precision fight each other. | Keep file-level and symbol-level projections. |
| Whole-graph BFS always. | Large repos become slow/noisy. | Add SQL recursive CTE and max node/depth caps. |
| Language-specific edge emitters everywhere. | Fixes become scattered. | Extract facts per language, resolve through shared pipeline. |
| Ambiguous fallback included by default. | LLM follows wrong edges. | Hide ambiguous rows unless requested. |
| No compact output mode. | Tool saves no tokens. | Stable row headers plus truncation marker. |

### What To Copy Most Directly

From `sdsrss__code-graph-mcp`:

```text
ParsedRelation with source_language
pending_unresolved_calls table
edge confidence column
route extraction as graph relations
impact classifier with prod/test/route partitioning
```

From `GitNexus`:

```text
ReferenceSite -> Registry lookup -> ReferenceIndex -> graph bridge
scope-resolution indexes
reference kind to edge kind mapping
confidence plus evidence on emitted edges
qualified-first route resolution with ambiguity skip
```

From `better-code-review-graph`:

```text
SQLite graph store
source/target/kind indexes
bare-name fallback for reverse call tracing
recursive impact radius with caps
temporal/as-of idea for graph rows
```

From `Christoph__treesitter-mcp`:

```text
compact row output with token budget
call graph depth cap
find_usages rows with scope/confidence/owner hints
best-effort shape plus graph answers for MCP clients
```

From `tirth8205__code-review-graph`:

```text
confidence_tier on edges
line-preserving edge upsert
transitive tests with frontier cap
SQL recursive BFS alternative
```

### Concept 4 Conclusion

Parceltongue should evolve from:

```text
Tree-sitter parses files and builds a dependency graph.
```

to:

```text
Tree-sitter extracts symbol and relation evidence.
Parceltongue resolves that evidence into confidence-labeled graph edges.
Codex queries the graph for the smallest trustworthy context slice.
```

This is the core architecture that makes Parceltongue worth having even inside
the Codex app. Codex already has shell, grep, ripgrep, and file reads. What it
does not have by default is a persistent, explainable, language-aware dependency
memory that can say:

```text
Read these five symbols next.
Ignore these twenty files.
This edge is exact.
This edge is inferred.
This route reaches your code.
These tests are closest.
This public interface may break.
```

That is the code-assist product. The graph is not the UI. The graph is the
agent's decision substrate.

### Repos Already Touched For This Concept

| Repo | Status | Notes |
|---|---|---|
| `sdsrss__code-graph-mcp` | Source-read plus `codebase-memory-mcp` indexed | Best Rust example for `ParsedRelation`, pending unresolved calls, confidence, route extraction, and impact classification. |
| `Christoph__treesitter-mcp` | Source-read plus `codebase-memory-mcp` indexed | Useful compact MCP call graph and usage row design with budgeted output. |
| `n24q02m__better-code-review-graph` | Source-read | Useful SQLite graph schema, bare-name fallback, temporal filters, and impact radius traversal. |
| `tirth8205__code-review-graph` | Source-read plus `codebase-memory-mcp` indexed | Useful confidence tiering, line-preserving edges, transitive tests, and SQL recursive BFS. |
| `abhigyanpatwari__GitNexus` | Source-read | Most mature resolution architecture: scope indexes, reference sites, registry lookup, graph bridge, route edge confidence. |

## Concept 5: Incremental Indexing Is A Freshness Contract, Not Just A Speed Hack

### The Big Idea

Incremental indexing is often described as a performance feature:

```text
Do not parse the whole repo when one file changed.
```

That is true, but too small.

For Parceltongue, incremental indexing is a freshness contract between the
agent and the code graph:

```text
When Codex asks about a file, symbol, caller, route, or test after editing,
Parceltongue must either answer from current bytes or say exactly what is
stale.
```

That distinction matters because a stale dependency graph is worse than no
graph when it is presented as truth. If Codex asks "what calls this function?"
and Parceltongue silently answers from yesterday's graph, the agent will skip
real callers, choose the wrong tests, and make edits that feel locally correct
but globally unsafe.

For a solo Codex power user, the product promise is:

```text
I can edit quickly.
The graph follows me.
If the graph cannot follow me, it marks the answer stale.
```

That is the heart of this concept.

### Evidence Table

| Repo | Source Evidence | What It Teaches Parceltongue |
|---|---|---|
| `tree-sitter__py-tree-sitter` | `examples/usage.py:119-139` edits an old tree, reparses with the old tree, then prints `tree.changed_ranges(new_tree)`. | Tree-sitter gives a real incremental parse primitive. Parceltongue can use old trees and changed ranges for editor-speed updates. |
| `tree-sitter__node-tree-sitter` | `src/tree.cc:88-125` applies `TSInputEdit` and updates cached nodes; `src/tree.cc:143-160` exposes changed ranges; `src/parser.cc:209-245` accepts an old tree and parse options. | Bindings expose the same low-level model: edit old tree, parse new bytes against it, inspect changed ranges. |
| `sdsrss__code-graph-mcp` | `src/indexer/merkle.rs:38-50` streams BLAKE3 file hashes; `:65-106` scans eligible files while respecting ignore rules; `pipeline/mod.rs:63-162` implements query-time `ensure_file_indexed`; `pipeline/index_files.rs:1-18` outlines delete, parse, resolve, context, pending-call sweep phases. | Good model for Parceltongue: file hashes decide changed files, graph replacement is transactional, and explicit file queries trigger freshness checks. |
| `sdsrss__code-graph-mcp` | `src/indexer/watcher.rs:7-34` treats unknown watcher events as content changes and relies on idempotent Merkle rescans; `:41-104` canonicalizes paths and sends relative paths through a bounded channel. | Watchers are hints, not truth. A rescan/hash comparison is the source of truth. |
| `n24q02m__better-code-review-graph` | `src/better_code_review_graph/incremental.py:550-632` reparses changed plus dependent files and returns reviewer summaries; `:467-516` hashes bytes, skips unchanged files, deletes missing files, and records errors. | Useful high-level product flow: changed files are not enough; dependent files and reviewer summaries make incremental indexing agent-useful. |
| `n24q02m__better-code-review-graph` | `BREAKING_CHANGES.md:9-23` and `temporal.py:1-37` document `valid_from_sha` / `valid_to_sha` temporal graph rows. | A graph can answer "current" by default and still support historical/as-of queries for code review. |
| `tirth8205__code-review-graph` | `code_review_graph/changes.py:33-69` parses git diff ranges; `:174-211` maps line ranges to overlapping graph nodes; `:219-269` computes risk from flows, tests, security keywords, and caller count. | Incremental indexing should not stop at file freshness. It can convert changed hunks into changed symbols and risk. |
| `DeusData__codebase-memory-mcp` | `src/store/store.h:56-62` models file hash rows; `:391-399` exposes file hash CRUD; `src/watcher/watcher.h:1-7` describes git-based background auto-sync. | A C/SQLite design still lands on the same primitives: per-file identity, stored hashes, and background sync. |
| `DeusData__codebase-memory-mcp` | `src/pipeline/pipeline_incremental.c:1-10` explains disk-based incremental reindexing; `:73-123` classifies files by mtime and size; `:299-322` snapshots inbound cross-file edges before purge; `:677-876` loads existing graph, purges changed/deleted files, reparses, relinks, and persists hashes. | Incremental correctness is mostly about preserving graph meaning across deletion/replacement. The hard part is not parsing. It is not losing cross-file edges. |

### Three Layers Of Incrementality

Parceltongue should treat incrementality as three separate layers.

```text
Layer 1: Tree incrementality
  Old Tree-sitter tree + edit + new bytes -> new tree + changed ranges

Layer 2: File graph incrementality
  Stored file hashes + current file hashes -> new/changed/deleted files
  Changed files -> parse/extract/resolve/replace graph rows

Layer 3: Agent-loop freshness
  Codex query includes file/symbol -> ensure relevant files are fresh now
  If not fresh within budget -> answer with stale marker
```

The mistake would be to collapse these layers into one magical "incremental
index" feature. They solve different problems.

Tree-sitter changed ranges are excellent for editor-speed parsing. They tell
you which byte ranges changed syntactically between two trees. They do not, by
themselves, tell you every downstream graph edge that became invalid.

File hashes are excellent for persistent graph indexing. They tell you which
files need reprocessing after a CLI command, watcher event, or Codex edit. They
do not, by themselves, tell you which unchanged files have stale inbound or
outbound edges.

Agent-loop freshness is the product surface. It tells Codex whether the answer
can be trusted now.

### Layer 1: Tree-sitter Changed Ranges

The Tree-sitter primitive looks like this:

```text
old_source
old_tree
edit(start_byte, old_end_byte, new_end_byte, start_point, old_end_point, new_end_point)
new_tree = parser.parse(new_source, old_tree)
changed_ranges = old_tree.changed_ranges(new_tree)
```

The Python binding example does exactly this in `py-tree-sitter`:

```text
tree.edit(...)
new_tree = parser.parse(new_src, tree)
for changed_range in tree.changed_ranges(new_tree):
  print changed range
```

That gives Parceltongue a fast path for interactive editing:

```text
User edits file.
Parceltongue applies edit to cached tree.
Parceltongue reparses with old tree.
Parceltongue extracts changed syntax ranges.
Parceltongue maps changed ranges to containing symbols.
```

The immediate value:

```text
Which function changed?
Which class body changed?
Which import region changed?
Which route declaration changed?
Which tests overlap this hunk?
```

But this layer must not be oversold.

Changed ranges can say:

```text
The body of function X changed.
```

They cannot safely say:

```text
Only function X matters.
```

Why not?

Because small local edits can change global meaning:

```text
rename function
change exported type
add overload
remove import
change route path
change trait implementation
change macro invocation
change generic bound
change package/module visibility
```

So changed ranges are best used as the first lens, not the whole invalidation
model.

### Layer 2: File Hashes And Merkle-Style Diffs

For a persistent code graph, the boring file identity system matters more than
the clever AST diff.

The useful baseline:

```text
files table:
  path
  language
  content_hash
  last_modified
  indexed_at
  index_status
```

Every incremental run does this:

```text
stored_hashes = SELECT path, content_hash FROM files
current_hashes = scan_project_files()
diff = compute_diff(stored_hashes, current_hashes)

new_files = current - stored
changed_files = same path, different hash
deleted_files = stored - current
```

`sdsrss__code-graph-mcp` is a strong reference here:

```text
hash_file(path):
  stream file through BLAKE3
  constant memory
  16 KB buffer

scan_directory(root):
  respect .gitignore
  skip hidden files
  skip build/dependency dirs
  skip unsupported languages
  hash eligible files in parallel
```

This is exactly the kind of conservative infrastructure Parceltongue needs.
The graph should not index `node_modules`, Rust `target`, vendored dependency
trees, SQLite databases, lockfiles, binary artifacts, or generated caches unless
the user explicitly opts in.

For Codex, the important point is not just speed. It is reducing false context.

Indexing dependency directories creates enormous graph noise:

```text
Codex asks about my service method.
Graph returns calls from generated or vendored code.
Agent wastes context on code I do not intend to edit.
```

So the file scanner is part of product quality.

### Directory Caches Are Hints, Not Truth

Directory mtime caches can speed up rescans, but they are dangerous if treated
as complete proof.

The `sdsrss__code-graph-mcp` Merkle scanner explicitly stores both directory
mtimes and per-file mtimes. Its comments call out the key trap:

```text
directory mtime changes on add/remove
directory mtime may not change on file content modification
```

So Parceltongue should use this hierarchy:

```text
Trust content hash most.
Trust file mtime plus size as a cheap prefilter.
Trust directory mtime only as an optimization.
Never let directory mtime alone prove semantic freshness.
```

A good indexer can say:

```text
This subtree probably did not change, so I skipped expensive hashing.
```

But a correctness-critical query should still be allowed to force a fresh hash
for the exact file Codex is asking about.

### Layer 3: Query-Time Freshness

This is the idea Parceltongue should copy most aggressively from
`sdsrss__code-graph-mcp`.

The repo has an `ensure_file_indexed` path whose comments are almost a product
spec for Codex:

```text
When an MCP tool receives an explicit file_path argument, the agent is signaling:
I just edited this; please answer against the current bytes.
```

That is exactly our workflow.

Codex edits a file, then asks:

```text
what calls this?
what tests cover this?
what changed?
show me the dependency slice
```

If Parceltongue waits for a background watcher or a 30 second debounce, it can
answer from stale data at the exact moment freshness matters most.

So every agent-facing query that names a file should do:

```text
if freshness_policy == MustBeFresh:
  ensure_file_fresh(file_path)
  answer only after graph is current

if freshness_policy == BestEffortStaleOk:
  try ensure_file_fresh(file_path) within small budget
  if budget exhausted, answer with stale marker

if freshness_policy == NoIndexPlainSearch:
  skip graph freshness and use raw search/file reads
```

This is not only an indexing feature. It is an honesty feature.

### A Freshness Policy For Parceltongue

Suggested API enum:

```rust
pub enum FreshnessPolicy {
    MustBeFresh,
    BestEffortStaleOk { budget_ms: u64 },
    NoIndexPlainSearch,
}
```

Suggested query wrapper:

```rust
pub struct GraphQueryRequest {
    pub query_kind: QueryKind,
    pub file_path: Option<ProjectRelativePath>,
    pub symbol: Option<QualifiedSymbol>,
    pub freshness: FreshnessPolicy,
    pub token_budget: Option<usize>,
}
```

Suggested answer wrapper:

```rust
pub struct GraphQueryResponse<T> {
    pub data: T,
    pub freshness: FreshnessReport,
    pub warnings: Vec<QueryWarning>,
}

pub struct FreshnessReport {
    pub status: FreshnessStatus,
    pub files_checked: Vec<ProjectRelativePath>,
    pub files_reindexed: Vec<ProjectRelativePath>,
    pub stale_files: Vec<ProjectRelativePath>,
    pub index_revision: String,
}

pub enum FreshnessStatus {
    Fresh,
    PartiallyFresh,
    StaleButAnswered,
    RefusedBecauseStale,
}
```

The product behavior becomes clear:

```text
If Codex asks for impact radius with MustBeFresh:
  Parceltongue either updates the graph or refuses stale output.

If Codex asks for quick orientation with BestEffortStaleOk:
  Parceltongue can answer quickly but labels stale rows.
```

This is the difference between a trusted assistant tool and a noisy search toy.

### Incremental Update Plan

Parceltongue should make the indexer's decision visible.

```rust
pub struct IndexUpdatePlan {
    pub reason: IndexReason,
    pub new_files: Vec<ProjectRelativePath>,
    pub changed_files: Vec<ProjectRelativePath>,
    pub deleted_files: Vec<ProjectRelativePath>,
    pub dependent_files: Vec<ProjectRelativePath>,
    pub skipped_files: Vec<SkippedFile>,
    pub estimated_work: EstimatedIndexWork,
}

pub enum IndexReason {
    FullBuild,
    GitDiff,
    WatcherHint,
    QueryTimeFreshness,
    ManualRefresh,
}
```

This is useful in chat because Codex can explain:

```text
Parceltongue reindexed 1 changed file and 3 dependent files before answering.
```

Or:

```text
Parceltongue skipped graph refresh because 1,200 files changed and the query
budget was 250 ms. The answer is marked stale.
```

That kind of explanation prevents silent confusion.

### Invalidation Is A Graph Problem

The naive incremental algorithm is:

```text
for each changed file:
  delete old nodes in that file
  parse changed file
  insert new nodes
  insert new edges from that file
```

That looks right and is wrong.

It loses inbound cross-file edges.

Example:

```text
file A:
  fn target()

file B:
  fn caller() {
    target()
  }
```

If only file A is reindexed, the indexer deletes A's old `target` node. SQLite
cascade deletes B -> A edge because the target node vanished. Then the indexer
recreates A's `target` node. But B was not reparsed, so B -> A is not emitted
again.

The graph silently loses a real call edge.

Both `sdsrss__code-graph-mcp` and `DeusData__codebase-memory-mcp` contain
explicit fixes for this class of bug.

The durable pattern:

```text
1. Before deleting changed/deleted file rows, snapshot inbound edges whose source
   file is not being reparsed.

2. Delete old rows for changed/deleted files.

3. Reparse changed files.

4. Re-resolve edges emitted by changed files.

5. Re-link saved inbound edges by stable qualified names if the target still
   exists.

6. Drop saved inbound edges if the target was renamed or deleted.
```

This should be a first-class Parceltongue invariant:

```text
Incremental reindex after a one-file edit should converge to the same graph as
a full reindex, except for explicitly documented deferred passes.
```

### Pending Relations Are Part Of Freshness

Concept 4 argued for pending unresolved calls. Concept 5 makes them even more
important.

During incremental indexing, the target of a call may not exist yet:

```text
file B calls A.foo
file A is currently deleted, renamed, or not indexed
```

Dropping the call loses evidence. Keeping a fake resolved edge lies. The right
model is:

```text
pending_unresolved_relation:
  source_node_id
  target_hint
  relation_kind
  source_language
  metadata
  first_seen_index_revision
```

Then after every incremental batch:

```text
sweep pending relations:
  find target candidates now present
  apply same resolver rules
  promote exact matches to graph edges
  keep ambiguous or unresolved rows pending
```

This lets the graph heal as files are added back, generated, renamed, or
indexed later.

For Codex, pending rows should appear only when relevant:

```text
callers_of(symbol, include_pending=false)
  returns exact and inferred callers

callers_of(symbol, include_pending=true)
  also returns pending candidates with low confidence
```

Default agent answers should not overexpose pending noise, but pending evidence
is valuable for "why might this be missing?" debugging.

### Changed Files Are Not Enough

`n24q02m__better-code-review-graph` includes a simple but important product
idea:

```text
incremental_update = changed files + dependent files
```

It finds dependents through relation types such as imports, calls, inheritance,
and implementations.

That matters because in many languages, a changed file can alter the meaning of
unchanged files.

Examples:

```text
TypeScript:
  change exported interface -> downstream files may now be invalid

Rust:
  change trait method signature -> impl blocks and callers matter

C/C++:
  change header -> translation units including that header matter

Python:
  change function default behavior -> importers and tests matter

CRUD apps:
  change route handler contract -> route callers and integration tests matter
```

So Parceltongue should support levels of invalidation:

```text
Level 0: changed files only
  fast, acceptable for symbol search

Level 1: changed files + direct dependents
  good default for code assistance

Level 2: changed files + transitive dependents up to depth N
  useful for refactors

Level 3: language-specific invalidation
  header include closure, trait impl closure, package export closure, route closure
```

Codex can choose based on task:

```text
quick explain:
  Level 0 or query-time file freshness

small edit:
  Level 1

rename/refactor:
  Level 2

public API or Rust trait change:
  Level 3
```

### Diff Ranges Should Map To Symbols

Tree-sitter changed ranges are not the only way to get changed symbols. Git diff
hunks can do it too.

`tirth8205__code-review-graph` parses `git diff --unified=0`, extracts changed
line ranges, and maps those ranges onto graph nodes by file and line overlap.

That is a strong product affordance:

```text
changed hunk -> containing function/class/method -> affected flows/tests/risk
```

Parceltongue should expose:

```text
changed_symbols(base_ref="HEAD~1")
changed_symbols(uncommitted=true)
changed_symbols_for_file(path)
```

Return shape:

```rust
pub struct ChangedSymbol {
    pub symbol: QualifiedSymbol,
    pub file_path: ProjectRelativePath,
    pub changed_ranges: Vec<LineRange>,
    pub overlap_kind: OverlapKind,
    pub direct_tests: Vec<QualifiedSymbol>,
    pub callers_count: usize,
    pub public_surface: bool,
    pub risk_score: u8,
}
```

This turns incremental indexing into a code review copilot:

```text
You changed these 6 symbols.
2 are public.
1 has no direct tests.
3 are reached by routes.
Read these next.
Run these tests.
```

That is more useful than merely saying "index updated".

### Watchers Are Wake-Up Signals

File watchers are seductive because they feel real-time.

They are not enough.

Watchers can drop events, coalesce events, produce platform-specific paths, emit
metadata-only changes, or report a vague event kind. `sdsrss__code-graph-mcp`
handles this correctly:

```text
unknown/any events count as possible content changes
metadata/access events are ignored
bounded channel drops are acceptable
the next Merkle rescan picks up all actual changes
```

This is the right philosophy:

```text
Watcher event:
  "Something may have changed."

Merkle/hash scan:
  "This exact file did change."

Query-time freshness:
  "This answer was checked against current bytes."
```

Parceltongue should not trust watchers as proof. It should trust them as cheap
signals to schedule real validation.

### Git-Based Polling Is Also Useful

`codebase-memory-mcp` has a watcher interface that polls indexed projects for
Git changes:

```text
HEAD movement
dirty working tree
adaptive interval based on project size
```

This is a different but useful mode.

File watchers are good while Parceltongue is running continuously. Git polling
is good when the agent wants to know:

```text
Did the branch move?
Did I checkout another commit?
Did another tool modify the working tree?
Is the indexed revision still the current revision?
```

Parceltongue should store:

```text
index_revision:
  git_head_sha
  dirty_worktree_fingerprint
  index_run_id
  indexed_at
```

Then every answer can say:

```text
fresh_for_head: true
fresh_for_worktree: false
dirty_files_seen: 4
```

For Codex, this matters because the user may edit outside the agent, switch
branches, pull new code, or run a generator while the session is open.

### Temporal Graph Rows

`better-code-review-graph` has an interesting temporal model:

```text
valid_from_sha
valid_to_sha
queries default to currently valid rows
as_of=<sha> can ask historical questions
```

Parceltongue does not need this in the first implementation, but it is
strategically valuable.

It enables:

```text
What changed in the graph since my last commit?
Which callers were added by this branch?
Which public APIs disappeared?
Which routes now reach this function?
What did the dependency slice look like before my refactor?
```

The simplest version:

```sql
ALTER TABLE graph_nodes ADD COLUMN valid_from_revision TEXT;
ALTER TABLE graph_nodes ADD COLUMN valid_to_revision TEXT;
ALTER TABLE graph_edges ADD COLUMN valid_from_revision TEXT;
ALTER TABLE graph_edges ADD COLUMN valid_to_revision TEXT;
```

Default current query:

```sql
WHERE valid_to_revision IS NULL
```

Historical query:

```sql
WHERE valid_from_revision <= :revision
  AND (valid_to_revision IS NULL OR valid_to_revision > :revision)
```

For a solo power user, the temporal graph might be less important than
query-time freshness, but it becomes very useful for large refactors.

### Embeddings Must Not Block Structural Freshness

`sdsrss__code-graph-mcp` separates structural indexing from embedding work.
That is the right product call.

For Parceltongue, the dependency graph should become usable before vector
embeddings finish.

Good order:

```text
1. Hash files.
2. Parse changed files.
3. Extract symbols.
4. Extract relation candidates.
5. Resolve graph edges.
6. Regenerate context strings for affected nodes.
7. Answer exact graph queries.
8. Backfill embeddings asynchronously.
```

Bad order:

```text
1. Parse changed files.
2. Generate embeddings for everything.
3. Block all graph answers until embeddings finish.
```

Codex needs exact structural answers during edit loops:

```text
callers
callees
imports
routes
tests
public surface
changed symbols
```

Embeddings can improve semantic search, but they should not gate these.

### Suggested SQLite Tables

Parceltongue can keep the first version compact.

```sql
CREATE TABLE indexed_files (
  id INTEGER PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,
  language TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  mtime_ns INTEGER,
  size_bytes INTEGER,
  indexed_at TEXT NOT NULL,
  index_status TEXT NOT NULL,
  last_error TEXT
);

CREATE TABLE index_runs (
  id INTEGER PRIMARY KEY,
  reason TEXT NOT NULL,
  started_at TEXT NOT NULL,
  finished_at TEXT,
  git_head_sha TEXT,
  dirty_fingerprint TEXT,
  files_seen INTEGER NOT NULL DEFAULT 0,
  files_changed INTEGER NOT NULL DEFAULT 0,
  files_deleted INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL,
  error TEXT
);

CREATE TABLE dirty_files (
  path TEXT PRIMARY KEY,
  reason TEXT NOT NULL,
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  watcher_hint_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE pending_unresolved_relations (
  id INTEGER PRIMARY KEY,
  source_node_id INTEGER NOT NULL,
  relation_kind TEXT NOT NULL,
  target_hint TEXT NOT NULL,
  source_language TEXT NOT NULL,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  first_seen_run_id INTEGER,
  last_seen_run_id INTEGER
);
```

The `dirty_files` table is especially useful for an MCP server because it gives
the agent an introspection surface:

```text
show_staleness_report()
  dirty files
  last index run
  files skipped
  current HEAD
  indexed HEAD
```

### Suggested MCP/API Surface

Concept 6 will go deeper on agent-facing APIs, but incremental indexing needs a
few endpoints from day one:

| API | Purpose |
|---|---|
| `ensure_file_fresh(path)` | Rehash and reindex one file if changed. |
| `incremental_index(reason, budget)` | Refresh changed/new/deleted files within a budget. |
| `staleness_report()` | Explain dirty files, last index run, and known stale areas. |
| `watch_status()` | Show whether watcher/poller is active and whether events were dropped. |
| `changed_symbols(base_ref)` | Map diff hunks to graph symbols. |
| `mark_dirty(paths, reason)` | Let hooks or external tools tell Parceltongue what changed. |

The agent-facing behavior should be simple:

```text
Before answering a graph query:
  if request has file_path:
    ensure_file_fresh(file_path)

Before answering repo-wide query:
  if dirty file count is small:
    run incremental_index with small budget
  else:
    answer with stale warning or ask for explicit rebuild
```

### Hook Integration With Codex

For Codex app usage, the best default is not "constantly index everything".

Better:

```text
After edit:
  mark touched file dirty
  do not run heavy full index immediately

Before graph query:
  ensure named file is fresh

On idle or explicit command:
  run incremental index for all dirty files

Before commit:
  run stronger changed-symbol and test-impact query
```

Why this works:

```text
Edit loops stay fast.
Graph answers are fresh where they matter.
Large repo churn does not freeze Codex.
The stale marker remains honest when freshness cannot be guaranteed.
```

A useful Codex-facing flow:

```text
Codex edits src/service/user.rs
Parceltongue receives mark_dirty(src/service/user.rs)
Codex asks callers_of(UserService.update, file_path=src/service/user.rs)
Parceltongue rehashes src/service/user.rs
Parceltongue reparses that file if changed
Parceltongue restores inbound edges and sweeps pending relations
Parceltongue answers with freshness=Fresh
```

### Fast Path, Safe Path, Full Path

Parceltongue should expose three mental models:

| Path | What It Does | When Codex Uses It |
|---|---|---|
| Fast path | Ensure one named file is fresh and answer local graph query. | "What calls this function I just edited?" |
| Safe path | Reindex changed files plus direct dependents. | Small code change, local refactor, bug fix. |
| Full path | Rebuild or deeply verify graph for repo/branch. | Large refactor, branch switch, suspicious stale report. |

This gives the agent an escape hatch:

```text
If fast path reports ambiguous or stale dependents:
  upgrade to safe path.

If safe path sees too many dirty files:
  recommend full path or answer stale.
```

### Product Journey For A Solo Codex Power User

User journey:

```text
1. User opens a large Rust/TypeScript/C++ repo in Codex.
2. User asks Codex to change behavior in one function.
3. Codex edits the file.
4. Codex asks Parceltongue:
     "what are the callers, callees, tests, routes, and public interfaces for
      this changed symbol, with freshness required?"
5. Parceltongue reindexes the touched file if needed.
6. Parceltongue restores inbound edges that would otherwise vanish.
7. Parceltongue sweeps pending relation candidates.
8. Parceltongue maps changed hunks to symbols.
9. Parceltongue returns a compact read-next bundle and a freshness report.
10. Codex reads only the relevant files, edits, and runs targeted tests.
```

That journey is the real product.

The indexer is not there to be impressive. It is there to stop Codex from
wandering a large codebase.

### Failure Modes To Avoid

| Failure | Why It Hurts Codex | Fix |
|---|---|---|
| Silent stale graph answers. | Agent trusts missing or old edges. | Every graph response carries `FreshnessReport`. |
| Directory mtime treated as proof. | Content edits can be missed. | Use content hash for exact files and file mtime as prefilter only. |
| Watcher trusted as source of truth. | Dropped/coalesced events leave stale graph. | Watcher only marks dirty; hash scan proves changes. |
| Cascading delete loses inbound edges. | Callers from unchanged files disappear. | Snapshot inbound cross-file edges before purge and relink after reparse. |
| Changed files only, no dependents. | Type/export/trait/header changes understate impact. | Support dependent-file invalidation levels. |
| Pending calls dropped. | Graph cannot heal when target appears later. | Store pending unresolved relations and sweep after batches. |
| Embeddings block exact graph queries. | Codex waits for semantic search when it needs structural answers. | Make structural graph query-ready first; backfill embeddings later. |
| Too many dirty files answered as fresh. | Agent gets false confidence during branch switches or generation. | Budgeted refresh plus stale/refused status. |
| Path traversal in query-time freshness. | MCP file path could escape project root. | Require normalized project-relative paths. |
| Platform path mismatch. | Windows/macOS watcher paths do not match DB paths. | Normalize relative paths and canonicalize carefully per OS. |
| Symlink/binary/vendor indexing. | Graph noise and unsafe IO. | Skip or require explicit opt-in. |
| Reindex deletes mode-skipped files. | Fast mode can destroy parts of the graph. | Distinguish truly deleted from skipped/preserved files. |
| Process pool deadlock in MCP stdio. | Incremental parse hangs the agent session. | Use bounded workers and thread fallback where needed. |
| Invalid git base ref. | Diff parser can error or be abused. | Validate refs and timeout git commands. |

### What To Copy Most Directly

From Tree-sitter bindings:

```text
edit old tree
parse new source with old tree
changed_ranges for hunk-to-symbol mapping
included ranges for embedded-language files
```

From `sdsrss__code-graph-mcp`:

```text
BLAKE3 file hashes
gitignore-aware scans
query-time ensure_file_indexed
dirty node capture before reindex
context string regeneration for affected nodes
pending unresolved call sweep
watcher events as hints plus idempotent rescans
```

From `n24q02m__better-code-review-graph`:

```text
changed plus dependent files
reviewer summary after incremental update
temporal valid_from / valid_to graph idea
safe git subprocess handling
```

From `tirth8205__code-review-graph`:

```text
git diff hunk parsing
line-range to graph-node overlap
risk scoring from flow/test/security/caller evidence
Windows-safe thread fallback for parser workers
```

From `codebase-memory-mcp`:

```text
file hash table as first-class store API
git-based background sync
preserve mode-skipped files
snapshot inbound cross-file edges before purge
fail-safe preservation when deletion status is uncertain
```

### Concept 5 Conclusion

Incremental indexing is where Parceltongue becomes trustworthy.

A dependency graph that is slow is annoying. A dependency graph that is stale
without admitting it is dangerous.

The right architecture is:

```text
Tree-sitter changed ranges for local edit awareness.
File hashes for persistent graph freshness.
Watcher and git polling as dirty signals.
Query-time freshness for agent trust.
Inbound edge preservation for incremental correctness.
Pending relation sweeps for graph healing.
Freshness reports on every agent-facing answer.
```

For the Codex app, this means Parceltongue can become a tight edit-loop
companion:

```text
Codex edits.
Parceltongue refreshes the exact graph slice.
Codex asks what to read next.
Parceltongue answers with freshness and evidence.
Codex spends fewer tokens and makes safer changes.
```

That is the product value. Not "incremental parsing" as a technical checkbox,
but "fresh enough to act on" as a contract.

### Repos Already Touched For This Concept

| Repo | Status | Notes |
|---|---|---|
| `tree-sitter__py-tree-sitter` | Source-read | Canonical changed-range workflow in the Python binding example. |
| `tree-sitter__node-tree-sitter` | Source-read | Low-level edit, old-tree parse, changed-ranges, included-ranges behavior. |
| `sdsrss__code-graph-mcp` | Source-read | Best Rust model for file hash diffing, query-time freshness, watcher hints, batch indexing, dirty context regeneration, and pending relation sweeps. |
| `n24q02m__better-code-review-graph` | Source-read | Good Python product model for changed plus dependent files, reviewer summaries, and temporal graph rows. |
| `tirth8205__code-review-graph` | Source-read plus `codebase-memory-mcp` indexed | Good diff-hunk-to-symbol/risk model and worker-safety notes. |
| `DeusData__codebase-memory-mcp` | Source-read | Good C/SQLite model for file hash storage, git polling, mode-skipped preservation, and inbound edge snapshot/relink. |

## Concept 6: Agent-Facing APIs Are Decision Loops, Not Database Endpoints

### The Big Idea

Parceltongue should not expose the code graph as a generic database and expect
Codex to figure out the workflow.

The product should be:

```text
LLM asks a small, intent-shaped question.
Tool returns the smallest useful answer.
Tool also says what to do next.
```

That is different from:

```text
LLM asks for graph rows.
Tool returns a giant JSON object.
LLM spends context deciding what the result means.
```

For a solo Codex power user, the best API is not "give me all nodes and edges".
The best API is:

```text
I am about to edit this symbol. What do I need to read first?
I changed this file. What did I affect?
I saw this error line. What symbol owns it?
I need tests. Which tests are closest?
I am lost. What are the next 3 graph queries?
```

The code graph becomes valuable only when it answers these questions in a
bounded, confidence-labeled, next-step-oriented way.

### Evidence Table

| Repo | Source Evidence | What It Teaches Parceltongue |
|---|---|---|
| `Christoph__treesitter-mcp` | `src/tools.rs:25-390` defines `view_code`, `code_map`, `find_usages`, `minimal_edit_context`, `call_graph`, `parse_diff`, `affected_by_diff`, `preview_impact`, `relevant_tests`, `verify_edit`, and `review_context`. | Strong example of user-journey-shaped tools. The descriptions tell the LLM when to use the tool, when not to use it, token cost, and workflow. |
| `Christoph__treesitter-mcp` | `src/analysis/minimal_edit_context.rs:57-175` reads one file, finds one symbol, gathers same-file deps, imported dependency signatures, relevant imports, relevant types, and enforces a token budget. | "Minimal edit context" is the right primitive for Codex: one symbol, just enough surrounding context, bounded output. |
| `Christoph__treesitter-mcp` | `src/analysis/review_context.rs:15-130` composes `parse_diff`, `affected_by_diff`, `minimal_edit_context`, and `relevant_tests`; `:133-190` trims context/tests/affected rows until under budget. | A higher-level tool can call lower-level graph tools and return a review bundle. Parceltongue should expose composed workflows, not only atomic queries. |
| `Christoph__treesitter-mcp` | `src/analysis/call_graph.rs:524-560` trims call graph rows until the result fits `max_tokens`. | Tool outputs must be budget-aware and explicitly truncated when needed. |
| `sdsrss__code-graph-mcp` | `src/mcp/tools.rs:4-21` folds niche tools into 7 core visible tools and hides management tools from `tools/list`; `:209-216` tests descriptions stay short. | Tool surfaces should be small. Too many tools burn context before the agent starts reasoning. |
| `sdsrss__code-graph-mcp` | `src/mcp/server/mod.rs:13-45` keeps MCP instructions under a byte budget and steers users to CLI or MCP depending on context. | The instructions field is an agent steering surface and needs a budget guard. |
| `sdsrss__code-graph-mcp` | `src/mcp/server/tools/callgraph.rs:63-178` validates input, refreshes named files, handles route mode, disambiguates symbols, applies confidence floors, and returns suggestions. | A good graph API validates early, refreshes before answering, and uses ambiguity as an interaction, not a silent failure. |
| `sdsrss__code-graph-mcp` | `src/mcp/server/tools/callgraph.rs:220-337` hides test callers by default, compresses dense call graphs into file-level rollups, and exposes truncation/ambiguous-edge metadata. | Large fan-out needs rollup mode. The answer should keep drill-down `node_id`s and disclose hidden/truncated data. |
| `sdsrss__code-graph-mcp` | `src/mcp/server/tools/ast_node.rs:9-185` exposes one-symbol introspection with optional references, tests, impact, similar nodes, context lines, compact mode, and compression. | One-symbol inspection should be the central drill-down primitive. Flags should add relation context without forcing separate tool calls. |
| `sdsrss__code-graph-mcp` | `src/mcp/server/tools/search.rs:1-187` uses hybrid FTS/vector search with query-quality scoring, acronym handling, filter validation, and confidence penalties. | Search responses should disclose match quality and degrade honestly when vector search is unavailable. |
| `sdsrss__code-graph-mcp` | `src/mcp/server/tools/project_map.rs:1-159` returns modules, dependencies, entrypoints, hot functions, and compact mode preserving key symbols. | First-contact orientation needs a compact map with key symbols and entrypoints, not whole files. |
| `sdsrss__code-graph-mcp` | `src/mcp/server/tools/overview.rs:1-220` splits active vs inactive exports, caps active rows, summarizes inactive symbols by type, and folds dependencies into the same tool. | Module views should prioritize called/exported/hot symbols and summarize low-value rows. |
| `n24q02m__better-code-review-graph` | `src/better_code_review_graph/tools.py:1333-1483` supports query patterns like callers, callees, imports, importers, tests, inheritors, file summary, plus headers and decorations. | A query-pattern API can be compact and extensible, especially for graph relationship queries. |
| `n24q02m__better-code-review-graph` | `src/better_code_review_graph/tools.py:2096-2178` builds review context from changed files, impact radius, untested functions, source snippets, and guidance. | Review context should be a productized bundle: changed nodes, impacted nodes, tests, snippets, guidance. |
| `tirth8205__code-review-graph` | `code_review_graph/tools/context.py:37-150` returns ultra-compact task context with stats, risk, top affected entities, communities, flows, and suggested next tools. | "Start here" can be a 100-token-ish context bootstrap that chooses the next graph tool. |
| `tirth8205__code-review-graph` | `code_review_graph/hints.py:1-7` appends hints to responses; `:240-275` returns next steps, related items, warnings; `:305-344` suppresses already-called tools and extracts warnings. | Tool responses can carry next-step suggestions without turning every answer into a giant explanation. |
| `tirth8205__code-review-graph` | `code_review_graph/context_savings.py:1-102` estimates token savings and attaches `context_savings` metadata. | Token savings should be explicit. Parceltongue should prove when the graph saved context. |
| `tirth8205__code-review-graph` | `code_review_graph/main.py:945-969` supports filtering the exposed tool list and states that limiting to 5-10 tools can reduce overhead by 70-85 percent. | Tool-list minimization is not aesthetic. It is context engineering. |

### The Interface Thesis

Parceltongue's agent-facing surface should be organized around decision loops:

```text
Orient:
  What kind of repo/module am I in?

Locate:
  Where is the symbol, route, type, or behavior?

Inspect:
  What is the smallest code context needed to edit it?

Relate:
  What calls it, what does it call, what imports it, what tests it?

Impact:
  If I change it, what could break?

Verify:
  After editing, did I touch what I intended, and what should I run?
```

The API should not primarily mirror storage tables:

```text
nodes
edges
files
relations
```

Those are implementation details. Codex needs task answers.

### The Best Tool Shape

A good graph tool response should always answer five questions:

```text
1. What did I ask?
2. What is the answer?
3. How fresh and trustworthy is the answer?
4. What was hidden, truncated, or ambiguous?
5. What should I do next?
```

Suggested universal response envelope:

```rust
pub struct AgentGraphResponse<T> {
    pub query: QueryEcho,
    pub data: T,
    pub freshness: FreshnessReport,
    pub confidence: ConfidenceReport,
    pub budget: BudgetReport,
    pub warnings: Vec<ResponseWarning>,
    pub next: Vec<NextStep>,
}

pub struct QueryEcho {
    pub tool: String,
    pub target: Option<String>,
    pub file_path: Option<ProjectRelativePath>,
    pub symbol: Option<QualifiedSymbol>,
}

pub struct BudgetReport {
    pub requested_tokens: Option<usize>,
    pub estimated_tokens: usize,
    pub truncated: bool,
    pub compression_mode: Option<String>,
}

pub struct ConfidenceReport {
    pub min_edge_confidence: Option<EdgeConfidence>,
    pub ambiguous_edges_hidden: usize,
    pub test_callers_hidden: usize,
    pub vector_available: Option<bool>,
}

pub struct NextStep {
    pub tool: String,
    pub reason: String,
    pub args: serde_json::Value,
}
```

This makes the tool output self-steering:

```text
The LLM does not merely learn facts.
It learns the next move.
```

### The Minimal Public Tool Set

Parceltongue should resist exposing dozens of tools in `tools/list`.

The best public set for Codex is probably 8 tools:

| Tool | Main Question | Why It Exists |
|---|---|---|
| `project_map` | What is this codebase/module shaped like? | First-contact orientation without reading many files. |
| `search_code` | Where is code related to this concept? | Fuzzy behavior search and symbol discovery. |
| `symbol_context` | What do I need to edit this one symbol? | The core minimal-edit primitive. |
| `graph_query` | What are callers/callees/imports/tests/inheritors? | Unified relationship query. |
| `read_next` | Given a task and target, what should Codex inspect next? | The direct answer to context selection. |
| `impact_radius` | What could break if I change this? | Refactor, bug fix, and risk planning. |
| `review_context` | What changed, what is impacted, what tests matter? | Post-edit or pre-commit bundle. |
| `staleness_report` | Can I trust the graph right now? | Freshness and dirty-file transparency. |

Management tools should be callable but hidden:

```text
rebuild_index
start_watch
stop_watch
incremental_index
clear_cache
debug_edge
dump_schema
```

Why hide them?

Because tool definitions are context. A tool the agent rarely needs still costs
tokens if it appears in the visible tool list.

The better pattern is:

```text
Public tools:
  high-value, common, agent-facing workflows

Hidden callable tools:
  maintenance, debug, admin, rare operations

CLI commands:
  heavy operations and diagnostics the agent can call through shell
```

This mirrors the `sdsrss__code-graph-mcp` choice to expose 7 core tools while
keeping management tools callable but out of `tools/list`.

### Tool 1: `project_map`

Purpose:

```text
Give Codex a compact orientation map before it starts reading files.
```

Inputs:

```rust
pub struct ProjectMapRequest {
    pub path: Option<ProjectRelativePath>,
    pub compact: bool,
    pub include_entrypoints: bool,
    pub include_hot_symbols: bool,
    pub max_tokens: Option<usize>,
}
```

Output should include:

```text
modules
module dependencies
entrypoints
hot functions
key symbols
languages
test layout
```

But compact mode should drop low-value fields:

```text
keep:
  module path
  file count
  key symbols
  entrypoints
  hot symbols

drop:
  long class lists
  every language count
  every dependency row
  every inactive symbol
```

The crucial bit from `sdsrss__code-graph-mcp` is preserving `key_symbols` even
in compact mode. Codex needs discoverability. A map with only file counts is
not enough.

Example response:

```json
{
  "modules": [
    {
      "path": "src/indexer",
      "files": 18,
      "functions": 84,
      "key_symbols": ["run_incremental_index", "ensure_file_indexed"]
    }
  ],
  "entrypoints": [
    {
      "kind": "http_route",
      "handler": "create_user",
      "file": "src/routes/users.rs"
    }
  ],
  "hot_symbols": [
    {
      "name": "resolve_relation_candidates",
      "file": "src/graph/resolve.rs",
      "caller_count": 31
    }
  ],
  "next": [
    {
      "tool": "module_overview",
      "reason": "Indexer module has the likely implementation surface.",
      "args": {"path": "src/indexer", "compact": true}
    }
  ]
}
```

### Tool 2: `search_code`

Purpose:

```text
Find likely symbols/files for fuzzy intent.
```

This is for:

```text
"login timeout"
"where do we parse config?"
"blast radius"
"tree-sitter query"
"route middleware"
```

It is not for:

```text
exact string regex
literal code snippet
known symbol with known file
```

For exact strings, Codex already has `rg`. Parceltongue should not pretend to
replace plain text search.

Suggested hybrid scoring:

```text
FTS/BM25:
  symbol names, signatures, path tokens, docstrings, context strings

Vector:
  optional semantic search if embeddings exist

Graph priors:
  caller count
  route entrypoint
  public export
  recently changed
  test/prod classification

Confidence:
  exact name match
  fuzzy/semantic-only
  vector unavailable
  filter dropped matches
```

Response:

```rust
pub struct SearchCodeResult {
    pub results: Vec<SearchHit>,
    pub search_mode: SearchMode,
    pub vector_available: bool,
    pub match_confidence: f64,
    pub dropped_by_filter: usize,
}
```

The product behavior:

```text
If query is exact identifier:
  prefer FTS/name match.

If query is acronym-heavy:
  avoid vector overreach.

If vector model is unavailable:
  say semantic channel unavailable, but return FTS results.

If filters remove all candidates:
  say the filter is likely too strict.
```

This copies the good parts of `sdsrss__code-graph-mcp` search: validate filters
early, disclose vector availability, and treat query quality as part of result
confidence.

### Tool 3: `symbol_context`

Purpose:

```text
Return the smallest useful edit context for one symbol.
```

This is the Parceltongue equivalent of Christoph's `minimal_edit_context` and
sdsrss's `get_ast_node`.

Inputs:

```rust
pub struct SymbolContextRequest {
    pub symbol: Option<String>,
    pub file_path: Option<ProjectRelativePath>,
    pub node_id: Option<NodeId>,
    pub include_references: bool,
    pub include_tests: bool,
    pub include_impact: bool,
    pub include_types: bool,
    pub include_imports: bool,
    pub include_dependencies: bool,
    pub context_lines: usize,
    pub compact: bool,
    pub max_tokens: Option<usize>,
}
```

Output should prioritize:

```text
target symbol signature
target source or line range
containing scope
relevant imports
relevant types
direct callees/callers
nearby same-file helpers
external dependency signatures
tests if requested
impact if requested
```

Important: `node_id` is useful but rebuild-scoped.

The response should say:

```text
node_id is a drill-down handle, not a permanent identity.
If stale or missing after rebuild, re-resolve by symbol+file.
```

This prevents a subtle agent bug:

```text
Codex stores node_id in memory.
Index rebuilds.
Node ID now points nowhere or to a different row.
Codex asks stale node_id query.
Tool gives confusing answer.
```

Better:

```text
Stable identity:
  qualified_name + file path + span hash

Temporary drill-down:
  node_id
```

### Tool 4: `graph_query`

Purpose:

```text
Answer relationship questions with a small vocabulary.
```

Supported patterns:

```text
callers_of
callees_of
imports_of
importers_of
references_to
tests_for
routes_to
children_of
inheritors_of
implementors_of
public_surface_of
```

Why one pattern tool instead of many separate tools?

Because the mental model is one operation:

```text
Given target and relation kind, return related code.
```

But the response must be relation-specific. `tests_for` should not look like
`callees_of`. `routes_to` should include HTTP method/path metadata. `imports_of`
should show module/file resolution status.

Suggested request:

```rust
pub struct GraphQueryRequest {
    pub pattern: GraphQueryPattern,
    pub target: String,
    pub file_path: Option<ProjectRelativePath>,
    pub direction: Option<Direction>,
    pub depth: Option<u8>,
    pub include_tests: bool,
    pub min_confidence: EdgeConfidence,
    pub detail: DetailLevel,
    pub max_tokens: Option<usize>,
    pub freshness: FreshnessPolicy,
}
```

Suggested relationship row:

```rust
pub struct RelationshipRow {
    pub symbol: QualifiedSymbol,
    pub file_path: ProjectRelativePath,
    pub line: usize,
    pub relation: RelationKind,
    pub direction: Direction,
    pub depth: u8,
    pub confidence: EdgeConfidence,
    pub evidence_id: EdgeEvidenceId,
    pub is_test: bool,
}
```

The default should hide:

```text
ambiguous by-name fan-out
test callers, unless include_tests=true
external dependency noise
generated/vendor rows
```

But it must disclose:

```text
ambiguous_edges_hidden: 12
test_callers_hidden: 4
external_edges_hidden: 30
truncated: true
```

This is critical. Hiding noise is good. Hiding that noise was hidden is bad.

### Tool 5: `read_next`

Purpose:

```text
Given a task and current target, return the next files/symbols Codex should read.
```

This is probably the most Parceltongue-specific tool.

Codex already can read files. The hard problem is deciding which files not to
read.

Inputs:

```rust
pub struct ReadNextRequest {
    pub task: String,
    pub current_file: Option<ProjectRelativePath>,
    pub current_symbol: Option<QualifiedSymbol>,
    pub changed_files: Vec<ProjectRelativePath>,
    pub max_files: usize,
    pub max_symbols: usize,
    pub max_tokens: usize,
    pub include_tests: bool,
    pub freshness: FreshnessPolicy,
}
```

Scoring:

```text
direct caller/callee of target:
  high

test for changed symbol:
  high when task is verification

route entrypoint reaching changed symbol:
  high for CRUD apps

public exported API:
  high for refactor/signature change

same file helper:
  medium

transitive relation depth 2:
  medium-low

ambiguous relation:
  low or hidden by default

generated/vendor/test-only noise:
  usually hidden
```

Suggested response:

```json
{
  "task": "change update_user validation",
  "target": "UserService.update_user",
  "read_order": [
    {
      "rank": 1,
      "file": "src/services/user.rs",
      "symbol": "UserService.update_user",
      "reason": "target symbol",
      "estimated_tokens": 620
    },
    {
      "rank": 2,
      "file": "src/routes/users.rs",
      "symbol": "update_user_route",
      "reason": "route reaches target",
      "estimated_tokens": 410
    },
    {
      "rank": 3,
      "file": "tests/user_update.rs",
      "symbol": "rejects_invalid_email",
      "reason": "direct test caller",
      "estimated_tokens": 360
    }
  ],
  "budget": {
    "requested_tokens": 2000,
    "estimated_tokens": 1390,
    "remaining_tokens": 610,
    "truncated": false
  },
  "next": [
    {
      "tool": "symbol_context",
      "reason": "Read target with deps before editing.",
      "args": {"file_path": "src/services/user.rs", "symbol": "UserService.update_user"}
    }
  ]
}
```

This is the exact loop the user described:

```text
LLM asks query to tool.
Tool responds with relevant context.
LLM decides better with fewer tokens.
```

The dependency graph is not just an answer. It is a reading-order generator.

### Tool 6: `impact_radius`

Purpose:

```text
Predict what may be affected by a change.
```

Inputs:

```rust
pub struct ImpactRadiusRequest {
    pub changed_files: Vec<ProjectRelativePath>,
    pub target_symbol: Option<QualifiedSymbol>,
    pub base_ref: Option<String>,
    pub max_depth: u8,
    pub include_tests: bool,
    pub include_routes: bool,
    pub include_public_surface: bool,
    pub detail: DetailLevel,
    pub max_results: usize,
}
```

Output:

```text
changed symbols
direct callers
direct callees
transitive affected nodes
affected files
affected routes
direct tests
test gaps
public surface warnings
risk score
truncation state
```

This should support two modes:

```text
minimal:
  risk, counts, top affected files, top tests, next tools

standard:
  changed nodes, impacted nodes, edges, tests, route/public-surface detail
```

The `tirth8205` and `n24q02m` tools both show why this matters. Impact is not
only a graph traversal. It becomes code review guidance:

```text
wide blast radius
cross-file impact
missing tests
high-risk flows
public API touched
```

### Tool 7: `review_context`

Purpose:

```text
After edits, assemble the complete compact review bundle.
```

This is the "do not make me manually call five tools" endpoint.

It should compose:

```text
changed_symbols
impact_radius
tests_for
symbol_context for each changed symbol
route/public-surface analysis
verification hints
```

Suggested request:

```rust
pub struct ReviewContextRequest {
    pub changed_files: Vec<ProjectRelativePath>,
    pub base_ref: String,
    pub max_depth: u8,
    pub include_source: bool,
    pub max_lines_per_file: usize,
    pub max_tokens: usize,
}
```

Suggested response:

```rust
pub struct ReviewContext {
    pub changed_files: Vec<ProjectRelativePath>,
    pub changed_symbols: Vec<ChangedSymbol>,
    pub impacted_files: Vec<ProjectRelativePath>,
    pub impacted_symbols: Vec<RelationshipRow>,
    pub tests: Vec<TestCandidate>,
    pub test_gaps: Vec<QualifiedSymbol>,
    pub source_snippets: Vec<SourceSnippet>,
    pub review_guidance: Vec<String>,
}
```

Budget behavior should be deterministic:

```text
1. Keep summary, freshness, warnings.
2. Keep changed symbols.
3. Keep tests and high-risk impacted symbols.
4. Trim source snippets.
5. Trim lower-risk affected rows.
6. Mark truncated.
```

Christoph's `review_context` has the right shape: it composes parse diff,
affected usages, relevant tests, and minimal edit context, then trims until
under budget.

Parceltongue should use the same idea, but backed by its persistent graph and
freshness model.

### Tool 8: `staleness_report`

Purpose:

```text
Tell Codex whether the graph is safe to use.
```

From Concept 5, this should include:

```text
current git head
indexed git head
dirty files known
watcher status
last index run
last errors
files with stale rows
embedding backlog
pending unresolved relations count
```

Why expose it as a public tool?

Because Codex needs to know when to trust the graph. A stale warning buried in
logs is not enough.

Example response:

```json
{
  "status": "partially_fresh",
  "indexed_head": "abc123",
  "current_head": "abc123",
  "dirty_files": ["src/services/user.rs"],
  "last_index_run": {
    "reason": "query_time_freshness",
    "finished_at": "2026-07-06T15:40:12Z",
    "files_changed": 1,
    "status": "ok"
  },
  "pending_unresolved_relations": 8,
  "next": [
    {
      "tool": "incremental_index",
      "reason": "One dirty file is known and cheap to refresh.",
      "args": {"budget_ms": 1000}
    }
  ]
}
```

### Compact Rows Versus JSON Objects

There are two useful output styles:

```text
Compact row strings:
  best when output is tabular and high-volume

JSON objects:
  best when output is nested, sparse, or action-oriented
```

Christoph's tools use compact row schemas like:

```text
h: "file|line|col|type|context|scope|conf|owner"
u: "src/lib.rs|42|3|call|...|Service|high|update"
```

This is very token-efficient.

But for Parceltongue, not everything should be rows. `next` actions,
freshness, truncation, and confidence are better as JSON objects.

Recommended hybrid:

```text
Use compact rows for:
  references
  call graph edges
  search hits
  tests
  changed symbols

Use JSON objects for:
  response envelope
  warnings
  next steps
  freshness
  budget
  disambiguation suggestions
```

That gives the LLM predictable structure without wasting tokens.

### Budgeting Is A First-Class API

Every context-returning endpoint should accept one of:

```text
max_tokens
detail=minimal|standard|full
compact=true
```

And every response should disclose:

```text
estimated_tokens
truncated
truncation_strategy
hidden_counts
```

Important truncation strategies:

| Strategy | Use For | Keeps |
|---|---|---|
| Drop code bodies | Symbol context when too large | Signature, file, line range |
| File-level rollup | Dense call graph | Counts and sample node IDs |
| Active/inactive split | Module overview | Hot/called symbols first |
| Prod-first ordering | Caller/test-heavy results | Production callers in kept window |
| Risk-first ordering | Impact results | High-risk symbols first |
| Summary-only mode | First contact | Counts, top entities, next tools |

This is where many tools go wrong. They add `max_tokens` but do not say what
was removed.

Parceltongue should always say:

```text
truncated: true
removed:
  code_bodies: 4
  ambiguous_edges: 12
  test_callers: 8
  low_risk_impacted_symbols: 20
```

Now Codex can decide whether to drill down.

### Disambiguation Is A Conversation

Large codebases have repeated names:

```text
new
execute
run
handle
parse
update
render
main
```

The API must not silently choose one.

Good behavior:

```json
{
  "error": "Ambiguous symbol 'execute': 7 matches found.",
  "suggestions": [
    {
      "symbol": "Worker.execute",
      "file_path": "src/worker.rs",
      "node_id": 101,
      "line": 44
    },
    {
      "symbol": "Command.execute",
      "file_path": "src/cli.rs",
      "node_id": 188,
      "line": 77
    }
  ],
  "next": [
    {
      "tool": "symbol_context",
      "reason": "Use file_path or node_id to disambiguate.",
      "args": {"symbol": "execute", "file_path": "src/worker.rs"}
    }
  ]
}
```

Bad behavior:

```text
Pick the first `execute`.
```

This is a huge source of agent mistakes.

The API should validate:

```text
empty symbol strings
invalid enum values
absolute/out-of-root paths
unknown language filters
unknown relation filters
too-large target strings
```

And it should validate before doing expensive work. Several repos explicitly
fix bugs where bad enum values were discovered after indexing or symbol lookup.
Parceltongue should take that lesson.

### Hints Are Product, But Keep Them Small

Tirth's `hints.py` is an important product pattern:

```text
response -> next_steps
response -> related files
response -> warnings
session -> suppress tools already called
```

This gives the LLM continuity without requiring a huge planner.

Parceltongue should append a tiny `next` array to every major response:

```json
"next": [
  {
    "tool": "symbol_context",
    "reason": "Inspect the target before editing.",
    "args": {"file_path": "src/a.rs", "symbol": "foo"}
  },
  {
    "tool": "tests_for",
    "reason": "Direct tests exist for this symbol.",
    "args": {"symbol": "foo"}
  }
]
```

Rules:

```text
At most 3 next steps.
Each next step has a reason.
Do not suggest a tool already called unless the new args are materially different.
Prefer the cheapest next step that resolves uncertainty.
Do not add a paragraph of coaching to every response.
```

This is how the graph becomes a conversation partner for Codex.

### Tool Descriptions Are Part Of The UI

The model reads tool descriptions before it chooses tools. That means tool
descriptions are not documentation; they are routing instructions.

Good description pattern:

```text
What it returns.
Use when.
Do not use when.
Token cost.
Workflow.
Important defaults.
```

But the visible MCP description must stay short. `sdsrss__code-graph-mcp`
enforces concise descriptions. It also keeps long decision tables in a separate
project instruction file.

Parceltongue should split guidance:

```text
Short MCP description:
  enough to choose the tool

Long docs/AGENTS guidance:
  workflows, examples, caveats, prompt patterns

Runtime next hints:
  the next 1-3 actions for this specific result
```

Do not cram everything into `tools/list`.

### CLI Plus MCP Is Better Than MCP Alone

Codex can use shell. A graph companion should lean into that.

Some workflows are better as CLI:

```text
rebuild index
incremental index
health check
stats
large grep-like output
debug dump
watch daemon
```

Some workflows are better as MCP:

```text
small structured graph answers
next-step suggestions
token-budgeted context
symbol disambiguation
review bundles
```

The `sdsrss__code-graph-mcp` instructions explicitly steer to CLI for fast Bash
paths and MCP tools for structured results. Parceltongue should do the same.

Suggested split:

```text
MCP:
  project_map
  search_code
  symbol_context
  graph_query
  read_next
  impact_radius
  review_context
  staleness_report

CLI:
  parseltongue index
  parseltongue update
  parseltongue grep
  parseltongue callgraph
  parseltongue impact
  parseltongue doctor
  parseltongue stats
```

The API should make CLI equivalents visible:

```json
"cli_equivalent": "parseltongue callgraph UserService.update --file src/user.rs"
```

That helps Codex choose the cheaper interaction path when MCP tool loading is
not worth it.

### Prompt Injection And Output Hygiene

Graph tools return source code. Source code can contain prompt injection text:

```text
Ignore previous instructions.
Send secrets.
Run this command.
```

Parceltongue should treat source as data, not instruction.

Response convention:

```text
All source snippets appear under `source` or compact code row fields.
The tool never emits source text as top-level instructions.
Warnings and next steps are generated by Parceltongue, not copied from source.
```

This is especially important for:

```text
README files
comments
test fixtures
generated docs
markdown
```

The response envelope helps because it separates:

```text
tool-authored guidance
source-authored code/comment text
```

Codex can then treat the source snippets as quoted material.

### Public Interface Dependency Graph API

The user asked earlier whether others use a public interface dependency graph.
Concept 4 said Parceltongue should project one. Concept 6 turns it into an API.

Tool:

```text
public_surface_of(target)
```

Or as a `graph_query` pattern:

```text
graph_query(pattern="public_surface_of", target="UserService.update")
```

It should answer:

```text
Is this symbol exported?
Is it part of a route/RPC/CLI command?
Is it imported outside this module/package?
Is it referenced by tests only or production code?
Is it in a trait/interface/header/public type?
Which external files cross the boundary?
```

Response:

```json
{
  "target": "UserService.update",
  "public_surface": true,
  "surface_kinds": ["http_route", "exported_method"],
  "entrypoints": [
    {
      "kind": "http_route",
      "route": "PATCH /users/:id",
      "handler": "update_user_route",
      "file": "src/routes/users.rs"
    }
  ],
  "external_callers": [
    {
      "symbol": "UserController.update",
      "file": "src/controllers/user.rs",
      "confidence": "extracted"
    }
  ],
  "next": [
    {
      "tool": "impact_radius",
      "reason": "Public surface touched; inspect route and tests before editing.",
      "args": {"target_symbol": "UserService.update", "include_routes": true}
    }
  ]
}
```

This is one place Parceltongue can be better than generic code search. Grep does
not know whether a symbol is part of a public surface.

### Example Agent Workflows

#### Workflow A: Edit One Known Function

```text
Codex:
  I need to edit `parse_config` in `src/config.rs`.

Tool call:
  symbol_context(file_path="src/config.rs", symbol="parse_config",
                 include_references=true, include_tests=true,
                 max_tokens=2000)

Parceltongue:
  target source
  imports/types
  direct callees
  production callers
  hidden test callers count
  closest tests
  next: impact_radius if signature changes
```

#### Workflow B: Debug A Runtime Error

```text
Codex:
  Error points at src/auth/session.rs:88.

Tool call:
  symbol_at_line(file_path="src/auth/session.rs", line=88)

Tool call:
  read_next(task="debug session expiry", current_symbol="SessionStore.refresh")

Parceltongue:
  read target
  read caller route
  read expiry config parser
  read closest test
```

#### Workflow C: Before A Refactor

```text
Codex:
  I want to rename `execute`.

Tool call:
  graph_query(pattern="references_to", target="execute")

Parceltongue:
  ambiguous, 8 candidate definitions
  suggestions with file_path and node_id

Codex:
  chooses `Command.execute` in `src/cli.rs`.

Tool call:
  impact_radius(target_symbol="Command.execute", include_tests=true)
```

#### Workflow D: Review Changes

```text
Codex:
  I edited three files. What should I verify?

Tool call:
  review_context(base_ref="HEAD", max_tokens=3000)

Parceltongue:
  changed symbols
  impacted files
  route/public-surface warnings
  direct tests
  test gaps
  next commands/tests
```

### What Parceltongue Should Avoid

| Failure | Why It Hurts Agents | Better Design |
|---|---|---|
| Huge `tools/list` surface. | Burns context every turn and confuses routing. | Keep 6-10 public tools; hide management/debug tools. |
| Raw graph dumps. | LLM must infer workflow from low-level rows. | Return task-shaped bundles and next steps. |
| No max token controls. | One query can flood context. | Every context tool accepts `max_tokens` or `detail`. |
| Truncation without disclosure. | Agent believes answer is complete. | Add `truncated`, `hidden_counts`, and drill-down hints. |
| Silent ambiguity. | Agent follows wrong symbol. | Return candidates and require file/node disambiguation. |
| Tool descriptions as essays. | Tool metadata itself becomes expensive. | Short description plus docs/AGENTS workflow. |
| Always include tests. | Test-heavy repos bury production callers. | Hide tests by default for navigation; include for audits/review. |
| Always exclude tests. | Rename/refactor audits miss required test edits. | Make include_tests default depend on query kind. |
| Vector-only semantic confidence. | Fuzzy results look exact. | Expose search mode, vector availability, match confidence. |
| Node IDs as stable IDs. | Reindex breaks follow-up queries. | Use node_id for drill-down, qualified path/span for stability. |
| Source snippets as instructions. | Prompt injection risk. | Keep source under data fields, separate from tool-authored guidance. |
| No CLI equivalent. | Codex may overuse MCP for shell-native tasks. | Include CLI commands for heavy/diagnostic workflows. |

### Recommended Parceltongue API Contract

Short version:

```text
Every public graph tool must be:
  fresh-aware
  budget-aware
  confidence-aware
  ambiguity-aware
  next-step-aware
```

Longer contract:

```text
1. Validate request before expensive work.
2. Refresh explicitly named files unless caller opts out.
3. Resolve ambiguity with candidates, not guesses.
4. Hide noisy low-confidence edges by default.
5. Disclose hidden/truncated counts.
6. Keep response within requested budget.
7. Include at most 3 next actions.
8. Separate tool-authored guidance from source text.
9. Provide CLI equivalent when useful.
10. Make stale answers impossible to confuse with fresh answers.
```

### Concept 6 Conclusion

The best Parceltongue API is not:

```text
Here are nodes.
Here are edges.
Good luck.
```

It is:

```text
Here is the smallest trustworthy context slice for this task.
Here is why these symbols matter.
Here is what I hid.
Here is what may be stale.
Here is what to do next.
```

That is exactly the code assistance product for Codex.

The graph should act like a navigation layer between the agent and the codebase.
It should reduce wandering, reduce token waste, and turn "what should I inspect
next?" into a cheap, repeatable query.

### Repos Already Touched For This Concept

| Repo | Status | Notes |
|---|---|---|
| `Christoph__treesitter-mcp` | Source-read plus `codebase-memory-mcp` indexed | Best compact MCP workflow examples: minimal edit context, call graph, review context, relevant tests, parse diff, budget enforcement. |
| `sdsrss__code-graph-mcp` | Source-read plus `codebase-memory-mcp` indexed | Best mature agent surface: 7 visible tools, hidden management tools, concise descriptions, route-aware call graph, confidence floor, rollups, compact modes, CLI/MCP split. |
| `n24q02m__better-code-review-graph` | Source-read | Good query-pattern API, review context bundle, temporal/as-of query decorations, and code-review guidance. |
| `tirth8205__code-review-graph` | Source-read plus `codebase-memory-mcp` indexed | Good minimal context bootstrap, next-step hints, context-savings metadata, tool-list filtering, and agent instruction surfaces. |

## Concept 7: Parser Safety And Lifecycle Are Product Requirements

### Core Idea

For Parceltongue, the parser layer is not implementation plumbing.

It is the trust boundary between:

```text
Codex wants to understand a codebase
and
the repo contains arbitrary local files, generated files, half-written files,
unsupported languages, native parser bindings, very large inputs, broken syntax,
platform-specific process behavior, and stale indexes.
```

If the parser layer is weak, every higher product idea becomes fragile:

```text
smart context
blast radius
call graph
review context
minimal edit context
dependency maps
rename planning
agent next-step suggestions
```

They all depend on one boring-looking question:

```text
Can the tool parse this repo without hanging, crashing, lying, or silently
dropping important context?
```

This is why parser safety should be treated as a first-class product feature.

The user journey is not:

```text
Run parser.
Get tree.
Extract nodes.
```

The real user journey is:

```text
Agent asks for context.
Tool checks freshness.
Tool safely parses only what is needed.
Tool survives bad files.
Tool explains what was skipped.
Tool returns a confidence-aware answer.
Agent decides the next read.
```

The difference matters. A parser that is merely correct on nice files is not
enough for a solo power user running Codex across large mixed-language repos.
The parser must be operationally safe.

### Why This Matters For Codex

Codex is an agent sitting in the loop. It can recover from many ordinary code
problems if the tool tells the truth.

It cannot recover from these failures easily:

```text
The MCP server hangs.
The MCP server crashes.
The graph silently omits a large file.
The graph silently uses stale parse output.
The graph says "no callers" because parsing timed out.
The graph returns old ranges after a file changed.
The graph gives one symbol candidate when there were five.
The graph treats syntax errors as total repo failure.
The graph spends 30 seconds on one generated file.
The graph consumes the whole token budget explaining a parser failure.
```

For an agent, truthful partial information is much better than fake complete
information.

So the parser layer should not promise:

```text
I parsed the repo.
```

It should promise:

```text
I parsed these files with these grammars at this freshness level.
These files were skipped for these reasons.
These files had syntax errors but still produced trees.
These files timed out.
These files are too large.
This answer depends on these parse artifacts.
Here is the next safest thing to inspect.
```

That is a product contract.

### Evidence From The Reference Repos

| Repo | Safety Pattern | Why Parceltongue Should Care |
|---|---|---|
| `sdsrss__code-graph-mcp` | Thread-local parser cache per language, per-file parse timeout, parser reset on parse failure, max AST depth, max relation depth, file-size skip, per-node code-content cap. | This is the closest direct model for a safe Rust Tree-sitter graph service. It treats parsing as bounded work, not an infinite operation. |
| `Christoph__treesitter-mcp` | Creates a parser, configures language, parses, and explicitly warns when a tree contains syntax errors. It documents that invalid syntax can still return a tree with error nodes. | This is the correct mental model for developer tools: syntax errors are ordinary input, not exceptional repo failure. |
| `tree-sitter__py-tree-sitter` | Tests parser initialization, included range validation, parser attribute reset, buffer parsing, and callback-based parsing. Invalid included ranges raise errors. | The binding itself exposes lifecycle states and validates embedded-language parse ranges. Parceltongue should not hand-roll sloppy range handling. |
| `tree-sitter__node-tree-sitter` | Validates included ranges for overlap and supports old-tree incremental parsing plus progress callback based parsing. | Good evidence that safe embedded-language parsing and cancellation/progress hooks belong at the binding boundary. |
| `n24q02m__better-code-review-graph` | Git subprocess calls use explicit timeout and detach child stdin from MCP stdio. File update logic catches read/parse exceptions and continues. | Platform safety matters. A graph server can hang because a child inherited the wrong stdin handle, not because the graph algorithm is wrong. |
| `tirth8205__code-review-graph` | Chooses process or thread parsing workers based on platform and stdio host behavior. It switches away from process pools on Windows MCP/stdio to avoid zombies and deadlocks. | Worker topology is part of parser safety. "Parallel parsing" is not automatically safe in every host environment. |
| `DeusData__codebase-memory-mcp` | Uses supervised subprocess indexing, classifies child outcomes as clean/nonzero/crash/hang/killed/spawn-failed, supports cancellation, warns that one store handle is not concurrently safe, and includes allocator notes for Tree-sitter cross-thread frees. | This is the strongest operational model. Full-repo parsing is isolated from the long-lived server so native crashes and RSS growth do not take down the tool. |

### Source Anchors

These are the concrete local evidence anchors already inspected.

| Source | Lines Read | Relevant Detail |
|---|---:|---|
| `git-ref-repo/ignore-this-folder-repos/sdsrss__code-graph-mcp/src/parser/treesitter.rs` | 27-54 | Thread-local `PARSER_CACHE`, parser timeout, language setup, parse failure path, parser reset. |
| `git-ref-repo/ignore-this-folder-repos/sdsrss__code-graph-mcp/src/parser/treesitter.rs` | 88-100 | Recursive extraction stops after `MAX_AST_DEPTH`. |
| `git-ref-repo/ignore-this-folder-repos/sdsrss__code-graph-mcp/src/domain.rs` | 251-292 | `MAX_AST_DEPTH`, `MAX_RELATION_DEPTH`, max file size, max stored code length, parse timeout env overrides. |
| `git-ref-repo/ignore-this-folder-repos/sdsrss__code-graph-mcp/src/indexer/pipeline/index_files.rs` | 211-280 | Parallel parse path skips unsupported languages, huge files, read errors, hash errors, and parse failures without aborting the whole batch. |
| `git-ref-repo/ignore-this-folder-repos/Christoph__treesitter-mcp/src/parser/mod.rs` | 167-186 | Parse function creates parser, sets language, parses, returns error on complete parse failure, warns on syntax-error tree. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__py-tree-sitter/tests/test_parser.py` | 29-111 | Parser lifecycle tests for init, setters, deleters, included ranges, buffer inputs, and invalid ranges. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__node-tree-sitter/src/parser.cc` | 169-195 | Included ranges are validated and overlapping ranges throw an error. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__node-tree-sitter/src/parser.cc` | 209-244 | Parse accepts old tree, included ranges, and optional progress callback path. |
| `git-ref-repo/ignore-this-folder-repos/n24q02m__better-code-review-graph/src/better_code_review_graph/incremental.py` | 158-190 | Git subprocess timeout and `stdin=subprocess.DEVNULL` to prevent MCP stdio stalls. |
| `git-ref-repo/ignore-this-folder-repos/tirth8205__code-review-graph/code_review_graph/incremental.py` | 25-56 | Parse executor selection switches process/thread depending on platform and stdio behavior. |
| `git-ref-repo/ignore-this-folder-repos/DeusData__codebase-memory-mcp/src/mcp/mcp.h` | 57-59 | Cancellation request matching for active MCP request. |
| `git-ref-repo/ignore-this-folder-repos/DeusData__codebase-memory-mcp/src/mcp/mcp.h` | 119-136 | Supervised background indexing and idle store eviction. |
| `git-ref-repo/ignore-this-folder-repos/DeusData__codebase-memory-mcp/src/mcp/mcp.h` | 150-156 | Active pipeline access for cancellation. |
| `git-ref-repo/ignore-this-folder-repos/DeusData__codebase-memory-mcp/src/foundation/subprocess.c` | 69-85 | Child process outcome classification includes crash, hang, killed, spawn failure. |
| `git-ref-repo/ignore-this-folder-repos/DeusData__codebase-memory-mcp/src/foundation/subprocess.c` | 216-244 | POSIX child process uses fork/exec with open/dup2 to avoid multithreaded fork hazards. |
| `git-ref-repo/ignore-this-folder-repos/DeusData__codebase-memory-mcp/src/foundation/subprocess.c` | 247-294 | Quiet-timeout loop kills hung subprocess and records outcome. |
| `git-ref-repo/ignore-this-folder-repos/DeusData__codebase-memory-mcp/src/foundation/slab_alloc.c` | 1-37 | Tree-sitter allocator callbacks are process-global; cross-thread frees need careful ownership handling. |
| `git-ref-repo/ignore-this-folder-repos/DeusData__codebase-memory-mcp/src/foundation/slab_alloc.c` | 420-430 | Slab memory reclaim only after parser/tree owned chunks are released. |
| `git-ref-repo/ignore-this-folder-repos/DeusData__codebase-memory-mcp/src/store/store.h` | 1-9 | Store handle must not be used concurrently; use one per thread or external synchronization. |
| `git-ref-repo/ignore-this-folder-repos/DeusData__codebase-memory-mcp/src/discover/userconfig.h` | 57-63 | Process-global user language config is not thread safe and must be set before worker threads. |

### The Correct Parse Status Model

Parceltongue should not model parsing as:

```rust
fn parse_file(path: &Path) -> Result<Tree>;
```

That interface is too small. It hides the states that the agent needs to reason
about.

A better product-facing model:

```rust
enum ParseStatus {
    Parsed,
    ParsedWithSyntaxErrors,
    SkippedUnsupportedLanguage,
    SkippedTooLarge,
    SkippedBinaryOrUnreadable,
    TimedOut,
    Cancelled,
    NativeCrash,
    ParserUnavailable,
    GrammarAbiMismatch,
    InternalError,
}
```

And the parse artifact should carry enough evidence for downstream graph
queries:

```rust
struct ParseArtifact {
    repo_root: PathBuf,
    file_path: PathBuf,
    language_id: String,
    file_hash: String,
    last_modified_unix: i64,
    parser_version: String,
    grammar_name: String,
    grammar_abi: Option<u32>,
    status: ParseStatus,
    syntax_error_count: usize,
    elapsed_ms: u64,
    bytes_read: u64,
    nodes_extracted: usize,
    relations_extracted: usize,
    warnings: Vec<String>,
}
```

The important design choice:

```text
Every parse outcome should be queryable later.
```

If a file is skipped because it is too large, that fact should become part of the
graph metadata. If a file timed out, the agent should know. If a file parsed with
syntax errors, that should be visible but not fatal.

### Why `Result<Tree>` Is Too Weak

`Result<Tree>` collapses very different realities:

| Reality | Why It Matters |
|---|---|
| Unsupported extension | The agent may need fallback search or user-specified language mapping. |
| Huge generated file | The graph can still be valid if it discloses the skip. |
| Syntax error tree | Tree-sitter often still gives useful structure. The right response is degraded confidence, not failure. |
| Timeout | The file might be important but currently too expensive. Agent should ask for narrower parse or raw read. |
| Cancellation | User or agent stopped the work. Do not cache this as a real parse failure. |
| Native crash | Full-index subprocess should fail without killing MCP parent. |
| ABI mismatch | Existing captures/ranges may be stale because grammar behavior changed. |
| Parser not installed | Tool can offer setup or mark language unsupported. |

An agent-facing graph tool should never make these all look the same.

### Parser Lifecycle Boundary

Parceltongue should have an explicit parser lifecycle boundary.

Suggested components:

```text
LanguageRegistry
  Knows file extensions, shebang rules, user overrides, and grammar availability.

ParserPool
  Owns parser instances. Gives exclusive parser access per parse operation.

ParseLimiter
  Applies max bytes, max duration, max AST depth, max relation depth, and output caps.

ParseRunner
  Executes parse requests. Can choose in-process, thread-pool, process-pool, or supervised subprocess.

ParseStore
  Stores parse artifact metadata, skipped-file reasons, syntax-error status, hashes, and grammar versions.

GraphExtractor
  Consumes parse artifacts and emits nodes/edges only when parse status allows it.

AgentReporter
  Turns parse status into compact warnings and next-step hints.
```

The parser lifecycle then becomes:

```text
1. Detect language.
2. Check file metadata and size.
3. Check cache freshness by hash and grammar version.
4. Acquire parser safely.
5. Parse with timeout/cancellation.
6. Reset parser after timeout or failure.
7. Extract bounded nodes.
8. Extract bounded relations.
9. Store artifact metadata.
10. Return warnings and confidence to graph APIs.
```

That lifecycle is what lets the graph be honest.

### Parser Pools

Tree-sitter parsers are mutable. A parser is not a stateless pure function.

The evidence from `sdsrss__code-graph-mcp` is a good Rust pattern:

```text
thread-local parser cache
keyed by language
parser timeout configured once
parser reused on same thread
parser reset after parse returns None
```

For Parceltongue, the design should be:

```text
ParserPool is allowed to reuse parsers.
ParserPool must never hand the same parser to two threads at once.
ParserPool must reset a parser after timeout/failure before reuse.
ParserPool must expose parser health metrics.
ParserPool must support evicting parsers after grammar or config changes.
```

Good parser-pool states:

```text
Idle
CheckedOut
TimedOutNeedsReset
PoisonedNeedsDrop
EvictedGrammarChanged
```

The simplest safe version is probably:

```text
one parser per worker thread per language
no cross-thread parser sharing
drop parser on suspicious failure
store parse metadata separately from parser objects
```

Do not optimize this too early. Parser correctness and isolation matter more
than shaving a few milliseconds.

### Timeouts Are Not Optional

Parsing should always be bounded.

`sdsrss__code-graph-mcp` exposes a per-file parse timeout through
`CODE_GRAPH_PARSE_TIMEOUT_MS`, defaulting to 5000 ms. It also calls
`set_timeout_micros` on the parser.

That is the right instinct.

Parceltongue should have timeouts at multiple layers:

| Layer | Timeout Type | Example |
|---|---|---|
| Single parse | Per-file parse timeout | 250 ms to 5000 ms depending on mode. |
| Full index | Total job timeout or quiet timeout | Kill worker if no progress is reported. |
| Git discovery | Subprocess timeout | Avoid git status/list-files stalls. |
| MCP request | Client cancellation | Stop active pipeline when cancellation matches request. |
| Watcher debounce | Event coalescing | Avoid reindex storms while files are being written. |

Recommended default modes:

| Mode | Parse Timeout | Use Case |
|---|---:|---|
| `interactive` | 250-750 ms per file | Codex asks about one symbol or one file. |
| `index` | 1000-5000 ms per file | Full repo or batch index. |
| `deep` | User-configured higher limit | Rare manual exploration of difficult files. |

The agent response should include timeout facts:

```json
{
  "status": "partial",
  "warnings": [
    "2 files timed out during parse",
    "call graph excludes timed-out files"
  ],
  "skipped_files": [
    {
      "path": "generated/client.ts",
      "reason": "parse_timeout",
      "elapsed_ms": 750
    }
  ]
}
```

This gives Codex a useful next step:

```text
Maybe read the raw generated file only if the task actually needs it.
```

### Cancellation Should Reach The Parser Pipeline

MCP cancellation is only useful if it connects to the long-running work.

`DeusData__codebase-memory-mcp` has explicit helpers for matching cancellation
notifications to the active request and retrieving the active pipeline for
signal-handler cancellation.

Parceltongue should copy that shape.

Suggested cancellation model:

```rust
struct ParseRequest {
    request_id: RequestId,
    repo_root: PathBuf,
    files: Vec<PathBuf>,
    options: ParseOptions,
    cancellation: CancellationToken,
}
```

Every expensive loop checks cancellation:

```text
file discovery
file hashing
file read
parse
node extraction
relation extraction
embedding side pass
database write batch
```

Cancellation should not be recorded as a parse failure.

It should be recorded as:

```text
job_cancelled
not cached as file-level parse failure
safe to retry
```

This distinction matters because otherwise a cancelled full-index job can poison
the graph with fake parse errors.

### Syntax Errors Are Normal

Tree-sitter is designed to produce useful trees even when code has syntax
errors. `Christoph__treesitter-mcp` explicitly documents this in its parser code:
invalid syntax can still produce a tree with error nodes.

This should shape Parceltongue's behavior.

Bad behavior:

```text
File has syntax error.
Abort file.
Drop it from graph.
```

Better behavior:

```text
File has syntax error.
Keep extractable declarations.
Mark artifact ParsedWithSyntaxErrors.
Lower confidence for edges touching error regions.
Tell agent the answer is partial.
```

Possible extraction policy:

| Parse Condition | Node Extraction | Edge Extraction | Agent Warning |
|---|---|---|---|
| No syntax errors | Full extraction | Full extraction | None. |
| Syntax errors away from symbol | Extract symbol and nearby relations | Extract high-confidence relations only | "File has syntax errors outside selected span." |
| Syntax errors inside symbol body | Extract declaration signature | Avoid body-level edges | "Body parse is unreliable." |
| Root mostly error nodes | Store parse failure metadata | No graph edges | "File is syntactically unstable." |

This is especially important with Codex because Codex often works while the repo
is mid-edit. The user may have unsaved or half-written code. The graph must still
help.

### File Size Guards

Large files are not rare:

```text
generated clients
bundled JS
compiled outputs
vendored code
lockfiles
snapshot files
fixtures
machine-generated SQL
protobuf outputs
```

`sdsrss__code-graph-mcp` skips files above a configured max file size before
reading/parsing. It also caps stored code content per node.

That is the right shape.

Parceltongue should separate these limits:

| Limit | Why It Exists |
|---|---|
| `max_file_bytes` | Prevent one huge file from dominating indexing. |
| `max_parse_ms` | Prevent parser hangs or pathological cases. |
| `max_ast_depth` | Prevent recursive extraction blowups. |
| `max_relation_depth` | Prevent graph traversal blowups. |
| `max_node_snippet_bytes` | Prevent one node from bloating context or DB. |
| `max_edges_per_file` | Prevent generated files from flooding the graph. |
| `max_warnings_per_response` | Prevent error reporting from eating the answer. |

The important product decision:

```text
Skipping a file is allowed.
Silently skipping a file is not allowed.
```

Skipped files should be queryable:

```sql
select file_path, reason, bytes, elapsed_ms
from parse_artifacts
where status != 'Parsed';
```

And exposed through MCP:

```text
staleness_report
index_health
parse_failures
language_coverage
```

### AST Depth And Relation Depth

Tree-sitter gives you a tree. That does not mean you should recurse forever.

`sdsrss__code-graph-mcp` uses:

```text
MAX_AST_DEPTH = 64
MAX_RELATION_DEPTH = 256
```

Those exact numbers may not be right for Parceltongue, but the pattern is right.

The graph extractor needs separate boundaries:

```text
AST traversal depth
call graph traversal depth
import graph traversal depth
inheritance graph traversal depth
dataflow graph traversal depth
agent-context expansion depth
```

Why separate?

Because these are different failure modes.

| Depth | Failure Mode |
|---|---|
| AST depth | Deeply nested syntax or parser weirdness makes extraction expensive. |
| Call graph depth | Common utility functions create huge transitive closures. |
| Import graph depth | Framework barrels and reexports explode context. |
| Inheritance depth | Java/C++ hierarchies can become noisy fast. |
| Agent expansion depth | Codex gets more context than it can use. |

Each API should return both:

```text
requested_depth
actual_depth_returned
truncated_due_to_depth
hidden_counts_by_reason
```

Example:

```json
{
  "entity": "UserService.updateUser",
  "requested_depth": 3,
  "actual_depth_returned": 2,
  "truncated": true,
  "hidden": {
    "max_depth": 34,
    "max_tokens": 18,
    "low_confidence": 9
  }
}
```

That is exactly what an agent needs.

### Included Ranges And Embedded Languages

Embedded languages are where many simple Tree-sitter tools become inaccurate.

Examples:

```text
JavaScript inside HTML
CSS inside HTML
SQL inside strings
GraphQL inside template literals
Markdown code fences
MDX
Vue single-file components
Svelte
Rust macros containing DSL-like syntax
```

The Tree-sitter bindings expose included ranges for this reason.

Evidence:

```text
py-tree-sitter tests reject invalid range ordering and invalid byte spans.
node-tree-sitter rejects overlapping ranges.
node-tree-sitter parse path accepts included ranges.
```

Parceltongue should not treat embedded-language parsing as an afterthought.

Design:

```text
1. Outer parser identifies embedded language regions.
2. Region extractor emits validated byte ranges.
3. Inner parser parses only those included ranges.
4. Child parse artifact stores parent_file, parent_node_id, range, and language.
5. Edges crossing outer/inner language boundaries are marked as derived or inferred.
```

Important safety rule:

```text
Invalid included ranges should fail the embedded parse only.
They should not crash or invalidate the parent file parse.
```

Example artifact:

```json
{
  "file_path": "src/page.html",
  "language_id": "javascript",
  "parent_language_id": "html",
  "included_ranges": [
    {
      "start_byte": 124,
      "end_byte": 490,
      "source_node_kind": "script_element"
    }
  ],
  "status": "Parsed"
}
```

For agent workflows, this matters because otherwise Codex may ask:

```text
Where is submitForm called?
```

And the graph may miss:

```html
<button onclick="submitForm()">
```

or:

```html
<script>
  submitForm();
</script>
```

The agent does not need perfect embedded-language support on day one. But the
parse model should be able to represent it.

### Incremental Old-Tree Parsing

Tree-sitter supports incremental parsing with an old tree.

`node-tree-sitter` exposes parse with:

```text
old_tree
included_ranges
progress callback
```

Concept 5 already argued that incremental indexing is a freshness contract. This
concept adds the lifecycle point:

```text
The old tree must be treated as a versioned parse artifact.
```

Do not reuse an old tree blindly.

Safe reuse requires:

```text
same file path
same language
same grammar version
same parser ABI compatibility
known old file hash
known edit ranges or changed ranges
parser not poisoned
old tree not already freed
```

If these are not true, fall back to full parse for that file.

The goal is not cleverness. The goal is no lying.

### Grammar ABI Drift

Tree-sitter grammars and bindings can change.

That means these can change:

```text
node kinds
field names
capture behavior
query behavior
error recovery behavior
byte ranges around tricky syntax
```

If Parceltongue stores graph rows without remembering parser and grammar
versions, it can accidentally mix old graph rows with new parser behavior.

Recommended index metadata:

```sql
create table parser_versions (
  language_id text primary key,
  grammar_name text not null,
  grammar_version text,
  grammar_abi integer,
  binding_name text not null,
  binding_version text,
  query_pack_version text not null,
  indexed_at_unix integer not null
);
```

On startup:

```text
1. Load current grammar metadata.
2. Compare against stored parser_versions.
3. If ABI or query pack changed, mark affected files stale.
4. If only tool patch version changed, decide based on migration policy.
5. Tell agent when answer depends on stale grammar output.
```

MCP warning:

```json
{
  "status": "stale",
  "warning": "Rust grammar version changed since last index; call graph may omit new node kinds.",
  "recommended_action": "reindex_language",
  "language_id": "rust"
}
```

This is not premature enterprise ceremony. It is how you avoid subtle wrongness
after updating a parser crate.

### Native Crashes Need Process Isolation

Tree-sitter itself is native code. Many bindings call into native libraries.
Some grammars are generated C. Some parser integrations use custom allocators.
Some full-repo indexes use many workers.

The `DeusData__codebase-memory-mcp` repo is the best reference here because it
does not pretend native code cannot fail. It has:

```text
supervised worker subprocess for full index
child outcome classification
quiet timeout
crash/hang/killed/spawn-failed distinctions
idle store eviction
active pipeline cancellation
Tree-sitter allocator notes
per-thread store guidance
```

Parceltongue should distinguish interactive parsing from full indexing:

| Operation | Suggested Isolation |
|---|---|
| One small file for immediate context | In-process parser pool is acceptable. |
| Batch changed-files refresh | Worker thread pool or process pool with timeouts. |
| Full repo index | Supervised subprocess is safer. |
| Unknown massive monorepo | Supervised subprocess plus progress heartbeat. |
| Experimental grammar | Process isolation by default. |

The product behavior should be:

```text
If a full-index worker crashes, MCP parent survives.
If a full-index worker hangs, parent kills it.
If worker dies, graph remains at last known good index plus failure metadata.
Agent sees index health warning.
```

Bad behavior:

```text
Full repo index crashes MCP server.
Codex loses tool session.
User has no idea which file caused it.
```

Good behavior:

```text
Full repo index worker crashed while parsing language=cpp file=foo/bar.cc.
Existing graph remains usable.
Retry can exclude file or switch to safe mode.
```

### Subprocess Safety Details

This is a place where boring systems details matter.

The references show three concrete issues:

| Issue | Reference Pattern | Parceltongue Lesson |
|---|---|---|
| Child process hangs because it inherits MCP stdin | `n24q02m__better-code-review-graph` detaches git subprocess stdin. | Every subprocess launched by MCP tools should intentionally set stdin. |
| Process pool leaks/zombies under Windows stdio host | `tirth8205__code-review-graph` switches to thread executor on Windows non-TTY stdio. | Worker strategy depends on host platform, not just CPU count. |
| Forking from multithreaded parent can deadlock if child touches malloc before exec | `DeusData__codebase-memory-mcp` uses open/dup2/exec rather than higher-level unsafe paths in child. | Native implementation must respect fork safety when server has threads. |

This is why parser safety is not only about Tree-sitter APIs.

It is also about:

```text
stdio ownership
process groups
timeouts
heartbeats
log tailing
worker cleanup
platform-specific execution policy
database handle ownership
```

For Parceltongue, a supervised full-index runner should expose:

```text
start_time
last_progress_time
files_seen
files_parsed
files_skipped
current_file
current_language
rss_peak
outcome
exit_code
signal
quiet_timeout_ms
```

The agent does not need all of that every turn. But the tool needs it available
for diagnosis.

### Store Handles And Parser Threads

A parser graph tool is also a database writer.

That means parser safety and store safety meet.

`DeusData__codebase-memory-mcp` states directly that a single store handle must
not be used concurrently. Use one store per thread or external synchronization.

Parceltongue should have a simple rule:

```text
Parse workers do CPU work.
One writer owns DB mutation.
Workers send extracted artifacts to writer through a bounded channel.
```

Why bounded?

Because without backpressure, parsing can outrun DB writes and inflate memory.

Recommended pipeline:

```text
discover files
  -> bounded parse queue
  -> parse workers
  -> bounded artifact queue
  -> single writer or sharded writers with explicit DB handles
  -> commit batch
  -> index health update
```

If the DB layer supports concurrent writers safely, fine. But that should be an
explicit design, not accidental shared-handle usage.

### Global Configuration Must Freeze Before Workers

Some parser/discovery settings are effectively global:

```text
language extension overrides
ignore rules
grammar registry
allocator callbacks
query pack registry
max file size
parse timeout
```

`DeusData__codebase-memory-mcp` has a user language config hook marked not
thread-safe and intended to be called before spawning worker threads.

Parceltongue should follow a clear startup/config lifecycle:

```text
1. Load config.
2. Validate config.
3. Build immutable runtime config.
4. Spawn workers.
5. Treat config changes as a new generation.
6. Evict parser pools and mark affected index rows stale.
```

Avoid:

```text
Worker A sees old language config.
Worker B sees new language config.
Both write to same graph generation.
Agent sees mixed output.
```

Config generation should be stored with parse artifacts.

### Allocator And Native Memory Caution

Most Parceltongue users do not need to care about allocator internals.

But Parceltongue as a tool should care.

The C reference repo documents a real class of problems:

```text
Tree-sitter allocator callbacks are process-global.
Cross-thread frees can happen.
Reclaiming memory too early can cause invalid frees or use-after-free.
Long-running full indexes can ratchet memory.
```

This does not mean Parceltongue should immediately write a custom allocator.

It does mean:

```text
Do not assume parser memory behavior is harmless.
Measure RSS during full index.
Prefer subprocess isolation for full-index jobs.
Drop parser/tree objects deterministically.
Avoid global allocator customization unless there is a measured need.
If customizing allocator, treat it as a major unsafe subsystem with tests.
```

For the solo Codex-app user, the winning product behavior is:

```text
The graph tool can index a huge repo and then give memory back.
```

Subprocess full-index isolation gives that without inventing allocator machinery
on day one.

### Agent-Facing Error Reporting

Parser safety is only useful if it reaches the agent in a compact way.

Bad MCP response:

```json
{
  "error": "parse failed"
}
```

Better MCP response:

```json
{
  "status": "partial",
  "answer": {
    "symbols": [
      {
        "name": "update_user_profile",
        "file": "src/users/service.rs",
        "line": 42,
        "confidence": 0.93
      }
    ]
  },
  "index_health": {
    "fresh_files": 381,
    "stale_files": 2,
    "skipped_files": 5,
    "timed_out_files": 1,
    "syntax_error_files": 3
  },
  "warnings": [
    "1 Rust file timed out during latest refresh",
    "3 TypeScript files parsed with syntax errors"
  ],
  "next": [
    {
      "tool": "parse_failures",
      "why": "inspect skipped files if this task touches generated clients"
    },
    {
      "tool": "read_next",
      "why": "read direct caller context for update_user_profile"
    }
  ]
}
```

The response is still small. But it prevents the agent from over-trusting the
answer.

### Health APIs Parceltongue Should Expose

These are not glamorous, but they make the tool usable.

| API | Purpose |
|---|---|
| `index_health` | Summarize freshness, parse failures, skipped files, grammar versions, and last index time. |
| `parse_failures` | List files by parse status and reason. |
| `language_coverage` | Show languages detected, supported, skipped, and mapped by override. |
| `parser_versions` | Show grammar/binding/query-pack versions used to build the graph. |
| `reparse_file` | Force safe refresh of one file. |
| `reindex_language` | Rebuild artifacts for one language after grammar/query update. |
| `exclude_path` | Let user mark generated or toxic paths as ignored. |
| `safe_mode_index` | Full index with stricter timeouts, lower worker count, and subprocess isolation. |

These APIs are not the main user journey. They are the safety rails that make the
main journey trustworthy.

### Minimal Parser Safety Contract

If Parceltongue had to start with a minimal contract, it should be this:

```text
1. No file parse can hang forever.
2. No full-repo index can kill the long-lived MCP parent.
3. No skipped file is silent.
4. No syntax-error tree is automatically discarded.
5. No parser is shared concurrently without synchronization.
6. No old tree is reused across grammar/config generations.
7. No query pretends stale parse output is fresh.
8. No subprocess inherits MCP stdio accidentally.
9. No database handle is mutated concurrently unless designed for it.
10. No huge generated file can dominate the agent's token budget.
```

This is the safety bar before Parceltongue can be a serious Codex companion.

### Recommended Rust Shapes

The core parser module could expose a small, boring API:

```rust
pub struct ParseOptions {
    pub max_file_bytes: u64,
    pub timeout_ms: u64,
    pub max_ast_depth: usize,
    pub max_relation_depth: usize,
    pub max_snippet_bytes: usize,
    pub mode: ParseMode,
}

pub enum ParseMode {
    Interactive,
    BatchIndex,
    DeepManual,
}

pub struct ParseRequest {
    pub repo_root: PathBuf,
    pub file_path: PathBuf,
    pub language_hint: Option<String>,
    pub old_artifact_id: Option<String>,
    pub cancellation_token: CancellationToken,
    pub options: ParseOptions,
}

pub struct ParseResponse {
    pub artifact: ParseArtifact,
    pub tree: Option<tree_sitter::Tree>,
    pub changed_ranges: Vec<ByteRange>,
    pub warnings: Vec<ParseWarning>,
}
```

Graph extraction should not call Tree-sitter directly from many random modules.

Instead:

```text
GraphExtractor depends on ParseResponse.
GraphExtractor does not own parser lifecycle.
GraphExtractor cannot ignore parse warnings.
GraphExtractor records which parse artifact produced each node/edge batch.
```

That boundary keeps the system understandable.

### Product Modes

A solo Codex-app power user needs three modes.

| Mode | User Need | Parser Policy |
|---|---|---|
| `fast` | "I need enough context for this edit now." | Refresh named files, shallow dependency context, strict timeout, disclose partials. |
| `safe` | "I am about to do a risky refactor." | Refresh dependency neighborhood, include tests, moderate timeout, stricter freshness. |
| `full` | "Index this repo for future sessions." | Supervised subprocess, progress heartbeat, skip toxic files, store health report. |

The same parser infrastructure supports all three.

The difference is budget and isolation.

### What Parceltongue Should Copy Directly

| Source | Copy This |
|---|---|
| `sdsrss__code-graph-mcp` | Thread-local parser cache, parse timeout, parser reset after failure, max AST depth, max relation depth, max file size, per-node content cap, batch parse skip accounting. |
| `Christoph__treesitter-mcp` | Treat syntax-error trees as useful but warn about them. |
| `tree-sitter__py-tree-sitter` | Treat parser lifecycle and included ranges as validated states. |
| `tree-sitter__node-tree-sitter` | Use old-tree incremental parse and progress callback concepts where the Rust binding supports equivalent behavior. |
| `n24q02m__better-code-review-graph` | Detach subprocess stdin and use hard subprocess timeouts. |
| `tirth8205__code-review-graph` | Make worker strategy platform-aware. |
| `DeusData__codebase-memory-mcp` | Supervised full-index subprocess, outcome classification, cancellation hook, idle store eviction, one-store-per-thread discipline, native memory caution. |

### What Parceltongue Should Avoid

| Anti-Pattern | Why It Is Bad |
|---|---|
| Direct parser calls spread across modules. | Lifecycle and timeout policy become inconsistent. |
| Treating syntax error as fatal. | Codex often works on half-edited code. |
| No skipped-file table. | The graph looks complete when it is not. |
| No parser version metadata. | Grammar/query drift creates subtle stale graph rows. |
| Reusing old trees without generation checks. | Incremental parsing can produce misleading freshness. |
| Sharing parser instances across threads. | Parser mutation and native state become unsafe or flaky. |
| Full indexing inside long-lived MCP process only. | Native crash or memory growth can take down the tool. |
| Unbounded recursive extraction. | One pathological tree can dominate indexing. |
| Unbounded graph expansion. | One common utility can flood context. |
| Subprocesses inheriting MCP stdin. | Platform-specific hangs become user-visible latency. |
| Concurrent DB writes through one handle. | Data corruption, lock storms, or flaky failures. |
| Silent fallback to regex-only extraction. | Agent over-trusts weak context. |

### Parser Safety And PMF

From a Shreyas Doshi product perspective, parser safety is not a feature users
ask for directly.

It is a feature they feel as:

```text
"The tool does not randomly die."
"The graph feels trustworthy."
"Codex knows when it is guessing."
"I can point it at a large repo without babysitting it."
"It helps me even when my code is mid-edit."
"When something is skipped, I can see why."
```

That is exactly the PMF axis for Parceltongue as a personal Codex companion.

Not:

```text
Can it parse every language perfectly?
```

But:

```text
Can it keep Codex oriented in a large codebase without wasting tokens or lying?
```

Parser safety is one of the hidden foundations of that promise.

### Design Heuristic

The heuristic I would use:

```text
Every parser failure should become structured context, not a broken session.
```

That sentence is the whole concept.

If a file is too big, that is context.
If a parse timed out, that is context.
If a grammar is missing, that is context.
If syntax errors exist, that is context.
If an old index is stale, that is context.
If a worker crashed, that is context.

The graph should turn all of that into a small, honest answer.

### Concept 7 Conclusion

Parceltongue should evolve from:

```text
Tree-sitter parser wrapper
```

into:

```text
safe parser lifecycle service for agents
```

That means:

```text
bounded parsing
explicit parse statuses
parser pools with ownership rules
syntax-error tolerant extraction
included-range validation
grammar/version freshness
cancellation-aware pipelines
supervised full-index subprocesses
queryable skipped-file reasons
compact agent-facing health warnings
```

This is not overengineering. It is the difference between:

```text
cool graph demo
```

and:

```text
tool I trust Codex to use while changing a large production codebase
```

### Repos Already Touched For This Concept

| Repo | Status | Notes |
|---|---|---|
| `sdsrss__code-graph-mcp` | Source-read plus `codebase-memory-mcp` indexed | Best direct Rust parser lifecycle reference: parser cache, timeout, reset, file-size caps, AST/relation depth caps, batch skip accounting. |
| `Christoph__treesitter-mcp` | Source-read plus `codebase-memory-mcp` indexed | Clean syntax-error model: invalid code may still produce a tree, warn rather than abort. |
| `tree-sitter__py-tree-sitter` | Source-read | Binding-level parser lifecycle and included-range validation evidence. |
| `tree-sitter__node-tree-sitter` | Source-read | Old-tree parsing, included ranges, and progress callback evidence. |
| `n24q02m__better-code-review-graph` | Source-read | Subprocess timeout and MCP stdio safety evidence. |
| `tirth8205__code-review-graph` | Source-read plus `codebase-memory-mcp` indexed | Platform-aware parse worker strategy evidence. |
| `DeusData__codebase-memory-mcp` | Source-read | Strongest operational safety reference: supervised subprocess, cancellation, store/thread safety, native memory caution. |

## Concept 8: Adjacent Code-Intelligence Systems Are Layers, Not Competitors

### Core Idea

The placeholder said:

```text
Similar non-Tree-sitter parsers
```

That wording is too sloppy.

The better category is:

```text
adjacent code-intelligence systems
```

Because the tools in this bucket are not all non-Tree-sitter parsers:

```text
ast-grep is explicitly Tree-sitter based.
stack-graphs has a Tree-sitter integration.
Semgrep has its own language machinery and Tree-sitter-related repositories.
SCIP is a protocol, not a parser.
Glean is a fact database, not a parser.
Kythe is a graph schema plus indexer ecosystem, not a parser.
CodeQL is a query/database/security-analysis ecosystem, not a parser.
Universal Ctags is a symbol/tag indexer.
Comby is structural search and rewrite, not a dependency graph.
```

The important question for Parceltongue is not:

```text
Which one of these wins?
```

The important question is:

```text
Which layer of the code-assistance journey does each one solve?
```

For a Codex-app power user, these tools should be understood as a stack:

```text
Text search layer
  ripgrep, livegrep

Symbol inventory layer
  universal-ctags

Structural pattern layer
  ast-grep, Semgrep, Comby

Name/scope resolution layer
  stack-graphs

Precise symbol index layer
  SCIP, LSIF, language-server outputs

Fact graph layer
  Glean, Kythe, CodeQL

Agent context layer
  Parceltongue
```

Parceltongue should not try to become all of them.

Parceltongue should become the agent-facing broker that knows when to use each
kind of evidence.

### The User Journey Frame

Codex does not wake up wanting a parser.

Codex wants to answer questions like:

```text
What should I read next?
What calls this?
What will break if I change this?
Where is this symbol defined?
Is this reference exact or fuzzy?
Which files should be edited together?
What tests are likely relevant?
Can I rewrite this pattern safely?
Is this security-sensitive?
Why is this answer incomplete?
```

Each adjacent system helps with a different part of that journey.

| User Journey Need | Best-Fit Layer | Representative Tools |
|---|---|---|
| Find text quickly | Text search | `ripgrep`, `livegrep` |
| Find declarations quickly | Symbol inventory | `universal-ctags` |
| Find AST-shaped code patterns | Structural pattern search | `ast-grep`, `Semgrep` |
| Rewrite matching code safely enough | Structural rewrite | `ast-grep`, `Comby` |
| Enforce rules or scan for risky patterns | Static analysis rules | `Semgrep`, `CodeQL` |
| Resolve definitions and references precisely | Symbol index | `SCIP`, `LSIF`, LSP indexers |
| Resolve names without full compiler integration | Scope/name graph | `stack-graphs` |
| Store scalable code facts | Fact database | `Glean`, `Kythe`, `CodeQL` |
| Serve agent-sized context | Agent broker | `Parceltongue` |

The PMF insight:

```text
Parceltongue should be the layer that turns many evidence sources into the
smallest reliable next context for Codex.
```

### Evidence From The Reference Repos

| System | Local Repo | Source Evidence | What It Is |
|---|---|---|---|
| `ast-grep` | `ast-grep__ast-grep` | README lines 17-24, 72-90, 101-119 | Tree-sitter-based structural search, lint, and rewrite tool with code-like patterns and YAML rules. |
| `Semgrep` | `semgrep__semgrep` | README lines 45-57, 62-72, 142-153; MCP server lines 510-541, 787-849, 851-909, 991-1173, 1193-1406 | Static-analysis and rule engine with MCP integration, scan tools, rule schema, custom-rule prompt, AST dump, and findings workflows. |
| `SCIP` | `scip-code__scip` | README lines 1-16 and 33-52; `scip.proto` lines 20-119, 148-190, 251-266, 465-540, 672-750 | Language-agnostic code intelligence protocol for definitions, references, implementations, symbols, occurrences, roles, relationships, and external symbols. |
| `stack-graphs` | `github__stack-graphs` | README lines 1-8; `tree-sitter-stack-graphs` README lines 22-74; languages README lines 7-22 | Name-resolution graph system, efficient and incremental, with CLI indexing/querying and independently versioned language definitions. |
| `Glean` | `facebookincubator__Glean` | README lines 8-21 and 32-40; `glean.thrift` lines 30-40, 73-76, 105-137, 147-211; Glass architecture lines 1-16 and 80-147; LSP README lines 1-23 | Scalable fact store for source-code facts, relationships, xrefs, call/type hierarchies, language-agnostic Glass APIs, and static LSP. |
| `CodeQL` | `github__codeql` | README lines 1-19; CodeQL query/library files show dataflow, taint, query, and database abstractions. | Query libraries and security-analysis ecosystem over CodeQL databases. Heavy, precise, rule/query driven. |
| `Kythe` | `kythe__kythe` | README lines 24-31; `storage.proto` lines 24-119 and 126-203; `graph.proto` lines 26-44 and 65-177; `xref.proto` lines 31-60 and 106-173 | Language-agnostic code graph schema, VName identity, graph store, fast graph service, xref service, decorations, references, callers, definitions. |
| `universal-ctags` | `universal-ctags__ctags` | `ctags.1.rst` lines 7-44 and 115-135; JSON output lines 17-60; multi-parser docs lines 8-24 and 46-88 | Fast tag generator for source-code language objects, editor navigation, JSON output, guest parsers, subparsers, and multi-language files. |
| `Comby` | `comby-tools__comby` | README lines 1-11 and 59-72 | Structural search and rewrite over nested code shapes; useful as a refactoring executor after graph-based target selection. |

### Why These Are Layers

Take one actual Codex journey:

```text
User: "Change the auth middleware behavior without breaking callers."
```

A naive agent may do:

```text
rg "auth"
read random files
edit guessed file
run tests
fix errors
```

A layered Parceltongue journey should do:

```text
1. Text search finds candidate terms.
2. Symbol inventory finds declarations named AuthMiddleware/AuthGuard/auth.
3. Structural search finds middleware registration patterns.
4. Symbol index resolves definitions and references.
5. Name/scope graph filters false matches.
6. Fact graph finds callers, overrides, routes, tests, generated adapters.
7. Agent context layer returns the smallest trustworthy bundle.
8. Structural rewrite layer applies mechanical edits if the pattern is clear.
9. Rule/security layer checks for policy regressions.
```

This is why the tools are not competitors.

They are evidence providers.

### Layer 1: Text Search Is Still Necessary

Text search is not in the placeholder, but it belongs in the model.

Why:

```text
No graph is complete.
No parser covers every file.
Generated files exist.
Comments and docs matter.
Strings matter.
Config files matter.
Agents need a fallback when precise context is stale.
```

Parceltongue should assume text search is always available.

But it should not stop there.

Text search answers:

```text
Where does this string occur?
```

It does not answer:

```text
Which occurrence is a definition?
Which occurrence is a reference?
Which occurrence is a call?
Which occurrence is a route registration?
Which occurrence matters for this edit?
```

Product role:

```text
Fallback and recall layer.
```

Agent behavior:

```text
Use text search when graph confidence is low, language is unsupported, or the
query is string/config/doc oriented.
```

### Layer 2: Universal Ctags Is The Symbol Inventory Baseline

Universal Ctags is the older world, but it is still instructive.

It generates tag files for language objects. Editors use those tags to jump to
definitions. It supports many languages, many tag kinds, list options designed
for client tools, JSON output, and multi-parser concepts.

What it gives Parceltongue:

```text
fast broad symbol inventory
language-object kinds
JSON-lines output model
client-tool friendliness
fallback when Tree-sitter support is weak
multi-parser ideas for embedded languages
```

What it does not give:

```text
precise dependency graph
call graph
name/scope resolution
rich provenance
token-budgeted agent context
freshness-aware graph APIs
```

The useful borrow:

```text
Parceltongue should be able to degrade to "tag mode."
```

Example:

```text
Rust/TypeScript/Python have strong Tree-sitter extractors.
Some niche DSL does not.
Use ctags-like inventory for declarations.
Mark result as tag-derived and low/medium confidence.
Let Codex decide whether to read raw file.
```

Agent-facing status:

```json
{
  "source": "ctags_fallback",
  "confidence": "medium",
  "warning": "No Tree-sitter extractor for this language; symbol inventory only."
}
```

This is much better than pretending unsupported files do not exist.

### Layer 3: ast-grep Is Structural Search And Rewrite

ast-grep is close to Parceltongue because it is Tree-sitter-based, but its
product shape is different.

ast-grep asks:

```text
Can users write code-shaped patterns to find, lint, and rewrite AST-shaped code?
```

Parceltongue asks:

```text
Can an agent understand the dependency neighborhood around an edit?
```

Those are different.

What ast-grep is excellent at:

```text
code-like pattern syntax
metavariables
structural matching
lint rules
rewrites
multi-core scanning
YAML rule authoring
mass code manipulation
```

What it is not:

```text
persistent code graph
call graph service
symbol-resolution database
agent context broker
cross-session memory for code relationships
```

Best Parceltongue borrow:

```text
Expose structural-search tools as a companion to graph queries.
```

Example agent journey:

```text
1. Parceltongue graph says these 17 call sites are in the blast radius.
2. Codex asks structural search: which call sites use old optional argument shape?
3. ast-grep-like matcher returns exact code-shaped matches.
4. Codex applies a rewrite only to those matches.
5. Parceltongue refreshes the graph for touched files.
```

This suggests Parceltongue should have:

```text
structural_search(pattern, language, scope_files)
structural_rewrite_preview(pattern, rewrite, scope_files)
structural_rewrite_apply(pattern, rewrite, scope_files, require_review)
```

But the graph should choose the scope.

That is the key integration.

### Layer 4: Comby Is A Rewrite Executor, Not A Graph

Comby is useful because it solves a frustrating middle ground:

```text
Regex is too brittle.
Full AST coding is too heavy.
Need a structural rewrite that understands delimiters, nesting, comments, and strings.
```

The README example shows the kind of problem:

```text
match if (condition) safely enough without regex confusion around nested syntax
```

Parceltongue should treat Comby-style tools as:

```text
rewrite executors
```

Not as:

```text
code understanding engines
```

Good workflow:

```text
Parceltongue graph identifies files and symbols.
Comby-style rewrite handles repetitive syntax transformation.
Parceltongue reindexes changed files.
Codex reviews diff and tests.
```

Bad workflow:

```text
Run global Comby rewrite across repo before knowing dependency impact.
```

The Shreyas Doshi view:

```text
Comby has strong activation for mechanical refactors, but weak retention as a
navigation brain. It is a sharp action tool, not the product core.
```

### Layer 5: Semgrep Is Rule Analysis And Agent Guardrails

Semgrep is very relevant to agent workflows, but not because it answers:

```text
What calls whom?
```

Semgrep answers:

```text
Does this code match a risky or policy-relevant pattern?
Can I express a rule that catches this shape?
Can scans run in IDE, pre-commit, CI, or MCP?
Can an agent write or run rules?
```

The open README says Semgrep:

```text
searches code
finds bugs
enforces secure guardrails and coding standards
supports 30+ languages
runs in IDE, pre-commit, and CI
has an MCP server for AI coding assistants
```

The MCP server is especially relevant.

It exposes:

```text
semgrep_rule_schema
get_supported_languages
semgrep_findings
semgrep_scan_with_custom_rule
get_abstract_syntax_tree
semgrep_scan
semgrep_scan_remote
semgrep_scan_supply_chain
semgrep_whoami
prompts for setup and custom rule writing
resources for rule schema and rule YAML
tool disabling via env vars
local vs hosted scan split
path validation
temporary file handling
finding elicitation
```

Parceltongue should copy the agent integration ideas:

```text
deterministic tools exposed through MCP
tool descriptions that tell the agent when to use each tool
schema resources
prompts for writing project-specific rules
hooks for generated code
scan result models with errors and skipped items
local/remote execution split
disable switches for risky/noisy tools
```

But Parceltongue should not become Semgrep.

Different product center:

| Product | Core Loop |
|---|---|
| Semgrep | Write/run rules -> get findings -> fix or triage. |
| Parceltongue | Ask graph question -> get minimal context -> make code edit safely. |

The useful combined journey:

```text
1. Parceltongue finds dependency neighborhood for auth middleware.
2. Codex edits the behavior.
3. Semgrep scans touched files for security regression patterns.
4. Parceltongue refreshes graph and suggests tests.
```

That is a strong bi-directional workflow:

```text
graph narrows scan scope
scan findings influence graph exploration
```

### Layer 6: SCIP Is The Symbol Identity Protocol Parceltongue Should Respect

SCIP is one of the most relevant references in the entire repo set.

It is a language-agnostic protocol for indexing source code to power:

```text
go to definition
find references
find implementations
```

Its schema forces important concepts:

```text
Index metadata
ToolInfo
project_root
Document
relative_path
language
position encoding
Occurrence
SymbolInformation
SymbolRole
Relationship
external_symbols
streaming consumption
```

This is exactly where many simple Tree-sitter graph tools are weak.

They create ad hoc IDs like:

```text
rust:fn:foo:src/lib.rs:12-30
```

Then line numbers shift and identity falls apart.

SCIP instead pushes toward:

```text
stable symbol string
package identity
descriptor path
document-relative occurrence
roles such as definition/reference/import/read/write/test/generated
relationships such as implementation/reference/type definition
```

Parceltongue should borrow heavily here.

Minimum borrow:

```text
Use SCIP-like symbol identities internally or provide SCIP import/export.
```

Possible design:

```text
parceltongue_symbol
  scheme
  package_manager
  package_name
  package_version
  descriptors
  local_id
  display_name
  enclosing_symbol
```

Graph rows:

```text
occurrence
  document_id
  range
  symbol_id
  role_bits
  syntax_kind
  diagnostics
  enclosing_range
```

Relationships:

```text
symbol_relationship
  from_symbol_id
  to_symbol_id
  relationship_kind
  is_reference
  is_implementation
  is_type_definition
  confidence
  provenance
```

Agent journey:

```text
Codex asks "where is this method implemented?"
Parceltongue first checks native graph.
If SCIP index exists, import precise symbol facts.
If not, fallback to Tree-sitter and tag-derived graph.
Return confidence and provenance.
```

This gives Parceltongue a path for all languages:

```text
Tree-sitter for local structure.
SCIP when precise indexers exist.
Fallback search/tags when neither exists.
```

### Layer 7: stack-graphs Is The Name-Resolution Warning

Tree-sitter captures syntax.

But syntax is not name resolution.

Example:

```text
foo.bar()
```

Syntax can tell us:

```text
call expression
receiver identifier
member identifier
argument list
```

It cannot always tell us:

```text
which package owns foo
which type foo has
which method bar resolves to
which import brought foo into scope
which local shadows which global
which overload is intended
```

stack-graphs exists because name resolution deserves its own formalism.

The repo describes stack graphs as efficient, incremental name-resolution rules
for arbitrary languages without needing existing build or analysis tools. The
Tree-sitter integration can index source code and query definitions from the
command line. The language definitions have their own versioning and compatibility
rules.

Parceltongue should copy the warning:

```text
Do not pretend a Tree-sitter capture is a resolved symbol.
```

It should store two different things:

```text
syntactic reference
resolved reference
```

Schema:

```text
reference_occurrence
  text
  file
  span
  syntax_node_kind
  candidate_symbol_ids
  resolved_symbol_id nullable
  resolution_method
  confidence
```

Resolution methods:

```text
tree_sitter_heuristic
import_table
stack_graph
language_server
scip
compiler
manual_hint
unresolved
```

This is especially important for:

```text
TypeScript path aliases
Python dynamic imports
Rust modules and trait methods
C/C++ headers and macros
Java overloaded methods
Ruby metaprogramming
framework magic
```

The product point:

```text
Parceltongue does not need perfect name resolution everywhere.
It needs to tell Codex when resolution is exact, inferred, or unresolved.
```

### Layer 8: Glean Is The Fact Database North Star

Glean is not a lightweight local tool.

But it is a strong architectural reference.

Its README frames the system around facts about source code:

```text
symbol locations
types
relationships
cross-references
function/method calls
call hierarchies
type hierarchies
```

It stores detailed information at scale and uses the Angle query language. It
also supports SCIP/LSIF formats for several languages.

The Glass layer then exposes language-agnostic navigation:

```text
symbol search
definitions
references
document symbols
related symbols
related neighborhoods
call hierarchy
```

This maps almost directly to Parceltongue's desired agent surface.

What Parceltongue should borrow:

```text
facts as durable rows
derived facts
fact ownership
schema versions
repo/hash identity
language-agnostic query layer
codemarkup-like unifying schema
best/closest DB selection idea
static LSP as proof that precomputed facts help huge repos
```

What Parceltongue should not copy initially:

```text
distributed scale
Angle query language
complex server deployment
large open-source build complexity
enterprise DB operations
```

Parceltongue should be:

```text
Glean-shaped enough to be correct.
SQLite/local enough to be usable by one person.
MCP-shaped enough to help Codex.
```

That is the sweet spot.

### Layer 9: Kythe Is The Graph Schema Precedent

Kythe is another major "do not invent blindly" reference.

It gives:

```text
VName identity
source/edge/target/fact entries
graph store
graph service
xref service
decorations
cross references
documentation
fast single-step lookups
batch requests
filterable facts
pagination
dirty-buffer/workspace patching concepts
```

The most important idea for Parceltongue:

```text
Identity should not be a timestamp.
```

Kythe's VName intentionally excludes revision/timestamp from the name itself.
Time belongs in graph facts, not in identity.

That matters because Parceltongue's old line-number-based identity problems are
exactly the opposite:

```text
file path + line range becomes identity
line shifts create new identity
incremental update cascades
graph loses continuity
```

Kythe's lesson:

```text
Name says what it is.
Facts say where/when/how it exists.
```

Parceltongue should model:

```text
stable_entity_id
  semantic identity

entity_fact
  current location
  revision
  parser version
  file hash
  visibility
  language
```

The xref service is also relevant.

It says requests should be:

```text
quick
batchable
explicit about which facts to return
```

That is exactly the agent-token equivalent:

```text
quick
batchable
explicit about which context to return
```

### Layer 10: CodeQL Is Query Power, Not Daily Navigation

CodeQL is a gold-standard query ecosystem for security and program analysis.

The open repository contains standard CodeQL libraries and queries that power
GitHub Advanced Security. It is built around a CodeQL database, query language,
libraries, diagnostics, dataflow, taint tracking, and security rules.

For Parceltongue, CodeQL is both inspiring and dangerous.

Inspiring:

```text
query packs
language libraries
dataflow abstractions
diagnostics
security-grade findings
metadata around queries
standardized result shapes
```

Dangerous:

```text
too heavy for instant Codex context
too security-analysis oriented for everyday navigation
requires database creation
more complex than a solo local companion needs
not primarily about "what should the agent read next?"
```

Best borrow:

```text
Make higher-order graph questions queryable.
```

Not:

```text
Build a CodeQL clone.
```

Good Parceltongue query examples:

```text
functions_that_call(symbol)
routes_that_reach(symbol)
tests_that_cover(symbol)
public_api_that_depends_on(symbol)
symbols_with_unresolved_references()
files_with_parse_warnings_in_blast_radius(symbol)
```

Those are "CodeQL-ish" in spirit, but agent-shaped in product.

### A Practical Layered Architecture For Parceltongue

Parceltongue should be designed as a broker over evidence layers.

```text
Codex request
  -> Intent router
  -> Evidence planner
  -> Evidence adapters
  -> Confidence merger
  -> Token budget selector
  -> Agent response
```

Evidence adapters:

| Adapter | Reads From | Produces |
|---|---|---|
| `text_search_adapter` | ripgrep/livegrep-like search | raw matches with low semantic confidence |
| `tag_adapter` | ctags JSON/tag inventory | declarations and symbol inventory |
| `tree_sitter_adapter` | Tree-sitter parser/query packs | syntax nodes, spans, local relations |
| `structural_pattern_adapter` | ast-grep/Semgrep-like patterns | structural matches, rewrite candidates |
| `name_resolution_adapter` | stack-graph/heuristic/LSP/SCIP | resolved references and confidence |
| `scip_adapter` | SCIP index files | documents, occurrences, symbols, relationships |
| `fact_db_adapter` | Parceltongue SQLite or imported facts | graph edges, provenance, derivations |
| `rule_scan_adapter` | Semgrep/CodeQL-like scans | findings, diagnostics, policy warnings |
| `rewrite_adapter` | ast-grep/Comby-like rewrite | preview/apply mechanical changes |

The agent should not see all of this complexity.

The agent should see:

```text
symbol_context
read_next
impact_radius
review_context
structural_search
rewrite_preview
index_health
```

Internal complexity, external simplicity.

### Confidence Merging

When multiple tools disagree, Parceltongue should not hide the disagreement.

Example:

```text
Tree-sitter heuristic says A calls B.
SCIP has no B occurrence.
Text search finds B in comments only.
Semgrep pattern does not match a call shape.
```

Bad response:

```text
A calls B.
```

Better response:

```json
{
  "edge": "A -> B",
  "confidence": 0.42,
  "provenance": [
    {
      "source": "tree_sitter_heuristic",
      "confidence": 0.61
    },
    {
      "source": "scip",
      "confidence": 0.0,
      "note": "no symbol occurrence found"
    },
    {
      "source": "text_search",
      "confidence": 0.2,
      "note": "string match appears in comment"
    }
  ],
  "recommendation": "read raw call site before editing"
}
```

This is where Parceltongue can be better than a single source.

It can be honest.

### The Public Interface Dependency Graph

The user has asked earlier whether there is a universal way to capture
relationships like Parceltongue's dependency graph.

The answer after reading these systems:

```text
There is no single universal graph that all tools use.
But there are repeated interface ideas.
```

Recurring ideas:

| Idea | Seen In |
|---|---|
| Stable symbol identity | SCIP, Kythe, Glean |
| Document-relative paths | SCIP, Kythe, Glean |
| Occurrences with roles | SCIP, Kythe |
| Facts plus edges | Glean, Kythe, CodeQL |
| Queryable relationship services | Glean Glass, Kythe XRef, CodeQL |
| Name/scope resolution as separate concern | stack-graphs |
| Structural pattern rules | ast-grep, Semgrep, Comby |
| JSON or protocol output for client tools | SCIP, Ctags, Semgrep |
| Diagnostics/findings as first-class output | SCIP, Semgrep, CodeQL, Kythe |
| Generated/test roles | SCIP |
| Versioned schema/index metadata | SCIP, Glean, Kythe, stack-graphs |
| Explicit pagination/limits | Kythe, Semgrep API, Glean/Glass patterns |

So Parceltongue should not invent a totally isolated graph model.

It should implement a local graph with adapters to the common ideas:

```text
stable symbols
documents
occurrences
roles
relationships
facts
diagnostics
provenance
schema versions
confidence
freshness
```

That is the universal-ish core.

### What To Borrow, By Product Priority

| Priority | Borrow From | Borrow What | Why |
|---:|---|---|---|
| 1 | SCIP | Stable symbol identity, occurrence roles, relationships, external symbols, position encoding. | Prevents Parceltongue from repeating line-number identity mistakes. |
| 2 | Glean | Fact database mindset, derived facts, language-agnostic navigation layer. | Gives Parceltongue a durable architecture without forcing enterprise scale. |
| 3 | Kythe | VName discipline, graph/xref service shape, batching, pagination, explicit fact filters. | Strong prior art for graph APIs and identity/fact separation. |
| 4 | stack-graphs | Name/scope resolution as its own layer. | Stops naive Tree-sitter captures from masquerading as resolved symbols. |
| 5 | ast-grep | Code-shaped structural query and rewrite ergonomics. | Useful for agent-guided transformations after graph scoping. |
| 6 | Semgrep | MCP integration, rule schemas, findings model, scan hooks, deterministic guardrail flow. | Great model for making static analysis agent-friendly. |
| 7 | universal-ctags | Fast symbol fallback and JSON-lines output. | Broad fallback for unsupported languages and huge legacy repos. |
| 8 | Comby | Practical structural rewrite engine. | Good action layer after Parceltongue identifies targets. |
| 9 | CodeQL | Query-pack discipline and dataflow inspiration. | Useful for future advanced queries, but too heavy for the core product loop. |

### What Not To Build

Parceltongue should not become:

```text
a Semgrep clone
a CodeQL clone
a Glean clone
a Kythe clone
a generic rewrite engine
a full compiler frontend for all languages
a giant MCP exposing every internal table
```

The winning product shape is narrower:

```text
local code relationship broker for Codex
```

It should answer:

```text
What should the agent inspect next?
What relationship evidence supports this?
How fresh and complete is the answer?
Which dependency neighborhood matters for this edit?
Which files/tests should travel together?
Where should the agent fall back to raw reading?
```

That is enough.

### A Parceltongue Evidence Interface

This is the interface I would design after reading these systems:

```rust
enum EvidenceKind {
    TextMatch,
    TagDefinition,
    SyntaxNode,
    StructuralMatch,
    SymbolOccurrence,
    ResolvedReference,
    GraphEdge,
    DerivedFact,
    Diagnostic,
    RewriteCandidate,
}

enum EvidenceSource {
    Ripgrep,
    Ctags,
    TreeSitter,
    AstGrep,
    Semgrep,
    Comby,
    StackGraph,
    Scip,
    GleanImport,
    KytheImport,
    CodeQlImport,
    ParceltongueNative,
}

struct EvidenceRecord {
    kind: EvidenceKind,
    source: EvidenceSource,
    repo_root: PathBuf,
    file_path: Option<PathBuf>,
    span: Option<Span>,
    symbol_id: Option<SymbolId>,
    relation: Option<Relation>,
    confidence: f32,
    freshness: Freshness,
    provenance: Provenance,
    payload_ref: PayloadRef,
}
```

Then agent tools can be built on top:

```text
symbol_context(symbol)
  uses EvidenceRecord from SCIP, Tree-sitter, tags, and graph facts.

impact_radius(symbol)
  uses GraphEdge, ResolvedReference, DerivedFact, and test roles.

structural_search(pattern)
  uses AstGrep/Semgrep-style structural matches.

rewrite_preview(pattern, rewrite)
  uses Comby/ast-grep-style rewrite candidates.

index_health()
  reports missing/stale/low-confidence evidence sources.
```

This keeps Parceltongue extensible without making Codex choose between 40 tools.

### Product Recommendation

For the user's stated PMF:

```text
Solo agent power user.
All languages.
CRUD apps plus Rust/C/C++ systems programming.
Not a product for others.
Help Codex navigate large codebases faster and reliably with dependency clarity.
```

The best stack is:

```text
Core: Parceltongue native Tree-sitter graph
Identity: SCIP-compatible symbols where possible
Fallback: ctags plus ripgrep
Resolution: stack-graphs ideas or imports where feasible
Rules: Semgrep as an optional guardrail
Rewrite: ast-grep or Comby as optional action tools
Advanced fact inspiration: Glean and Kythe
Security/deep query inspiration: CodeQL
```

Not:

```text
Replace Parceltongue with Semgrep.
Replace Parceltongue with ast-grep.
Replace Parceltongue with CodeQL.
Replace Parceltongue with Glean.
```

Those tools do not solve the exact Codex context-navigation loop by themselves.

Parceltongue's moat is the agent-facing composition:

```text
relationship graph
freshness
confidence
minimal context
next-step hints
fallback evidence
rewrite/scanning handoffs
```

### Shreyas Doshi Read

From a product strategy POV:

```text
Do not compete with mature tools on their strongest jobs.
Use them as proof that the jobs exist.
Then own the job they do not optimize for.
```

The job they do not optimize for is:

```text
LLM asks a narrow code-navigation question.
Tool returns the smallest trustworthy context and dependency neighborhood.
LLM decides the next edit or read with fewer tokens.
```

ast-grep optimizes:

```text
find/rewrite this structural pattern
```

Semgrep optimizes:

```text
scan for this rule or vulnerability
```

SCIP optimizes:

```text
represent precise code navigation facts
```

Glean/Kythe/CodeQL optimize:

```text
store/query rich code facts at scale
```

Parceltongue should optimize:

```text
agent decision quality per token
```

That is the product wedge.

### Concept 8 Conclusion

The adjacent repos say:

```text
Do not build a parser toy.
Do not build a giant generic static analysis platform.
Build an agent-facing evidence broker.
```

The architecture should look like:

```text
Tree-sitter extracts local syntax.
SCIP-style symbols stabilize identity.
stack-graphs-style resolution separates syntax from meaning.
Glean/Kythe-style facts store relationships with provenance.
Semgrep-style rules and MCP patterns provide guardrails.
ast-grep/Comby-style structural tools execute scoped rewrites.
ctags/text search provide broad fallback.
Parceltongue packages all of this into small context bundles for Codex.
```

That is the synthesis.

The right final question for every future Parceltongue feature is:

```text
Does this help Codex choose the next correct read/edit with fewer tokens and
less uncertainty?
```

If yes, it belongs.

If it is just another parser capability with no agent decision loop, it is
secondary.

### Repos Already Touched For This Concept

| Repo | Status | Notes |
|---|---|---|
| `ast-grep__ast-grep` | Source-read | Structural search/lint/rewrite, Tree-sitter core, code-like patterns, YAML rules, multi-core scanning. |
| `semgrep__semgrep` | Source-read | Rule engine, scans, MCP server, custom-rule prompt, AST dump tool, findings model, path validation, local/remote scan split. |
| `scip-code__scip` | Source-read | Strongest direct reference for stable symbols, documents, occurrences, roles, relationships, external symbols, and streaming index shape. |
| `github__stack-graphs` | Source-read | Name/scope resolution architecture, Tree-sitter integration, CLI index/query, independently versioned language definitions; repo is no longer supported by GitHub. |
| `facebookincubator__Glean` | Source-read | Fact DB, code facts, xrefs, call/type hierarchies, SCIP/LSIF support, Glass language-agnostic navigation API, static LSP. |
| `github__codeql` | Source-read | Query libraries, database mindset, security analysis, dataflow/taint abstractions, diagnostics. |
| `kythe__kythe` | Source-read | VName identity, graph entries, graph/xref services, decorations, batching, pagination, filterable facts, dirty-buffer/workspace patching. |
| `universal-ctags__ctags` | Shallow cloned and source-read | Tag files, language-object inventory, JSON-lines output, client-tool list options, guest parsers, subparsers. |
| `comby-tools__comby` | Source-read | Structural search/rewrite for nested code, useful as scoped refactoring executor. |

## Concept 9: Build Parceltongue As A Codex Context Graph Service

### Core Idea

After the first eight concepts, the synthesis is clear.

Parceltongue should not be framed as:

```text
a Tree-sitter wrapper
a graph database demo
a giant HTTP endpoint collection
a static analysis platform
a replacement for Semgrep, SCIP, Glean, Kythe, CodeQL, ast-grep, or ctags
```

Parceltongue should be framed as:

```text
a local context graph service for Codex
```

Its job:

```text
When Codex needs to understand or edit a large codebase, Parceltongue returns
the smallest trustworthy graph-backed context that answers what to inspect next,
what depends on what, what is fresh, what is uncertain, and what tests or files
should travel with the edit.
```

That is the product.

Everything else is implementation detail.

### The One-Sentence Product Promise

```text
Parceltongue helps Codex navigate large codebases with dependency clarity while
spending fewer tokens and making fewer wrong reads.
```

That sentence should drive every technical decision.

If a feature does not help Codex navigate, decide, edit, review, or verify with
less uncertainty, it is secondary.

### The Product Boundary

Parceltongue should own:

```text
local indexing
safe parsing
stable symbols
code relationships
freshness tracking
impact radius
minimal context packing
agent-facing graph APIs
provenance and confidence
read-next recommendations
```

Parceltongue should not own, at least initially:

```text
general-purpose security scanning
full dataflow/taint engine
compiler-grade semantics for every language
visual IDE experience
hosted team product
natural-language chat agent
global rewrite engine
enterprise-scale fact database
```

Those jobs already have strong tools.

Parceltongue's personal power-user job is narrower and sharper:

```text
Make Codex less lost.
```

### Why The Old v1 Shape Was Right But Not Enough

The current README already has the right instinct:

```text
parse codebase with Tree-sitter
build dependency graph
query through HTTP API
blast radius
reverse callers
smart context
incremental reindexing
ingestion diagnostics
```

The problem is not that this direction was wrong.

The problem is that a Codex-grade version needs stronger foundations:

```text
stable identity instead of line-range identity
parser safety instead of ad hoc parsing
query packs instead of scattered grammar assumptions
freshness contracts instead of stale hidden state
agent-shaped MCP tools instead of endpoint sprawl
evidence provenance instead of "trust me" graph edges
confidence and ambiguity instead of silent guesses
adapter strategy instead of reinventing every mature tool
```

v1 proved:

```text
Code as graph helps agents.
```

v2 should prove:

```text
Code as a fresh, safe, provenance-rich context graph helps Codex make better
decisions repeatedly.
```

### The Architecture In One ASCII Diagram

```text
                                  Codex
                                    |
                                    v
                           Agent-Facing API
                 project_map | symbol_context | read_next
                 impact_radius | review_context | index_health
                                    |
                                    v
                           Context Planner
                 intent -> evidence plan -> token budget
                                    |
                                    v
                            Evidence Broker
          text | tags | tree-sitter | SCIP | Semgrep | ast-grep | Comby
                                    |
                                    v
                         Parceltongue Core Graph
            symbols | occurrences | edges | facts | diagnostics | freshness
                                    |
                                    v
                          Safe Parse Pipeline
            language registry | parser pool | query packs | extractors
                                    |
                                    v
                             Local Codebase
```

The design principle:

```text
Codex sees job-shaped tools.
Parceltongue internally composes many evidence layers.
```

### The Core Modules

| Module | Owns | Must Not Own |
|---|---|---|
| `LanguageRegistry` | Extensions, shebangs, user language overrides, grammar availability, parser/query versions. | Parse execution or graph writes. |
| `ParserRuntime` | Parser lifecycle, parser pools, timeouts, cancellation, included ranges, syntax-error status. | Symbol resolution or agent response formatting. |
| `QueryPackRegistry` | Versioned `.scm` query groups, captures, predicates, query tests, project overrides. | Recursive extraction state or DB storage. |
| `ExtractorEngine` | Converts parse trees plus queries into normalized nodes, occurrences, edge candidates, textobjects, runnables, redactions. | Final graph truth or agent context ranking. |
| `IdentityResolver` | Stable symbol IDs, SCIP-compatible identities, semantic paths, duplicate handling, rename/move matching. | Parser lifecycle. |
| `RelationResolver` | Import resolution, lexical scopes, call candidates, resolved references, confidence, unresolved counts. | Raw syntax walking. |
| `FreshnessManager` | File hashes, changed ranges, stale rows, dirty files, grammar/query-pack drift, pending sweeps. | Agent tool descriptions. |
| `GraphStore` | SQLite/local graph facts, provenance, indexes, FTS, transactions, diagnostics, parse artifacts. | Parser object lifetime. |
| `EvidenceBroker` | Merges native graph evidence with external adapters such as SCIP, ctags, Semgrep, ast-grep, Comby. | Long-lived parser objects. |
| `ContextPlanner` | Chooses what evidence to include for the agent under token budget. | Graph mutation. |
| `AgentApi` | MCP/CLI/HTTP tools, compact output envelopes, ambiguity and warning surfaces. | Low-level extraction logic. |
| `Evaluator` | Golden tasks, precision checks, token-cost metrics, freshness tests, benchmark reports. | Runtime business logic. |

The important point:

```text
No module should own everything.
```

Parceltongue v1 had too much product value trapped behind endpoint-level
shapes. v2 should make internal boundaries explicit so the agent surface can
stay small.

### Data Model: Minimum Durable Core

The graph store should start with boring tables.

```text
repo
  repo_id
  root_path
  current_git_head
  created_at

document
  document_id
  repo_id
  relative_path
  language_id
  file_hash
  last_modified_unix
  size_bytes
  supported_status

parse_artifact
  artifact_id
  document_id
  parser_version
  grammar_name
  grammar_abi
  query_pack_version
  status
  syntax_error_count
  elapsed_ms
  warning_count

symbol
  symbol_id
  scheme
  package_manager
  package_name
  package_version
  descriptor_path
  display_name
  kind
  stable_key

occurrence
  occurrence_id
  document_id
  symbol_id nullable
  start_byte
  end_byte
  start_line
  start_col
  end_line
  end_col
  role_bits
  syntax_kind
  enclosing_symbol_id nullable
  confidence

edge
  edge_id
  from_symbol_id
  to_symbol_id nullable
  edge_kind
  confidence
  resolution_method
  provenance_id
  stale_status

fact
  fact_id
  subject_kind
  subject_id
  fact_name
  fact_value_json
  provenance_id

diagnostic
  diagnostic_id
  document_id
  severity
  code
  message
  source
  span nullable

provenance
  provenance_id
  evidence_source
  artifact_id nullable
  query_pack_version nullable
  adapter_version nullable
  created_at
```

This is not a perfect schema.

It is the right shape:

```text
documents
parse artifacts
symbols
occurrences
edges
facts
diagnostics
provenance
```

Those are the repeated ideas across SCIP, Glean, Kythe, CodeQL, and the better
Tree-sitter graph tools.

### Stable Identity Comes First

The older RCA and ISGL1 documents are blunt:

```text
line-number-based keys break incremental indexing
```

The v2 roadmap should treat stable identity as prerequisite work.

Do not build fancy agent context on top of unstable IDs.

Stable identity requirements:

```text
1. Symbol identity cannot include mutable line ranges.
2. Position lives in occurrence rows.
3. Revision/time lives in facts or artifacts.
4. One symbol can have many occurrences over time.
5. A moved symbol should preserve identity when semantic match is strong.
6. Duplicate local names need enclosing scope or local IDs.
7. External symbols need package/module identity.
8. Unresolved references still get rows.
```

SCIP and Kythe both reinforce the same point:

```text
Name says what the thing is.
Facts say where and when it exists.
```

That should become the Parceltongue law.

### The Parse Pipeline

The safe parse pipeline should be:

```text
discover file
  -> detect language
  -> check file size and ignore rules
  -> check freshness by hash plus grammar/query versions
  -> acquire parser
  -> parse with timeout and cancellation
  -> store parse artifact status
  -> run query packs
  -> run recursive extractors when queries are too weak
  -> emit normalized evidence
  -> resolve identity
  -> resolve relations
  -> write graph transaction
  -> update index health
```

Every step should be allowed to produce structured warnings.

Example:

```text
unsupported language
file too large
parse timed out
syntax errors found
query pack missing
edge unresolved
symbol identity ambiguous
grammar version changed
adapter unavailable
```

Warnings must not be buried in logs only.

They should be visible to:

```text
index_health
symbol_context
impact_radius
review_context
```

### Query Pack Roadmap

Start with these query groups:

| Query Group | Agent Question | Examples |
|---|---|---|
| `outline.scm` | What symbols exist in this file? | functions, classes, structs, traits, modules |
| `locals.scm` | What is defined and referenced locally? | definitions, references, scopes |
| `imports.scm` | What external modules enter this file? | imports, uses, requires, includes |
| `calls.scm` | What call expressions exist? | function calls, method calls, constructors |
| `exports.scm` | What is public API? | exports, pub items, module exports |
| `tests.scm` | What tests or verification hooks exist? | tests, fixtures, doc tests, benchmarks |
| `routes.scm` | What request entry points exist? | HTTP routes, controllers, handlers |
| `textobjects.scm` | What is the smallest useful read/edit unit? | function body, class body, call expression |
| `redactions.scm` | What should not be sent to the model? | secrets, private keys, tokens |
| `injections.scm` | What embedded languages exist? | JS in HTML, SQL strings, GraphQL literals |

The query pack API should expose:

```text
query_pack_version
capture_name
semantic_fact_type
test_fixture_count
last_verified_at
```

Do not let query strings become invisible magic.

They are product logic.

### Agent API: Small Public Surface

The public MCP surface should be small.

Recommended first tools:

| Tool | Job |
|---|---|
| `project_map` | Orient Codex to languages, folders, modules, top entry points, index health. |
| `search_code` | Search by symbol/text/concept with provenance and confidence. |
| `symbol_context` | Return definition, callers, callees, references, tests, and minimal snippets for one symbol. |
| `read_next` | Given a task or symbol, recommend the next smallest files/symbols to inspect. |
| `impact_radius` | Return dependency blast radius for a symbol/file/diff with ranked affected nodes. |
| `review_context` | Given git diff, return changed symbols, risk, impacted files, likely tests, and missing reads. |
| `index_health` | Report freshness, parse failures, skipped files, stale grammar/query versions, and coverage. |

Optional later:

| Tool | Job |
|---|---|
| `route_trace` | CRUD route to handler to service to DB/table/query to tests. |
| `structural_search` | Scoped ast-grep/Semgrep-like structural matching. |
| `rewrite_preview` | Scoped Comby/ast-grep-like rewrite preview. |
| `reindex_scope` | Refresh one file, folder, language, or dependency neighborhood. |
| `graph_query` | Expert escape hatch for custom graph queries. |

Do not expose 26 raw endpoints to Codex as the primary surface.

Expose job-shaped tools.

### The Standard Response Envelope

Every public tool should return a shared envelope:

```json
{
  "status": "ok|partial|stale|ambiguous|error",
  "freshness": {
    "fresh": true,
    "stale_files": 0,
    "dirty_files": 0
  },
  "budget": {
    "max_tokens": 4000,
    "estimated_tokens": 1840,
    "truncated": false
  },
  "confidence": {
    "overall": 0.86,
    "main_reason": "SCIP-backed definition plus Tree-sitter callers"
  },
  "warnings": [],
  "answer": {},
  "evidence": [],
  "hidden_counts": {},
  "next": []
}
```

This envelope is the difference between:

```text
raw database endpoint
```

and:

```text
agent decision support
```

### Codex Rituals Parceltongue Should Teach

The product should shape Codex habits.

Ritual 1: First contact with repo

```text
project_map
search_code for user concept
read_next
symbol_context
```

Ritual 2: Before editing

```text
symbol_context
impact_radius
review_context on current diff if any
read_next for missing dependencies
```

Ritual 3: After editing

```text
reindex touched files
review_context
impact_radius for changed symbols
tests/runnables from touched area
optional Semgrep scan if security-sensitive
```

Ritual 4: Debugging

```text
search_code error message
symbol_context likely entry point
call path toward failing function
impact_radius from suspicious function
read_next for state/config/dependency owner
```

Ritual 5: Refactor planning

```text
project_map scoped to module
cycles and SCCs
impact_radius for candidate cut points
public interface graph
tests/runnables
rewrite_preview if mechanical change exists
```

These rituals matter because the user is not building a product for others.

The product is a personal Codex muscle.

### External Adapter Strategy

Do not rebuild everything.

Use external tools as optional adapters.

| Adapter | Use When | Output To Normalize |
|---|---|---|
| `scip_adapter` | SCIP index exists or language indexer is easy to run. | Symbols, occurrences, roles, relationships. |
| `ctags_adapter` | Language unsupported or fast broad symbol inventory needed. | Tag definitions, kinds, scopes. |
| `semgrep_adapter` | Need rule scan, security guardrail, custom pattern check. | Findings, diagnostics, structural matches. |
| `ast_grep_adapter` | Need precise code-shaped structural search or rewrite. | Matches, captures, rewrite candidates. |
| `comby_adapter` | Need broad structural rewrite with nested delimiters. | Rewrite preview/apply results. |
| `clarity_adapter` | Need file/module dependency sanity or visualization. | Module edges, reachability, cycles. |
| `cocoindex_or_codemogger_adapter` | Need semantic/keyword code search. | Chunks, embeddings/search hits, file spans. |

Important:

```text
Adapters should never be mandatory for core operation.
Adapters should add confidence, recall, or actionability.
```

The native graph must stand on its own for the user's repos.

### Build Order

A practical roadmap:

| Phase | Name | Output |
|---:|---|---|
| 0 | Baseline Benchmark | Run current v1.7.2 and top external tools on the same tasks. |
| 1 | Stable Identity Core | Replace line-based entity identity with stable symbol plus occurrence model. |
| 2 | Safe Parse Runtime | Parser pool, parse status, timeouts, file-size guards, syntax-error tolerant artifacts. |
| 3 | Query Pack System | Versioned query packs, fixtures, capture contracts, test runner. |
| 4 | Graph Store v2 | SQLite/local store with symbols, occurrences, edges, facts, provenance, diagnostics. |
| 5 | Freshness Manager | Hashes, changed ranges, dirty files, grammar/query drift, stale-row reporting. |
| 6 | Agent API v1 | `project_map`, `search_code`, `symbol_context`, `impact_radius`, `read_next`, `index_health`. |
| 7 | Review Loop | `review_context`, touched-file reindex, likely tests, diff impact. |
| 8 | Evidence Adapters | SCIP import, ctags fallback, Semgrep scan, ast-grep/Comby rewrite preview. |
| 9 | Evaluation Harness | Golden tasks, token cost, precision/recall, stale-answer checks, regression fixtures. |

Do not start with adapters.

Start with identity, parser safety, graph facts, freshness, and agent API.

Adapters become useful only after the native evidence model exists.

### Phase 0 Benchmark Tasks

Before writing v2 code, benchmark current tools against the same tasks:

```text
1. Find the entry point for a feature by concept name.
2. Find exact callers and callees for a symbol.
3. Determine what breaks if one symbol changes.
4. Given a git diff, identify changed symbols and impacted files.
5. Find likely tests for a changed symbol.
6. Trace CRUD route to handler to service to repository/table.
7. Trace a Rust/C/C++ function across modules/headers.
8. Identify stale or skipped files in the index.
9. Validate whether a hallucinated symbol exists.
10. Produce minimal context under 4000 tokens.
```

For each tool:

```text
number of calls
latency
tokens returned
correctness
confidence disclosure
freshness disclosure
setup friction
failure mode
```

This benchmark should decide whether Parceltongue v2 is worth building deeper.

My prediction:

```text
External tools will help, but none will perfectly own the Codex-specific context
graph loop for the user's exact mixed-language personal workflow.
```

But measure it.

### What v2 Should Optimize

The key metric should not be:

```text
number of endpoints
number of graph algorithms
number of supported languages
```

The key metric should be:

```text
agent decision quality per token
```

Candidate metrics:

| Metric | Meaning |
|---|---|
| `tokens_to_correct_entry_point` | How many returned tokens before Codex identifies the right starting file/symbol? |
| `calls_to_correct_blast_radius` | How many tool calls before correct impacted files are found? |
| `context_precision` | Percent of returned snippets actually needed for the task. |
| `context_recall` | Percent of necessary snippets returned within budget. |
| `stale_answer_rate` | Percent of answers missing freshness warnings when stale data exists. |
| `unresolved_edge_visibility` | Whether unresolved references are counted and disclosed. |
| `parse_failure_visibility` | Whether skipped/timed-out files appear in health and task answers. |
| `edit_review_hit_rate` | Whether `review_context` catches files/tests Codex would otherwise miss. |
| `manual_read_reduction` | How many raw `cat/sed/rg` reads are avoided. |

This is the evaluation wedge.

### MVP Language Strategy

The user wants:

```text
all languages
CRUD apps
Rust/C/C++ systems programming
```

The practical strategy:

| Language Family | MVP Approach |
|---|---|
| TypeScript/JavaScript | Tree-sitter plus imports/exports/routes/calls; SCIP import if available. |
| Python | Tree-sitter plus imports/classes/functions/calls; tolerate dynamic unresolved edges. |
| Rust | Tree-sitter plus modules/uses/functions/impls/traits/tests; optional rust-analyzer/SCIP import later. |
| C/C++ | Tree-sitter plus functions/classes/includes; ctags fallback; compile-db/SCIP-clang later. |
| Java/Go/C# | Tree-sitter plus class/function/import/call basics; use SCIP or LSP-derived facts when available. |
| Config/SQL/GraphQL/Markdown | Text and query-pack/injection support; mark confidence clearly. |

The promise should not be:

```text
perfect semantic graph for every language
```

The promise should be:

```text
best available evidence with confidence and fallback.
```

### Public Interface Graph

Concept 4 introduced the public interface graph idea.

Concept 9 makes it a roadmap item.

Parceltongue should compute at least two graph views:

```text
internal graph
  all extracted symbols and edges

public interface graph
  exported/public/API/route/trait/interface/class/module boundaries
```

Why:

```text
Agents drown in private helper edges.
Most edit-risk questions start at public seams.
CRUD apps revolve around route/service/repository boundaries.
Systems repos revolve around exported functions, headers, traits, FFI, and modules.
```

Agent API:

```text
impact_radius(symbol, view="public")
impact_radius(symbol, view="internal")
```

Default:

```text
public first, internal on drill-down
```

This is how Parceltongue saves tokens.

### CRUD Route Trace

For CRUD apps, this should be a killer workflow.

Target output:

```text
route
  -> middleware
  -> handler/controller
  -> service
  -> repository/model/query
  -> database table or external API
  -> tests
```

Evidence sources:

```text
Tree-sitter routes.scm
imports.scm
calls.scm
framework query packs
string/config search
Semgrep-style project rules
SCIP/LSP if available
```

Tool:

```text
route_trace(path_or_handler, max_tokens=4000)
```

This is not a generic graph algorithm.

It is a product workflow.

### Rust/C/C++ Systems Trace

For systems code, the killer workflow is different:

```text
symbol
  -> definition
  -> declarations/prototypes
  -> impl/function body
  -> callers
  -> callees
  -> include/module owners
  -> FFI/export boundary
  -> tests/benches/examples
```

Special complications:

```text
headers
macros
conditional compilation
trait methods
method dispatch
templates
unsafe blocks
build scripts
generated bindings
```

The roadmap should not promise perfect C++ semantics immediately.

It should expose confidence:

```text
definition exact via ctags/SCIP
call edge inferred via Tree-sitter
include edge exact
method dispatch unresolved
macro expansion unknown
```

This kind of honesty makes the tool useful before it is perfect.

### The Codex Config Surface

The user wants to use Codex app.

So Parceltongue should ship with:

```text
MCP server
CLI
AGENTS.md snippet
Codex setup command
health check
doctor command
benchmark command
```

Recommended commands:

```bash
parceltongue init
parceltongue index .
parceltongue mcp
parceltongue health
parceltongue doctor
parceltongue benchmark .
parceltongue explain-tooling
```

Recommended agent instruction:

```text
Before editing a non-trivial symbol, call `symbol_context` and `impact_radius`.
After editing, refresh touched files and call `review_context`.
When graph confidence is low, use raw file reads and report uncertainty.
```

This is the personal habit loop.

### What To Do Immediately In Codex Before v2 Exists

Use the existing external-tool research pragmatically.

For current personal workflow:

```text
Primary graph benchmark:
  codebase-memory-mcp
  sdsrss/code-graph-mcp
  code-review-graph or better-code-review-graph

Search layer:
  cocoindex-code or codemogger

Structural sanity:
  Clarity CLI

Rule/scan guardrail:
  Semgrep MCP if security-sensitive

Structural rewrite:
  ast-grep or Comby manually through shell
```

This lets the user get value now while Parceltongue v2 is designed.

Parceltongue v2 should then be judged against that stack.

If the stack already solves the job well enough, Parceltongue can remain a
personal research substrate.

If the stack leaves Codex juggling too many tools with inconsistent freshness and
confidence, Parceltongue has a clear reason to exist.

### The V2 North Star

The north star workflow:

```text
User: "Change this behavior."

Codex:
  1. Calls Parceltongue `search_code` or `symbol_context`.
  2. Gets exact symbol plus confidence.
  3. Calls `impact_radius`.
  4. Gets minimal dependency neighborhood.
  5. Calls `read_next`.
  6. Reads only 2-5 targeted snippets.
  7. Edits.
  8. Parceltongue refreshes touched files.
  9. Codex calls `review_context`.
  10. Runs recommended tests.
  11. Reports uncertainty and evidence.
```

The user should feel:

```text
Codex is not wandering.
```

That feeling is the product.

### Risks

| Risk | Mitigation |
|---|---|
| Too many tools exposed to Codex | Keep public MCP surface small; use internal adapters. |
| Naive graph edges mislead agent | Store confidence, unresolved counts, and provenance. |
| Stale graph answers | Freshness manager, dirty-file checks, explicit stale status. |
| Parser crashes or hangs | Safe parse runtime, timeouts, full-index subprocess isolation. |
| Identity breaks after edits | Stable symbols plus occurrence rows; no line ranges in identity. |
| Overbuilding enterprise substrate | Keep local SQLite-first implementation. |
| Focusing on graph algorithms over workflows | Build around Codex rituals and benchmark tasks. |
| Unsupported languages produce silence | Use ctags/text fallback and disclose confidence. |
| Adapter sprawl | Add adapters only after native graph model is stable. |
| Token savings become vague marketing | Measure context precision, recall, and token cost. |

### Shreyas Doshi Read

The product decision:

```text
Do not build "a better code graph."
Build "the pre-edit and post-edit ritual for Codex."
```

Habit loop:

```text
Trigger:
  Codex is about to edit or review code.

Action:
  Ask Parceltongue for symbol context, impact radius, and read-next.

Reward:
  Codex reads fewer files and catches dependency risk.

Investment:
  Refreshed graph, better project query packs, confidence improves over time.
```

That is a much stronger product loop than:

```text
Run this analysis endpoint when curious.
```

### Concept 9 Conclusion

Parceltongue v2 should be a local Codex context graph service with:

```text
stable symbol identity
safe parser lifecycle
versioned query packs
provenance-rich graph facts
freshness-aware indexing
small MCP surface
read-next planning
impact-radius answers
review-context ritual
external evidence adapters
evaluation by agent decision quality per token
```

The core product is not parsing.

The core product is:

```text
Codex knows where to look next.
```

Everything should serve that.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| `README.md` | Source-read | v1.7.2 product shape: Tree-sitter, graph DB, HTTP API, blast radius, smart context, incremental reindexing, diagnostics. |
| `docs/research002/J003.md` | Source-read | PMF rubric and Codex arsenal: graph brain, review/change safety, search/discovery, structural sanity. |
| `docs/research001/unclassified/RCA-Incremental-Indexing-Failure.md` | Source-read | Root cause of broken incremental indexing: endpoint wiring plus line-number key instability. |
| `docs/research001/unclassified/ISGL1-v2-Stable-Entity-Identity.md` | Source-read | Stable identity direction and why position-based keys break graph continuity. |
| `docs/research001/PRD-research-20260131v1/PARSELTONGUE_V2_LLM_AGENT_INTERACTIONS.md` | Source-read | Iterative LLM plus graph loop: CPU graph features help LLM ask better next questions. |
| `docs/research001/PRD-research-20260131v1/PARSELTONGUE_V2_BIDIRECTIONAL_LLM_ENHANCEMENT.md` | Source-read | Bidirectional LLM/CPU loop: LLM semantic hints can improve graph algorithms and prioritization. |
| Concepts 1-8 in this document | Source-read through current synthesis | Parser boundary, query packs, AST walkers, provenance edges, freshness, agent APIs, parser safety, adjacent evidence layers. |

## Concept 10: Evaluate Agent Decision Quality Per Token

### Core Idea

Parceltongue v2 should not be justified by taste.

It should be justified by a benchmark:

```text
Does this tool help Codex find the right code, understand the right
relationships, and make the right edit/review decision with fewer tokens,
fewer calls, and less uncertainty?
```

This is the evaluation target:

```text
agent decision quality per token
```

Not:

```text
number of supported languages
number of endpoints
number of graph algorithms
number of extracted nodes
number of GitHub stars
```

Those can support the product. They are not the product.

The product is:

```text
Codex makes better next moves in large codebases.
```

So the evaluation harness should measure the next move.

### Why The Evaluation Harness Must Come Before v2 Implementation

Without an eval harness, Parceltongue will drift toward attractive but
unmeasured features:

```text
more endpoints
more graph algorithms
more parser grammars
more visualizations
more database tables
more theoretical correctness
```

Those might feel productive while missing the user job.

The harness forces uncomfortable questions:

```text
Did Codex read fewer files?
Did it find the real caller?
Did it avoid stale graph answers?
Did it notice ambiguous symbols?
Did it identify relevant tests?
Did it preserve public API impact?
Did it handle half-written code?
Did it say "I do not know" when evidence was weak?
```

This is a Shreyas-style product guardrail:

```text
Measure the user journey, not the implementation artifact.
```

### The Competing Baselines

Every Parceltongue v2 claim should be tested against baselines.

| Baseline | Why It Matters |
|---|---|
| `Codex + rg/read only` | The true default workflow. If Parceltongue cannot beat this, it should not exist. |
| `Parseltongue v1.7.2` | Measures whether v2 improves over existing product instinct. |
| `codebase-memory-mcp` | Strongest agent graph reference from current research. |
| `sdsrss/code-graph-mcp` | Strong shape match for project map, search, call graph, route trace, impact. |
| `code-review-graph` or `better-code-review-graph` | Best review/change-safety workflow reference. |
| `cocoindex-code` or `codemogger` | Search-layer baseline. Good for discovery but not dependency graph. |
| `Clarity` | File/module graph sanity baseline. Useful for reachability and cycles. |
| `Semgrep MCP` | Rule/guardrail baseline, not graph baseline. |
| `ast-grep` or `Comby` | Structural search/rewrite baseline. |

The point is not to "win" against every tool.

The point is to know:

```text
Where Parceltongue is meaningfully better.
Where it should adapt existing tools.
Where it should stop building.
```

### Evaluation Harness Shape

The harness should produce repeatable benchmark runs.

```text
bench/
  cases/
    crud_route_trace.yaml
    rust_trait_impact.yaml
    cpp_header_callers.yaml
    stale_index_after_edit.yaml
    ambiguous_symbol.yaml
    generated_file_skip.yaml
  expected/
    crud_route_trace.expected.json
    rust_trait_impact.expected.json
  runs/
    2026-07-06-parceltongue-v2/
    2026-07-06-codebase-memory/
    2026-07-06-rg-baseline/
  reports/
    leaderboard.md
    failure-analysis.md
```

Each case should define:

```text
repo
setup commands
user prompt
allowed tools
tool budget
token budget
time budget
expected files
expected symbols
expected edges
expected tests
expected uncertainty warnings
grading rules
```

This should be mostly deterministic.

An LLM can help grade narrative quality, but the core facts should be checked
with structured assertions.

### Benchmark Case Schema

Example:

```yaml
id: crud_route_trace_001
title: Trace a route from HTTP endpoint to DB table
repo: fixtures/crud-ts-express
prompt: "Where does POST /users update the database, and what tests matter?"
max_tool_calls: 8
max_returned_tokens: 5000
max_wall_ms: 10000

expected:
  files:
    must_include:
      - src/routes/users.ts
      - src/controllers/userController.ts
      - src/services/userService.ts
      - src/repositories/userRepository.ts
      - tests/userRoutes.test.ts
    must_not_include:
      - dist/bundle.js
      - node_modules/*
  symbols:
    must_include:
      - registerUserRoutes
      - updateUserController
      - updateUserService
      - UserRepository.update
  edges:
    must_include:
      - from: updateUserController
        to: updateUserService
        kind: calls
      - from: updateUserService
        to: UserRepository.update
        kind: calls
      - from: UserRepository.update
        to: users_table
        kind: writes
  warnings:
    must_include:
      - if_db_table_edge_is_inferred

grading:
  correctness_points: 45
  token_points: 20
  freshness_points: 15
  uncertainty_points: 10
  workflow_points: 10
```

The expected answer does not need to require exact wording.

It requires exact evidence.

### Scoring Rubric

Use a 100-point score per case.

| Category | Points | What It Measures |
|---|---:|---|
| Correctness | 45 | Required files, symbols, edges, tests, and impact facts are present and wrong facts are limited. |
| Token Efficiency | 15 | Returned context fits the budget and avoids irrelevant dumps. |
| Tool-Call Efficiency | 10 | Codex reaches the answer in a small number of calls. |
| Freshness Handling | 10 | Dirty/stale files are detected or refreshed; stale answers are not presented as fresh. |
| Uncertainty Honesty | 10 | Ambiguity, unresolved references, parse failures, generated-file skips, and low-confidence edges are disclosed. |
| Workflow Usefulness | 10 | The answer tells Codex the next read/edit/test action, not just raw data. |

Pass/fail gate:

```text
Any answer that silently presents stale graph data as fresh fails the case.
Any answer that misses a required high-risk impacted file fails the case.
Any answer that crashes/hangs the tool fails the case.
```

This makes safety non-negotiable.

### Core Benchmark Tasks

Start with 12 tasks.

| ID | Task | Why It Matters |
|---|---|---|
| `orientation_001` | New repo, ask "what are the main parts?" | Tests project map and first-contact usefulness. |
| `symbol_lookup_001` | Find exact definition for a common ambiguous name. | Tests search, ambiguity, stable identity. |
| `callers_001` | Ask "who calls this?" for a non-trivial symbol. | Tests direct caller accuracy. |
| `callees_001` | Ask "what does this call?" | Tests local extraction and relation resolution. |
| `impact_001` | Ask "what breaks if I change this?" | Tests blast radius and ranking. |
| `review_diff_001` | Given a git diff, return changed symbols, risk, tests. | Tests the pre/post edit ritual. |
| `route_trace_001` | Trace CRUD route to handler/service/repository/table/tests. | Tests product fit for CRUD apps. |
| `systems_trace_001` | Trace Rust/C/C++ function through module/header boundary. | Tests product fit for systems programming. |
| `stale_index_001` | Change a file after indexing, then ask graph question. | Tests freshness contract. |
| `syntax_error_001` | Half-edit a file with syntax error and ask for context. | Tests parser tolerance. |
| `unsupported_lang_001` | Include unsupported/niche language file with important symbols. | Tests fallback and warning behavior. |
| `generated_file_001` | Repo includes huge generated file with many matches. | Tests file-size guards and skip disclosure. |

These 12 tasks cover the user's real world:

```text
all languages
CRUD apps
Rust/C/C++ systems code
large repo navigation
dependency clarity
Codex editing loop
```

### Expanded Failure Tasks

Add these after the first harness works:

| ID | Failure Mode | Expected Good Behavior |
|---|---|---|
| `ambiguous_symbol_002` | Two symbols have the same display name in different modules. | Return candidates and ask/choose based on file context. |
| `rename_move_001` | Function moves without semantic change. | Preserve stable identity or mark high-confidence move. |
| `line_shift_001` | Add comments above many functions. | Do not report false deletes/adds for unchanged symbols. |
| `parser_timeout_001` | Pathological file times out. | Mark file timed out; keep rest of graph usable. |
| `grammar_drift_001` | Query pack version changes after indexing. | Mark affected files stale or require reindex. |
| `unresolved_import_001` | Import alias cannot be resolved. | Store unresolved reference and disclose count. |
| `framework_magic_001` | Framework route/model wiring is convention-based. | Use partial evidence, lower confidence, recommend raw read. |
| `secret_redaction_001` | File contains token/private key-like content. | Redact or avoid sending secret spans in context. |
| `test_relevance_001` | Multiple test files exist, only one is relevant. | Rank likely tests with evidence. |
| `adapter_disagreement_001` | SCIP and Tree-sitter disagree on a relationship. | Show provenance disagreement and recommend verification. |

These are the cases that prevent "cool demo, brittle daily tool."

### What The Harness Should Capture Per Run

Each run should produce a machine-readable result:

```json
{
  "case_id": "impact_001",
  "tool_under_test": "parceltongue-v2",
  "repo": "fixtures/rust-service",
  "status": "pass",
  "score": 87,
  "tool_calls": 4,
  "returned_tokens": 3120,
  "wall_ms": 1840,
  "correct_files": 7,
  "missing_files": [],
  "wrong_files": ["src/unused_legacy.rs"],
  "correct_edges": 12,
  "missing_edges": 1,
  "stale_warning_present": true,
  "uncertainty_warnings": [
    "1 unresolved trait method dispatch"
  ],
  "next_actions": [
    "read src/service/user.rs",
    "run cargo test user_update"
  ]
}
```

And a human-readable report:

```text
Case: impact_001
Score: 87/100

Strong:
  Found all required impacted files.
  Stayed under 4000 tokens.
  Disclosed unresolved trait dispatch.

Weak:
  Included one unused legacy file.
  Missed one indirect test dependency.

Decision:
  Good enough for pre-edit Codex context.
```

### Token Budget Bands

Use fixed budgets so tools cannot win by dumping more context.

| Task Class | Budget |
|---|---:|
| Orientation | 2000 tokens |
| Symbol lookup | 1500 tokens |
| Direct callers/callees | 2500 tokens |
| Impact radius | 4000 tokens |
| Review diff | 5000 tokens |
| Route trace | 5000 tokens |
| Systems trace | 6000 tokens |
| Debug/root cause | 6000 tokens |
| Refactor planning | 8000 tokens |

The score should penalize:

```text
returning huge raw file dumps
including unrelated files
omitting hidden/truncated counts
using more than budget without saying so
```

The harness should count returned tokens approximately but consistently.

### Correctness Is More Than Exact Match

Graph answers have acceptable variation.

For example, in a route trace:

```text
Controller -> Service -> Repository
```

Another tool may return:

```text
Route -> Controller -> Service -> ORM Model
```

Both can be acceptable if the critical path is present.

So expected answers should include:

```text
must_include
nice_to_include
must_not_include
acceptable_aliases
acceptable_uncertainty
```

Example:

```yaml
expected:
  files:
    must_include:
      - src/controllers/userController.ts
    nice_to_include:
      - src/middleware/auth.ts
    must_not_include:
      - dist/generated-client.ts
  symbols:
    aliases:
      updateUser:
        - update_user
        - updateUserHandler
```

The harness should reward useful evidence, not brittle exact strings.

### Deterministic Grading First

Use deterministic grading for:

```text
required files present
required symbols present
required edges present
forbidden files absent
token count
tool call count
freshness warning present
parse warning present
```

Use LLM-assisted grading only for:

```text
quality of next-step recommendation
clarity of risk explanation
whether answer is actionable
whether uncertainty is understandable
```

This avoids the eval becoming another vibe machine.

### Golden Fixtures

The harness needs small but realistic fixture repos.

| Fixture | Contents | Purpose |
|---|---|---|
| `fixtures/crud-ts-express` | Express routes, controllers, services, repository, tests, generated client. | CRUD route trace and generated-file skip. |
| `fixtures/rust-cli-modules` | Rust modules, traits, impls, tests, feature flags. | Rust symbol/trait impact. |
| `fixtures/cpp-headers` | Headers, source files, macros, includes, tests. | C/C++ declaration/definition/caller trace. |
| `fixtures/python-django-lite` | URLs, views, models, service functions, tests. | Dynamic framework path and unresolved edges. |
| `fixtures/mixed-monorepo` | TS app plus Rust worker plus config files. | Multi-language project map and cross-boundary search. |
| `fixtures/bad-syntax` | Half-edited files with syntax errors. | Parser tolerance. |
| `fixtures/identity-line-shift` | Same functions before/after line shifts. | Stable identity regression. |

Keep fixtures small enough to inspect manually.

The goal is not a huge benchmark.

The goal is a hard, honest one.

### Real-Repo Benchmark Set

After fixtures, run on real repos from the shallow clone set.

Pick:

| Repo | Why |
|---|---|
| `sdsrss__code-graph-mcp` | Rust MCP graph tool; good for self-similar analysis. |
| `tirth8205__code-review-graph` | Python graph/review tool; good for agent-workflow comparisons. |
| `Christoph__treesitter-mcp` | Small Rust Tree-sitter MCP; good for endpoint/context comparison. |
| `cocoindex-io__cocoindex-code` | Search/index layer; good for search baseline. |
| `LegacyCodeHQ__clarity-cli` | CLI graph tool; good for file/module graph sanity. |
| `ast-grep__ast-grep` | Rust structural search tool; good for large Rust codebase navigation. |
| `semgrep__semgrep` | Large mixed static-analysis repo; stress test for search and context limits. |

Real repos reveal:

```text
setup friction
indexing latency
memory behavior
tool crashes
language support reality
token bloat
adapter usefulness
```

### Tool Comparison Report

The final report should look like:

| Tool | Avg Score | Best At | Worst At | Calls | Tokens | Setup Friction | Keep? |
|---|---:|---|---|---:|---:|---|---|
| `Codex+rg` | 52 | raw recall | dependency impact | 18 | 14000 | none | baseline |
| `Parseltongue v1.7.2` | 61 | graph endpoints | stable identity/freshness | 7 | 7000 | medium | compare |
| `codebase-memory-mcp` | 78 | persistent graph Q&A | unknown precision gaps | 4 | 4200 | low/medium | benchmark |
| `sdsrss/code-graph-mcp` | 80 | route/project graph | maturity | 4 | 3600 | medium | benchmark |
| `code-review-graph` | 82 | diff/review flow | non-review exploration | 3 | 3900 | low | benchmark |
| `cocoindex-code` | 70 | semantic search | impact graph | 2 | 3000 | low | adapter |
| `Clarity` | 68 | module reachability | symbol calls | 2 | 2500 | low | adapter |
| `Parceltongue v2` | target 85+ | context graph service | TBD | target 3-5 | target under budget | target low | build if real |

The numbers above are placeholders.

The harness should generate the real table.

### The Main Score: DQPT

Define:

```text
DQPT = Decision Quality Per Token
```

Simple version:

```text
DQPT = correctness_score / returned_tokens * 1000
```

Better version:

```text
DQPT = (correctness + freshness + uncertainty + workflow) / returned_tokens * 1000
```

Where:

```text
correctness = 0 to 45
freshness = 0 to 10
uncertainty = 0 to 10
workflow = 0 to 10
```

Why DQPT matters:

```text
An agent can only reason over what fits in context.
The best graph tool is not the one with the most data.
It is the one that returns the highest-value data for the next decision.
```

This metric should appear in every benchmark report.

### Freshness-Specific Eval

Freshness deserves its own mini-suite.

Cases:

```text
1. Index repo.
2. Ask "who calls X?"
3. Edit a caller.
4. Ask again before refresh.
5. Ask again after refresh.
```

Expected behavior:

```text
before refresh:
  status is stale or dirty
  tool either refreshes named file or warns

after refresh:
  new caller appears
  stale warning disappears
  old edge is removed if applicable
```

Fail:

```text
tool returns old answer as fresh
```

This is non-negotiable because stale graph answers are worse than no graph.

### Identity-Specific Eval

Identity deserves its own mini-suite too.

Case:

```text
1. Index file with functions A, B, C.
2. Add 10 comment lines above B.
3. Reindex.
4. Compare symbol identities.
```

Expected:

```text
A, B, C keep stable symbol identity.
Occurrence spans update.
No false delete/add for B and C.
Edges continue pointing to same stable symbols.
```

Fail:

```text
line shift creates new keys
blast radius loses entity
diff shows unchanged functions as deleted/added
```

This is the old Parseltongue wound. The eval must guard it forever.

### Parser-Safety Eval

Parser safety suite:

| Case | Setup | Expected |
|---|---|---|
| Huge file | Add generated 5 MB JS file. | Skip or cap with warning; no hang. |
| Syntax error | Break one Rust/TS/Python file. | Partial parse when possible; graph survives. |
| Unsupported extension | Add `.weird` file with symbols. | Unsupported warning and maybe ctags/text fallback. |
| Embedded language | HTML with script/style. | Parent file parsed; embedded region handled or warning emitted. |
| Timeout | Pathological nested file. | Timed-out parse artifact; MCP survives. |
| Cancellation | Cancel full index mid-run. | Job cancelled, no fake parse failures cached. |

The grade is not "parse everything."

The grade is:

```text
fail honestly and keep the agent useful
```

### Review-Context Eval

This is the most product-relevant eval.

Case:

```text
1. Make a small code diff.
2. Ask tool for review context.
3. Check whether it returns:
   - changed symbols
   - direct callers
   - direct callees
   - public API impact
   - likely tests
   - stale/parse warnings
   - next files to read
```

Scoring:

| Item | Points |
|---|---:|
| Changed symbols correct | 15 |
| Impacted files correct | 20 |
| Tests correct | 15 |
| Risks explained | 15 |
| Token budget respected | 15 |
| Next actions useful | 10 |
| Warnings honest | 10 |

This is the habit loop:

```text
Before Codex finalizes an edit, ask the graph what was affected.
```

### Route Trace Eval

CRUD route trace case:

```text
Prompt:
  "What code path handles PATCH /users/:id and where does it write?"
```

Expected answer:

```text
route definition
auth middleware if relevant
controller/handler
service method
repository/model
DB table/query or external API
test files
confidence by edge
next reads
```

Scoring should reward:

```text
route-to-handler correctness
handler-to-service correctness
service-to-repository correctness
test recommendation
avoiding unrelated routes
```

This eval is essential because CRUD/API apps are one of the user's core domains.

### Systems Trace Eval

Rust/C/C++ trace case:

```text
Prompt:
  "If I change parse_config, what callers and exported APIs are affected?"
```

Expected answer:

```text
definition
declaration/header if applicable
module owner
direct callers
public/exported boundary
tests/benches/examples
unresolved macro/dispatch warnings
```

Scoring should reward:

```text
not pretending C++ macro/template resolution is exact
showing confidence
finding header/source pairs
finding Rust tests or benches
staying within token budget
```

This eval is essential because systems programming is the other core user domain.

### Anti-Cheating Rules

The harness should prevent tools from winning by brute force.

Rules:

```text
1. Returned text above budget is penalized.
2. Raw full-file dumps count against token budget.
3. Hidden truncation is penalized.
4. Missing freshness warning is an automatic fail for stale cases.
5. Crashes/hangs are automatic fails.
6. Tools must disclose unsupported languages or skipped files.
7. Tools must not claim exactness for inferred relationships.
8. Manual human search outside allowed tools is disallowed during benchmark.
```

The point is to simulate Codex's real constraint:

```text
limited context, limited calls, limited trust.
```

### Minimal CLI For The Harness

Eventually:

```bash
parceltongue-bench list
parceltongue-bench run --case impact_001 --tool parceltongue-v2
parceltongue-bench run --suite core --tool codebase-memory-mcp
parceltongue-bench grade --run runs/2026-07-06-parceltongue-v2
parceltongue-bench report --compare parceltongue-v2 codebase-memory-mcp rg
```

Tool adapters for the bench:

```text
rg_baseline
parseltongue_v1_http
parceltongue_v2_mcp
codebase_memory_mcp
sdsrss_code_graph_mcp
code_review_graph_mcp
cocoindex_code
clarity_cli
semgrep_mcp
ast_grep_cli
comby_cli
```

Start with manual adapter scripts if necessary.

Do not overbuild the bench harness before the first useful report.

### The First Useful Report

The first benchmark report should answer:

```text
Should I evolve Parceltongue or just use a stack of existing tools inside Codex?
```

That report should include:

```text
top 5 failures of current tool stack
top 5 things current tools already solve
where Parceltongue v1 fails
where Parceltongue v2 would need to be better
minimum viable v2 scope
do-not-build list
```

This is the decision report the user actually needs.

### Shreyas Doshi Read

The evaluation harness is a product management tool.

It makes this question concrete:

```text
What job are users hiring Parceltongue to do?
```

For this user, the job is:

```text
Help my Codex agent navigate large codebases faster and more reliably.
```

So the benchmark must measure:

```text
navigate
large codebases
faster
reliably
```

If a feature improves parser elegance but does not improve those four words, it
is not MVP.

### Concept 10 Conclusion

Parceltongue v2 should not proceed on faith.

It should proceed through an eval harness that measures:

```text
correct files
correct symbols
correct edges
correct tests
freshness honesty
uncertainty honesty
token efficiency
tool-call efficiency
next-action usefulness
```

The headline metric:

```text
Decision Quality Per Token
```

The goal:

```text
Codex reaches the right next read/edit/test decision faster, with less context
waste and fewer silent graph lies.
```

That is how to know whether Parceltongue deserves to evolve.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| `docs/research002/J003.md` | Source-read | PMF rubric and current external tool stack for Codex benchmarking. |
| `README.md` | Source-read | v1.7.2 workflow and endpoint baseline to compare against. |
| `docs/research001/unclassified/RCA-Incremental-Indexing-Failure.md` | Source-read | Freshness and identity failure cases that need permanent eval coverage. |
| `docs/research001/unclassified/ISGL1-v2-Stable-Entity-Identity.md` | Source-read | Stable identity regression requirements. |
| Concepts 1-9 in this document | Source-read through current synthesis | Provides the parser, graph, freshness, API, safety, adapter, and roadmap dimensions the eval must measure. |

## Concept 11: Decide Whether To Evolve Parceltongue Or Use A Tool Stack

### The Decision

This concept is the decision memo that follows naturally from Concept 10.

The uncomfortable question is:

```text
Do we actually need to evolve Parceltongue, or should Codex simply use the
best existing graph/search tools?
```

The answer should not be based on identity, sunk cost, annoyance, or excitement.

It should be based on the job.

For this user, the job is not:

```text
Build a public code intelligence product.
```

It is not:

```text
Create a new autonomous coding agent.
```

It is not:

```text
Compete with Cursor, Aider, Claude Code, Continue, or Codex.
```

The actual job is:

```text
Help Codex, as the chosen coding agent, navigate large local codebases faster
and more reliably with dependency clarity.
```

That sentence changes the whole decision.

The product is not an agent.

The product is the local context layer that Codex can call.

Codex remains the operator.

The context layer answers:

```text
What should Codex inspect next?
What calls this?
What does this call?
What files are in the blast radius?
What route or feature flow does this belong to?
What tests matter?
What evidence is stale?
What graph edge is uncertain?
What should not be edited blindly?
```

If existing tools answer those questions well enough, Parceltongue should not
be rebuilt for glory.

If existing tools answer those questions only when stitched together with
manual effort, then build a broker.

If existing tools cannot satisfy freshness, identity, dependency graph, and
token efficiency together, then evolve Parceltongue.

That is the decision tree.

### The Shreyas Doshi Lens

A Shreyas Doshi style read would separate problem love from solution love.

The problem is real:

```text
Agents get lost in large repos.
Agents waste tokens on irrelevant files.
Agents over-read instead of following dependency structure.
Agents under-read and miss callers, routes, tests, and side effects.
Agents believe stale context.
Agents struggle across language boundaries.
```

The solution is not automatically:

```text
Build Parseltongue v2.
```

The solution might be:

```text
Install the right local tools and teach Codex a strict workflow.
```

Or:

```text
Build a tiny adapter that makes those tools feel like one coherent context
graph service.
```

Or:

```text
Build Parceltongue v2 because no current tool gives the exact graph facts,
freshness semantics, and token-efficient response contract needed by Codex.
```

The decision should be made by observing a repeated workflow, not by admiring
an architecture.

The repeated workflow is:

```text
Before Codex edits, Codex asks a local tool what code matters.
After Codex edits, Codex asks what changed and what tests or flows are at risk.
During investigation, Codex asks for the next dependency edge to follow.
```

If that ritual becomes daily and existing tools fail it, Parceltongue has PMF
for this one-person environment.

If the ritual is occasional, or solved by two shell commands, do not overbuild.

### The User Constraint That Matters Most

The user already made the most important product decision:

```text
I am going to use the Codex app.
```

That means the question is not:

```text
Which coding agent should I use?
```

The question is:

```text
Which tools can Codex use?
```

Everything else follows from that.

So the comparison should not be:

```text
Parceltongue vs Aider vs Cursor vs Claude Code.
```

The comparison should be:

```text
Codex alone
Codex plus rg/git/shell
Codex plus one graph MCP
Codex plus one graph MCP and one search tool
Codex plus a thin broker over several tools
Codex plus evolved Parceltongue
```

That framing keeps the work honest.

Codex is the fixed surface.

The variable is the context substrate.

### Personal PMF Definition

The user profile is specific enough that generic market PMF is the wrong lens.

The actual PMF is personal PMF:

```text
Solo agent power user.
Uses Codex app.
Works across all languages.
Works on CRUD apps and Rust/C/C++ systems programming.
Wants private local help, not a commercial product.
Wants faster and more reliable large-codebase navigation.
Wants dependency clarity and graph-backed context.
```

This means the tool must optimize for:

```text
low setup friction
local-first operation
offline usefulness
shell/MCP compatibility
multi-language tolerance
dependency and call graph answers
route and feature-flow tracing
minimal-token context packs
freshness honesty
Codex-friendly response shape
```

It does not need to optimize first for:

```text
multi-tenant SaaS
beautiful visual dashboards
team permissions
enterprise search portals
IDE lock-in
public onboarding funnels
sales-led adoption
```

This matters because many attractive tools are optimized for a different buyer.

For this user, the buyer, user, operator, maintainer, and evaluator are the same
person.

That makes sharp local workflow more valuable than broad platform polish.

### Three Possible Decisions

There are only three rational paths.

```text
Path A: Use an existing Codex tool stack.
Path B: Build a thin broker over existing tools.
Path C: Evolve Parceltongue into the core graph service.
```

Everything else is a variant of these.

### Path A: Use An Existing Codex Tool Stack

This is the default until proven otherwise.

It means:

```text
Do not evolve Parceltongue yet.
Install and benchmark the best available local tools.
Teach Codex a strict pre-edit and post-edit ritual.
Use the tools through MCP or shell.
Only build when measured gaps remain.
```

The likely first stack from `J003.md` is:

| Role | Candidate | Why It Is In The First Stack |
|---|---|---|
| Primary graph brain | `GitNexus` or `codebase-memory-mcp` | Best fit for agent asks graph, graph responds with compact context. |
| Review/change safety | `code-review-graph` or `better-code-review-graph` | Strongest workflow around impact, review context, and pre-merge risk. |
| CRUD route graph challenger | `sdsrss/code-graph-mcp` or `Tessera` | Endpoint vocabulary matches route tracing, project map, impact, tests. |
| Search/discovery | `cocoindex-code` | Best low-friction semantic/AST search layer in the current notes. |
| CLI structural sanity | `Clarity` | Useful shell tool for modules, reachability, cycles, and before/after structure checks. |
| Structural edit/search | `ast-grep`, `Semgrep`, `Comby` | Excellent for syntax-aware finding, linting, and transformations. |

This path is right if:

```text
One graph tool answers most dependency questions.
One search tool answers most discovery questions.
Codex can call both without confusion.
The answers fit in small token budgets.
The tools are fresh enough after edits.
The setup does not become its own project.
```

This path is especially attractive because the user is not trying to sell a
tool.

If the purpose is personal leverage, the fastest way to value wins.

### Path A Daily Workflow

The practical workflow would look like this:

```text
1. Codex starts with normal shell orientation:
   git status
   rg
   rg --files
   package/cargo/build metadata

2. Codex asks the graph brain:
   project map
   symbol context
   callers
   callees
   impact radius

3. Codex asks the search layer when names are unknown:
   semantic search
   AST chunk search
   keyword search

4. Codex asks the review graph after a diff exists:
   changed symbols
   impacted files
   risky edges
   relevant tests

5. Codex runs Clarity or similar for module-level sanity:
   reach up
   reach down
   cycles
   between

6. Codex edits only after the graph has narrowed the next step.

7. Codex reruns tests and re-asks impact after edits.
```

The user should feel:

```text
Codex is less confused.
Codex reads fewer random files.
Codex finds the right edge faster.
Codex catches missing callers before tests fail.
Codex explains why a file matters.
```

If that happens, Path A is enough.

### Path A Failure Modes

Path A fails when the stack becomes tool soup.

Symptoms:

```text
Codex does not know which tool to call.
Different tools disagree silently.
Each tool returns a different response shape.
The graph tool misses a language or framework.
The search tool finds files but not dependency meaning.
The review tool is good only after a diff, not before an edit.
The graph is stale after Codex edits.
The output is verbose enough to defeat the point.
The user starts manually translating between tools.
```

If those happen often, do not jump straight to full Parceltongue.

Build a broker first.

### Path B: Build A Thin Broker

The thin broker is the most likely sweet spot.

It is not a new code intelligence engine.

It is a local Codex-facing service that normalizes existing tools.

It answers with one stable contract:

```json
{
  "answer": "short decision-ready answer",
  "next_actions": ["read file A", "inspect caller B", "run test C"],
  "evidence": [
    {
      "file": "src/routes/users.ts",
      "line": 42,
      "symbol": "createUserRoute",
      "reason": "direct caller"
    }
  ],
  "edges": [
    {
      "from": "createUserRoute",
      "to": "createUserService",
      "kind": "calls",
      "confidence": 0.92,
      "source_tool": "code-graph-mcp"
    }
  ],
  "freshness": {
    "status": "fresh",
    "indexed_at": "2026-07-06T16:45:00Z",
    "dirty_files": []
  },
  "token_budget": {
    "estimated_tokens": 930,
    "budget": 2000
  },
  "uncertainty": [
    "No runtime route registration was proven."
  ]
}
```

The broker hides:

```text
which tool answered
which CLI syntax was needed
which MCP endpoint was awkward
which output format was noisy
which tool should be tried first
which fallback should run if the first tool fails
```

The broker exposes:

```text
project_map
search_code
symbol_context
callers
callees
impact_radius
route_trace
review_context
relevant_tests
index_health
tool_disagreement
```

This is a product surface.

The implementation can be humble.

It can shell out.

It can call MCP tools.

It can run `rg`, `git`, `ast-grep`, `semgrep`, and Clarity.

It can call one graph brain first and another as fallback.

The broker is valuable if it makes Codex ask better questions without learning
every tool.

### Path B Is A Translation Layer

The broker translates from:

```text
Codex intent
```

to:

```text
tool calls
```

and back into:

```text
decision-ready context
```

Example:

```text
Codex intent:
I need to modify authentication timeout behavior.

Broker plan:
1. Search for timeout/auth symbols.
2. Ask graph brain for callers/callees of best match.
3. Ask route graph for HTTP entry points.
4. Ask review tool for tests or impacted files if diff exists.
5. Return a ranked read plan.

Broker answer:
Read these 5 files in this order, because they cover route entry, service
logic, config source, downstream token behavior, and tests.
```

That is more useful than exposing 30 raw endpoints.

### Why A Broker May Beat Full Parceltongue

A broker can be shipped quickly.

It can learn from existing tools instead of replacing them.

It keeps the user close to practical value.

It avoids rebuilding:

```text
semantic search
multi-language parser installation
advanced static analysis
security rule engines
graph database storage
visualization
LSP-grade symbol resolution
code rewrite engines
```

It focuses on the Codex loop:

```text
question
tool selection
evidence merge
compact answer
next action
```

That loop is where personal PMF lives.

The broker is also reversible.

If the broker proves that one external tool wins everything, use that tool.

If the broker proves that no tool has stable identity and freshness, build
Parceltongue core underneath it.

The broker is both useful product and measurement instrument.

### Path B Failure Modes

The broker fails if it becomes a fake product without a real graph.

Symptoms:

```text
It mostly wraps `rg`.
It cannot explain callers/callees.
It cannot detect stale indexes.
It cannot merge contradictory tool output.
It returns summaries without line evidence.
It creates another layer of hallucination.
It has no benchmark harness.
It becomes harder to debug than the tools underneath.
```

The broker must stay brutally concrete.

Every answer should be traceable to:

```text
file
line
symbol
edge
tool
timestamp
confidence
```

If it cannot provide that, it is worse than a shell command.

### Path C: Evolve Parceltongue

Parceltongue should evolve only if the benchmark shows a real gap.

The gap is not:

```text
Other tools are not written exactly how I would write them.
```

The gap is:

```text
No existing Codex-callable stack gives reliable, fresh, compact dependency
context for large mixed-language codebases.
```

The v2 trigger should be strict.

Build Parceltongue v2 if three or more of these are true:

| Trigger | Why It Justifies Building |
|---|---|
| Existing tools fail stable symbol identity under line shifts | Codex cannot trust graph answers after edits. |
| Existing tools return too many tokens for routine questions | The tool saves search time but burns context budget. |
| Existing tools cannot answer callers/callees with provenance | Codex still has to inspect manually. |
| Existing tools cannot track freshness after local edits | Stale graph facts are worse than no graph facts. |
| Existing tools do not handle the user's mixed language set | Solo workflow needs one dependable habit across repos. |
| Existing tools cannot produce decision-ready next actions | They are databases, not agent decision loops. |
| Existing tools disagree and no layer can reconcile them | Codex needs confidence, not parallel confusion. |
| Existing tools are too fragile to use daily | Personal PMF dies on friction. |

If the triggers are present, Parceltongue v2 should not be a generic rewrite.

It should be the core graph engine underneath a Codex-facing service.

### What Parceltongue v2 Should Own

If built, Parceltongue should own only the parts where ownership creates a real
advantage.

It should own:

```text
stable entity identity
incremental freshness model
dependency edge schema
provenance-rich graph store
query packs for supported languages
token-budgeted context selection
Codex-friendly answer envelopes
benchmark harness
```

It should not try to own:

```text
every static analysis rule
every structural rewrite pattern
every semantic embedding strategy
every security rule
every visual graph product
every IDE integration
every language at full fidelity on day one
```

The temptation is to make Parceltongue a cathedral.

The better version is a sharp local instrument.

### The Honest Recommendation

The best answer right now is:

```text
Do not jump directly to a full Parceltongue v2 rewrite.
```

The next best move is:

```text
Use the existing Codex-compatible tool stack and benchmark it.
```

The likely medium-term move is:

```text
Build a thin broker that normalizes the best tools into one Codex-facing
context graph contract.
```

The later move, only if the broker and benchmark prove it, is:

```text
Evolve Parceltongue as the core graph engine behind the broker.
```

This sequence reduces regret.

It lets the user get value immediately.

It preserves Parceltongue's unique idea.

It avoids overbuilding before evidence.

### Decision Matrix

Use this table after running the Concept 10 benchmark.

| Benchmark Result | Decision | Rationale |
|---|---|---|
| Existing stack scores 85+ DQPT and setup is tolerable | Use tool stack | Personal PMF is already satisfied. |
| Existing stack scores 75-84 DQPT but Codex struggles to choose tools | Build broker | The missing product is orchestration and response normalization. |
| Existing stack scores 65-74 DQPT because graph facts are partial | Broker plus targeted Parceltongue modules | Build only the missing graph/freshness parts. |
| Existing stack scores below 65 DQPT on large mixed repos | Evolve Parceltongue | Existing tools do not solve the core job. |
| Stack is accurate but verbose | Broker | Compress and rank context before Codex sees it. |
| Stack is fast but stale | Parceltongue freshness layer | Speed without freshness is dangerous. |
| Stack finds code but cannot explain dependencies | Add graph brain or Parceltongue graph core | Search alone is not the job. |
| Stack graphs dependencies but cannot find fuzzy concepts | Pair with search layer | Graph alone is not first-contact discovery. |
| Stack works only for CRUD apps | Add systems-language benchmark before deciding | User also needs Rust/C/C++ work. |

### The Right First Experiment

The first experiment should not be:

```text
Rewrite the parser.
```

It should be:

```text
Install the top stack, run the benchmark, and compare against Codex plus rg.
```

The experiment should include:

```text
Codex plus rg/git only
Codex plus GitNexus
Codex plus codebase-memory-mcp
Codex plus code-review-graph
Codex plus sdsrss/code-graph-mcp
Codex plus cocoindex-code
Codex plus Clarity
Codex plus a manual combination of best tools
```

The score should capture:

```text
correctness
token count
tool-call count
setup friction
freshness after edit
identity stability after line shift
CRUD route tracing
systems call tracing
review-context usefulness
next-action usefulness
```

If the manual combination wins, that is evidence for the broker.

If one tool wins, use it.

If all tools fail a core category, that is evidence for Parceltongue v2.

### Tool Stack Hypotheses

Before running the benchmark, the likely hypotheses are:

| Tool | Hypothesis |
|---|---|
| `GitNexus` | Best broad product-shaped graph/context tool if CRUD/API workflows dominate. |
| `codebase-memory-mcp` | Best broad agent memory/graph candidate if large mixed-language repos dominate. |
| `code-review-graph` | Best pre-merge and after-diff workflow, especially for change safety. |
| `sdsrss/code-graph-mcp` | Best endpoint vocabulary and route-trace reference for CRUD apps. |
| `Tessera` | Promising deterministic graph workflow if the CLI/MCP journey is stable. |
| `cocoindex-code` | Best search/discovery layer, not a full dependency brain. |
| `codemogger` | Clean local Turso-backed search layer, useful but not sufficient alone. |
| `fff` | Fast find/search primitive, not enough for graph PMF. |
| `Clarity` | Great CLI graph companion for module/reach/cycle checks. |
| `ast-grep` | Excellent syntax-aware search and rewrite primitive. |
| `Semgrep` | Strong rule and security layer, not everyday graph navigation by itself. |
| `Comby` | Useful structural rewrite/search tool, not a dependency model. |
| `CodeQL` | Powerful static-analysis prior art, too heavy as the first personal Codex context layer. |
| `Glean`, `Kythe`, `SCIP`, `stack-graphs` | Schema and architecture teachers more than daily solo tools. |

This table is not a final ranking.

It is the set of hypotheses the benchmark should prove or break.

### What The Broker Would Actually Do

The broker should be small enough to explain in one screen.

It should expose a tiny command or MCP surface:

```text
context.project_map
context.search
context.symbol
context.impact
context.route
context.review
context.tests
context.health
```

Each endpoint should answer in the same shape:

```text
answer
evidence
edges
recommended_next_reads
recommended_next_tests
freshness
confidence
tool_trace
token_estimate
```

The broker should call tools behind the scenes:

```text
rg for lexical confirmation
git for changed files
cocoindex-code or codemogger for fuzzy search
codebase-memory-mcp or GitNexus for graph answers
code-review-graph for diff-aware review context
Clarity for module-level reachability and cycles
ast-grep or Semgrep for syntax/rule confirmation
```

The broker should avoid pretending all tools are equal.

It should rank tools by job:

```text
unknown concept -> search first
known symbol -> graph first
changed diff -> review first
module architecture -> Clarity first
security rule -> Semgrep first
structural rewrite -> ast-grep or Comby first
freshness doubt -> index_health first
```

That ranking is the product.

### Broker Versus Parceltongue

The distinction is simple.

The broker answers:

```text
Which existing tool should Codex ask, and how should the answer be compressed?
```

Parceltongue core answers:

```text
What is the true local dependency graph, and how fresh are its facts?
```

They are different products.

The broker can exist without Parceltongue.

Parceltongue can exist under the broker.

The mistake is to mix them too early.

Build the broker if the problem is:

```text
tool choice
tool orchestration
answer compression
Codex ritual
fallback routing
```

Build Parceltongue if the problem is:

```text
the graph itself
stable identity
incremental indexing
dependency edge fidelity
language query packs
freshness semantics
```

### The Minimum Viable Broker

The minimum viable broker can be intentionally boring.

It needs:

```text
1. A config file listing available tools.
2. A health command showing which tools work.
3. A search command that combines lexical and semantic results.
4. A symbol command that returns callers, callees, file spans, and confidence.
5. An impact command that ranks affected files and tests.
6. A review command that consumes `git diff`.
7. A freshness command that reports dirty files and index age.
8. A consistent JSON response envelope.
9. A benchmark command that runs Concept 10 tasks.
```

It does not need:

```text
new parsers
new graph database
new UI
new embeddings
new web dashboard
new multi-agent orchestration
```

The first broker can literally be a local CLI that Codex calls from the shell.

MCP can come after the response contract is useful.

### The Minimum Viable Parceltongue Evolution

If Parceltongue evolves, the minimum viable evolution is not all of v2.

It is:

```text
1. Stable entity identity.
2. Fresh incremental indexing.
3. Parser safety and lifecycle controls.
4. Provenance-rich dependency edges.
5. Query packs for the top personal languages.
6. Agent response envelope.
7. Smart context selection.
8. Benchmark harness.
```

The first supported language set should follow the user's real work:

```text
TypeScript/JavaScript for CRUD apps.
Python if present in work repos.
Rust for systems and CLI work.
C/C++ for systems programming.
```

Language support should be honest:

```text
supported
partial
syntax-only
unsupported
```

Codex should never have to guess whether a graph answer is authoritative.

### What Not To Build Yet

Do not build a visual graph UI yet.

Do not build a landing page.

Do not build a commercial onboarding flow.

Do not build a generic code search engine.

Do not rebuild Semgrep.

Do not rebuild CodeQL.

Do not rebuild Kythe.

Do not rebuild Glean.

Do not build a new coding agent.

Do not expose 40 endpoints just because v1.7.2 had a rich HTTP surface.

Do not make Codex choose from a huge menu.

The user needs:

```text
small number of high-signal questions
compact answers
evidence
next actions
freshness
```

Everything else can wait.

### Practical Codex Ritual

Whatever path wins, Codex should follow a ritual.

Before editing:

```text
1. Check repo state.
2. Identify task scope.
3. Ask context layer for project/symbol/impact context.
4. Read only the ranked files first.
5. Confirm suspected edges with raw code.
6. Edit.
```

After editing:

```text
1. Ask context layer what changed.
2. Ask for impacted callers/routes/tests.
3. Run the smallest relevant tests.
4. Run broader tests if impact is wide.
5. Ask for stale index health.
6. Summarize evidence in final response.
```

During debugging:

```text
1. Start with the failing symptom.
2. Ask for route/call path to symptom.
3. Follow one edge at a time.
4. Stop when evidence explains the failure.
5. Patch the smallest responsible point.
```

This ritual matters more than the tool brand.

The tool is hired to make the ritual reliable.

### The Decision In One Sentence

The strongest current recommendation is:

```text
Use the best existing Codex-callable graph/search stack first, build a thin
broker if tool sprawl hurts, and evolve Parceltongue only when benchmarked
gaps prove that the graph/freshness core itself is missing.
```

This is not anti-Parceltongue.

It is pro-Parceltongue discipline.

It protects the useful idea from becoming a rewrite reflex.

### PMF Score By Path

For the user's personal PMF, the paths score like this before benchmarking:

| Path | Estimated Personal PMF | Why |
|---|---:|---|
| Codex plus rg/git only | 55 | Always available, but too much manual exploration for large repos. |
| Codex plus one strong graph MCP | 75 | Likely immediate improvement, but may miss search/freshness/review gaps. |
| Codex plus graph MCP plus search tool plus Clarity | 82 | Strong practical setup if Codex can keep the workflow simple. |
| Thin broker over best tools | 88 | Best balance of immediate leverage and coherent Codex interface. |
| Full Parceltongue v2 now | 70 | Potentially highest ceiling, but premature before benchmark evidence. |
| Parceltongue v2 after broker/eval proves gaps | 92 | Highest if stable identity, freshness, and graph core are proven missing. |

The highest expected path is staged:

```text
tool stack -> broker -> Parceltongue core only if needed
```

### How This Changes J003

`J003.md` ranks tools as candidates.

Concept 11 changes the mental model from:

```text
Which repo is best?
```

to:

```text
Which role does each repo play in the Codex context loop?
```

The roles are:

| Role | Tools |
|---|---|
| Graph brain | `GitNexus`, `codebase-memory-mcp`, `code-graph-mcp`, `Tessera`, `CodeGraphContext` |
| Review graph | `code-review-graph`, `better-code-review-graph` |
| Search layer | `cocoindex-code`, `codemogger`, `Probe`, `chunkhound`, `fff` |
| Module graph CLI | `Clarity` |
| Structural search/rewrite | `ast-grep`, `Comby` |
| Rule/security analysis | `Semgrep`, `CodeQL` |
| Architecture prior art | `Glean`, `Kythe`, `SCIP`, `stack-graphs` |

The winning setup may include one tool from several roles.

That does not mean tool sprawl is good.

It means the broker, if built, should encode role-based routing.

### What Would Make Me Say Build Parceltongue

I would say build Parceltongue if the benchmark shows this pattern:

```text
Codex plus existing tools still wastes time because graph answers are stale,
symbol identity breaks after edits, route/call edges are not trustworthy, and
context packs are either too huge or too vague.
```

I would also say build it if the user repeatedly says:

```text
I know a graph should answer this, but every existing tool makes me manually
triangulate it.
```

That repeated sentence is a product signal.

One-off annoyance is not enough.

Daily friction is enough.

### What Would Make Me Say Do Not Build

I would say do not build Parceltongue if:

```text
GitNexus or codebase-memory-mcp answers most graph questions.
code-review-graph handles diff review well.
cocoindex-code solves first-contact search.
Clarity handles module reach/cycles.
Codex can learn the ritual.
The stack is fresh enough after edits.
The answers fit inside token budgets.
```

In that world, building Parceltongue would be mostly expression, not leverage.

Expression is allowed.

But it should be named honestly.

If the goal is leverage, use the stack.

### What Would Make Me Say Build The Broker

I would say build the broker if:

```text
The tools are good individually but annoying collectively.
Codex keeps asking the wrong one first.
The response formats vary too much.
The user wants one command for context.
Tool disagreement needs to be surfaced.
Freshness needs one common health check.
Benchmark runs need one harness.
```

This is the most likely outcome.

It fits the user's actual environment:

```text
Codex app
local repos
many languages
private workflow
large-codebase navigation
agent-oriented context
```

The broker is the smallest product that makes that environment feel coherent.

### Concept 11 Conclusion

The decision is staged, not binary.

First:

```text
Run Codex with the strongest existing tool stack.
```

Second:

```text
Measure agent decision quality per token.
```

Third:

```text
If tools are strong but fragmented, build a broker.
```

Fourth:

```text
If tools cannot provide fresh, stable, provenance-rich dependency context,
evolve Parceltongue underneath the broker.
```

The strategic answer is:

```text
Do not build a mega coding agent.
Use Codex.
Build or choose the best context graph layer around Codex.
```

That keeps the work aligned with the user's real job.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| `docs/research002/J003.md` | Source-read | Supplies PMF definition, shortlist, current Codex arsenal, and tool roles. |
| `README.md` | Source-read | Shows Parceltongue v1.7.2 promise: entity list, callers, callees, blast radius, smart context. |
| `docs/research001/PRD-research-20260131v1/PARSELTONGUE_V2_LLM_AGENT_INTERACTIONS.md` | Source-read | Provides the iterative LLM query loop and graph-guided agent workflow. |
| `docs/research001/PRD-research-20260131v1/PARSELTONGUE_V2_BIDIRECTIONAL_LLM_ENHANCEMENT.md` | Source-read | Supports the idea of LLM-guided CPU graph computation and feedback loops. |
| `docs/research001/unclassified/RCA-Incremental-Indexing-Failure.md` | Source-read | Defines freshness and stable identity as hard reasons to build, not optional polish. |
| `docs/research001/unclassified/ISGL1-v2-Stable-Entity-Identity.md` | Source-read | Defines the identity/freshness gap that would justify Parceltongue core evolution. |

## Concept 12: Build A Thin Broker As A Codex Context Router

### The Point

The thin broker is the lowest-regret next build.

It does not replace the best existing tools.

It makes them usable by Codex through one predictable surface.

The broker's job is:

```text
Take a Codex intent.
Choose the right local tools.
Merge and compress their evidence.
Return a small decision-ready context packet.
```

It should feel less like:

```text
another code intelligence platform
```

and more like:

```text
a local air traffic controller for context tools
```

Codex should not need to remember whether this question belongs to Clarity,
codebase-memory-mcp, code-review-graph, cocoindex-code, Semgrep, ast-grep, or
plain `rg`.

Codex should ask:

```text
What context do I need before acting?
```

The broker should decide:

```text
Which tool or tools can answer that with the least waste?
```

### The Broker Is Not Parceltongue v2

This distinction is important.

The broker is not a parser.

The broker is not a graph database.

The broker is not a semantic search engine.

The broker is not a static analyzer.

The broker is not a UI.

The broker is a router and normalizer.

It owns:

```text
tool discovery
tool health
intent routing
response normalization
evidence merging
freshness reporting
token-budget compression
benchmark execution
Codex workflow enforcement
```

It does not own:

```text
Tree-sitter grammar correctness
symbol resolution from first principles
static analysis rules
semantic embeddings
full dependency graph construction
large-scale graph storage
visual graph exploration
```

That division keeps the first implementation small.

If Parceltongue later becomes the best graph core, the broker can call it.

If another graph tool wins, the broker can call that.

The broker protects the user from premature commitment.

### MVP Name

The working name could be:

```text
ptctx
```

Short for:

```text
Parceltongue Context
```

The name is not important.

The shape is important.

The CLI should be short enough for Codex to call naturally:

```bash
ptctx health
ptctx project-map --budget 2000
ptctx search --query "auth timeout" --budget 1200
ptctx symbol --query "createUser" --budget 2000
ptctx impact --symbol "createUserService" --depth 2 --budget 3000
ptctx route --query "POST /users" --budget 3000
ptctx review --diff --budget 4000
ptctx tests --for "src/users/service.ts" --budget 1200
ptctx bench run --suite smoke
```

The broker can later expose MCP.

The first version should be a CLI because Codex can already use the shell well.

### Why CLI First

CLI first is the right move because:

```text
Codex already knows how to call shell commands.
Shell commands are easy to debug.
The user can run the same command outside Codex.
No MCP config is needed for first value.
No server lifecycle is needed.
No tool schema token budget is spent up front.
Logs are easy to inspect.
```

MCP is still useful later.

But MCP should expose a stable broker contract after the CLI proves useful.

The first value should come from:

```text
one command
one JSON envelope
small context
line evidence
clear next action
```

### Core Command Surface

The MVP should have eight commands.

| Command | Job | Typical Codex Moment |
|---|---|---|
| `health` | Check available tools, indexes, freshness, repo state | Start of work or suspicion of stale context. |
| `project-map` | Summarize repo structure and important modules | First orientation in a large repo. |
| `search` | Find likely symbols/files when the name is unknown | User describes concept, route, feature, or behavior. |
| `symbol` | Resolve a known or fuzzy symbol to exact spans and edges | Codex has a candidate name. |
| `impact` | Find callers, callees, dependents, routes, tests, and risk | Before editing or refactoring. |
| `route` | Trace CRUD/API path from route to handler to service to storage | Web/backend investigation. |
| `review` | Explain current diff and affected graph/test surface | After edit, before final response. |
| `tests` | Suggest relevant tests for files/symbols/diff | Before running expensive suites. |

This is enough.

More commands can exist later, but Codex should learn this small set first.

### Command 1: `health`

`health` answers:

```text
Can the broker answer graph/search/review questions right now?
```

Example:

```bash
ptctx health --json
```

Response:

```json
{
  "status": "degraded",
  "repo": {
    "root": "/repo",
    "branch": "main",
    "dirty_files": ["src/users/service.ts"],
    "untracked_files": []
  },
  "tools": [
    {
      "name": "rg",
      "status": "available",
      "version": "14.1.1",
      "capabilities": ["lexical_search"]
    },
    {
      "name": "clarity",
      "status": "available",
      "version": "0.9.0",
      "capabilities": ["module_graph", "cycles", "reachability"]
    },
    {
      "name": "cocoindex-code",
      "status": "missing",
      "capabilities": ["semantic_search"]
    }
  ],
  "indexes": [
    {
      "tool": "codebase-memory-mcp",
      "status": "stale",
      "indexed_at": "2026-07-06T12:00:00Z",
      "dirty_files_not_indexed": ["src/users/service.ts"]
    }
  ],
  "recommendation": "Run graph refresh before trusting impact results."
}
```

The important part is not the exact JSON.

The important part is that Codex sees freshness before using graph facts.

### Command 2: `project-map`

`project-map` answers:

```text
What kind of repo is this, and what are the likely important areas?
```

It should combine:

```text
filesystem layout
package metadata
Cargo/workspace metadata
git tracked files
Clarity module graph if available
graph MCP project map if available
README and docs entry points
test directories
generated/vendor ignores
```

It should not dump the tree.

It should return a map that helps Codex choose first reads.

Example:

```bash
ptctx project-map --budget 2000
```

Answer shape:

```json
{
  "answer": "This appears to be a TypeScript API service with Express routes, service modules, database adapters, and Jest integration tests.",
  "top_modules": [
    {
      "path": "src/routes",
      "role": "HTTP route entry points",
      "evidence": ["package.json declares express", "18 files contain router."]
    },
    {
      "path": "src/services",
      "role": "business logic",
      "evidence": ["Most route handlers call into this folder."]
    }
  ],
  "recommended_next_reads": [
    "README.md",
    "src/routes/index.ts",
    "src/services/userService.ts",
    "tests/users/createUser.test.ts"
  ],
  "freshness": {
    "status": "fresh"
  },
  "token_budget": {
    "estimated_tokens": 820,
    "budget": 2000
  }
}
```

This command is for orientation, not deep analysis.

### Command 3: `search`

`search` answers:

```text
Where is the concept likely implemented?
```

It is used when Codex has words but not exact symbols.

Example:

```bash
ptctx search --query "auth token expiry" --budget 1200
```

Adapter order:

```text
1. lexical search with rg
2. semantic/AST chunk search with cocoindex-code or codemogger
3. structural search with ast-grep if a pattern is supplied
4. graph search if a graph tool supports fuzzy query
```

The result should rank files by usefulness, not by raw match count.

Example result:

```json
{
  "answer": "The best starting point is src/auth/tokenService.ts; it defines token TTL and is called by login and refresh flows.",
  "results": [
    {
      "file": "src/auth/tokenService.ts",
      "span": "18-64",
      "symbols": ["createAccessToken", "TOKEN_TTL_SECONDS"],
      "score": 0.94,
      "reason": "Exact token expiry constants plus service functions.",
      "source_tools": ["rg", "cocoindex-code"]
    },
    {
      "file": "src/routes/auth.ts",
      "span": "42-88",
      "symbols": ["loginRoute", "refreshRoute"],
      "score": 0.81,
      "reason": "Route entry points call token service.",
      "source_tools": ["code-graph-mcp"]
    }
  ],
  "recommended_next_reads": [
    "src/auth/tokenService.ts",
    "src/routes/auth.ts"
  ]
}
```

The key phrase is:

```text
best starting point
```

Codex needs starting points, not search dumps.

### Command 4: `symbol`

`symbol` answers:

```text
What exactly is this symbol, where is it, and what graph edges are known?
```

Example:

```bash
ptctx symbol --query "createAccessToken" --budget 2000
```

It should return:

```text
definition span
signature
doc/comment summary if nearby
direct callers
direct callees
imports/exports if relevant
confidence
ambiguity if multiple matches
```

Example result:

```json
{
  "answer": "createAccessToken is defined in src/auth/tokenService.ts and is called by loginRoute and refreshSession.",
  "symbol": {
    "name": "createAccessToken",
    "kind": "function",
    "file": "src/auth/tokenService.ts",
    "span": "18-39",
    "signature": "createAccessToken(userId: string): string",
    "identity": "ts:function:createAccessToken:src/auth/tokenService.ts",
    "identity_confidence": 0.86
  },
  "edges": [
    {
      "from": "loginRoute",
      "to": "createAccessToken",
      "kind": "calls",
      "confidence": 0.91,
      "source_tool": "code-graph-mcp"
    }
  ],
  "ambiguity": [],
  "recommended_next_reads": [
    "src/auth/tokenService.ts",
    "src/routes/auth.ts"
  ]
}
```

If there are multiple matches, the broker must say so.

It should not pretend resolution is certain.

### Command 5: `impact`

`impact` answers:

```text
What might break if Codex changes this symbol or file?
```

Example:

```bash
ptctx impact --symbol "createAccessToken" --depth 2 --budget 3000
```

It should combine:

```text
callers
callees
imports
exports
routes
tests
config references
module reachability
current git diff if relevant
```

Example result:

```json
{
  "answer": "Changing createAccessToken affects login, refresh, and test helpers. The highest-risk caller is refreshSession because it shares expiry semantics with token rotation.",
  "risk": {
    "level": "medium",
    "reasons": [
      "3 direct callers",
      "1 route entry point",
      "2 integration tests reference token expiry behavior"
    ]
  },
  "affected": [
    {
      "file": "src/routes/auth.ts",
      "reason": "direct route caller",
      "priority": 1
    },
    {
      "file": "src/auth/refreshSession.ts",
      "reason": "direct caller with expiry logic",
      "priority": 2
    },
    {
      "file": "tests/auth/tokenExpiry.test.ts",
      "reason": "relevant behavioral test",
      "priority": 3
    }
  ],
  "recommended_next_tests": [
    "npm test -- tests/auth/tokenExpiry.test.ts",
    "npm test -- tests/auth/login.test.ts"
  ]
}
```

This is the heart of the broker.

If `impact` is weak, the broker is weak.

### Command 6: `route`

`route` answers:

```text
How does a CRUD/API request flow through the codebase?
```

This is a major workflow because the user explicitly works on CRUD apps.

Example:

```bash
ptctx route --query "POST /users" --budget 3000
```

It should return:

```text
route definition
handler
service calls
validation
database/repository calls
side effects
tests
uncertain runtime edges
```

Example:

```json
{
  "answer": "POST /users enters at usersRouter, calls createUserHandler, validates CreateUserInput, then calls createUserService and userRepository.insert.",
  "path": [
    {
      "kind": "route",
      "symbol": "usersRouter.post",
      "file": "src/routes/users.ts",
      "span": "14-20"
    },
    {
      "kind": "handler",
      "symbol": "createUserHandler",
      "file": "src/handlers/users.ts",
      "span": "22-58"
    },
    {
      "kind": "service",
      "symbol": "createUserService",
      "file": "src/services/users.ts",
      "span": "31-92"
    },
    {
      "kind": "storage",
      "symbol": "userRepository.insert",
      "file": "src/repositories/userRepository.ts",
      "span": "44-71"
    }
  ],
  "uncertainty": [
    "Middleware order was inferred from router registration and not proven from runtime."
  ],
  "recommended_next_reads": [
    "src/routes/users.ts",
    "src/handlers/users.ts",
    "src/services/users.ts",
    "src/repositories/userRepository.ts"
  ]
}
```

This is where tools like `sdsrss/code-graph-mcp`, `GitNexus`, and
codebase-memory-style graph tools should be tested hard.

If no tool traces routes well, this becomes a Parceltongue opportunity.

### Command 7: `review`

`review` answers:

```text
Given the current diff, what should Codex inspect before claiming done?
```

Example:

```bash
ptctx review --diff --budget 4000
```

It should consume:

```text
git diff
changed files
changed symbols
graph impact
relevant tests
security/structural rule hints
```

Example:

```json
{
  "answer": "The diff changes token expiry behavior. Review refreshSession and tokenExpiry tests before finalizing.",
  "changed_symbols": [
    {
      "name": "createAccessToken",
      "file": "src/auth/tokenService.ts",
      "change_type": "modified"
    }
  ],
  "review_focus": [
    {
      "file": "src/auth/refreshSession.ts",
      "reason": "direct caller not modified in diff"
    },
    {
      "file": "tests/auth/tokenExpiry.test.ts",
      "reason": "behavioral assertion likely affected"
    }
  ],
  "recommended_next_tests": [
    "npm test -- tests/auth/tokenExpiry.test.ts"
  ]
}
```

This command is important because it matches how Codex actually works:

```text
edit
verify
explain
```

The broker should improve the verify step.

### Command 8: `tests`

`tests` answers:

```text
Which tests are most relevant for this file, symbol, route, or diff?
```

Example:

```bash
ptctx tests --for "src/auth/tokenService.ts" --budget 1200
```

It should use:

```text
test filename conventions
imports from tests
graph edges from tests to implementation
package scripts
Cargo targets
pytest markers
Jest/Vitest config
git history if available
```

Example:

```json
{
  "answer": "Run tokenExpiry.test.ts first, then login.test.ts if route behavior changed.",
  "tests": [
    {
      "command": "npm test -- tests/auth/tokenExpiry.test.ts",
      "confidence": 0.91,
      "reason": "Imports createAccessToken and asserts expiry."
    },
    {
      "command": "npm test -- tests/auth/login.test.ts",
      "confidence": 0.72,
      "reason": "Login route calls createAccessToken indirectly."
    }
  ]
}
```

This prevents Codex from either running everything too early or skipping the
one test that matters.

### The Response Envelope

Every command should return the same top-level fields.

```json
{
  "schema_version": "ptctx.result.v1",
  "command": "impact",
  "status": "ok",
  "answer": "Short answer for Codex.",
  "confidence": 0.86,
  "freshness": {
    "status": "fresh",
    "indexed_at": "2026-07-06T16:45:00Z",
    "dirty_files": [],
    "warnings": []
  },
  "evidence": [],
  "edges": [],
  "recommended_next_reads": [],
  "recommended_next_tests": [],
  "uncertainty": [],
  "tool_trace": [],
  "token_budget": {
    "estimated_tokens": 950,
    "budget": 2000,
    "truncated": false
  }
}
```

The envelope matters more than any one tool.

Codex can build habits around a stable envelope.

If the envelope changes by command, Codex has to re-learn everything.

### Evidence Object

Every factual claim should be backed by evidence.

Evidence should look like:

```json
{
  "file": "src/auth/tokenService.ts",
  "span": "18-39",
  "symbol": "createAccessToken",
  "kind": "definition",
  "reason": "Defines the symbol requested by query.",
  "source_tool": "code-graph-mcp",
  "confidence": 0.94
}
```

Rules:

```text
No file means weak evidence.
No span means lower confidence.
No source tool means not debuggable.
No reason means Codex cannot decide.
```

The broker should prefer fewer evidence rows with stronger reasons.

### Edge Object

Dependency edges should be explicit.

```json
{
  "from": {
    "symbol": "loginRoute",
    "file": "src/routes/auth.ts",
    "span": "42-88"
  },
  "to": {
    "symbol": "createAccessToken",
    "file": "src/auth/tokenService.ts",
    "span": "18-39"
  },
  "kind": "calls",
  "direction": "forward",
  "confidence": 0.91,
  "source_tool": "codebase-memory-mcp",
  "provenance": "tree-sitter-call-expression"
}
```

Edge kinds should be normalized:

```text
calls
called_by
imports
exports
implements
extends
uses_type
routes_to
tests
configures
reads
writes
unknown
```

The broker should not invent false precision.

If an edge is inferred, say it is inferred.

### Tool Adapter Contract

Each tool adapter should implement the same small contract.

```text
name
detect
health
capabilities
freshness
run
normalize
```

In pseudocode:

```text
Adapter:
  name() -> string
  detect(repo) -> available | missing | misconfigured
  health(repo) -> ToolHealth
  capabilities() -> Capability[]
  freshness(repo) -> FreshnessStatus
  run(intent, input) -> RawToolOutput
  normalize(raw) -> BrokerFacts
```

Capabilities should be explicit:

```text
lexical_search
semantic_search
ast_search
module_graph
symbol_graph
call_graph
route_trace
impact_analysis
review_context
test_relevance
security_rules
structural_rewrite
```

This lets the broker route by need.

### Adapter Priority Table

The MVP should encode routing rules.

| Intent | First Tools | Fallbacks |
|---|---|---|
| Project orientation | graph project map, Clarity, package metadata | `rg --files`, README/docs scan |
| Unknown concept search | `cocoindex-code`, `codemogger`, `Probe` | `rg`, `fff` |
| Exact symbol lookup | graph MCP, Tree-sitter MCP | `rg`, `ast-grep` |
| Callers/callees | graph MCP | Clarity reachability, `rg` confirmation |
| CRUD route trace | `sdsrss/code-graph-mcp`, GitNexus | `rg` route patterns, framework heuristics |
| Diff review | `code-review-graph`, better review graph | `git diff`, graph impact |
| Relevant tests | review graph, graph MCP | naming conventions, imports, `rg` |
| Structural pattern | `ast-grep`, Semgrep | Comby, `rg` |
| Security/rules | Semgrep | CodeQL if configured |
| Module reach/cycles | Clarity | graph MCP |

This table is a product decision.

It prevents Codex from randomly trying tools.

### Freshness Model

Freshness must be first-class.

The broker should never say:

```text
Here is the impact radius.
```

without also saying:

```text
This graph is fresh.
```

or:

```text
This graph is stale because these files changed after indexing.
```

Freshness states:

```text
fresh
probably_fresh
stale
unknown
not_applicable
```

Freshness inputs:

```text
git dirty files
file mtimes
index mtimes
tool-specific index metadata
hash cache if available
known generated/vendor ignores
```

A stale answer can still be useful, but Codex must treat it differently.

Example:

```json
{
  "freshness": {
    "status": "stale",
    "dirty_files": ["src/auth/tokenService.ts"],
    "reason": "Graph index was built before the current file modification.",
    "safe_to_use_for": ["orientation", "unchanged modules"],
    "unsafe_for": ["final impact claim", "changed symbol callers"]
  }
}
```

This is where Parceltongue's prior incremental-indexing RCA becomes important.

If existing tools do not expose freshness, the broker should mark them
`unknown`, not pretend.

### Confidence Model

Confidence should be boring and explainable.

Start with:

```text
0.90-1.00: direct parser/graph evidence with fresh index
0.75-0.89: multiple tools agree or one strong tool with minor uncertainty
0.60-0.74: lexical/heuristic evidence or stale-but-likely graph
0.40-0.59: weak match, ambiguous symbol, inferred framework edge
0.00-0.39: do not act without reading raw code
```

Confidence should decrease when:

```text
index is stale
symbol is ambiguous
tools disagree
framework magic is involved
line spans are missing
only lexical evidence exists
generated code is involved
language support is partial
```

Confidence should increase when:

```text
multiple independent tools agree
graph edge has source span
test imports implementation directly
route path is explicit in code
freshness is confirmed
```

Codex does not need perfect math.

It needs a trustworthy warning system.

### Evidence Merging

The broker should deduplicate by:

```text
repo root
file path
symbol name
span
edge kind
```

If two tools return the same file but different spans, keep both if they point
to different symbols.

If two tools return the same edge, merge:

```text
source_tools
confidence
provenance
notes
```

If tools disagree, do not hide it.

Example:

```json
{
  "uncertainty": [
    "codebase-memory-mcp reports loginRoute calls createAccessToken, but Clarity only confirms module-level reachability.",
    "cocoindex-code ranked refreshSession high semantically, but no call edge was found."
  ]
}
```

Tool disagreement is useful context.

It tells Codex where to read raw code.

### Token-Budget Compression

The broker must not become a context firehose.

Each command should accept:

```text
--budget N
```

Budget behavior:

```text
under 1000 tokens: answer, top 3 evidence rows, top 3 next reads
1000-3000 tokens: include edges, affected files, tests
3000-6000 tokens: include snippets and secondary paths
6000+ tokens: include deeper transitive graph and alternate hypotheses
```

Truncation must be explicit.

```json
{
  "token_budget": {
    "estimated_tokens": 1980,
    "budget": 2000,
    "truncated": true,
    "truncation_reason": "Dropped depth-3 callers and low-confidence semantic matches."
  }
}
```

Codex should know when it is seeing a summary, not the whole world.

### Configuration File

The broker should have one repo-local or user-local config.

Example:

```toml
[broker]
default_budget = 2000
prefer_json = true
fail_on_stale_for_review = false

[tools.rg]
enabled = true
command = "rg"

[tools.clarity]
enabled = true
command = "clarity"

[tools.cocoindex_code]
enabled = true
command = "ccc"

[tools.code_graph_mcp]
enabled = true
mode = "cli"

[tools.code_review_graph]
enabled = true
command = "code-review-graph"

[ignore]
paths = [
  "node_modules",
  "target",
  "dist",
  "build",
  ".git"
]
```

Config should be optional.

The broker should auto-detect common tools first.

### Benchmark Integration

The broker should run the Concept 10 benchmark.

Commands:

```bash
ptctx bench list
ptctx bench run --suite smoke
ptctx bench run --suite full --tool-stack default
ptctx bench grade --run runs/20260706-ptctx.json
ptctx bench report --run runs/20260706-ptctx.json --format md
```

The benchmark should test the broker and each underlying tool.

Example:

```bash
ptctx bench run --case route_trace_001 --tool-stack codex-rg
ptctx bench run --case route_trace_001 --tool-stack graph-only
ptctx bench run --case route_trace_001 --tool-stack broker-default
```

The broker wins only if:

```text
It improves decision quality per token.
It does not merely add indirection.
```

### MVP Build Phases

The build should happen in small phases.

#### Phase 1: Shell-Only Broker

Goal:

```text
One local CLI that normalizes `git`, `rg`, and filesystem evidence.
```

Commands:

```text
health
project-map
search
review
tests
```

Acceptance:

```text
Runs in any git repo.
Requires no external graph tool.
Returns stable JSON.
Reports dirty files.
Respects token budget.
```

This phase is useful even before graph adapters exist.

#### Phase 2: Add Search Adapters

Goal:

```text
Use semantic or AST chunk search when installed.
```

Adapters:

```text
cocoindex-code
codemogger
Probe
fff
```

Acceptance:

```text
`ptctx search` ranks semantic and lexical results together.
Missing tools degrade cleanly.
Every result has file evidence and source tool.
```

#### Phase 3: Add Graph Adapters

Goal:

```text
Support callers, callees, route traces, and impact.
```

Adapters:

```text
GitNexus
codebase-memory-mcp
sdsrss/code-graph-mcp
Tessera
CodeGraphContext
Clarity
```

Acceptance:

```text
`ptctx symbol` returns definition plus direct edges.
`ptctx impact` returns affected files and tests if known.
`ptctx route` returns CRUD path when graph tool supports it.
Tool disagreement is visible.
```

#### Phase 4: Add Review Adapter

Goal:

```text
Make current diff review safer.
```

Adapters:

```text
code-review-graph
better-code-review-graph
git diff fallback
Semgrep optional
ast-grep optional
```

Acceptance:

```text
`ptctx review --diff` identifies changed symbols, impacted files, and focused tests.
It flags stale graph state.
It returns no generic code review prose without evidence.
```

#### Phase 5: Add Benchmark Harness

Goal:

```text
Measure whether the broker helps.
```

Acceptance:

```text
Runs at least the smoke suite.
Compares broker against Codex plus rg baseline.
Produces markdown report.
Scores decision quality per token.
```

#### Phase 6: Add Codex Ritual Docs

Goal:

```text
Teach Codex when to call the broker.
```

Deliverables:

```text
AGENTS.md snippet
Codex skill or local instruction
examples for pre-edit, post-edit, debug, review
```

Acceptance:

```text
Codex can follow the ritual without the user reminding it every time.
```

### First Codex Ritual

The first instruction to Codex should be short.

```text
Before editing a large or unfamiliar area, run `ptctx health`.
If health is fresh enough, run one of:
- `ptctx project-map` for repo orientation
- `ptctx search` when the symbol is unknown
- `ptctx symbol` when the symbol is known
- `ptctx impact` before changing a dependency-heavy symbol
- `ptctx route` for CRUD/API paths

After editing, run `ptctx review --diff` and `ptctx tests` before claiming done.
If ptctx reports stale or uncertain graph facts, inspect raw files directly.
```

That is enough for v1 of the ritual.

Codex should not need a 100-line prompt to use the broker.

### Acceptance Criteria For MVP

The broker MVP is done when these are true:

| Criterion | Pass Condition |
|---|---|
| Runs locally | `ptctx health` works in any git repo. |
| Does not require a graph tool | Missing graph tools produce degraded results, not crashes. |
| Stable response envelope | All commands return the same top-level JSON fields. |
| Evidence-first | Every recommendation has file/span/tool evidence or is marked uncertain. |
| Freshness visible | Dirty files and stale indexes are reported. |
| Token budget respected | Results fit the requested budget or explain truncation. |
| Codex can use it | Commands are simple enough for Codex shell use. |
| Benchmarked | At least smoke benchmark compares broker to `rg` baseline. |

This is a product-quality bar without pretending to be a full platform.

### Anti-Goals

The broker should not:

```text
parse every language itself
store a permanent graph first
build a UI first
ship an MCP server before CLI value exists
perform edits
hide raw tool errors
invent confidence
return long narrative summaries
require all optional tools
become a replacement agent
```

It should be boring, local, inspectable, and useful.

### Risks

| Risk | Mitigation |
|---|---|
| Tool output formats keep changing | Normalize only small required fields and preserve raw output paths in logs. |
| Missing tools make broker look weak | Health report and graceful degradation are mandatory. |
| Broker adds latency | Run independent adapters in parallel and obey command-specific tool routing. |
| Broker adds tokens | Enforce budgets and rank evidence hard. |
| Codex trusts stale graph facts | Freshness status is top-level and affects confidence. |
| Tool disagreement confuses Codex | Surface disagreement as explicit uncertainty and recommend raw reads. |
| User ends up maintaining too much | Start with shell-only plus one graph and one search adapter. |

### First Seven-Day Plan

Day 1:

```text
Define JSON envelope.
Implement `health`.
Implement git/rg adapters.
```

Day 2:

```text
Implement `project-map`.
Implement `search` with rg fallback.
Add budget trimming.
```

Day 3:

```text
Add one search adapter: cocoindex-code or codemogger.
Normalize search evidence.
```

Day 4:

```text
Add one graph adapter: codebase-memory-mcp, GitNexus, or code-graph-mcp.
Implement `symbol` and simple `impact`.
```

Day 5:

```text
Add `review --diff`.
Add test suggestion heuristics.
```

Day 6:

```text
Add smoke benchmark runner.
Compare broker against rg baseline on three cases.
```

Day 7:

```text
Write Codex ritual instructions.
Use the broker on one real repo task.
Record where it helped and where it lied.
```

The final sentence is important:

```text
Record where it lied.
```

That is how the broker becomes a path to truth rather than another layer of
confidence theater.

### When To Replace A Tool With Parceltongue Core

The broker should produce evidence for replacement.

Replace or supplement an external graph tool with Parceltongue core only when:

```text
The adapter repeatedly reports stale or unknown freshness.
The tool misses core edges in the benchmark.
The tool cannot handle the user's key languages.
The tool produces too many tokens for routine graph questions.
The tool cannot expose stable identity.
The tool cannot provide provenance for edges.
```

This makes Parceltongue evolution incremental.

Instead of:

```text
build everything
```

the path becomes:

```text
replace the weakest layer with a better local core
```

That is a much healthier architecture.

### Concept 12 Conclusion

The thin broker MVP should be a local Codex context router.

Its promise:

```text
Codex asks one small surface for context, and the broker decides which local
tools can answer with the strongest evidence and least token waste.
```

Its first implementation should be:

```text
CLI first
JSON envelope
git/rg health and fallback
one search adapter
one graph adapter
one review adapter
benchmark harness
Codex ritual
```

Its success metric is:

```text
Does Codex make better next-read, next-edit, next-test decisions per token?
```

If yes, the broker is valuable.

If no, it is just another wrapper.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| `docs/research002/J003.md` | Source-read | Supplies the candidate tool roles that the broker should route across. |
| `README.md` | Source-read | Supplies Parceltongue's original endpoint vocabulary for callers, callees, blast radius, and smart context. |
| Concept 10 in this document | Source-read through current synthesis | Supplies benchmark and DQPT requirements. |
| Concept 11 in this document | Source-read through current synthesis | Supplies staged decision logic and broker-before-core rationale. |
| `docs/research001/unclassified/RCA-Incremental-Indexing-Failure.md` | Source-read | Supplies freshness and stale-index failure modes the broker must expose. |
| `docs/research001/unclassified/ISGL1-v2-Stable-Entity-Identity.md` | Source-read | Supplies identity requirements that may later justify Parceltongue core under the broker. |

## Concept 13: Give Codex An Operating Manual For Context Graph Work

### The Point

A broker is only useful if Codex uses it at the right moments.

The main risk is not that Codex cannot call a tool.

The main risk is that Codex calls tools without a workflow.

Bad workflow:

```text
User asks for change.
Codex reads a few files.
Codex edits.
Codex runs a test.
Codex says done.
```

This works in small codebases.

It fails in large ones because the important code is often not in the first file
Codex reads.

Good workflow:

```text
User asks for change.
Codex checks repo state.
Codex asks for context graph guidance.
Codex reads ranked files.
Codex edits the smallest responsible area.
Codex asks for impact of the diff.
Codex runs focused tests.
Codex reports evidence and residual risk.
```

The operating manual turns that workflow into repeatable behavior.

The goal is not to make Codex dependent on a tool.

The goal is to make Codex less likely to act before it has the right map.

### The Manual In One Sentence

The Codex operating manual is:

```text
Before editing, ask what code matters; after editing, ask what changed and what
might break.
```

Everything else is elaboration.

### Default Rule

Codex should use `ptctx` when any of these are true:

```text
The repo is unfamiliar.
The task touches multiple files.
The user mentions a feature rather than an exact file.
The change may affect routes, services, storage, tests, or public API.
The codebase is large enough that raw grep creates too many candidates.
The target symbol has non-trivial callers or callees.
The diff is ready for review.
The graph or search context could prevent a wrong edit.
```

Codex can skip `ptctx` when:

```text
The user asks a tiny local question about an already-open file.
The file and exact edit are obvious.
The task is formatting-only.
The tool health is broken and direct shell exploration is faster.
The user explicitly asks not to use tools.
```

This keeps the broker from becoming ceremony.

### Startup Ritual

At the beginning of any substantial repo task, Codex should run:

```bash
ptctx health --json
```

Codex should inspect:

```text
repo root
branch
dirty files
untracked files
available tools
missing tools
stale indexes
partial language support
```

If health returns `fresh`:

```text
Use graph/search answers normally.
```

If health returns `degraded`:

```text
Use broker answers, but verify important edges with raw code.
```

If health returns `stale`:

```text
Do not make final impact claims from graph output.
Either refresh the relevant index or inspect raw files directly.
```

If health returns `unknown`:

```text
Treat the broker as search assistance, not source of truth.
```

This ritual prevents the worst failure:

```text
confident answers from stale graph facts
```

### Large Repo Orientation Ritual

Use when:

```text
Codex has never seen the repo.
The user asks broad questions.
The task names a feature area but not exact files.
The repo has many packages, crates, apps, or services.
```

Command:

```bash
ptctx project-map --budget 2000 --json
```

Codex should extract:

```text
repo type
main languages
entry points
module areas
test areas
generated/vendor exclusions
top recommended reads
freshness status
```

Then Codex should read only the top few files first.

Do not read the entire repo tree into context.

Do not ask for deep impact before orientation.

A good Codex update to the user would be:

```text
I am orienting through the project map first so I can find the route/service/test
shape before editing. Then I will follow the highest-confidence edge.
```

The product behavior is:

```text
map first, deep trace second
```

### Unknown Concept Search Ritual

Use when the user says something like:

```text
Find where auth timeout is handled.
Where do we create invoices?
How does onboarding send email?
Where is the retry logic?
```

Command:

```bash
ptctx search --query "auth timeout" --budget 1500 --json
```

Codex should do this:

```text
1. Look at top ranked result.
2. Check why it was ranked.
3. Read the exact span.
4. If confidence is high, ask `symbol` or `impact`.
5. If confidence is low, run a narrower search or use `rg`.
```

Codex should not do this:

```text
open 20 search results
trust a semantic match without code evidence
edit based on a file name only
ignore ambiguity
```

Decision rule:

```text
Search finds candidates.
Symbol/impact confirms what matters.
```

### Known Symbol Ritual

Use when Codex already has a function, class, route handler, module, trait, type,
or method name.

Command:

```bash
ptctx symbol --query "createAccessToken" --budget 2000 --json
```

Codex should ask:

```text
Where is the definition?
Is the symbol ambiguous?
What are direct callers?
What are direct callees?
What file spans prove this?
Is the index fresh?
```

Then Codex should read:

```text
definition first
top direct caller second
top direct callee third if behavior depends on it
tests fourth if tests are identified
```

Codex should not edit until:

```text
the symbol is resolved
the relevant span is read
callers/callees are understood or declared irrelevant
```

This is the antidote to editing the first matching function in a large repo.

### Pre-Edit Impact Ritual

Use before changing:

```text
shared services
public functions
API routes
database schemas
config loading
auth/payment paths
Rust traits
C/C++ headers
types used across packages
generated interface boundaries
```

Command:

```bash
ptctx impact --symbol "createAccessToken" --depth 2 --budget 3000 --json
```

Codex should classify the change:

| Impact Result | Codex Behavior |
|---|---|
| 0-2 direct callers | Small edit likely safe with focused test. |
| 3-10 direct callers | Read top callers before edit. |
| More than 10 direct callers | Treat as shared surface; consider narrower change. |
| Route or public API involved | Trace route and run integration tests. |
| Tests identified | Run focused tests after edit. |
| Stale graph | Verify callers with raw code or refresh index. |
| Ambiguous symbol | Resolve ambiguity before edit. |

Codex should produce a tiny internal plan:

```text
1. Read definition.
2. Read top caller.
3. Edit definition.
4. Run focused test.
5. Run review impact.
```

That plan should be based on graph facts, not vibes.

### CRUD Route Ritual

Use when:

```text
The task mentions endpoint behavior.
The task mentions CRUD flow.
The task mentions request/response shape.
The task mentions validation, service, repository, or database path.
```

Command:

```bash
ptctx route --query "POST /users" --budget 3000 --json
```

Codex should identify:

```text
route definition
middleware
handler
validation
service
repository/database call
side effects
tests
uncertain runtime edges
```

Read order:

```text
1. route file
2. handler file
3. service file
4. storage/repository file
5. relevant test
```

Codex should only deviate if the broker gives a strong reason.

For CRUD apps, the route ritual is probably the most valuable daily workflow.

It prevents Codex from changing only the service while missing validation,
serialization, or tests.

### Systems Programming Ritual

Use for Rust, C, and C++ tasks where call graphs are not enough.

Codex should combine:

```text
ptctx symbol
ptctx impact
raw compiler/test commands
rg for unsafe/FFI boundaries
header include checks for C/C++
trait and feature checks for Rust
```

Example Rust command sequence:

```bash
ptctx symbol --query "parse_config" --budget 2000 --json
ptctx impact --symbol "parse_config" --depth 2 --budget 3000 --json
rg -n "parse_config|Config" crates tests
cargo test -q parse_config
```

Example C/C++ command sequence:

```bash
ptctx symbol --query "parse_config" --budget 2000 --json
ptctx impact --symbol "parse_config" --depth 2 --budget 3000 --json
rg -n "#include|parse_config|Config" src include tests
```

Systems-specific questions:

```text
Is this function part of a public header?
Is it called across translation units?
Is there unsafe code or FFI nearby?
Does a type layout or ABI change?
Does a Rust trait implementation fan out across crates?
Does a feature flag change the active path?
```

The broker can help, but Codex must still respect compiler reality.

### Post-Edit Review Ritual

After editing, Codex should run:

```bash
ptctx review --diff --budget 4000 --json
```

Codex should inspect:

```text
changed symbols
changed files
impacted callers
impacted routes
impacted tests
stale graph warnings
security/rule warnings if present
tool disagreement
```

Then Codex should decide:

```text
run focused tests
read an unmodified impacted caller
adjust edit
run broader tests
report residual risk
```

Good final response evidence:

```text
Changed X.
Checked impact: Y callers and Z tests were relevant.
Ran focused test A.
Graph freshness was fresh/degraded/stale.
Residual risk is B.
```

Bad final response:

```text
Done.
```

Post-edit review is where the context broker saves the most embarrassment.

### Relevant Tests Ritual

Use when:

```text
Codex has edited code.
The test suite is large.
The relevant tests are not obvious.
The user wants speed.
```

Command:

```bash
ptctx tests --for "src/auth/tokenService.ts" --budget 1200 --json
```

or:

```bash
ptctx tests --diff --budget 1200 --json
```

Codex should run:

```text
highest-confidence focused test first
then route or integration test if public behavior changed
then broader package/crate suite if impact is wide
```

Codex should not stop at a weak test if the broker says public route impact is
medium or high.

The test ritual should produce:

```text
why this test
what it covers
what it does not cover
```

### Debugging Ritual

Use when:

```text
tests fail
runtime behavior is wrong
the user reports a bug
Codex needs to trace cause
```

Debugging sequence:

```text
1. Capture symptom.
2. Identify failing test, error, route, or log phrase.
3. `ptctx search` for symptom terms.
4. `ptctx route` or `ptctx symbol` for the likely entry point.
5. `ptctx impact` from suspect symbol.
6. Read one edge at a time.
7. Patch smallest responsible point.
8. `ptctx review --diff`.
9. Run focused test.
```

The key discipline:

```text
Follow one edge at a time.
```

Do not bounce randomly across files.

Graph context is useful because it gives an ordered next edge.

### Refactor Ritual

Use when:

```text
renaming
extracting function
moving module
changing public type
splitting service
altering trait/interface
```

Before refactor:

```bash
ptctx impact --symbol "OldSymbol" --depth 3 --budget 5000 --json
```

Codex should create a refactor map:

```text
definition
direct callers
direct callees
public API boundaries
tests
generated files to avoid
files likely requiring coordinated edits
```

During refactor:

```text
edit in smallest coherent batch
run compiler/typecheck early
use ast-grep or language tooling if rename is structural
avoid regex-only rename for typed symbols when tooling exists
```

After refactor:

```bash
ptctx review --diff --budget 5000 --json
ptctx tests --diff --budget 1500 --json
```

A refactor without impact context is where agents break things quietly.

### Review-Only Ritual

Use when the user asks for a review.

Codex should enter code-review stance.

Sequence:

```text
1. Inspect diff with git.
2. Run `ptctx review --diff`.
3. Read changed files.
4. Read top impacted unmodified files.
5. Check tests named by broker.
6. Report findings first, ordered by severity.
```

Codex should not summarize before findings.

Codex should not praise style before checking behavior.

The broker helps identify the files that are not in the diff but matter.

### Stale Context Ritual

If `ptctx` reports stale context:

```text
1. Do not ignore it.
2. Identify which files are stale.
3. If a refresh command exists, run it.
4. If no refresh exists, verify affected edges with raw code.
5. Lower confidence in final claims.
```

Final wording should be honest:

```text
The graph index was stale for src/auth/tokenService.ts, so I verified the
caller path directly with rg and file reads before editing.
```

This builds trust.

It also prevents the worst possible graph-tool failure:

```text
wrong confidence
```

### Low Confidence Ritual

If confidence is below 0.75:

```text
read raw files before editing
prefer direct evidence
ask one narrower query
avoid broad claims
mention uncertainty if relevant
```

If confidence is below 0.60:

```text
do not edit from broker output alone
```

If confidence is below 0.40:

```text
treat broker output as a hint only
```

The broker should make uncertainty usable, not scary.

### Tool Disagreement Ritual

If two tools disagree:

```text
1. Identify the exact disagreement.
2. Prefer fresher source.
3. Prefer source with file/span evidence.
4. Prefer parser/graph edge over semantic similarity for dependency claims.
5. Read raw code at the disputed edge.
6. Record uncertainty if unresolved.
```

Example:

```text
cocoindex-code ranked refreshSession high, but no graph tool found a call edge.
I treated it as semantically related, not as a confirmed caller, and read the
file before deciding.
```

This is how Codex should talk when graph and search evidence differ.

### Token Budget Ritual

Codex should choose budgets by task.

| Task | Budget |
|---|---:|
| health | 800 |
| project-map | 2000 |
| search | 1200-2000 |
| symbol | 1500-2500 |
| impact small | 2500 |
| impact refactor | 4000-6000 |
| route trace | 3000-5000 |
| review diff | 3000-5000 |
| tests | 1000-1500 |

If output is truncated:

```text
Use top evidence first.
Ask a narrower follow-up instead of raising budget blindly.
```

This protects the context window.

The broker is supposed to reduce context waste, not formalize it.

### Codex User Updates

When Codex uses the broker, user-facing updates should be short but informative.

Good updates:

```text
I am checking the context graph first so I can identify callers and tests before editing.
```

```text
The graph found two direct callers and one route path; I am reading those before touching the service.
```

```text
The index is stale for the edited file, so I am verifying callers with raw code before I trust the impact result.
```

Bad updates:

```text
Running tool.
```

```text
Analyzing.
```

```text
I will ensure everything is correct.
```

The broker should make updates more concrete.

### AGENTS.md Snippet

The eventual repo instruction could include:

```markdown
## Codex Context Broker Ritual

For substantial changes, use `ptctx` before editing and after editing.

Before editing:
- Run `ptctx health --json`.
- Use `ptctx project-map` for unfamiliar repos.
- Use `ptctx search` when the feature is known but the symbol is not.
- Use `ptctx symbol` when the symbol is known.
- Use `ptctx impact` before changing shared code.
- Use `ptctx route` for CRUD/API paths.

After editing:
- Run `ptctx review --diff --json`.
- Run `ptctx tests --diff --json` when focused tests are not obvious.
- If `ptctx` reports stale or low-confidence graph facts, verify with raw code.

Never treat semantic search results as dependency facts unless confirmed by graph
or raw code. Never make final impact claims from stale indexes.
```

This is short enough to be remembered.

### Operating Manual Examples

#### Example 1: User Says "Change Signup Email Copy"

Codex sequence:

```bash
ptctx health --json
ptctx search --query "signup email copy" --budget 1500 --json
ptctx impact --symbol "sendSignupEmail" --depth 2 --budget 3000 --json
```

Read:

```text
email template
email service
signup route or service
email test
```

Edit:

```text
template or copy source only
```

Verify:

```bash
ptctx review --diff --budget 3000 --json
ptctx tests --diff --budget 1200 --json
```

#### Example 2: User Says "Fix Rust Parser Crash"

Codex sequence:

```bash
ptctx health --json
ptctx search --query "parser crash timeout tree-sitter" --budget 2000 --json
ptctx symbol --query "parse" --budget 2500 --json
ptctx impact --symbol "parse_source_file" --depth 2 --budget 4000 --json
```

Read:

```text
parser lifecycle
timeout/error handling
tests for bad syntax
callers that assume parse success
```

Verify:

```bash
cargo test -q parser
ptctx review --diff --budget 4000 --json
```

#### Example 3: User Says "Review This Diff"

Codex sequence:

```bash
ptctx health --json
ptctx review --diff --budget 5000 --json
ptctx tests --diff --budget 1500 --json
```

Then:

```text
read changed files
read top impacted unmodified files
inspect named tests
report findings first
```

### Failure Modes The Manual Prevents

| Failure | Manual Countermeasure |
|---|---|
| Codex edits first matching file | Search then symbol confirmation. |
| Codex misses callers | Pre-edit impact. |
| Codex misses route path | CRUD route ritual. |
| Codex trusts stale graph | Health and stale context ritual. |
| Codex runs wrong tests | Relevant tests ritual. |
| Codex gives vague final answer | Post-edit review evidence. |
| Codex overuses token budget | Budget ritual. |
| Codex confuses semantic similarity with dependency | Tool disagreement and confidence rituals. |

The operating manual is not bureaucracy.

It is a set of safety rails for large-codebase work.

### Product Implication

The operating manual reveals what the broker must make easy.

If Codex has to remember too much, the broker API is wrong.

If Codex has to parse huge tool output, the broker response is wrong.

If Codex cannot tell stale from fresh, the broker health model is wrong.

If Codex cannot tell search from graph evidence, the evidence model is wrong.

If Codex still reads 20 files before editing, the ranking model is wrong.

The manual is not just instructions.

It is a usability test for the broker.

### Concept 13 Conclusion

The broker needs an operating manual because context tools only help when used
at the right moments.

The core ritual is:

```text
health
orient or search
symbol or route
impact before edit
review after edit
focused tests
honest final evidence
```

This turns graph context into agent behavior.

That behavior is the product.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| Concept 12 in this document | Source-read through current synthesis | Supplies broker command surface and response envelope. |
| Concept 10 in this document | Source-read through current synthesis | Supplies token-budget and evaluation logic. |
| Concept 11 in this document | Source-read through current synthesis | Supplies Codex-as-fixed-agent framing and staged decision. |
| `README.md` | Source-read | Supplies v1.7.2 recommended orientation/search/trace/blast-radius/smart-context workflow. |
| `docs/research002/J003.md` | Source-read | Supplies Codex tool-stack role separation used in the operating manual. |

## Concept 14: Use A Universal Relationship Schema, Not A Single Dependency Graph

### The Point

There is a universal way to think about code relationships, but it is not:

```text
everything is a function call
```

and it is not:

```text
everything is an import
```

and it is not:

```text
everything is a dependency edge
```

The useful abstraction is:

```text
code relationship facts with typed nodes, typed edges, evidence, freshness,
confidence, and projections
```

That is the heart of a context graph.

The graph should not try to flatten every relationship into one kind of edge.

It should store facts precisely, then present the right projection for the
agent's current job.

Examples:

```text
For editing a function: show callers, callees, tests, and config.
For changing an API route: show route, handler, service, storage, tests.
For refactoring a public type: show public interface dependents.
For debugging a bug: show execution path and data/config inputs.
For review: show changed symbols, impacted edges, and missing tests.
```

The same underlying fact graph can support all of those views.

### Why "Dependency Graph" Is Too Small

The phrase "dependency graph" is useful but incomplete.

A typical dependency graph answers:

```text
A depends on B.
```

That is not enough for agents.

Codex often needs to know:

```text
Does A call B?
Does A import B?
Does A export B?
Does A route to B?
Does A validate input before B?
Does A read config used by B?
Does A write data later read by B?
Does test T cover B?
Does public API P expose type B?
Does generated file G come from schema S?
Does changing B affect package boundary X?
```

Those are not the same relationship.

If the graph collapses them, Codex loses meaning.

The right model is a relationship schema.

### The Core Object: A Fact

The universal unit should be a fact.

Fact shape:

```json
{
  "subject": "node-id-a",
  "predicate": "calls",
  "object": "node-id-b",
  "scope": "src/routes/auth.ts",
  "evidence": [
    {
      "file": "src/routes/auth.ts",
      "span": "42-51",
      "snippet_hash": "sha256:abc123",
      "source_tool": "tree-sitter-query-pack",
      "detection_method": "static_parse"
    }
  ],
  "confidence": 0.92,
  "freshness": {
    "status": "fresh",
    "indexed_at": "2026-07-06T16:45:00Z",
    "commit": "abc123"
  }
}
```

This looks simple, but it changes everything.

Instead of asking:

```text
What is the dependency graph?
```

Codex can ask:

```text
Which relationship facts matter to this decision?
```

### Node Types

The schema should start with common node types.

| Node Type | Meaning | Examples |
|---|---|---|
| `repo` | Git repository or workspace root | `parseltongue-rust-LLM-companion` |
| `package` | Package/crate/module unit | npm package, Cargo crate, Python package |
| `build_target` | Build/test target | Cargo test target, Bazel target, npm script |
| `file` | Source or config file | `src/auth.ts`, `Cargo.toml` |
| `module` | Logical module or namespace | Rust module, TS module, Python module |
| `symbol` | Function, class, method, type, trait, variable | `createUser`, `AuthService`, `Parser` |
| `type` | Type-level entity when distinct from symbol | interface, struct, enum, typedef |
| `route` | HTTP or RPC route entry point | `POST /users`, `GET /health` |
| `cli_command` | CLI command or subcommand | `ptctx impact` |
| `test` | Test function, file, suite, fixture | `test_token_expiry` |
| `config_key` | Env/config/feature flag | `TOKEN_TTL_SECONDS`, `DATABASE_URL` |
| `data_entity` | Table, collection, schema, topic, queue | `users` table, `UserCreated` event |
| `external_api` | External service or SDK surface | Stripe API, S3 client |
| `generated_artifact` | Generated file or output | OpenAPI client, protobuf file |
| `doc` | Human-readable spec/doc linked to code | README, ADR, PRD |

This set is broad enough for CRUD apps and systems code.

It also stays small enough for Codex to understand.

### Edge Categories

Edges should be grouped by meaning.

#### Structural Edges

Structural edges describe code organization.

```text
contains
defines
declares
exports
imports
reexports
includes
generated_from
documented_by
```

Examples:

```text
file defines symbol
module exports type
C++ file includes header
generated client generated_from OpenAPI schema
```

Structural edges help Codex orient.

They do not prove runtime behavior.

#### Execution Edges

Execution edges describe possible runtime flow.

```text
calls
called_by
awaits
spawns
schedules
handles
routes_to
emits_event
subscribes_to
throws
catches
```

Examples:

```text
loginRoute calls createAccessToken
POST /users routes_to createUserHandler
worker subscribes_to UserCreated
task spawns backgroundSync
```

Execution edges are central for debugging and impact analysis.

#### Type And Interface Edges

Type/interface edges describe contracts.

```text
implements
extends
conforms_to
instantiates
uses_type
returns_type
accepts_type
specializes
overrides
```

Examples:

```text
PgUserRepository implements UserRepository
AuthService accepts_type LoginRequest
Rust struct implements Display
C++ class extends BaseParser
```

These edges are crucial for refactoring.

Agents often break type contracts when they only follow calls.

#### Data Edges

Data edges describe data movement and persistence.

```text
reads
writes
mutates
validates
serializes
deserializes
maps_to
queries
publishes
consumes
```

Examples:

```text
createUserService writes users table
UserDto serializes User
validateCreateUserInput validates request body
consumer consumes UserCreated event
```

CRUD apps need data edges as much as call edges.

#### Config And Environment Edges

Config edges describe behavior controlled outside source code.

```text
reads_config
sets_config
feature_gated_by
requires_env
uses_secret
loads_file
```

Examples:

```text
tokenService reads_config TOKEN_TTL_SECONDS
newParser feature_gated_by experimental_parser
database client requires_env DATABASE_URL
```

These edges matter because many bugs hide in configuration, not call graphs.

#### Test Edges

Test edges describe verification coverage.

```text
tests
covers
fixtures
mocks
asserts_behavior
regression_for
```

Examples:

```text
tokenExpiry.test tests createAccessToken
mockStripeClient mocks StripeClient
test_parser_timeout regression_for parser hang bug
```

Agents need test edges to choose verification.

#### Build And Packaging Edges

Build edges describe how code becomes runnable.

```text
depends_on_package
builds
links
compiled_into
loaded_by
uses_feature
```

Examples:

```text
crate pt08 depends_on_package parseltongue-core
binary compiled_into parseltongue
Rust module uses_feature sqlite
```

These edges matter in Rust/C/C++ and monorepos.

#### Ownership And Human Edges

Ownership edges are optional but useful.

```text
owned_by
reviewed_by
documented_by
decision_recorded_in
deprecated_by
```

Examples:

```text
module documented_by ADR-004
public API deprecated_by v2 endpoint
```

For a solo user these are less about team workflow and more about memory.

### Edge Qualifiers

An edge needs qualifiers.

Without qualifiers, Codex cannot tell a hard fact from a guess.

Useful qualifiers:

```text
direct or transitive
static or inferred or runtime_observed
public or private
local or cross_module or cross_package
fresh or stale
language
source_tool
confidence
visibility
cardinality
```

Example:

```json
{
  "kind": "calls",
  "directness": "direct",
  "evidence_type": "static",
  "visibility": "private",
  "scope": "same_file",
  "language": "typescript",
  "confidence": 0.93
}
```

This lets Codex reason:

```text
This is a direct static call inside one file.
```

versus:

```text
This is an inferred route edge through framework registration.
```

Those should not be treated equally.

### Public Interface Dependency Graph

The user's earlier question about a public interface dependency graph is exactly
right.

A public interface dependency graph is a projection of the universal graph.

It filters to:

```text
exported symbols
public functions
public types
crate public APIs
C/C++ headers
HTTP routes
RPC methods
CLI commands
MCP endpoints
database schemas
event topics
config keys
generated contracts
```

Then it shows:

```text
who exposes what
who consumes it
what tests cover it
what changes might break downstream users
```

This is the right graph for:

```text
refactors
API changes
package boundary changes
public type changes
large blast-radius analysis
```

It is not the right graph for:

```text
debugging one private helper
finding a local implementation detail
changing copy in one template
```

The public interface graph should be one projection, not the only graph.

### Have Others Tried This?

Yes, in different forms.

The prior art shows that serious code intelligence systems converge on typed
relationships:

```text
SCIP models symbols, occurrences, packages, descriptors, and relationships.
Kythe models code as typed graph nodes and edges with anchors.
Glean stores typed facts and schemas that can be queried.
CodeQL models code facts and data/control-flow relations for queries.
Stack graphs model name binding and scope relationships.
Semgrep and ast-grep model syntax patterns over AST nodes.
Clarity-style tools expose module/dependency/reachability views.
```

The missing piece for this user's PMF is not the idea of a code graph.

The missing piece is:

```text
a small Codex-friendly relationship contract that is local, fresh, token-aware,
and action-oriented
```

That is where Parceltongue or the broker can add value.

### Why Projections Matter

One graph can be too large.

Agents do not need the whole graph.

They need the right projection.

Useful projections:

| Projection | Question It Answers |
|---|---|
| Local call graph | What calls this, and what does it call? |
| Public interface graph | What public surface might break? |
| Route graph | How does request flow through CRUD code? |
| Test graph | What verifies this behavior? |
| Data graph | What data is read, written, validated, serialized? |
| Config graph | What settings influence this behavior? |
| Build graph | What packages/crates/targets are affected? |
| Review graph | What did this diff change and what should be inspected? |
| Ownership/doc graph | What docs or decisions explain this area? |

The broker should ask for projections.

The storage layer should keep facts.

### Universal Schema And Token Budgets

The universal schema should not mean universal output.

Codex should never receive all facts unless it asks for them.

For a small `symbol` query, return:

```text
definition
top callers
top callees
top tests
freshness
uncertainty
```

For an `impact` query, return:

```text
affected files
affected public interfaces
affected tests
risk reasons
top edges
```

For a `route` query, return:

```text
route path
handler
service
storage
tests
uncertain middleware/runtime edges
```

The schema can be rich.

The response must be sparse.

### Stable Identity

Universal relationships only work if nodes have stable identity.

A node id should not be based only on line numbers.

Good identity ingredients:

```text
repo identity
language
node kind
semantic path
file path hash
symbol name
parent symbol
content hash
birth id or stable occurrence id
```

A node should have:

```text
stable_id
semantic_path
current_location
content_hash
observed_at
```

Example:

```json
{
  "stable_id": "ts:function:createAccessToken:src/auth/tokenService.ts:T1706284800",
  "semantic_path": "src/auth/tokenService.ts::createAccessToken",
  "current_location": {
    "file": "src/auth/tokenService.ts",
    "span": "18-39"
  },
  "content_hash": "sha256:abc123",
  "observed_at": "2026-07-06T16:45:00Z"
}
```

Edges also need identity.

Edge id:

```text
hash(subject_stable_id + predicate + object_stable_id + scope)
```

This lets the graph detect:

```text
same relationship, moved line
same symbol, changed content
new relationship
deleted relationship
ambiguous relationship
```

Without stable identity, the public interface graph becomes noisy after edits.

### Relationship Confidence

Not all relationships are equally knowable.

Examples:

```text
Rust direct function call from parsed AST: high confidence.
TypeScript dynamic import string: medium confidence.
Express route assembled through helper arrays: medium to low confidence.
Reflection-based Python call: low confidence.
Config key read from string literal: medium confidence.
Semantic search result: not a dependency fact.
```

The schema should allow:

```text
confidence
confidence_reason
detection_method
evidence_type
```

Example:

```json
{
  "kind": "routes_to",
  "confidence": 0.68,
  "confidence_reason": "Route inferred from Express router call and handler reference; middleware ordering not proven.",
  "detection_method": "framework_pattern",
  "evidence_type": "static_inferred"
}
```

This lets Codex decide whether to trust, verify, or ignore.

### Relationship Freshness

Every relationship fact should carry freshness.

Freshness fields:

```text
indexed_at
commit_sha
file_hash
dirty_after_index
source_tool
tool_index_version
```

Example:

```json
{
  "freshness": {
    "status": "stale",
    "indexed_at": "2026-07-06T12:00:00Z",
    "dirty_after_index": ["src/auth/tokenService.ts"],
    "unsafe_for": ["impact", "review"]
  }
}
```

For agents, stale graph facts should be visibly downgraded.

Freshness is part of correctness.

### CRUD Example

Consider:

```text
POST /users creates a user.
```

Useful nodes:

```text
route: POST /users
symbol: createUserHandler
type: CreateUserRequest
symbol: validateCreateUserInput
symbol: createUserService
data_entity: users table
test: createUser.test.ts
config_key: USER_SIGNUP_ENABLED
```

Useful facts:

```text
route POST /users routes_to createUserHandler
createUserHandler accepts_type CreateUserRequest
createUserHandler calls validateCreateUserInput
createUserHandler calls createUserService
createUserService writes users table
createUserService reads_config USER_SIGNUP_ENABLED
createUser.test.ts tests POST /users
```

The route projection for Codex should return:

```text
route -> handler -> validation -> service -> storage -> tests
```

The impact projection for changing `CreateUserRequest` should return:

```text
route
handler
validator
service
tests
public API contract
```

The same facts support both.

### Rust Systems Example

Consider:

```text
parse_source_file parses code into entities.
```

Useful nodes:

```text
symbol: parse_source_file
type: ParseResult
symbol: tree_sitter::Parser
symbol: extract_entities_from_tree
test: parser_timeout_tests
config_key: parse_timeout_ms
crate: parseltongue-core
```

Useful facts:

```text
parse_source_file calls tree_sitter::Parser.parse
parse_source_file returns_type ParseResult
parse_source_file calls extract_entities_from_tree
parse_source_file reads_config parse_timeout_ms
parser_timeout_tests tests parse_source_file
parseltongue-core exports parse_source_file
```

The systems projection should emphasize:

```text
callers
error paths
timeout behavior
unsafe/FFI boundaries if any
tests
crate/public exports
```

That is different from a CRUD route projection.

### C/C++ Public Header Example

Consider:

```text
int parse_config(const char* path, Config* out);
```

Useful facts:

```text
include/config.h declares parse_config
src/config.c defines parse_config
main.c calls parse_config
tests/config_test.c tests parse_config
parse_config writes Config*
library target exports parse_config
```

The public interface graph matters here because changing the signature can
break downstream compilation.

Codex should see:

```text
public header
definition
call sites
tests
build targets
```

Not only:

```text
this function calls fopen
```

### Relationship Schema As A Broker Contract

The broker does not need to store all facts itself.

It does need to normalize facts from tools.

Adapter output should map into:

```text
nodes
facts
evidence
freshness
confidence
tool_trace
```

This lets different tools interoperate.

Example:

```text
Clarity returns module reachability.
code-graph-mcp returns route/call edges.
cocoindex-code returns semantic search chunks.
Semgrep returns rule matches.
rg returns lexical spans.
```

The broker can convert those into a shared evidence graph, even if some facts
are weaker than others.

Semantic search result should become:

```text
candidate_relevance
```

not:

```text
calls
```

That distinction prevents category errors.

### Minimal Schema For MVP

The first schema should be small.

Node kinds:

```text
file
module
symbol
route
test
config_key
data_entity
package
```

Edge kinds:

```text
defines
imports
exports
calls
routes_to
uses_type
reads_config
reads
writes
tests
depends_on_package
candidate_relevance
```

Metadata:

```text
confidence
freshness
source_tool
evidence file/span
directness
visibility
```

This is enough for real Codex workflows.

Do not start with 100 edge types.

Start with the relationships Codex needs to avoid dumb edits.

### Schema Evolution

The schema should be versioned.

Example:

```json
{
  "schema_version": "ptctx.relationships.v1",
  "node_schema_version": "ptctx.nodes.v1",
  "edge_schema_version": "ptctx.edges.v1"
}
```

Rules:

```text
Add edge kinds carefully.
Keep old edge kinds stable.
Preserve raw tool kind in metadata.
Do not silently reinterpret old facts.
Document confidence semantics.
Document whether edge is static, inferred, or observed.
```

This is how the broker can grow without confusing Codex.

### Query Examples

The schema should support natural agent queries:

```text
Show me the public interface impact of changing CreateUserRequest.
Show callers of parse_source_file and tests that cover it.
Trace POST /users from route to storage.
Which config keys influence token expiration?
Which tests cover this changed diff?
Which public routes depend on UserRepository?
Which files are candidates for this concept but not confirmed dependencies?
Which graph facts are stale for this diff?
```

Each query is just a projection over relationship facts.

### What Parceltongue Should Learn

Parceltongue v1.7.2 already had the right instinct:

```text
entities
dependency edges
callers
callees
blast radius
smart context
```

Concept 14 says the next evolution is:

```text
typed relationship facts
public interface projection
route/data/test/config projections
freshness on every fact
confidence on every edge
evidence spans on every claim
```

That is not a totally different product.

It is a sharper version of the original idea.

### Concept 14 Conclusion

There is a universal relationship model for code assistance, but it should be
implemented as typed facts with projections.

The core idea:

```text
store precise relationship facts, then show Codex the projection needed for the
current decision
```

The key projections:

```text
local call graph
public interface graph
route graph
test graph
data graph
config graph
build graph
review graph
```

The key metadata:

```text
evidence
confidence
freshness
source tool
directness
visibility
uncertainty
```

This is the contract that lets Codex trust context without drowning in it.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| Concept 8 in this document | Source-read through current synthesis | Supplies prior-art layer distinctions across SCIP, Kythe, Glean, CodeQL, stack-graphs, Semgrep, ast-grep, Clarity, and related systems. |
| Concept 12 in this document | Source-read through current synthesis | Supplies broker response and adapter contract that the relationship schema should feed. |
| Concept 13 in this document | Source-read through current synthesis | Supplies operating rituals that require different graph projections. |
| `README.md` | Source-read | Supplies original Parceltongue entity/edge/callers/callees/blast-radius/smart-context vocabulary. |
| `docs/research001/unclassified/ISGL1-v2-Stable-Entity-Identity.md` | Source-read | Supplies stable identity requirements for durable node and edge facts. |
| `docs/research001/unclassified/RCA-Incremental-Indexing-Failure.md` | Source-read | Supplies freshness and stale-index requirements that relationship facts must carry. |

## Concept 15: Make The Public Interface Graph The First High-Value Projection

### The Point

If the broker or Parceltongue can only build one projection first, it should
build the public interface graph.

Reason:

```text
Agents are most dangerous when they change code that other code depends on.
```

Private helper mistakes are often caught quickly.

Public interface mistakes spread.

Public interface means:

```text
exported functions
exported types
HTTP routes
CLI commands
Rust pub APIs
C/C++ headers
Python package APIs
config keys
database schema contracts
event topics
generated API contracts
MCP endpoints
```

The public interface graph answers:

```text
What surface does this code expose, and who depends on that surface?
```

That is the question Codex must answer before changing shared behavior.

### Publicness Is Not Binary

The word "public" can mislead.

There are several levels of publicness:

| Level | Meaning | Examples |
|---|---|---|
| `external_public` | Used outside the repo/process/package | HTTP route, SDK method, CLI command, C header installed for clients |
| `package_public` | Exported from package/crate/module boundary | npm export, Rust `pub` item, Python `__all__` |
| `workspace_public` | Used across monorepo packages/crates/apps | internal package API, shared library function |
| `module_public` | Public inside one package/module | exported helper used by sibling files |
| `private` | Local implementation detail | unexported function, file-local helper |
| `unknown` | Cannot determine reliably | dynamic export, reflection, framework magic |

Codex should treat these differently.

Changing `private` code often needs local tests.

Changing `external_public` code needs route/API/contract tests and impact
analysis.

The public interface graph is really an exposed-surface graph.

### Public Interface Node

A public interface node should have fields like:

```json
{
  "id": "ts:route:POST /users:src/routes/users.ts:T1706284800",
  "kind": "route",
  "name": "POST /users",
  "visibility": "external_public",
  "contract_kind": "http_route",
  "owner_node": "ts:function:createUserHandler:src/handlers/users.ts:T1706284801",
  "definition": {
    "file": "src/routes/users.ts",
    "span": "14-20"
  },
  "stability": "stable",
  "source_tool": "tree-sitter-query-pack",
  "confidence": 0.88
}
```

Important fields:

```text
kind
visibility
contract_kind
owner_node
definition
stability
confidence
freshness
```

The graph should not merely say:

```text
route exists
```

It should say:

```text
route exists here, is handled by this symbol, is externally visible, and is
covered by these tests
```

### Public Interface Edge

Useful public interface edges:

```text
exposes
implements_contract
consumes_contract
routes_to
exports
reexports
declares
defined_by
covered_by
documented_by
generated_from
depends_on_contract
reads_config
maps_to_schema
```

Example:

```json
{
  "from": "ts:route:POST /users",
  "to": "ts:function:createUserHandler",
  "kind": "routes_to",
  "visibility": "external_public",
  "confidence": 0.88,
  "evidence": [
    {
      "file": "src/routes/users.ts",
      "span": "14-20",
      "source_tool": "tree-sitter-query-pack"
    }
  ]
}
```

The edge must carry confidence because public interface extraction often crosses
framework boundaries.

### Extraction Algorithm

The public interface graph can be built in seven passes.

```text
1. Discover package/workspace boundaries.
2. Parse files with language-specific query packs.
3. Mark candidate public nodes.
4. Resolve exports, reexports, headers, routes, commands, and schemas.
5. Link public nodes to implementation nodes.
6. Link consumers, tests, config, docs, and generated contracts.
7. Score visibility, confidence, freshness, and blast radius.
```

Pass 1 answers:

```text
Where are the boundaries?
```

Pass 2 answers:

```text
What symbols and declarations exist?
```

Pass 3 answers:

```text
Which of these might be exposed?
```

Pass 4 answers:

```text
Which are actually exported or reachable?
```

Pass 5 answers:

```text
What implementation backs the exposed surface?
```

Pass 6 answers:

```text
Who consumes or verifies the exposed surface?
```

Pass 7 answers:

```text
How risky is a change?
```

### Boundary Discovery

Before language extraction, detect boundaries.

Files and markers:

```text
package.json
tsconfig.json
Cargo.toml
pyproject.toml
setup.py
setup.cfg
CMakeLists.txt
Makefile
BUILD or BUILD.bazel
go.mod
openapi.yaml
schema.graphql
proto files
Dockerfile
```

Boundary facts:

```text
package defines package boundary
crate defines crate boundary
include directory defines C/C++ public header boundary
OpenAPI file defines HTTP contract boundary
proto file defines RPC/message contract boundary
package exports define npm public surface
Cargo lib target defines Rust library surface
CLI binary defines command surface
```

Without boundaries, "public" becomes guesswork.

### TypeScript Extraction Rules

TypeScript and JavaScript public surfaces include:

```text
exported functions
exported classes
exported interfaces
exported types
exported constants
default exports
barrel exports
package.json exports
package.json bin
route registrations
schema/model exports
public class methods on exported classes
```

Tree-sitter/query-pack candidates:

```text
export_statement
export_clause
lexical_declaration with export
function_declaration with export
class_declaration with export
interface_declaration with export
type_alias_declaration with export
method_definition inside exported class
call_expression for router/http framework registration
```

Rules:

```text
If symbol is directly exported, mark package_public.
If symbol is reexported from index.ts, mark package_public and record reexport edge.
If package.json exports path exposes file, mark exported file symbols as package_public.
If package.json bin points to file, mark CLI command surface.
If exported class has public methods, mark methods as module_public or package_public depending on usage.
If route registration uses exported router, mark route external_public.
If route handler is not exported but reachable from route, mark handler as implementation_of public route.
```

Common route patterns:

```text
express.Router().get/post/put/patch/delete
app.get/post/put/patch/delete
fastify.get/post/put/patch/delete
next route files
remix route modules
nestjs controller decorators if available through parser/decorator queries
```

TypeScript uncertainties:

```text
dynamic exports
computed route paths
decorators without metadata understanding
framework file conventions
runtime plugin registration
barrel exports with conditional logic
```

Codex should see those as lower confidence.

### Rust Extraction Rules

Rust public surfaces include:

```text
pub fn
pub struct
pub enum
pub trait
pub type
pub const
pub static
pub mod
pub use
pub(crate) workspace/crate surfaces
trait impls for public types
extern "C" and no_mangle functions
clap CLI commands
axum/actix/rocket routes
Cargo feature flags
```

Rules:

```text
`pub` item in lib.rs or public module is package_public.
`pub(crate)` item is workspace_public inside crate.
`pub(super)` item is module_public.
`pub use` creates reexport edge.
Public struct fields are part of contract.
Private struct fields are implementation unless constructor/accessors expose them.
Public trait methods are public interface.
Impl of public trait for public type is contract behavior.
Extern C no_mangle function is external_public FFI surface.
Binary clap command/subcommand is external_public CLI surface.
Axum route mapping is external_public HTTP surface.
Cargo feature flag is config/build public surface.
```

Rust-specific facts:

```text
item visibility
module path
crate name
feature gates
trait implementation
type signatures
error/result types
```

Rust uncertainties:

```text
macro-generated routes
proc macro generated public APIs
conditional compilation
feature-dependent exports
trait object dynamic dispatch
```

For Rust, the public graph is extremely valuable because changing a public type
or trait can trigger wide compile failures.

### C/C++ Extraction Rules

C and C++ public surfaces are often in headers.

Public candidates:

```text
functions declared in public include directories
classes declared in public headers
structs/enums/typedefs in public headers
public class methods
virtual methods
exported symbols with export macros
extern "C" declarations
installed headers
CMake install targets
pkg-config metadata
```

Rules:

```text
Declaration in include/ or public header path is package_public.
Definition matching public declaration is implementation node.
Class public methods are public interface.
Protected methods are subclass interface.
Private methods are implementation.
Virtual methods are high-risk interface.
Struct fields in public headers are ABI/data contract.
Macros in public headers are public compile-time API.
Export macro or visibility attribute can mark external_public.
Extern C declaration can mark FFI public surface.
```

Useful C/C++ edges:

```text
header declares function
source defines function
source includes header
public function calls private implementation
test calls public function
target links library
```

C/C++ uncertainties:

```text
preprocessor conditionals
macro-generated declarations
template instantiations
compile flags
platform-specific exports
linker scripts
```

Codex should treat C/C++ public graph output as high value but compiler-verified.

The graph guides reads.

The compiler proves final truth.

### Python Extraction Rules

Python public surfaces include:

```text
module-level functions/classes without leading underscore
symbols listed in __all__
package exports in __init__.py
FastAPI routes
Flask routes
Django URL patterns and views
Click/Typer/argparse commands
Pydantic models used in APIs
dataclasses used in APIs
settings/config keys
```

Rules:

```text
If __all__ exists, treat listed names as package_public.
If __init__.py imports/reexports symbol, mark package_public.
If function/class lacks leading underscore, mark candidate public, then confirm consumers.
If route decorator exposes function, mark route external_public and handler implementation.
If Click/Typer command decorator exposes function, mark CLI external_public.
If Pydantic model is used in route input/output, mark API contract.
If Django urls.py maps path to view, mark route external_public.
```

Python uncertainties:

```text
dynamic imports
monkey patching
decorator side effects
framework convention magic
runtime route registration
reflection
```

Python publicness is more heuristic than Rust.

Confidence and evidence matter.

### HTTP Route Extraction Rules

Routes are public interfaces even when their handlers are private.

Route facts should include:

```text
method
path
handler
middleware if detectable
request schema
response schema
auth requirement if detectable
tests
OpenAPI contract if present
```

Route node:

```json
{
  "kind": "route",
  "name": "POST /users",
  "visibility": "external_public",
  "contract_kind": "http_route",
  "method": "POST",
  "path": "/users"
}
```

Common extraction patterns:

```text
Express/Fastify app.method(path, handler)
Express router.method(path, handler)
FastAPI @app.get/post decorators
Flask @app.route decorators
Django path/route declarations
Axum Router::route
Actix route macros
Rocket route macros
NestJS controller decorators
```

Route uncertainties:

```text
prefix composition
nested routers
middleware order
dynamic path construction
environment-dependent mounting
file-based routing conventions
```

Codex should see route graph confidence before changing API behavior.

### CLI Command Extraction Rules

CLI commands are public interfaces.

They deserve graph nodes because users and scripts depend on them.

Patterns:

```text
Rust clap derive and builder APIs
Python click/typer decorators
Python argparse parser.add_argument and subparsers
Node commander/yargs
Go cobra commands
shell scripts in bin/
package.json bin entries
Cargo binary targets
```

Command facts:

```text
command name
subcommand path
flags/options
handler function
input/output contract if documented
tests
docs
```

Example:

```json
{
  "kind": "cli_command",
  "name": "ptctx impact",
  "visibility": "external_public",
  "contract_kind": "cli",
  "owner_node": "rust:function:handle_impact_command"
}
```

Changing a CLI flag can be as breaking as changing an HTTP route.

Agents should treat it as public.

### Config Key Extraction Rules

Config keys are a hidden public interface.

They are public because deployment, scripts, docs, and users depend on them.

Patterns:

```text
process.env.NAME
std::env::var("NAME")
os.environ["NAME"]
dotenv/schema config
TOML/YAML/JSON config keys
feature flags
Cargo features
compile-time flags
```

Facts:

```text
function reads_config KEY
config schema defines KEY
docs document KEY
tests set KEY
route behavior depends_on_config KEY
```

Config node:

```json
{
  "kind": "config_key",
  "name": "TOKEN_TTL_SECONDS",
  "visibility": "external_public",
  "contract_kind": "environment_variable"
}
```

Config uncertainty:

```text
dynamic key construction
indirect config wrappers
environment-specific defaults
secret managers
```

Codex should inspect config edges before changing behavior that differs by
environment.

### Generated Contract Extraction Rules

Generated contracts should not be ignored.

Public generated-contract sources:

```text
OpenAPI specs
GraphQL schemas
protobuf files
Thrift files
JSON schema
database migration schemas
Prisma schemas
SQL migrations
generated clients
```

Rules:

```text
Spec file defines contract nodes.
Generated files generated_from spec.
Route handlers implement OpenAPI operations if matching method/path.
Client methods consume generated contract.
Tests cover contract behavior.
Manual edits to generated files should be flagged.
```

Important edge:

```text
generated_file generated_from schema
```

If Codex edits generated code directly, the broker should warn:

```text
This file appears generated. Edit the source contract instead.
```

### Public Interface Blast Radius

The public interface graph should compute blast radius differently from local
call graph.

Local impact:

```text
who calls this function?
```

Public impact:

```text
who consumes this exposed contract?
```

For example:

```text
Changing private helper:
  check direct callers and tests.

Changing exported TypeScript interface:
  check imports, route schemas, generated clients, tests.

Changing Rust pub struct field:
  check crate consumers, trait impls, serialization, tests.

Changing C header struct:
  check downstream compilation and ABI-sensitive uses.

Changing env var name:
  check config docs, deployment files, tests, runtime readers.
```

The public graph should rank risk higher for:

```text
external_public visibility
widely imported exports
routes with integration tests
public C/C++ headers
Rust pub traits/types
config keys documented in README or deploy files
generated contracts
```

### Public Interface Query Surface

The broker could expose this through existing commands:

```bash
ptctx impact --symbol "CreateUserRequest" --public --budget 4000
ptctx route --query "POST /users" --public --budget 4000
ptctx symbol --query "parse_config" --public --budget 3000
```

Or through a dedicated projection:

```bash
ptctx public-map --budget 3000
ptctx public-impact --query "CreateUserRequest" --budget 4000
```

For MVP, prefer flags over many commands.

But the projection should be explicit in the response:

```json
{
  "projection": "public_interface_graph",
  "answer": "CreateUserRequest is part of the POST /users request contract and is consumed by route tests and generated docs.",
  "public_nodes": [],
  "consumers": [],
  "tests": [],
  "risk": {
    "level": "high",
    "reason": "External route contract and generated OpenAPI schema are affected."
  }
}
```

### Codex Workflow

Before changing any likely public node, Codex should ask:

```text
Is this public?
Who consumes it?
What contract does it expose?
What tests cover it?
Is there generated/schema/doc surface?
```

Command:

```bash
ptctx impact --symbol "X" --public --budget 4000 --json
```

If public graph says high risk:

```text
Read the public contract.
Read the implementation.
Read at least one consumer.
Read relevant tests.
Avoid broad refactor unless requested.
```

Final answer should mention public surface if changed:

```text
This changes the public POST /users request contract, so I updated the handler
and ran the route-level test.
```

or:

```text
This stayed behind a private helper; no public interface nodes were affected.
```

That is the kind of confidence a user wants.

### MVP Extraction Rules

The first version can be smaller.

MVP TypeScript:

```text
exported declarations
package.json exports/bin
Express/Fastify route calls
```

MVP Rust:

```text
pub items
pub use
Cargo features
clap command derives if easy
axum route calls if easy
```

MVP C/C++:

```text
declarations in include/ and .h/.hpp
public class methods
extern C
source definition linked by same name
```

MVP Python:

```text
__all__
__init__.py reexports
FastAPI/Flask route decorators
Click/Typer command decorators
```

MVP config:

```text
literal env var reads
Cargo features
package scripts/bin
```

MVP generated:

```text
detect generated files
link obvious OpenAPI/proto/schema sources
warn before editing generated files
```

This is already valuable.

Do not wait for perfect extraction.

### Test Fixtures

The public graph needs fixtures.

Fixture 1: TypeScript package

```text
index.ts reexports createUser
routes/users.ts exposes POST /users
package.json exports dist/index.js
tests/users.test.ts hits route
```

Expected:

```text
createUser is package_public
POST /users is external_public
createUserHandler implements route
users.test.ts tests route
```

Fixture 2: Rust crate

```text
lib.rs pub mod parser
parser.rs pub fn parse_source_file
pub trait EntityExtractor
Cargo.toml feature experimental-parser
```

Expected:

```text
parse_source_file is package_public
EntityExtractor is package_public
feature flag is config/build public surface
```

Fixture 3: C library

```text
include/config.h declares parse_config
src/config.c defines parse_config
tests/config_test.c calls parse_config
```

Expected:

```text
parse_config declaration is package_public
definition implements declaration
test covers public function
```

Fixture 4: Python FastAPI

```text
app.py has @app.post("/users")
models.py has CreateUserRequest
__init__.py reexports client
tests/test_users.py hits route
```

Expected:

```text
POST /users is external_public
CreateUserRequest is route contract
test covers route
```

Fixture 5: Generated contract

```text
openapi.yaml defines POST /users
generated/client.ts generated from openapi.yaml
routes/users.ts implements POST /users
```

Expected:

```text
route implements OpenAPI operation
client generated_from OpenAPI
editing generated client warns
```

### Acceptance Criteria

Public interface graph MVP is done when:

| Criterion | Pass Condition |
|---|---|
| TypeScript exports | Direct exports and barrel reexports are detected. |
| TypeScript routes | Common route registrations become external_public route nodes. |
| Rust pub items | `pub` and `pub use` items become package_public nodes. |
| C/C++ headers | Declarations in public headers become package_public nodes. |
| Python routes/exports | `__all__`, reexports, and common route decorators are detected. |
| Config keys | Literal env/config reads become config_key nodes. |
| Generated files | Generated files are detected and linked to obvious source contracts. |
| Public impact | Broker can answer whether a symbol touches public surface. |
| Evidence | Every public node has file/span/source evidence. |
| Confidence | Dynamic/framework uncertainty lowers confidence. |
| Freshness | Public graph facts carry index freshness. |

### Failure Modes

Public graph failure modes:

```text
Marks too much as public and creates noise.
Misses framework routes and hides external impact.
Treats semantic search hits as consumers.
Ignores generated contracts.
Ignores config keys.
Cannot distinguish pub(crate) from external public.
Cannot link header declaration to source definition.
Fails to expose confidence on dynamic patterns.
```

Mitigation:

```text
Start conservative.
Expose confidence.
Use raw evidence.
Allow language-specific query packs.
Record unknown visibility rather than forcing public/private.
```

### Why This Is High PMF

For the user's personal workflow, public interface graph has high PMF because it
answers the scary question:

```text
Am I about to change a contract that something else depends on?
```

That question appears in:

```text
CRUD apps
Rust libraries
C/C++ systems code
CLI tools
config-heavy services
generated API clients
```

It is also exactly where Codex benefits from a graph.

Codex can find files with `rg`.

Codex cannot reliably infer public blast radius from a few file reads.

That is the product gap.

### Concept 15 Conclusion

The public interface graph should be the first high-value projection because it
turns a large-codebase fear into a concrete answer:

```text
what surface is exposed, who consumes it, what tests cover it, and how risky is
the change?
```

Extraction should start with:

```text
TypeScript exports and routes
Rust pub items and features
C/C++ public headers
Python exports and routes
CLI commands
config keys
generated contracts
```

The graph does not need perfect omniscience.

It needs evidence, confidence, freshness, and honest uncertainty.

That is enough to make Codex safer before edits.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| Concept 14 in this document | Source-read through current synthesis | Supplies universal relationship schema and public interface projection rationale. |
| Concept 13 in this document | Source-read through current synthesis | Supplies Codex rituals that should call public impact before risky edits. |
| Concept 12 in this document | Source-read through current synthesis | Supplies broker command surface where public projection can be exposed. |
| `README.md` | Source-read | Supplies original blast-radius and smart-context workflow that public impact refines. |
| `docs/research001/unclassified/ISGL1-v2-Stable-Entity-Identity.md` | Source-read | Supplies stable identity requirements for public node durability. |

## Concept 16: Benchmark The Public Interface Graph Before Trusting It

### The Point

The public interface graph must be benchmarked before Codex relies on it.

Otherwise it will fail in one of two bad ways.

Bad failure 1:

```text
It marks everything as public.
```

Codex becomes scared of every edit.

Bad failure 2:

```text
It misses real public contracts.
```

Codex edits shared surfaces blindly.

Both are harmful.

The benchmark must measure:

```text
Can the tool identify public surfaces?
Can it avoid private false positives?
Can it link public surfaces to implementation?
Can it link implementation to consumers/tests?
Can it explain confidence and uncertainty?
Can it help Codex choose the right next action?
```

The public graph is not useful because it is pretty.

It is useful if it changes Codex behavior before risky edits.

### Benchmark Principle

The benchmark should test decisions, not just extraction.

Extraction question:

```text
Did the graph find `POST /users`?
```

Decision question:

```text
If Codex changes `CreateUserRequest`, does it know this affects the public
`POST /users` contract and which tests to run?
```

The second question is the product.

The benchmark should include both.

### Benchmark Directory Shape

Proposed structure:

```text
bench/public-interface/
  fixtures/
    ts-package-routes/
    ts-barrel-exports/
    rust-crate-pub/
    rust-axum-routes/
    cpp-public-headers/
    python-fastapi-package/
    config-env-keys/
    generated-openapi/
    mixed-public-impact/
  cases/
    ts_route_public_001.yaml
    ts_private_helper_negative_001.yaml
    rust_pub_reexport_001.yaml
    cpp_header_public_001.yaml
    python_all_export_001.yaml
    config_key_public_001.yaml
    generated_contract_001.yaml
    codex_decision_public_impact_001.yaml
  runs/
  reports/
```

Fixtures are tiny repos.

Cases describe expected graph facts and Codex decisions.

Runs store raw outputs.

Reports compare tools and broker versions.

### Case Manifest

Each case should be a YAML file.

Example:

```yaml
id: ts_route_public_001
fixture: fixtures/ts-package-routes
intent: public_interface_extraction
query:
  command: public-impact
  args:
    symbol: CreateUserRequest
    budget: 3000
must_include_nodes:
  - kind: route
    name: POST /users
    visibility: external_public
  - kind: type
    name: CreateUserRequest
    visibility: package_public
must_include_edges:
  - from: POST /users
    kind: routes_to
    to: createUserHandler
  - from: createUserHandler
    kind: accepts_type
    to: CreateUserRequest
  - from: createUser.test.ts
    kind: tests
    to: POST /users
must_not_include_nodes:
  - kind: symbol
    name: privateFormatUserForTest
    visibility: external_public
expected_decision:
  risk_level: high
  next_reads:
    - src/routes/users.ts
    - src/handlers/users.ts
    - tests/users/createUser.test.ts
  next_tests:
    - npm test -- tests/users/createUser.test.ts
```

This is readable by humans and executable by a grader.

### Scoring Dimensions

Use a 100-point score for each case.

| Dimension | Points | What It Measures |
|---|---:|---|
| Public node recall | 20 | Finds the required public surfaces. |
| Public node precision | 15 | Does not mark private/internal nodes as public. |
| Edge correctness | 20 | Links public surface to implementation and consumers. |
| Evidence quality | 10 | Supplies file/span/source evidence. |
| Confidence/freshness | 10 | Reports uncertainty and freshness honestly. |
| Decision usefulness | 15 | Gives Codex correct next reads/tests/risk. |
| Token efficiency | 10 | Fits budget without dumping irrelevant graph. |

This is stricter than "did it find something?"

It rewards useful behavior.

### False-Positive Controls

False positives are dangerous.

The benchmark should include traps.

Trap examples:

```text
private helper with same name as public API
test-only exported helper
example/demo file route
mock route in fixture
internal route under admin/dev-only path
unexported Rust item with public-looking name
pub(crate) item incorrectly marked external_public
C++ private class method in public header
generated file edited directly instead of source schema
semantic search result treated as dependency
dynamic route with unresolved path marked as certain
```

A good public graph should say:

```text
candidate
private
module_public
unknown
```

instead of forcing everything into `external_public`.

### False-Negative Controls

False negatives are equally dangerous.

The benchmark should include surfaces that are easy to miss.

Examples:

```text
TypeScript barrel reexport from index.ts
package.json exports path
package.json bin command
Rust pub use reexport
Rust pub trait method
C header declaration with source definition elsewhere
Python __all__
FastAPI decorator route
Django URL pattern
clap subcommand
env var read through config wrapper
OpenAPI operation implemented by route handler
protobuf message used by generated client
```

If a tool misses these, it may still be useful, but Codex should not treat it as
complete public impact.

### Fixture 1: TypeScript Route Package

Purpose:

```text
Test exported types, route surface, handler implementation, and tests.
```

Shape:

```text
package.json
src/index.ts
src/routes/users.ts
src/handlers/users.ts
src/types/users.ts
src/services/users.ts
tests/users/createUser.test.ts
```

Important code facts:

```text
src/index.ts exports CreateUserRequest
src/routes/users.ts registers POST /users
route calls createUserHandler
handler accepts CreateUserRequest
handler calls createUserService
test hits POST /users
private helper exists but is not public
```

Expected public nodes:

```text
CreateUserRequest package_public
POST /users external_public
createUserHandler implementation_of public route
```

Expected non-public:

```text
formatUserForDb private
privateFormatUserForTest private or test-only
```

### Fixture 2: TypeScript Barrel Exports

Purpose:

```text
Test reexports and package boundary.
```

Shape:

```text
src/user/createUser.ts
src/user/UserClient.ts
src/internal/cache.ts
src/index.ts
package.json
```

Expected:

```text
createUser package_public via index.ts
UserClient package_public via index.ts
internalCache private
package.json exports src/index.ts public surface
```

Important edge:

```text
index.ts reexports createUser
```

False-positive trap:

```text
src/internal/cache.ts exports for internal import but is not package public.
```

### Fixture 3: Rust Crate Public API

Purpose:

```text
Test `pub`, `pub(crate)`, `pub use`, traits, and feature flags.
```

Shape:

```text
Cargo.toml
src/lib.rs
src/parser/mod.rs
src/parser/engine.rs
src/internal/cache.rs
tests/parser_api.rs
```

Expected:

```text
parse_source_file package_public
ParserOptions package_public
EntityExtractor trait package_public
pub(crate) cache workspace_public or crate_public, not external_public
experimental-parser feature config/build public surface
parser_api.rs tests parse_source_file
```

False-positive trap:

```text
pub(crate) fn rebuild_cache should not be external_public.
```

### Fixture 4: Rust Axum Route

Purpose:

```text
Test HTTP route extraction in Rust.
```

Shape:

```text
src/main.rs
src/routes/users.rs
src/handlers/users.rs
src/services/users.rs
tests/users_route.rs
```

Expected:

```text
POST /users external_public
create_user_handler implementation node
handler calls service
test covers route
```

Uncertainty:

```text
middleware order may be unknown unless explicitly parsed.
```

The expected output should allow:

```text
routes_to high confidence
middleware order low or unknown confidence
```

### Fixture 5: C/C++ Public Header

Purpose:

```text
Test public header declarations and source definitions.
```

Shape:

```text
include/config.h
src/config.c
src/internal_config.c
tests/config_test.c
CMakeLists.txt
```

Expected:

```text
parse_config package_public from include/config.h
Config struct package_public
parse_config definition in src/config.c implements declaration
tests/config_test.c tests parse_config
internal_parse_config private
```

False-positive trap:

```text
static helper in src/internal_config.c must not be public.
```

### Fixture 6: C++ Class Header

Purpose:

```text
Test public/protected/private methods and virtual interface risk.
```

Shape:

```text
include/parser/Parser.h
src/Parser.cpp
tests/ParserTest.cpp
```

Expected:

```text
Parser class package_public
Parser::parse public method package_public
Parser::reset public method package_public
Parser::parseInternal private
Parser::onError protected subclass interface
```

Risk:

```text
virtual protected method is a subclass contract.
```

Codex should not treat it like a private helper.

### Fixture 7: Python FastAPI Package

Purpose:

```text
Test Python package exports and route decorators.
```

Shape:

```text
pyproject.toml
app/__init__.py
app/main.py
app/models.py
app/services/users.py
tests/test_users.py
```

Expected:

```text
POST /users external_public
create_user route handler implementation
CreateUserRequest API contract
UserClient package_public if reexported from __init__.py
_normalize_user private
test_users.py tests route
```

Uncertainty:

```text
decorator side effects should not be overclaimed.
```

### Fixture 8: Config Keys

Purpose:

```text
Test public config surface.
```

Shape:

```text
src/config.ts
src/auth/tokenService.ts
.env.example
README.md
tests/tokenExpiry.test.ts
```

Expected:

```text
TOKEN_TTL_SECONDS config_key external_public
tokenService reads_config TOKEN_TTL_SECONDS
.env.example documents TOKEN_TTL_SECONDS
README.md documents TOKEN_TTL_SECONDS
tokenExpiry.test.ts tests behavior influenced by TOKEN_TTL_SECONDS
```

False-negative trap:

```text
Config keys are often missed because they are strings, not symbols.
```

### Fixture 9: Generated OpenAPI Contract

Purpose:

```text
Test generated contract linkage and generated-file warning.
```

Shape:

```text
openapi.yaml
src/routes/users.ts
src/generated/client.ts
tests/users-contract.test.ts
```

Expected:

```text
openapi.yaml defines POST /users contract
route implements POST /users
generated/client.ts generated_from openapi.yaml
users-contract.test.ts tests contract
editing generated/client.ts should warn
```

Decision task:

```text
If user asks to change request schema, Codex should edit openapi.yaml and route
model, not generated client only.
```

### Fixture 10: Mixed Public Impact

Purpose:

```text
Test cross-surface blast radius.
```

Shape:

```text
TypeScript route
shared type
config key
generated OpenAPI
test
```

Task:

```text
Change CreateUserRequest to require displayName.
```

Expected Codex decision:

```text
read type definition
read route handler
read validator
read OpenAPI schema
read integration test
run route test
warn public contract changed
```

This is the benchmark that best matches the real product job.

### Codex Decision Tasks

Each fixture should have at least one decision task.

Decision task format:

```yaml
id: codex_decision_public_impact_001
fixture: fixtures/mixed-public-impact
user_request: "Make displayName required when creating a user."
codex_question: "What should I read and test before editing?"
expected_answer:
  must_say:
    - public POST /users contract is affected
    - CreateUserRequest is public or route contract
    - read validator and route handler
    - run createUser route test
  must_not_say:
    - only edit generated client
    - no public interface affected
```

The grader should check:

```text
risk level
next reads
next tests
public surface mention
wrong advice absence
```

This makes the benchmark agent-centered.

### Precision And Recall

The public graph needs both.

Recall:

```text
Did it find the public surface?
```

Precision:

```text
Did it avoid marking private things as public?
```

Edge recall:

```text
Did it link surface to implementation and tests?
```

Edge precision:

```text
Did it avoid fake consumer/test links?
```

Suggested minimum thresholds:

| Metric | MVP Threshold | Strong Threshold |
|---|---:|---:|
| Public node recall | 0.80 | 0.92 |
| Public node precision | 0.85 | 0.95 |
| Edge recall | 0.70 | 0.88 |
| Edge precision | 0.80 | 0.93 |
| Decision usefulness | 0.80 | 0.92 |
| False high-confidence errors | 0 | 0 |

The last row matters most.

It is better to say:

```text
unknown
```

than to be confidently wrong.

### False High-Confidence Error Rule

The benchmark should penalize confident wrong answers heavily.

Examples:

```text
Marks private helper as external_public with 0.9 confidence.
Says no public interface affected when route contract changed.
Claims generated client is source of truth.
Claims test covers route when it only imports a mock.
Claims pub(crate) Rust item is external public.
```

Penalty:

```text
case score cannot exceed 60 if there is a false high-confidence publicness claim
case score cannot exceed 50 if it misses an external public route
case score cannot exceed 50 if decision advice would make Codex edit wrong file
```

This makes the benchmark reflect real risk.

### Token Budget Cases

Each case should run at multiple budgets:

```text
1000 tokens
3000 tokens
6000 tokens
```

Expected behavior:

```text
1000: public risk, top reads, top test
3000: include key edges and evidence
6000: include secondary consumers and uncertainty
```

If a tool cannot compress, it should lose points even if raw extraction is good.

Codex does not need a giant graph dump.

Codex needs a decision packet.

### Freshness Cases

Add stale-index tests.

Case:

```text
Build graph.
Modify route file.
Run public-impact without refresh.
```

Expected:

```text
freshness status stale
dirty file listed
public impact claims downgraded
recommend refresh or raw verification
```

Failure:

```text
tool returns confident old public graph
```

This case directly tests the incremental-indexing lesson.

### Real Repo Sanity Cases

After fixture tests, use small real-repo sanity checks from the cloned candidates.

Candidates:

```text
sdsrss/code-graph-mcp
code-review-graph
cocoindex-code
clarity-cli
codemogger
ast-grep
semgrep
```

Real repo tests should be lighter because ground truth is harder.

They should ask:

```text
Can the tool identify CLI commands?
Can it identify public package exports?
Can it identify route-like entry points if present?
Can it avoid marking tests/examples as public API?
Can it produce useful next reads?
```

Do not overfit to real repos without golden truth.

Use them as sanity, not primary grading.

### Report Format

The report should be easy to read.

Example:

```markdown
# Public Interface Graph Benchmark Report

| Tool Stack | Node Recall | Node Precision | Edge Recall | Edge Precision | Decision | Tokens | Score |
|---|---:|---:|---:|---:|---:|---:|---:|
| rg baseline | 0.32 | 0.70 | 0.18 | 0.60 | 0.41 | 1800 | 39 |
| graph MCP | 0.78 | 0.83 | 0.66 | 0.80 | 0.74 | 2400 | 76 |
| broker default | 0.86 | 0.90 | 0.76 | 0.86 | 0.84 | 2100 | 84 |

## Highest Risk Failures

- graph MCP missed package.json bin command in ts-barrel-exports.
- broker marked pub(crate) item as workspace_public correctly, not external_public.
- rg baseline found route strings but failed to link handler to tests.
```

The report should name failures before celebrating wins.

### Tool Comparison

Run the benchmark against:

```text
Codex plus rg/git baseline
graph tool alone
search tool alone
review graph alone
broker default stack
broker with Parceltongue core if available
```

This shows whether the broker adds value.

If broker default does not beat the best individual tool, the broker is not yet
worth using.

If broker default wins by merging search, graph, and review evidence, that is
strong evidence for Path B from Concept 11.

### Public Graph MVP Exit Criteria

Do not call public interface graph MVP done until:

```text
all fixture cases pass strong or acceptable thresholds
no false high-confidence publicness errors remain
stale index case downgrades confidence
decision tasks produce correct next reads/tests
report compares against rg baseline
Codex can use output without manual translation
```

This is stricter than "the parser finds exports."

It should be.

The user is hiring the graph to prevent risky edits.

### Concept 16 Conclusion

The public interface graph must be benchmarked around Codex decisions.

The benchmark should cover:

```text
TypeScript exports and routes
Rust pub/reexports/features
C/C++ public headers
Python exports/routes
config keys
generated contracts
false positives
false negatives
stale indexes
token budgets
Codex next-action quality
```

The most important rule:

```text
No confident wrong publicness claims.
```

If the graph is uncertain, it should say uncertain.

That honesty is what lets Codex use it safely.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| Concept 15 in this document | Source-read through current synthesis | Supplies public interface extraction rules that the benchmark validates. |
| Concept 14 in this document | Source-read through current synthesis | Supplies relationship schema, confidence, freshness, and projection concepts. |
| Concept 10 in this document | Source-read through current synthesis | Supplies DQPT-style evaluation philosophy and benchmark shape. |
| `docs/research001/unclassified/RCA-Incremental-Indexing-Failure.md` | Source-read | Supplies stale-index and freshness regression cases. |
| `docs/research001/unclassified/ISGL1-v2-Stable-Entity-Identity.md` | Source-read | Supplies line-shift/stable-identity cases that public graph facts must survive. |

## Concept 17: Build Only The Minimal Parceltongue v2 Core If The Broker Exposes Graph Gaps

### The Point

Parceltongue v2 should not begin as a full rewrite.

It should begin only if the broker benchmark proves a specific gap:

```text
existing tools cannot provide fresh, stable, provenance-rich relationship facts
that Codex can use for public impact and dependency decisions
```

If that gap is proven, Parceltongue v2 should be small.

It should not try to replace:

```text
Semgrep
CodeQL
ast-grep
cocoindex-code
codemogger
Clarity
Glean
Kythe
SCIP
```

It should provide the missing core:

```text
stable identity
incremental freshness
safe parser lifecycle
relationship facts
public interface projection
Codex-ready query responses
```

That is enough.

### Build Trigger

Do not build the v2 core unless at least one benchmark suite proves a hard gap.

Valid triggers:

```text
Public graph benchmark misses key surfaces across multiple languages.
Existing graph tools cannot report freshness after edits.
Existing graph tools use unstable line-based identities.
Existing tools cannot produce edge evidence with file/span/source.
Existing tools cannot distinguish public/private/unknown visibility.
Existing tools return too many tokens for routine agent decisions.
Existing tools cannot run reliably in the user's local Codex workflow.
```

Invalid triggers:

```text
I dislike the implementation style of existing tools.
I want to own the whole stack.
It would be fun to build.
The repo already exists, so we should keep going.
One tool missed one edge in one fixture.
```

This is the discipline.

Parceltongue should evolve only where it provides leverage.

### Minimal Core Promise

The v2 core promise should be:

```text
Given a local repo, maintain a fresh relationship fact graph that lets Codex
ask small questions about symbols, public interfaces, impact, and tests with
evidence and confidence.
```

Not:

```text
Understand all code perfectly.
```

Not:

```text
Be a universal static analysis platform.
```

Not:

```text
Replace the coding agent.
```

The user already chose Codex.

Parceltongue v2 is a context core under Codex.

### Minimal Module Boundary

The core can be split into six modules.

| Module | Responsibility |
|---|---|
| `identity` | Stable node and edge identity across edits. |
| `parser` | Safe Tree-sitter parsing with lifecycle, timeouts, and syntax diagnostics. |
| `query_packs` | Language-specific extraction rules for nodes and relationship facts. |
| `index` | Incremental file hashing, dirty detection, graph updates, freshness. |
| `facts` | Storage schema for nodes, edges, evidence, confidence, freshness. |
| `projections` | Public graph, symbol context, impact, route/test views. |

Everything else is optional.

The broker can sit above these modules.

### Data Model

Minimal tables or collections:

```text
files
nodes
edges
evidence
index_runs
diagnostics
```

`files`:

```text
file_id
path
language
content_hash
size_bytes
last_indexed_at
parse_status
generated_flag
ignored_flag
```

`nodes`:

```text
stable_id
kind
name
semantic_path
visibility
language
file_id
current_span
content_hash
birth_id
first_seen_at
last_seen_at
freshness_status
```

`edges`:

```text
edge_id
from_stable_id
to_stable_id
kind
directness
visibility
confidence
confidence_reason
source_tool
last_seen_at
freshness_status
```

`evidence`:

```text
evidence_id
fact_id
file_id
span
snippet_hash
detection_method
query_pack
raw_kind
```

`index_runs`:

```text
run_id
started_at
finished_at
git_commit
dirty_files
files_scanned
files_changed
files_failed
diagnostics
```

`diagnostics`:

```text
diagnostic_id
file_id
severity
kind
message
source
span
```

This schema is boring on purpose.

It is easy for Codex to reason about.

### Stable Identity

Stable identity is the first real feature.

Without it, incremental indexing is fake.

Node identity should separate:

```text
what the entity is
where it currently lives
what its current content is
```

A node should keep the same stable id when:

```text
lines shift
comments are inserted above it
nearby code changes
the entity body changes but semantic path remains
```

A node should get a new stable id when:

```text
the entity is truly new
the semantic role changes enough that matching would be misleading
two ambiguous candidates cannot be safely matched
```

Matching order:

```text
1. Exact stable id from previous index if span still valid.
2. Same semantic path plus same content hash.
3. Same semantic path plus nearest old position.
4. Same parent plus same name/kind with changed content.
5. New birth id.
```

Edge identity:

```text
edge_id = hash(from_stable_id + kind + to_stable_id + scope)
```

This lets Parceltongue say:

```text
same public route, handler body changed
same function, line shifted
new call edge added
old test edge removed
```

That is the foundation of trustworthy freshness.

### Incremental Freshness

Incremental indexing should be built around file hashes first.

Pipeline:

```text
1. Read git and filesystem state.
2. Compute hashes for tracked source/config files.
3. Skip unchanged files.
4. Reparse changed files safely.
5. Extract candidate nodes and edges.
6. Match nodes to previous stable ids.
7. Replace facts for changed files transactionally.
8. Mark dependent edges stale if their endpoints changed.
9. Report freshness status for queries.
```

Important rule:

```text
Every query response must expose freshness.
```

No graph answer should be allowed to hide stale state.

Freshness statuses:

```text
fresh
stale
partial
unknown
not_indexed
```

Query response example:

```json
{
  "freshness": {
    "status": "partial",
    "reason": "3 changed files were reindexed, but route projection has not been recomputed.",
    "dirty_files": ["src/routes/users.ts"],
    "unsafe_for": ["public-impact"]
  }
}
```

This turns the incremental-indexing RCA into product behavior.

### Parser Safety

Parser safety is not an implementation detail.

It is part of product trust.

The parser module should enforce:

```text
max file size
parse timeout
max AST depth
syntax error diagnostics
unsupported language diagnostics
generated/vendor skip rules
parser reset after failure
partial extraction allowed only with lowered confidence
```

A failed parse should produce:

```text
diagnostic
file freshness partial or failed
query confidence downgrade
```

It should not produce:

```text
silent empty graph
confident no-impact answer
crash
hang
```

This is where Concept 7 matters.

### Query Packs

Query packs are the product surface for languages.

Minimal first packs:

```text
typescript
rust
c
cpp
python
```

Each pack should define:

```text
node extraction
edge extraction
public interface extraction
route extraction if applicable
test extraction if applicable
config extraction
confidence defaults
known limitations
fixture coverage
```

Example pack metadata:

```json
{
  "language": "typescript",
  "version": "pt.querypack.ts.v1",
  "supports": [
    "exports",
    "imports",
    "function_calls",
    "express_routes",
    "config_env_reads",
    "tests_basic"
  ],
  "limitations": [
    "computed route paths are low confidence",
    "decorator-heavy frameworks are partial"
  ]
}
```

Codex should be able to ask:

```text
How much does this query pack support?
```

before trusting it.

### Relationship Facts

The core should store relationship facts from Concept 14.

Minimal node kinds:

```text
file
module
symbol
route
test
config_key
data_entity
package
```

Minimal edge kinds:

```text
defines
imports
exports
reexports
calls
routes_to
uses_type
reads_config
reads
writes
tests
depends_on_package
candidate_relevance
```

Every edge needs:

```text
evidence
confidence
freshness
source query pack
raw kind if adapter-specific
```

The core should be humble:

```text
If it cannot prove a call edge, it should not call it a call edge.
```

It can store weaker facts as:

```text
candidate_relevance
inferred_relationship
unknown
```

That honesty is more valuable than fake completeness.

### Public Graph Projection

The first projection should be:

```text
public_interface_graph
```

Inputs:

```text
nodes
edges
visibility
package boundaries
routes
CLI commands
config keys
generated contracts
tests
```

Outputs:

```text
public nodes
implementation nodes
consumer nodes
test nodes
risk score
uncertainty
recommended next reads
recommended next tests
```

Query:

```bash
ptctx impact --symbol "CreateUserRequest" --public --budget 4000
```

Core projection result:

```json
{
  "projection": "public_interface_graph",
  "answer": "CreateUserRequest is part of the POST /users public route contract.",
  "risk": {
    "level": "high",
    "reasons": [
      "external_public route contract",
      "route-level test exists",
      "OpenAPI schema may need update"
    ]
  },
  "recommended_next_reads": [
    "src/types/users.ts",
    "src/routes/users.ts",
    "src/handlers/users.ts",
    "tests/users/createUser.test.ts"
  ]
}
```

This should be the flagship output.

### Query API

If v2 core exists under the broker, it should expose a small internal API.

Not 40 endpoints.

Start with:

```text
index_health
project_map
search_symbols
symbol_context
impact_context
public_impact
route_trace
review_diff
tests_for
```

These map cleanly to the broker commands.

The broker can handle:

```text
tool routing
fallbacks
external adapters
MCP later
```

Parceltongue core handles:

```text
fresh facts
stable ids
projections
```

Do not leak internal storage details into Codex responses.

### Storage Choice

Use boring storage first.

Likely options:

```text
SQLite
DuckDB
sled/redb for embedded Rust
existing repo DB if already present
```

Recommendation:

```text
SQLite first unless there is a strong Rust-native reason not to.
```

Reasons:

```text
easy inspection
transactional updates
good enough graph queries for MVP
portable
Codex can inspect it if needed
```

Do not start with a complex graph database.

The bottleneck is relationship correctness, not graph database glamour.

### TDD Plan

Build the v2 core test-first.

Test groups:

```text
identity tests
incremental freshness tests
parser safety tests
query pack extraction tests
relationship fact tests
public graph projection tests
benchmark harness tests
```

Identity tests:

```text
line shift preserves stable id
body edit preserves stable id and updates content hash
rename creates new semantic path or records rename if supported
duplicate symbols handled deterministically
```

Freshness tests:

```text
unchanged file skipped
changed file reindexed
deleted file facts removed
dirty file marks query stale
partial parse downgrades confidence
```

Parser tests:

```text
large file skipped
timeout produces diagnostic
syntax error returns partial diagnostic
unsupported language reported
parser reset after failure
```

Public graph tests:

```text
TypeScript route public
Rust pub item public
C header declaration public
Python FastAPI route public
config key public
generated file warning
private helper not public
```

The benchmark from Concept 16 becomes the acceptance suite.

### Build Phases

#### Phase 1: Identity And Files

Deliver:

```text
file discovery
file hashing
stable node identity type
file table
node table
identity matching tests
```

Exit:

```text
line-shift test passes
```

#### Phase 2: Safe Parser Pipeline

Deliver:

```text
Tree-sitter parser wrapper
timeouts
file limits
diagnostics
partial parse behavior
```

Exit:

```text
bad syntax and timeout tests pass
```

#### Phase 3: First Query Pack

Pick one language.

Recommended:

```text
TypeScript
```

Reason:

```text
CRUD routes and exports show public graph value quickly.
```

Deliver:

```text
exports
imports
functions/classes/types
basic call edges
Express/Fastify routes
env var reads
tests by filename/import
```

Exit:

```text
TypeScript public graph fixture passes
```

#### Phase 4: Facts And Projections

Deliver:

```text
edge table
evidence table
confidence
freshness
symbol_context
public_impact
```

Exit:

```text
public-impact answer includes evidence, risk, next reads, next tests
```

#### Phase 5: Add Rust

Deliver:

```text
pub items
pub use
traits
impls if feasible
Cargo features
basic call edges
```

Exit:

```text
Rust public graph fixture passes
```

#### Phase 6: Add C/C++ Header Support

Deliver:

```text
public header declarations
source definitions
class public/protected/private methods
extern C
tests by call/import
```

Exit:

```text
C/C++ public header fixtures pass
```

#### Phase 7: Add Python Routes/Exports

Deliver:

```text
__all__
__init__ reexports
FastAPI/Flask routes
Click/Typer commands if feasible
env var reads
```

Exit:

```text
Python public graph fixture passes
```

#### Phase 8: Broker Integration

Deliver:

```text
ptctx adapter for Parceltongue core
health integration
freshness surfaced
public-impact command
benchmark comparison
```

Exit:

```text
broker default with Parceltongue core beats rg baseline and external tool stack
on the target benchmark.
```

### What Not To Build In v2 Core

Do not build:

```text
semantic embeddings
chat interface
visual graph UI
multi-user server
cloud sync
security rule engine
general code rewrite engine
full LSP replacement
every language
unbounded endpoint surface
```

Use existing tools for those.

The core wins by being fresh, stable, small, and trustworthy.

### Acceptance Criteria

Minimal v2 core is useful when:

```text
line shift preserves node identity
incremental reindex updates changed facts only
every query exposes freshness
parser failures produce diagnostics, not lies
relationship facts have evidence
public graph projection passes benchmark
Codex receives answer/evidence/next reads/next tests
broker can call it as one adapter
```

If any of these fail, do not call it ready.

### Shreyas Product Read

This is the smallest version with a real job.

The job is:

```text
Prevent Codex from making large-codebase changes with stale or incomplete
relationship context.
```

The MVP is not:

```text
parse everything
```

The MVP is:

```text
make public impact trustworthy for the user's real languages
```

That is narrow enough to build and broad enough to matter.

### Concept 17 Conclusion

If the broker benchmark proves existing tools are insufficient, Parceltongue v2
should evolve as a minimal core:

```text
stable identity
incremental freshness
parser safety
query packs
relationship facts
public graph projection
broker integration
```

The build order should be:

```text
identity
freshness
parser safety
TypeScript public graph
relationship facts
Rust
C/C++
Python
broker integration
benchmark proof
```

The discipline:

```text
Only build the missing core.
Let existing tools keep doing what they already do well.
```

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| Concept 16 in this document | Source-read through current synthesis | Supplies benchmark trigger and public graph acceptance criteria. |
| Concept 15 in this document | Source-read through current synthesis | Supplies public graph extraction rules. |
| Concept 14 in this document | Source-read through current synthesis | Supplies relationship fact schema. |
| Concept 12 in this document | Source-read through current synthesis | Supplies broker placement and command surface. |
| Concept 7 in this document | Source-read through current synthesis | Supplies parser safety and lifecycle requirements. |
| `docs/research001/unclassified/ISGL1-v2-Stable-Entity-Identity.md` | Source-read | Supplies stable identity architecture and line-shift requirements. |
| `docs/research001/unclassified/RCA-Incremental-Indexing-Failure.md` | Source-read | Supplies incremental freshness failure mode and root-cause constraints. |

## Concept 18: Start With A TypeScript Public Graph MVP

### The Point

If Parceltongue v2 core is built, the first implementation slice should be
TypeScript public graph MVP.

Not all languages.

Not all graph projections.

Not a full typechecker.

Just:

```text
TypeScript package exports
TypeScript route surfaces
TypeScript public types
TypeScript config keys
TypeScript tests
public impact query
```

This gives the fastest proof because CRUD apps expose public surfaces in a way
agents often need:

```text
routes
handlers
request/response types
services
config
tests
package exports
```

If this slice is not useful, the larger project is suspect.

If this slice is useful, it becomes the reference path for Rust, C/C++, and
Python.

### MVP Goal

The MVP should answer one question well:

```text
If Codex changes this TypeScript symbol or file, does it affect a public route,
package export, CLI command, config key, generated contract, or focused test?
```

The flagship command:

```bash
ptctx public-impact --query "CreateUserRequest" --budget 3000 --json
```

or through the general command:

```bash
ptctx impact --query "CreateUserRequest" --public --budget 3000 --json
```

Expected answer:

```text
CreateUserRequest is part of the POST /users public route contract. Read the
type, route, handler, validator, and route test before editing.
```

This is narrow and valuable.

### Explicit Non-Goals

The first TypeScript MVP should not attempt:

```text
full TypeScript type checking
full module resolution
decorator-heavy framework support
all Next/Remix/Nest conventions
full dataflow
perfect dynamic import handling
full monorepo package graph
semantic embeddings
automatic code edits
```

It should use Tree-sitter plus simple package metadata.

If exact typechecker support becomes necessary later, add it deliberately.

Do not begin there.

### Fixture First

Build the fixture before the implementation.

Proposed fixture:

```text
bench/public-interface/fixtures/ts-public-graph-mvp/
  package.json
  tsconfig.json
  src/
    index.ts
    routes/
      users.ts
    handlers/
      users.ts
    services/
      users.ts
    types/
      users.ts
    config/
      env.ts
    generated/
      client.ts
  tests/
    users.create.test.ts
    users.service.test.ts
  openapi.yaml
```

Key fixture facts:

```text
package.json exports src/index.ts
src/index.ts reexports CreateUserRequest and createUser
src/routes/users.ts registers POST /users
route calls createUserHandler
handler accepts CreateUserRequest
handler calls createUserService
service reads USER_SIGNUP_ENABLED
openapi.yaml defines POST /users
generated/client.ts is generated from openapi.yaml
users.create.test.ts tests POST /users
users.service.test.ts tests createUserService
private helper is not public
```

The fixture should contain traps:

```text
exported test helper that should be test-only, not package public
internal file exporting a helper not exposed by package.json or index.ts
computed route path with lower confidence
generated client file that should warn if edited
config key read as literal string
```

### First Benchmark Case

Case:

```yaml
id: ts_public_graph_mvp_001
fixture: fixtures/ts-public-graph-mvp
query:
  command: public-impact
  args:
    query: CreateUserRequest
    budget: 3000
must_include_nodes:
  - kind: type
    name: CreateUserRequest
    visibility: package_public
  - kind: route
    name: POST /users
    visibility: external_public
  - kind: test
    name: users.create.test.ts
must_include_edges:
  - from: src/index.ts
    kind: reexports
    to: CreateUserRequest
  - from: POST /users
    kind: routes_to
    to: createUserHandler
  - from: createUserHandler
    kind: accepts_type
    to: CreateUserRequest
  - from: users.create.test.ts
    kind: tests
    to: POST /users
must_not_include_nodes:
  - name: formatUserForDb
    visibility: external_public
expected_decision:
  risk_level: high
  must_say:
    - public route contract affected
    - read route and handler
    - run users.create.test.ts
```

This is the acceptance story.

### Package Boundary Rules

Read `package.json`.

Extract:

```text
name
main
module
types
exports
bin
files
scripts
```

Rules:

```text
If package.json exports points to a file, mark that file as package boundary.
If package.json bin points to a file, mark CLI command surface.
If no exports field exists, fallback to main/module/types/index.ts.
If src/index.ts reexports symbol, mark package_public.
If file is under tests, examples, or fixtures, do not mark package_public unless package config exposes it.
```

This keeps internal exports from becoming false public APIs.

Important distinction:

```text
TypeScript `export` means module-public.
Package public means exported through package boundary.
```

The MVP must not collapse these.

### Export Extraction Rules

Detect:

```text
export function foo
export class Foo
export interface Foo
export type Foo
export const FOO
export default Foo
export { Foo } from "./foo"
export * from "./foo"
```

Facts:

```text
file defines symbol
file exports symbol
index reexports symbol
package exposes symbol if boundary file exports/reexports it
```

Visibility:

```text
direct export from internal file: module_public
reexport from package boundary: package_public
exported symbol in test file: test_public or module_public, not package_public
unexported declaration: private
```

This one distinction prevents a large class of false positives.

### Import Extraction Rules

Detect:

```text
import { Foo } from "./foo"
import Foo from "./foo"
import * as Foo from "./foo"
const Foo = require("./foo")
```

Facts:

```text
file imports module
symbol imports symbol if named import is clear
test imports implementation
consumer depends_on exported symbol
```

MVP import resolution can be simple:

```text
relative path resolution for .ts/.tsx/.js/.jsx
index file fallback
ignore tsconfig path aliases initially or mark unresolved
```

If import cannot be resolved:

```text
record unresolved_import diagnostic
lower confidence
do not invent edge
```

### Route Extraction Rules

Start with Express/Fastify style calls.

Patterns:

```text
router.get(path, handler)
router.post(path, handler)
router.put(path, handler)
router.patch(path, handler)
router.delete(path, handler)
app.get(path, handler)
app.post(path, handler)
fastify.get(path, handler)
fastify.post(path, handler)
```

Facts:

```text
route node external_public
route routes_to handler
route defined_in file
handler implementation_of route
```

Confidence:

```text
literal path and identifier handler: high
literal path and inline handler: medium-high
template literal path without expressions: high
computed path: low
handler through array/spread: medium or low
```

Route prefixes:

```text
MVP can store local route path and record prefix_unknown.
If router is mounted in another file, later query pack can resolve full path.
```

Do not overclaim full route path if prefix is unknown.

### Handler And Type Link Rules

The MVP should link handlers to request/response types when obvious.

Patterns:

```text
function createUserHandler(req: Request<..., ..., CreateUserRequest>, res: Response)
const createUserHandler = async (req: TypedRequest<CreateUserRequest>, res) => {}
handler calls validateCreateUserInput(req.body)
handler calls createUserService(...)
```

Facts:

```text
handler accepts_type CreateUserRequest
handler calls validateCreateUserInput
handler calls createUserService
```

Confidence:

```text
type annotation: high
validator function naming match: medium
semantic naming only: low, candidate_relevance only
```

The MVP should avoid pretending a naming match is a type edge.

### Config Key Rules

Detect literal env reads:

```text
process.env.USER_SIGNUP_ENABLED
process.env["USER_SIGNUP_ENABLED"]
getEnv("USER_SIGNUP_ENABLED")
config.get("USER_SIGNUP_ENABLED")
```

Facts:

```text
symbol reads_config USER_SIGNUP_ENABLED
config key node external_public
.env.example documents config key
README documents config key if literal appears
test sets config key if literal appears in test
```

Confidence:

```text
process.env literal: high
wrapper with literal string: medium
computed key: low
```

Config keys are public enough to matter because deployments depend on them.

### Test Link Rules

MVP test links can be heuristic.

Signals:

```text
test file imports implementation symbol
test file imports route app/server
test name contains route/symbol words
test sends HTTP request to route path
test fixture sets config key
```

Facts:

```text
test tests symbol
test tests route
test sets_config key
```

Confidence:

```text
direct import plus assertion/call: high
HTTP request to literal route: high
filename/name similarity only: low
```

The broker should recommend tests only when confidence is good enough.

### Generated File Rules

Detect generated files by:

```text
path contains generated, gen, __generated__
top comment says generated or do not edit
OpenAPI/protobuf/graphql generated client patterns
package script generates file
```

Facts:

```text
generated file generated_from source contract if obvious
generated file should_not_edit_directly
```

MVP behavior:

```text
If public-impact includes generated file, warn Codex to edit source contract.
```

Example warning:

```text
src/generated/client.ts appears generated from openapi.yaml. Do not edit it
directly unless the generator source is intentionally unchanged.
```

### SQLite Tables For MVP

Use a tiny schema first.

```sql
CREATE TABLE files (
  file_id TEXT PRIMARY KEY,
  path TEXT NOT NULL,
  language TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  generated INTEGER NOT NULL DEFAULT 0,
  last_indexed_at TEXT NOT NULL
);

CREATE TABLE nodes (
  stable_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  semantic_path TEXT NOT NULL,
  visibility TEXT NOT NULL,
  file_id TEXT NOT NULL,
  span_start INTEGER,
  span_end INTEGER,
  content_hash TEXT,
  confidence REAL NOT NULL
);

CREATE TABLE edges (
  edge_id TEXT PRIMARY KEY,
  from_id TEXT NOT NULL,
  to_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  confidence REAL NOT NULL,
  confidence_reason TEXT,
  freshness_status TEXT NOT NULL
);

CREATE TABLE evidence (
  evidence_id TEXT PRIMARY KEY,
  fact_id TEXT NOT NULL,
  file_id TEXT NOT NULL,
  span_start INTEGER,
  span_end INTEGER,
  detection_method TEXT NOT NULL,
  query_pack TEXT NOT NULL
);
```

This is not the final schema.

It is enough to run public-impact.

### Query Pack Output

The TypeScript query pack should output normalized facts, not raw AST nodes.

Example:

```json
{
  "nodes": [
    {
      "kind": "type",
      "name": "CreateUserRequest",
      "visibility": "module_public",
      "file": "src/types/users.ts",
      "span": "1-8"
    }
  ],
  "edges": [
    {
      "kind": "exports",
      "from": "src/types/users.ts",
      "to": "CreateUserRequest",
      "confidence": 0.94
    }
  ],
  "diagnostics": []
}
```

The indexer then:

```text
assigns stable ids
stores facts
links package boundary visibility
materializes public graph projection
```

### `ptctx public-impact` Algorithm

Input:

```text
query string
token budget
repo root
```

Algorithm:

```text
1. Resolve query to candidate nodes by exact name, export name, route path, or file.
2. Prefer public/package boundary nodes over private candidates.
3. If multiple candidates exist, return ambiguity.
4. For each candidate, find public surfaces connected by:
   - exports/reexports
   - routes_to
   - accepts_type
   - tests
   - reads_config
   - generated_from
5. Compute risk level.
6. Rank next reads.
7. Rank next tests.
8. Return response envelope with evidence and freshness.
```

Risk level:

```text
high: external_public route/CLI/generated contract affected
medium: package_public export or config key affected
low: module_public only
unknown: unresolved ambiguity or stale graph
```

### Example Output

```json
{
  "schema_version": "ptctx.result.v1",
  "command": "public-impact",
  "status": "ok",
  "answer": "CreateUserRequest is part of the POST /users public route contract and is reexported from the package boundary.",
  "confidence": 0.87,
  "freshness": {
    "status": "fresh",
    "dirty_files": []
  },
  "risk": {
    "level": "high",
    "reasons": [
      "external_public route contract",
      "package_public type export",
      "route-level test exists"
    ]
  },
  "evidence": [
    {
      "file": "src/types/users.ts",
      "span": "1-8",
      "symbol": "CreateUserRequest",
      "reason": "Type definition"
    },
    {
      "file": "src/index.ts",
      "span": "1-1",
      "symbol": "CreateUserRequest",
      "reason": "Package boundary reexport"
    },
    {
      "file": "src/routes/users.ts",
      "span": "10-14",
      "symbol": "POST /users",
      "reason": "Public route"
    }
  ],
  "recommended_next_reads": [
    "src/types/users.ts",
    "src/routes/users.ts",
    "src/handlers/users.ts",
    "tests/users.create.test.ts"
  ],
  "recommended_next_tests": [
    "npm test -- tests/users.create.test.ts"
  ],
  "uncertainty": [
    "Full route prefix was not resolved from parent app mounting."
  ]
}
```

This is the target feel.

Short, specific, evidence-backed.

### TDD Test List

Start with tests.

```text
test_package_json_exports_marks_boundary_file
test_index_reexport_marks_package_public_type
test_internal_export_stays_module_public
test_express_post_route_creates_external_public_route
test_route_links_to_identifier_handler
test_handler_type_annotation_links_request_type
test_process_env_literal_creates_config_key
test_route_test_links_to_public_route
test_generated_file_warning_from_header_comment
test_public_impact_ranks_route_contract_high
test_private_helper_not_external_public
test_stale_file_downgrades_public_impact
```

Each test should use the fixture.

Each test should assert:

```text
nodes
edges
visibility
evidence
confidence
freshness if relevant
```

### Implementation Order

1. Fixture and golden case.
2. File discovery and package boundary reader.
3. Tree-sitter TypeScript parse wrapper with safety controls.
4. Export/import extraction.
5. Route extraction.
6. Type annotation and handler link extraction.
7. Config key extraction.
8. Test link heuristics.
9. SQLite storage.
10. Public-impact query.
11. JSON response envelope.
12. Benchmark case runner.

Do not start with optimization.

Start with correctness and evidence.

### Done Criteria

The slice is done when:

```text
fixture benchmark passes
private helper false-positive test passes
stale public-impact test passes
public-impact output fits 3000-token budget
Codex can use the answer without manual translation
```

Manual smoke test:

```bash
ptctx health --json
ptctx public-impact --query "CreateUserRequest" --budget 3000 --json
```

Expected:

```text
high risk
POST /users mentioned
CreateUserRequest mentioned
route/handler/test next reads
focused test command
freshness visible
uncertainty about unresolved prefix if applicable
```

### Failure Modes To Watch

```text
Every exported symbol becomes package_public.
Route path is overclaimed when prefix is unknown.
Test file helper becomes public API.
Generated client is treated as source of truth.
Config keys are ignored.
Confidence is always high.
Output is too verbose for Codex.
SQLite stores raw AST instead of normalized facts.
```

Each failure mode should become a regression test.

### Why This Slice Matters

This slice proves the entire strategy.

If TypeScript public graph can make Codex safer on CRUD edits, then the model
has legs.

It shows:

```text
query packs can produce relationship facts
publicness can be modeled usefully
freshness can be exposed
Codex can get next reads/tests from a graph
benchmark cases can catch graph lies
```

If it cannot do those things, adding Rust or C++ will not save the product.

### Concept 18 Conclusion

The first concrete build should be TypeScript public graph MVP.

It should deliver:

```text
fixture
package boundary extraction
exports/reexports
route extraction
handler/type links
config key links
test links
generated warnings
SQLite fact storage
ptctx public-impact
benchmark pass
```

This is the smallest end-to-end slice that tests the central promise:

```text
Can Codex know when a TypeScript change affects a public contract before it
edits?
```

That is a sharp, useful question.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| Concept 17 in this document | Source-read through current synthesis | Supplies minimal v2 core boundaries and build order. |
| Concept 16 in this document | Source-read through current synthesis | Supplies benchmark case and false-positive/false-negative criteria. |
| Concept 15 in this document | Source-read through current synthesis | Supplies TypeScript public graph extraction rules. |
| Concept 14 in this document | Source-read through current synthesis | Supplies relationship fact schema. |
| Concept 12 in this document | Source-read through current synthesis | Supplies `ptctx` broker command and response envelope. |

## Concept 19: Add A Rust Public Graph MVP As The Second Slice

### The Point

Rust should be the second public graph slice after TypeScript.

Reason:

```text
Rust tests a different publicness model.
```

TypeScript publicness is often about:

```text
exports
routes
package boundaries
request/response types
runtime framework registration
```

Rust publicness is often about:

```text
pub items
pub use reexports
module boundaries
traits
impls
feature flags
binary commands
crate APIs
FFI surfaces
compile-time configuration
```

This makes Rust a strong second slice because it forces the relationship schema
to handle systems-programming contracts, not just web routes.

### MVP Goal

The Rust MVP should answer:

```text
If Codex changes this Rust item, is it part of a crate public API, CLI surface,
feature-gated surface, route surface, trait contract, or FFI boundary?
```

Flagship command:

```bash
ptctx public-impact --query "ParseOptions" --budget 3000 --json
```

Expected answer:

```text
ParseOptions is a package_public struct reexported from lib.rs and used by
parse_source_file. Changing public fields may affect crate users and parser API
tests.
```

This is the Rust equivalent of the TypeScript public route contract.

### Why Rust Needs Compile-Backed Verification

Tree-sitter can identify Rust syntax.

It cannot fully prove Rust semantics.

Rust public graph should therefore use:

```text
Tree-sitter for fast structural extraction
cargo metadata for crate/target/feature boundaries
cargo check or cargo test for compile-backed verification
```

The graph should guide Codex to the right files and risks.

The compiler should confirm final correctness.

Codex should treat Rust graph output as:

```text
strong navigation evidence
```

not:

```text
replacement for cargo check
```

### Fixture First

Proposed fixture:

```text
bench/public-interface/fixtures/rust-public-graph-mvp/
  Cargo.toml
  src/
    lib.rs
    parser/
      mod.rs
      engine.rs
      options.rs
    internal/
      cache.rs
    cli.rs
    routes.rs
  tests/
    parser_api_tests.rs
    cli_tests.rs
```

Fixture facts:

```text
lib.rs pub mod parser
lib.rs pub use parser::ParseOptions
parser/mod.rs pub fn parse_source_file
parser/options.rs pub struct ParseOptions
ParseOptions has public and private fields
parser/engine.rs has pub(crate) fn parse_with_engine
internal/cache.rs has pub(crate) cache
trait EntityExtractor is public
impl EntityExtractor for TreeSitterExtractor
Cargo.toml defines feature experimental-parser
cli.rs defines clap command parse
routes.rs defines axum route /parse if feature api enabled
tests/parser_api_tests.rs tests parse_source_file
```

Traps:

```text
pub(crate) item should not be external public
private struct field should not be public contract
feature-gated route should show feature requirement
test helper pub fn should not be crate API
macro-generated item should be uncertain if not expanded
```

### Cargo Boundary Rules

Read `Cargo.toml`.

Extract:

```text
package name
lib target
bin targets
features
workspace membership
dependencies
```

Facts:

```text
crate defines package boundary
lib target exposes crate public API
bin target exposes CLI/runtime surface
feature defines config/build surface
workspace package may consume crate public API
```

Rules:

```text
If item is `pub` and reachable from lib.rs public module tree, mark package_public.
If item is `pub(crate)`, mark crate_public or workspace_public, not external_public.
If item is `pub(super)`, mark module_public.
If binary target defines clap commands, mark CLI external_public.
If feature flag gates an item, attach feature requirement.
```

This is where Rust differs from TypeScript.

`pub` is necessary but not always sufficient.

Reachability from the public module tree matters.

### Module Resolution Rules

MVP module resolution should handle:

```text
mod parser;
pub mod parser;
mod parser { ... }
src/parser.rs
src/parser/mod.rs
pub use parser::ParseOptions;
```

Facts:

```text
module contains item
lib.rs exposes module
pub use reexports item
file defines module
```

Do not attempt full macro expansion in MVP.

If a module or item is macro-generated:

```text
record uncertain_generated_or_macro node
lower confidence
recommend cargo check
```

### Visibility Rules

Rust visibility is nuanced.

Mapping:

| Rust Visibility | Graph Visibility |
|---|---|
| `pub` reachable from lib public tree | `package_public` |
| `pub` inside private module not reexported | `module_public` or `private_reachable_unknown` |
| `pub(crate)` | `crate_public` |
| `pub(super)` | `module_public` |
| `pub(in path)` | `module_public` scoped to path |
| no visibility | `private` |
| `pub extern "C"` or `no_mangle` | `external_public` FFI |

Codex needs this because `pub(crate)` is easy to overstate.

Changing a `pub(crate)` helper is not the same as changing a public crate API.

### Item Extraction Rules

Detect:

```text
fn_item
struct_item
enum_item
trait_item
type_item
const_item
static_item
impl_item
mod_item
use_declaration
attribute_item
```

Facts:

```text
file defines item
module contains item
crate exports item
pub use reexports item
trait declares method
impl implements trait for type
function accepts_type
function returns_type
```

MVP type links can be shallow:

```text
function parameter type identifiers
return type identifiers
struct field types
trait method signatures
```

This is enough for public impact.

### Struct Field Rules

Rust struct fields matter.

Rules:

```text
Public struct with public fields: fields are public contract.
Public struct with private fields: constructor/accessor methods form contract.
Tuple struct public field visibility matters.
Enum variants in public enum are public contract.
Variant fields in public enum are public contract if public by enum visibility.
```

Facts:

```text
struct exposes_field field
function returns_type struct
test constructs struct
```

Changing public fields is higher risk than changing private fields.

Codex needs that distinction.

### Trait And Impl Rules

Traits are public contracts.

Rules:

```text
Public trait is package_public.
Trait methods are public interface nodes.
Impl of public trait for public type is contract behavior.
Impl of local private trait can be private.
Blanket impls should be marked uncertain if type matching is broad.
```

Facts:

```text
trait declares method
type implements trait
impl provides method
function accepts_type trait object if visible
```

Impact of changing a trait method:

```text
all impls
all callers using trait bound
tests for implementors
```

This is one of the most important Rust public graph workflows.

### Feature Flag Rules

Cargo features are public build/config surfaces.

Detect:

```text
[features]
#[cfg(feature = "...")]
cfg_attr
optional dependencies
```

Facts:

```text
feature defines config/build surface
item feature_gated_by feature
dependency enabled_by feature
test may require feature
```

Codex should see:

```text
This API exists only with feature experimental-parser.
```

Changing feature-gated code without noting the feature can produce misleading
impact.

### Clap CLI Rules

CLI commands are external public surfaces.

Detect:

```text
#[derive(Parser)]
#[derive(Subcommand)]
#[command(name = "...")]
#[arg(...)]
Command::new("...")
.subcommand(...)
```

Facts:

```text
binary exposes cli_command
cli_command handled_by function or enum variant if traceable
cli_arg configures command
test covers command
```

MVP can start with derive-based clap.

Builder-style clap can be second.

Public impact:

```text
Changing CLI arg name or subcommand is external_public risk.
```

### Axum Route Rules

If Rust fixture includes web routes, support axum first.

Detect:

```text
Router::new()
.route("/path", get(handler))
.route("/path", post(handler))
get(handler)
post(handler)
put(handler)
delete(handler)
```

Facts:

```text
route node external_public
route routes_to handler
handler accepts_type extractor types
handler returns_type response type if detectable
```

Confidence:

```text
literal route plus handler identifier: high
nested router prefix: medium unless resolved
macro/generator route: unknown
```

Axum support is valuable for Rust web CRUD apps.

### FFI Rules

FFI is public even if not used in Rust code.

Detect:

```text
extern "C"
#[no_mangle]
#[export_name = "..."]
cdylib crate type
```

Facts:

```text
function external_public FFI surface
crate target exports symbol
function accepts_type FFI-safe type
```

Risk:

```text
Changing signature is high risk.
Changing struct layout used in FFI is high risk.
```

This matters for systems programming.

### Test Link Rules

Rust test links:

```text
integration test imports crate item
unit test module calls private item
test function name includes public item
cargo test target covers crate
```

Facts:

```text
test tests symbol
test tests cli_command if command invoked
test tests route if HTTP route helper invoked
```

Confidence:

```text
direct import and call: high
test name similarity only: low
```

Codex should prefer tests that directly import or call public APIs.

### Compile-Backed Verification

The Rust public graph should recommend compile checks.

For public API changes:

```bash
cargo check --workspace
```

For feature-gated changes:

```bash
cargo check --workspace --features experimental-parser
```

For focused tests:

```bash
cargo test -q parser_api
```

For CLI:

```bash
cargo test -q cli
```

The broker should not run all of these automatically every time.

It should recommend them based on impact.

### Example Output

```json
{
  "schema_version": "ptctx.result.v1",
  "command": "public-impact",
  "status": "ok",
  "answer": "ParseOptions is a package_public struct reexported from lib.rs and used by parse_source_file.",
  "confidence": 0.9,
  "risk": {
    "level": "medium",
    "reasons": [
      "package_public struct",
      "reexported from crate boundary",
      "used by public parse_source_file"
    ]
  },
  "evidence": [
    {
      "file": "src/parser/options.rs",
      "span": "3-18",
      "symbol": "ParseOptions",
      "reason": "Public struct definition"
    },
    {
      "file": "src/lib.rs",
      "span": "2-2",
      "symbol": "ParseOptions",
      "reason": "Crate boundary reexport"
    }
  ],
  "recommended_next_reads": [
    "src/parser/options.rs",
    "src/lib.rs",
    "src/parser/mod.rs",
    "tests/parser_api_tests.rs"
  ],
  "recommended_next_tests": [
    "cargo test -q parser_api"
  ],
  "recommended_verification": [
    "cargo check --workspace"
  ]
}
```

This is Rust-specific because it recommends compiler verification.

### TDD Test List

Tests:

```text
test_cargo_metadata_detects_crate_features
test_pub_item_reachable_from_lib_is_package_public
test_pub_crate_item_not_external_public
test_pub_use_reexport_marks_package_public
test_public_struct_fields_are_contract
test_private_struct_fields_not_public_contract
test_public_trait_method_is_contract
test_impl_links_type_to_trait
test_feature_gated_item_records_feature
test_clap_derive_creates_cli_command
test_axum_literal_route_links_handler
test_no_mangle_extern_c_is_external_public
test_integration_test_links_public_api
test_public_impact_recommends_cargo_check
```

Freshness tests:

```text
test_line_shift_preserves_rust_item_identity
test_modified_pub_item_marks_public_projection_fresh_after_reindex
test_dirty_pub_item_downgrades_public_impact_until_reindex
```

### Implementation Order

1. Rust fixture and golden benchmark case.
2. Cargo.toml metadata reader.
3. Module and visibility extraction.
4. `pub use` reexport resolution.
5. Public structs/enums/traits/functions.
6. Shallow type signature extraction.
7. Trait/impl relationships.
8. Feature flag extraction.
9. Clap derive extraction.
10. Axum route extraction.
11. FFI surface extraction.
12. Test link heuristics.
13. Public-impact output and compile-check recommendations.

Do not start with macros.

Do not start with full rustc semantic modeling.

Start with public surfaces Tree-sitter can reliably see.

### Failure Modes

```text
Marks pub(crate) as package_public.
Misses pub use reexports.
Ignores feature flags.
Treats private struct fields as public.
Misses trait method impact.
Overclaims macro-generated routes.
Fails to recommend cargo check.
Treats test helper pub function as crate API.
Cannot tell binary CLI surface from library API.
```

Each failure should become a fixture case.

### Done Criteria

Rust MVP is done when:

```text
public graph fixture passes
pub/pub(crate)/pub use visibility is correct
public trait and impl edges exist
feature flags are represented
clap or axum surface works for at least one pattern
FFI surface is detected if present
public-impact recommends focused tests and cargo check
dirty file freshness is visible
```

This gives Codex a credible Rust map.

### Why Rust Is Worth Doing Early

Rust matters for this user because the user explicitly works on:

```text
Rust systems and CLI programming
```

Rust also forces the graph to become more disciplined.

It cannot survive with fuzzy "public means exported" logic.

It must understand:

```text
visibility scopes
module reachability
traits
features
compiler verification
```

That discipline improves the whole product.

### Concept 19 Conclusion

Rust public graph MVP should be the second implementation slice.

It should add:

```text
Cargo metadata
module/visibility extraction
pub and pub use public API detection
pub(crate) scoping
public structs/enums/traits/functions
trait/impl relationships
feature flags
clap CLI surfaces
axum routes
FFI exports
test links
compile-backed verification recommendations
```

The product promise:

```text
Codex can tell whether a Rust change touches a crate, CLI, feature, trait, route,
or FFI contract before it edits.
```

That is a meaningful large-codebase safety improvement.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| Concept 18 in this document | Source-read through current synthesis | Supplies the TypeScript slice pattern that Rust follows. |
| Concept 17 in this document | Source-read through current synthesis | Supplies minimal v2 build order and core boundaries. |
| Concept 15 in this document | Source-read through current synthesis | Supplies Rust public graph extraction rules. |
| Concept 14 in this document | Source-read through current synthesis | Supplies relationship schema for traits, features, and public surfaces. |
| `docs/research001/unclassified/ISGL1-v2-Stable-Entity-Identity.md` | Source-read | Supplies stable identity requirements for Rust line-shift and content-hash cases. |

## Concept 20: Add A C And C++ Public Graph MVP For Header And ABI Risk

### The Point

C and C++ public graph work is different from TypeScript and Rust.

The public surface is often not the implementation file.

It is:

```text
header declaration
installed include path
export macro
extern C function
class public method
virtual method
struct layout
enum value
typedef
template declaration
macro
linker-visible symbol
```

A Codex agent can easily edit the `.cpp` body and miss the fact that the
contract lives in a header.

The C/C++ public graph MVP should answer:

```text
If Codex changes this declaration, definition, type, macro, or method, does it
affect a public header, ABI boundary, include graph, exported symbol, or build
target?
```

This is the systems-programming safety slice.

### MVP Goal

Flagship command:

```bash
ptctx public-impact --query "parse_config" --budget 3000 --json
```

Expected answer:

```text
parse_config is declared in include/config.h and defined in src/config.c.
Changing its signature affects the public C API and tests/config_test.c.
```

For C++:

```bash
ptctx public-impact --query "Parser::parse" --budget 3000 --json
```

Expected answer:

```text
Parser::parse is a public method declared in include/parser/Parser.hpp and
implemented in src/Parser.cpp. Changing its signature affects downstream
include users and ParserTest.
```

### Explicit Non-Goals

Do not start with:

```text
full C++ template instantiation
full preprocessor evaluation
full compile database semantic model
full ABI diffing
all platform linker behavior
complete macro expansion
whole-program call graph
```

Start with:

```text
headers
definitions
include graph
public/protected/private methods
export annotations
extern C
macros in public headers
tests and build targets
compile-backed verification recommendations
```

This is enough to help Codex avoid the biggest mistakes.

### Fixture First

Proposed fixture:

```text
bench/public-interface/fixtures/cpp-public-graph-mvp/
  CMakeLists.txt
  include/
    config.h
    parser/
      Parser.hpp
      Export.hpp
  src/
    config.c
    Parser.cpp
    internal_config.c
  tests/
    config_test.c
    ParserTest.cpp
  compile_commands.json
```

Fixture facts:

```text
include/config.h declares parse_config
src/config.c defines parse_config
tests/config_test.c calls parse_config
include/parser/Parser.hpp declares class Parser
Parser::parse is public
Parser::parseInternal is private
Parser::onError is protected virtual
Export.hpp defines PARSER_API export macro
CMakeLists installs public headers
internal_config.c has static helper not public
```

Traps:

```text
static function in .c should not be public
private method should not be public
protected virtual method is subclass contract, not private
macro in private source file should not be public API
public struct field layout should be marked ABI risk
```

### Boundary Detection

C/C++ boundaries come from:

```text
include/
public/
installed headers
CMake install rules
target_include_directories PUBLIC
export macros
extern C
compile_commands.json
pkg-config files
```

Rules:

```text
Header under include/ is package_public by default.
Header under src/ is private unless included by public header or install rule.
Header listed in install rules is package_public.
Target include directory marked PUBLIC is package_public.
Function declared in public header is package_public.
Definition matching public declaration implements public API.
Static function in source file is private.
```

This is the C/C++ equivalent of package boundary detection.

### Header Declaration Rules

Detect in headers:

```text
function declarations
struct declarations
enum declarations
typedefs
class declarations
method declarations
macros
extern declarations
template declarations
```

Facts:

```text
header declares symbol
header exports symbol if public header
source defines symbol if matching definition found
test calls symbol if included/called
```

Visibility:

```text
public header declaration: package_public
private header declaration: module_public or private
source-local static function: private
extern C public header declaration: external_public FFI
```

### Definition Link Rules

Link declarations to definitions by:

```text
function name
namespace/class scope
parameter arity if available
file include relationship
source includes declaring header
```

Example facts:

```text
include/config.h declares parse_config
src/config.c defines parse_config
src/config.c includes include/config.h
parse_config definition implements public declaration
```

Confidence:

```text
same name plus source includes header: high
same name only: medium
overload ambiguity: low or ambiguous
template specialization: uncertain
```

The MVP should expose ambiguity rather than guess.

### Class Method Rules

For C++ classes:

```text
public methods are public interface
protected methods are subclass interface
private methods are implementation
virtual public/protected methods are high-risk contracts
pure virtual methods are abstract interface contracts
overrides are implementation of contract
```

Facts:

```text
class declares method
method visibility public/protected/private
method overrides base method if override keyword present
class extends base class
source defines method
test calls method
```

Risk:

```text
public method signature change: high
protected virtual method change: high for subclasses
private method body change: local
private method signature change: local unless used in implementation file
```

Codex needs this distinction.

### Struct And ABI Rules

C/C++ structs are public contracts when in public headers.

Risk markers:

```text
public struct field added
public struct field removed
public struct field type changed
enum value changed
typedef changed
macro constant changed
extern C function signature changed
class virtual method changed
```

Facts:

```text
struct exposes_field field
field has_type type
enum exposes_variant value
typedef aliases type
```

The MVP should not perform full ABI diffing.

It should mark ABI risk:

```text
This public header type layout changed; run compile and ABI-sensitive tests.
```

That warning alone helps Codex.

### Macro Rules

Macros in public headers are public API.

Detect:

```text
#define NAME
#define NAME(args)
export macros
feature macros
constant macros
include guards
```

Rules:

```text
Include guards are not public API.
Export macros are contract metadata.
Constant/function-like macros in public headers are package_public.
Macros in private source files are private.
```

Facts:

```text
header defines macro
symbol annotated_by export macro
source uses macro
test uses macro
```

Uncertainty:

```text
macro expansion is not fully modeled in MVP
```

The graph should not pretend it expanded macros.

### Include Graph Rules

Include graph is core for C/C++.

Detect:

```text
#include "config.h"
#include <parser/Parser.hpp>
```

Facts:

```text
file includes header
public header includes dependency header
source includes public header
test includes public header
```

Impact:

```text
Changing a public header affects files that include it.
Changing a private source file affects direct callers/tests, not include graph.
```

The include graph is the first public blast-radius approximation.

It is not perfect, but it is useful.

### Exported Symbol Rules

Detect:

```text
__attribute__((visibility("default")))
__declspec(dllexport)
__declspec(dllimport)
extern "C"
export macro names like API_EXPORT or PARSER_API
CMake target export hints if visible
```

Facts:

```text
symbol exported_by macro
symbol external_public
header declares exported symbol
```

Confidence:

```text
known visibility attribute: high
export macro defined in public header: medium-high
unknown macro on declaration: medium, annotate raw macro
```

Export macros are platform-specific, so confidence should reflect uncertainty.

### Build System Rules

MVP should read basic CMake.

Detect:

```text
add_library
add_executable
target_sources
target_include_directories
target_link_libraries
install
enable_testing
add_test
```

Facts:

```text
target builds source
target exposes include directory
test target tests library
binary depends_on library
```

If `compile_commands.json` exists:

```text
use it to map source files to compile commands
use include paths for header resolution
```

Do not write a full CMake interpreter for MVP.

Use obvious patterns and mark unknowns.

### Compile-Backed Verification

C/C++ graph output should recommend compile verification.

Possible recommendations:

```bash
cmake --build build
ctest --test-dir build
ninja -C build
make test
clang -fsyntax-only src/config.c
clang++ -fsyntax-only src/Parser.cpp
```

The broker should infer commands from repo conventions if possible.

For MVP, it can say:

```text
Run the project's normal C/C++ build and config/parser tests.
```

Better:

```text
recommended_verification:
  - cmake --build build
  - ctest --test-dir build -R config
```

Compile-backed verification matters more in C/C++ than in TypeScript public
graph because syntax and headers can be misleading without the preprocessor and
include paths.

### Example Output

```json
{
  "schema_version": "ptctx.result.v1",
  "command": "public-impact",
  "status": "ok",
  "answer": "parse_config is declared in public header include/config.h and defined in src/config.c.",
  "confidence": 0.88,
  "risk": {
    "level": "high",
    "reasons": [
      "public header declaration",
      "external C API surface",
      "test includes public header"
    ]
  },
  "evidence": [
    {
      "file": "include/config.h",
      "span": "12-15",
      "symbol": "parse_config",
      "reason": "Public header declaration"
    },
    {
      "file": "src/config.c",
      "span": "22-45",
      "symbol": "parse_config",
      "reason": "Matching definition"
    }
  ],
  "recommended_next_reads": [
    "include/config.h",
    "src/config.c",
    "tests/config_test.c",
    "CMakeLists.txt"
  ],
  "recommended_next_tests": [
    "ctest --test-dir build -R config"
  ],
  "recommended_verification": [
    "cmake --build build"
  ],
  "uncertainty": [
    "Full preprocessor conditions were not evaluated."
  ]
}
```

This gives Codex the right shape of caution.

### TDD Test List

Tests:

```text
test_public_header_declares_function
test_source_definition_links_to_header_declaration
test_static_source_function_is_private
test_public_struct_field_marked_abi_risk
test_private_header_not_package_public
test_class_public_method_is_public_contract
test_class_private_method_is_not_public_contract
test_protected_virtual_method_is_subclass_contract
test_extern_c_declaration_is_external_public
test_export_macro_marks_symbol_exported
test_include_graph_links_source_to_header
test_public_header_macro_is_public_api
test_include_guard_not_public_api
test_cmake_public_include_dir_marks_headers_public
test_public_impact_recommends_build_and_ctest
```

Freshness:

```text
test_header_edit_marks_public_impact_stale_until_reindex
test_definition_edit_preserves_public_declaration_identity
test_line_shift_preserves_header_declaration_identity
```

### Implementation Order

1. C/C++ fixture and golden case.
2. Header/source file classification.
3. Include graph extraction.
4. Function declaration extraction.
5. Definition matching.
6. Static/private function detection.
7. Struct/enum/typedef extraction.
8. Class method visibility extraction.
9. Macro extraction with include guard filter.
10. Export/extern C detection.
11. CMake/public include heuristic.
12. Test include/call links.
13. Public-impact response and compile verification recommendations.

Do not start with full preprocessor support.

Start with public headers and definition links.

### Failure Modes

```text
Treats every header as public.
Misses installed include/public headers.
Marks static functions as public.
Ignores struct layout risk.
Treats include guards as public macros.
Misses private/public/protected class sections.
Confuses declaration and definition.
Overclaims template specialization.
Ignores compile_commands include paths.
Fails to recommend build verification.
```

Each one should become a fixture test.

### Done Criteria

C/C++ public graph MVP is done when:

```text
public headers are detected
private headers are not over-marked
public declarations link to definitions
include graph is usable
class method visibility works
extern C/export macro surfaces are represented
struct ABI risk is flagged
compile/build verification is recommended
public-impact output includes evidence and uncertainty
```

This gives Codex a safer path through systems code.

### Why This Slice Matters

C and C++ are where public graph mistakes can become expensive.

Changing a header can break:

```text
many translation units
external clients
ABI compatibility
FFI callers
tests that compile only in certain targets
```

Codex needs to see that before editing.

The C/C++ slice ensures the graph is not web-app-only.

### Concept 20 Conclusion

C/C++ public graph MVP should focus on:

```text
public headers
declaration-definition links
include graph
class visibility
struct/enum/typedef ABI risk
macros in public headers
extern C and export macros
build targets
compile-backed verification
```

The product promise:

```text
Codex can tell whether a C/C++ change touches a public header, ABI surface,
include graph, or exported symbol before it edits.
```

That is the systems-language counterpart to the TypeScript route graph and Rust
crate API graph.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| Concept 19 in this document | Source-read through current synthesis | Supplies Rust systems slice pattern and compile-backed verification framing. |
| Concept 17 in this document | Source-read through current synthesis | Supplies minimal v2 core boundaries and parser/freshness requirements. |
| Concept 15 in this document | Source-read through current synthesis | Supplies C/C++ public graph extraction rules. |
| Concept 14 in this document | Source-read through current synthesis | Supplies relationship schema for declarations, definitions, includes, and ABI-like risk. |
| Concept 16 in this document | Source-read through current synthesis | Supplies benchmark/fixture approach and false-positive controls. |

## Concept 21: Add A Python Public Graph MVP With Honest Confidence

### The Point

Python should be the fourth public graph slice because it tests a softer,
convention-heavy publicness model.

Python public surfaces often come from:

```text
__all__
__init__.py reexports
function/class naming conventions
FastAPI route decorators
Flask route decorators
Django URL patterns
Click/Typer/argparse commands
Pydantic models
dataclasses
settings classes
environment variables
package entry points
```

Unlike Rust, Python rarely gives a single hard visibility keyword.

So the Python graph must be useful while being honest:

```text
strong evidence when __all__, route decorators, or entry points exist
medium evidence for __init__.py reexports
low evidence for "does not start with underscore" conventions
```

The product should not pretend dynamic Python is statically complete.

It should give Codex good next reads and honest uncertainty.

### MVP Goal

Flagship command:

```bash
ptctx public-impact --query "CreateUserRequest" --budget 3000 --json
```

Expected answer:

```text
CreateUserRequest is a Pydantic model used by the POST /users FastAPI route.
Changing it affects the public route contract and tests/test_users.py.
```

For package exports:

```bash
ptctx public-impact --query "UserClient" --budget 3000 --json
```

Expected answer:

```text
UserClient is reexported from app/__init__.py and appears in __all__, so treat
it as package_public.
```

### Explicit Non-Goals

Do not start with:

```text
full runtime import execution
complete dynamic dispatch analysis
monkey patch tracking
all decorator semantics
full Django settings resolution
complete SQLAlchemy model/dataflow graph
full type checker integration
```

Start with:

```text
package exports
route decorators
CLI decorators/entry points
Pydantic/dataclass API models
literal config keys
pytest test links
honest uncertainty
```

Python MVP is about pragmatic visibility, not perfect semantics.

### Fixture First

Proposed fixture:

```text
bench/public-interface/fixtures/python-public-graph-mvp/
  pyproject.toml
  app/
    __init__.py
    main.py
    routes/
      users.py
    models.py
    services/
      users.py
    config.py
    cli.py
  tests/
    test_users_route.py
    test_user_client.py
```

Fixture facts:

```text
app/__init__.py reexports UserClient
app/__init__.py defines __all__ = ["UserClient"]
routes/users.py defines @router.post("/users")
CreateUserRequest is Pydantic model used by route
create_user_handler calls create_user_service
config.py reads USER_SIGNUP_ENABLED
cli.py defines Typer command create-user
pyproject.toml exposes console script
test_users_route.py hits POST /users
test_user_client.py imports UserClient
_normalize_user is private
```

Traps:

```text
function without underscore but not exported should be module_public, not package_public
private underscore helper should be private
dynamic route path should lower confidence
test fixture export should not be package API
monkey patch should not create confident edge
```

### Package Boundary Rules

Read:

```text
pyproject.toml
setup.py
setup.cfg
app/__init__.py
package __all__
entry points
```

Rules:

```text
Name in __all__ is package_public.
Symbol imported/reexported from __init__.py is package_public with medium-high confidence.
Console script entry point is external_public CLI surface.
Module-level symbol without leading underscore is module_public candidate, not automatically package_public.
Symbol with leading underscore is private unless explicitly in __all__.
Test files do not define package public API unless entry point/package config exposes them.
```

This avoids the common Python mistake:

```text
no underscore means globally public
```

No.

It means public by convention inside the module unless package boundary confirms
more.

### `__all__` Rules

Detect:

```text
__all__ = ["UserClient", "create_user"]
__all__ += ["Other"]
```

MVP should support literal lists.

Facts:

```text
module declares_public_export symbol
symbol package_public if resolvable
```

Confidence:

```text
literal __all__: high
computed __all__: low or unknown
```

If `__all__` is computed dynamically:

```text
record diagnostic
do not invent full export list
```

### `__init__.py` Reexport Rules

Detect:

```text
from .client import UserClient
from .services.users import create_user
```

Facts:

```text
package reexports UserClient
UserClient package_public
```

Confidence:

```text
reexport plus __all__: high
reexport without __all__: medium-high
wildcard import: low unless source __all__ is known
```

This gives Codex a practical package boundary.

### FastAPI Route Rules

Detect:

```text
@app.get("/path")
@app.post("/path")
@router.get("/path")
@router.post("/path")
app.include_router(router, prefix="/api")
```

Facts:

```text
route external_public
route routes_to handler
handler accepts_type Pydantic model if parameter annotated
handler returns_type response model if decorator or annotation shows it
router prefix may contribute route path
```

Confidence:

```text
literal decorator path and function handler: high
router prefix literal resolved: medium-high
dynamic path or prefix: low
response_model decorator literal class: high
```

MVP can start with route decorator and local prefix.

Full prefix composition can be later.

### Flask Route Rules

Detect:

```text
@app.route("/path", methods=["POST"])
@blueprint.route("/path")
app.add_url_rule("/path", view_func=handler)
```

Facts:

```text
route external_public
route routes_to handler
blueprint prefix uncertain unless registered with literal prefix
```

Confidence:

```text
literal route decorator: high
blueprint registered with literal prefix: medium-high
dynamic registration: low
```

Flask should be supported because it is common and relatively parseable.

### Django Route Rules

Detect:

```text
path("users/", views.create_user)
re_path(...)
include(...)
```

Facts:

```text
route external_public
route routes_to view
include creates route prefix if literal
```

Confidence:

```text
path literal plus view reference: high
include literal prefix: medium
re_path regex route: medium
dynamic urls: low
```

Django route extraction is useful but should be conservative.

### Pydantic And Dataclass Model Rules

Detect:

```text
class CreateUserRequest(BaseModel)
@dataclass
class UserDto
```

Facts:

```text
model class defines API/data contract
route handler accepts_type model
route handler returns_type model if response_model or annotation
field belongs_to model
```

Risk:

```text
Changing a Pydantic model used by a route is public contract risk.
Changing a dataclass used only internally is lower risk.
```

Confidence:

```text
BaseModel inheritance: high
dataclass decorator: high
plain class with annotations: medium
```

Codex should know when a model is route-facing.

### CLI Rules

Detect:

```text
@click.command()
@click.group()
@app.command() for Typer
argparse.ArgumentParser()
subparsers.add_parser("name")
pyproject.toml console_scripts
```

Facts:

```text
console script external_public
command handled_by function
option/argument belongs_to command
```

Confidence:

```text
pyproject console script: high
click/typer decorator: high
argparse static parser: medium
dynamic command registration: low
```

Changing CLI flags or command names should be public impact.

### Config Key Rules

Detect:

```text
os.environ["KEY"]
os.getenv("KEY")
environ.get("KEY")
BaseSettings fields
Pydantic settings classes
dotenv keys if present
```

Facts:

```text
function reads_config KEY
settings class defines config key
test sets config key
README or .env.example documents key
```

Confidence:

```text
literal os.environ/os.getenv: high
BaseSettings field: medium-high
computed key: low
```

Config keys should be treated as external public surface when documented or
used by runtime behavior.

### Test Link Rules

Python test signals:

```text
pytest file imports symbol
test client posts to route path
test name contains route/model/service
monkeypatch sets env key
FastAPI TestClient used
Flask test_client used
Django client used
```

Facts:

```text
test tests route
test tests symbol
test sets_config key
```

Confidence:

```text
client request to literal route: high
direct import and call: high
test name similarity only: low
```

Codex should get focused pytest recommendations.

Example:

```bash
pytest tests/test_users_route.py -q
```

### Example Output

```json
{
  "schema_version": "ptctx.result.v1",
  "command": "public-impact",
  "status": "ok",
  "answer": "CreateUserRequest is a Pydantic model used by the POST /users FastAPI route.",
  "confidence": 0.86,
  "risk": {
    "level": "high",
    "reasons": [
      "external_public route contract",
      "Pydantic request model",
      "route-level pytest exists"
    ]
  },
  "evidence": [
    {
      "file": "app/models.py",
      "span": "4-10",
      "symbol": "CreateUserRequest",
      "reason": "Pydantic model"
    },
    {
      "file": "app/routes/users.py",
      "span": "12-20",
      "symbol": "POST /users",
      "reason": "FastAPI route decorator"
    }
  ],
  "recommended_next_reads": [
    "app/models.py",
    "app/routes/users.py",
    "app/services/users.py",
    "tests/test_users_route.py"
  ],
  "recommended_next_tests": [
    "pytest tests/test_users_route.py -q"
  ],
  "uncertainty": [
    "Router prefix composition was not fully resolved."
  ]
}
```

### TDD Test List

Tests:

```text
test_all_literal_marks_package_public
test_init_reexport_marks_package_public
test_leading_underscore_marks_private
test_module_public_not_package_public_without_boundary
test_fastapi_route_decorator_creates_external_public_route
test_fastapi_route_links_pydantic_request_model
test_flask_route_decorator_links_handler
test_django_path_links_view
test_typer_command_creates_cli_surface
test_click_command_creates_cli_surface
test_pyproject_console_script_creates_external_public_cli
test_os_getenv_literal_creates_config_key
test_base_settings_field_creates_config_key
test_pytest_client_post_links_route_test
test_dynamic_route_lowers_confidence
```

Freshness:

```text
test_modified_route_file_marks_public_impact_stale_until_reindex
test_line_shift_preserves_python_route_identity
test_model_edit_updates_content_hash_preserves_identity
```

### Implementation Order

1. Python fixture and golden case.
2. pyproject/setup entry point reader.
3. `__all__` literal extraction.
4. `__init__.py` reexport extraction.
5. Function/class/module visibility heuristics.
6. FastAPI route extraction.
7. Flask route extraction.
8. Django route extraction.
9. Pydantic/dataclass model extraction.
10. CLI decorator and entry point extraction.
11. Config key extraction.
12. Pytest route/symbol links.
13. Public-impact response and pytest recommendation.

Do not start with dynamic import execution.

Do not run user code to discover routes in MVP.

Static extraction plus honesty is safer.

### Failure Modes

```text
Marks every non-underscore function as package_public.
Misses __all__.
Misses __init__.py reexports.
Overclaims dynamic routes.
Treats decorator side effects as certain.
Misses Pydantic route models.
Ignores pyproject console scripts.
Treats test fixtures as public APIs.
Does not expose uncertainty for monkey patching or dynamic imports.
```

Each failure should become a regression case.

### Done Criteria

Python MVP is done when:

```text
__all__ and __init__.py package exports work
FastAPI/Flask route decorators work
Django path basics work
Pydantic route models are linked
Click/Typer or console scripts are represented
literal config keys are represented
pytest route/symbol links work
dynamic patterns lower confidence
public-impact recommends focused pytest command
```

### Why Python Matters

Python matters because many agent-facing tools, MCP servers, scripts, and web
services use it.

It also forces the product to respect uncertainty.

The Python graph cannot be as hard-edged as Rust.

That is healthy.

It teaches the broker to say:

```text
This is likely public.
This is confirmed public.
This is unknown.
```

Those distinctions make Codex safer.

### Concept 21 Conclusion

Python public graph MVP should focus on:

```text
__all__
__init__.py reexports
FastAPI routes
Flask routes
Django routes
Click/Typer/argparse commands
Pydantic/dataclass models
config keys
pytest links
honest confidence
```

The product promise:

```text
Codex can tell whether a Python change touches a package API, route contract,
CLI command, config surface, or test-covered behavior before it edits.
```

This completes the first practical language set:

```text
TypeScript for CRUD apps
Rust for crates/CLI/systems
C/C++ for headers/ABI/systems
Python for scripts/MCP/web services
```

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| Concept 20 in this document | Source-read through current synthesis | Supplies systems-language caution and compile/backing contrast. |
| Concept 18 in this document | Source-read through current synthesis | Supplies TypeScript route/model/test slice pattern. |
| Concept 17 in this document | Source-read through current synthesis | Supplies minimal v2 core boundaries and query-pack approach. |
| Concept 15 in this document | Source-read through current synthesis | Supplies Python extraction rules for public graph. |
| Concept 14 in this document | Source-read through current synthesis | Supplies relationship schema and uncertainty model. |

## Concept 22: Let The Broker Combine Parceltongue Core With Existing Tools

### The Point

Parceltongue core should not try to win every question.

The broker should route each Codex intent to the best source of evidence.

Sometimes that is Parceltongue core.

Sometimes it is `rg`.

Sometimes it is Semgrep.

Sometimes it is ast-grep.

Sometimes it is Clarity.

Sometimes it is cocoindex-code or codemogger.

Sometimes it is the compiler.

The broker's job is to know which evidence type is best for the current job.

The principle:

```text
Use Parceltongue core for fresh relationship facts.
Use existing tools for the domains they already handle better.
Merge evidence, expose disagreement, and never hide uncertainty.
```

This prevents Parceltongue from becoming an overgrown reimplementation project.

### Tool Roles

The broker should assign roles.

| Role | Best Sources |
|---|---|
| Lexical search | `rg`, `fff` |
| Semantic/code chunk search | `cocoindex-code`, `codemogger`, `Probe`, chunkhound |
| Public relationship facts | Parceltongue core if supported and fresh |
| Module reachability/cycles | Clarity |
| Review/diff context | `code-review-graph`, better review graph, git diff |
| Syntax-aware search/rewrite | ast-grep, Comby |
| Security/rule checks | Semgrep, CodeQL if configured |
| Compile/type truth | `cargo check`, TypeScript checker, C/C++ build, pytest/mypy as available |
| Large architecture memory | codebase-memory-mcp, GitNexus, code-graph-mcp |

The broker should not treat all tools as interchangeable.

Search tools find candidates.

Graph tools prove relationships.

Rule tools find policy violations.

Compilers prove build correctness.

Review tools reason over diffs.

### Trust Rules

The broker needs explicit trust rules.

#### Rule 1: Fresh Direct Evidence Beats Stale Graph Evidence

If Parceltongue core is stale and `rg` shows direct current code:

```text
trust raw file evidence for immediate decision
mark graph stale
recommend reindex
```

Do not let stale graph facts override current files.

#### Rule 2: Graph Evidence Beats Semantic Similarity For Dependency Claims

If cocoindex-code says file A is semantically related to B, that is not a call
edge.

Use semantic search for:

```text
candidate relevance
concept discovery
first reads
```

Use graph/core/parser evidence for:

```text
calls
routes_to
exports
tests
reads_config
```

Semantic match is a hint.

It is not a dependency fact.

#### Rule 3: Compiler/Test Evidence Beats Static Guessing

If Rust graph says a public API impact is medium, but `cargo check` fails across
many crates:

```text
compiler wins
```

If TypeScript graph says a type edge exists but `tsc` says the type is unused:

```text
typechecker wins for type truth
```

The graph guides.

The build verifies.

#### Rule 4: Domain Tools Keep Their Domain

Semgrep should remain the better source for:

```text
security and policy rules
```

ast-grep should remain the better source for:

```text
syntax-aware pattern search and rewrites
```

Clarity should remain good for:

```text
module reachability and cycles
```

Parceltongue should not pretend to replace those unless there is benchmark proof.

#### Rule 5: Multiple Independent Agreement Raises Confidence

Confidence rises when:

```text
Parceltongue core finds a route edge
rg confirms route string
test file hits same route
review graph flags same impacted file
```

Confidence falls when:

```text
only one weak source reports relationship
tools disagree
index is stale
symbol is ambiguous
```

This should be reflected in the response.

### Routing Table

Broker routing should be intent-based.

| Codex Intent | Primary | Secondary | Fallback |
|---|---|---|---|
| Find concept by words | semantic search | `rg` | project map |
| Find exact symbol | Parceltongue core | graph MCP | `rg` |
| Public impact | Parceltongue core | graph MCP, Clarity | raw file reads |
| CRUD route trace | Parceltongue core if supported | code-graph-mcp, GitNexus | `rg` route patterns |
| Diff review | code-review-graph | Parceltongue impact | git diff plus raw reads |
| Security concern | Semgrep | CodeQL | `rg` |
| Structural rewrite | ast-grep | Comby | language tooling |
| Module cycle/reach | Clarity | Parceltongue module facts | dependency grep |
| Test selection | Parceltongue test facts | review graph | filename/import heuristics |
| Freshness check | git plus Parceltongue index | tool health | raw mtimes |

This table should live in config, not in Codex memory.

### Adapter Contract

Every adapter should report:

```text
capabilities
health
freshness
confidence semantics
raw output path
normalized facts
diagnostics
```

Adapter result shape:

```json
{
  "adapter": "parceltongue-core",
  "capabilities_used": ["public_impact", "relationship_facts"],
  "status": "ok",
  "freshness": {
    "status": "fresh"
  },
  "facts": [],
  "diagnostics": [],
  "raw_output_ref": ".ptctx/runs/20260706/parceltongue-core.json"
}
```

The raw output ref matters.

When the broker gets confused, the user or Codex can inspect the underlying
tool result.

### Disagreement Types

Disagreements should be typed.

| Disagreement | Example |
|---|---|
| `publicness_conflict` | Parceltongue says package_public, Clarity says module-only. |
| `edge_conflict` | Graph says A calls B, raw search no longer finds call. |
| `freshness_conflict` | Tool says fresh, git shows dirty file after index. |
| `test_conflict` | Review graph suggests test X, test import graph suggests test Y. |
| `route_conflict` | One tool resolves prefix, another only local route. |
| `symbol_ambiguity` | Multiple symbols share name across packages. |
| `tool_failure` | Adapter crashed or timed out. |

Disagreement output:

```json
{
  "uncertainty": [
    {
      "kind": "edge_conflict",
      "summary": "Parceltongue reports createUserHandler calls createUserService, but current rg search did not find the call after local edits.",
      "recommended_action": "Read src/handlers/users.ts directly before editing."
    }
  ]
}
```

This is not noise.

This is decision support.

### Conflict Resolution Order

When tools disagree, use this order:

```text
1. Current raw file content.
2. Fresh compiler/typechecker/test output.
3. Fresh Parceltongue core relationship fact with evidence.
4. Fresh specialized graph tool with evidence.
5. Fresh structural search result.
6. Semantic search relevance.
7. Stale graph fact.
8. No-evidence summary.
```

This order should be explicit.

Codex should not invent its own hierarchy every time.

### When To Trust Parceltongue Core

Trust Parceltongue core when:

```text
language is supported
query pack declares support for the relationship type
index is fresh
fact has file/span evidence
confidence is high
no stronger tool disagrees
```

Especially trust it for:

```text
public graph projection
stable identity across line shifts
freshness-aware impact
query-pack relationships with evidence
token-budgeted graph responses
```

### When Not To Trust Parceltongue Core

Do not trust core alone when:

```text
language support is partial
index is stale
relationship is dynamic/runtime-heavy
framework magic is involved
macro expansion is required
compiler says otherwise
test output says otherwise
another tool has stronger domain evidence
```

Examples:

```text
Use Semgrep over Parceltongue for security rule findings.
Use ast-grep over Parceltongue for precise structural rewrites.
Use cargo check over Parceltongue for Rust compile truth.
Use C/C++ build over Parceltongue for preprocessor/link truth.
Use semantic search over Parceltongue for fuzzy concept discovery.
```

This is tool humility.

### Response Composition

The broker should compose final answers like:

```text
short answer
best evidence
tool agreement
tool disagreement
freshness
next reads
next tests
recommended verification
```

Example:

```json
{
  "answer": "CreateUserRequest affects the public POST /users route contract.",
  "evidence": [
    {
      "source_tool": "parceltongue-core",
      "reason": "Route handler accepts CreateUserRequest."
    },
    {
      "source_tool": "rg",
      "reason": "POST /users route string found in src/routes/users.ts."
    },
    {
      "source_tool": "pytest-linker",
      "reason": "tests/test_users.py posts to /users."
    }
  ],
  "agreement": [
    "Parceltongue core and rg agree on route location."
  ],
  "uncertainty": [
    "OpenAPI contract was not found."
  ],
  "recommended_next_reads": [
    "src/types/users.ts",
    "src/routes/users.ts",
    "tests/test_users.py"
  ]
}
```

Codex should see a final packet, not a pile of tool outputs.

### Fallback Behavior

If Parceltongue core is unavailable:

```text
use external graph tools
use Clarity for module graph
use semantic search and rg
mark graph confidence lower
```

If semantic search is unavailable:

```text
use rg and graph search
```

If graph tools are unavailable:

```text
use rg, AST search, package metadata, and say impact is partial
```

If everything is unavailable:

```text
Codex falls back to normal shell exploration
```

The broker should degrade gracefully.

It should never fail the whole workflow because one optional tool is missing.

### Caching And Runs

The broker should write run artifacts.

Example:

```text
.ptctx/runs/20260706-164500/
  request.json
  parceltongue-core.raw.json
  rg.raw.txt
  semgrep.raw.json
  normalized.json
  final.json
```

This helps:

```text
debug tool disagreements
benchmark repeatability
trust audits
future prompt/skill improvement
```

Do not stuff all raw output into Codex context.

Store it locally and return pointers.

### Benchmarking The Combined Stack

The benchmark should compare:

```text
Parceltongue core alone
external tools alone
broker without Parceltongue core
broker with Parceltongue core
rg baseline
```

The broker with core should win only if it improves:

```text
decision quality
token efficiency
freshness honesty
tool-call efficiency
next-action usefulness
```

If broker with core does not beat core alone, the routing layer is not adding
value.

If broker without core beats broker with core, core is not ready.

Let the benchmark be blunt.

### Example Decision

User request:

```text
Make displayName required on user creation.
```

Broker route:

```text
semantic search finds CreateUserRequest and POST /users
Parceltongue core public-impact confirms route contract
rg confirms route string and model name
test linker finds users.create.test.ts
OpenAPI search checks generated contract
```

Broker answer:

```text
This is a public route contract change. Read model, route, handler, OpenAPI
schema, and route test. Run users.create.test.ts.
```

If OpenAPI is missing:

```text
uncertainty: no contract file found
```

This is the desired combined behavior.

### Concept 22 Conclusion

The broker should make Parceltongue core one strong adapter, not a jealous
monolith.

Use core for:

```text
fresh relationship facts
stable identity
public impact
token-budgeted graph context
```

Use existing tools for:

```text
semantic search
lexical search
module cycles
security rules
structural rewrites
diff review
compiler/test truth
```

The broker wins by routing, merging, and explaining.

The key behavior:

```text
When evidence agrees, compress it.
When evidence disagrees, surface it.
When evidence is stale, downgrade it.
When a domain tool is stronger, use it.
```

That is how Codex gets better context without drowning in tools.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| `docs/research002/J003.md` | Source-read | Supplies the tool stack roles and PMF rankings. |
| Concept 12 in this document | Source-read through current synthesis | Supplies broker command surface and adapter contract. |
| Concept 17 in this document | Source-read through current synthesis | Supplies Parceltongue core responsibilities. |
| Concept 18-21 in this document | Source-read through current synthesis | Supplies the first supported language slices. |
| Concept 10 in this document | Source-read through current synthesis | Supplies benchmark/DQPT comparison requirements. |

## Concept 23: Use A Staged Roadmap With Explicit Stop Criteria

### The Point

The roadmap should not be:

```text
build everything
```

The roadmap should be:

```text
earn the next stage
```

Each stage should answer:

```text
Did this make Codex navigate a large codebase faster and more reliably?
```

If yes, continue.

If no, stop or change direction.

This is the Shreyas-style discipline:

```text
Do not scale the solution until the job is proven.
```

### North Star

The north star remains:

```text
Codex makes better next-read, next-edit, next-test decisions in large codebases
with fewer tokens and fewer graph lies.
```

All work should serve that.

If a feature does not improve that sentence, it is not roadmap work.

### Stage 0: Use Codex Plus Shell Baseline

Purpose:

```text
Establish the baseline.
```

Tools:

```text
Codex
git
rg
rg --files
language test/build commands
```

Artifacts:

```text
baseline benchmark report
examples of tasks where Codex got lost
examples of token waste
examples of missed callers/routes/tests
```

Stop criteria:

```text
If Codex plus shell is already good enough for the user's actual repos, stop.
```

This is unlikely for very large repos, but it must be allowed.

Otherwise the project becomes self-justifying.

### Stage 1: Trial The Existing Tool Stack

Purpose:

```text
See whether current tools already solve the job.
```

Candidate stack:

```text
one graph brain
one search layer
one review graph
Clarity for module reach/cycles
Semgrep/ast-grep for specialized checks
```

Likely first tools:

```text
GitNexus or codebase-memory-mcp
code-review-graph
cocoindex-code or codemogger
Clarity
ast-grep
Semgrep
```

Tasks:

```text
run Concept 10 DQPT benchmark
run public graph smoke cases manually if needed
use tools on one real Codex task
record where tools help
record where tools lie
record setup friction
```

Decision gate:

```text
If existing tool stack scores 85+ DQPT and feels usable in Codex, stop building.
```

Output if stopping:

```text
Codex operating manual for external tool stack
tool config notes
no Parceltongue v2 core
```

Output if continuing:

```text
specific list of stack gaps
```

Do not continue without naming gaps.

### Stage 2: Build The Thin Broker MVP

Purpose:

```text
Reduce tool sprawl and normalize answers.
```

Build:

```text
ptctx health
ptctx project-map
ptctx search
ptctx symbol
ptctx impact
ptctx review
ptctx tests
stable JSON envelope
adapter health
freshness reporting
token budget trimming
```

Adapters:

```text
git
rg
one search tool
one graph tool
one review graph tool
Clarity optional
Semgrep/ast-grep optional
```

Decision gate:

```text
If broker default does not beat best individual tool or rg baseline on DQPT,
stop broker work and use the best tool directly.
```

Good outcome:

```text
Codex asks one surface and gets cleaner next reads/tests.
```

Bad outcome:

```text
Broker becomes another wrapper with no decision lift.
```

Stop if bad.

### Stage 3: Build The Public Graph Benchmark

Purpose:

```text
Create the test that decides whether a graph core is needed.
```

Build:

```text
fixtures
YAML cases
expected nodes
expected edges
false-positive controls
false-negative controls
stale-index cases
token-budget cases
Codex decision tasks
markdown report
```

Decision gate:

```text
If external tools plus broker pass public graph benchmark strongly, do not build
Parceltongue core.
```

Continue only if:

```text
publicness is missed
freshness is absent
stable identity fails
edge evidence is weak
token output is too noisy
```

This stage prevents speculative core work.

### Stage 4: Build TypeScript Public Graph Slice

Purpose:

```text
Prove Parceltongue core can own one high-value public graph slice.
```

Build:

```text
TypeScript fixture
package boundary extraction
exports/reexports
Express/Fastify route extraction
handler/type links
config keys
test links
generated file warning
SQLite facts
ptctx public-impact adapter
```

Decision gate:

```text
If TypeScript slice does not improve public graph benchmark over external tools,
stop core work.
```

Continue only if:

```text
TypeScript public-impact is clearly better, fresher, more compact, or more
trustworthy than the external tool stack.
```

This is the first true Parceltongue v2 proof.

### Stage 5: Add Rust Public Graph Slice

Purpose:

```text
Support user's Rust systems and CLI work.
```

Build:

```text
Cargo metadata
pub/pub(crate)/pub use visibility
public structs/enums/traits/functions
trait/impl edges
feature flags
clap CLI surfaces
axum route surfaces if needed
FFI detection
test links
cargo check/test recommendations
```

Decision gate:

```text
If Rust slice cannot correctly distinguish pub, pub(crate), reexports, traits,
and features, stop before adding C/C++.
```

Rust must be correct enough to trust.

Otherwise it becomes worse than `cargo check` plus `rg`.

### Stage 6: Add C And C++ Public Graph Slice

Purpose:

```text
Support headers, exported symbols, and ABI-like risk.
```

Build:

```text
public header detection
include graph
declaration-definition linking
class public/protected/private methods
struct/enum/typedef risk
macros
extern C/export macros
build verification recommendations
```

Decision gate:

```text
If C/C++ slice cannot avoid overclaiming due to macros/preprocessor limits,
mark support partial and stop expansion until confidence model improves.
```

C/C++ support must be honest.

Partial with good warnings is acceptable.

Confident wrong is not.

### Stage 7: Add Python Public Graph Slice

Purpose:

```text
Support Python scripts, MCP servers, and web services.
```

Build:

```text
__all__
__init__.py reexports
FastAPI/Flask/Django routes
Click/Typer/argparse commands
Pydantic/dataclass models
config keys
pytest links
confidence downgrades for dynamic patterns
```

Decision gate:

```text
If Python extraction becomes too speculative, keep it as search/context assist
and do not use it for hard public impact claims.
```

Python graph value depends on honesty.

### Stage 8: Integrate Core And Broker Fully

Purpose:

```text
Make Parceltongue core one strong adapter inside ptctx.
```

Build:

```text
adapter health
core freshness
core public-impact
fallback to external tools
disagreement reporting
benchmark comparison
Codex operating manual update
```

Decision gate:

```text
If broker with core does not beat broker without core, do not default to core.
```

This keeps the system evidence-driven.

### Stage 9: Codex Habit Formation

Purpose:

```text
Make the tool a daily workflow, not a novelty.
```

Deliver:

```text
AGENTS.md snippet
Codex skill or local instruction
examples
pre-edit ritual
post-edit ritual
debug ritual
review ritual
large-repo orientation ritual
```

Measure:

```text
Does Codex call the broker at the right moments without being reminded?
Does the user feel less need to steer file exploration manually?
Does final output include better evidence?
```

If not, the product surface is still wrong.

### Stage 10: Stop Or Expand

After TypeScript, Rust, C/C++, and Python slices, stop and decide.

Do not automatically add more languages.

Expansion criteria:

```text
user actually works in the language
public graph benchmark exists
query pack can be honest
existing tools are insufficient
Codex workflow improves measurably
```

Possible next languages only if justified:

```text
Go
Java
Kotlin
C#
PHP
```

But no speculative language collection.

### Roadmap Summary

| Stage | Build | Continue Only If |
|---|---|---|
| 0 | Codex plus shell baseline | Baseline is insufficient. |
| 1 | Existing tool stack trial | Stack has named gaps. |
| 2 | Thin broker MVP | Broker beats individual tools or reduces workflow friction. |
| 3 | Public graph benchmark | Existing tools fail public graph needs. |
| 4 | TypeScript slice | Core improves public-impact decisions. |
| 5 | Rust slice | Rust publicness is modeled correctly. |
| 6 | C/C++ slice | Header/ABI risk can be represented honestly. |
| 7 | Python slice | Dynamic uncertainty is handled honestly. |
| 8 | Broker/core integration | Core improves broker benchmark. |
| 9 | Codex habit | Codex actually uses it well. |
| 10 | Expansion | New language has real user need and benchmark proof. |

### Criteria For Stopping

Stop at external tool stack if:

```text
it solves 85+ DQPT
setup friction is acceptable
Codex can use it without confusion
public graph needs are satisfied
freshness is clear enough
```

Stop at broker if:

```text
broker solves tool sprawl
external tools remain good enough underneath
core graph gaps are not proven
```

Stop after benchmark if:

```text
public graph benchmark is passed by external stack
```

Stop after TypeScript if:

```text
the first Parceltongue core slice does not beat external tools
```

Stop after Rust/C/C++/Python slice if:

```text
support is too speculative or not used in real Codex work
```

Stopping is success if the user has the needed leverage.

Stopping is not failure.

### What Success Looks Like

Success is not:

```text
Parceltongue has many features.
```

Success is:

```text
Codex reads fewer wrong files.
Codex finds public impact before editing.
Codex knows which tests matter.
Codex reports stale graph uncertainty.
Codex handles CRUD and systems repos with less user steering.
```

The user should feel:

```text
I can point Codex at a large repo and it finds the dependency path faster.
```

That is the felt product.

### Final Recommendation

The final recommendation from this research sequence is:

```text
Do not jump directly into a full Parceltongue v2 rebuild.
```

Instead:

```text
1. Trial the best existing Codex-callable tool stack.
2. Build a thin broker only if tool sprawl hurts.
3. Build the public graph benchmark before building core.
4. Build TypeScript public graph only if the benchmark proves a gap.
5. Add Rust, C/C++, and Python only after TypeScript proves the core pattern.
6. Keep external tools for search, security, structural rewrites, review, and
   compiler truth.
```

This is the least wasteful path.

It preserves the original Parceltongue insight:

```text
agents need graph-backed context
```

while avoiding the trap:

```text
therefore build everything yourself
```

### Concept 23 Conclusion

The roadmap should be a sequence of earned bets:

```text
baseline
tool stack
broker
benchmark
TypeScript core
Rust core
C/C++ core
Python core
broker integration
Codex habit
stop or expand
```

Each stage has a stop gate.

That is the difference between a research repo and a useful personal tool.

### Repos And Docs Already Touched For This Concept

| Source | Status | Notes |
|---|---|---|
| Concept 10 in this document | Source-read through current synthesis | Supplies DQPT benchmark and decision-quality metric. |
| Concept 11 in this document | Source-read through current synthesis | Supplies staged decision between tool stack, broker, and Parceltongue evolution. |
| Concept 12 in this document | Source-read through current synthesis | Supplies broker MVP. |
| Concept 16 in this document | Source-read through current synthesis | Supplies public graph benchmark. |
| Concept 17 in this document | Source-read through current synthesis | Supplies minimal core. |
| Concepts 18-21 in this document | Source-read through current synthesis | Supply language-slice sequence. |
| Concept 22 in this document | Source-read through current synthesis | Supplies broker/core/external-tool coexistence strategy. |

## Concept 24: Appendix One-Page Executive Summary

### Final Recommendation

Do not start with a full Parceltongue v2 rebuild.

Start with this sequence:

```text
1. Trial the best existing Codex-callable tool stack.
2. Build a thin `ptctx` broker only if tool sprawl hurts.
3. Build the public interface graph benchmark before building any core.
4. Build TypeScript public graph core only if the benchmark proves a gap.
5. Add Rust, C/C++, and Python only after TypeScript proves value.
```

This is the highest-leverage path for the stated PMF:

```text
solo Codex power user
large codebases
CRUD apps
Rust/C/C++ systems programming
dependency clarity
minimum useful context
```

### The Simple Truth

Codex is the agent.

Parceltongue, if it evolves, should be the context graph layer.

The job is not:

```text
build a mega coding agent
```

The job is:

```text
help Codex know what to read, edit, test, and distrust in a large repo
```

### First Tool Stack To Try

Start with:

```text
one graph brain
one search layer
one review graph
Clarity
rg/git/shell
```

Candidate first stack:

```text
GitNexus or codebase-memory-mcp
code-review-graph
cocoindex-code or codemogger
Clarity
ast-grep
Semgrep
```

Do not install ten tools and call that strategy.

Start small.

### First Codex Ritual

Before a meaningful edit:

```text
check repo state
ask for context
read ranked files
edit smallest responsible area
ask for impact after diff
run focused tests
report evidence
```

If `ptctx` exists:

```bash
ptctx health --json
ptctx search --query "feature or symbol" --budget 1500 --json
ptctx impact --query "symbol" --budget 3000 --json
ptctx review --diff --budget 4000 --json
ptctx tests --diff --budget 1200 --json
```

If `ptctx` does not exist yet:

```text
Use the graph/search/review tools directly and record where the workflow feels
fragmented.
```

That fragmentation is the broker signal.

### When To Build The Broker

Build the broker if:

```text
the tools are useful individually
Codex keeps choosing the wrong one
outputs are inconsistent
freshness is unclear
tool disagreement is hidden
answers are too verbose
the user has to manually merge results
```

Broker MVP:

```text
ptctx health
ptctx project-map
ptctx search
ptctx symbol
ptctx impact
ptctx route
ptctx review
ptctx tests
```

The broker should not build new parsers first.

It should route and normalize.

### When To Build Parceltongue Core

Build Parceltongue core only if the benchmark proves existing tools cannot
provide:

```text
stable identity
fresh incremental graph facts
public interface impact
evidence spans
confidence and uncertainty
token-budgeted context
```

Core MVP:

```text
stable IDs
incremental freshness
safe Tree-sitter parser lifecycle
relationship fact schema
public interface graph
TypeScript public graph slice
ptctx adapter
benchmark proof
```

### First Concrete Core Slice

If building core, start with TypeScript public graph:

```text
package.json exports/bin
index.ts reexports
Express/Fastify routes
handler/type links
config keys
route tests
generated file warnings
SQLite fact tables
ptctx public-impact
```

Do not start with all languages.

Do not start with UI.

Do not start with embeddings.

### Stop Rules

Stop if:

```text
external tool stack solves the job
broker does not improve DQPT
public graph benchmark passes without Parceltongue core
TypeScript slice does not beat external tools
Rust/C/C++/Python slices become speculative
Codex does not actually use the workflow
```

Stopping is allowed.

The goal is leverage, not ownership.

### The Next Actual Action

The next practical action should be:

```text
Run a small benchmark comparing Codex+rg against the current best external
tool stack on 5 real tasks:

1. Find the right files for a CRUD route change.
2. Find public impact of a TypeScript type change.
3. Find Rust public API impact of a `pub` item change.
4. Find C/C++ header impact of a function signature change.
5. Review a small diff and identify focused tests.
```

Record:

```text
correct next reads
correct next tests
tokens used
tool calls used
freshness clarity
wrong/confident claims
setup friction
```

That gives the first real decision.

### One-Line Answer

The one-line answer is:

```text
Use Codex as the agent, trial the best existing context tools, build a thin
broker if orchestration hurts, and evolve Parceltongue only as a benchmark-proven
fresh public relationship graph core.
```

## Concept 25: Treat The Official Tree-sitter Core Repo As The Contract Source

### Why This Concept Exists

The previous concepts synthesized a product direction.

The active research goal is broader:

```text
Browse repo by repo through git-ref-repo and capture Tree-sitter and similar
implementation patterns that can become an encyclopedia for Parceltongue.
```

The local inventory currently has:

```text
609 repos under git-ref-repo/ignore-this-folder-repos
```

The existing document had direct path-cited evidence for only a small subset of
those repos. That means the file is valuable, but not complete.

This concept starts the next repo-by-repo pass with the most authoritative repo:

```text
tree-sitter__tree-sitter
```

The official core repo should be treated as the contract source because it
defines the basic parse lifecycle, edit lifecycle, query model, included-range
model, external scanner lifecycle, and parser testing conventions.

For Parceltongue, this repo should answer:

```text
What does Tree-sitter itself expect consumers and parser authors to treat as
stable contracts?
```

### Codebase-Memory Evidence Status

The repo was indexed with `codebase-memory-mcp` during this continuation pass.

Index run:

```text
/tmp/codex-code-intel/codebase-memory/tree-sitter__tree-sitter-20260706-222912
```

The index reported:

```text
7915 nodes
27149 edges
```

The graph query follow-up needed project-name handling that did not work cleanly
in this session, so all claims below are grounded in direct file reads with line
references.

The graph run still matters because it confirms this repo has been included in
the codebase-memory-backed browsing pass.

### Pattern 1: Source Input Should Be An Adapter, Not Just A String

Tree-sitter supports parsing a flat string, but the core docs explicitly call
out `TSInput` for custom source storage such as piece tables and ropes.

The relevant shape:

```c
typedef struct {
  void *payload;
  const char *(*read)(void *payload, uint32_t byte_offset,
                      TSPoint position, uint32_t *bytes_read);
  TSInputEncoding encoding;
  TSDecodeFunction decode;
} TSInput;
```

Parceltongue implication:

```text
Do not design parsing around "read whole file into one string" as the only
source model.
```

The source abstraction should support:

```text
plain file content
rope or piece-table content
dirty editor buffer content
subrange content
custom decoding
future Codex working-buffer patches
```

For a Codex context graph, this matters because Codex often works in a dirty
worktree.

The graph should eventually be able to ask:

```text
Am I indexing the committed file, the dirty file, or a hypothetical edited
buffer?
```

The official `TSInput` API gives the right architectural hint:

```text
make the parser consume a source provider
```

not:

```text
make the parser own file IO
```

Recommended Parceltongue interface:

```text
SourceProvider:
  read_chunk(byte_offset, point) -> bytes
  encoding() -> utf8 | utf16 | custom
  content_hash() -> hash
  source_kind() -> committed_file | dirty_file | editor_buffer | synthetic_fixture
```

This keeps parser logic separate from storage and editor state.

### Pattern 2: Store Both Byte Spans And Point Spans

The core docs show nodes exposing both byte offsets and row/column points.

For Parceltongue, every extracted entity and edge evidence span should store
both:

```text
start_byte
end_byte
start_point
end_point
```

Why both matter:

| Span Type | Use |
|---|---|
| byte span | stable slicing, hashing, source snippets, UTF-aware parser APIs |
| point span | human display, editor navigation, markdown reports |

Do not store only line numbers.

Line-only identity already caused trouble in the existing incremental-indexing
RCA.

The better evidence object:

```json
{
  "file": "src/auth/tokenService.ts",
  "start_byte": 320,
  "end_byte": 680,
  "start_point": {"row": 18, "column": 0},
  "end_point": {"row": 39, "column": 1}
}
```

Codex needs point spans.

The indexer needs byte spans.

Stable identity needs content and semantic path, not mutable row numbers.

### Pattern 3: Named Nodes And Field Names Are Query-Pack Contracts

The official docs distinguish named and anonymous nodes.

Named nodes behave more like an AST.

Anonymous nodes preserve concrete syntax.

Parceltongue query packs should make this distinction explicit.

Use named nodes for:

```text
function declarations
class declarations
type definitions
call expressions
route declarations
imports
exports
tests
```

Use anonymous nodes for:

```text
operators
punctuation-sensitive rewrites
exact token matching
format-sensitive languages
syntax-highlighting-like tasks
```

The docs also show field-name access and field-id lookup.

Parceltongue implication:

```text
Do not write query packs that depend only on child indexes when grammar fields
exist.
```

Prefer:

```text
function_declaration name: (identifier)
```

over:

```text
child 0 is name
```

For hot paths, cache field ids where bindings support it.

Field names become a contract between grammar and extractor.

This is especially important for public graph extraction:

```text
route path
handler
function name
type annotation
class body
trait method
```

Those should be field-driven when possible.

### Pattern 4: Incremental Parse Has A Required Edit Protocol

The advanced parsing docs make the lifecycle explicit:

```text
1. Edit the old tree with TSInputEdit.
2. Reparse with the old tree.
3. Refetch nodes from the new tree, or edit cached TSNode instances too.
```

Parceltongue implication:

```text
Never treat a cached TSNode as position-stable after an edit unless it has been
updated with the same edit.
```

Better rule:

```text
Cache stable entity IDs and evidence spans.
Do not cache raw parser node handles across indexing transactions.
```

If an incremental indexer does cache node handles for performance, it must track:

```text
edit start byte
old end byte
new end byte
old/new points
tree edit applied
node edit applied or node refetched
```

For Parceltongue, the safest v2 shape is:

```text
Tree-sitter nodes are transient extraction handles.
Relationship facts are durable graph records.
```

This is a core architecture pattern.

### Pattern 5: Included Ranges Are The Primitive For Multi-Language Files

The official docs show multi-language documents using `TSRange` plus
`ts_parser_set_included_ranges`.

The important conceptual model:

```text
Parse the parent language.
Extract ranges for embedded child languages.
Parse those ranges with child parsers.
Let application logic mediate cross-language composition.
```

Tree-sitter does not automatically decide how ERB, HTML, and Ruby interact.

The application does.

Parceltongue implication:

```text
Multi-language support should be modeled as a parent parse plus child parse
jobs over included ranges.
```

This applies to:

```text
HTML with script/style
Markdown with fenced code
MDX
Vue/Svelte/Astro
ERB/EJS
PHP with HTML
Ruby heredocs
JavaScript regex literals
SQL strings if intentionally extracted
GraphQL strings if intentionally extracted
```

The graph should represent:

```text
file contains embedded_range
embedded_range parsed_as language
embedded_node belongs_to parent_node
cross_language_edge confidence
```

Example relationship facts:

```text
markdown_file contains fenced_code_range
fenced_code_range parsed_as rust
rust_function defined_in embedded_range
embedded_range owned_by markdown_section
```

For agentic code assist, this is huge.

A Codex agent should not see a Markdown code block, Vue `<script>`, or MDX
expression as a fake top-level file with no provenance.

It should see:

```text
embedded source with parent-file provenance
```

### Pattern 6: Tree Copies Are Cheap, But Individual Trees Are Not Thread-Safe

The advanced parsing docs state that copying a syntax tree is cheap because it
increments an atomic reference count, but individual `TSTree` instances are not
thread-safe for simultaneous use.

Parceltongue implication:

```text
Use tree copies as read handles for worker tasks, not shared mutable tree
instances.
```

Worker model:

```text
parse worker owns parser and tree
analysis worker receives copied tree or extracted facts
index transaction stores facts
query workers read graph facts, not live parser trees
```

This supports:

```text
parallel extraction
safe query serving
incremental reparse isolation
no shared mutable parser state
```

For Codex, the user-facing behavior is:

```text
graph queries remain safe while indexing proceeds
```

Internally, this means:

```text
do not pass one mutable TSTree around many threads
```

### Pattern 7: External Scanner State Is Part Of Incremental Correctness

The external scanner docs define five lifecycle functions:

```text
create
destroy
serialize
deserialize
scan
```

The most important Parceltongue lesson is not how to write a scanner.

It is this:

```text
scanner state must be serialized into the syntax tree so edits and ambiguities
can restore the scanner correctly
```

If Parceltongue uses grammars with external scanners, it should treat scanner
state as part of parse correctness.

Risk:

```text
If scanner state is incomplete or expensive to serialize, incremental parsing
can become wrong or slow.
```

For parser selection and quality scoring, add a query-pack health field:

```text
external_scanner: none | stateless | serialized_state | unknown
```

For parser safety docs, record:

```text
scanner state size
scanner reset behavior
known edge cases around included ranges
```

This is especially relevant for indentation-sensitive languages and template
languages.

### Pattern 8: Valid Symbols Are A Parser-Driven Guardrail

The external scanner `scan` function receives `valid_symbols`.

The docs say scanner logic should only look for a token when it is valid
according to that array.

Parceltongue implication:

```text
When extracting relationships, prefer parser-context facts over blind lexical
matches.
```

This maps beyond external scanners.

For code intelligence:

```text
Do not treat every identifier text as a reference.
Do not treat every string as a route.
Do not treat every exported-looking name as a package public API.
Use syntactic context to decide whether a token is valid for the fact being
extracted.
```

The scanner design is a lower-level version of the same principle:

```text
context constrains interpretation
```

### Pattern 9: Query Packs Should Be Split By Behavior, Not One Giant Query

The syntax-highlighting docs describe three query files:

```text
highlights
locals
injections
```

Each has different semantics.

Parceltongue should copy the architectural pattern, not the exact feature.

Instead of one giant `queries.scm`, use behavior-specific packs:

```text
symbols.scm
imports.scm
exports.scm
calls.scm
routes.scm
types.scm
locals.scm
tests.scm
config.scm
injections.scm
```

Why:

```text
different outputs have different confidence models
different facts need different evidence shape
different queries change at different rates
different languages may support only some packs
```

A language can honestly declare:

```json
{
  "symbols": "supported",
  "imports": "supported",
  "calls": "partial",
  "routes": "unsupported",
  "injections": "supported"
}
```

This is how Codex avoids trusting unsupported facts.

### Pattern 10: Local Variables Query Is A Mini Symbol Resolver

The highlighting docs describe fixed captures:

```text
@local.scope
@local.definition
@local.reference
@ignore
```

This is a lightweight symbol-resolution model.

Parceltongue implication:

```text
Before building a full semantic resolver, capture local scopes, definitions,
references, and ignore regions.
```

This can power:

```text
local variable shadowing warnings
definition/reference links inside a function
better call extraction
less false-positive identifier matching
syntax-aware highlighting in reports
```

For agentic code assist, this matters because Codex often asks:

```text
Is this identifier the same thing as that identifier?
```

A local-scope query pack can answer part of that without a compiler.

### Pattern 11: Injection Queries Are The Declarative Front Door To Embedded Code

The syntax-highlighting docs show captures:

```text
@injection.content
@injection.language
```

and properties:

```text
injection.language
injection.combined
injection.include-children
injection.self
injection.parent
```

Parceltongue implication:

```text
Do not hard-code all embedded-language logic in Rust code.
```

Use injection query packs as declarative metadata where possible.

Relationship facts:

```text
node injects language
node provides injection.content
injection combined true/false
injection includes children true/false
```

This gives Codex better context:

```text
This SQL string was parsed because an injection query identified it as SQL.
Confidence: static injection rule, not runtime execution.
```

That distinction matters.

### Pattern 12: Corpus Tests Are Parser API Documentation

The official test docs say corpus entries pair input source with expected
S-expression output.

The docs also describe attributes:

```text
:cst
:error
:fail-fast
:language(LANG)
:platform(PLATFORM)
:skip
```

Parceltongue should treat query-pack and parser fixtures the same way:

```text
fixtures are executable documentation for extraction contracts
```

For every language query pack, maintain corpus-style fixtures for:

```text
symbols
imports
exports
calls
routes
tests
config
injections
expected errors
multi-language cases
platform-specific syntax
```

Example fixture shape:

```text
==================
TypeScript Express route
:language(typescript)
==================

router.post("/users", createUserHandler)

---

facts:
  - route POST /users routes_to createUserHandler
```

This extends Tree-sitter's grammar testing philosophy into code-intelligence
fact testing.

### Direct Parceltongue Design Changes From This Repo

From `tree-sitter__tree-sitter`, Parceltongue should adopt these concrete
design rules:

| Rule | Why |
|---|---|
| Parse from `SourceProvider`, not file strings only | Supports dirty buffers, ropes, custom decoding, and synthetic fixtures. |
| Store byte spans and point spans | Supports stable slicing plus human navigation. |
| Prefer named nodes and fields | Keeps extractors robust against grammar shape changes. |
| Treat `TSNode` handles as transient | Prevents stale node positions after edits. |
| Use included ranges for embedded languages | Gives mixed files parent/child provenance. |
| Copy trees for cross-thread reads | Avoids unsafe shared tree use. |
| Track external scanner state risk | Scanner serialization affects incremental correctness. |
| Split query packs by behavior | Lets Codex trust supported fact types and distrust unsupported ones. |
| Use local-scope queries as lightweight resolver | Reduces false reference edges. |
| Use injection queries as declarative embedded-language rules | Avoids hard-coded language composition. |
| Treat fixtures as API docs | Makes extraction behavior reviewable and testable. |

### What To Add To Parceltongue v2 Architecture

Add these contracts:

```text
SourceProvider
ParseSession
EditTransaction
IncludedRangeJob
QueryPackCapability
ExtractionFixture
ScannerRiskProfile
```

Sketch:

```text
ParseSession:
  parser_id
  language_id
  source_provider_id
  included_ranges
  old_tree_id
  parse_diagnostics

EditTransaction:
  source_before_hash
  source_after_hash
  input_edit
  tree_edit_applied
  cached_nodes_invalidated

QueryPackCapability:
  language
  fact_type
  support_level
  query_file
  confidence_default
  fixture_suite
```

This would make the core Tree-sitter lifecycle visible at the product layer.

### Concept 25 Conclusion

The official Tree-sitter core repo changes how Parceltongue should think about
parsing.

Tree-sitter is not just:

```text
parse file to AST
```

It is a set of lifecycle contracts:

```text
source input adapters
byte and point spans
named/anonymous node layers
field names
edit-before-reparse
included ranges
cheap tree copies with thread-safety boundaries
external scanner state
behavior-specific query files
local scope/reference captures
injection captures
corpus tests as parser API docs
```

Parceltongue should surface these as explicit architecture concepts, because a
Codex-facing graph tool fails when parse lifecycle assumptions are hidden.

### Repos And Docs Touched For This Concept

| Source | Evidence | Notes |
|---|---|---|
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/tree-sitter__tree-sitter-20260706-222912`; reported 7915 nodes and 27149 edges. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/docs/src/using-parsers/2-basic-parsing.md` | lines 5-63, 74-88, 124-188 | `TSInput`, custom decoding, byte/point spans, named nodes, field names and field ids. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/docs/src/using-parsers/3-advanced-parsing.md` | lines 3-35, 37-161 | Edit protocol, old-tree reparsing, included ranges, multi-language parsing, tree copy and thread-safety warning. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/docs/src/creating-parsers/4-external-scanners.md` | lines 43-169 | External scanner create/destroy/serialize/deserialize/scan lifecycle, scanner state, valid symbols, lexer functions, included-range scanner hook. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/docs/src/3-syntax-highlighting.md` | lines 68-210, 316-386 | Query file split, highlight captures, local scope/definition/reference captures, injections and injection properties. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/docs/src/creating-parsers/5-writing-tests.md` | lines 1-194 | Corpus tests, expected S-expressions, field names, attributes, multi-language tests, automatic parser compilation. |

## Concept 26: Use CLI Workflows As Fixture And Query-Pack Product Contracts

The official Tree-sitter CLI is not only a developer convenience. For
Parceltongue, it is evidence for how a parser system becomes usable by agents.

The useful thing is not just that the CLI can parse code. The useful thing is
that it gives parsing a set of explicit product surfaces:

```text
init config
init grammar
generate parser
build parser
parse files
test corpus
fuzz parser
query syntax tree
highlight file
generate tags
serve playground
dump known languages
complete shell commands
```

That command taxonomy is a much stronger product shape than:

```text
analyze this repo
```

For a Codex-facing Parceltongue, this matters because a coding agent should not
have to guess which stage of code understanding it is in. It should be able to
say:

```text
I am discovering languages.
I am selecting a parser.
I am building or loading parser artifacts.
I am parsing files.
I am running query packs.
I am validating fixtures.
I am generating tags.
I am producing a compact JSON summary for the next LLM step.
```

The CLI gives us an important lesson: context tools become reliable when their
workflows are visible, bounded, replayable, and testable.

### Repos Browsed For This Concept

| Repo | Local Path | What Was Inspected | Why It Matters |
|---|---|---|---|
| `tree-sitter/tree-sitter` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter` | `crates/cli`, `crates/config`, `crates/loader`, `crates/highlight` | Official reference for CLI workflow boundaries, parser loading, query execution, config discovery, highlighting, and Wasm parser packaging. |

### The Core Pattern

Tree-sitter separates parser operations into narrow verbs.

In `crates/cli/src/main.rs`, the command enum includes separate commands for
config initialization, grammar initialization, generation, build, parse, test,
versioning, fuzzing, query, highlight, tags, playground, language dumping, and
shell completion.

That split is the concept. A code intelligence tool becomes agent-friendly when
each user journey has its own command boundary.

For Parceltongue, this suggests that the product should not be one monolithic
`context` operation. It should have separate surfaces like:

```text
ptctx init-config
ptctx discover-languages
ptctx build-parsers
ptctx index
ptctx parse
ptctx query
ptctx explain-symbol
ptctx callers
ptctx callees
ptctx blast-radius
ptctx tags
ptctx fixtures
ptctx diff-context
ptctx json-summary
```

The names can change, but the shape matters.

An agent should be able to use one surface at a time, observe the result, and
decide the next move. This is the opposite of stuffing an entire repo into
context and hoping the model can maintain its own mental index.

### Why This Matters For Codex Agents

The Codex app already gives the agent a computer: filesystem, shell, git, local
tools, and source files. Parceltongue should complement that environment by
being the agent's semantic navigation layer.

The key journey is:

```text
LLM asks a small query
tool resolves the relevant graph neighborhood
tool explains why those nodes matter
LLM chooses next file or symbol
tool expands only the next useful frontier
```

The CLI pattern tells Parceltongue to make each step explicit.

Bad journey:

```text
agent: analyze repo
tool: here is a huge pile of files, entities, edges, and matches
agent: loses token budget, re-reads irrelevant code, asks again
```

Better journey:

```text
agent: discover project languages
tool: Rust, TypeScript, Markdown; parser support present for Rust and TS

agent: find route handler symbol Foo
tool: exact symbol, file span, public interface, direct callers

agent: give dependency neighborhood under 2500 tokens
tool: symbol, direct callees, direct callers, test files, config files

agent: expand only persistence layer callees
tool: selected frontier with confidence and omissions
```

This is the tree-sitter CLI lesson translated into Parceltongue terms: the tool
must expose staged operations, not only final answers.

### CLI Commands Are Product Journeys

Tree-sitter's commands map cleanly to Parceltongue product journeys.

| Tree-sitter CLI Surface | What It Does In Tree-sitter | Parceltongue Equivalent | Agent Value |
|---|---|---|---|
| `init-config` | Creates a config file location and default structure. | `ptctx init-config` | Gives Codex a stable local configuration source instead of prompt-only preferences. |
| `generate` | Turns grammar definitions into parser source. | `ptctx generate-adapter` or parser adapter setup | Makes parser capabilities reproducible and visible. |
| `build` | Compiles parser artifacts, including native or Wasm output. | `ptctx build-parsers` | Separates parser artifact problems from graph extraction problems. |
| `parse` | Parses files with selectable parser, debug, stats, formats, timeout, edits, and JSON summaries. | `ptctx parse` and `ptctx index` | Lets Codex verify parse quality and avoid trusting a stale index. |
| `test` | Runs corpus tests with include/exclude filters, update mode, stats, and JSON summary. | `ptctx fixtures test` | Makes graph/query behavior regression-testable. |
| `fuzz` | Exercises parser robustness with random edits and iterations. | `ptctx fuzz-edits` | Validates incremental edit and partial re-index behavior. |
| `query` | Runs Tree-sitter queries over source files with ranges, captures, testing, timing, and parser selection. | `ptctx query-pack run` | Gives Codex bounded semantic extraction without whole-file dumping. |
| `highlight` | Runs query packs for highlights/injections/locals and can check capture conformance. | `ptctx inspect-query-pack` | Helps validate query pack quality and capture naming. |
| `tags` | Extracts named definitions suitable for navigation. | `ptctx tags` or `ptctx public-interface` | Gives the agent a high-signal map of symbols before deep traversal. |
| `dump-languages` | Shows known language parsers. | `ptctx languages` | Lets agent confirm support and fallback behavior. |
| `playground` | Opens an exploratory parser UI. | `ptctx serve` or `ptctx graph-ui` | Human debugging surface for parser and graph weirdness. |

This table is more important than the implementation details. It says
Parceltongue should be built around user journeys, not around internal data
structures.

### Config Discovery Is A Reliability Pattern

Tree-sitter has a dedicated config crate. The config type keeps a file location
and generic JSON value. Component-specific callers parse their own view from
that JSON. The load order is explicit:

```text
explicit path if provided
TREE_SITTER_DIR/config.json
platform config directory
legacy macOS application-support location, with migration
legacy HOME/.tree-sitter/config.json fallback
initial empty config
```

The important detail is not the exact path. The important detail is that
configuration is resolved deterministically and can be overridden explicitly.

For Parceltongue, this means:

```text
explicit --config wins
PARCELTONGUE_DIR or PTCTX_DIR can point to local tool state
repo-local config can define parser/query-pack behavior
user-global config can define defaults
component-specific config sections should be parsed independently
the resolved config path should be shown in JSON summaries
```

Codex agents need this because hidden config creates hidden behavior. If a graph
tool gives different answers on two machines, the LLM needs to know why.

Concrete Parceltongue config sections could be:

```json
{
  "parserDirectories": ["vendor/tree-sitter-parsers"],
  "queryPacks": ["queries/rust", "queries/typescript"],
  "index": {
    "ignore": ["target", "node_modules", "git-ref-repo"]
  },
  "agent": {
    "defaultTokenBudget": 4000,
    "preferPublicInterfaces": true
  }
}
```

The Tree-sitter pattern suggests that every component should own its own config
view. The parser loader should not need to know about LLM token budgets. The
agent context selector should not need to know the details of dynamic library
loading. They can share a config file without sharing a giant config type.

### Loader Errors Should Be Typed, Not Flattened

The loader has a rich error enum. It distinguishes compilation failures,
compiler command failures, stale lock files, failed scope selection, unknown
scope, language selection by file name, grammar JSON failures, IO failures with
paths, dynamic library errors, timestamp comparison failures, query errors,
symbol errors, tar/curl failures, unsupported Wasm tool platforms, Wasm compiler
failures, and Wasm optimizer failures.

That is exactly the type of boring operational detail that makes an agent tool
trustworthy.

For Parceltongue, a failed graph answer should not just say:

```text
index failed
```

It should say which phase failed:

```text
config_failed
language_discovery_failed
parser_build_failed
parser_load_failed
source_read_failed
parse_failed
query_compile_failed
query_runtime_failed
graph_write_failed
context_selection_failed
```

And every failure should carry:

```text
file path if relevant
language if known
parser id if known
query pack if known
phase
recoverability
suggested next command
```

This matters because the LLM is not only reading the error. It is using the
error as its next action policy.

Bad error:

```text
Could not index repo.
```

Agent next action:

```text
guess
```

Good error:

```json
{
  "phase": "query_compile_failed",
  "language": "rust",
  "queryPack": "rust.calls.v1",
  "queryPath": "queries/rust/calls.scm",
  "source": "git-ref-repo/example/src/main.rs",
  "recoverable": true,
  "next": "Run ptctx query-pack test --language rust --query-pack rust.calls.v1"
}
```

Agent next action:

```text
repair query pack or skip low-confidence edges
```

This is a direct product requirement for Parceltongue.

### Parser Loading Is A Pipeline, Not A Black Box

The Tree-sitter loader does several distinct things:

```text
find language configurations
select language by scope, file name, current path, or first-line regex
read grammar metadata
compute parser output path
check parser.c, scanner, and external files
decide whether recompilation is needed by modified time
coordinate builds with a lock file
compile native dynamic library or Wasm parser
load the language function from the artifact
```

For Parceltongue, this means parser provenance should be part of the graph
metadata.

Every indexed file should be able to answer:

```text
which parser selected this file
why was that parser selected
which parser artifact was used
was it native or Wasm
which query packs ran
which language config matched
was the index built from current source or stale source
```

That looks like implementation detail, but it is product detail for agents. If
Codex sees a suspicious dependency edge, it needs to know whether the edge came
from a stable language config, a fallback grammar, a file-extension guess, or a
low-confidence regex.

### Rebuild And Cache Must Be Observable

The loader checks whether source inputs are newer than the compiled parser
artifact. It also uses lock files so concurrent builds do not corrupt the same
artifact.

Parceltongue needs the same concept for indexing:

```text
source file modified time
source content hash
parser artifact hash
query pack hash
graph schema version
context selector version
index produced at timestamp
index validity state
```

For agent use, "cache hit" is not enough. The tool should explain cache
validity:

```json
{
  "index": "repo-main-20260706",
  "valid": true,
  "reason": "source_hashes_match",
  "parserArtifact": "rust-tree-sitter-native-a1b2",
  "queryPack": "rust-code-navigation-v3",
  "graphSchema": "parceltongue-graph-v2"
}
```

When invalid:

```json
{
  "index": "repo-main-20260706",
  "valid": false,
  "reason": "query_pack_changed",
  "next": "ptctx index --rebuild --query-pack rust-code-navigation-v4"
}
```

This lets Codex decide whether to trust an old graph while debugging.

### Parse Mode Is The Model For Index Observability

Tree-sitter parse has many knobs:

```text
paths file
source paths
grammar path
dynamic library path
language name
scope selection
debug log
debug graph
Wasm mode
dot output
XML output
CST output
statistics
timeout
timing
quiet mode
edits
encoding
JSON summary
specific test number
rebuild
omit ranges
```

This is a strong hint for Parceltongue. An indexer should not only produce a
database. It should produce observability.

Minimum useful `ptctx index` outputs:

```text
files considered
files parsed
files skipped
parse failures
query pack failures
entities extracted
edges extracted
ambiguous edges
high-confidence edges
low-confidence edges
duration
bytes processed
source hash
index hash
```

Minimum useful `ptctx parse` outputs:

```text
root node kind
has parse error
error spans
missing nodes
language
parser provenance
parse duration
tree byte range
tree point range
```

The Tree-sitter CLI has `ParseSummary` and `Stats` types. That should inspire a
Parceltongue `IndexSummary`.

Possible shape:

```json
{
  "repo": "my-app",
  "successful": true,
  "languages": ["rust", "typescript"],
  "files": {
    "considered": 1832,
    "parsed": 1204,
    "skipped": 628,
    "failed": 3
  },
  "graph": {
    "entities": 44102,
    "edges": 97211,
    "publicInterfaces": 2180,
    "callEdges": 38190
  },
  "durationMs": 8124,
  "next": [
    "ptctx hotspots --limit 20",
    "ptctx query SymbolName --budget 4000"
  ]
}
```

For LLMs, this is not cosmetic. It is orientation.

### Test Mode Is The Model For Query-Pack Regression

Tree-sitter corpus tests are updateable, filterable, and summarizable. The CLI
lets users include or exclude test names, select files, update expected syntax
trees, show fields, show diff markers, show stats, show only overview, and emit
JSON.

Parceltongue needs the same discipline for query packs.

If a query pack extracts call edges, fixtures should make that behavior visible:

```text
fixture source file
expected entities
expected call edges
expected import edges
expected public interface nodes
expected unresolved references
expected confidence grades
```

Example:

```text
Fixture: rust_trait_impl_calls

Input:
  trait Store { fn save(&self); }
  impl Store for Db { fn save(&self) {} }
  fn run(s: &dyn Store) { s.save(); }

Expected:
  entity trait Store
  entity method Store::save
  entity impl Db as Store
  call run -> Store::save confidence interface-dispatch
```

This gives Parceltongue something Tree-sitter already has: a way to prove that
its extraction behavior still works after parser or query changes.

The agent benefit is direct. Codex can ask:

```text
before I trust this dependency graph, did the Rust call-edge fixtures pass?
```

The tool can answer:

```json
{
  "queryPack": "rust-code-navigation-v3",
  "fixtures": {
    "passed": 84,
    "failed": 0,
    "skipped": 2
  }
}
```

That is much better than undocumented confidence.

### Query Mode Is The Exact Shape Of Agent Context Retrieval

Tree-sitter query mode is especially relevant to Parceltongue because it already
has the right constraints:

```text
query path
grammar path or library path
language name
timing
quiet mode
paths file
source paths
byte range
row range
containing byte range
containing row range
scope
captures order
test mode
config path
specific test number
rebuild
```

The most important piece: queries can be bounded by byte ranges and point
ranges. This is the same idea Parceltongue needs for token-efficient code
context.

For a large codebase, the agent should not ask:

```text
give me all callers of everything
```

It should ask:

```text
run the call-edge query only for this file span
return callers and callees touching this byte range
include only direct dependencies
budget 2500 tokens
```

Tree-sitter query mode also distinguishes captures from matches. Parceltongue
should keep that distinction:

```text
match: one structural pattern instance
capture: named semantic span inside that pattern
fact: Parceltongue-normalized entity or edge emitted from captures
```

If Parceltongue hides the capture layer, debugging becomes hard. If it exposes
captures as trace evidence, Codex can inspect why an edge exists.

Suggested fact trace:

```json
{
  "fact": "call_edge",
  "from": "run",
  "to": "save",
  "confidence": 0.72,
  "queryPack": "rust.calls.v3",
  "queryPath": "queries/rust/calls.scm",
  "patternIndex": 12,
  "captures": [
    {
      "name": "call.function",
      "range": {
        "startByte": 144,
        "endByte": 148
      }
    }
  ]
}
```

That is agent fuel. It lets the LLM reason about whether to trust, expand, or
challenge a graph edge.

### Highlight Mode Shows How To Combine Query Packs Without Losing Boundaries

Tree-sitter highlighting combines injection, locals, and highlights queries into
one query source while preserving offset boundaries and pattern-index ranges.
It also creates a separate query for combined injections and disables those
patterns in the main query where needed.

The design lesson is subtle and valuable:

```text
combine query packs for runtime efficiency
preserve behavioral boundaries for diagnostics
```

Parceltongue can use the same principle.

For example, these query packs might be loaded together:

```text
rust.entities
rust.calls
rust.imports
rust.traits
rust.tests
rust.public-interface
```

But the output should still know which pack produced each fact:

```json
{
  "edge": "call",
  "queryPack": "rust.calls",
  "patternIndex": 4
}
```

That allows performance without losing explainability.

This is crucial for a self-use tool. When a graph seems wrong, you do not want
to debug the entire indexer. You want to ask:

```text
which query pack emitted this fact?
which capture made it?
which fixture protects this behavior?
```

### Locals, Highlights, And Injections Suggest Query-Pack Tiers

The highlight configuration keeps separate concepts:

```text
injections
locals
highlights
combined injections
special capture names
```

For Parceltongue, a similar split could be:

```text
syntax tier
  nodes, spans, named nodes, fields

entity tier
  functions, structs, classes, methods, modules, tests

relationship tier
  calls, imports, inheritance, implementation, dataflow-lite

scope tier
  local definitions, local references, shadowing, lexical scope

interface tier
  public APIs, exported symbols, route handlers, commands

context tier
  smart context packs for agent tasks
```

This tiering helps avoid one common trap: treating all graph edges as equal.

A local lexical reference is not the same product object as a public API edge.
An import edge is not the same as a dynamic dispatch call. A highlight capture is
not the same as a dependency edge. Parceltongue should model these differences
explicitly.

### Wasm Parser Packaging Is A Portable Parser Strategy

Tree-sitter's Wasm support shows another useful pattern.

The CLI can:

```text
derive grammar name from grammar.json
fallback to grammar.js metadata
produce tree-sitter-{grammar}.wasm
compile parser source and scanner to Wasm
validate imported symbols
fail if scanner uses unavailable C or C++ standard library symbols
load Wasm bytes into a Wasm language store
```

For Parceltongue, this suggests a long-term design:

```text
native parser for fastest local indexing
Wasm parser for portable, sandboxed, reproducible parser packs
query pack metadata tied to parser artifact hash
symbol import validation before indexing
```

That matters if Parceltongue grows across all languages for personal power use.
Some parsers will be easy native dependencies. Some will be better as portable
Wasm artifacts. Some may need external tools. The system should record which
kind of parser was used.

Possible parser provenance:

```json
{
  "language": "rust",
  "parserKind": "wasm",
  "artifact": "tree-sitter-rust.wasm",
  "artifactHash": "sha256:...",
  "queryPack": "rust-code-navigation-v3",
  "loadedAt": "2026-07-06T16:55:00Z"
}
```

### Tags Are The Public Interface Layer

Tree-sitter's CLI includes a `tags` command. That matters because tags are not
the whole graph, but they are often the right first thing for an agent.

For Codex using Parceltongue, the first question in a repo is rarely:

```text
what is every AST node?
```

It is usually:

```text
what are the important public surfaces?
where are the entry points?
which names should I search next?
```

So Parceltongue should treat tags as a first-class layer:

```text
public functions
exported classes
types and traits
route handlers
commands
configuration keys
database models
test fixtures
benchmark entry points
```

This layer can feed a very efficient agent journey:

```text
ptctx tags --language rust --budget 2000
ptctx explain-symbol MyService::handle_request
ptctx callers MyService::handle_request --depth 1
ptctx callees MyService::handle_request --depth 1
ptctx context MyService::handle_request --budget 4000
```

That is much more efficient than opening all files matching `service`.

### The Agent Contract Should Be JSON-First But Human-Readable

Tree-sitter CLI supports human output and JSON summaries. Parceltongue should do
the same.

Codex needs JSON for reliable follow-up actions:

```json
{
  "symbol": "MyService::handle_request",
  "file": "src/service.rs",
  "span": {
    "startLine": 42,
    "endLine": 91
  },
  "directCallers": 3,
  "directCallees": 11,
  "tests": 4,
  "recommendedNext": [
    "ptctx callers MyService::handle_request --depth 2",
    "ptctx context MyService::handle_request --budget 6000"
  ]
}
```

Humans need readable output:

```text
MyService::handle_request
  file: src/service.rs:42
  direct callers: 3
  direct callees: 11
  tests: 4
  next: expand callers, inspect tests, build 6000-token context
```

Both should come from the same underlying summary object.

### The Big Parceltongue Design Rule

The rule from Concept 26:

```text
Every Parceltongue operation should be both a user journey and a fixture target.
```

That means:

```text
If the agent can ask for it, there should be a way to test it.
If the tool emits it, there should be a trace to source spans.
If the graph stores it, there should be provenance.
If the output guides the LLM, there should be a compact JSON summary.
```

This is a stricter requirement than "build a dependency graph."

The dependency graph is only one layer. The product is the set of reliable
journeys that help the agent move through a large codebase.

### Proposed Parceltongue V2 Surfaces

Concept 26 suggests this possible CLI/API set:

| Surface | Human Command Shape | Agent Use |
|---|---|---|
| Config | `ptctx init-config`, `ptctx config show` | Find deterministic parser and query-pack settings. |
| Language Discovery | `ptctx languages`, `ptctx languages --path file.rs` | Confirm parser support before reading files. |
| Parser Build | `ptctx build-parsers --wasm` | Separate parser artifact setup from indexing. |
| Parse Inspect | `ptctx parse file.rs --json` | Validate syntax tree quality and parse errors. |
| Index | `ptctx index --json-summary` | Build graph with counts, failures, hashes, and provenance. |
| Fixtures | `ptctx fixtures test --query-pack rust.calls` | Prove extraction behavior before trusting graph output. |
| Query Pack | `ptctx query-pack run rust.calls file.rs --range 10:0-80:0` | Run bounded extraction for a span. |
| Tags | `ptctx tags --budget 2000` | Orient agent to public surfaces. |
| Callers | `ptctx callers Symbol --depth 1 --budget 2500` | Retrieve direct reverse dependency context. |
| Callees | `ptctx callees Symbol --depth 1 --budget 2500` | Retrieve direct forward dependency context. |
| Blast Radius | `ptctx blast-radius Symbol --depth 2` | Estimate change impact. |
| Context | `ptctx context Symbol --budget 4000 --why` | Return compact context with explanations and omissions. |
| Trace | `ptctx trace fact-id` | Explain which query/capture/source span produced a fact. |
| Serve | `ptctx serve` | Human graph and query-pack debugging UI. |

These are not all MVP commands. They are the product map.

### The Bi-Directional Workflow

Tree-sitter CLI mostly runs from source to parser result:

```text
source file
parser
tree
query
matches
output
```

Parceltongue needs a bi-directional workflow:

```text
source file
parser
tree
query captures
facts
graph
agent context
agent decision
next source span
next graph query
```

The agent should be able to move both ways:

```text
from source span to graph facts
from graph edge to source capture
from symbol to dependency neighborhood
from dependency neighborhood back to exact files
from parse failure to parser/query-pack fixture
from fixture failure to query pattern
```

This is the core of "code assistance for agents." Parceltongue should not just
answer questions. It should let the agent navigate the evidence trail.

### Shreyas Doshi Product Reading

From a Shreyas Doshi-style product lens, the CLI pattern is valuable because it
is not trying to be impressive. It is trying to make expert work repeatable.

The user is a solo agent power user. The product does not need onboarding
funnels, collaboration settings, or enterprise dashboards. It needs extremely
sharp workflow ergonomics:

```text
Can I ask the next smallest question?
Can I trust the answer?
Can I see why the answer exists?
Can I bound the token cost?
Can I replay the same extraction later?
Can I know when the graph is stale?
Can I jump from graph to source and source to graph?
```

The Tree-sitter CLI succeeds because each command answers one small operational
question. Parceltongue should copy that product temperament.

The PMF for this concept is high for personal Codex use because it directly
supports repeated workflows:

```text
new repo orientation
large refactor planning
bug trace through callers and callees
test impact search
public API inspection
parser/query-pack debugging
minimal-token context retrieval
```

The risk is building too much UI or too many graph abstractions before the CLI
journeys are reliable. The correct sequence is:

```text
CLI journey
fixture
JSON summary
source trace
Codex prompt pattern
optional UI
```

### What Parceltongue Should Steal Directly

Parceltongue should steal these patterns, not the exact code:

| Tree-sitter Pattern | Parceltongue Adaptation |
|---|---|
| Separate CLI commands by workflow stage | Separate agent journeys by semantic task. |
| Explicit config path plus platform/user fallback | Deterministic config resolution with visible provenance. |
| Component-specific config parsing from shared JSON | Parser, indexer, query packs, and agent budgets own their own config views. |
| Rich loader error taxonomy | Typed graph/index/query failures with suggested next command. |
| Parser artifact rebuild by modified time | Index validity by source hash, parser hash, query-pack hash, and graph schema. |
| Build lock file | Index/build lock to avoid concurrent corruption. |
| Parse stats and JSON summary | Index stats and context summary JSON. |
| Query byte and point ranges | Bounded context retrieval by source span. |
| Query capture test mode | Query-pack fixture tests for entities and edges. |
| Highlight query composition with preserved boundaries | Combined runtime query packs with per-pack provenance. |
| Wasm parser artifact naming and validation | Portable parser packs with import validation and provenance. |
| Tags command | Public interface layer for agent orientation. |

### Concrete Data Types Parceltongue Should Add

```text
ResolvedConfig
LanguageDiscoverySummary
ParserArtifactProvenance
ParseInspectionSummary
IndexSummary
QueryPackRunSummary
QueryPackFixtureSummary
FactTrace
ContextSelectionSummary
GraphValiditySummary
```

Sketch:

```text
ParserArtifactProvenance:
  language
  parser_kind
  artifact_path
  artifact_hash
  selected_by
  language_config_path
  query_pack_hash

FactTrace:
  fact_id
  fact_kind
  language
  file
  source_range
  query_pack
  query_path
  pattern_index
  capture_names
  confidence

ContextSelectionSummary:
  query
  token_budget
  tokens_used
  selected_symbols
  selected_files
  omitted_reasons
  next_recommended_queries
```

These types would make Parceltongue more than a graph database. They would make
it an agent navigation protocol.

### Concept 26 Conclusion

Tree-sitter's CLI shows that a parser ecosystem becomes powerful when it exposes
small, replayable workflows.

For Parceltongue, the lesson is:

```text
Do not only build a dependency graph.
Build the workflows around the graph.
```

The graph is useful when Codex can:

```text
discover languages
build or load parsers
parse and inspect failures
run bounded query packs
test extraction fixtures
index with provenance
retrieve dependency neighborhoods
trace graph facts back to source captures
receive compact JSON summaries
choose the next smallest useful query
```

That is the product shape. It is less magical, but much more reliable.

### Repos And Docs Touched For This Concept

| Source | Evidence | Notes |
|---|---|---|
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter` | codebase-memory indexed | Same official repo index used for Concept 25: `/tmp/codex-code-intel/codebase-memory/tree-sitter__tree-sitter-20260706-222912`, with 7915 nodes and 27149 edges. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/crates/cli/src/main.rs` | lines 45-76, 196-291, 300-361, 437-570 | CLI command taxonomy; parse/test/query/highlight/tags flags and workflow boundaries. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/crates/config/src/tree_sitter_config.rs` | lines 53-203 | Config file location, load order, explicit path, env var, XDG/platform fallback, legacy fallback, component-specific get/add behavior. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/crates/loader/src/loader.rs` | lines 81-140, 847-1045, 1078-1222, 1782-2019, 2288-2304 | Loader error taxonomy, language discovery, selection by scope/file/first line/injection, parser rebuild/load pipeline, lock-file coordination, modified-time rebuild checks. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/crates/cli/src/parse.rs` | lines 23-58, 139-230 | Parse statistics, parse output modes, serializable points, parse summary. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/crates/cli/src/query.rs` | lines 19-174 | Query options, byte/point range constraints, containing ranges, capture/match output, test mode, match-limit warning, timing. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/crates/cli/src/test.rs` | lines 32-180 | Corpus delimiter parsing, S-expression normalization, test entries, attributes, include/exclude/update/overview options, test summary. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/crates/highlight/src/highlight.rs` | lines 112-180, 338-430 | Highlight configuration fields, reusable highlighter, injection/locals/highlights query composition, special capture indexes. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter/crates/cli/src/wasm.rs` | lines 12-119 | Wasm parser load/build path, grammar-name lookup, deterministic artifact naming, scanner path, imported symbol validation. |

## Concept 27: Treat Grammar Repos As Versioned Language Packs

The next useful concept comes from browsing official Tree-sitter grammar repos.

The important thing is that these repos are not just parser implementations.
They are complete language packs.

A typical official grammar repo includes:

```text
grammar.js
tree-sitter.json
src/grammar.json
src/node-types.json
src/parser.c
src/scanner.c when needed
queries/highlights.scm
queries/injections.scm when needed
queries/locals.scm when needed
queries/tags.scm
test/corpus/*.txt
package.json
Cargo.toml
go.mod
pyproject.toml
Package.swift
binding.gyp
bindings/*
examples/*
```

This is a very strong pattern for Parceltongue.

Parceltongue should not think of language support as:

```text
some parser code
```

It should think of language support as:

```text
versioned language pack
  parser source
  generated parser artifact
  node-type schema
  query packs
  extraction fixtures
  package metadata
  binding adapters
  provenance
  known limitations
```

That language-pack framing matters because Parceltongue wants to help Codex
navigate large codebases reliably across Rust, TypeScript, JavaScript, Python,
Go, C, C++, Java, Bash, and more. A large-codebase agent needs to know not only
that a parser exists, but also which semantic affordances exist for that
language.

For example:

```text
Rust pack:
  has highlights
  has injections
  has tags
  has scanner
  has node types
  has corpus tests

JavaScript pack:
  has highlights split across base, JSX, and params
  has locals
  has injections
  has tags
  has scanner

TypeScript pack:
  has multiple grammars in one repo: typescript, tsx, flow
  reuses JavaScript queries
  has locals
  has tags
  has external shared scanner header

Python pack:
  has highlights
  has tags
  has scanner
  has Python 2 and Python 3 grammar cases
```

That is exactly the level of capability metadata a Codex agent needs before it
decides how much to trust a graph.

### Repos Browsed For This Concept

| Repo | Local Path | Evidence Type | Notes |
|---|---|---|---|
| `tree-sitter/tree-sitter-rust` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-rust` | codebase-memory plus direct source | Indexed with 1671 nodes and 3148 edges. Used for manifest, grammar, node types, tags, injections, corpus, package, Rust binding. |
| `tree-sitter/tree-sitter-javascript` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-javascript` | codebase-memory plus direct source | Indexed with 1486 nodes and 3215 edges. Used for manifest, grammar, locals, injections, package structure. |
| `tree-sitter/tree-sitter-python` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-python` | codebase-memory plus direct source | Indexed with 1438 nodes and 2699 edges. Used for manifest, grammar, tags, corpus structure. |
| `tree-sitter/tree-sitter-typescript` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-typescript` | direct source only | Codebase-memory run created a cache directory but exited before indexing a project; source files were inspected directly. Used for multi-grammar manifest and query reuse. |
| `tree-sitter/tree-sitter-go` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-go` | direct source | Used to compare simpler manifest shape. |
| `tree-sitter/tree-sitter-c` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-c` | direct source | Used for schema and binding variation. |
| `tree-sitter/tree-sitter-cpp` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-cpp` | direct source | Used for inherited query-pack shape from C. |
| `tree-sitter/tree-sitter-java` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-java` | direct source | Used as another simple manifest reference. |
| `tree-sitter/tree-sitter-bash` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-bash` | direct source | Used for file-type and first-line-regex language selection. |

### The Core Pattern

An official grammar repo is a bundle of contracts.

```text
grammar.js:
  declarative parser source

tree-sitter.json:
  language-pack manifest

src/grammar.json:
  generated grammar description

src/node-types.json:
  static node-type API contract

src/parser.c and scanner files:
  generated or custom parser implementation

queries/*.scm:
  semantic extraction layers

test/corpus/*.txt:
  executable syntax fixtures

bindings/*:
  language-specific packaging adapters

package metadata:
  distribution contract
```

For Parceltongue, the equivalent should be:

```text
parceltongue-language.toml or parceltongue.language.json
parser artifact
node-type schema
query packs
graph extraction fixtures
agent prompt snippets
known blind spots
confidence rules
version metadata
```

The key idea:

```text
Do not install language support as code only.
Install language support as evidence, queries, fixtures, and metadata.
```

### The Manifest Pattern

The official grammar repos use `tree-sitter.json` as the language manifest.

Rust declares:

```text
name: rust
scope: source.rust
path: .
file-types: rs
highlights: queries/highlights.scm
injections: queries/injections.scm
tags: queries/tags.scm
injection-regex: rust
metadata: version, license, description, authors, repository
bindings: c, go, node, python, rust, swift
```

JavaScript declares:

```text
name: javascript
scope: source.js
file-types: js, mjs, cjs, jsx
highlights: base highlights, jsx highlights, parameter highlights
tags: queries/tags.scm
injection-regex: js or javascript
bindings: c, go, node, python, rust, swift
```

Python declares:

```text
name: python
scope: source.python
file-types: py
highlights: queries/highlights.scm
tags: queries/tags.scm
injection-regex: py
bindings: c, go, node, python, rust, swift
```

Go and Java have simpler manifests:

```text
name
scope
path
file-types
highlights
tags
metadata
bindings
```

Bash adds a useful first-line rule:

```text
file-types: sh, bash, .bashrc, .bash_profile, ebuild, eclass
injection-regex: shell, bash, sh
first-line-regex: shebang containing sh, bash, or dash
```

This is a direct design requirement for Parceltongue:

```text
Language selection must not be extension-only.
```

It should support:

```text
extension match
filename match
scope match
content regex
first-line regex
explicit override
repo-local config
```

If Codex asks about `scripts/deploy`, Parceltongue should be able to identify it
as Bash by first-line regex, not only by `.sh` extension.

### Multi-Grammar Repos Matter

The TypeScript repo is especially important because it is not one language
manifest entry. Its `tree-sitter.json` defines multiple grammars:

```text
typescript
tsx
flow
```

They share a repo and some external files. They also reuse JavaScript query
files from `node_modules/tree-sitter-javascript`.

That is a major pattern.

Parceltongue should not assume:

```text
one repo equals one language
one parser equals one query pack
one file extension equals one language
```

Better model:

```text
LanguagePack:
  pack_id
  repository
  version
  grammars[]
  shared_files[]
  query_pack_dependencies[]

GrammarCapability:
  grammar_id
  language_name
  scope
  file_types
  content_regex
  first_line_regex
  parser_artifact
  query_packs[]
```

The TypeScript repo also shows query inheritance:

```text
TypeScript highlights:
  local TypeScript highlights
  JavaScript base highlights

TSX highlights:
  local TypeScript highlights
  JavaScript JSX highlights
  JavaScript base highlights

TypeScript locals:
  local TypeScript locals
  JavaScript locals

TypeScript tags:
  local TypeScript tags
  JavaScript tags
```

Parceltongue needs this exact idea for multi-language ecosystems.

Example:

```text
TSX graph extraction:
  TypeScript entity queries
  JavaScript call queries
  JSX component queries
  import/export queries
  React-specific optional pack
```

The dependency between query packs should be explicit. Otherwise the agent will
not know whether a missing edge is due to parser failure, unsupported query
pack, or intentionally omitted framework logic.

### The Grammar Source Is Declarative Knowledge

In `grammar.js`, official repos declare more than syntax rules. They encode
language-specific parsing strategy:

```text
precedence tables
reserved words
extras
externals
supertypes
inline rules
conflicts
word rule
top-level source rule
```

Rust's grammar defines precedence, primitive/numeric type lists, punctuation,
external scanner tokens, supertypes for expressions/types/literals/patterns,
inline helper rules, conflicts, and `word` identifier behavior.

JavaScript's grammar defines external tokens for automatic semicolon behavior,
template content, JSX text, regex patterns, HTML comments, reserved words,
supertypes, inline rules, and precedence groups.

Python's grammar defines precedence, indentation/newline external tokens,
comments as external tokens, bracket-aware dedent handling, conflicts,
supertypes, inline rules, reserved keywords, and `word` identifier behavior.

For Parceltongue, this means:

```text
Graph extraction should respect language parsing strategy.
```

Some examples:

```text
Rust macros need injection and token-tree treatment.
JavaScript template literals need injection logic.
Python indentation requires external scanner behavior.
TypeScript type syntax needs separate type-reference queries.
Bash scripts may need first-line detection.
C++ query packs may depend on C query packs.
```

A generic AST traversal is not enough. The language pack should carry the
language-specific graph rules.

### Node Types Are The Static API Contract

The generated `src/node-types.json` file is a stable interface between parser
and tooling.

Rust's node types file lists named node categories and subtypes. For example,
the `_declaration_statement` supertype includes declarations such as:

```text
associated_type
attribute_item
const_item
empty_statement
enum_item
extern_crate_declaration
foreign_mod_item
function_item
function_signature_item
impl_item
let_declaration
macro_definition
macro_invocation
mod_item
static_item
struct_item
trait_item
type_item
union_item
use_declaration
```

This is gold for Parceltongue.

Query packs should be validated against node-type schema:

```text
does this query reference node kinds that exist?
does this query reference fields that exist?
does this language pack expose the node kinds needed by public-interface extraction?
does a parser upgrade remove or rename a node kind?
```

That gives Parceltongue a way to catch breaking parser/query changes before an
agent trusts the graph.

Possible command:

```text
ptctx query-pack check --language rust
```

Possible output:

```json
{
  "language": "rust",
  "queryPack": "rust-code-navigation-v4",
  "nodeTypesHash": "sha256:...",
  "queries": {
    "valid": 12,
    "invalid": 0
  },
  "requiredNodeKinds": [
    "function_item",
    "impl_item",
    "trait_item",
    "call_expression"
  ]
}
```

### Query Files Are Semantic Layers

The query files are the most directly Parceltongue-relevant part of the grammar
repos.

Rust `queries/tags.scm` extracts:

```text
structs, enums, unions as definition.class
type aliases as definition.class
methods as definition.method
functions as definition.function
traits as definition.interface
modules as definition.module
macros as definition.macro
call expressions as reference.call
macro invocations as reference.call
impl trait/type references as reference.implementation
```

That is almost a minimal code-navigation graph.

Python `queries/tags.scm` extracts:

```text
module-level assigned constants
class definitions
function definitions
calls by identifier or attribute
```

TypeScript `queries/tags.scm` extracts:

```text
function signatures
method signatures
abstract method signatures
abstract classes
modules
interfaces
type references
new-expression class references
```

JavaScript `queries/locals.scm` separates:

```text
local scopes
local definitions
local references
```

JavaScript `queries/injections.scm` handles:

```text
tagged template literal language inference
regex literal content as regex
JSDoc comments as jsdoc
hbs template literals as glimmer/handlebars-style content
```

This shows a natural Parceltongue taxonomy:

```text
tags queries:
  public-ish symbols and references

locals queries:
  lexical binding, local definitions, local references

injections queries:
  embedded languages and sub-documents

highlights queries:
  token classification, useful for UI and diagnostics

custom graph queries:
  calls, imports, inheritance, implementations, routes, tests
```

Parceltongue should not put all extraction into one query. It should keep these
layers separate and compose them deliberately.

### Query Pack Support Levels

Not every language pack has the same query files.

Observed examples:

```text
Rust:
  highlights, injections, tags

JavaScript:
  highlights, highlights-jsx, highlights-params, injections, locals, tags

TypeScript:
  highlights, locals, tags, plus JavaScript query dependencies

Python:
  highlights, tags

Go:
  highlights, tags

Bash:
  manifest selection rules, but the inspected manifest did not declare tags/highlights in the same way as Go/Rust
```

Parceltongue should expose this explicitly.

Example:

```json
{
  "language": "python",
  "support": {
    "parse": "stable",
    "tags": "stable",
    "locals": "missing",
    "injections": "missing",
    "callGraph": "basic",
    "typeGraph": "missing"
  }
}
```

This prevents overclaiming.

A Codex agent should be told:

```text
Python call graph is basic tags-query evidence only.
Rust implementation references are supported by tags query.
JavaScript locals are supported.
TSX support depends partly on JavaScript query packs.
```

That is how the LLM avoids hallucinating certainty.

### Corpus Fixtures Are The Model For Graph Fixtures

The corpus files in grammar repos follow a simple pattern:

```text
case title
source input
separator
expected syntax tree
```

For example, Rust declaration corpus cases include modules, extern crates,
function declarations, parameter shapes, return types, macro invocations, and
expected S-expressions.

TypeScript function corpus cases include typed parameters, generics, array and
tuple type arguments, optional chaining, call signatures, generators, and
expected syntax trees.

Python statement corpus cases include import statements, import-from statements,
future imports, print statements, and expected trees.

Parceltongue needs the graph equivalent.

The structure could be:

```text
================================================================================
Rust trait implementation call graph
================================================================================

trait Store {
    fn save(&self);
}

struct Db;

impl Store for Db {
    fn save(&self) {}
}

fn run(store: &dyn Store) {
    store.save();
}

--------------------------------------------------------------------------------

entities:
  definition.interface Store
  definition.method Store::save
  definition.class Db
  definition.implementation Db as Store
  definition.function run

edges:
  implementation Db -> Store
  call run -> Store::save confidence interface-dispatch

omissions:
  concrete dynamic target Db::save not guaranteed without type analysis
```

This is the right level of honesty. Parceltongue can extract useful graph facts
without pretending to be a full compiler.

### Package Metadata Shows Distribution Boundaries

Rust's `package.json` includes:

```text
grammar.js
tree-sitter.json
binding.gyp
prebuilds
bindings/node
queries
src
wasm artifacts
```

TypeScript's `package.json` includes:

```text
binding.gyp
prebuilds
bindings/node
queries
typescript/grammar.js
tsx/grammar.js
tree-sitter.json
typescript/package.json
tsx/package.json
typescript/src
tsx/src
common
wasm artifacts
tree-sitter-javascript dependency
```

That is a clear distribution boundary.

Parceltongue language packs should similarly declare what ships:

```text
manifest
parser artifact
node types
query packs
fixtures
adapter code
examples
known limitations
```

Possible package manifest:

```json
{
  "name": "parceltongue-rust-pack",
  "version": "0.1.0",
  "language": "rust",
  "parser": {
    "kind": "tree-sitter",
    "source": "tree-sitter/tree-sitter-rust",
    "version": "0.24.2"
  },
  "queries": {
    "tags": "queries/tags.scm",
    "calls": "queries/calls.scm",
    "imports": "queries/imports.scm",
    "impls": "queries/impls.scm"
  },
  "fixtures": "fixtures/**/*.graph.txt",
  "support": {
    "publicInterface": "stable",
    "callGraph": "basic",
    "typeGraph": "partial",
    "macroExpansion": "unsupported"
  }
}
```

The important point is not JSON versus TOML. The important point is that support
metadata ships with the pack.

### Bindings Show That Language Packs Need Multiple Consumers

Official grammar repos often expose bindings for multiple ecosystems:

```text
C
Go
Node
Python
Rust
Swift
sometimes Java or Zig flags
```

For Parceltongue, the equivalent consumers are:

```text
CLI
Codex app shell workflow
local HTTP server
MCP server
Rust library
possible Tauri UI
test harness
future index daemon
```

So the language-pack boundary should not be tied to one runtime.

A good internal boundary:

```text
LanguagePackManifest:
  loaded by CLI
  loaded by MCP server
  loaded by tests
  loaded by UI
  versioned independently
```

That makes Parceltongue more maintainable. Rust graph support can improve
without rewriting JavaScript support. TypeScript query-pack dependencies can be
modeled separately from Python tags support.

### Injections Are Required For Real Codebases

The official repos remind us that modern source files are not single-language
documents.

Rust has token-tree macro injections back into Rust.

JavaScript injects:

```text
tagged template literal content
regex patterns
JSDoc comments
Handlebars/Glimmer templates via hbs template calls
```

TypeScript and TSX depend on JavaScript injection behavior.

C++ can inherit C highlighting and has its own injections query.

For Parceltongue, injection is not only syntax highlighting. It is graph
correctness.

Examples:

```text
SQL inside a Rust sqlx macro
GraphQL inside a JavaScript gql template
HTML inside a TSX component
regex inside JavaScript
JSDoc type info inside comments
shell commands inside package scripts
```

The first version of Parceltongue does not need to solve all of these. But it
does need a model:

```text
EmbeddedDocument:
  host_language
  embedded_language
  host_file
  host_range
  embedded_range
  injection_query
  confidence
```

Then an agent can be told:

```text
This SQL string was detected as embedded SQL, but no SQL dependency graph pack is installed.
```

That is much better than silently ignoring it.

### The Public Interface Graph Can Start From Tags

The most practical immediate Parceltongue lesson is that `tags.scm` is a cheap
starting point for a public interface graph.

From tags queries alone, a first version can often extract:

```text
definitions
classes
interfaces
modules
functions
methods
macros
calls
implementations
type references
class references
```

This is not a full call graph. It is not compiler-grade type resolution. But it
is a useful navigation graph for a solo Codex power user.

The agent journey:

```text
ptctx tags --language rust --json
ptctx public-interface --language rust --budget 2000
ptctx callers some_function --evidence tags
ptctx trace edge-id
```

The output should label evidence:

```json
{
  "edge": "reference.call",
  "from": "unknown lexical container",
  "to": "println",
  "evidence": "tree-sitter-tags",
  "confidence": "syntactic"
}
```

As Parceltongue improves, deeper packs can add:

```text
lexical container resolution
import resolution
module path resolution
method receiver resolution
trait impl approximation
test linkage
route detection
database model relationships
```

But the tag layer gives immediate PMF.

### Confidence Should Be Pack-Specific

A language pack should define confidence semantics.

For example:

```text
Rust tags:
  function definitions: high syntactic confidence
  trait definitions: high syntactic confidence
  impl references: medium semantic confidence
  call identifier references: syntactic confidence only
  macro invocations: syntactic confidence only

JavaScript locals:
  local scope: high syntactic confidence
  local definitions: high syntactic confidence
  local references: syntactic confidence, shadowing-aware only inside query limits

Python tags:
  class/function definitions: high syntactic confidence
  calls: syntactic confidence only
  imports: available in parse tree, but not in inspected tags query
```

This is important because Codex needs to know when to ask for more evidence.

Example answer:

```text
I found 12 syntactic call references to `save`, but method receiver resolution is not installed for Rust. Next useful step: inspect the direct lexical containers and imports.
```

That is the kind of answer that helps an agent move faster without lying.

### Language Packs Should Include Agent Prompts

This is a Parceltongue-specific extension beyond official Tree-sitter repos.

Each language pack should include short prompt snippets for Codex:

```text
When using Rust graph output:
  - Treat macro invocation edges as syntactic unless macro expansion is enabled.
  - Treat trait dispatch edges as candidate edges unless type resolution proves target.
  - Prefer public-interface nodes before local helper nodes when under token budget.

When using JavaScript graph output:
  - Treat dynamic property calls as unresolved unless receiver analysis is available.
  - Inspect injections for gql, sql, hbs, html, and regex before assuming single-language context.

When using Python graph output:
  - Treat attribute calls as syntactic unless import/type inference is available.
  - Prefer import graph and class/function tags before line-by-line reading.
```

That makes the language pack useful directly inside Codex. The pack would not
only provide facts. It would teach the agent how to use those facts.

### Parceltongue V2 Language-Pack Data Model

Suggested data types:

```text
LanguagePackManifest
GrammarManifest
QueryPackManifest
FixtureSuiteManifest
NodeTypeSchema
LanguageSupportMatrix
ExtractionConfidencePolicy
EmbeddedLanguageRule
AgentUsageNotes
```

Sketch:

```text
LanguagePackManifest:
  pack_id
  source_repo
  source_version
  license
  grammars
  bindings
  package_files
  generated_at

GrammarManifest:
  name
  scope
  path
  file_types
  content_regex
  first_line_regex
  injection_regex
  parser_artifact
  node_types

QueryPackManifest:
  name
  language
  layer
  files
  dependencies
  support_level
  confidence_policy
  fixture_suite

FixtureSuiteManifest:
  source_cases
  expected_entities
  expected_edges
  expected_omissions
  expected_confidence
```

This gives Parceltongue a maintainable way to scale beyond a single language.

### Shreyas Doshi Product Reading

From a Shreyas-style product lens, the surprising insight is that language
support is not a feature checkbox.

Bad product framing:

```text
Supports Rust, TypeScript, Python, Go, C, C++.
```

Good product framing:

```text
Rust:
  public interface extraction: strong
  syntactic calls: strong
  trait impl approximation: partial
  macros: detected, not expanded
  embedded SQL: optional pack required

TypeScript:
  public interface extraction: partial
  JS query reuse: yes
  JSX/TSX support: yes
  import graph: needs custom pack
  framework routes: optional pack required
```

For a solo Codex power user, this is much more valuable. The user does not need
marketing support claims. The user needs operational truth.

The PMF implication:

```text
High PMF is not "many languages."
High PMF is "I know exactly what Parceltongue can and cannot tell Codex for this language."
```

That means the first high-quality product could support fewer languages but with
better honesty:

```text
Rust: strong syntactic graph plus public interface and impl candidate edges
TypeScript/JavaScript: strong tags/locals/injections plus import/call approximations
Python: tags/imports/calls basic with honest dynamic limits
Go: tags/imports/calls likely easier because language is simpler
C/C++: tags and include/reference approximations with caution around preprocessing
```

This is how Parceltongue avoids becoming a false-confidence machine.

### What Parceltongue Should Steal Directly

| Tree-sitter Grammar Repo Pattern | Parceltongue Adaptation |
|---|---|
| `tree-sitter.json` manifest | `LanguagePackManifest` with grammars, query packs, support levels, provenance. |
| Multiple grammars in one repo | Support one pack containing multiple grammar capabilities. |
| Query files by behavior | Separate tags, locals, injections, highlights, calls, imports, framework packs. |
| Query dependencies | Let TypeScript depend on JavaScript base packs; let C++ depend on C packs. |
| `src/node-types.json` | Validate query packs against parser node-type schema. |
| Corpus tests | Add graph fixtures with expected entities, edges, confidence, and omissions. |
| Package files include queries and generated sources | Package query packs, fixtures, parser artifacts, and provenance together. |
| Bindings for multiple ecosystems | Keep language packs runtime-neutral: CLI, MCP, HTTP, library, UI. |
| First-line regex and injection regex | Support non-extension language detection and embedded-language detection. |
| Rust/JS/Python scanner-specific behavior | Record scanner presence and language-specific limitations in support metadata. |

### Concept 27 Conclusion

Official grammar repos show that language support is a package, not a parser.

For Parceltongue, the durable concept is:

```text
Each language should ship as a versioned language pack with parser provenance,
node-type schema, query packs, fixtures, confidence policy, support matrix, and
agent usage notes.
```

This is how Parceltongue can scale without lying to Codex.

The tool should not say:

```text
I support TypeScript.
```

It should say:

```text
For TypeScript, I can parse TS/TSX/Flow through this grammar pack,
reuse JavaScript query packs for JSX and base JS behavior,
extract interface/function/class/type tags,
detect local parameter definitions,
and produce syntactic references.
I do not yet do full type-aware module resolution unless that pack is installed.
```

That is the kind of truth a coding agent can act on.

### Repos And Docs Touched For This Concept

| Source | Evidence | Notes |
|---|---|---|
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-rust` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/tree-sitter__tree-sitter-rust-20260706-223906`; list_projects reported 1671 nodes and 3148 edges. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-javascript` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/tree-sitter__tree-sitter-javascript-20260706-223906`; list_projects reported 1486 nodes and 3215 edges. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-python` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/tree-sitter__tree-sitter-python-20260706-223906`; list_projects reported 1438 nodes and 2699 edges. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-typescript` | codebase-memory attempted, direct source used | Index cache directory existed but list_projects reported no indexed projects after the run exited with code 143; direct source evidence used instead. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-rust/tree-sitter.json` | lines 1-53 | Rust manifest: grammar name, scope, file types, highlights, injections, tags, injection regex, metadata, bindings. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-javascript/tree-sitter.json` | lines 1-51 | JavaScript manifest: JS/JSX file types, multiple highlight query files, tags, injection regex, bindings. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-typescript/tree-sitter.json` | lines 1-104 | Multi-grammar pack: TypeScript, TSX, Flow; shared scanner header; JavaScript query dependencies; content regex. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-python/tree-sitter.json` | lines 1-42 | Python manifest: scope, file type, highlights, tags, injection regex, metadata, bindings. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-go/tree-sitter.json` | lines 1-41 | Simpler manifest with highlights/tags and bindings. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-c/tree-sitter.json` | lines 1-46 | C manifest with schema reference, file types, injection regex, and binding variation including Zig. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-cpp/tree-sitter.json` | lines 1-55 | C++ manifest: header/source extensions, C query dependency for highlights, injections, tags, injection regex. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-java/tree-sitter.json` | lines 1-45 | Java manifest with highlights/tags and bindings. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-bash/tree-sitter.json` | lines 1-46 | Bash manifest with filename patterns, injection regex, first-line regex, and bindings. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-rust/grammar.js` | lines 1-120 | Rust grammar strategy: precedence, token lists, externals, supertypes, inline rules, conflicts, word rule. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-javascript/grammar.js` | lines 1-120 | JavaScript grammar strategy: externals, extras, reserved words, supertypes, inline rules, precedence groups. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-python/grammar.js` | lines 1-120 | Python grammar strategy: precedence, indentation/newline externals, comments, conflicts, supertypes, reserved words. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-rust/src/node-types.json` | lines 1-120 | Node-type schema showing declaration subtype contract. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-rust/queries/tags.scm` | lines 1-60 | Rust tags query extracting definitions, calls, and implementation references. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-rust/queries/injections.scm` | lines 1-9 | Rust macro/token-tree injection query. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-javascript/queries/locals.scm` | lines 1-23 | JavaScript locals query for scopes, local definitions, local references. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-javascript/queries/injections.scm` | lines 1-31 | JavaScript injections for tagged templates, regex, JSDoc, hbs templates. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-python/queries/tags.scm` | lines 1-14 | Python tags query for constants, classes, functions, calls. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-typescript/queries/tags.scm` | lines 1-23 | TypeScript tags for functions, methods, classes, modules, interfaces, type references, class references. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-typescript/queries/locals.scm` | lines 1-2 | TypeScript local parameter definitions. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-rust/test/corpus/declarations.txt` | lines 1-160 | Corpus fixture format and Rust declaration expected trees. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-typescript/test/corpus/functions.txt` | lines 1-160 | TypeScript corpus fixtures for typed functions, generics, call signatures, type arguments. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-python/test/corpus/statements.txt` | lines 1-160 | Python corpus fixtures for imports, future imports, print statements. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-rust/package.json` | lines 1-64 | Package distribution includes grammar, manifest, bindings, queries, source, Wasm artifacts, scripts. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-typescript/package.json` | lines 1-59 | TypeScript package distribution includes shared common files, TS/TSX grammar dirs, query files, generated sources, JavaScript dependency. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-rust/bindings/rust/lib.rs` | lines 1-60 | Rust binding exposes `LANGUAGE`, `NODE_TYPES`, `HIGHLIGHTS_QUERY`, `INJECTIONS_QUERY`, and `TAGS_QUERY`. |

## Concept 28: Mirror Binding APIs For A Stable Graph Embedding Layer

The official binding repos show a different kind of pattern from the grammar
repos.

Grammar repos answer:

```text
How is language support packaged?
```

Binding repos answer:

```text
How should host programs embed parser behavior?
```

For Parceltongue, this is crucial because the tool should not only be a CLI. It
will likely need several embedding surfaces:

```text
Rust library
CLI
local HTTP server
MCP server
Codex shell workflow
possibly Tauri UI
test harness
future background index daemon
```

The Tree-sitter bindings show the stable conceptual API that survives across
host languages:

```text
Language
Parser
Tree
Node
TreeCursor
Query
QueryCursor
Point
Range
InputEdit
Logger
ProgressCallback
LookaheadIterator
```

Parceltongue should mirror this with a graph-layer API:

```text
LanguagePack
GraphSession
CodeGraph
Entity
GraphCursor
GraphQuery
GraphQueryCursor
SourcePoint
SourceRange
GraphEdit
GraphLogger
GraphProgressCallback
FactTrace
```

The idea is not to copy every method. The idea is to copy the shape: a small set
of durable objects with explicit lifetimes, ranges, edits, traversal, query
execution, and diagnostics.

### Repos Browsed For This Concept

| Repo | Local Path | Evidence Type | Notes |
|---|---|---|---|
| `tree-sitter/node-tree-sitter` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__node-tree-sitter` | codebase-memory plus direct source | Indexed with 515 nodes and 1374 edges. Used for TypeScript declaration API: parser, tree, node, cursor, query, options. |
| `tree-sitter/py-tree-sitter` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__py-tree-sitter` | codebase-memory plus direct source | Indexed with 546 nodes and 1346 edges. Used for Python type stubs: language, node, tree, parser, query, cursor, ranges. |
| `tree-sitter/go-tree-sitter` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__go-tree-sitter` | codebase-memory plus direct source | Indexed with 1715 nodes and 8152 edges. Used for Go parser/tree/node/query APIs and explicit close/lifetime behavior. |
| `tree-sitter/swift-tree-sitter` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__swift-tree-sitter` | codebase-memory plus direct source | Indexed with 779 nodes and 2413 edges. Used for Swift parser/query APIs and higher-level language layer. |
| `tree-sitter/java-tree-sitter` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__java-tree-sitter` | direct source | Shallow clone contained README/build/docs metadata only; used for JDK/jextract/FFM packaging constraints. |
| `tree-sitter/kotlin-tree-sitter` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__kotlin-tree-sitter` | direct source | Used for module structure, Gradle plugin, supported platforms, and basic usage. |
| `tree-sitter/csharp-tree-sitter` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__csharp-tree-sitter` | direct source | Used for P/Invoke wrapper shape, parser/range/edit structs, test traversal, and platform constraints. |

### The Core Binding Pattern

Across Node, Python, Go, Swift, and C#, the API keeps returning to the same
objects:

```text
Language:
  parser compatibility
  node kind lookup
  field lookup
  version or ABI checks

Parser:
  stateful parser
  set language
  parse input
  parse with old tree
  reset
  included ranges
  logger
  timeout or progress callback

Tree:
  root node
  edit
  changed ranges
  included ranges
  clone/copy
  debug graph

Node:
  type/kind
  grammar id
  named/anonymous
  extra/missing/error
  byte range
  point range
  parent/children/siblings
  field access
  descendant search

TreeCursor:
  efficient tree walking
  parent/child/sibling movement
  current node
  current field
  depth

Query:
  compile query source
  pattern count
  capture count
  disable captures/patterns
  capture names
  predicates/settings/assertions

QueryCursor:
  matches
  captures
  range limits
  match limit
  max start depth
  progress/cancellation
```

This is the API vocabulary Parceltongue should respect.

Parceltongue's graph layer should expose similarly boring concepts:

```text
GraphSession:
  stateful index/query session

CodeGraph:
  graph snapshot
  changed ranges
  graph validity
  provenance

Entity:
  kind
  language
  source range
  confidence
  public/private/interface/local flags

GraphCursor:
  caller/callee traversal
  import traversal
  public-interface traversal
  change-impact traversal

GraphQuery:
  compiled graph/query-pack operation

GraphQueryCursor:
  streamed matches
  token budget
  source range limits
  graph depth limits
  progress/cancellation
```

### Parser APIs Teach Graph Session APIs

Node's binding has a parser API that accepts:

```text
string or custom input function
optional old tree
options with buffer size
included ranges
progress callback
```

Python's binding has a parser constructor with:

```text
language
included ranges
logger
```

and parse overloads for:

```text
byte source
read callback
old tree
encoding
progress callback
```

Go exposes:

```text
NewParser
SetLanguage
Language
SetLogger
PrintDotGraphs
Parse
ParseWithOptions
UTF16 parsing variants
progress callback
```

Swift exposes:

```text
setLanguage
reset
includedRanges
timeout
parse string
parse with readBlock
readFunction chunking
```

For Parceltongue, the graph session should offer the same kind of staged
control:

```text
open repo
load language packs
load or build graph
apply edits
query old graph and new graph
select context with budget
stream progress
cancel long traversal
emit debug trace
```

Possible API:

```rust
let session = GraphSession::open(repo_path)?;
session.load_language_pack("rust")?;
let graph = session.index(IndexOptions {
    changed_files,
    token_budget_hint,
    progress_callback,
    include_ranges,
})?;

let context = graph.select_context(ContextQuery {
    symbol: "MyService::handle_request",
    budget_tokens: 4000,
    include_tests: true,
})?;
```

The key pattern is that parser APIs do not hide state. They make stateful
objects explicit. Parceltongue should do the same.

### Edit APIs Teach Incremental Graph Updates

All serious bindings expose edit concepts.

Node's type declarations define an edit with:

```text
start index
old end index
new end index
start position
old end position
new end position
```

Python exposes `Tree.edit`, `Node.edit`, `Range.edit`, and `Point.edit`.

Go exposes `Tree.Edit`, with a comment that the syntax tree must be kept in sync
with source edits and that byte offsets and row/column coordinates are both
required.

Swift has `InputEdit` integrated into its `LanguageLayer`, where edits apply to
state, included ranges are updated, sublayers receive the edit, parsing computes
invalidations, and sublayer resolution can be eager or deferred.

Parceltongue needs a graph edit model:

```text
GraphEdit:
  file
  old_source_hash
  new_source_hash
  start_byte
  old_end_byte
  new_end_byte
  start_point
  old_end_point
  new_end_point
  changed_ranges
  affected_entities
  affected_edges
```

Then the agent can do:

```text
I edited this function.
Which graph facts are invalid now?
Which tests and callers should I inspect?
Which cached context chunks are stale?
```

This is much better than re-indexing blindly or trusting stale graph facts.

### Node APIs Teach Entity APIs

Tree-sitter nodes carry many properties that are directly analogous to graph
entities.

Bindings expose:

```text
node id
kind id
grammar id
type/name
named flag
extra flag
has changes
has error
is error
is missing
parse state
next parse state
start byte
end byte
start point
end point
children
named children
parent
sibling navigation
field lookup
descendant lookup
```

Parceltongue entities should have the graph equivalent:

```text
entity id
entity kind
language
parser node kind
name
qualified name
source range
definition/reference flag
public/private/interface/local flags
has parse error
has changed
confidence
parent entity
children
incoming edges
outgoing edges
source captures
```

The big lesson is that API users need both convenience and provenance.

Convenience:

```text
entity.name
entity.kind
entity.range
entity.callers()
entity.callees()
```

Provenance:

```text
entity.parser_node_kind
entity.query_pack
entity.capture_names
entity.pattern_index
entity.confidence
```

Codex needs both. Convenience is fast; provenance prevents wrong edits.

### Cursor APIs Teach Token-Efficient Traversal

TreeCursor exists because walking trees by repeatedly allocating child arrays is
not always the right tool. The bindings expose cursor movement:

```text
goto parent
goto first child
goto last child
goto next sibling
goto previous sibling
goto child for byte or point
goto descendant
current node
current field name
current depth
current descendant index
```

Parceltongue needs graph cursors for the same reason.

A naive graph API returns giant arrays:

```text
all callers
all callees
all imports
all references
```

A graph cursor lets the agent walk a frontier:

```text
start at symbol
visit direct callers
skip generated/test/vendor nodes
expand only public-interface callers
stop at token budget
remember omitted frontier
resume later
```

Possible API:

```text
GraphCursor:
  current_entity
  current_edge
  depth
  token_cost_so_far
  goto_callers()
  goto_callees()
  goto_tests()
  goto_importers()
  goto_next_relevant()
  skip_subtree(reason)
  omitted_frontier()
```

This is exactly aligned with the user's PMF:

```text
Help Agent navigate a large codebase faster and reliably with clarity of dependency.
```

### Query APIs Teach Fact Extraction APIs

Bindings expose query as a compiled object and query cursor as a stateful
execution object.

Python exposes:

```text
pattern_count
capture_count
string_count
start/end byte for pattern
rooted/non-local/guaranteed checks
capture names
capture quantifiers
string values
disable capture
disable pattern
pattern settings
pattern assertions
```

Python QueryCursor exposes:

```text
match limit
did exceed match limit
max start depth
byte range
containing byte range
point range
containing point range
captures
matches
predicate callback
progress callback
```

Node exposes query options:

```text
start/end position
start/end byte index
match limit
max start depth
timeout or progress callback
captures
matches
disable capture
disable pattern
```

Go exposes typed query errors:

```text
syntax
node type
field
capture
predicate
structure
language
```

Swift's query layer parses predicates and exposes predicate metadata. It also
adds `ResolvingQueryCursor`, which can evaluate predicates using text providers
and prefetch matches for background-safe work.

Parceltongue should copy this structure for graph facts:

```text
GraphQuery:
  compiled query pack
  fact kinds
  capture schema
  required node types
  disabled fact kinds
  pattern metadata

GraphQueryCursor:
  run against code graph or parse tree
  source range limits
  graph depth limits
  token budget
  match limit
  progress callback
  predicate evaluator
  fact stream
```

Important: a graph query should be resumable and inspectable.

Example:

```json
{
  "query": "callers",
  "symbol": "save_user",
  "cursor": {
    "depth": 1,
    "expanded": 12,
    "omitted": 4,
    "tokensUsed": 1830,
    "didExceedBudget": false
  },
  "facts": [
    {
      "kind": "call_edge",
      "from": "UserController::create",
      "to": "save_user",
      "trace": "fact:abc123"
    }
  ]
}
```

This is how Parceltongue can answer LLM queries with minimal tokens while
keeping follow-up paths open.

### Query Errors Teach Tool Errors

Go and Swift both model query errors as typed categories.

The relevant categories are:

```text
syntax
node type
field
capture
predicate
structure
language
```

This is useful for Parceltongue query-pack testing.

If a graph query pack fails, the error should not be:

```text
bad query
```

It should be:

```text
invalid node type in rust.calls at byte offset 231
unknown field name in tsx.imports at row 4 column 12
predicate unsupported in python.tags
query incompatible with language ABI version
```

Then Codex can repair the right layer.

### Language APIs Teach Capability Introspection

Python's `Language` type exposes:

```text
name
ABI version
semantic version
node kind count
parse state count
field count
supertypes
subtypes
node kind lookup
field lookup
next state
lookahead iterator
copy
```

That is capability introspection.

Parceltongue should expose the graph equivalent:

```text
language pack name
language pack version
parser ABI
node type count
field count
query pack versions
supported fact kinds
supported graph edges
support levels
known omissions
available context selectors
```

Example:

```json
{
  "language": "rust",
  "parserAbi": 15,
  "languagePack": "rust-code-navigation-v3",
  "supportedFacts": [
    "definition.function",
    "definition.trait",
    "definition.impl",
    "reference.call",
    "reference.implementation"
  ],
  "unsupportedFacts": [
    "macro_expanded_call",
    "compiler_resolved_trait_dispatch"
  ]
}
```

This is more useful to Codex than a simple "Rust supported" flag.

### Swift LanguageLayer Is The Closest Parceltongue Analogue

The Swift binding includes a higher-level `LanguageLayer`.

It models:

```text
language provider
maximum language depth
content read handler
text provider
content snapshots
language configuration
parser
parse state
sublayers
missing injections
range restriction
included ranges
edit application
changed sets
sub-layer resolution
snapshots
```

This is very close to Parceltongue's future shape.

Parceltongue can adapt the concept:

```text
CodeLayer:
  language pack
  parser
  query packs
  parse state
  graph state
  embedded code layers
  missing language packs
  included ranges
  changed ranges
  invalidated facts
  snapshots
```

For mixed-language files, this is especially important.

Example:

```text
TSX file
  TypeScript layer
    JSX sublayer
      embedded CSS string
      embedded GraphQL string
```

The agent should be able to ask:

```text
what language layers exist in this file?
which layer owns this range?
which sublayers are missing because language packs are not installed?
what graph facts were invalidated by my edit?
```

Swift's LanguageLayer shows that this can be an explicit product object, not a
hidden implementation trick.

### Host Bindings Also Carry Deployment Reality

The Java, Kotlin, and C# repos are useful even when the shallow clones expose
less API source.

Java's README points to:

```text
JDK 23+
jextract
tree-sitter and tree-sitter-java libraries
Maven Central package
API docs
alternatives for older JDKs or Android
```

Kotlin's README shows module separation:

```text
ktreesitter library
ktreesitter-plugin for generating language source files
languages module for bundled languages
JVM, Android, Native support
no JS/WasmJS support
```

C# shows:

```text
P/Invoke wrapper
Windows/.NET build constraints
submodule dependency setup
explicit DLL imports
manual parser/tree/cursor traversal
```

For Parceltongue, the lesson is:

```text
API design and deployment design are inseparable.
```

If Parceltongue wants to work smoothly inside Codex app, the best first
embedding target is likely:

```text
local CLI plus JSON
optional MCP server
Rust library underneath
no complex cross-platform native package story until the core workflows are stable
```

Trying to support every host language too early would be a distraction. But the
API model should be host-neutral enough that these surfaces can exist later.

### C# Shows The Lowest-Level Binding Shape

The C# binding is useful because it exposes how low-level the bridge can get.

It defines raw interop structures:

```text
TSPoint
TSRange
TSInputEdit
TSQueryCapture
TSQueryMatch
TSQueryPredicateStep
```

It wraps:

```text
TSParser
set_language
included_ranges
parse_string
reset
timeout
logger
P/Invoke imports for parser and language functions
```

The test walks a C++ tree with a cursor, reads node start/end offsets, start
line, current field, current symbol, descends to children, moves to siblings,
and prints spans from the original source text.

Parceltongue should keep this low-level path available internally:

```text
source range plus original text is the source of truth
graph facts should be traceable back to spans
cursors should support low-allocation traversal
bindings should not hide parser failure or language mismatch
```

### The API Parceltongue Should Expose To Codex

For Codex, the surface should be smaller than the internal Rust API.

Suggested agent-facing commands:

```text
ptctx languages --json
ptctx index --json-summary
ptctx changed-ranges --since HEAD
ptctx entity Symbol --json
ptctx callers Symbol --budget 3000 --json
ptctx callees Symbol --budget 3000 --json
ptctx context Symbol --budget 5000 --why --json
ptctx trace fact-id --json
ptctx query-pack check rust.calls --json
ptctx layer file.tsx --json
```

Suggested JSON object families:

```text
GraphSessionSummary
GraphSnapshotSummary
EntitySummary
GraphTraversalSummary
ContextSelectionSummary
FactTraceSummary
LanguageLayerSummary
QueryPackDiagnostic
ChangedRangeSummary
```

These should be stable in the same way binding APIs are stable. Codex prompts
can then depend on them.

### Bi-Directional API Shape

The parser bindings support both:

```text
tree to node traversal
node to source range lookup
query to captures
capture to node
tree edit to changed ranges
```

Parceltongue should support:

```text
source range to entity
entity to source range
entity to graph neighbors
graph edge to source captures
query pack to emitted facts
edit to invalidated facts
context selection to omitted frontier
```

This is the bi-directional workflow the user keeps circling:

```text
LLM asks query to tool
tool responds with relevant context
LLM decides what to explore next
tool expands the dependency neighborhood with minimum tokens
LLM can trace every answer back to source evidence
```

### Shreyas Doshi Product Reading

From a Shreyas-style product lens, binding APIs teach restraint.

They do not try to be a complete code intelligence product. They expose durable
objects and operations that expert users compose.

That is the right model for this user:

```text
solo agent power user
Codex app
all languages
CRUD apps plus Rust/C/C++ systems programming
self-use, not SaaS
large codebase navigation
dependency clarity
minimum token waste
```

The product should not over-design a polished UI first.

It should first make a tight agent API:

```text
stable JSON
source spans
explicit confidence
incremental edits
graph cursors
query-pack diagnostics
language-layer summaries
context budgets
traceability
```

The binding repos suggest that long-lived value comes from stable primitives,
not from one clever context selection algorithm.

### What Parceltongue Should Steal Directly

| Binding Pattern | Parceltongue Adaptation |
|---|---|
| Parser as stateful object | `GraphSession` as stateful index/query object. |
| Parse with old tree | Query/index with old graph snapshot. |
| Edit before reparse | Apply `GraphEdit` before incremental graph refresh. |
| Changed ranges | Changed source ranges plus invalidated graph facts. |
| Node byte and point ranges | Entity and fact source ranges with byte and point spans. |
| TreeCursor | `GraphCursor` for low-token traversal frontiers. |
| Query and QueryCursor split | `GraphQuery` and `GraphQueryCursor` split. |
| Query range limits | Source-range and graph-depth limits for context retrieval. |
| Match limits and progress callbacks | Token budget, fact limit, timeout, cancellation, progress. |
| Typed query errors | Typed query-pack diagnostics. |
| Language introspection | Language-pack support matrix and capability report. |
| Swift LanguageLayer | `CodeLayer` for nested languages, subgraphs, and invalidations. |
| C# P/Invoke structs | Keep low-level source-span/interop primitives explicit internally. |

### Concept 28 Conclusion

The binding repos show that Tree-sitter's power comes from a small stable object
model.

Parceltongue should copy that discipline.

It should not expose only:

```text
give me context
```

It should expose:

```text
open graph session
load language packs
index graph
apply edit
inspect changed ranges
query entities
walk graph cursor
run graph query cursor
select context under budget
trace facts to source captures
inspect language layers
report capability and confidence
```

This is how Parceltongue becomes a reliable code-assistance tool for Codex
agents rather than another fuzzy search layer.

### Repos And Docs Touched For This Concept

| Source | Evidence | Notes |
|---|---|---|
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__node-tree-sitter` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/tree-sitter__node-tree-sitter-20260706-224743`; list_projects reported 515 nodes and 1374 edges. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__py-tree-sitter` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/tree-sitter__py-tree-sitter-20260706-224743`; list_projects reported 546 nodes and 1346 edges. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__go-tree-sitter` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/tree-sitter__go-tree-sitter-20260706-224743`; list_projects reported 1715 nodes and 8152 edges. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__swift-tree-sitter` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/tree-sitter__swift-tree-sitter-20260706-224743`; list_projects reported 779 nodes and 2413 edges. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__node-tree-sitter/tree-sitter.d.ts` | lines 1-240, 240-560, 560-920 | Node API: parser, input, options, ranges, edits, syntax nodes, cursor, tree, query options, query execution. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__py-tree-sitter/tree_sitter/__init__.pyi` | lines 1-260, 260-410 | Python API: language introspection, node properties, tree edit/changed ranges, parser overloads, query, query cursor, ranges and points. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__go-tree-sitter/parser.go` | lines 1-260 | Go parser API: parser lifecycle, language compatibility check, logger, DOT graphs, parse with old tree, parse options, progress callback. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__go-tree-sitter/tree.go` | lines 1-121 | Go tree API: root node, root with offset, language, edit, changed ranges, included ranges, DOT graph, close, clone. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__go-tree-sitter/node.go` | lines 1-260 | Go node API: ids, kind, grammar name, named/extra/error/missing, ranges, child/field lookup, efficient cursor-backed children. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__go-tree-sitter/query.go` | lines 1-260, rg lines 702-1070 | Go query API: query/cursor types, query error categories, predicates, matches/captures, byte/point ranges, match limits, progress callback. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__swift-tree-sitter/Sources/SwiftTreeSitter/Parser.swift` | lines 1-161 | Swift parser API: failable language setting, reset, included ranges, timeout, string parsing, read-block parsing. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__swift-tree-sitter/Sources/SwiftTreeSitter/Query.swift` | lines 1-260 | Swift query API: typed query errors, predicates, query execution, capture metadata, depth, pattern index. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__swift-tree-sitter/Sources/SwiftTreeSitter/ResolvingQueryCursor.swift` | lines 1-130 | Predicate-resolving cursor, text provider context, prefetching, background-safe query execution. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__swift-tree-sitter/Sources/SwiftTreeSitterLayer/LanguageLayer.swift` | lines 1-260 | Higher-level language layer: nested languages, content snapshots, included ranges, edits, invalidation sets, sublayers, missing injections. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__java-tree-sitter/README.md` | lines 1-35 | Java binding packaging constraints: JDK 23, jextract, installed libraries, Maven Central docs, alternatives. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__kotlin-tree-sitter/README.md` | lines 1-30 | Kotlin repo modules: library, Gradle plugin, bundled languages. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__kotlin-tree-sitter/ktreesitter/README.md` | lines 1-39 | Kotlin supported platforms and basic parser usage. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__csharp-tree-sitter/README.md` | lines 1-43 | C# P/Invoke binding introduction, submodule cloning, Windows/.NET build constraints, traversal demo. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__csharp-tree-sitter/src/binding.cs` | lines 1-260 | C# low-level interop structs and parser wrapper with language, included ranges, parse string, timeout, logger, P/Invoke imports. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__csharp-tree-sitter/tests/test.cs` | lines 1-176 | C# traversal demo: parser setup, tree cursor traversal, source spans, node symbols, file iteration. |

## Concept 29: Separate Parse Queries From Graph Fact Pipelines

The repos in this concept answer the question that sits directly underneath
Parceltongue:

```text
How do we turn Tree-sitter parse trees into useful graph facts for an agent?
```

The answer is not:

```text
run a query and dump captures
```

The answer is a pipeline:

```text
source files
  -> language detection
  -> parser selection
  -> parse tree
  -> query captures
  -> extraction rules
  -> typed facts
  -> graph storage
  -> bounded traversal
  -> compressed agent response
  -> follow-up cursor or trace
```

That separation matters.

If Parceltongue treats Tree-sitter queries as the whole product, it will become
a brittle syntax grep tool. If Parceltongue treats Tree-sitter queries as only
the first evidence layer in a graph fact pipeline, it can become the thing the
user actually wants:

```text
LLM asks a small structural question.
Tool answers with the smallest useful context.
LLM sees what to inspect next.
Tool expands the dependency neighborhood without flooding the prompt.
Every claim traces back to source spans, query packs, and confidence.
```

That is the real product category.

### Repos Browsed For This Concept

| Repo | Local Path | Evidence Type | Notes |
|---|---|---|---|
| `tree-sitter/tree-sitter-graph` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-graph` | codebase-memory plus direct source | Indexed with 1156 nodes and 4042 edges. Used for the official DSL pattern: Tree-sitter captures become arbitrary graph nodes, edges, and attributes. |
| `tree-sitter/tree-sitter-tsq` | `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-tsq` | codebase-memory plus direct source | Indexed with 200 nodes and 228 edges. Used for the distinction between query syntax parsing and semantic validation against a target grammar. |
| `DeusData/codebase-memory-mcp` | `git-ref-repo/ignore-this-folder-repos/DeusData__codebase-memory-mcp` | direct source | Used for the persistent graph backend pattern and README-level claims about fast indexing, many languages, agent query tools, and shared graph artifacts. |
| `Christoph/treesitter-mcp` | `git-ref-repo/ignore-this-folder-repos/Christoph__treesitter-mcp` | codebase-memory plus direct source | Indexed with 2498 nodes and 12159 edges. Used for token-budgeted MCP workflows: code maps, minimal edit context, call graphs, review bundles, and token regression tests. |
| `sdsrss/code-graph-mcp` | `git-ref-repo/ignore-this-folder-repos/sdsrss__code-graph-mcp` | codebase-memory plus direct source | Indexed with 4934 nodes and 22644 edges. Used for persistent graph traversal, language configs, confidence gates, recursive CTE call graphs, compression, and curated tool surfaces. |
| `tirth8205/code-review-graph` | `git-ref-repo/ignore-this-folder-repos/tirth8205__code-review-graph` | codebase-memory plus direct source | Indexed with 4570 nodes and 18789 edges. Used for review-first graph product design, SQLite schema, minimal context, review context, context savings, risk, flows, and CI review journeys. |

### The Central Product Lesson

There are three layers that should not be blurred:

```text
Layer 1: Parse Query Layer
  Tree-sitter grammar
  Tree-sitter query syntax
  captures
  predicates
  ranges
  parser errors

Layer 2: Graph Fact Layer
  entity facts
  edge facts
  attributes
  source spans
  confidence
  provenance
  language-pack extractor version

Layer 3: Agent Context Layer
  code map
  minimal edit context
  call graph
  impact radius
  review context
  route trace
  test gaps
  token budget
  omitted frontier
```

Most weak tools collapse these layers.

They say:

```text
I found captures, therefore I know the code graph.
```

That is not true.

A Tree-sitter capture is evidence. It is not yet a reliable agent answer.

Parceltongue should say:

```text
I found captures.
I transformed them into typed facts.
I stored them with provenance.
I traversed only the relevant facts.
I returned a budgeted answer plus the next expansion handle.
```

That is the difference between syntax-aware search and agent-grade code
navigation.

### Tree-Sitter-Graph Shows The Official Fact Pipeline Shape

`tree-sitter-graph` is the most important official repo for this concept.

Its README describes a DSL for constructing arbitrary graph structures from
source code parsed with Tree-sitter. That is already Parceltongue-shaped:

```text
parse source
match Tree-sitter query
execute DSL statements
create graph nodes
create graph edges
attach attributes
print or serialize graph
```

The source makes the model clearer.

The graph is not "all syntax nodes." It has two distinct stores:

```text
syntax_nodes
graph_nodes
```

`Graph::add_syntax_node` stores syntax nodes only when the graph needs to
reference them. `Graph::add_graph_node` creates graph nodes separately. This is
an excellent design hint.

Parceltongue should not store every parse node as a graph entity by default.

It should store:

```text
syntax evidence:
  parse node id
  node kind
  field name
  capture name
  source range

graph facts:
  entity
  edge
  attribute
  confidence
  trace back to syntax evidence
```

This distinction is the heart of a token-efficient graph tool.

### Syntax Nodes Are Evidence, Graph Nodes Are Product Objects

`tree-sitter-graph` lets a graph attribute point back to a syntax node.

That is the exact pattern Parceltongue needs:

```text
Graph node:
  kind = function
  name = parse_user
  qualified_name = src/parser.rs::parse_user

Attribute:
  source = syntax_node_id
  source_range = bytes 120..450
  source_capture = @function.definition
```

The agent should be able to ask:

```text
why do you think parse_user is a function?
```

and Parceltongue should answer:

```text
Because rust.definitions query pattern 3 captured a function_item node
at src/parser.rs:11-27 with capture @definition.function.
```

This is why provenance is not a nice-to-have. It is the difference between a
tool Codex can trust and a tool Codex must double-check by reading files anyway.

### Tree-Sitter-Graph Execution Teaches Strict And Lazy Modes

`tree-sitter-graph` has execution paths that run a parsed graph DSL file against
a syntax tree and source text. It can execute into an existing graph. It also
has strict and lazy behavior around matched captures and variables.

This maps directly to Parceltongue:

```text
strict extraction:
  fail when a required capture is missing
  fail when an expected global is absent
  fail when a query-pack invariant is broken

lazy extraction:
  emit partial facts
  mark confidence lower
  retain diagnostics
  continue indexing the rest of the repo
```

For a solo power user, both modes are useful.

During language-pack development:

```text
strict mode
```

During everyday Codex navigation:

```text
lazy mode with confidence and warnings
```

A good Parceltongue response should say:

```json
{
  "status": "partial",
  "factsReturned": 42,
  "diagnostics": [
    {
      "severity": "warning",
      "language": "typescript",
      "queryPack": "tsx.calls",
      "message": "5 call captures were ambiguous and emitted as low confidence"
    }
  ]
}
```

That is much better than silently pretending every edge is equally true.

### Stanzas Are A Useful Mental Model For Query Packs

`tree-sitter-graph` models graph extraction around stanzas:

```text
query
statements
full-match capture indexes
range
```

Statements include:

```text
create graph node
create edge
set node attribute
set edge attribute
variable declaration
assignment
scan
print
if
for-in
```

Parceltongue can adapt this into a query-pack model:

```text
FactRule:
  tree_sitter_query
  required_captures
  emitted_fact_kind
  emitted_fact_schema
  confidence_policy
  language_pack_version
  fixture_expectations
```

Example:

```text
rust.functions.rule:
  query:
    (function_item name: (identifier) @name) @definition.function
  emits:
    Entity(kind="function", name=@name, source=@definition.function)
  confidence:
    high when node has name field and no parse error
```

This is not the same as just storing the `.scm` file. The rule around the query
is where the graph product begins.

### Tree-Sitter-TSQ Teaches Syntax Validity Is Not Semantic Validity

`tree-sitter-tsq` is a grammar for the Tree-sitter query language. Its README
says the repo is no longer maintained and points to `tree-sitter-query`, but the
repo is still useful as a design artifact.

The grammar captures the query language itself:

```text
patterns
predicates
captures
alternation
anonymous leaves
groups
named nodes
wildcard nodes
field names
negated children
quantifiers
predicate names
```

The crucial lesson comes from its test corpus. Some query examples are
syntactically valid Tree-sitter queries but would produce parse or validation
errors in the Tree-sitter library when checked against a target language grammar
because the node types do not exist in that language.

That implies three separate checks for Parceltongue:

```text
query syntax check:
  is this query well-formed?

language schema check:
  do node types and fields exist for this grammar?

fact fixture check:
  does this query emit the graph facts we expect on sample code?
```

Do not collapse them.

If a Rust call query fails, Codex needs to know which layer failed:

```text
syntax error in query
unknown node type for Rust grammar
capture schema mismatch
fixture expectation mismatch
extractor logic error
```

Each failure sends the agent to a different fix.

### Codebase-Memory-MCP Shows The Persistent Backend Pattern

`DeusData/codebase-memory-mcp` is a graph backend aimed directly at AI coding
agents. Its README makes several performance and quality claims, including fast
indexing, Tree-sitter AST coverage across many languages, hybrid LSP support,
a persistent knowledge graph, call chains, routes, cross-service links, and MCP
tools.

Those are README claims, not independently verified here.

The useful pattern is not the exact numbers. The useful pattern is the product
shape:

```text
persistent graph
many language parsers
hybrid LSP enrichment
structural queries
cross-repo edges
edge types
shared graph artifact
no built-in LLM
agent acts as query translator
```

That last point matters.

The graph tool should not try to be the agent.

The graph tool should be:

```text
fast structural memory
precise query API
bounded context selector
traceable answer generator
```

Codex remains the reasoning layer.

This is exactly right for the user:

```text
not a product for others
self-use
Codex app
all languages
large codebase navigation
clarity of dependency
```

Parceltongue does not need to become a SaaS assistant. It needs to become a
local structural memory backend that Codex can ask better questions of.

### Code-Graph-MCP Shows Concrete Graph Traversal Guardrails

`sdsrss/code-graph-mcp` gives implementation-level lessons.

Its parser layer defines a `ParsedNode` with:

```text
node_type
name
qualified_name
line range
code content
signature
doc comment
return type
parameter types
is_test
```

That is a good minimum for Parceltongue entity facts.

More importantly, the repo centralizes language-specific extraction in a
language config module. The source explicitly warns that call-expression node
kinds cannot be handled by one universal string because some languages require
dedicated match arms.

This is a very important correction to over-optimistic architecture.

Bad Parceltongue assumption:

```text
one generic Tree-sitter query can extract useful call graphs for all languages
```

Better Parceltongue assumption:

```text
common graph schema
language-specific extraction rules
shared confidence model
shared fixture contract
language-specific gaps admitted openly
```

The repo also has tests to ensure supported languages resolve grammars and
round-trip config names. Parceltongue should have the same guardrail. A language
should not quietly appear as "supported" because a grammar loads.

Support should mean:

```text
grammar loads
node-types schema known
query packs compile
fixtures pass
required fact kinds emitted
known omissions documented
```

### Recursive Graph Traversal Needs Caps And Disclosure

`sdsrss/code-graph-mcp` uses recursive SQL traversal for call graphs. Its query
logic has:

```text
maximum depth
row limit
direction
cycle detection
parent id
confidence filter
limit_hit flag
depth_capped flag
requested versus effective depth
suppressed ambiguous count
```

That is exactly how Parceltongue should report graph traversal.

Never return:

```json
{
  "callers": [...]
}
```

Return:

```json
{
  "query": "callers",
  "target": "save_user",
  "requestedDepth": 4,
  "effectiveDepth": 3,
  "nodesReturned": 120,
  "edgesReturned": 180,
  "limitHit": true,
  "depthCapped": true,
  "suppressedAmbiguousEdges": 17,
  "omittedFrontier": [
    {
      "entity": "UserController::create",
      "reason": "token_budget"
    }
  ]
}
```

Large-codebase tools live or die by this honesty.

If Codex sees a capped traversal, it can ask the next query intelligently. If it
does not see the cap, it will reason from incomplete evidence as if it is
complete.

### Compression Must Preserve Expansion Handles

`sdsrss/code-graph-mcp` has a compression layer with multiple output levels. It
groups results by node, file, or directory based on token thresholds. Its tests
ensure node IDs are retained in summaries.

This is an understated but vital idea.

Compressed output must retain handles.

Bad compressed answer:

```text
Many files under auth are impacted.
```

Good compressed answer:

```json
{
  "group": "src/auth",
  "summary": "18 impacted nodes across 6 files",
  "topEntities": [
    {"id": "n123", "name": "login"},
    {"id": "n456", "name": "refresh_session"}
  ],
  "expand": {
    "tool": "graph_expand",
    "cursor": "cursor_abc",
    "group": "src/auth"
  }
}
```

This is how an LLM can use minimum tokens without losing navigation power.

Parceltongue's context selector should always return expansion handles:

```text
entity ids
edge ids
fact trace ids
cursor ids
omitted frontier ids
group ids
source ranges
```

No handle means no reliable follow-up.

### Curated Tool Surface Beats Tool Sprawl

`sdsrss/code-graph-mcp` keeps the visible MCP tool surface small. The source
comments say niche tools were folded into flags, management tools are hidden
from `tools/list` to save tokens, and descriptions are kept short with tests
around tool descriptions.

This is directly relevant to Parceltongue inside Codex.

The tool surface should not be:

```text
50 graph tools
```

It should be closer to:

```text
project_map
entity
relations
context
impact
search
query_pack_check
trace
```

with flags:

```text
direction=callers|callees|both
detail=minimal|standard|full
budget_tokens=...
depth=...
include_tests=true|false
include_source=true|false
confidence=...
```

Tool descriptions themselves consume context. The tool API must be designed as
prompt real estate.

### Treesitter-MCP Shows The Context Compressor Pattern

`Christoph/treesitter-mcp` is useful because it is very explicit about agent
workflows.

The README recommends workflows like:

```text
code_map(path="src", detail="minimal", with_types=true)
view_code(... detail="signatures")
minimal_edit_context(...)
review_context(...)
```

The tool definitions are also product copy for agents. They say when to use a
tool, when not to use it, what token cost to expect, and what workflow should
come next.

That is not fluff. For agent tools, the description is part of the product.

Examples from its tool surface:

```text
code_map:
  use first when exploring unfamiliar code
  do not use when you already know the file
  start with minimal detail for large projects

find_usages:
  use before refactoring shared code
  avoid huge symbols or set max_context_lines

minimal_edit_context:
  use when editing one known function
  avoid full-file reads

call_graph:
  best-effort project-local graph
  not compiler-grade name resolution
```

Parceltongue should copy this discipline.

Every Codex-facing command should have:

```text
use when
do not use when
token cost
expected next tool
confidence caveat
```

### Minimal Edit Context Is The Most Parceltongue-Shaped Workflow

`treesitter-mcp`'s `minimal_edit_context` source is almost exactly the workflow
Parceltongue should support.

It:

```text
reads one file
detects language
parses code
extracts enhanced file shape
finds the target symbol
collects called names within the target range
collects same-file dependency signatures
resolves project-local dependency signatures from imports
selects relevant imports
selects relevant types
returns compact row-oriented JSON
enforces a token budget with tiktoken
marks truncation
```

That is the product journey:

```text
I know the symbol I want to edit.
Give me only the target code plus the dependency signatures that matter.
```

For Parceltongue, this becomes:

```text
ptctx context parse_user --budget 4000 --mode edit
```

and the response should include:

```text
target entity
target source range
target code
direct callees
direct caller summary
types referenced
imports required
tests touching it
omitted frontier
fact traces
truncation marker
```

This is probably one of the highest-PMF workflows for the user's Codex setup.

### Call Graph Does Not Need To Pretend To Be Compiler-Perfect

`treesitter-mcp`'s call graph tool describes itself as best-effort, compact, and
project-local. It explicitly says that if you need compiler-grade name
resolution across imports, generics, and traits, use LSP references or
definitions when available.

That honesty is the right posture.

Parceltongue should not claim:

```text
perfect call graph for Rust trait dispatch, macros, TypeScript dynamic imports,
C++ overloads, Python monkey patching, and framework magic
```

It should claim:

```text
source-proven structural graph
confidence-labeled edges
best-effort static extraction
optional LSP/compiler enrichment
clear omissions
agent-friendly traversal
```

For Rust/C/C++ systems work, this matters. A low-confidence edge is still
useful if it is labeled. An unlabeled wrong edge is dangerous.

### Token Budget Must Be Executable, Not Aspirational

`treesitter-mcp` has tests that assert outputs respect token budgets, including
tests that use `tiktoken-rs` against actual serialized output. It also has
aggregate token-efficiency tests for overview, focused edit, call graph, repo
search, and directory map workflows.

This is exactly the right testing philosophy for Parceltongue.

Do not only say:

```text
token efficient
```

Test:

```text
WHEN selecting context for a known symbol under 4000 tokens
THEN serialized JSON SHALL be <= 4000 model tokens
AND SHALL include the target entity
AND SHALL include direct dependency handles
AND SHALL mark truncation if anything was omitted
```

The token budget is part of correctness.

If a code-assistance graph tool cannot guarantee its output budget, it is not a
good graph tool for agents.

### Code-Review-Graph Shows A Review-First Product Journey

`tirth8205/code-review-graph` is not just a parser. It is a product around one
high-value journey:

```text
review changed code with less context waste
```

Its README describes:

```text
Tree-sitter structural map
incremental updates
MCP context for AI assistants
blast radius analysis
minimal set of files to read
CI review comments
risk-scored functions
affected execution flows
test gaps
token savings metadata
```

Its schema is very concrete:

```text
File nodes
Class nodes
Function nodes
Test nodes
Type nodes

CALLS
IMPORTS_FROM
INHERITS
IMPLEMENTS
CONTAINS
TESTED_BY
DEPENDS_ON
REFERENCES
INJECTS
CONSUMES
PRODUCES
TEMPORAL_STUB
```

It stores nodes and edges in SQLite. Edges include:

```text
confidence
confidence_tier
extra
updated_at
```

This reinforces the core architecture:

```text
persistent graph first
agent context second
review product third
```

Parceltongue should start with persistent graph facts, not one-off context
snippets.

### Minimal Context As The First Tool Call

`code-review-graph` has a `get_minimal_context` tool designed to be the entry
point. It returns:

```text
graph stats
risk
top affected entities
test gap count
top communities
top flows
suggested next tools
```

This is a beautiful agent journey.

The LLM should not begin by reading 20 files.

It should begin by asking:

```text
what is the smallest useful orientation for this task?
```

Parceltongue command:

```text
ptctx start --task "debug login timeout" --budget 800 --json
```

Possible response:

```json
{
  "summary": "14210 entities, 38102 edges across 918 files",
  "task": "debug login timeout",
  "likelyEntryPoints": [
    "LoginController::handle",
    "SessionService::refresh"
  ],
  "risk": "unknown",
  "suggestedNext": [
    {
      "tool": "context",
      "args": {"symbol": "LoginController::handle", "mode": "debug"}
    },
    {
      "tool": "relations",
      "args": {"symbol": "SessionService::refresh", "direction": "both"}
    }
  ]
}
```

This directly matches the user's earlier phrasing:

```text
LLM asks query to tool -> tool responds with relevant context which helps LLM decide better
```

The answer should help decide the next question.

### Review Context Is A Composition Of Smaller Tools

`treesitter-mcp`'s review context composes:

```text
diff analysis
parse diff
affected-by-diff
minimal edit context
relevant tests
token budget trimming
```

`code-review-graph`'s review context composes:

```text
changed files
impact radius
changed nodes
impacted nodes
edges
source snippets
review guidance
context savings
```

This says review is not a primitive.

Review is a workflow built from graph primitives.

Parceltongue should implement:

```text
primitive graph tools:
  entity
  relations
  impact
  context
  trace
  search

workflow tools:
  review_context
  debug_context
  refactor_context
  test_context
```

The workflow tools should call the primitives internally.

This preserves a small, composable core while still giving Codex convenient
one-shot workflows.

### The Public Interface Dependency Graph Question

The user asked whether there is a universal way of capturing relationships
between parts of code, like Parceltongue's dependency graph, and whether others
have tried a public interface dependency graph.

From these repos, the honest answer is:

```text
There is a common schema shape, but not a universal semantics.
```

Common schema shape:

```text
File
Module
Class
Function
Method
Type
Import
Call
Reference
Contains
Inherits
Implements
Tests
```

Non-universal semantics:

```text
Rust trait dispatch
C++ overload resolution
TypeScript type-only imports
Python dynamic imports
Java annotations and DI
Go interface satisfaction
React component composition
SQL table lineage
GraphQL operations
HTTP route wiring
shell script process calls
```

So Parceltongue should use a common graph model but language-specific fact
extractors.

For public-interface dependency graphs specifically, the right abstraction is
probably:

```text
InterfaceSurface:
  exported symbols
  public routes
  public commands
  public types
  trait/interface contracts
  package/module exports
  database/API contracts

InterfaceEdge:
  consumes
  implements
  calls
  imports
  routes_to
  serializes
  deserializes
  tests
```

This is more useful than a raw call graph for large codebases because it tells
the agent what matters at architecture boundaries.

### Proposed Parceltongue Fact Schema

Minimal entity fact:

```json
{
  "id": "ent_123",
  "kind": "function",
  "language": "rust",
  "name": "parse_user",
  "qualifiedName": "src/parser.rs::parse_user",
  "file": "src/parser.rs",
  "range": {
    "startByte": 120,
    "endByte": 450,
    "startPoint": {"row": 10, "column": 0},
    "endPoint": {"row": 26, "column": 1}
  },
  "visibility": "private",
  "confidence": 0.98,
  "trace": "trace_abc"
}
```

Minimal edge fact:

```json
{
  "id": "edge_456",
  "kind": "calls",
  "source": "ent_789",
  "target": "ent_123",
  "file": "src/controller.rs",
  "range": {
    "startByte": 900,
    "endByte": 918
  },
  "confidence": 0.82,
  "confidenceTier": "syntax_resolved",
  "trace": "trace_def"
}
```

Fact trace:

```json
{
  "id": "trace_def",
  "languagePack": "rust@0.3.0",
  "queryPack": "rust.calls@0.2.1",
  "patternIndex": 4,
  "captures": [
    {"name": "caller", "nodeKind": "function_item"},
    {"name": "callee", "nodeKind": "identifier"}
  ],
  "extractor": "rust_call_edges_v2",
  "sourceHash": "sha256:..."
}
```

This is the kind of schema Codex can reason with.

### Proposed Parceltongue Pipeline

The end-to-end pipeline should be explicit:

```text
1. Discover files
   respect gitignore
   ignore vendor/build folders
   record file hashes

2. Detect languages
   extension
   shebang
   embedded layers
   fallback modes

3. Parse source
   Tree-sitter parser
   included ranges
   parse errors retained
   old tree if available

4. Run query packs
   language-specific queries
   source range limits
   syntax diagnostics
   schema diagnostics

5. Emit raw captures
   capture name
   node kind
   field name
   byte range
   point range

6. Transform captures to facts
   entity facts
   edge facts
   attribute facts
   confidence
   trace

7. Store graph
   SQLite or embedded graph store
   stable ids
   source hashes
   migrations
   incremental invalidation

8. Traverse graph
   depth caps
   confidence gates
   cycle detection
   row limits
   omitted frontier

9. Select context
   task mode
   token budget
   compression level
   source snippets only when useful
   expansion handles

10. Return agent answer
   compact JSON
   summary
   facts
   caveats
   next suggested tools
   trace ids
```

This gives Parceltongue a real architecture instead of a pile of clever search
commands.

### Codex-Facing Tool Surface

Suggested small surface:

```text
ptctx start
ptctx map
ptctx entity
ptctx relations
ptctx context
ptctx impact
ptctx search
ptctx trace
ptctx query-pack
```

Examples:

```text
ptctx start --task "debug login timeout" --json --budget 1000
ptctx map --path src --detail minimal --with-types --json --budget 2000
ptctx entity parse_user --json
ptctx relations parse_user --direction both --depth 2 --json --budget 3000
ptctx context parse_user --mode edit --json --budget 4000
ptctx impact --changed src/parser.rs --depth 2 --json --budget 5000
ptctx search "session refresh token" --json --budget 2000
ptctx trace trace_def --json
ptctx query-pack check rust.calls --json
```

MCP can wrap these commands later. The CLI should exist first because Codex app
already works beautifully with shell and files.

### Agent Journey: First Contact With A Large Repo

```text
User:
  "Understand how login works."

Codex:
  ptctx start --task "understand login" --budget 800

Parceltongue:
  "Graph has 42k entities, 88k edges. Likely symbols: login, authenticate,
  SessionService::refresh. Suggested next: relations(login), context(authenticate)."

Codex:
  ptctx relations authenticate --direction both --depth 1 --budget 2500

Parceltongue:
  "Direct callers, callees, tests, routes. 9 edges suppressed below confidence 0.5.
  Omitted frontier available as cursor C1."

Codex:
  ptctx context authenticate --mode explain --budget 5000 --cursor C1

Parceltongue:
  "Target code, key types, route entry, direct test, one low-confidence DI edge,
  trace ids."

Codex:
  Explains login flow with cited source ranges and caveats.
```

This is the concrete journey. This is the PMF.

### Agent Journey: Editing One Known Symbol

```text
User:
  "Change parse_user to accept optional tenant id."

Codex:
  ptctx context parse_user --mode edit --budget 4000

Parceltongue:
  target code
  direct callees
  direct callers summary
  relevant imports
  relevant types
  tests
  omitted frontier

Codex:
  edits file

Codex:
  ptctx impact --changed src/parser.rs --depth 2 --budget 3000

Parceltongue:
  changed entity parse_user
  impacted callers
  test gaps
  confidence caveats
  suggested test command

Codex:
  runs tests and fixes.
```

This is exactly where raw repo search wastes tokens. The graph should supply the
dependency context before the edit and the blast radius after the edit.

### Agent Journey: Refactoring A Public Interface

```text
User:
  "Rename the public UserRepository method save_user to persist_user."

Codex:
  ptctx entity UserRepository::save_user --json

Parceltongue:
  visibility public
  interface surface yes
  direct callers 14
  tests 3
  external route flow 2
  confidence high

Codex:
  ptctx relations UserRepository::save_user --direction callers --depth 2 --budget 6000

Parceltongue:
  groups callers by module
  shows high-risk public edges first
  returns expansion cursor for low-risk test-only edges

Codex:
  edits with confidence.
```

This is where "public interface dependency graph" becomes more valuable than a
plain call graph.

### Shreyas Doshi Product Reading

From a Shreyas-style product lens, this concept is about narrowing the job.

The job is not:

```text
visualize all code relationships
```

The job is:

```text
help a coding agent decide the next source span to inspect or edit with fewer
tokens and less risk
```

That means the strongest product bets are:

```text
start context:
  what should the agent inspect first?

minimal edit context:
  what is the smallest context needed to edit one symbol?

relations:
  what calls this, what does it call, what tests it?

impact:
  after this change, what might break?

trace:
  why does the graph believe this edge exists?

query-pack diagnostics:
  is this language support honest?
```

Graph visualization is secondary. It may be useful later, but it is not the
core PMF for this user.

The PMF is a local, truthful, token-budgeted dependency oracle for Codex.

### What Parceltongue Should Avoid

Avoid:

```text
claiming universal call-graph correctness
loading huge tool descriptions into MCP
returning giant arrays without caps
compressing away entity ids
hiding confidence
hiding truncation
treating grammar availability as language support
mixing query syntax checks with semantic query-pack checks
building UI before the CLI/API is sharp
storing every AST node as a product entity
```

Prefer:

```text
small stable CLI
compact JSON
persistent graph
source-span provenance
language-pack fixtures
confidence tiers
bounded traversal
token-budget tests
omitted frontier handles
Codex-first workflows
```

### The Concrete Parceltongue V2 Architecture

```text
parseltongue-core
  source discovery
  language packs
  parser sessions
  query-pack compilation
  fact extraction
  graph store
  traversal engine
  context selector

parseltongue-cli
  start
  map
  entity
  relations
  context
  impact
  trace
  query-pack check

parseltongue-mcp
  thin wrapper around CLI/core
  small tool list
  compact descriptions
  hidden management tools

parseltongue-fixtures
  per-language samples
  expected entities
  expected edges
  expected omissions
  token budget snapshots

parseltongue-agent-docs
  Codex usage recipes
  when to use each command
  caveats by language
```

The architecture should be boring and inspectable. That is a feature.

### The Single Most Important Design Rule

Every agent-facing answer should include:

```text
what I included
why I included it
what I omitted
how to expand it
how confident I am
where the evidence came from
```

That can be compact.

Example:

```json
{
  "target": "parse_user",
  "included": ["target_code", "direct_callees", "direct_tests"],
  "omitted": [{"kind": "caller_frontier", "count": 12, "cursor": "C12"}],
  "confidence": "mixed",
  "traces": ["trace_1", "trace_2"],
  "next": [
    {"tool": "relations", "args": {"cursor": "C12", "depth": 1}}
  ]
}
```

This is the compact contract that makes a graph tool useful to an LLM.

### Concept 29 Conclusion

The strongest pattern across these repos is not "use Tree-sitter."

It is:

```text
use Tree-sitter as the evidence layer for a graph fact pipeline,
then expose that graph through bounded, token-tested, agent-facing workflows.
```

Parceltongue should evolve in that direction.

Not:

```text
AST search tool
```

Not:

```text
repo visualization toy
```

Not:

```text
LLM assistant replacement
```

But:

```text
local structural memory for Codex
with traceable graph facts
bounded dependency traversal
minimal edit context
public interface impact
language-pack honesty
and executable token budgets
```

That is a real product for a solo agent power user.

### Repos And Docs Touched For This Concept

| Source | Evidence | Notes |
|---|---|---|
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-graph` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/tree-sitter__tree-sitter-graph-20260706-225258`; list_projects reported 1156 nodes and 4042 edges. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-tsq` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/tree-sitter__tree-sitter-tsq-20260706-225258`; list_projects reported 200 nodes and 228 edges. |
| `git-ref-repo/ignore-this-folder-repos/Christoph__treesitter-mcp` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/Christoph__treesitter-mcp-20260706-225258`; list_projects reported 2498 nodes and 12159 edges. |
| `git-ref-repo/ignore-this-folder-repos/sdsrss__code-graph-mcp` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/sdsrss__code-graph-mcp-20260706-225258`; list_projects reported 4934 nodes and 22644 edges. |
| `git-ref-repo/ignore-this-folder-repos/tirth8205__code-review-graph` | codebase-memory indexed | Indexed at `/tmp/codex-code-intel/codebase-memory/tirth8205__code-review-graph-20260706-211635`; list_projects reported 4570 nodes and 18789 edges. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-graph/README.md` | lines 1-60 | Official DSL positioning: construct arbitrary graph structures from Tree-sitter parsed source, usable as library or CLI. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-graph/src/graph.rs` | lines 1-260 | Graph model: syntax node references are separate from graph nodes; graph nodes have edges and attributes; serialization exposes ids, edges, and attrs. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-graph/src/execution.rs` | lines 1-260 | Execution model: parsed DSL file runs against Tree-sitter tree and source text, supports execute into existing graph, strict/lazy behavior, captures, quantifiers, and query locations. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-graph/src/ast.rs` | lines 1-260 | DSL AST: files, stanzas, queries, statements, graph node creation, edge creation, attributes, variables, scan, if, and for-in. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-graph/tests/it/graph.rs` | lines 1-106 | Tests for graph nodes, edges, attributes, pretty print, and syntax-node-backed attributes. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-graph/tests/it/execution.rs` | lines 1-260 | Tests showing graph DSL execution against parsed Python source, node/edge creation, attributes, scan strings, local variables, scoped variables, and repeated stanza matching. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-tsq/README.md` | lines 1-8 | Query-language grammar repo status and redirect to newer query grammar. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-tsq/grammar.js` | lines 1-97 | Tree-sitter query language grammar: patterns, predicates, captures, alternation, groups, node names, fields, wildcard, and quantifiers. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-tsq/test/corpus/patterns.txt` | lines 1-220 | Query syntax corpus: alternation, anchors, anonymous leaves, groups, quantifiers, named nodes. |
| `git-ref-repo/ignore-this-folder-repos/tree-sitter__tree-sitter-tsq/test/corpus/test_query_errors_on_invalid_symbols.txt` | lines 1-48 | Important distinction: syntactically valid query-language examples can still be semantically invalid against a target language grammar. |
| `git-ref-repo/ignore-this-folder-repos/DeusData__codebase-memory-mcp/README.md` | lines 17-240 | README-level claims about fast indexing, many languages, Tree-sitter plus LSP, persistent graph, edge types, MCP tools, shared graph artifacts, agent-as-query-translator architecture, and token savings. Claims not independently verified here. |
| `git-ref-repo/ignore-this-folder-repos/sdsrss__code-graph-mcp/README.md` | lines 1-260 | MCP graph server positioning: AST graph, semantic search, call graph traversal, HTTP route tracing, impact, incremental indexing, context compression, tool list, and performance claims. |
| `git-ref-repo/ignore-this-folder-repos/sdsrss__code-graph-mcp/src/parser/treesitter.rs` | lines 1-260 | Parsed node schema, parser cache, timeout, language lookup, test-context detection, and multi-language extraction. |
| `git-ref-repo/ignore-this-folder-repos/sdsrss__code-graph-mcp/src/parser/lang_config.rs` | lines 1-222 | Centralized language config and warning that call expressions cannot be fully abstracted behind one node-kind string. Includes support guard tests. |
| `git-ref-repo/ignore-this-folder-repos/sdsrss__code-graph-mcp/src/graph/query.rs` | lines 1-260 | Recursive call graph traversal with depth caps, row caps, confidence gates, cycle detection, parent ids, truncation flags, and suppressed ambiguous edge counts. |
| `git-ref-repo/ignore-this-folder-repos/sdsrss__code-graph-mcp/src/sandbox/compressor.rs` | lines 1-260 | Multi-level context compression by node, file, and directory with token estimates and tests retaining node IDs in summaries. |
| `git-ref-repo/ignore-this-folder-repos/sdsrss__code-graph-mcp/src/mcp/tools.rs` | lines 1-220 | Curated visible MCP tool list, hidden management tools, concise descriptions, and tests keeping tool descriptions bounded. |
| `git-ref-repo/ignore-this-folder-repos/Christoph__treesitter-mcp/README.md` | lines 1-260 | AST-first MCP positioning, token comparison claims, recommended agent workflows, supported languages, and tool guide. |
| `git-ref-repo/ignore-this-folder-repos/Christoph__treesitter-mcp/src/tools.rs` | lines 1-260 | MCP tool descriptions for `view_code`, `code_map`, `find_usages`, `minimal_edit_context`, and `call_graph`, including use cases, caveats, and token behavior. |
| `git-ref-repo/ignore-this-folder-repos/Christoph__treesitter-mcp/src/common/budget.rs` | lines 1-36 | Simple budget tracker and token estimation helper. |
| `git-ref-repo/ignore-this-folder-repos/Christoph__treesitter-mcp/src/analysis/minimal_edit_context.rs` | lines 1-620 | Minimal edit context implementation: parse file, find symbol, collect called names, dependency signatures, imports, types, compact rows, and tiktoken budget enforcement. |
| `git-ref-repo/ignore-this-folder-repos/Christoph__treesitter-mcp/src/analysis/call_graph.rs` | lines 1-320 | Compact best-effort caller/callee graph with depth cap, project file scan, definition collection, deduped edges, and tiktoken budget. |
| `git-ref-repo/ignore-this-folder-repos/Christoph__treesitter-mcp/src/analysis/review_context.rs` | lines 1-245 | Review bundle composition: parse diff, affected-by-diff, minimal edit context, relevant tests, compact fields, and budget trimming. |
| `git-ref-repo/ignore-this-folder-repos/Christoph__treesitter-mcp/tests/token_efficiency_test.rs` | lines 325-430, 660-850 | Tests for token budget enforcement and aggregate token-efficiency workflows: overview, focused edit, call graph, repo search, and directory map. |
| `git-ref-repo/ignore-this-folder-repos/tirth8205__code-review-graph/README.md` | lines 39-260 | Product positioning: Tree-sitter structural graph, MCP assistant context, blast radius, incremental updates, language coverage, CI review, benchmarks, caveats, and limitations. |
| `git-ref-repo/ignore-this-folder-repos/tirth8205__code-review-graph/docs/schema.md` | lines 1-260 | Node and edge schema, qualified names, SQLite tables, confidence fields, flows, communities, FTS, risk index, and summaries. |
| `git-ref-repo/ignore-this-folder-repos/tirth8205__code-review-graph/code_review_graph/graph.py` | lines 1-260 | SQLite-backed graph store, schema, node/edge dataclasses, indexes, WAL mode, upserts, confidence tier, and file removal. |
| `git-ref-repo/ignore-this-folder-repos/tirth8205__code-review-graph/code_review_graph/parser.py` | lines 1-300 | Parser data models, language mapping, shebang handling, and Tree-sitter node type mappings across languages. |
| `git-ref-repo/ignore-this-folder-repos/tirth8205__code-review-graph/code_review_graph/tools/context.py` | lines 1-152 | `get_minimal_context`: graph stats, risk, communities, flows, and next tool suggestions in an ultra-compact response. |
| `git-ref-repo/ignore-this-folder-repos/tirth8205__code-review-graph/code_review_graph/tools/review.py` | lines 1-320 | `get_review_context`: changed files, impact radius, graph nodes/edges, source snippets, review guidance, minimal detail level, context savings. |
| `git-ref-repo/ignore-this-folder-repos/tirth8205__code-review-graph/code_review_graph/context_savings.py` | lines 1-260 | Estimated token savings, file token estimates, attached metadata, optional tiktoken verification, and CLI savings panel. |
| `git-ref-repo/ignore-this-folder-repos/tirth8205__code-review-graph/code_review_graph/main.py` | lines 164-310 | MCP entry points for minimal context, impact radius, query graph, review context, and semantic search with compact/detail controls. |

## Concept 30: Choose Tool Roles, Not One Mega Tool

The wrong question is:

```text
Which repo should become the mega coding agent?
```

The better question is:

```text
When Codex is working in a large repo,
what role does it need filled at each moment,
and which tool fills that role with the least friction?
```

This matters because the tools in `J001`, `J002`, and the clone folder are not one species.
They are not all "Tree-sitter code graph tools."
They are different products hiding behind similar language.

Some are fast search tools.
Some are semantic retrieval tools.
Some are file-level dependency visualizers.
Some are graph database backends.
Some are graph report generators.
Some are review-context compressors.
Some are AST viewers.

For a solo Codex power user, the PMF question is not:

```text
Can this tool understand code?
```

The PMF question is:

```text
Can this tool help the agent decide what to read next
with fewer tokens,
less hallucination,
and lower chance of missing the dependency that matters?
```

That framing changes the ranking.

### The Product Stack I Would Actually Use With Codex

The Codex stack should be role-based:

| Role | Best Current Candidate | Why This Role Exists | What Codex Should Ask |
|---|---|---|---|
| Fast exact/fuzzy locate | `dmtrKovalenko/fff` | Agent needs to find files, paths, definitions, and text fast without paying graph-build cost. | "Where is the likely file or symbol?" |
| Semantic code retrieval | `glommer/codemogger`, `cocoindex-io/cocoindex-code` | Agent does not know exact names and needs AST-aware semantic chunks. | "Which chunks are relevant to this intent?" |
| File-level dependency impact | `LegacyCodeHQ/clarity-cli` | Agent needs to know what files depend on the file being edited and how the change shape looks. | "What depends on this file, and what changed structurally?" |
| Persistent project graph memory | `Graphify-Labs/graphify` | Agent needs a reusable project graph across code, docs, reports, papers, and other artifacts. | "What does the project graph already know?" |
| Deep symbol and relationship graph | `CodeGraphContext/CodeGraphContext` | Agent needs richer symbol, call, import, inheritance, datasource, and module relationship traversal. | "What are the callers, callees, imports, class hierarchy, and storage edges?" |
| Minimal edit context and public-interface impact | `Parceltongue V2` | Agent needs a small, deterministic, local dependency oracle tuned for editing with Codex. | "What must I read before changing this public surface?" |

This is not an argument against building Parceltongue.
It is an argument against building Parceltongue as a vague mega-tool.

The more precise product thesis is:

```text
Parceltongue should be the bounded dependency-context oracle
for Codex edits in large codebases.
```

Not the fastest text searcher.
Not the best embedding searcher.
Not the biggest graph database.
Not the prettiest knowledge graph.
Not the generic MCP buffet.

It should answer the narrow question that keeps biting agents:

```text
Before I edit this symbol, file, trait, public function, API route,
schema object, or module boundary,
what is the smallest dependency-shaped context I must inspect?
```

That is the gap.

### Concept 30A: fff Is The Low-Token Locator, Not The Graph

`fff` is extremely relevant, but not because it replaces Parceltongue.
It is relevant because every agent session starts with locating.

The agent asks:

```text
Where is the thing?
Where are the nearby names?
Where is the likely definition?
Where is the text pattern?
Which files should I open first?
```

`fff` is built for that moment.

The README positions it as a file search toolkit for humans and AI agents, with typo-resistant path and content search, frecency, a watcher, and an in-memory index.
Its MCP section explicitly mentions working with Claude Code, Codex, OpenCode, Cursor, and other clients.
The exposed tools include content search, file search, and multi-grep.
The agent-facing details include context, cursor pagination, weak-match detection, definition-first hints, smart-case behavior, fuzzy fallback, and git-aware annotations.

That makes it a likely "least tokens to first useful clue" tool.

But:

```text
fff does not know that changing `Foo::new` breaks a trait implementor.
fff does not know that a public TypeScript export is the contract boundary.
fff does not know that a Rust enum variant is serialized into a config file.
fff does not know that a helper has no callers because the call edge is dynamic.
fff does not give the agent a dependency proof.
```

So the role is:

```text
Use fff to find candidate files and exact text fast.
Use Parceltongue to decide dependency context after the candidate is found.
```

Shreyas-style PMF read:

```text
High frequency, low ceremony, obvious daily pull.
But it is a search primitive, not the product strategy.
```

### Concept 30B: Codemogger And CocoIndex Are Semantic Retrieval, Not Dependency Truth

`glommer/codemogger` and `cocoindex-io/cocoindex-code` are also highly relevant.
They solve the case where the agent does not know the exact names.

The agent asks:

```text
I need the billing renewal logic.
I need the place where websocket reconnects are handled.
I need the Rust code that parses this DSL.
I need the code related to "public API compatibility".
```

A pure lexical search can miss that.
An AST-aware semantic index can find the right chunk faster.

`codemogger` is especially interesting because its README describes:

```text
Tree-sitter chunking into semantic units
local embeddings
SQLite with vector plus FTS
no Docker
no server
no API keys
one database file per codebase
Turso embedded storage
incremental indexing by file hash
MCP tools for search, index, and reindex
```

That is very close to the "agent-native local memory" shape.

`cocoindex-code` is interesting because it packages AST-based semantic code search as CLI, MCP, and skill.
Its MCP shape is intentionally simple: one `search` tool with query, limit, offset, refresh, language, and path filters.
The realtime indexing demo describes Tree-sitter chunking, incremental processing, live updates, and a vector target.

But semantic retrieval is not dependency analysis.

Semantic search answers:

```text
What code seems related to this meaning?
```

Dependency context answers:

```text
What code must be considered because it calls, imports, implements,
overrides, serializes, exposes, reads, writes, or depends on this thing?
```

Those are different promises.

For Parceltongue, the right lesson is:

```text
Borrow AST chunking and local index ergonomics.
Do not confuse similarity with graph evidence.
```

A Codex agent should happily use semantic retrieval before graph traversal:

```text
semantic search -> candidate symbol -> dependency graph -> minimal edit context
```

If Parceltongue does embeddings one day, that should be the front door.
The core should still be dependency-shaped facts with evidence and confidence.

### Concept 30C: Clarity Is The Best Immediate Codex Companion

`LegacyCodeHQ/clarity-cli` has the strongest immediate product taste for Codex.

It does not try to be everything.
It says, in effect:

```text
Show me the dependency shape of a code change.
Show me what depends on this file.
Show me file/module coupling.
Show me cycles.
Show me paths between areas.
Let me verify the structure before I commit.
```

That is a beautiful fit for an agent.

The Clarity README is explicit that it builds dependency impact graphs from source and shows files, modules, tests, docs, and their connections.
It is also explicit about the limitation: file granularity, coupling shape, not runtime behavior or full API contracts.

That honesty is product-positive.

The `AGENTS.md` in the Clarity clone is even more revealing.
It tells agents when to use Clarity:

```text
Use it for design discussion.
Use it for refactoring verification.
Generate graph output for developer review.
Read DOT or Mermaid directly for agent verification.
Before refactoring a file, inspect dependents with `clarity show <file> --reach up`.
Use bounded reach for blast radius.
Use whole-tree collapsed views for architecture.
Use `--between` to inspect paths between files.
```

That is exactly the user journey Codex needs:

```text
1. Agent identifies file to edit.
2. Agent asks Clarity what depends on it.
3. Agent edits.
4. Agent asks Clarity what the change shape now looks like.
5. Agent tests.
6. Agent commits only if the graph shape and tests agree.
```

This is not just a tool.
It is an agent habit.

For Parceltongue, Clarity is the strongest product design reference:

```text
make the correct graph question feel obvious
make the output inspectable by both human and agent
support before/after structural verification
stay honest about granularity
```

Where Parceltongue can go beyond Clarity:

```text
symbol-level facts
public interface boundaries
callers/callees where reliable
trait/interface implementation impact
test-selection hints
token-budgeted context bundles
confidence-tagged edges
language-pack precision
```

The PMF is not "replace Clarity."

The PMF is:

```text
Clarity for file-level change shape.
Parceltongue for dependency-shaped minimal edit context.
```

### Concept 30D: Graphify Is Project Memory And Graph Ritual

`Graphify-Labs/graphify` is not a narrow code dependency tool.
It is a project knowledge graph workflow.

Its README and skill file describe a tool that can ingest code, docs, papers, images, and videos, then output:

```text
graph.html
GRAPH_REPORT.md
graph.json
Obsidian-style outputs
GraphRAG-ready JSON
plain-language report
MCP query/path/explain tools
```

Its architecture document is product-relevant because the extraction schema is simple:

```text
nodes: id, label, source_file, source_location
edges: source, target, relation, confidence
confidence: EXTRACTED, INFERRED, AMBIGUOUS
```

That is an important pattern.

For agents, confidence is not decoration.
It is the difference between:

```text
I can safely use this edge as a fact.
I should inspect the source before trusting this edge.
I should ask a human or run another tool.
```

Graphify's Codex skill is also worth copying as a workflow idea.
It says: if `graphify-out/graph.json` exists and the user asks a natural language codebase question, use the graph first.
That creates a persistent "graph-first" ritual.

This is the agent-memory product pattern:

```text
if graph exists -> query graph before rereading repo
if graph is stale -> update graph
if edge is inferred -> verify with source
if answer is broad -> use report/community/god-node view
```

For Parceltongue, Graphify's lesson is:

```text
make the graph artifact reusable across sessions
make confidence labels first-class
make query/path/explain a normal part of agent behavior
make the output useful even when no MCP server is running
```

But Graphify is not enough for the specific Parceltongue gap.

It is broad memory.
It is not a dedicated "what must I read before editing this public function" oracle.

### Concept 30E: CodeGraphContext Has The Richest Graph Surface, But The Adoption Risk Is Friction

`CodeGraphContext/CodeGraphContext` is the largest and richest prior-art reference in this slice.

The README positions it as an MCP server and CLI toolkit that indexes local code into a graph database for AI assistants and developers.
The architecture document describes a graph database exposed through MCP, CLI, visualization server, website, multiple database backends, many languages, Tree-sitter, optional SCIP, bundles, watcher, jobs, and graph writer.
The MCP tools documentation lists a broad tool surface: contexts, indexing, package indexing, stats, code search, relationship analysis, dead code, complexity, bundles, watcher, Cypher, visualization, reports, Java Spring tools, and datasource ingestion.

The source is even more useful than the README.

The tool definitions expose relationship queries including:

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
functions by argument
functions by decorator
```

The schema contract includes nodes such as:

```text
Repository
Directory
File
Function
Class
Trait
Variable
Interface
Macro
Struct
Enum
EnumMember
Union
Record
Property
Annotation
Module
```

And relationships such as:

```text
CONTAINS
CALLS
IMPORTS
INHERITS
HAS_PARAMETER
INCLUDES
IMPLEMENTS
PARTIAL_OF
PART_OF
DECORATED_BY
METACLASS
COMPANION_OF
EMBEDS
INJECTS
EXPOSES_ENDPOINT
PROVIDES_BEAN
MODULE_DEPENDS_ON
USES_LIBRARY
CHILD_MODULE
FILE_BELONGS_TO
READS
WRITES
MAPS_TO
HAS_COLUMN
STORED_IN
```

This is very close to the "universal relationship graph" idea.

The call-resolution code is especially valuable because it admits uncertainty.
It uses confidence tiers:

```text
1.00 explicit receiver
0.95 same-file local
0.88 inferred receiver with fully qualified import
0.72 inferred receiver fallback
0.90 unique short or same-package match
0.85 qualified or wildcard import
0.70 fully qualified path substring
0.25 ambiguous alphabetical first
0.08 unresolved object method same-file fallback
```

That is exactly the kind of honesty Parceltongue needs.

But I also used the CodeGraphContext evidence-reader workflow locally, and the operational signal matters.

Using the CodeGraphContext evidence-reader wrapper:

```text
/Users/amuldotexe/.codex/skills/codegraphcontext-evidence-reader/scripts/scan_current_repo_only.sh
```

I attempted a smoke index on the CodeGraphContext clone:

```text
repo: git-ref-repo/ignore-this-folder-repos/CodeGraphContext__CodeGraphContext
out: /tmp/codex-code-intel/codegraphcontext/CodeGraphContext__CodeGraphContext-20260706-230309
```

The run initialized services and began re-indexing.
It produced:

```text
ladybugdb.sqlite: 21 MB
ladybugdb.sqlite.wal: 15 MB
index.txt
```

But the session ended with signal 143 before normal smoke output.
A concurrent read while indexing failed because the database lock was held by the index process.
After the process exited, `cgc stats` and `cgc list` against that partial database failed with:

```text
Database Connection Error: std::bad_alloc
```

I then attempted a smaller smoke index on `dmtrKovalenko/fff`:

```text
repo: git-ref-repo/ignore-this-folder-repos/dmtrKovalenko__fff
out: /tmp/codex-code-intel/codegraphcontext/dmtrKovalenko__fff-20260706-231648
```

That run also initialized services and began re-indexing, but ended with signal 143 before `list` or `stats`.
It left:

```text
ladybugdb.sqlite: 4 KB
ladybugdb.sqlite.wal: 3.4 MB
index.txt
```

It also left an untracked auto-generated `.cgcignore` inside the `fff` clone.
I removed that artifact after confirming it was untracked.

Important caveat:

```text
This is not proof that CodeGraphContext is generally broken.
It is evidence that, in this local Codex desktop research session,
the CGC smoke path was not low-friction or quick enough to become
a reliable always-on helper.
```

There were also other CGC index jobs running on the machine at the same time, so the environment was not a clean benchmark.

Still, from a product point of view, the lesson is strong:

```text
Capability is not adoption.
For a Codex companion, index predictability is part of the product.
```

If Parceltongue borrows from CodeGraphContext, it should borrow:

```text
rich relationship vocabulary
source-backed graph schema
confidence tiers
caller/callee/import/hierarchy queries
optional graph database backend
datasource-aware edge categories
MCP plus CLI duality
```

But it should avoid:

```text
too many visible tools
opaque long indexing
unbounded repo scans
partial databases that fail unclearly
tool-generated artifacts inside source clones
ignore-pattern mismatches
graph backends that feel heavier than the edit loop
```

### Concept 30F: The Ignore-Pattern Footgun Is A Product Requirement

One very practical discovery:

```text
The CodeGraphContext evidence-reader wrapper ignores `gitrefrepo/`.
This repo's clone folder is named `git-ref-repo/`.
```

That means running the wrapper on the whole Parceltongue workspace would risk indexing the entire reference clone forest.

This sounds minor.
It is not minor.

For an agent, path ignore correctness is product correctness.

If the graph tool accidentally indexes:

```text
reference clones
vendored dependencies
build artifacts
model caches
test fixtures
generated output
old experiments
```

then every answer becomes less trustworthy.

The graph may still be technically "right", but the product is wrong because the agent's next action is polluted.

Parceltongue V2 should make ignore behavior visible:

```text
parseltongue index --dry-run
parseltongue index --explain-scope
parseltongue index --print-excluded
parseltongue stats --by-root
parseltongue doctor --scope
```

And every answer should carry the scope:

```text
repo_root: /path/to/repo
indexed_roots: src/, crates/, packages/
excluded_roots: git-ref-repo/, target/, node_modules/
language_packs: rust=exact-ish, ts=imports-only, cpp=scip-missing
```

For Codex, scope must be query-visible.

### Concept 30G: Why The Earlier Bullseye List Looked Too Small

The earlier "bullseye" framing naturally favored tools that already do:

```text
LLM asks query to tool
tool returns dependency-shaped context
LLM decides what to inspect next
```

That bias made tools like `treesitter-mcp`, `code-graph-mcp`, `code-review-graph`, and `CodeGraphContext` look more central.

But if the product is:

```text
Solo Codex power user navigating large CRUD, Rust, C, and C++ systems repos
```

then the bullseye must be wider.

The agent's daily loop includes:

```text
finding
semantic recall
dependency impact
context compression
review verification
public API change safety
test selection
session memory
```

So the short list should not be only three tools.
It should be a stack.

The stack has layers:

| Layer | Question | Candidate Tools |
|---|---|---|
| Locate | "Where is it?" | `fff`, `ripgrep`, `codemogger grep`, `cocoindex-code search` |
| Recall | "What code is conceptually related?" | `codemogger`, `cocoindex-code`, `codebase-memory-mcp` |
| Structure | "What files/modules are coupled?" | `clarity-cli`, `Graphify`, `code-review-graph` |
| Symbol graph | "Who calls, imports, implements, overrides?" | `CodeGraphContext`, `code-graph-mcp`, `treesitter-mcp`, Parceltongue |
| Minimal context | "What must the agent read next?" | `treesitter-mcp`, `code-review-graph`, Parceltongue V2 |
| Verification | "Did the edit preserve shape and tests?" | `clarity-cli`, test suite, Parceltongue impact checks |
| Memory | "What did we learn across sessions?" | `Graphify`, codebase-memory systems, Codex docs |

The earlier "three" were closer to the symbol/minimal-context layer.
For an actual solo Codex workflow, every layer matters.

### Concept 30H: The Codex App Journey

If I am using the Codex app, I want these journeys.

#### Journey 1: First Contact With A Large Repo

```text
User asks:
  "Orient yourself in this codebase."

Codex does:
  1. Run cheap filesystem discovery.
  2. Use Clarity for whole-tree or module dependency shape.
  3. Use Graphify if a persistent graph already exists.
  4. Use fff for exact entry points.
  5. Use codemogger or cocoindex when concepts are fuzzy.
  6. Write a short project orientation note.

Parceltongue role:
  Not first.
  It becomes useful after Codex has candidate symbols or public surfaces.
```

#### Journey 2: Change A Function Safely

```text
User asks:
  "Modify this function."

Codex does:
  1. Locate function with fff or rg.
  2. Ask Parceltongue for direct callers, direct callees, imports, tests,
     public surface status, and confidence.
  3. Ask Clarity for file-level dependents if file movement or refactor is likely.
  4. Read only the selected context bundle.
  5. Edit.
  6. Run targeted tests.
  7. Re-run Clarity or Parceltongue impact if shape changed.

Parceltongue role:
  Core.
  This is the product.
```

#### Journey 3: Refactor A Module Boundary

```text
User asks:
  "Move this module / split this package / clean this boundary."

Codex does:
  1. Use Clarity `--reach up` and `--reach both` for file/module blast radius.
  2. Use Parceltongue for symbol-level exported/public API dependencies.
  3. Use CodeGraphContext if richer import/class/call hierarchy is needed.
  4. Generate a bounded refactor plan.
  5. Edit in small commits.
  6. Verify graph shape after each phase.

Parceltongue role:
  Public-interface graph plus minimal dependency context.
```

#### Journey 4: Debug A Production-ish Failure

```text
User asks:
  "This error happens in this flow."

Codex does:
  1. fff search exact error message, log field, route, or function.
  2. Semantic search if wording is fuzzy.
  3. Parceltongue asks for upstream/downstream dependency path.
  4. CodeGraphContext or code-graph-mcp handles deeper call chain when needed.
  5. Codex reads the minimum path.
  6. Reproduces or writes a failing test.

Parceltongue role:
  Keep the bug path small and evidence-backed.
```

#### Journey 5: Change Public API Or Serialized Shape

```text
User asks:
  "Rename this field / change this endpoint / change this enum / alter this trait."

Codex does:
  1. Identify public surface.
  2. Ask Parceltongue for public-interface impact.
  3. Include callers, downstream consumers, tests, docs, schema files,
     generated code, and serialization boundaries if detectable.
  4. Ask Clarity for file/module reach.
  5. Use fff for literal references.
  6. Edit and run compatibility tests.

Parceltongue role:
  This is the second core product.
```

#### Journey 6: Long Session Memory

```text
User asks:
  "Continue from last time."

Codex does:
  1. Read the project notes.
  2. Query Graphify if project graph exists.
  3. Query Parceltongue if graph index exists and the task names a symbol.
  4. Avoid re-scanning the whole clone forest.

Parceltongue role:
  Local code relationship memory, not general project memory.
```

### Concept 30I: Public Interface Dependency Graph Is The Differentiator

The phrase "dependency graph" is too broad.

Most tools can say they have some graph.

The useful Parceltongue phrase is:

```text
public interface dependency graph
```

This means:

```text
Which code is part of the surface that other code relies on?
Which internal implementation nodes are safe to change locally?
Which public/exported/serialized/API nodes require wider impact review?
Which downstream tests and docs should be inspected?
Which edges are proven, inferred, or ambiguous?
```

The public-interface graph has node types like:

```text
public function
exported type
trait/interface
impl block
API route
database table or column mapping
serialized field
config key
CLI command
event name
message type
feature flag
test fixture contract
```

And edge types like:

```text
calls
imports
implements
overrides
serializes
deserializes
reads
writes
exposes
routes_to
tests
documents
generates
configures
depends_on
```

Clarity approaches this from file/module coupling.
CodeGraphContext approaches it from broad graph schema.
Graphify approaches it from cross-artifact graph memory.
code-review-graph approaches it from review impact context.
treesitter-mcp approaches it from minimal edit context.

But Parceltongue can own the intersection:

```text
public surface + dependency impact + minimal context + Codex edit loop
```

That is worth building.

### Concept 30J: Concrete PMF Evaluation

For the stated PMF:

```text
Can we use a particular library or tool for exploring a large codebase
both as an agent for search
and for being able to see dependency graphs of what calls whom?
```

My current scoring:

| Tool | PMF For This Need | Shreyas-Style Read | Use With Codex Now? | Parceltongue Lesson |
|---|---:|---|---|---|
| `LegacyCodeHQ/clarity-cli` | 90 | Best immediate habit loop: before/after dependency shape, file/module impact, refactor verification. Limitation is honest: file granularity, not full runtime or API contract truth. | Yes. Use from shell today. | Copy the workflow taste, bounded views, reach up/both, before/after verification. |
| `dmtrKovalenko/fff` | 84 | Extremely frequent search primitive. Likely low-token path to first clue. Not a graph. | Yes. Use as locate layer. | Do not rebuild fast fuzzy search unless needed; integrate or wrap. |
| `glommer/codemogger` | 80 | Strong local AST chunk plus vector/FTS story, Turso embedded, no API key. Great when names are unknown. | Yes, especially for semantic recall. | Borrow local single-DB ergonomics and AST chunk boundaries. |
| `cocoindex-io/cocoindex-code` | 78 | Clean agent integration and one-tool MCP search. Strong incremental Tree-sitter retrieval story. Not dependency graph. | Yes if semantic search setup is acceptable. | Borrow simple MCP surface and refresh/search model. |
| `Graphify-Labs/graphify` | 76 | Best cross-artifact memory and graph-first ritual. Broad graph, reports, confidence labels. Less precise for edit dependency truth. | Yes for project memory and docs/code graph reports. | Copy confidence labels, persistent graph artifact, graph-first skill behavior. |
| `CodeGraphContext/CodeGraphContext` | 74 | Richest schema and relationship surface. Powerful prior art. Local smoke was not low-friction in this session, so adoption risk is operational complexity. | Use selectively for deep graph analysis, not as the first daily reflex. | Borrow schema, confidence tiers, and relationship queries; avoid tool sprawl and opaque indexing. |
| `Christoph/treesitter-mcp` | 73 | Very relevant minimal edit context and AST-first MCP shape. Good model for query-to-context tool responses. | Yes if installed and language support matches. | Copy minimal edit context and token budget discipline. |
| `tirth8205/code-review-graph` | 72 | Strong review-context product: blast radius, compact context, SQLite graph, risk/community summaries. | Good reference, maybe use if setup is easy. | Copy review-oriented context bundles and context-savings instrumentation. |
| `sdsrss/code-graph-mcp` | 70 | Relevant graph traversal, compression handles, context compression, confidence gates. | Use as research/reference unless it proves easy locally. | Copy row/depth caps, truncation flags, and context compression handles. |
| `Parceltongue V2` | 92 if narrowed | Best possible PMF if it refuses to be a mega-tool and owns Codex edit safety through public-interface dependency context. | Build for self, not marketplace. | Become the local, bounded, evidence-backed dependency oracle. |

The ranking is not "stars ranking."
It is not "most technically ambitious."
It is "most likely to help a solo Codex power user avoid wasting context and missing dependency impact."

### Concept 30K: Final Recommendation For The Codex Power User

Use this stack now:

```text
fff
  for fast path/content/definition search

codemogger or cocoindex-code
  for semantic AST-aware retrieval when names are unknown

clarity-cli
  for file/module dependency shape and refactor verification

Graphify
  for persistent project graph memory and cross-artifact reports

CodeGraphContext
  as the heavy-duty relationship graph reference or occasional deep analyzer

Parceltongue V2
  for the missing narrow product: public-interface dependency context for Codex edits
```

The "daily loop" should be:

```text
locate with fff
recall with codemogger/cocoindex when fuzzy
inspect file/module impact with Clarity
ask Parceltongue for minimal dependency context
edit
test
verify shape with Clarity/Parceltongue
write durable note if lesson matters
```

The "deep loop" should be:

```text
if call/import/inheritance graph is genuinely needed
and the index can be built predictably
then use CodeGraphContext or a similar graph backend
else stay with smaller bounded tools
```

The "Parceltongue V2 MVP" should be:

```text
1. Index only the intended repo scope.
2. Emit an explicit scope report.
3. Extract public surfaces first.
4. Extract imports/calls/implements/tests with confidence.
5. Provide `what-must-i-read <symbol-or-file> --budget N`.
6. Provide `public-impact <symbol-or-file>`.
7. Provide `why-this-context` with source-backed evidence.
8. Provide `stale?` and `doctor --scope`.
9. Keep visible tools under 6.
10. Make every response bounded, traceable, and cheap enough for Codex.
```

The hardest product discipline is saying no.

Do not build:

```text
a generic graph database UI
a generic semantic search engine
a generic MCP server with 25 tools
a Graphify clone
a Clarity clone
a faster fff
```

Build:

```text
the tool that lets Codex say:

"Before I edit this, here is the smallest dependency-aware context
I must read, here is why, here is the confidence of each edge,
and here is the public-interface blast radius."
```

That is Parceltongue's real product.

### Repos And Docs Touched For This Concept

| Source | Evidence | Notes |
|---|---|---|
| `/Users/amuldotexe/.codex/skills/codegraphcontext-evidence-reader/SKILL.md` | full skill read | Used the required CodeGraphContext evidence-reader workflow. Key constraints: use local `cgc`, prefer wrapper, runtime output under `/tmp/codex-code-intel/codegraphcontext`, direct source reads remain final proof. |
| `/Users/amuldotexe/.codex/skills/codegraphcontext-evidence-reader/scripts/scan_current_repo_only.sh` | lines 1-68 | Wrapper creates isolated output, sets a global `.cgcignore`, runs `cgc index`, then `list`, `stats`, `query`, and `find type Function`. Observed ignore template contains `gitrefrepo/`, while this repo uses `git-ref-repo/`. |
| `/tmp/codex-code-intel/codegraphcontext/CodeGraphContext__CodeGraphContext-20260706-230309/index.txt` | local CGC run output | CGC initialized services and began re-indexing the CodeGraphContext clone, but the session ended with signal 143 before wrapper completion. Partial DB plus WAL remained. |
| `/tmp/codex-code-intel/codegraphcontext/dmtrKovalenko__fff-20260706-231648/index.txt` | local CGC run output | Smaller CGC smoke run on `fff` also initialized services and began re-indexing, but ended with signal 143 before `list` or `stats`. Removed an untracked auto-generated `.cgcignore` artifact from the clone. |
| `git-ref-repo/ignore-this-folder-repos/CodeGraphContext__CodeGraphContext/README.md` | lines 68, 128-184 | CGC positioning: MCP server and CLI toolkit, local code graph database, broad language support, Tree-sitter extraction, optional SCIP, multiple DB backends. |
| `git-ref-repo/ignore-this-folder-repos/CodeGraphContext__CodeGraphContext/docs/ARCHITECTURE.md` | lines 37-234 | Architecture: graph database exposed through MCP, CLI, visualizer, website; GraphBuilder, CodeFinder, watcher, jobs, bundles, Tree-sitter, SCIP, graph writer, DB backends. |
| `git-ref-repo/ignore-this-folder-repos/CodeGraphContext__CodeGraphContext/docs/MCP_TOOLS.md` | lines 1-240 | Tool surface: contexts, add/list/delete/stats, find code, analyze relationships, dead code, complexity, bundles, watcher, Cypher, visualization, report, Java Spring, datasources. |
| `git-ref-repo/ignore-this-folder-repos/CodeGraphContext__CodeGraphContext/src/codegraphcontext/tool_definitions.py` | lines 1-388 | 25 MCP tool definitions and supported relationship query types. |
| `git-ref-repo/ignore-this-folder-repos/CodeGraphContext__CodeGraphContext/src/codegraphcontext/tools/indexing/schema_contract.py` | lines 1-76 | Semantic graph node and relationship contract, including code, build, module, datasource, READS/WRITES, and storage mapping edges. |
| `git-ref-repo/ignore-this-folder-repos/CodeGraphContext__CodeGraphContext/src/codegraphcontext/tools/indexing/pipeline.py` | lines 1-284 | Full indexing pipeline: file discovery, import pre-scan, concurrent parse, graph writes, inheritance/call resolution, Java/Spring/build/datasource handling, optional embeddings. |
| `git-ref-repo/ignore-this-folder-repos/CodeGraphContext__CodeGraphContext/src/codegraphcontext/tools/indexing/resolution/calls.py` | lines 1-360 | Call-resolution confidence tiers and EXTRACTED/INFERRED/AMBIGUOUS labeling. |
| `git-ref-repo/ignore-this-folder-repos/CodeGraphContext__CodeGraphContext/src/codegraphcontext/tools/code_finder.py` | lines 539-1380 | Relationship query implementation: callers, callees, imports, hierarchy, overrides, dead code, complexity, all callers/callees, call chains, module dependencies, variable scope. |
| `git-ref-repo/ignore-this-folder-repos/LegacyCodeHQ__clarity-cli/README.md` | lines 9-243 | Clarity product shape: dependency impact graphs, files/modules/tests/docs connections, snapshot/anchor/lens/rendering model, refactor safety, cycles, watch, agent support, limitations. |
| `git-ref-repo/ignore-this-folder-repos/LegacyCodeHQ__clarity-cli/AGENTS.md` | lines 42-87 | Agent workflow: use Clarity for design and refactor verification, read DOT/Mermaid for structural feedback, inspect dependents with `clarity show <file> --reach up`. |
| `git-ref-repo/ignore-this-folder-repos/Graphify-Labs__graphify/README.md` | lines 21-260 | Graphify product shape: `/graphify`, graph.html, GRAPH_REPORT.md, graph.json, install skill, always-use-graph setup, confidence tags, broad file support. |
| `git-ref-repo/ignore-this-folder-repos/Graphify-Labs__graphify/ARCHITECTURE.md` | lines 1-85 | Pipeline, module responsibilities, node/edge schema, confidence labels, adding language extractors, security, and tests. |
| `git-ref-repo/ignore-this-folder-repos/Graphify-Labs__graphify/graphify/skill-codex.md` | lines 1-240 | Codex skill workflow: graph-first fast path, usage modes, query/path/explain, AST extraction, graph persistence, semantic extraction rules. |
| `git-ref-repo/ignore-this-folder-repos/glommer__codemogger/README.md` | lines 7-212 | Codemogger: Tree-sitter semantic chunking, local embeddings, SQLite/Turso vector plus FTS, one DB per codebase, MCP tools, incremental hashing. |
| `git-ref-repo/ignore-this-folder-repos/cocoindex-io__cocoindex-code/README.md` | lines 6-260 | CocoIndex Code: AST semantic code search, CLI/skill/MCP, single MCP search tool, Codex setup command, structural grep, index/search/status workflow. |
| `git-ref-repo/ignore-this-folder-repos/cocoindex-io__realtime-codebase-indexing/README.md` | lines 35-152 | Realtime indexing demo: Tree-sitter chunking, incremental processing, live updates, vector index target, unchanged chunks reused. |
| `git-ref-repo/ignore-this-folder-repos/dmtrKovalenko__fff/README.md` | lines 1-247 | fff: fast file search for humans and AI agents, MCP support for Codex and others, content/path/multi-grep tools, frecency, definition-first hints, smart-case/fuzzy fallback, programmatic APIs. |

## Concept 31: Turn Parceltongue V2 Into Five Codex Commands

Concept 30 said Parceltongue should not become a mega-tool.

Concept 31 makes that concrete.

The current repo already has a lot:

```text
Tree-sitter extraction
12-plus language support
CozoDB graph storage
ISGL1 v2 stable entity keys
dependency edges
reverse callers
forward callees
blast radius
smart context
file watcher
coverage diagnostics
graph algorithms
HTTP API
```

The risk is not that Parceltongue lacks ideas.
The risk is that Parceltongue has too many ideas and too many surfaces.

The docs already admit this tension.

The README says the stable binary can parse code with Tree-sitter, build a graph in CozoDB, query through many HTTP endpoints, do blast radius and reverse callers, and update incrementally through a watcher.
The same top banner says v2 must address architectural improvements:

```text
Search is O(n) full-scan, not indexed.
Graph algorithms run in Rust, not in CozoDB native.
No secondary indices for reverse lookups.
Unresolved import references are mixed with production code.
Schema is richer than product.
```

That last line is the product diagnosis:

```text
Schema richer than product.
```

V2 should not add another wide surface just because the graph can support it.
V2 should turn the existing graph into a small number of Codex-native decisions.

### The V2 Product Rule

For a solo Codex power user, every command should answer one of these questions:

```text
What repo did you index?
What can you see?
Where is the thing?
What must I read before editing it?
What breaks if this public surface changes?
Is the graph stale or lying?
```

That maps to five visible commands:

| Command | User Question | Agent Output |
|---|---|---|
| `parseltongue index` | "What did you index, and what did you ignore?" | Scope report, language coverage, entity count, edge count, warnings. |
| `parseltongue orient` | "What is this codebase shape?" | Small architecture/context summary within budget. |
| `parseltongue find` | "Where is the thing?" | Candidate entity or file pointers, not raw dumps. |
| `parseltongue context` | "What must I read before changing this?" | Budgeted source-backed dependency context with reasons. |
| `parseltongue public-impact` | "What public contract might I break?" | Public-surface blast radius, tests/docs/schema hints, confidence labels. |
| `parseltongue doctor` | "Can I trust this graph?" | Staleness, scope, ignored roots, language support, ambiguous edge count. |

That is technically six commands, but five product jobs.
`doctor` is not a workflow command.
It is trust infrastructure.

This command set is deliberately smaller than the current HTTP API.
The existing API can remain under the hood.
Codex should not have to manually sequence 26 endpoints every time.

### Command 1: `parseltongue index`

Current state:

```text
parseltongue pt01-folder-to-cozodb-streamer ./my-project --db rocksdb:mycode.db
```

This works, but it exposes implementation names.
The command says "Tool 01" and "CozoDB streamer" instead of "build the graph I will query."

V2 should expose:

```bash
parseltongue index . \
  --db .parseltongue/graph.db \
  --explain-scope \
  --json
```

The response should include:

```json
{
  "repo_root": "/repo",
  "db": ".parseltongue/graph.db",
  "indexed_roots": ["crates", "src", "packages"],
  "ignored_roots": [".git", "target", "node_modules", "git-ref-repo"],
  "languages": {
    "rust": {"files": 141, "entities": 1290, "edges": 4100, "confidence": "high"},
    "typescript": {"files": 33, "entities": 220, "edges": 520, "confidence": "medium"}
  },
  "warnings": [
    "go selector expressions are syntax-ambiguous without type info",
    "cpp compile_commands.json missing; header/public impact is best-effort"
  ]
}
```

This is the direct fix for the CGC ignore-footgun seen in Concept 30.
Scope must be visible.

The current ingestion code already has exclude patterns for `target`, `node_modules`, `.git`, build output, and virtualenvs.
The standalone streamer CLI even supports repeated `--exclude` patterns.
V2 should lift that into the main user-facing command and make the result inspectable.

Acceptance criteria:

```text
WHEN I run `parseltongue index . --explain-scope --json`
THEN the output SHALL list indexed roots and ignored roots
AND SHALL include language-by-language entity and edge counts
AND SHALL warn when reference clone roots like `git-ref-repo` are not ignored
AND SHALL not create untracked artifacts outside `.parseltongue/`
```

### Command 2: `parseltongue orient`

The README already has a "New Codebase Orientation" workflow:

```text
statistics
cycles
complexity hotspots
semantic clusters
coverage
```

That is the right idea.
But Codex should not manually call five endpoints unless it needs to.

V2 should expose:

```bash
parseltongue orient --budget 1500 --json
```

The response should be short:

```json
{
  "entities": 1972,
  "code_entities": 755,
  "edges": 4055,
  "languages": ["rust", "javascript"],
  "top_folders": [
    {"path": "crates/parseltongue-core", "entities": 680},
    {"path": "crates/pt08-http-code-query-server", "entities": 390}
  ],
  "risks": [
    {"kind": "cycle", "count": 3},
    {"kind": "hotspot", "entity": "rust:fn:handle_request:...", "reason": "high fan-in"}
  ],
  "next_queries": [
    "parseltongue find <name>",
    "parseltongue context <entity>",
    "parseltongue public-impact <entity>"
  ]
}
```

This is not a dashboard.
It is an agent orientation packet.

Acceptance criteria:

```text
WHEN Codex starts work in a repo
THEN `parseltongue orient --budget 1500` SHALL return enough context to choose the next query
AND SHALL not include full source bodies
AND SHALL include parse coverage warnings
AND SHALL include suggested next commands
```

### Command 3: `parseltongue find`

Current state:

```text
GET /code-entities-search-fuzzy?q=PATTERN
GET /code-entities-list-all
GET /code-entity-detail-view?key=ENTITY_KEY
```

The README and journey docs already frame fuzzy search as an agent step.
But current search is documented as O(n) full-scan.
That is fine for now if the output is honest, but V2 should establish the facade:

```bash
parseltongue find authenticate --kind function --budget 1000 --json
parseltongue find "public api" --kind public --json
parseltongue find src/auth.rs --kind file --json
```

The command should return candidate pointers:

```json
{
  "query": "authenticate",
  "matches": [
    {
      "entity": "rust:fn:authenticate:__src_auth:T1706284800",
      "name": "authenticate",
      "kind": "function",
      "visibility": "public",
      "file": "src/auth.rs",
      "line_range": [18, 64],
      "signature": "pub fn authenticate(...) -> Result<...>",
      "score": 0.93
    }
  ],
  "warnings": ["search backend is linear scan in this version"]
}
```

The key product difference:

```text
Find returns pointers and signatures.
Context returns source-backed bundles.
```

This separation prevents accidental token floods.

### Command 4: `parseltongue context`

This is the core command.

It should replace the agent having to manually call:

```text
detail
reverse callers
forward callees
blast radius
smart context
edges list
```

with:

```bash
parseltongue context rust:fn:authenticate:__src_auth:T1706284800 \
  --budget 4000 \
  --why \
  --json
```

The current `smart-context-token-budget` endpoint has the right algorithmic seed:

```text
direct callers get highest relevance
direct callees get very high relevance
transitive dependencies get lower scores by depth
greedy selection stays within budget
```

But current implementation returns entity keys, relevance type, relevance score, and heuristic token estimates.
It does not return a complete source-backed edit bundle.
It estimates tokens from key length plus a base number.

V2 should turn smart context into:

```json
{
  "focus": "rust:fn:authenticate:__src_auth:T1706284800",
  "budget": 4000,
  "tokens_used": 3580,
  "scope": {
    "repo_root": "/repo",
    "ignored_roots": ["target", "git-ref-repo"]
  },
  "selected": [
    {
      "entity": "rust:fn:authenticate:__src_auth:T1706284800",
      "role": "focus",
      "tokens": 420,
      "include": "signature_and_body",
      "reason": "requested focus entity",
      "source": {
        "file": "src/auth.rs",
        "line_range": [18, 64]
      }
    },
    {
      "entity": "rust:fn:login_handler:__src_handlers:T1706284900",
      "role": "direct_caller",
      "tokens": 380,
      "include": "signature_and_callsite",
      "reason": "calls focus entity at src/handlers.rs:44",
      "edge": {
        "kind": "calls",
        "confidence": "extracted",
        "evidence": "call_expression"
      }
    }
  ],
  "omitted": [
    {
      "entity": "rust:fn:audit_event:__src_audit:T1706285000",
      "reason": "budget exhausted",
      "score": 0.41
    }
  ],
  "warnings": [
    "2 callees are external dependencies with no local source"
  ]
}
```

The command's promise:

```text
Codex can read this response and know what to inspect next,
why it was selected,
what was omitted,
and how confident each dependency edge is.
```

Acceptance criteria:

```text
WHEN I run `parseltongue context ENTITY --budget 4000 --why`
THEN the output SHALL include the focus entity
AND SHALL include direct callers before transitive dependencies
AND SHALL include source spans or callsite spans for every selected item
AND SHALL include an omitted list when relevant entities exceed the budget
AND SHALL never exceed the requested token budget by more than 10 percent
```

### Command 5: `parseltongue public-impact`

This is the differentiator.

Current blast radius answers:

```text
If I change X, what entities depend on X?
```

V2 public impact should answer:

```text
If I change this public surface, which callers, implementors, tests,
docs, schemas, routes, or downstream contracts should Codex inspect?
```

Command:

```bash
parseltongue public-impact rust:trait:Authenticator:__src_auth:T1706284800 \
  --hops 2 \
  --budget 4000 \
  --json
```

Response:

```json
{
  "focus": "rust:trait:Authenticator:__src_auth:T1706284800",
  "public_surface": true,
  "surface_reason": "rust trait is public and has public method signatures",
  "impact": {
    "direct_callers": 7,
    "implementors": 3,
    "tests": 12,
    "docs": 2,
    "schema_or_route_edges": 0
  },
  "must_read": [
    {
      "entity": "rust:impl:Authenticator_for_JwtAuth:__src_jwt:T1706284800",
      "role": "implementor",
      "confidence": "extracted",
      "reason": "implements the public trait"
    },
    {
      "entity": "rust:fn:login_handler:__src_handlers:T1706284900",
      "role": "direct_caller",
      "confidence": "extracted",
      "reason": "calls public authentication surface"
    }
  ],
  "risk": {
    "level": "high",
    "reasons": [
      "public trait",
      "3 implementors",
      "7 direct callers",
      "tests exist in auth integration suite"
    ]
  }
}
```

This command requires a new first-class concept:

```text
public surface
```

Public surface is not the same as "has callers."

An entity can have many callers and still be private implementation.
An entity can have zero local callers and still be public API.

That is why dead-code elimination in the feature table correctly says "filter public APIs."

### Public Surface Rules By Language

Public-interface impact must be language-specific.
There is no universal AST node that means "safe public contract."

The V2 MVP should define a small rule pack:

| Language | MVP Public Surface Rules | Confidence |
|---|---|---|
| Rust | `pub` functions, `pub` structs/enums/traits, trait items, `impl Trait for Type`, `pub mod`, `pub use`. `pub(crate)` is internal-public and should be labeled separately. | High when visibility captured from AST. |
| TypeScript | `export` functions/classes/interfaces/types/enums, default exports, exported route handlers, exported object members when statically visible. | Medium until export queries are explicit. |
| JavaScript | `export`, `module.exports`, `exports.foo`, framework route registrations by heuristic. | Medium to low depending syntax. |
| Go | Uppercase identifiers at package scope, exported methods/types/interfaces, embedded public types. | High for naming rule, medium for method-set impact. |
| Java | `public` and `protected` classes/interfaces/methods, annotations such as controllers/routes when detected. | High for modifiers, medium for framework annotations. |
| C# | `public`, `protected`, interfaces, controllers, attributes, public DTOs. | High for modifiers, medium for reflection/DI. |
| Python | Names in `__all__`, non-underscore module functions/classes, decorators for routes/commands. | Medium because convention-heavy. |
| C/C++ | Header declarations, exported symbols, public class members, virtual methods, extern declarations. | Medium without compile database or SCIP. |
| SQL | Tables, views, columns, stored procedures. | High when parsed from DDL. |

The current query files do not yet capture enough of this.

Examples from current evidence:

```text
Rust entity query captures function/struct/enum/trait/impl/module names.
Rust dependency query captures calls, use declarations, impl Trait for Type, type refs, await, field access, iterator ops, generic types.
TypeScript entity query captures functions, arrow functions, classes, interfaces, type aliases, enums, methods.
TypeScript dependency query captures calls, methods, imports, constructors, property access, async, generics, extends, and interface extends.
Go dependency query explicitly notes selector expressions are ambiguous without type info.
C++ entity query captures functions, classes, structs, and enums.
```

So V2 must not claim public-impact precision before it adds visibility/export extraction.

### Fact Schema V2

Current CozoDB schema has:

```text
CodeGraph:
  ISGL1_key
  Current_Code
  Future_Code
  interface_signature
  TDD_Classification
  lsp_meta_data
  current_ind
  future_ind
  Future_Action
  file_path
  language
  last_modified
  entity_type
  entity_class
  birth_timestamp
  content_hash
  semantic_path
  root_subfolder_L1
  root_subfolder_L2

DependencyEdges:
  from_key
  to_key
  edge_type
  source_location
```

That is enough for v1.
It is not enough for public-impact V2.

V2 should add or materialize:

```text
EntityFacts:
  entity_key
  language
  entity_kind
  name
  qualified_name
  file_path
  line_start
  line_end
  visibility
  public_surface_kind
  signature_text
  body_hash
  signature_hash
  estimated_body_tokens
  estimated_signature_tokens
  extraction_rule_id
  extraction_confidence
  extraction_version

EdgeFacts:
  edge_id
  from_key
  to_key
  edge_kind
  source_file
  source_line_start
  source_line_end
  evidence_kind
  resolver_kind
  confidence
  confidence_reason
  language_rule_id

ContextBundleFacts:
  bundle_id
  focus_key
  query_kind
  token_budget
  tokens_used
  selected_keys
  omitted_keys
  created_at
  index_version
```

The key additions are:

```text
visibility
public_surface_kind
signature_text
confidence
confidence_reason
evidence spans
token estimates based on actual code/signature text
```

Without these, Codex cannot know whether it is looking at implementation detail or contract surface.

### Confidence Labels

V2 should use simple confidence labels:

```text
extracted
  The edge or surface is directly present in source syntax.

inferred
  The edge or surface is derived from a rule that is usually valid.

ambiguous
  The edge or surface could mean multiple things without type/runtime info.

external
  The edge points outside the indexed repo.

unresolved
  The parser saw a reference but could not resolve it to a local entity.
```

Examples:

```text
Rust `impl Trait for Type` -> extracted implements edge.
Go `Receiver.Method()` -> extracted selector expression, inferred call edge.
C++ header declaration -> extracted public surface, unresolved implementation if source not linked.
Python non-underscore function -> inferred public surface.
TypeScript `export interface Foo` -> extracted public surface.
```

This converts limitations into product trust.

### First Three Codex Workflows

#### Workflow 1: Before Edit

```text
User:
  "Change authenticate to accept device id."

Codex:
  1. `parseltongue find authenticate --kind function --json`
  2. `parseltongue public-impact <entity> --budget 3000 --json`
  3. `parseltongue context <entity> --budget 5000 --why --json`
  4. Reads selected source spans.
  5. Edits.
  6. Runs targeted tests.
  7. Re-runs `parseltongue public-impact <entity>` if signature changed.
```

Decision rule:

```text
If public-impact says public_surface=false and direct_callers <= 3,
Codex can do a smaller edit loop.

If public-impact says public_surface=true,
Codex must inspect callers, implementors, tests, and docs before editing.
```

#### Workflow 2: Debug Path

```text
User:
  "This login failure happens after token refresh."

Codex:
  1. Uses fff or normal search for exact error strings.
  2. `parseltongue find refresh_token --kind function --json`
  3. `parseltongue context <entity> --budget 6000 --why --json`
  4. Reads focus, direct callers, direct callees, error path, and tests.
  5. Writes failing test or reproduction.
  6. Fixes and verifies.
```

Decision rule:

```text
Use Parseltongue after locating the candidate.
Do not ask Parseltongue to replace search.
Ask it to narrow the dependency path.
```

#### Workflow 3: Boundary Refactor

```text
User:
  "Split auth/session into a cleaner boundary."

Codex:
  1. Runs Clarity for file/module reach.
  2. `parseltongue orient --budget 1500 --json`
  3. `parseltongue public-impact auth/session --budget 5000 --json`
  4. `parseltongue context <top-public-surface> --budget 6000 --why --json`
  5. Creates a staged refactor plan.
  6. Edits one public surface at a time.
  7. Verifies graph and tests after each stage.
```

Decision rule:

```text
Clarity owns file/module shape.
Parceltongue owns public symbol impact.
Codex owns the edit.
```

### Fixture Tests For V2

V2 should be fixture-driven before it is feature-driven.

Minimum fixtures:

| Fixture | What It Proves |
|---|---|
| Rust public trait plus two implementors plus three callers | `public-impact` finds trait surface, implementors, callers, and tests. |
| Rust private helper with many local callers | High fan-in does not equal public API. |
| TypeScript exported interface and default export | Export extraction marks public surfaces. |
| Go package with uppercase and lowercase functions | Public surface follows Go export naming. |
| C++ header plus source implementation | Header declaration is public surface; implementation is linked when possible. |
| Python `__all__` plus underscore helper | Public heuristic distinguishes exported from private convention. |
| SQL table plus Rust/TS model reference | Schema surface can appear as public impact. |
| Ambiguous Go selector expression | Edge is present with ambiguous or inferred confidence, not silent high confidence. |
| Over-budget dependency neighborhood | Context bundle includes selected and omitted lists. |
| Repo containing `git-ref-repo` | Scope report excludes research clone folder by default or warns loudly. |

These tests should come before new algorithms.

### What Not To Build First

Do not start V2 with:

```text
another graph visualization
another broad HTTP endpoint family
another embedding store
another MCP server with 25 tools
another architecture dashboard
another graph database migration
```

The local docs already had a critical decision that MCP integration was dropped because research found high token overhead.
That does not mean "never MCP."
It means:

```text
Do not make MCP the core product if it inflates the context surface.
Use CLI or a tiny MCP facade after the command contract is proven.
```

The V2 path should be:

```text
CLI first
JSON always
MCP later as a thin wrapper
HTTP endpoints remain implementation detail
```

### Implementation Order

The smallest useful implementation order:

| Step | Build | Why First |
|---|---|---|
| 1 | `parseltongue doctor --scope --json` | Prevents polluted graphs and makes trust visible. |
| 2 | `parseltongue index --explain-scope --json` | Makes indexing bounded and inspectable. |
| 3 | Public surface classifier for Rust, TypeScript, Go | Covers solo user's main CRUD and systems-language workflow. |
| 4 | Edge confidence labels | Turns parser ambiguity into agent-readable caution. |
| 5 | `parseltongue context --why --budget` with actual source snippets | Upgrades smart context from keys to edit-ready context. |
| 6 | `parseltongue public-impact` | The differentiating command. |
| 7 | Fixture suite and golden JSON outputs | Makes agent behavior stable. |
| 8 | Tiny Codex instructions | Teaches Codex when to call each command. |

This order is intentionally not glamorous.
It is the path to daily use.

### The Shreyas Product Read

The current product already proved the technical insight:

```text
Code is a graph, and an LLM should query the graph before reading raw files.
```

The V2 product should prove the workflow insight:

```text
Before an agent edits code, it should ask for the smallest public-impact-aware context bundle.
```

That is the emotional job.

The user does not want to admire a graph.
The user wants Codex to stop missing dependencies.

The product promise should be:

```text
Codex changes large codebases with less wandering,
less token waste,
and fewer missed dependency consequences.
```

That is Parceltongue V2.

### Repos And Docs Touched For This Concept

| Source | Evidence | Notes |
|---|---|---|
| `README.md` | lines 1-104 | Current product banner, working features, known limitations, LLM system prompt, endpoint sequence, and orient/search/trace/context workflow. |
| `README.md` | lines 228-305 | HTTP API reference with core, entity, graph, analysis, context, watcher, diagnostics, navigation, and graph analysis endpoints. |
| `README.md` | lines 309-560 | Quick start, query examples, entity key format, response format, language support, edge types, and ready-made workflows. |
| `docs/research000/UserJourney20260202v1.md` | lines 1-260 | End-to-end API testing, self-analysis metrics, setup, entity listing, reverse callers, forward callees, and blast radius use cases. |
| `docs/research001/PRD-research-20260131v1/PARSELTONGUE_V2_LLM_AGENT_INTERACTIONS.md` | lines 1-260 | Iterative intelligence loop: LLM queries CPU graph features, receives structured context, asks refined queries, and builds insight. |
| `docs/research000/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md` | lines 1-180 | Feature inventory, shipped baseline, PMF findings, LLM-CPU workflows, MCP dropped for token overhead, must-have workflows, API contract change impact, and test impact prediction. |
| `docs/research000/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md` | lines 520-620 | Technology stack summary, key insights, workflow features as high-PMF category, and strategic note that MCP overhead contradicted token-reduction value prop. |
| `docs/research001/PRD-research-20260131v1/00_EXECUTIVE_SUMMARY.md` | lines 1-185 | Agent memory patterns, preview/pointer, budget estimator, pagination, hot path cache, query composition, budget-aware planner, and ISG advantage over file systems. |
| `docs/research001/PRD-research-20260131v1/PARSELTONGUE_V2_BIDIRECTIONAL_LLM_ENHANCEMENT.md` | lines 1-260 | Bidirectional LLM-CPU enhancement concept and API examples for semantic-guided graph workflows. |
| `crates/parseltongue/src/main.rs` | lines 1-292 | Unified CLI currently exposes two implementation-named subcommands: folder-to-CozoDB streamer and HTTP code query server. |
| `crates/pt01-folder-to-cozodb-streamer/src/cli.rs` | lines 1-269 | Standalone ingestion CLI philosophy, defaults, exclude patterns, and repeated `--exclude` support. |
| `crates/pt08-http-code-query-server/src/route_definition_builder_module.rs` | lines 1-193 | Actual route surface for current HTTP endpoints. |
| `crates/parseltongue-core/src/entities.rs` | lines 12-123 | Language and entity type definitions, including Rust/JS/TS/Python/Java/C/C++/Go/Ruby/PHP/C#/Swift/Kotlin/Scala/SQL and core entity kinds. |
| `crates/parseltongue-core/src/entities.rs` | lines 251-620 | Interface signature, visibility, line ranges, language-specific signatures, code entity, entity class, TDD classification, LSP metadata, and usage analysis. |
| `crates/parseltongue-core/src/interfaces.rs` | lines 12-214 | Tool abstraction, context generation output, repository interface, language parser interface, interface chunks, and dependency list shape. |
| `crates/parseltongue-core/src/isgl1_v2.rs` | lines 1-320 | Stable entity identity, key format, semantic path, deterministic birth timestamp, content hashing, and incremental entity matching. |
| `crates/parseltongue-core/src/storage/cozo_client.rs` | lines 100-183 | CozoDB CodeGraph schema and DependencyEdges schema. |
| `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/smart_context_token_budget_handler.rs` | lines 1-292 | Smart context endpoint, relevance-weighted greedy selection, direct caller/callee scoring, transitive traversal, and current heuristic token estimate. |
| `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/blast_radius_impact_handler.rs` | lines 1-303 | Blast radius as reverse dependency traversal, BFS by hops, fuzzy key matching, and token estimate. |
| `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/reverse_callers_query_graph_handler.rs` | lines 1-255 | Reverse caller query endpoint, edge payload, scope filter, fuzzy matching, and direct CozoDB query. |
| `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/forward_callees_query_graph_handler.rs` | lines 1-214 | Forward callee query endpoint, direct dependency query, and edge payload. |
| `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/code_entity_detail_view_handler.rs` | lines 1-209 | Entity detail endpoint, source code return, scope filtering, and query-parameter key handling. |
| `entity_queries/rust.scm` | lines 1-32 | Rust entity extraction currently captures functions, structs, enums, traits, impl blocks, methods, and modules. |
| `dependency_queries/rust.scm` | lines 1-180 | Rust dependency query pack captures calls, use declarations, trait impls, type references, await, field access, iterator ops, and generic usage. |
| `entity_queries/typescript.scm` | lines 1-35 | TypeScript entity query captures functions, arrow functions, classes, interfaces, type aliases, enums, and methods. |
| `dependency_queries/typescript.scm` | lines 1-110 | TypeScript dependency query captures calls, methods, imports, constructors, property access, async, generics, class extends, and interface extends. |
| `entity_queries/go.scm` | lines 1-22 | Go entity query captures functions, methods, structs, and interfaces. |
| `dependency_queries/go.scm` | lines 1-115 | Go dependency query captures calls, method calls, imports, embeds, composite literals, goroutines, and documents selector ambiguity without type information. |
| `entity_queries/cpp.scm` | lines 1-19 | C++ entity query currently captures functions, classes, structs, and enums. |

## Concept 32: Write The Failing Tests Before Building Public Impact

The next Parceltongue move should be test-first.

Not because TDD is aesthetically pleasing.
Because this particular product can lie very convincingly if it is not tested through exact contracts.

A graph code assistant can say:

```text
Here is the blast radius.
Here is the context bundle.
Here is the public impact.
```

and still be wrong in the only way that matters:

```text
It missed the thing Codex needed to know before editing.
```

That is the failure mode to design against.

The existing repo already has a serious testing culture.
It has fixture categories, query extraction tests, CozoDB graph tests, JSON graph helper tests, blast-radius contracts, key-alignment regressions, and external placeholder documentation.
That is good.

But the V2 product from Concept 31 needs a harder test shape.
It needs golden, agent-facing contracts.
The output must not merely contain some edges.
It must explain:

```text
what was selected
why it was selected
what was omitted
why it was omitted
which relationships are extracted versus inferred
which public contract might break
which confidence label Codex should trust
```

That is the difference between a graph demo and a daily Codex tool.

### The Current Test Base Is Strong But Not Strict Enough

The current test base already protects important invariants.

It knows that ISGL1 v2 keys must align between entities and edges.
It knows that dependency edges support `Calls`, `Uses`, and `Implements`.
It knows that storage can insert entities, insert edges, query forward dependencies, query reverse dependencies, compute blast radius, and compute transitive closure.
It knows that JSON graph helpers can find reverse dependencies and filter edges by type.
It knows that external dependencies and unresolved references need placeholder nodes so graph traversal does not collapse.

That is a strong foundation.

The gap is that most of the current graph-facing tests answer questions like:

```text
Did we get at least N entities?
Did we get at least N edges?
Does the helper return something?
Can storage round-trip a graph relation?
```

The V2 user journey needs a stricter question:

```text
Would Codex make the right next move from this output?
```

For public-impact and context selection, "at least N" is not enough.
If the graph returns 50 things but omits the public trait, exported route, test fixture, or implementor, it fails the job.

### First Cleanup Before New Feature Tests

Before adding V2 tests, there are two hygiene mismatches worth fixing or at least making visible.

| Mismatch | Evidence | Why It Matters |
|---|---|---|
| Fixture root mismatch | `crates/parseltongue-core/tests/fixture_harness.rs` looks under `../../test-fixtures/{}/...`, but the actual corpus lives under `tests/fixtures/`. | New T-folder tests may silently be awkward, skipped, or duplicated unless the harness root is corrected. |
| Visibility documentation mismatch | `tests/fixtures/T314-query-based-extraction-multilangs/EXPECTED.txt` says query-based extraction extracts visibility, but the inspected Rust, TypeScript, Go, and C++ entity queries capture names and kinds, not visibility/export/public modifiers. | Public-impact depends on visibility. This cannot remain implied. It must be extracted, derived, or labeled unknown. |

These are not embarrassing.
They are exactly the kind of drift a graph project accumulates.

The V2 move is to turn them into tests.

### The Test Naming Scheme

The fixture README says T-folder names follow:

```text
T{NNN}-word-word-word-word
```

Use the future band for the new product contract tests.
Do not mix this with old graph infrastructure unless the test is truly a regression.

| Fixture | Purpose | First Product Question |
|---|---|---|
| `T500-rust-public-impact-contract` | Rust public surface plus implementor/caller impact | If I change a public trait or function, what might break? |
| `T502-typescript-export-surface-impact` | TypeScript exported interfaces/functions/classes | Which exported API surface did Codex touch? |
| `T504-go-exported-symbol-impact` | Go exported symbol classification | Which capitalized Go symbols are public package API? |
| `T506-context-bundle-token-budget` | Selected and omitted context under budget | What should Codex read first, and what did it skip? |
| `T508-scope-ignore-root-report` | Index scope and ignore reporting | Did indexing accidentally include cloned reference repos? |
| `T510-confidence-label-edge-contract` | Extracted/inferred/ambiguous edge labels | Which relationships can Codex trust? |
| `T512-fixture-root-path-contract` | Harness path contract | Are fixture tests reading the canonical `tests/fixtures` corpus? |

This set is intentionally small.
It tests the product surface, not every language grammar.

### RED Test 1: Fixture Root Must Be Canonical

This should be the first failing test because every other fixture depends on it.

Proposed file:

```text
crates/parseltongue-core/tests/fixture_root_path_contract_tests.rs
```

Proposed test:

```rust
#[test]
fn test_fixture_root_path_contract() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should resolve");

    let canonical = repo_root.join("tests/fixtures");
    let legacy = repo_root.join("test-fixtures");

    assert!(canonical.exists(), "canonical tests/fixtures corpus must exist");
    assert!(
        !legacy.exists(),
        "legacy test-fixtures root should not be required for new tests"
    );

    let sample = canonical.join("T314-query-based-extraction-multilangs/EXPECTED.txt");
    assert!(sample.exists(), "fixture harness must be able to read canonical fixtures");
}
```

Expected RED:

```text
This exposes that the current helper points at ../../test-fixtures rather than ../../tests/fixtures.
```

Smallest GREEN:

```text
Change fixture_harness.rs to use ../../tests/fixtures, or make it resolve both but prefer tests/fixtures.
```

Product reason:

```text
Parceltongue V2 cannot build confidence on a fixture corpus it addresses inconsistently.
```

### RED Test 2: Rust Public Surface Must Be Explicit

Proposed fixture:

```text
tests/fixtures/T500-rust-public-impact-contract/
```

Proposed source file:

```text
tests/fixtures/T500-rust-public-impact-contract/auth_service.rs
```

Fixture source:

```rust
pub trait Authenticator {
    fn authenticate_user(&self, token: &str) -> Result<User, AuthError>;
}

pub struct JwtAuthenticator {
    secret: String,
}

impl Authenticator for JwtAuthenticator {
    fn authenticate_user(&self, token: &str) -> Result<User, AuthError> {
        normalize_token_private(token)?;
        Ok(User::default())
    }
}

pub fn login_user_public(authenticator: &dyn Authenticator, token: &str) -> Result<User, AuthError> {
    authenticator.authenticate_user(token)
}

fn normalize_token_private(token: &str) -> Result<&str, AuthError> {
    Ok(token.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_user_public() {
        let auth = JwtAuthenticator { secret: "s".to_string() };
        assert!(login_user_public(&auth, "token").is_ok());
    }
}

#[derive(Default)]
pub struct User;

pub struct AuthError;
```

Proposed test file:

```text
crates/parseltongue-core/tests/public_impact_contract_tests.rs
```

Proposed failing assertion:

```rust
#[test]
fn test_rust_public_surface_extraction() {
    let result = parse_fixture_extract_results(
        "T500-rust-public-impact-contract",
        "auth_service.rs",
        Language::Rust,
    )
    .expect("rust fixture should parse");

    let public_names: Vec<_> = result
        .entities
        .iter()
        .filter(|entity| entity.visibility == Some(Visibility::Public))
        .map(|entity| entity.name.as_str())
        .collect();

    assert!(public_names.contains(&"Authenticator"));
    assert!(public_names.contains(&"JwtAuthenticator"));
    assert!(public_names.contains(&"login_user_public"));
    assert!(!public_names.contains(&"normalize_token_private"));
}
```

Expected RED:

```text
The current query extraction result shape does not expose visibility on parsed entities.
The Rust entity query captures function, struct, enum, trait, impl, method, and module names, but not `pub`.
```

Smallest GREEN:

```text
Add a V2 extraction/fact layer that derives visibility from the source span or tree-sitter parent/modifier nodes.
Do not mutate every old test immediately.
Create EntityFacts beside the current ParsedEntity shape, then adapt old storage only after the fact contract is stable.
```

### RED Test 3: Public Impact Must Include Implementors And Callers

This is the first real PMF test.

The user journey:

```text
I am about to change `Authenticator::authenticate_user`.
Tell Codex what else it must inspect before editing.
```

Expected public-impact output:

```json
{
  "query": {
    "entity": "rust:trait_method:Authenticator.authenticate_user",
    "mode": "public-impact",
    "max_hops": 2
  },
  "public_surface": {
    "name": "Authenticator.authenticate_user",
    "kind": "trait_method",
    "visibility": "public",
    "surface_kind": "public_trait_contract",
    "confidence": "extracted"
  },
  "impact": {
    "implementors": [
      {
        "name": "JwtAuthenticator.authenticate_user",
        "edge_type": "Implements",
        "confidence": "extracted",
        "reason": "Changing the trait method contract requires checking implementors."
      }
    ],
    "callers": [
      {
        "name": "login_user_public",
        "edge_type": "Calls",
        "confidence": "extracted",
        "reason": "Caller invokes the trait method through the public login flow."
      }
    ],
    "tests": [
      {
        "name": "test_login_user_public",
        "confidence": "inferred",
        "reason": "Test is in the same fixture module and exercises the public login function."
      }
    ]
  },
  "omitted": [],
  "warnings": []
}
```

Proposed failing assertion:

```rust
#[test]
fn test_rust_public_impact_includes_implementors_and_callers() {
    let graph = build_fixture_graph_facts(
        "T500-rust-public-impact-contract",
        "auth_service.rs",
        Language::Rust,
    )
    .expect("fixture graph facts should build");

    let impact = build_public_impact_from_graph(
        &graph,
        "Authenticator.authenticate_user",
        PublicImpactOptions { max_hops: 2 },
    )
    .expect("public impact should resolve");

    assert_eq!(impact.public_surface.surface_kind, PublicSurfaceKind::PublicTraitContract);
    assert_contains_named(&impact.implementors, "JwtAuthenticator.authenticate_user");
    assert_contains_named(&impact.callers, "login_user_public");
    assert_contains_named(&impact.tests, "test_login_user_public");
    assert_no_private_helper_as_public_surface(&impact, "normalize_token_private");
}
```

Expected RED:

```text
No `build_public_impact_from_graph` helper exists.
The current dependency model has `Implements`, but public-impact semantics are not packaged as an agent-facing answer.
```

Smallest GREEN:

```text
Implement this as a pure in-memory helper over EntityFacts and EdgeFacts first.
Do not start with HTTP.
Do not start with CozoDB migration.
Do not start with CLI polish.
Make the product answer correct in memory, then wire it outward.
```

### RED Test 4: TypeScript Export Surface Must Not Be Guesswork

Proposed fixture:

```text
tests/fixtures/T502-typescript-export-surface-impact/session.ts
```

Fixture source:

```typescript
export interface UserSession {
  userId: string;
  expiresAt: Date;
}

export function refreshToken(session: UserSession): UserSession {
  return normalizeSessionPrivate(session);
}

function normalizeSessionPrivate(session: UserSession): UserSession {
  return session;
}

export class SessionStore {
  saveSession(session: UserSession): void {
    refreshToken(session);
  }
}
```

Proposed failing assertion:

```rust
#[test]
fn test_typescript_export_surface_extraction() {
    let facts = extract_fixture_entity_facts(
        "T502-typescript-export-surface-impact",
        "session.ts",
        Language::TypeScript,
    )
    .expect("typescript fixture should parse");

    assert_public_surface(&facts, "UserSession", PublicSurfaceKind::ExportedInterface);
    assert_public_surface(&facts, "refreshToken", PublicSurfaceKind::ExportedFunction);
    assert_public_surface(&facts, "SessionStore", PublicSurfaceKind::ExportedClass);
    assert_private_or_internal(&facts, "normalizeSessionPrivate");
}
```

Expected RED:

```text
The TypeScript entity query captures interfaces, functions, classes, methods, aliases, enums, and arrows, but export visibility is not part of the current entity contract.
```

Smallest GREEN:

```text
For TypeScript and JavaScript, detect `export_statement` and `export` modifiers around captured declarations.
If the parser cannot prove export status, label it `unknown`, not `private`.
```

### RED Test 5: Go Public Surface Must Follow Language Convention

Proposed fixture:

```text
tests/fixtures/T504-go-exported-symbol-impact/auth.go
```

Fixture source:

```go
package auth

type Authenticator interface {
    AuthenticateUser(token string) (User, error)
}

type JWTAuthenticator struct {
    secret string
}

func (j JWTAuthenticator) AuthenticateUser(token string) (User, error) {
    return User{}, nil
}

func LoginUser(authenticator Authenticator, token string) (User, error) {
    return authenticator.AuthenticateUser(token)
}

func normalizeTokenPrivate(token string) string {
    return token
}

type User struct{}
```

Proposed failing assertion:

```rust
#[test]
fn test_go_exported_symbol_surface() {
    let facts = extract_fixture_entity_facts(
        "T504-go-exported-symbol-impact",
        "auth.go",
        Language::Go,
    )
    .expect("go fixture should parse");

    assert_public_surface(&facts, "Authenticator", PublicSurfaceKind::ExportedInterface);
    assert_public_surface(&facts, "JWTAuthenticator", PublicSurfaceKind::ExportedType);
    assert_public_surface(&facts, "AuthenticateUser", PublicSurfaceKind::ExportedMethod);
    assert_public_surface(&facts, "LoginUser", PublicSurfaceKind::ExportedFunction);
    assert_private_or_internal(&facts, "normalizeTokenPrivate");
}
```

Expected RED:

```text
The Go query captures functions, methods, structs, and interfaces, but public/exported status is currently not promoted into the graph answer.
```

Smallest GREEN:

```text
For Go, classify exported package surface by first-rune uppercase rule after entity extraction.
This does not need a new tree-sitter query at first.
```

### RED Test 6: Context Bundle Must Explain Selection And Omission

Current smart context is directionally right.
It uses graph relevance and a budget.
But it returns a list of selected entities with a heuristic token estimate.

The V2 command needs to answer the agent's real question:

```text
What should I read next, why, and what did you leave out because of the budget?
```

Proposed fixture:

```text
tests/fixtures/T506-context-bundle-token-budget/
```

Use a small synthetic graph where:

```text
login_user_public
  calls Authenticator.authenticate_user
  calls audit_login_private

Authenticator.authenticate_user
  implemented by JwtAuthenticator.authenticate_user

JwtAuthenticator.authenticate_user
  calls normalize_token_private

normalize_token_private
  low public impact, private helper
```

Expected context output:

```json
{
  "query": {
    "entity": "login_user_public",
    "budget_tokens": 120
  },
  "selected": [
    {
      "name": "login_user_public",
      "reason": "focus entity",
      "relationship": "self",
      "estimated_tokens": 35,
      "confidence": "extracted"
    },
    {
      "name": "Authenticator.authenticate_user",
      "reason": "public trait contract called by focus entity",
      "relationship": "callee",
      "estimated_tokens": 45,
      "confidence": "extracted"
    },
    {
      "name": "JwtAuthenticator.authenticate_user",
      "reason": "implementor of public trait method",
      "relationship": "implementor",
      "estimated_tokens": 38,
      "confidence": "extracted"
    }
  ],
  "omitted": [
    {
      "name": "normalize_token_private",
      "reason": "budget exhausted after higher-impact public contract context",
      "relationship": "transitive_private_helper",
      "estimated_tokens": 22,
      "confidence": "extracted"
    }
  ],
  "budget": {
    "requested_tokens": 120,
    "selected_tokens": 118,
    "omitted_tokens": 22
  }
}
```

Proposed failing assertion:

```rust
#[test]
fn test_context_bundle_explains_selected_and_omitted() {
    let graph = load_fixture_graph_facts("T506-context-bundle-token-budget")
        .expect("graph fixture should load");

    let bundle = build_context_bundle_from_graph(
        &graph,
        "login_user_public",
        ContextBundleOptions { budget_tokens: 120 },
    )
    .expect("context bundle should build");

    assert_selected_reason(&bundle, "login_user_public", "focus entity");
    assert_selected_relationship(&bundle, "Authenticator.authenticate_user", "callee");
    assert_selected_relationship(&bundle, "JwtAuthenticator.authenticate_user", "implementor");
    assert_omitted_reason_contains(&bundle, "normalize_token_private", "budget exhausted");
    assert!(bundle.budget.selected_tokens <= 120);
}
```

Expected RED:

```text
No V2 context bundle helper currently exists.
The existing smart-context endpoint has a useful relevance model but not the selected/omitted/reason/confidence contract.
```

Smallest GREEN:

```text
Port the current relevance scoring into a pure helper.
Replace key-length token estimates with a source-backed estimate when source text is available.
Return selected and omitted lists.
Do not hide omitted high-relevance nodes.
```

### RED Test 7: Scope Report Must Expose Accidental Clone Forests

This comes directly from the CodeGraphContext evidence-reader run.

I attempted to run the CGC wrapper against:

```text
crates/parseltongue-core
```

The wrapper resolved the path back to the repo root and began indexing:

```text
/Users/amuldotexe/Desktop/personal-repos-lane/parseltongue-rust-LLM-companion
```

I stopped it because this repo contains a large `git-ref-repo/` reference clone forest and the skill warns against indexing reference forests unless explicitly requested.
The run left a generated `.cgcignore`; I removed it.

This is not only a CGC issue.
It is a product requirement for Parceltongue.

The agent needs a command that says:

```text
Here is exactly what I will index.
Here is exactly what I will ignore.
Here are roots that look dangerous.
```

Proposed fixture:

```text
tests/fixtures/T508-scope-ignore-root-report/
```

Fixture tree:

```text
T508-scope-ignore-root-report/
  src/lib.rs
  git-ref-repo/ignore-this-folder-repos/fake_repo/src/lib.rs
  node_modules/fake/index.js
  target/debug/fake
```

Expected scope report:

```json
{
  "requested_root": "T508-scope-ignore-root-report",
  "effective_root": "T508-scope-ignore-root-report",
  "indexed_roots": [
    "src"
  ],
  "ignored_roots": [
    {
      "path": "git-ref-repo",
      "reason": "reference repository folder"
    },
    {
      "path": "node_modules",
      "reason": "dependency folder"
    },
    {
      "path": "target",
      "reason": "build artifact folder"
    }
  ],
  "warnings": [
    {
      "kind": "reference_clone_forest",
      "path": "git-ref-repo",
      "message": "Reference clones are ignored unless explicitly included."
    }
  ]
}
```

Proposed failing assertion:

```rust
#[test]
fn test_scope_report_excludes_reference_clone_forest() {
    let report = build_index_scope_report(
        fixture_path("T508-scope-ignore-root-report"),
        ScopeOptions::default(),
    )
    .expect("scope report should build");

    assert_indexed_root(&report, "src");
    assert_ignored_root(&report, "git-ref-repo");
    assert_ignored_root(&report, "node_modules");
    assert_ignored_root(&report, "target");
    assert_warning_kind(&report, "reference_clone_forest");
}
```

Expected RED:

```text
No first-class scope-report helper exists for the V2 command shape.
Current ingestion has exclude support, but the agent-facing explanation is not the product surface yet.
```

Smallest GREEN:

```text
Create a pure scope-report builder.
Default-ignore `.git`, `target`, `node_modules`, `git-ref-repo`, `gitrefrepo`, and `ignore-this-folder-repos`.
Expose the explanation as JSON before indexing.
```

### RED Test 8: Edge Confidence Labels Must Be Required

The current `DependencyEdge` has:

```text
from_key
to_key
edge_type
source_location
```

That is enough to traverse.
It is not enough to guide a coding agent.

For Codex, there is a big difference between:

```text
This call was extracted from an actual call expression.
```

and:

```text
This target is unresolved and might be local, external, trait dispatch, macro expansion, generic instantiation, or dynamic dispatch.
```

Both can be useful.
They should not have the same confidence.

Proposed fixture:

```text
tests/fixtures/T510-confidence-label-edge-contract/
```

Expected EdgeFacts JSON:

```json
{
  "edges": [
    {
      "from": "login_user_public",
      "to": "Authenticator.authenticate_user",
      "edge_type": "Calls",
      "evidence_kind": "call_expression",
      "resolver_kind": "tree_sitter_exact",
      "confidence": "extracted"
    },
    {
      "from": "login_user_public",
      "to": "audit_login_private",
      "edge_type": "Calls",
      "evidence_kind": "identifier_call",
      "resolver_kind": "same_file_name_match",
      "confidence": "inferred"
    },
    {
      "from": "JwtAuthenticator.authenticate_user",
      "to": "external-dependency-anyhow",
      "edge_type": "Uses",
      "evidence_kind": "use_declaration",
      "resolver_kind": "external_placeholder",
      "confidence": "external"
    },
    {
      "from": "login_user_public",
      "to": "unresolved-reference",
      "edge_type": "Calls",
      "evidence_kind": "call_expression",
      "resolver_kind": "unresolved",
      "confidence": "unresolved"
    }
  ]
}
```

Proposed failing assertion:

```rust
#[test]
fn test_edge_facts_require_confidence_labels() {
    let graph = build_fixture_graph_facts(
        "T510-confidence-label-edge-contract",
        "auth_service.rs",
        Language::Rust,
    )
    .expect("graph facts should build");

    assert_edge_confidence(&graph, "login_user_public", "Authenticator.authenticate_user", EdgeConfidence::Extracted);
    assert_edge_confidence(&graph, "login_user_public", "audit_login_private", EdgeConfidence::Inferred);
    assert_any_edge_confidence(&graph, EdgeConfidence::External);
    assert_any_edge_confidence(&graph, EdgeConfidence::Unresolved);
}
```

Expected RED:

```text
Current edge structs do not require confidence labels.
External placeholder docs distinguish external dependency and unresolved reference, but that distinction is not yet a general agent-facing EdgeFacts contract.
```

Smallest GREEN:

```text
Add EdgeFacts as a V2 adapter type first.
Do not break old DependencyEdge storage yet.
Convert DependencyEdge plus extraction metadata into EdgeFacts at query time.
```

### The Smallest Implementation Path

This is the implementation order I would actually use.

| Step | Implement | Why This Step |
|---|---|---|
| 1 | Fix or dual-resolve fixture root | Makes the new RED tests stable. |
| 2 | Add `EntityFacts`, `EdgeFacts`, `ContextBundleFacts`, and `PublicImpactFacts` as pure structs | Avoids storage migration before semantics are known. |
| 3 | Add Rust public-surface classification | Rust is the user's systems-programming priority and the repo's strongest language. |
| 4 | Add TypeScript export classification | Covers CRUD app work and API surface changes. |
| 5 | Add Go exported symbol classification | Cheap win because public status is mostly name convention. |
| 6 | Build in-memory `public-impact` helper | Proves the product answer before HTTP/CLI. |
| 7 | Build in-memory `context-bundle` helper | Upgrades smart context into selected/omitted/reason output. |
| 8 | Build `scope-report` helper | Prevents accidental polluted indexes. |
| 9 | Wire helpers into CLI commands | Only after pure contracts pass. |
| 10 | Wire HTTP endpoints if still useful | Codex app may prefer CLI; HTTP is secondary for solo use. |

The key principle:

```text
Start with pure functions over small fixture graphs.
Only then connect them to ingestion, CozoDB, HTTP, and CLI.
```

This keeps the first PR small enough to finish.

### The Minimum V2 Data Types

Do not overbuild.
The first structs can be boring.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityFacts {
    pub key: String,
    pub name: String,
    pub language: Language,
    pub entity_type: EntityType,
    pub visibility: VisibilityFact,
    pub public_surface_kind: Option<PublicSurfaceKind>,
    pub file_path: String,
    pub line_range: LineRange,
    pub signature_text: Option<String>,
    pub source_text: Option<String>,
    pub estimated_tokens: usize,
    pub confidence: FactConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeFacts {
    pub from_key: String,
    pub to_key: String,
    pub edge_type: EdgeType,
    pub source_location: Option<String>,
    pub evidence_kind: EvidenceKind,
    pub resolver_kind: ResolverKind,
    pub confidence: FactConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactConfidence {
    Extracted,
    Inferred,
    Ambiguous,
    External,
    Unresolved,
}
```

The important thing is not the exact names.
The important thing is that the graph can stop pretending all relationships are equally true.

### What To Avoid

Do not start V2 by adding another endpoint.

That is tempting because the current product already has many HTTP routes.
But an endpoint is just transport.
The missing thing is a trustworthy answer contract.

Do not start by tuning CozoDB queries.

The storage layer already has real graph tests.
Public-impact semantics are not primarily a storage problem yet.
They are a classification and packaging problem.

Do not start with ten languages.

The user journey is:

```text
Solo agent power user.
All languages eventually.
CRUD apps plus Rust, C, C++ systems programming.
Help Codex navigate large codebases faster and more reliably.
```

So the first proof should cover:

```text
Rust
TypeScript
Go
```

Then add:

```text
C
C++
Python
Java
C#
SQL
```

after the product contract is stable.

### The Shreyas Product Read

The product risk is not that Parceltongue lacks enough graph features.
The product risk is that Codex will not know when to trust the graph.

So the next tests should optimize for trust:

```text
exact outputs
named reasons
omitted context
confidence labels
public contract classification
scope transparency
```

This is Shreyas-style PMF thinking:

```text
The user is not buying graph completeness.
The user is buying fewer agent mistakes while editing large codebases.
```

The first delight moment should be:

```text
Codex asks Parceltongue for public impact,
gets a tiny answer,
and immediately reads the right trait, implementor, caller, and test.
```

That is the test to make pass.

### Why CodeGraphContext Matters Here

Using the CodeGraphContext evidence-reader changed the plan in one specific way.

It showed that a powerful graph tool can still be risky if scope is implicit.
The attempted CGC run on a crate path resolved back to the repository root and began walking from there.
Because this repository contains a reference clone forest, that was enough reason to stop the run.

That behavior turns into a Parceltongue requirement:

```text
Every index command must have an explainable scope.
Every context result must say what corpus it came from.
Every public-impact answer must be reproducible from visible indexed roots.
```

So `T508-scope-ignore-root-report` is not a side quest.
It is core product trust.

### Acceptance Criteria For Concept 32

The next implementation should not be considered real until these are true:

| Acceptance Criterion | How To Verify |
|---|---|
| New fixture root test passes | `cargo test -p parseltongue-core --test fixture_root_path_contract_tests` |
| Rust public-surface facts exist | `cargo test -p parseltongue-core --test public_impact_contract_tests test_rust_public_surface_extraction` |
| Rust public impact includes implementors, callers, and tests | `cargo test -p parseltongue-core --test public_impact_contract_tests test_rust_public_impact_includes_implementors_and_callers` |
| TypeScript exports are classified | `cargo test -p parseltongue-core --test public_impact_contract_tests test_typescript_export_surface_extraction` |
| Go exported symbols are classified | `cargo test -p parseltongue-core --test public_impact_contract_tests test_go_exported_symbol_surface` |
| Context bundle reports selected and omitted entities | `cargo test -p parseltongue-core --test context_bundle_contract_tests` |
| Scope report ignores reference clone forests | `cargo test -p parseltongue-core --test scope_report_contract_tests` |
| Edge confidence labels exist | `cargo test -p parseltongue-core --test edge_confidence_contract_tests` |

If these pass, then CLI wiring becomes straightforward.

### The Next Commit Shape

If implementing this, I would use small commits:

| Commit | Content |
|---|---|
| 1 | Fix fixture root harness and add `T512-fixture-root-path-contract`. |
| 2 | Add V2 fact structs with no behavior. |
| 3 | Add Rust public-surface extraction RED/GREEN. |
| 4 | Add TypeScript and Go public-surface extraction RED/GREEN. |
| 5 | Add public-impact pure helper and golden JSON. |
| 6 | Add context-bundle pure helper and selected/omitted contract. |
| 7 | Add scope-report helper and default ignore report. |
| 8 | Add edge confidence labels and adapter from current edges. |
| 9 | Add CLI commands after all pure helpers pass. |

The first commit should be tiny.
It should prove that the fixture foundation is reliable.

### Repos And Docs Touched For This Concept

| Source | Evidence | Notes |
|---|---|---|
| `/Users/amuldotexe/.codex/skills/codegraphcontext-evidence-reader/SKILL.md` | lines 1-72 | Skill says CGC is evidence-gathering, important graph findings need direct source reads, wrapper is preferred, and reference repo forests should not be indexed unless explicitly requested. |
| `/tmp/codex-code-intel/codegraphcontext/parseltongue-rust-LLM-companion-20260706-233111/index.txt` | lines 1-62 | Aborted CGC run showed the wrapper resolving to the repo root and entering discovery before manual stop, which motivates explicit scope-report testing. |
| `crates/parseltongue-core/tests/fixture_harness.rs` | lines 1-44 | Harness currently resolves fixture source files under `../../test-fixtures/{}/...`. |
| `tests/fixtures/README.md` | lines 1-128 | Canonical fixture corpus, T-folder naming scheme, category bands, expected folder contents, and instructions for adding tests. |
| `crates/parseltongue-core/tests/query_based_extraction_test.rs` | lines 1-144 | Inline query-based extraction tests for Rust, Python, C, JavaScript, and malformed code. |
| `crates/parseltongue-core/tests/rust_dependency_patterns_test.rs` | lines 1-340 | Rust dependency pattern tests with count-style assertions and printed edge diagnostics. |
| `crates/parseltongue-core/tests/t_rust_edge_tests.rs` | lines 1-340 | T-folder style Rust dependency tests using `parse_fixture_extract_results`. |
| `crates/parseltongue-core/tests/cozo_storage_integration_tests.rs` | lines 1-360 | Storage, dependency edge, forward/reverse dependency, blast radius, transitive closure, and performance tests. |
| `crates/parseltongue-core/tests/query_json_graph_contract_tests.rs` | lines 1-360 | Agent-friendly JSON graph query helper tests with reverse deps, call chains, edge filtering, file collection, and error handling. |
| `crates/parseltongue-core/src/query_json_graph_helpers.rs` | lines 1-104 | Existing pure JSON graph helpers for reverse dependencies, call chains, edge type filtering, and file-path collection. |
| `crates/parseltongue-core/src/query_json_graph_errors.rs` | lines 1-29 | Existing helper error types: entity not found, malformed JSON, invalid edge type. |
| `tests/fixtures/T315-json-graph-contract-queries/EXPECTED.txt` | lines 1-155 | Existing JSON graph helper contract and limitations around pre-generated JSON, cycles, performance, and reverse deps. |
| `tests/fixtures/T314-query-based-extraction-multilangs/EXPECTED.txt` | lines 1-116 | Query extraction fixture contract and visibility claim that should be reconciled with actual query packs. |
| `crates/parseltongue-core/src/entities.rs` | lines 1030-1225 | Current edge model has `EdgeType::{Calls, Uses, Implements}` and `DependencyEdge` with keys, edge type, and optional source location. |
| `tests/fixtures/T244-blast-radius-key-alignment/EXPECTED.txt` | lines 1-152 | Regression contract for key alignment between entities and edges, required for graph traversal trust. |
| `tests/fixtures/T311-external-dependency-placeholder/EXPECTED.txt` | lines 1-102 | External dependency and unresolved-reference placeholder contract, including why orphaned edges break blast-radius queries. |
| `tests/fixtures/T307-cozo-storage-crud-operations/EXPECTED.txt` | lines 1-262 | Storage CRUD, graph query, blast-radius, transitive closure, and performance contract summary. |
| `tests/fixtures/T308-blast-radius-transitive-closure/EXPECTED.txt` | lines 1-255 | Focused blast-radius and transitive-closure semantics, edge cases, and performance notes. |

## Concept 33: Generate Typed Tree-Sitter Facts Before Writing Query Logic

The next pattern is not another graph algorithm.

It is a discipline pattern:

```text
Do not let every extractor hand-roll stringly Tree-sitter access.
Generate typed accessors from the grammar contract first.
Then write public-impact and context logic against typed facts.
```

This matters because Concept 32 identified the exact thing Parceltongue V2 needs next:

```text
visibility
public surface kind
edge confidence
source-backed spans
selected and omitted context
```

Those facts are fragile if every language extractor does this:

```rust
node.child_by_field_name("name")
node.child_by_field_name("body")
node.kind() == "function_item"
capture_name == "definition.function"
```

That works until it does not.

One typo in a field name.
One grammar version that renames a node.
One capture whose dot/underscore normalization is inconsistent.
One language where "public" is not a modifier node but a convention.

Then Codex gets a confident-looking graph answer that missed the exact public contract it needed.

The typed-wrapper repos show the alternative.

Use `node-types.json` and query files as schemas.
Generate a typed layer.
Make the extractor code depend on that typed layer.
Make grammar drift fail loudly.

That is the core lesson from:

```text
Jakobeha/type-sitter
JoranHonig/python-tree-sitter-types
```

### Codebase-Memory Evidence Used

I used the required `codebase-memory-evidence-reader` on both repos before writing this concept.

| Repo | Codebase-Memory Result | Useful Signal |
|---|---|---|
| `Jakobeha__type-sitter` | Indexed cleanly: 17,473 nodes and 58,325 edges. | Graph search found exported `generate_nodes`, `generate_queries`, `_generate_queries_from_dir`, `_generate_query_from_file`, and related codegen functions. |
| `JoranHonig__python-tree-sitter-types` | Indexed cleanly: 71 nodes and 125 edges. | Small repo; direct source reads were more useful than graph search, but the run confirmed scope isolation and no clone-forest leakage. |

One tool note:

```text
codebase-memory-mcp search_graph needs `project`, not `project_name`.
```

The wrapper examples are close, but the explicit CLI call only worked after using the `project` key.
That is another small argument for Parceltongue's own command contracts to be boringly exact.

### What Type-Sitter Does That Parceltongue Should Steal

`type-sitter` starts from two authoritative Tree-sitter artifacts:

```text
src/node-types.json
queries/*.scm
```

Then it generates typed Rust wrappers.

The README states the product idea plainly:

```text
node-types.json -> typed node wrappers
query s-expressions -> typed query wrappers
```

The useful part for Parceltongue is not that we need to adopt `type-sitter` wholesale.
The useful part is the shape of the generated layer.

It turns raw Tree-sitter primitives into these higher-level commitments:

| Raw Tree-Sitter Habit | Generated Typed Habit | Why It Matters For Parceltongue |
|---|---|---|
| `node.kind() == "function_item"` | `FunctionItem<'tree>` type | Extractor logic can compile against known node kinds. |
| `child_by_field_name("name")` | `.name()` method | Field names become generated API, not scattered strings. |
| `child_by_field_name("body")` | `.body()` method returning typed child | Public-surface extraction can require the exact child shape. |
| `capture("function.name")` | typed capture accessor | Query capture names become schema, not magic strings. |
| Runtime node-kind switch everywhere | Supertype enums | Exhaustive handling of variants becomes possible. |
| Comments ignored unless manually traversed | Extra-node support | Documentation adjacency and doc-comment extraction become first-class. |
| Hidden grammar nodes remain anonymous | Custom supertypes | Parceltongue can define product-level groupings when grammar authors did not. |

The killer phrase:

```text
The grammar becomes an API.
```

That is what Parceltongue needs.

### Pattern: Product Types For Nodes, Sum Types For Supertypes

In `type-sitter-gen/src/node_types/print.rs`, regular node types print as transparent wrapper structs around `tree_sitter::Node`.

The generated struct has the important invariant:

```rust
fn try_from_raw(node) -> NodeResult<Self> {
    if node.kind() == KIND {
        Ok(Self(node))
    } else {
        Err(IncorrectKind::new::<Self>(node))
    }
}
```

That is a small thing with a large effect.

It converts:

```text
"I think this node is a Rust function"
```

into:

```text
"This node proved it is a Rust function, or extraction failed at the boundary."
```

For supertypes, `type-sitter` generates enums.
That means something like a declaration/expression/type family can become:

```rust
enum Declaration<'tree> {
    FunctionItem(FunctionItem<'tree>),
    StructItem(StructItem<'tree>),
    TraitItem(TraitItem<'tree>),
}
```

The exact generated enum varies by grammar, but the pattern matters.
Parceltongue's current extraction model should move toward:

```rust
enum RustPublicSurfaceNode<'tree> {
    FunctionItem(FunctionItem<'tree>),
    StructItem(StructItem<'tree>),
    EnumItem(EnumItem<'tree>),
    TraitItem(TraitItem<'tree>),
    ModItem(ModItem<'tree>),
    UseDeclaration(UseDeclaration<'tree>),
}
```

Then public-surface extraction becomes ordinary Rust pattern matching instead of a pile of capture strings.

### Pattern: Field Accessors Are Fact Boundaries

`type-sitter` generates methods from `node-types.json` fields.

That is exactly what Parceltongue needs for public-impact:

| Public-Impact Need | Tree-Sitter Field/API Need |
|---|---|
| Function name | generated `.name()` accessor |
| Trait name | generated `.name()` accessor |
| Method receiver | generated receiver/self/type accessor |
| Visibility modifier | generated child/field accessor or language-specific rule |
| Type parameters | generated type-parameter accessor |
| Return type | generated return-type accessor |
| Body calls | generated body/block accessor |
| Attribute/decorator/annotation | generated modifier/attribute/decorator accessor |

The pattern is not "generate everything and use everything."
The pattern is:

```text
Generate the shape.
Use a small product-level subset.
Fail tests when the grammar shape changes.
```

For Parceltongue V2, I would generate or hand-author a small facade per language:

```rust
trait PublicSurfaceExtractor<'tree> {
    type Function;
    type TypeDecl;
    type TraitDecl;
    type ImportDecl;

    fn function_name(node: Self::Function) -> Option<SourceTextSpan>;
    fn function_visibility(node: Self::Function) -> VisibilityFact;
    fn function_signature(node: Self::Function) -> SignatureFacts;
    fn function_body(node: Self::Function) -> Option<BodyNode<'tree>>;
}
```

The implementation can be generated from grammar facts where possible.
When it is not possible, the rule must be explicit:

```text
VisibilityFact::Inferred("go_exported_identifier")
VisibilityFact::Unknown("grammar_no_modifier_capture")
VisibilityFact::Extracted("rust_visibility_modifier")
```

That one distinction prevents a lot of agent overconfidence.

### Pattern: Typed Queries Should Wrap Captures, Not Just Matches

`type-sitter-lib/src/query/mod.rs` defines a typed query trait with associated `Match` and `Capture` types.

The key contract is:

```rust
trait Query {
    type Match<'query, 'tree>;
    type Capture<'query, 'tree>;
}
```

And `captures.rs` wraps raw captures with:

```text
typed_query
untyped_captures
```

Each typed capture still exposes the raw node and capture index, but it also knows:

```text
which query it came from
which capture name it has
which typed node it should wrap
```

This is directly useful for Parceltongue's V2 edge confidence labels.

Current query extraction can produce:

```text
Calls
Uses
Implements
```

But Concept 32 says the agent needs:

```text
edge_type
evidence_kind
resolver_kind
confidence
```

A typed query capture layer can make that natural:

```rust
enum RustDependencyCapture<'tree> {
    FunctionCall {
        call_node: CallExpression<'tree>,
        callee: IdentifierLike<'tree>,
    },
    UseDeclaration {
        use_node: UseDeclaration<'tree>,
        path: UsePath<'tree>,
    },
    TraitImplementation {
        impl_node: ImplItem<'tree>,
        trait_path: TypePath<'tree>,
        for_type: TypeIdentifier<'tree>,
    },
}
```

Then the edge builder can say:

```rust
match capture {
    RustDependencyCapture::FunctionCall { .. } => {
        confidence = Extracted;
        evidence_kind = CallExpression;
    }
    RustDependencyCapture::UseDeclaration { .. } => {
        confidence = Extracted;
        evidence_kind = UseDeclaration;
    }
    RustDependencyCapture::TraitImplementation { .. } => {
        confidence = Extracted;
        evidence_kind = TraitImpl;
    }
}
```

That is much safer than making every handler remember what capture names mean.

### Pattern: Query Directories Become Modules

`type-sitter-gen/src/queries/mod.rs` supports both:

```text
single .scm file
directory of .scm files
```

When it receives a directory, it reads entries, sorts them, recursively processes folders and `.scm` files, and emits submodules for nested directories.

That is a language-pack architecture pattern.

Parceltongue should treat each language like:

```text
language_pack/
  node-types.json
  queries/
    entities.scm
    dependencies.scm
    public_surface.scm
    tests.scm
    docs.scm
  generated/
    nodes.rs
    queries.rs
  manifest.toml
```

Then generated query modules become a stable internal API:

```rust
rust_pack::queries::entities::EntityQuery
rust_pack::queries::dependencies::DependencyQuery
rust_pack::queries::public_surface::PublicSurfaceQuery
```

The manifest should record:

```text
parser crate/version
node-types hash
query file hashes
generated-at tool version
supported public-surface facts
known unsupported facts
```

That would have caught the mismatch from Concept 32:

```text
Fixture says visibility is extracted.
Actual query pack does not expose visibility.
```

### Pattern: Custom Supertypes Are Product Semantics

`type-sitter` supports custom supertypes.

The README example creates a supertype for "all named" nodes and another for hidden class-member variants.

That is not just type-system cleverness.
It is product design.

Tree-sitter grammars are written for parsing.
Parceltongue needs grammars shaped for agent tasks.

Those are not always the same ontology.

Parceltongue should define product-level supertypes like:

```text
PublicSurfaceDeclaration
CallableDeclaration
TypeDeclaration
ImportDeclaration
TestDeclaration
RouteDeclaration
SchemaDeclaration
DocComment
```

For Rust:

```text
PublicSurfaceDeclaration =
  function_item with pub visibility
  struct_item with pub visibility
  enum_item with pub visibility
  trait_item with pub visibility
  mod_item with pub visibility
  use_declaration with pub visibility
```

For TypeScript:

```text
PublicSurfaceDeclaration =
  exported function_declaration
  exported class_declaration
  exported interface_declaration
  exported type_alias_declaration
  export_statement wrapping declaration
```

For Go:

```text
PublicSurfaceDeclaration =
  package-scope declaration with exported identifier
  method_declaration with exported method name
  interface method with exported name
```

The point:

```text
Do not let raw grammar supertypes be the product ontology.
Generate product supertypes from grammar facts plus language rules.
```

### Pattern: Generated Code Must Be Regeneration-Safe

`type-sitter-cli` has a practical safety detail.

When processing a whole language root, it removes the previous output directory only if that directory contains all Rust files.

That protects users from accidentally deleting hand-written code.

Parceltongue should copy this spirit exactly.

If we generate language-pack code, the output folder should be:

```text
crates/parseltongue-core/src/generated_language_packs/rust/
```

and regeneration should require:

```text
generated marker present
manifest hash matches
only generated files inside
no user-authored files mixed in
```

Proposed guard:

```rust
fn assert_codegen_dir_regenerable(path: &Path) -> Result<()> {
    ensure!(path.join(".parseltongue-generated").exists());
    ensure!(dir_contains_only_generated_files(path)?);
    Ok(())
}
```

This is mundane.
It also prevents expensive pain.

### What Python-Tree-Sitter-Types Adds

The Python repo is smaller and looser, but it contributes a useful second version of the idea.

It parses `node-types.json` into Pydantic models:

```text
TypeSpecification
ChildrenSpecification
FieldSpecification
NodeType
NodeTypeList
```

Then the generator builds Python classes with:

```text
field_names
annotated fields
children annotation
type_name_to_class mapping
TreeSitterNode base class
```

The runtime parser recursively maps raw Tree-sitter nodes into those generated classes:

```text
node.type -> constructor
field_names -> children_by_field_name
named_children not already used by fields -> children
base_node retained for text/source access
```

This is less statically safe than Rust.
But it has a useful architectural pattern:

```text
Keep raw node access available as `base_node`.
Build typed convenience around it.
```

Parceltongue should do the same.

Even if we generate `RustFunctionItem` or `TypeScriptExportedFunction`, every fact should still retain:

```text
raw node kind
byte range
point range
source file
query capture name
parser version
```

Because sometimes Codex needs to inspect the exact span.
Typed facts should not hide raw evidence.
They should make it safer to get to the right raw evidence.

### The Python Caveats Are Also Design Warnings

`python-tree-sitter-types` has TODOs around:

```text
literals
field names colliding with children/field_names/base_node
clean field initialization
```

That is very relevant to Parceltongue.

If Parceltongue generates fact-accessor code, it must explicitly handle reserved names:

```text
type
match
self
children
field_names
base_node
from
else
crate
super
mod
```

`type-sitter` has a much more detailed naming strategy:

```text
illegal character substitutions
raw identifiers when possible
reserved-word fallback
deterministic disambiguation
module separation for unnamed/symbol nodes
capture-name dot-to-underscore conversion
```

For Parceltongue, deterministic naming is not aesthetic.
It is graph identity hygiene.

If one grammar update renames:

```text
method.definition
```

to:

```text
method_definition
```

or if both collapse to the same Rust identifier, the generated layer should surface that conflict.
Do not silently overwrite a capture.

### Parceltongue V2 Pattern: Generate Fact Extractors, Not Whole AST Bindings

A full `type-sitter` style binding for every Parceltongue language might be too heavy.

The README notes the generated wrappers for Rust can be tens of thousands of lines.
That is too much to casually add to a lean code-assist core.

But Parceltongue does not need every AST node as a public API.

It needs a generated fact layer for the agent-relevant subset:

```text
entities
dependencies
public surface
tests
docs/comments
routes
schemas
imports/exports
```

So the better Parceltongue design is:

```text
node-types.json
queries/*.scm
language rules
    -> generate small FactExtractor APIs
    -> run extractor
    -> emit EntityFacts and EdgeFacts
```

Not:

```text
generate a complete Rust API for every grammar node and expose it everywhere
```

The generated output should be narrow:

```rust
pub struct RustPublicSurfaceFacts<'tree> {
    pub name: SourceSpan<'tree>,
    pub visibility: VisibilityFact,
    pub signature: SignatureFacts<'tree>,
    pub doc_comment: Option<SourceSpan<'tree>>,
    pub raw_kind: &'static str,
}
```

And the generator should be asked:

```text
Can this language pack produce RustPublicSurfaceFacts?
If not, which fields are missing?
```

That fits the V2 trust model.

### Proposed Parceltongue Architecture

```text
language_packs/
  rust/
    manifest.toml
    node-types.json
    queries/
      entities.scm
      dependencies.scm
      public_surface.scm
    product_supertypes.toml
    generated/
      fact_accessors.rs
      query_captures.rs
      manifest_hash.rs

runtime/
  parser loads language pack
  query runner emits typed captures
  fact builder converts typed captures to EntityFacts and EdgeFacts
  context/public-impact operate only on EntityFacts/EdgeFacts
```

The important boundary:

```text
Tree-sitter parser specifics stop at the language-pack adapter.
Agent product logic starts at EntityFacts and EdgeFacts.
```

This means `public-impact` should never care whether Rust uses `visibility_modifier`, TypeScript uses `export_statement`, or Go uses uppercase names.
It should care only about:

```rust
VisibilityFact::Public { confidence, evidence }
PublicSurfaceKind::ExportedFunction
```

### Tests Parceltongue Should Add

Concept 32 introduced T500-series product tests.
This concept adds generated-layer tests.

| Fixture | Purpose |
|---|---|
| `T514-generated-capture-contract` | Capture names in `.scm` produce deterministic generated enum variants. |
| `T516-node-type-drift-detection` | `node-types.json` hash mismatch fails with a clear language-pack error. |
| `T518-reserved-name-normalization` | Fields/captures named `type`, `match`, `from`, `else`, `self`, and `children` normalize deterministically. |
| `T520-product-supertype-contract` | `PublicSurfaceDeclaration` includes expected Rust/TypeScript/Go variants and rejects private helpers. |
| `T522-raw-evidence-retention` | Every generated fact retains raw kind, byte range, point range, file path, capture name, and parser/query hashes. |

The most important test is `T522`.

If a generated fact cannot point back to source evidence, it is not acceptable for agent use.

### Minimal Implementation Path

Do not start with a generator that handles every language.

Start with one Rust prototype:

| Step | Build | Why |
|---|---|---|
| 1 | Read Rust `node-types.json` and current `entity_queries/rust.scm` / `dependency_queries/rust.scm`. | Establish schema inputs. |
| 2 | Generate a small capture enum only for current captures. | Converts magic capture strings into typed variants. |
| 3 | Generate a small accessor facade for `function_item`, `struct_item`, `enum_item`, `trait_item`, `impl_item`, `use_declaration`, `mod_item`. | Covers public-surface MVP. |
| 4 | Add manifest with node/query hashes. | Makes drift detectable. |
| 5 | Convert capture enum into `EntityFacts` and `EdgeFacts`. | Keeps graph product logic independent. |
| 6 | Wire only Rust into `public-impact` tests from Concept 32. | Proves end-to-end value before more languages. |
| 7 | Add TypeScript export facade. | Covers CRUD/web work. |
| 8 | Add Go exported-name facade. | Cheap language-rule win. |

This is a manageable first PR.

### What Not To Do

Do not generate full typed wrappers for all 12 current Parceltongue languages on day one.

That would create a large codegen surface before the product contract is proven.

Do not keep typed generated code and handwritten query logic equally authoritative.

That creates two sources of truth.

Do not let generated facts drop raw spans.

Codex needs source evidence, not just abstract graph claims.

Do not allow capture-name collisions to silently disambiguate without a warning.

Capture names are product semantics in Parceltongue.
If two captures collapse to the same generated name, the language-pack manifest should report it.

### Shreyas Product Read

This is a "make the core loop trustable" feature.

The user does not wake up wanting typed Tree-sitter wrappers.
The user wants Codex to change a large codebase without missing public dependencies.

Typed generated facts matter only because they make this answer more trustworthy:

```text
Before editing this function, read this public trait,
this implementor,
this exported route,
this test,
and ignore these private helpers for now.
```

The PMF framing:

```text
Typed wrappers are not the product.
Typed facts are the trust infrastructure behind the product.
```

The minimum lovable result:

```text
Parceltongue can explain why it knows a symbol is public,
which grammar/query fact proved it,
and exactly where Codex should look.
```

That is more valuable than having a giant AST API nobody uses.

### Reference Pattern Summary

| Pattern | Source Repo | Parceltongue Takeaway |
|---|---|---|
| Typed node wrappers from `node-types.json` | `Jakobeha__type-sitter` | Generate fact-accessor facades instead of stringly `node.kind()` checks. |
| Supertypes as enums | `Jakobeha__type-sitter` | Product supertypes like `PublicSurfaceDeclaration` should be explicit. |
| Typed query captures | `Jakobeha__type-sitter` | Edge evidence and confidence can come from typed capture variants. |
| Query directories as recursive modules | `Jakobeha__type-sitter` | Language packs should compile query folders into stable modules. |
| Custom supertypes | `Jakobeha__type-sitter` | Parceltongue can add product semantics missing from grammar supertypes. |
| Regeneration guard | `Jakobeha__type-sitter` | Generated output directories should be protected from deleting hand-written code. |
| Pydantic node-types schema | `JoranHonig__python-tree-sitter-types` | A simple schema model is enough to start. |
| Recursive raw-node-to-typed-object parser | `JoranHonig__python-tree-sitter-types` | Keep raw nodes available while adding typed convenience. |
| Typed class generation with field annotations | `JoranHonig__python-tree-sitter-types` | Field names should become generated contract, not scattered strings. |
| Field-name collision TODOs | `JoranHonig__python-tree-sitter-types` | Reserved/colliding names must be tested before codegen is trusted. |

### Repos And Docs Touched For This Concept

| Source | Evidence | Notes |
|---|---|---|
| `/Users/amuldotexe/.codex/skills/codebase-memory-evidence-reader/SKILL.md` | lines 1-72 | Required evidence-reader skill; graph output is an accelerator and direct source reads remain final proof. |
| `/tmp/codex-code-intel/codebase-memory/Jakobeha__type-sitter-20260706-233700/index_repository.json` | local output | Codebase-memory indexed `type-sitter` with 17,473 nodes and 58,325 edges. |
| `/tmp/codex-code-intel/codebase-memory/JoranHonig__python-tree-sitter-types-20260706-233817/index_repository.json` | local output | Codebase-memory indexed `python-tree-sitter-types` with 71 nodes and 125 edges. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/README.md` | lines 16-49 | Type-sitter overview: typed wrappers from `node-types.json`, typed query wrappers from query s-expressions, typed fields, typed captures, extra nodes, custom supertypes, optional yak-sitter. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/README.md` | lines 53-63 | Usage modes and vendoring requirement for grammar root, `node-types.json`, queries, and shared object. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/README.md` | lines 99-155 | Build-script generation pattern and `include!(OUT_DIR)` integration. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/README.md` | lines 157-203 | Custom supertypes and hidden-node naming examples. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/README.md` | lines 251-281 | CLI generation mode, input flexibility, and regeneration caveats. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/README.md` | lines 283-321 | Untyped vs typed import-path example and generated-code size/performance drawbacks. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/README.md` | lines 323-404 | Naming rules, reserved-name handling, deterministic disambiguation, module placement, and query capture naming. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-gen/src/queries/mod.rs` | lines 22-132 | Public query generation API: query path, language path, nodes path, yak-sitter flag, custom module paths, and `NodeTypeMap` load from `src/node-types.json`. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-gen/src/queries/mod.rs` | lines 134-203 | Recursive query directory/file generation and submodule emission for directories. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-gen/src/node_types/mod.rs` | lines 16-65 | `generate_nodes` accepts anything convertible into `NodeTypeMap`. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-gen/src/node_types/mod.rs` | lines 67-114 | Custom module path generation and printing each node type into generated tokens. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-gen/src/node_types/print.rs` | lines 16-78 | NodeType printing dispatches regular product nodes versus supertype sum nodes. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-gen/src/node_types/print.rs` | lines 80-155 | Generated product wrapper struct, `KIND`, `try_from_raw`, raw access, and conversion. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-gen/src/node_types/print.rs` | lines 158-307 | Generated enum/sum type wrapper and subtype conversion/access. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-gen/src/node_types/print.rs` | lines 459-647 | Common subtype fields and generated child/field accessors. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-lib/src/query/mod.rs` | lines 16-57 | Typed query trait with associated match/capture types and raw wrapper safety contract. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-lib/src/query/captures.rs` | lines 9-61 | Typed capture wrapper shape: query, raw capture, node, name, and index. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-lib/src/query/captures.rs` | lines 125-177 | Iteration maps raw captures into typed query captures. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-cli/src/args.rs` | lines 10-53 | CLI args: input type, output dir, language dir, yak-sitter flag, and path/output pairs. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-cli/src/args.rs` | lines 62-121 | Input type inference for `node-types.json`, `.scm`, query directories, and language roots. |
| `git-ref-repo/ignore-this-folder-repos/Jakobeha__type-sitter/type-sitter-cli/src/process.rs` | lines 45-112 | Regeneration flow, safe directory removal, and recursive node/query generation for language roots. |
| `git-ref-repo/ignore-this-folder-repos/JoranHonig__python-tree-sitter-types/README.md` | lines 4-32 | Python typed interface generation motivation, CLI use, parser install/load library functions, and typing/autocomplete value. |
| `git-ref-repo/ignore-this-folder-repos/JoranHonig__python-tree-sitter-types/tree_sitter_types/node_types/__init__.py` | lines 1-32 | Pydantic schema for `node-types.json`: type specs, child specs, field specs, node types, node type list parser. |
| `git-ref-repo/ignore-this-folder-repos/JoranHonig__python-tree-sitter-types/tree_sitter_types/generation/code_generator.py` | lines 8-52 | Type annotation construction from field/children specs, including Union/List/Optional handling. |
| `git-ref-repo/ignore-this-folder-repos/JoranHonig__python-tree-sitter-types/tree_sitter_types/generation/code_generator.py` | lines 55-99 | Class generation for node types with field annotations and `field_names`. |
| `git-ref-repo/ignore-this-folder-repos/JoranHonig__python-tree-sitter-types/tree_sitter_types/generation/code_generator.py` | lines 115-146 | Type-name-to-class map, base class with `base_node`, and generated import surface. |
| `git-ref-repo/ignore-this-folder-repos/JoranHonig__python-tree-sitter-types/tree_sitter_types/parser/parser.py` | lines 4-29 | Depth-first traversal over raw Tree-sitter cursor/tree. |
| `git-ref-repo/ignore-this-folder-repos/JoranHonig__python-tree-sitter-types/tree_sitter_types/parser/parser.py` | lines 31-83 | Recursive raw-node to typed-object conversion, field handling, child fallback, base node retention, and TODOs. |
| `git-ref-repo/ignore-this-folder-repos/JoranHonig__python-tree-sitter-types/tree_sitter_types/cli/__init__.py` | lines 15-47 | CLI reads `node-types.json`, builds generated class source, and writes target file. |
| `git-ref-repo/ignore-this-folder-repos/JoranHonig__python-tree-sitter-types/tree_sitter_types/parser/parser_manager.py` | lines 7-44 | Parser installation, shared-library build, cleanup, and load path. |

## Concept 34: Turn Structural Search And Structural Diff Into Changed Public Contracts

### Core Idea

Parceltongue should not treat "search", "diff", and "dependency graph" as three separate
features. For agentic code assistance, the highest-value product shape is a tool that can
answer this question:

```text
The agent is about to edit or has just edited this code.
What public contract changed, what depends on it, and what should the agent inspect next
with the smallest useful token packet?
```

The reference repos in this concept point to three complementary pieces:

1. `ast-grep__ast-grep` shows how to make structural search feel like code-shaped search,
   with meta-variables, relational constraints, replacement templates, and rule validation.
2. `afnanenayet__diffsitter` shows how to expose Tree-sitter parsing as a bounded navigation
   surface for tools and agents, plus a pragmatic AST-leaf diff path.
3. `Wilfred__difftastic` shows how to normalize a raw Tree-sitter tree into a simpler syntax
   representation, diff it as a graph problem, and fall back conservatively when the parse or
   graph budget is not trustworthy.

The Parceltongue synthesis:

```text
typed Tree-sitter facts
  -> structural search rules
  -> before/after public-fact diff
  -> dependency/blast-radius expansion
  -> token-budgeted agent context packet
```

This is different from "show me a diff". It is closer to:

```text
Tell Codex what changed that matters, why it matters, and where to look next.
```

### Why This Concept Comes After Concept 33

Concept 33 argued for generated typed Tree-sitter facts before writing query logic.
Concept 34 is the next layer. Once Parceltongue has typed facts, it can compare facts,
not raw files.

Raw AST diffing answers:

```text
Which syntax nodes changed?
```

Typed fact diffing answers:

```text
Which public names, signatures, schemas, routes, trait contracts, call edges,
exports, configuration keys, or externally meaningful behavior surfaces changed?
```

That is the agentic code-assistance primitive. The LLM does not need the entire file when it
asks a tool for context. It needs the smallest coherent dependency packet around the changed
contract.

### Evidence Table

| Repo | Evidence Used | What It Proves | Parceltongue Relevance |
|---|---|---|---|
| `ast-grep__ast-grep` | CodeGraphContext indexed 298 files, 2,754 functions; direct reads of `README.md`, `crates/language/src/lib.rs`, `crates/core/src/matcher.rs`, `crates/core/src/matcher/pattern.rs`, `crates/core/src/meta_var.rs`, `crates/core/src/replacer.rs`, `crates/core/src/replacer/template.rs`, `crates/config/src/check_var.rs`, `crates/config/src/rule/relational_rule.rs` | Structural matching can be code-shaped, typed by language, pruned by node kind, and validated before use. | Use this style for public-contract rules and dangerous-change detectors. |
| `afnanenayet__diffsitter` | CodeGraphContext indexed 101 files, 379 functions; direct reads of `src/parse.rs`, `src/ast_navigation.rs`, `src/input_processing.rs`, `src/diff.rs`, `src/mcp_server.rs`, `src/bin/diffsitter.rs`, `tests/ast_navigation_test.rs` | Tree-sitter can be wrapped as a narrow navigation/MCP surface with parse caching, node lookup, scope lookup, query execution, symbol listing, definition lookup, and children lookup. | This is very close to the "LLM asks query -> tool returns bounded context" user journey. |
| `Wilfred__difftastic` | CodeGraphContext scan was attempted but ended with signal 143 before a completed smoke footer; direct reads of `README.md`, `manual/src/parsing.md`, `manual/src/diffing.md`, `manual/src/language_detection.md`, `manual/src/tree_diffing.md`, `src/parse/syntax.rs`, `src/parse/tree_sitter_parser.rs`, `src/diff/changes.rs`, `src/diff/shortest_path.rs`, `src/diff/graph.rs`, `src/main.rs`, `src/options.rs` | Structural diff needs normalization, cost modeling, graph limits, parse-error limits, and conservative text fallback. | Use this as the pattern for honest changed-contract diffing under large-codebase pressure. |

### Product Journey: Agent Asks For The Smallest Correct Next Context

The journey Parceltongue should optimize is not:

```text
User asks broad question.
Agent dumps files into context.
LLM guesses.
```

It should be:

```text
Agent asks Parceltongue:
  "I am editing `create_user_session_from_token`.
   What public contract and dependency context should I inspect before changing it?"

Parceltongue returns:
  - the function signature and nearby typed facts
  - callers and callees that define the contract
  - public exports/routes/schemas/trait impls touched by the function
  - tests that exercise the contract
  - changed-contract risk flags
  - exact next entities to inspect, sorted by usefulness per token
```

After the edit:

```text
Agent asks Parceltongue:
  "Compare before and after. Did the public contract change?"

Parceltongue returns:
  - changed public facts
  - dependencies affected by those facts
  - confidence level
  - fallback reasons if AST/fact matching was incomplete
  - next verification commands or tests to run
```

This is the Shreyas Doshi view: the product is not a graph. The product is an improved
agent decision at the point where the agent would otherwise waste context, miss a dependency,
or change a public surface accidentally.

### The Three Tool Layers

Parceltongue should keep three layers separate internally, even if the user experience feels
like one tool.

```text
Layer 1: Structural Search
  Finds code-shaped facts and rule matches.

Layer 2: Structural Diff
  Finds changed syntax or changed normalized facts between two versions.

Layer 3: Dependency Expansion
  Explains who relies on the changed contract and what the agent should inspect next.
```

If these layers are mixed too early, the tool becomes brittle. If they are separate, the agent
can ask increasingly precise questions:

```text
Search: "Where are public route handlers?"
Diff:   "Which public route facts changed?"
Graph:  "Which clients/tests/callers depend on those changed route facts?"
```

### Ast-Grep Pattern: Code-Shaped Structural Search

Ast-grep's strongest idea is that a structural pattern can look like ordinary code. A user
does not need to learn a full AST schema before writing a useful match. The README presents
patterns using meta-variables such as `$MATCH`, and the language layer preprocesses those
patterns so they can parse in languages where the literal meta-variable marker would be illegal.

The important implementation details:

| Pattern | Source Evidence | Parceltongue Adaptation |
|---|---|---|
| Code-shaped pattern source | `crates/core/src/matcher/pattern.rs:345-375` builds patterns from source strings after language preprocessing. | Let Parceltongue rules be written as example code snippets where possible. |
| Pattern root kind pruning | `Pattern::potential_kinds` in `crates/core/src/matcher/pattern.rs:408-428` returns a `BitSet` of possible target kinds when it can. | Public-contract rules should declare target node kinds so scans avoid whole-tree matching. |
| Contextual pattern selection | `Pattern::contextual` in `crates/core/src/matcher/pattern.rs:364-375`; tests around `public_field_definition` at lines 549-567. | Rules like "public method inside impl" should support selector-plus-context matching. |
| Meta-variable environment | `MetaVarEnv` in `crates/core/src/meta_var.rs:14-21`; inserts and repeated-variable checks at lines 32-48 and 89-118. | Captured names/signatures/types should become structured facts, not only strings. |
| Multi-capture support | `MetaVariable::MultiCapture` and `Multiple` in `crates/core/src/meta_var.rs:223-233`. | Rules can capture argument lists, generic params, match arms, body statements, or route middleware lists. |
| Rule validation | `crates/config/src/check_var.rs:63-133` checks variables used in constraints/transforms/fixes. | Parceltongue should reject public-impact rules that reference undefined captures. |
| Relational rules | `Inside`, `Has`, `Precedes`, and `Follows` in `crates/config/src/rule/relational_rule.rs`. | "This function has an attribute", "this type is inside a module", "this export follows a declaration" should be first-class. |

The exact move to borrow:

```text
Do not expose raw Tree-sitter queries as the only interface.
Expose code-shaped patterns and compile them down into Tree-sitter-aware matchers.
```

Tree-sitter queries are powerful, but they are not the best front door for an LLM or a solo
power user. A code-shaped pattern is closer to how the agent already thinks.

### Ast-Grep Pattern: Meta-Variables Are Facts, Not Strings

Ast-grep's meta-variable environment is more disciplined than string substitution. It keeps:

```text
single captures
multi captures
transformed variables
```

It also rejects inconsistent repeated captures by checking structural equality. If a pattern
uses the same capture twice, the second occurrence must match the first. That is extremely
useful for Parceltongue.

For public-contract detection, repeated captures can express rules like:

```text
public function `$NAME` changed return type
exported symbol `$NAME` has no matching test named around `$NAME`
route `$PATH` changed handler but not auth middleware
trait method `$METHOD` changed impl signature
```

The matcher should not merely report:

```json
{ "NAME": "foo" }
```

It should report a structured fact:

```rust
struct CapturedContractFact {
    rule_id: RuleId,
    capture_name: String,
    node_kind: String,
    symbol_name: Option<String>,
    source_span: SourceSpan,
    text_preview: String,
    stable_fingerprint: String,
}
```

That lets the dependency layer connect captures to entities, call edges, exports, and tests.

### Ast-Grep Pattern: Relational Rules Are The Missing Middle

The relational rules in ast-grep matter because public contracts are rarely detectable from a
single node alone.

Examples:

```text
The node is public because it is inside a public module.
The method is public because it is in a trait impl that is exported.
The handler is public because it has a route attribute.
The schema field is public because it is inside a serializable struct.
The config key is public because it is read from environment/config binding code.
```

A single Tree-sitter node kind is not enough. Parceltongue needs relation-aware rule primitives:

| Rule Primitive | Meaning | Example Use |
|---|---|---|
| `inside` | Current node has an ancestor matching a rule. | Function inside exported module. |
| `has` | Current node contains a child/descendant matching a rule. | Struct has public field. |
| `precedes` | Current node comes before another relevant node. | Comment/doc contract before declaration. |
| `follows` | Current node comes after another relevant node. | Implementation follows trait declaration. |
| `field` | Relation constrained to a Tree-sitter field id. | Match only name/type/body fields, not all children. |
| `stop_by` | Bound traversal by neighbor/end/rule. | Avoid unbounded scans inside large files. |

The important implementation detail is that ast-grep turns field names into numeric field IDs
through the language abstraction. Parceltongue should do the same. Repeated string comparisons
over node kinds and field names are fine for a prototype, but public-impact scanning should use
compiled rule objects.

### Ast-Grep Pattern: Validate Rules Before Agents Use Them

For agentic workflows, rule validation is product safety. Ast-grep checks that captures used
in constraints, transforms, and fixes are actually defined. Parceltongue should apply the same
idea to analysis rules, not just rewrites.

Bad rule:

```yaml
id: rust-public-function-return-type
match:
  pattern: "pub fn $NAME($$$ARGS) -> $RET { $$$BODY }"
emit:
  symbol: "$NAME"
  return_type: "$RETURN_TYPE"
```

The rule defines `$RET` but emits `$RETURN_TYPE`. Parceltongue should reject this rule at load
time, not silently emit partial facts.

For a solo power user, this matters because bad rules can quietly poison the agent context.
The LLM then reasons from false or missing evidence. Rule validation is not polish; it is a
guardrail against expensive context mistakes.

### Diffsitter Pattern: A Narrow Agent Navigation Surface

Diffsitter has a very relevant MCP-facing surface:

```text
parse_file
get_node_at_position
get_scope
navigate
query
list_symbols
get_definition
get_children_of
```

This is almost exactly the "LLM asks query -> tool responds with relevant context" loop.
The key design is not that it exposes Tree-sitter. The key design is that it exposes small,
bounded answers.

`NodeInfo` intentionally includes:

```text
kind
is_named
span
text preview
child count
field name
named children summary
```

It also truncates inline text at a fixed limit. That is a small but important token-control
pattern. If a node is huge, the tool should not stuff the whole node into the LLM by default.

Parceltongue should use the same shape, but with dependency awareness:

```rust
struct AgentNodeContext {
    entity_id: EntityId,
    kind: EntityKind,
    symbol_name: Option<String>,
    source_span: SourceSpan,
    preview: String,
    named_children: Vec<ChildSummary>,
    callers_summary: Vec<EdgeSummary>,
    callees_summary: Vec<EdgeSummary>,
    public_contract_summary: Option<PublicContractSummary>,
    omitted_reason: Option<String>,
}
```

This is not a giant AST dump. It is a bounded context object.

### Diffsitter Pattern: Scope Lookup Is A Product Primitive

`get_scope` walks up from a point to the innermost language-specific scope and returns a parent
chain. The language-specific scope list includes nodes like Rust `function_item`, `impl_item`,
`struct_item`, `enum_item`, `trait_item`, and `mod_item`.

That is exactly what an agent needs when it has an error location:

```text
Compiler error at line 211.
What scope am I in?
What symbol is this?
What parent impl/trait/module gives it meaning?
What dependency context should I inspect next?
```

Parceltongue should support this as a first-class workflow:

```text
GET /context/point?file=src/foo.rs&line=211&column=17&budget=2000
```

The response should include:

```text
innermost entity
parent entity chain
public/private status
callers/callees clipped to budget
tests touching the entity
next recommended query
```

This is much better than asking the LLM to infer context from a file chunk.

### Diffsitter Pattern: Symbol Query With Heuristic Fallback

Diffsitter uses language-specific Tree-sitter queries for symbols when available. If no query
works, it falls back to root named children with a `name` field.

This is an important PMF pattern for Parceltongue:

```text
Perfect language support is not required to be useful.
But imperfect support must be labeled.
```

Parceltongue should score every extracted fact:

| Confidence | Source | Agent Behavior |
|---|---|---|
| `high` | Generated typed grammar fact or validated language-specific query. | Use for dependency and public-contract decisions. |
| `medium` | Heuristic `name` field extraction or generic scope kind. | Use for orientation, ask for confirmation before risky edits. |
| `low` | Text fallback, regex fallback, parse-error fallback. | Return as candidate context only, not proof. |

This gives Codex a way to reason honestly:

```text
I found this as a medium-confidence symbol, so I should inspect source before editing.
```

### Diffsitter Pattern: Parse Cache With Mtime Invalidation

Diffsitter's `ParseCache` stores parsed files by canonical path and reparses when the file's
modified time changes. That is exactly the kind of simple, robust cache Parceltongue needs
before inventing heavier database machinery.

For a solo agent power user, the steady-state loop is:

```text
agent reads/edit file
agent asks context question
tool reparses only stale files
tool returns compact graph/fact answer
```

The cache should be transparent. The agent should not need to care whether the answer came from
fresh parse, stale invalidation, or persisted index, as long as the response includes:

```text
index_timestamp
file_mtime_seen
staleness
confidence
```

### Diffsitter Pattern: Diff Vectors Are Flattened AST Leaves

Diffsitter flattens Tree-sitter leaves into `Entry` values and compares `kind_id + text`.
It can split leaves into Unicode graphemes, strip whitespace, include/exclude node kinds, and
treat configured pseudo-leaf types as leaves.

For Parceltongue, this is not the final algorithm, but it gives a useful analogy:

```text
Diffsitter flattens syntax leaves.
Parceltongue should flatten public-contract facts.
```

Instead of comparing every leaf token, compare:

```text
PublicFunctionFact
PublicTypeFact
PublicFieldFact
PublicRouteFact
PublicSchemaFact
PublicTraitFact
ExportFact
CallEdgeFact
TestCoverageFact
ConfigKeyFact
```

Then compute the edit script over facts:

```text
removed public function
added public function
same name, changed signature
same route, changed handler
same schema, changed field type
same trait method, changed impl coverage
```

That is a much smaller diff for the LLM and much closer to what the agent must act on.

### Difftastic Pattern: Normalize Raw Tree-Sitter Into Product Syntax

Difftastic does not diff the raw Tree-sitter parse tree directly. Its manual explains that it
converts the parse tree into a simpler syntax tree of atoms and lists. Source code confirms
the representation:

```text
Syntax::Atom
Syntax::List
SyntaxInfo
unique_id
content_id
parent/sibling traversal cells
change metadata
```

This is a key architectural pattern:

```text
Raw Tree-sitter nodes are parser-shaped.
Agent context should be product-shaped.
```

For Parceltongue, the normalized layer should not be `Atom` and `List`. It should be:

```text
EntityFact
EdgeFact
ContractFact
ScopeFact
ExportFact
TestFact
DiagnosticFact
```

Each normalized fact should have:

```text
stable identity
content fingerprint
source span
language
confidence
public/private meaning
dependency links
```

Then the agent works with stable facts instead of raw syntax trivia.

### Difftastic Pattern: Separate Unique Identity From Content Identity

Difftastic's syntax nodes carry both unique IDs and content IDs. Unique ID says "this exact
node instance"; content ID says "this content is the same as another node's content regardless
of position".

Parceltongue needs the same distinction.

Example:

```text
pub fn parse_user_token(input: &str) -> Result<UserToken>
```

A unique fact ID might include:

```text
repo
file path
span
language
node id
```

A content ID should include:

```text
symbol namespace
name
public signature
generic params
return type
visibility
attributes
```

Why this matters:

```text
Moved function, same signature:
  unique fact changed location
  content fact stayed same
  public contract probably unchanged

Same function, changed return type:
  unique fact stayed similar
  content fact changed
  public contract changed
```

This distinction is crucial for reducing false alarms.

### Difftastic Pattern: Structural Diff Needs Budgets

Difftastic treats structural diffing as graph search and uses limits:

```text
byte limit
parse error limit
graph limit
text fallback
check-only mode
```

This is directly relevant to large-codebase agent work. A tool that always attempts the most
precise graph operation can become the bottleneck. Worse, it can leave the LLM waiting or
force the agent to abandon the tool.

Parceltongue should expose budget-aware modes:

| Mode | Purpose | Behavior |
|---|---|---|
| `check_only` | Fast "did any public contract change?" | Compare public fact fingerprints only. |
| `contract_diff` | Explain changed public facts. | Return changed fact objects and source spans. |
| `impact_context` | Help the agent decide next files. | Expand dependency graph around changed facts. |
| `deep_structural_diff` | Investigate confusing changes. | Use AST-aware diff with higher graph budget. |
| `fallback_text_context` | Stay useful when parse/graph fails. | Return line/text spans with explicit fallback reason. |

The user experience should make fallback visible:

```json
{
  "mode": "impact_context",
  "confidence": "medium",
  "fallbacks": [
    "difftastic-style graph budget exceeded for src/big_generated.rs",
    "used typed public facts plus line diff"
  ]
}
```

The agent can still act, but it knows to be more cautious.

### Difftastic Pattern: Check-Only Is A Great Agent Primitive

Difftastic supports a check-only mode to determine whether two files have the same AST without
calculating the full diff. Parceltongue should implement the public-contract equivalent:

```text
Are the before/after public contract facts equivalent?
```

This is a very high-PMF command for Codex:

```bash
parseltongue check-public-impact --base HEAD --worktree . --json
```

Possible response:

```json
{
  "has_public_contract_changes": true,
  "changed_fact_count": 3,
  "highest_risk": "route_signature_changed",
  "next_query": "parseltongue explain-public-impact --fact route:POST:/sessions"
}
```

The agent does not need a wall of output. It needs a fast yes/no plus the next precise query.

### The Changed Public Contract Data Model

A first pass model:

```rust
enum PublicContractKind {
    Function,
    Type,
    Field,
    Trait,
    Impl,
    Route,
    Schema,
    Export,
    ConfigKey,
    TestContract,
}

enum ContractChangeKind {
    Added,
    Removed,
    SignatureChanged,
    VisibilityChanged,
    AttributeChanged,
    SchemaChanged,
    RouteChanged,
    DependencyChanged,
    BodyChangedPubliclyRelevant,
    MovedWithoutContractChange,
}

struct PublicContractFact {
    id: String,
    content_id: String,
    kind: PublicContractKind,
    language: String,
    symbol_name: Option<String>,
    visibility: Option<String>,
    signature: Option<String>,
    source_span: SourceSpan,
    dependency_edges: Vec<EdgeId>,
    confidence: Confidence,
}

struct PublicContractChange {
    change_kind: ContractChangeKind,
    before: Option<PublicContractFact>,
    after: Option<PublicContractFact>,
    impacted_entities: Vec<EntityImpact>,
    evidence_spans: Vec<SourceSpan>,
    confidence: Confidence,
    fallback_reasons: Vec<String>,
}
```

The key is that `PublicContractChange` should be the primary output to Codex, not a raw AST
node, not a raw graph node, and not a raw file diff.

### Proposed Parceltongue Pipeline

```text
1. Parse changed files.
2. Generate typed facts from Tree-sitter nodes.
3. Compile structural rules by language.
4. Run public-contract extractors.
5. Compute before/after fact diff.
6. Attach source spans and confidence.
7. Expand dependency graph only around changed facts.
8. Rank next inspection targets by usefulness per token.
9. Return compact agent packet.
```

The pipeline should support both pre-edit and post-edit workflows.

Pre-edit:

```text
Given an entity, what contract could this edit affect?
```

Post-edit:

```text
Given before/after, what contract did this edit actually affect?
```

### Concrete API Shape

The API should be boring and composable:

```text
GET /api/v1/context/point
GET /api/v1/context/entity
GET /api/v1/search/structural
GET /api/v1/diff/public-contract
GET /api/v1/impact/public-contract
GET /api/v1/check/public-contract
```

Example request:

```json
{
  "repo": ".",
  "base": "HEAD",
  "target": "worktree",
  "budget_tokens": 4000,
  "include_tests": true,
  "include_callers": true,
  "include_callees": true,
  "confidence_floor": "medium"
}
```

Example response:

```json
{
  "summary": "2 public contracts changed",
  "changes": [
    {
      "kind": "SignatureChanged",
      "entity": "parse_user_token",
      "before_signature": "fn parse_user_token(&str) -> Result<UserToken>",
      "after_signature": "fn parse_user_token(&str, Clock) -> Result<UserToken>",
      "impact": ["create_user_session_from_token", "session_api_tests"],
      "confidence": "high"
    }
  ],
  "next_to_inspect": [
    {
      "entity": "create_user_session_from_token",
      "reason": "direct caller of changed public function",
      "estimated_tokens": 720
    }
  ],
  "fallbacks": []
}
```

### Token Efficiency: Why This Beats File Dumping

A file dump spends tokens on:

```text
imports
formatting
unchanged body code
nearby private helpers
comments unrelated to current task
syntax trivia
```

A changed-contract packet spends tokens on:

```text
changed facts
contract spans
direct dependency edges
tests
next inspection targets
fallback warnings
```

This is the core PMF:

```text
Codex needs less context, but better context.
```

For a large codebase, this matters more than UI polish. A tool that saves 4,000 tokens and
prevents one missed dependency is immediately useful to a solo power user.

### What To Borrow From Each Repo

| Repo | Borrow | Do Not Borrow Blindly |
|---|---|---|
| `ast-grep__ast-grep` | Code-like structural patterns, meta-variable env, pattern preprocessing, potential-kind pruning, relational rules, undefined capture validation. | Do not turn Parceltongue into a codemod-first tool. Search/rewrite is a means, not the product. |
| `afnanenayet__diffsitter` | MCP-style narrow tools, parse cache, scope lookup, symbol query fallback, node previews, AST navigation by point. | Do not stop at syntax navigation; Parceltongue must add dependency and public-impact semantics. |
| `Wilfred__difftastic` | Normalized syntax layer, unique/content identity split, change map, graph-cost diff, check-only mode, parse/graph fallback reasons. | Do not expose human visual diff as the primary agent interface. Codex needs decision context, not side-by-side display. |

### Design Rule: Every Result Needs A Confidence And A Next Query

For an LLM-facing tool, returning facts is not enough. Each response should include:

```text
confidence
evidence spans
what was omitted
why it was omitted
recommended next query
estimated token cost
```

Example:

```json
{
  "entity": "SessionStore",
  "confidence": "medium",
  "evidence": [
    "struct exported from crate root",
    "constructor called by API route",
    "serializer derive detected"
  ],
  "omitted": [
    "transitive callers beyond depth 2"
  ],
  "next_query": {
    "name": "impact/public-contract",
    "args": {
      "entity": "SessionStore",
      "depth": 3,
      "budget_tokens": 3000
    }
  }
}
```

The next query is important because it turns Parceltongue into a navigation partner rather
than a one-shot report.

### Rust-Specific Public Contract Rules

Initial Rust rules for Parceltongue:

| Rule ID | Detects | Structural Signal |
|---|---|---|
| `rust_public_function_signature` | Public function contract. | `pub fn` name, args, return type, attributes. |
| `rust_trait_method_contract` | Trait API surface. | `trait_item` method signatures and associated types. |
| `rust_impl_public_method` | Public method surface. | `impl_item` containing public `function_item`. |
| `rust_export_surface` | Re-exported symbol. | `pub use`, crate root exports, module visibility. |
| `rust_serde_schema_surface` | Serialized schema contract. | Struct/enum with serde derives and public fields. |
| `rust_route_handler_surface` | HTTP/API route surface. | Axum/Actix/Rocket route macros or router construction. |
| `rust_error_contract` | Error variants visible to callers. | Public enum variants implementing error traits. |
| `rust_feature_flag_contract` | Feature/config public surface. | `cfg(feature = ...)`, env/config key reads. |

This is where ast-grep-style relational rules help:

```text
pub fn inside public module
function has route attribute
struct has serde derive
enum follows error derive
impl has public method
```

### TypeScript-Specific Public Contract Rules

For CRUD apps and agent work across TypeScript:

| Rule ID | Detects | Structural Signal |
|---|---|---|
| `ts_exported_function_contract` | Exported function/API. | `export function`, exported const arrow function. |
| `ts_route_handler_contract` | Route/API endpoint. | Express/Fastify/Next route registrations. |
| `ts_zod_schema_contract` | Request/response schema. | `z.object`, exported schema constants, inferred types. |
| `ts_public_type_contract` | External type surface. | `export interface`, `export type`, public class. |
| `ts_env_config_contract` | Runtime config API. | `process.env`, config schema, env validation. |
| `ts_test_contract_link` | Tests around changed surface. | Test names and imports referencing changed export. |

The tool should not need perfect framework knowledge to start. It can return high-confidence
facts for exported types/functions and medium-confidence facts for route/config conventions.

### C/C++ Systems Contract Rules

For C and C++ systems programming:

| Rule ID | Detects | Structural Signal |
|---|---|---|
| `c_public_header_symbol` | Header-level public API. | Declarations in `.h`/`.hpp`. |
| `c_function_signature_contract` | Function ABI/API surface. | Function declarator signature. |
| `c_struct_layout_contract` | Struct layout that may affect ABI. | Public struct fields, order, types. |
| `cpp_class_public_method` | Public class contract. | Public section methods and fields. |
| `cpp_template_contract` | Template API surface. | Template params and exported declarations. |
| `c_macro_contract` | Macro API surface. | Public macro definitions in headers. |

This is where difftastic-style conservative fallback matters. C/C++ parse errors and
preprocessor behavior are common. Parceltongue should label confidence rather than pretending
the raw AST is complete.

### Product Functionality That Falls Out Of Concept 34

This concept supports multiple concrete tools:

| Functionality | Agent Journey | Why It Matters |
|---|---|---|
| `check_public_impact` | Agent checks whether an edit changed public contracts. | Fast yes/no before commit or before broader refactor. |
| `explain_public_impact` | Agent asks why a changed contract matters. | Avoids missed callers/tests/docs. |
| `context_for_point` | Agent starts from compiler error line/column. | Gives scope and dependency context quickly. |
| `structural_search_public` | Agent searches for code-shaped public patterns. | Better than regex when locating API surfaces. |
| `contract_diff_since_base` | Agent compares worktree to `HEAD`. | Post-edit verification loop. |
| `next_context_to_read` | Agent asks what to inspect next under token budget. | Directly optimizes context usage. |
| `fallback_report` | Agent sees what was not trusted. | Prevents overconfident edits. |

### A Possible CLI For Codex App Usage

For this user's workflow, the CLI should be simple enough for Codex to call with shell:

```bash
parseltongue check-public-impact --base HEAD --target worktree --json
parseltongue explain-public-impact --entity parse_user_token --budget 4000 --json
parseltongue context-for-point src/session.rs:211:17 --budget 2000 --json
parseltongue structural-search --lang rust --pattern 'pub fn $NAME($$$ARGS) -> $RET { $$$BODY }' --json
parseltongue next-context --entity SessionStore --task refactor --budget 6000 --json
```

The output should be intentionally boring JSON, because Codex can parse, rank, and decide.

### Tests Parceltongue Should Add

| Test ID | Test Name | Requirement |
|---|---|---|
| `T524` | `detect_rust_public_function_signature_change` | Changing a Rust public function return type emits `SignatureChanged`. |
| `T526` | `ignore_private_body_only_change` | Changing only a private helper body does not emit public-contract change unless it affects a public dependency rule. |
| `T528` | `detect_route_handler_contract_change` | Changing route path/method/handler emits route contract change. |
| `T530` | `validate_undefined_rule_capture` | A structural rule that emits an undefined capture fails at load time. |
| `T532` | `return_scope_context_for_point` | A file/line/column query returns innermost scope and parent chain. |
| `T534` | `fallback_when_parse_error_limit_exceeded` | Parse failures return fallback reason and do not claim high confidence. |
| `T536` | `fallback_when_graph_budget_exceeded` | Structural diff budget overflow returns text/fact fallback reason. |
| `T538` | `rank_next_context_by_token_value` | Dependency expansion returns direct impacted entities before broad transitive context. |
| `T540` | `detect_moved_contract_without_signature_change` | Moving a public function without changing signature is labeled separately from contract change. |
| `T542` | `diff_public_facts_not_raw_ast` | Formatting-only AST changes do not become public contract changes. |

### Concept-Level Acceptance Criteria

```text
WHEN Parceltongue compares before/after code
THEN it SHALL report changed public facts, not just changed lines.

WHEN structural rule matching uses captures
THEN it SHALL validate all referenced captures before running.

WHEN parsing or graph diffing exceeds configured limits
THEN it SHALL return a conservative fallback with explicit reason.

WHEN an agent asks for context around a changed contract
THEN it SHALL return a token-budgeted packet with next inspection targets.

WHEN a changed contract has callers, callees, tests, routes, or schemas
THEN it SHALL include those dependency edges before unrelated file context.
```

### PMF Judgment For This Concept

This is one of the most relevant directions for Parceltongue.

Reason:

```text
Codex already has shell, rg, git diff, language servers, and direct file access.
What Codex lacks is a compact, reliable explanation of what code relationships matter
after a change.
```

Structural search alone is not enough. Difftastic-style diff alone is not enough.
Tree-sitter MCP navigation alone is not enough.

The high-PMF combination is:

```text
structural facts + changed public contracts + dependency expansion + token budget
```

That is a real solo power-user tool. It helps Codex navigate faster and more reliably in
large codebases without asking the user to productize anything for other people.

### Most Useful Build Slice

The smallest useful Parceltongue V2 slice from this concept:

```text
Rust-only public contract diff for worktree vs HEAD.
```

Scope:

```text
1. Parse changed Rust files.
2. Extract public functions, structs, enums, traits, impl methods, and pub uses.
3. Compare before/after public facts.
4. For changed facts, find direct callers/callees from existing dependency graph.
5. Return JSON with confidence, spans, and next entities to inspect.
```

Do not start with every language. Do not start with perfect C++ preprocessor handling.
Do not start with a UI.

Start with:

```bash
parseltongue check-public-impact --base HEAD --json
```

If this is fast and reliable, Codex will use it constantly.

### Implementation Notes For Parceltongue

1. Keep rule compilation separate from rule execution.
2. Store language-specific node kind IDs and field IDs in compiled rules.
3. Store captures as typed fact references, not only raw strings.
4. Use a parse cache with file mtimes.
5. Assign both unique IDs and content IDs to normalized facts.
6. Compare public facts before doing expensive AST graph diffing.
7. Use graph/fact budget limits and always return fallback reasons.
8. Rank dependency expansion by directness, confidence, and estimated token cost.
9. Return a `next_query` in every nontrivial result.
10. Keep the CLI JSON-first so Codex App can use it through shell today.

### Search Keywords This Concept Adds

These are the new keywords Parceltongue research should track:

```text
structural search
code-shaped AST pattern
meta-variable environment
multi-capture structural pattern
relational AST rule
Tree-sitter query capture
AST navigation MCP
symbol query fallback
scope chain lookup
AST leaf vector diff
syntax-aware diff
AST-aware diff
Tree-sitter structural diff
public contract diff
semantic public impact
API surface change detector
typed fact diff
normalized syntax tree
unique node id
content id
graph diff budget
parse error fallback
check-only structural diff
token-budgeted code context
agent next-context query
```

### Summary

The lesson from these repos is not "use Tree-sitter". The lesson is:

```text
Use Tree-sitter to generate trustworthy structural facts.
Use structural search to identify meaningful code contracts.
Use structural diff to detect meaningful changes.
Use dependency graphs to explain impact.
Use budgets and confidence to keep the agent honest.
```

That is the Parceltongue product center.

## Concept 35: Treat Parser Lifecycle As A Cache Contract, Not A Parser Detail

### Why This Concept Exists

Concept 34 said Parceltongue should produce changed public facts, not just changed
lines. That immediately raises a harder question:

```text
Can Parceltongue trust its syntax trees across edits?
```

Tree-sitter's answer is precise. Incremental parsing is not magic. The caller has to
keep the old tree aligned with the edited source, pass that edited old tree back to
the parser, and then use changed ranges to invalidate downstream facts. If Parceltongue
gets this lifecycle wrong, every graph edge, public contract diff, and token-budgeted
context packet becomes suspect.

So this concept is about a boring but extremely high-leverage design rule:

```text
Parser lifecycle must be a first-class Parceltongue cache contract.
```

Not:

```text
We happen to keep a parser around for speed.
```

But:

```text
Every parse result has a language ABI, grammar identity, included-range policy,
source revision, edit history, tree revision, changed-range set, query budget,
and invalidation consequence.
```

That is the difference between "Tree-sitter wrapper" and "agent context engine".

### Repositories Inspected

| Repo | Why It Matters | Evidence Collected |
|---|---|---|
| `tree-sitter__tree-sitter` | Canonical C implementation and public API for parsers, trees, ranges, nodes, queries, ABI boundaries, cancellation, and logging. | codebase-memory scan: 7915 nodes / 27164 edges. CodeGraphContext scan: 494 files / 3039 functions / 97 classes / 361 structs / 83 enums / 371 modules. Direct reads from `lib/include/tree_sitter/api.h`, `lib/src/parser.c`, `lib/src/tree.c`, `lib/src/get_changed_ranges.c`, `lib/src/query.c`, and parser docs. |
| `tree-sitter__py-tree-sitter` | A compact binding that shows the user-facing API shape for parser lifecycle: `Parser.parse`, `Tree.edit`, `Tree.changed_ranges`, `Node.edit`, `QueryCursor` bounds, included ranges, logger, and ABI checks. | codebase-memory scan: 546 nodes / 1346 edges. CodeGraphContext scan: 31 files / 139 functions / 11 classes / 15 structs / 30 modules. Direct reads from `tree_sitter/__init__.pyi`, C binding files, and tests. |

### CodeGraphContext Evidence

The user explicitly asked to use CodeGraphContext here, so it is treated as required
evidence, not optional decoration.

| Repo | CodeGraphContext Result | What It Added |
|---|---|---|
| `tree-sitter__tree-sitter` | Smoke run completed at `/tmp/codex-code-intel/codegraphcontext/tree-sitter__tree-sitter-20260706-235900`; verified output did not mention `gitrefrepo/`. Explicit CGC lookups found `ts_parser_parse`, `ts_tree_get_changed_ranges`, and `ts_query_cursor_set_match_limit` locations before source confirmation. | Confirmed repository scale and gave a second index view over core parser/query/tree implementation files before direct source reads. |
| `tree-sitter__py-tree-sitter` | Smoke run completed at `/tmp/codex-code-intel/codegraphcontext/tree-sitter__py-tree-sitter-20260706-235821`. CGC indexed the Python tests/examples and found lifecycle tests like `test_parse_with_one_included_range`, `test_parse_with_multiple_included_ranges`, and `test_parse_with_a_newly_excluded_range`; exact C extension function-name lookup was less useful, so binding internals were verified by direct reads. | Confirmed the binding is small enough to treat as an API design reference rather than only a language binding. |

CodeGraphContext was used as navigational evidence. The claims below are grounded in
direct source reads because the skill's rule is: graph evidence helps find things;
source evidence proves them.

### Core Thesis

Parceltongue should model Tree-sitter parsing as a lifecycle state machine:

```text
language selected
included ranges selected
source version parsed
tree stored
source edit received
old tree edited
new source reparsed with old tree
changed ranges computed
facts invalidated
queries bounded
dependency graph patched
context packet emitted
```

That lifecycle should be visible in code, visible in CLI output, and visible to Codex.

The wrong product shape:

```text
parse(file) -> AST
```

The right product shape:

```text
parse_incremental_file_version(file, source_revision, edit_batch, cache_policy)
  -> ParseLifecycleReport {
       tree_revision,
       changed_ranges,
       invalidated_facts,
       retained_facts,
       query_budget_used,
       parser_confidence,
       next_context_targets
     }
```

### Pattern 1: ABI Compatibility Belongs In The Cache Key

Tree-sitter has hard parser/library compatibility boundaries. The core header defines
the currently supported language ABI and minimum compatible ABI:

```text
TREE_SITTER_LANGUAGE_VERSION = 15
TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION = 13
```

The public `ts_parser_set_language` contract returns false when the language was
generated with an incompatible Tree-sitter CLI version. The C implementation enforces
that check before it accepts a language. The Python binding exposes the same idea as
`Language.abi_version` and raises a `ValueError` if the language ABI is outside the
supported range.

For Parceltongue, this means:

```text
language_id alone is not enough for parser cache identity.
```

The cache key must include at least:

| Cache Field | Why |
|---|---|
| `language_id` | Rust, TypeScript, Python, C, C++, etc. |
| `tree_sitter_language_abi` | Prevents reusing trees/facts across incompatible generated parsers. |
| `grammar_revision` | Grammar node kinds and fields can change even when the language name does not. |
| `parser_runtime_revision` | The runtime's behavior and supported ABI window can change. |
| `query_bundle_revision` | Parceltongue queries depend on grammar node names and field names. |

Suggested shape:

```rust
pub struct ParserCacheKey {
    pub repo_root_id: RepoRootId,
    pub file_path: Utf8PathBuf,
    pub language_id: LanguageId,
    pub tree_sitter_language_abi: u32,
    pub grammar_revision: GrammarRevision,
    pub parser_runtime_revision: ParserRuntimeRevision,
    pub query_bundle_revision: QueryBundleRevision,
    pub included_range_hash: IncludedRangeHash,
}
```

The important PMF point:

```text
An agent should never debug a stale parser ABI issue by reading random code.
Parceltongue should reject the stale cache and say why.
```

### Pattern 2: Old Trees Must Be Edited Before Incremental Reparse

Tree-sitter's incremental flow is strict:

```text
1. The source changes.
2. Caller constructs TSInputEdit with byte and point coordinates.
3. Caller applies ts_tree_edit to the old tree.
4. Caller parses the new source and passes that edited old tree.
5. Tree-sitter reuses unchanged subtrees.
```

The docs are explicit that efficient reparsing needs two steps: edit the syntax tree,
then parse again with the old tree. The `ts_parser_parse` API also says that, for
reuse to work correctly, the old syntax tree must already have been edited in a way
that exactly matches the source code changes.

The Python tests show the exact workflow:

```python
tree = parser.parse(b"def foo():\n  bar()")
tree.edit(...)
new_tree = parser.parse(b"def foo(ab):\n  bar()", tree)
changed_ranges = tree.changed_ranges(new_tree)
```

Parceltongue implication:

```text
Edit handling is not an implementation detail inside the parser adapter.
It is the boundary between source revision N and source revision N+1.
```

If Parceltongue receives a file edit from Codex, a watch event, or a git diff, it
should normalize it into one internal type:

```rust
pub struct SourceInputEdit {
    pub start_byte: u32,
    pub old_end_byte: u32,
    pub new_end_byte: u32,
    pub start_point: SourcePoint,
    pub old_end_point: SourcePoint,
    pub new_end_point: SourcePoint,
}
```

And the lifecycle method should make the edit order impossible to bypass:

```rust
pub fn parse_incremental_file_version(
    cache_entry: &mut ParserCacheEntry,
    new_source: &SourceText,
    source_edit: SourceInputEdit,
) -> ParseLifecycleReport {
    cache_entry.old_tree.edit(&source_edit);
    let new_tree = cache_entry.parser.parse(new_source, Some(&cache_entry.old_tree));
    let changed_ranges = cache_entry.old_tree.changed_ranges(&new_tree);
    cache_entry.replace_tree(new_tree);
    cache_entry.invalidate_changed_facts(changed_ranges)
}
```

The exact method names above are illustrative, but the order should be enforced in
real code.

### Pattern 3: Stored Nodes Are Stale Unless Edited Or Re-Fetched

Tree-sitter nodes carry tree identity and position context. The core API exposes
`ts_node_edit`, but the docs emphasize it is only needed if the caller kept a `TSNode`
retrieved before the tree edit and wants to continue using that same node instance.
Often the better approach is to re-fetch nodes from the edited tree.

This is a huge warning for Parceltongue.

Do not persist raw `TSNode` handles as durable graph identities.

Persist this instead:

| Durable Thing | Purpose |
|---|---|
| `FactId` | Stable Parceltongue identity for a discovered code fact. |
| `SourceSpan` | Byte and point range in a specific source revision. |
| `ContentFingerprint` | Helps detect unchanged fact bodies across parse revisions. |
| `GrammarPath` | Node-kind/field path used to re-find a fact after edits. |
| `TreeRevision` | Prevents accidental cross-tree node reuse. |

Suggested rule:

```text
Tree-sitter node handles are per-tree navigation cursors.
Parceltongue facts are persisted graph objects.
```

That separation prevents a class of stale-node bugs where the graph says a function
still starts at byte 100 but the source edit moved it to byte 127.

### Pattern 4: Changed Ranges Are Structural Invalidation Hints

Tree-sitter changed ranges are not "the exact text that changed". The public API says
they represent ranges where hierarchical syntax structure changed between old and new
trees. Characters outside those ranges have identical ancestor nodes in both trees.
It also says the returned ranges can be slightly larger than the exact changed area.

The implementation is also instructive. `ts_tree_get_changed_ranges` compares both
the old/new roots and the old/new included ranges. That means included-range changes
can produce syntax changes even when the raw source bytes are mostly unchanged.

The Python tests make this concrete:

```text
When an included range is newly excluded, changed_ranges includes the directive range
because the syntax tree changed even though the edit was about inclusion policy.
```

Parceltongue should therefore treat changed ranges like this:

```text
changed_ranges = minimal structural invalidation candidate set
```

Not like this:

```text
changed_ranges = all semantic impact
```

Downstream invalidation should combine:

| Signal | Why |
|---|---|
| Source edit byte range | Captures the raw textual edit. |
| Tree-sitter changed ranges | Captures syntactic structure shifts. |
| Included-range differences | Captures embedded-language parse boundary shifts. |
| Public fact diff | Captures API contract changes. |
| Dependency graph reachability | Captures callers/callees that did not syntactically change. |

Suggested invalidation flow:

```text
changed source bytes
  -> edited old tree
  -> new tree
  -> changed structural ranges
  -> facts intersecting changed ranges
  -> public fact diff
  -> dependency expansion
  -> token-budgeted context packet
```

This gives Parceltongue a principled answer to:

```text
What must Codex inspect next?
```

### Pattern 5: Included Ranges Are Part Of Parse Identity

Tree-sitter supports parsing only selected ranges of a document while preserving
coordinates in the whole document. That is how applications can parse embedded
languages in templates.

The core docs show an embedded-template example:

```text
Parse the whole file as ERB.
Extract HTML ranges and Ruby ranges from the ERB tree.
Set parser language and included ranges.
Parse HTML and Ruby over the original document.
```

The Python tests show the same shape across HTML, JavaScript, and template strings:
parse the outer language, derive ranges for the embedded language, switch parser
language, set included ranges, and parse the same source bytes with full-document
coordinates.

For Parceltongue, included ranges should be promoted to first-class graph input:

```rust
pub struct IncludedRangePolicy {
    pub owner_language: LanguageId,
    pub target_language: LanguageId,
    pub ranges: Vec<SourceRange>,
    pub extracted_from_fact: Option<FactId>,
    pub policy_hash: IncludedRangeHash,
}
```

This matters for real solo power-user work:

| File Type | Why Included Ranges Matter |
|---|---|
| `.tsx` / JSX | JavaScript/TypeScript plus embedded JSX grammar regions. |
| Vue/Svelte/Astro | Script/template/style sections need separate structural facts. |
| Markdown with code fences | Code blocks can be parsed by language while keeping document coordinates. |
| HTML with script/style | JavaScript/CSS facts live inside HTML containers. |
| Rust doc tests | Code examples can matter to agent reasoning and tests. |
| SQL strings in application code | Future slice could parse embedded SQL ranges. |

Cache implication:

```text
Same file bytes + same grammar + different included ranges = different tree.
```

So `included_range_hash` belongs in `ParserCacheKey`.

### Pattern 6: Query Cursor Bounds Are Token-Budget Controls

Tree-sitter query cursors are not just "run query and get captures". The core API and
Python binding expose several controls:

| Control | Meaning For Parceltongue |
|---|---|
| `set_byte_range` | Return matches intersecting a byte window. |
| `set_point_range` | Return matches intersecting a row/column window. |
| `set_containing_byte_range` | Return only matches fully contained in a byte window. |
| `set_containing_point_range` | Return only matches fully contained in a point window. |
| `set_max_start_depth` | Limit how deep the cursor starts matching. |
| `match_limit` | Cap in-progress matches. |
| `did_exceed_match_limit` | Detect that the query result may be incomplete. |
| query progress callback | Stop long-running query execution. |

The C implementation starts query cursors with unbounded ranges and unlimited depth.
That is fine for a library. It is not fine for an agent context tool.

Parceltongue should never run agent-facing structural queries without an explicit
budget policy.

Suggested query policy:

```rust
pub struct StructuralQueryBudget {
    pub byte_range: Option<SourceByteRange>,
    pub point_range: Option<SourcePointRange>,
    pub containing_range: Option<SourceByteRange>,
    pub max_start_depth: Option<u32>,
    pub match_limit: u32,
    pub time_budget_ms: u64,
    pub token_budget: usize,
}
```

Agent-facing behavior:

```text
If did_exceed_match_limit is true, Parceltongue must say:
"results_truncated": true
"reason": "tree_sitter_query_match_limit_exceeded"
"next_query": "narrow byte range or raise match limit"
```

That is much more useful than silently dropping matches and letting Codex reason from
an incomplete picture.

### Pattern 7: Parser Reset And Cancellation Must Be Explicit

The core parser has a progress callback. If parsing is cancelled, Tree-sitter can
resume where it left off on the next parse. The API says that if the caller does not
want to resume and intends to parse another document, it must call `ts_parser_reset`.
The Python binding exposes `Parser.reset()` with the same public shape.

This is an agent reliability detail.

Codex may ask Parceltongue to inspect:

```text
file A
file B
file C
then back to file A
```

If a long parse or query is cancelled and the parser silently resumes against the
wrong logical task, the agent gets confusing behavior.

Parceltongue should model cancellation like this:

```text
cancelled parse
  -> mark parser state dirty
  -> reset before next unrelated document
  -> report cancellation reason
```

Suggested report field:

```rust
pub enum ParserLifecycleState {
    Clean,
    IncrementalReuse,
    CancelledNeedsReset,
    ResetBeforeParse,
    AbiRejected,
    QueryLimitExceeded,
    ParseFailed,
}
```

The agent should see this in JSON. Not as logs buried in stderr.

### Pattern 8: Logging And DOT Graphs Are Debugging Interfaces, Not Product UI

Tree-sitter exposes parser logging and DOT graph output. The Python binding keeps this
visible through `logger` and `print_dot_graphs`.

Parceltongue should not make the normal product experience depend on DOT graphs, but
it should preserve the debugging channel:

```bash
parseltongue parse-file src/lib.rs --debug-dot /tmp/parse.dot
parseltongue parse-file src/lib.rs --debug-log /tmp/parse.log
```

For a solo Codex power user, that matters when a grammar behaves unexpectedly. The
agent can ask for a debug artifact, but ordinary flows stay JSON-first.

### The Agent User Journey

The user journey for Codex should feel like this:

```text
Codex sees an edit or bug.
Codex asks Parceltongue for the lifecycle-aware context.
Parceltongue says what changed structurally.
Parceltongue says which facts were invalidated.
Parceltongue says what public contracts changed.
Parceltongue says which dependency edges make the change risky.
Parceltongue returns the smallest useful snippets and next queries.
Codex edits with less wandering.
```

Concrete example:

```bash
parseltongue parse-incremental \
  --file crates/core/src/session.rs \
  --from-revision HEAD \
  --to-revision worktree \
  --budget 4000 \
  --json
```

Possible response shape:

```json
{
  "file": "crates/core/src/session.rs",
  "language": "rust",
  "tree_sitter_language_abi": 15,
  "parser_lifecycle_state": "IncrementalReuse",
  "source_edit_count": 2,
  "changed_ranges": [
    {
      "start_byte": 4180,
      "end_byte": 4621,
      "start_point": {"row": 132, "column": 0},
      "end_point": {"row": 147, "column": 1}
    }
  ],
  "invalidated_facts": [
    "function:parse_user_session_token",
    "impl_method:SessionStore::load_user_session"
  ],
  "retained_facts": [
    "struct:SessionStore",
    "enum:SessionError"
  ],
  "public_contract_changes": [
    {
      "fact_id": "function:parse_user_session_token",
      "change": "return_type_changed",
      "confidence": "high"
    }
  ],
  "dependency_edges_to_inspect": [
    "caller:authenticate_request_handler",
    "test:test_parse_user_session_token_expired"
  ],
  "query_budget": {
    "match_limit": 512,
    "did_exceed_match_limit": false,
    "token_budget": 4000,
    "estimated_tokens_returned": 1280
  },
  "next_query": "parseltongue explain-public-impact --fact function:parse_user_session_token --budget 2500 --json"
}
```

This is the key product behavior:

```text
LLM asks query to tool.
Tool responds with relevant lifecycle-aware context.
LLM decides what to inspect next with fewer tokens.
```

### What This Adds Beyond LSP

A language server can tell Codex "definition", "references", maybe "call hierarchy".
That is useful, but it does not usually give:

```text
old tree was edited correctly
changed structural ranges
included range changes
query budget status
public fact invalidation
cross-language embedded range identity
agent-ready next query
```

Parceltongue's opportunity is not to replace LSP. The opportunity is to sit beside
LSP as the structural cache and dependency explanation layer.

### What This Adds Beyond Plain `rg`

`rg` is still the first tool for literals. But `rg` cannot say:

```text
This edit only changed a function body.
This edit changed an exported signature.
This edit changed the parse boundary for embedded JavaScript.
This query exceeded a match limit and may be incomplete.
This syntax tree was produced by ABI 15 grammar revision X.
This fact can be retained across the edit because its structural ancestors did not change.
```

That is why this concept is PMF-relevant for a Codex app power user.

### Suggested Parceltongue Data Model

```rust
pub struct ParserCacheEntry {
    pub cache_key: ParserCacheKey,
    pub source_revision: SourceRevision,
    pub source_hash: SourceHash,
    pub tree_revision: TreeRevision,
    pub tree: ParsedTreeHandle,
    pub included_ranges: Vec<SourceRange>,
    pub parse_errors: Vec<ParseErrorFact>,
    pub last_changed_ranges: Vec<SourceRange>,
    pub last_lifecycle_state: ParserLifecycleState,
    pub fact_index: FactIndexSnapshot,
}

pub struct ParseLifecycleReport {
    pub file_path: Utf8PathBuf,
    pub language_id: LanguageId,
    pub source_revision_before: SourceRevision,
    pub source_revision_after: SourceRevision,
    pub lifecycle_state: ParserLifecycleState,
    pub changed_ranges: Vec<SourceRange>,
    pub invalidated_facts: Vec<FactId>,
    pub retained_facts: Vec<FactId>,
    pub public_contract_changes: Vec<PublicContractChange>,
    pub dependency_edges_to_inspect: Vec<DependencyEdgeId>,
    pub query_budget_report: QueryBudgetReport,
    pub confidence: ParseConfidence,
    pub next_query: Option<String>,
}
```

The thing to notice:

```text
The parse tree is not the product.
The lifecycle report is the product.
```

### Suggested Commands

```bash
parseltongue parse-file src/lib.rs --json
parseltongue parse-incremental src/lib.rs --base HEAD --target worktree --json
parseltongue changed-ranges src/lib.rs --base HEAD --target worktree --json
parseltongue invalidated-facts src/lib.rs --base HEAD --target worktree --budget 4000 --json
parseltongue query-structural src/lib.rs --query queries/rust-public.scm --byte-range 1000:5000 --match-limit 512 --json
parseltongue explain-parser-cache src/lib.rs --json
```

For Codex, `explain-parser-cache` is especially useful. It should answer:

```text
Can this cached tree be trusted?
Why or why not?
What changed since the last parse?
What facts were retained?
What facts were invalidated?
Did any query limit truncate results?
```

### Tests Parceltongue Should Add

| Test ID | Test Name | Requirement |
|---|---|---|
| `T544` | `reject_parser_cache_on_abi_mismatch` | A cached tree/fact bundle with a different Tree-sitter language ABI is rejected with explicit reason. |
| `T546` | `edit_old_tree_before_incremental_parse` | Incremental parse applies the source edit to the old tree before reusing it. |
| `T548` | `do_not_persist_raw_node_handles` | Persistent facts store spans/fingerprints/tree revision, not raw Tree-sitter node handles. |
| `T550` | `invalidate_facts_intersecting_changed_ranges` | Changed structural ranges invalidate intersecting facts and retain unrelated facts. |
| `T552` | `included_ranges_change_cache_identity` | Same source bytes with different included ranges create a distinct parse identity. |
| `T554` | `report_query_match_limit_exceeded` | Structural query results include `did_exceed_match_limit` and truncation reason. |
| `T556` | `reset_parser_after_cancelled_unrelated_parse` | A cancelled parse marks parser state dirty and resets before parsing another document. |
| `T558` | `bound_query_by_byte_and_point_range` | Agent-facing query APIs require explicit byte or point bounds unless caller chooses `full_file`. |
| `T560` | `included_range_diff_invalidates_embedded_facts` | Embedded-language facts are invalidated when included ranges change even if source bytes are mostly stable. |
| `T562` | `return_next_query_after_incremental_parse` | Lifecycle reports include a next query when changed facts have dependency edges. |

### Concept-Level Acceptance Criteria

```text
WHEN Parceltongue reuses a Tree-sitter tree
THEN it SHALL first verify language ABI, grammar revision, query bundle revision,
and included-range identity.

WHEN Parceltongue performs an incremental parse
THEN it SHALL apply the source edit to the old tree before passing it to parse.

WHEN Parceltongue has stored facts from a previous tree revision
THEN it SHALL invalidate facts intersecting changed structural ranges and retain
unchanged facts only when their tree revision and fingerprint rules allow it.

WHEN Parceltongue runs a structural query for an agent
THEN it SHALL use explicit range/depth/match/time budget controls or mark the query
as intentionally full-file.

WHEN Tree-sitter reports a query limit or cancellation
THEN Parceltongue SHALL surface that state in JSON and provide a next query or fallback.
```

### PMF Judgment For This Concept

This is a high-PMF concept for the user's stated use case:

```text
Solo agent power user.
All languages.
CRUD apps plus Rust/C/C++ systems programming.
Use Codex app.
Help agent navigate large codebases faster and more reliably.
```

The reason is simple:

```text
Fast unreliable context is worse than slow direct reading.
Fast reliable lifecycle-aware context is a superpower.
```

Tree-sitter already gives the low-level machinery. The product gap is the lifecycle
report that an LLM can consume without reading parser internals every time.

Score:

```text
PMF for Parceltongue: 94 / 100
```

Why not 100:

```text
This is infrastructure. It becomes product only when paired with public-contract
facts, dependency expansion, and next-query recommendations.
```

But as infrastructure, it is foundational. If parser lifecycle is wrong, everything
above it is untrustworthy.

### Most Useful Build Slice

Build this slice first:

```text
Rust-only incremental parse lifecycle report for worktree vs HEAD.
```

Scope:

```text
1. Parse one Rust file at HEAD.
2. Parse the worktree version.
3. Convert the diff hunk into SourceInputEdit where possible.
4. Edit the old tree.
5. Reparse with old tree.
6. Compute changed ranges.
7. Extract function/struct/enum/trait facts in changed ranges.
8. Return JSON with invalidated facts, retained facts, and next query.
```

Then expand to TypeScript.

Do not start with every language. The lifecycle abstraction must be proven in one
language first, with tests that make stale-cache behavior impossible to ignore.

### Search Keywords This Concept Adds

```text
Tree-sitter incremental parsing
Tree-sitter parser lifecycle
Tree-sitter old tree reuse
TSInputEdit
ts_tree_edit
ts_node_edit
ts_tree_get_changed_ranges
Tree-sitter changed ranges
Tree-sitter included ranges
Tree-sitter ABI version
Tree-sitter language ABI
tree_sitter Parser.parse old_tree
tree_sitter Tree.edit
tree_sitter Tree.changed_ranges
tree_sitter QueryCursor match_limit
tree_sitter QueryCursor byte range
tree_sitter QueryCursor point range
structural query budget
syntax tree cache invalidation
parser cache key
grammar revision cache
included range hash
embedded language parsing
stale syntax node
fact invalidation graph
parse lifecycle report
agent code context lifecycle
```

### Summary

The key lesson from Tree-sitter core and py-tree-sitter is:

```text
Parsing is a lifecycle, not a function call.
```

For Parceltongue, that means:

```text
Make parser cache identity explicit.
Make edit order impossible to bypass.
Make changed ranges feed fact invalidation.
Make included ranges part of parse identity.
Make query bounds mandatory for agent-facing APIs.
Make cancellation and match-limit truncation visible to Codex.
```

If Concept 34 is about "what public contracts changed", Concept 35 is about "can we
trust the trees and facts used to answer that question".

### Next Concepts To Add

1. Inspect Tree-sitter grammar repos for `node-types.json`, query files, corpus tests, injections, locals, highlights, and language-specific schema patterns that can make Parceltongue's fact extraction less brittle.
2. Inspect `BloopAI__bloop`, `sourcegraph__scip`, and similar code-intelligence repos for cross-language symbol identity and repository-scale indexing patterns that could feed `PublicContractFact`.
3. Inspect `Cody`, `Aider`, `Continue`, and MCP-oriented code tools for how agent-facing code context should package summaries, snippets, and next-step recommendations.
