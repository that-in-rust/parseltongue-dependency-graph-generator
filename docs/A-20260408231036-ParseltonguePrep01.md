# Parseltongue Code Graph PRD

```
Document:     Parseltongue Code Graph PRD
Status:       DRAFT v0.1
Authors:      [WIP]
Reviewers:    [WIP]
Last Updated: 2026-04-07
File:         parseltongue-code-graph-prd.md
```

---

## 1. Executive summary

**Parseltongue is a tree-sitter-only code analysis tool that builds a persisted, multi-dimensional dependency graph from source code without invoking any compiler or performing semantic analysis.** It extracts entities (functions, classes, structs, imports, tests, comments) and directed edges (calls, containment, sibling relationships) across nine languages, persists them in a compact graph format, and exposes them through a Tauri desktop application for visualization, querying, and export. A parallel objective is generating a large-scale open-source dataset of low-level design (LLD) documentation from OSS codebases for model training.

The system enforces three hard constraints: (1) **tree-sitter is the sole parsing technology** — no `rustc`, `javac`, `tsc`, or any compiler frontend; (2) the graph captures only forward calls, backward calls, and public interface exports — no type inference, control flow, or data flow; (3) **no source code is stored** — only graph structure, entity metadata, and edges.

This PRD specifies the parsing model, entity taxonomy, edge extraction rules, MECE accounting invariants, three complete architectural options, the OSS dataset pipeline, and the Tauri application design. It is intended for a senior engineering audience and includes decision matrices, WIP markers, and open questions.

---

## 2. Scope and constraints

### 2.1 Supported languages

| Language | Tree-sitter Grammar | File Extensions |
|----------|-------------------|-----------------|
| Rust | `tree-sitter-rust` | `.rs` |
| C | `tree-sitter-c` | `.c`, `.h` |
| C++ | `tree-sitter-cpp` | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hh`, `.hxx`, `.h` |
| Java | `tree-sitter-java` | `.java` |
| JavaScript | `tree-sitter-javascript` | `.js`, `.mjs`, `.cjs`, `.jsx` |
| TypeScript | `tree-sitter-typescript` | `.ts`, `.tsx`, `.mts`, `.cts` |
| Python | `tree-sitter-python` | `.py`, `.pyi` |
| Go | `tree-sitter-go` | `.go` |
| Ruby | `tree-sitter-ruby` | `.rb`, `.rake`, `.gemspec` |

### 2.2 Non-negotiable constraints

- **Tree-sitter ONLY.** No compiler, no rustc, no javac, no tsc, no clang, no semantic analysis of any kind.
- **Three edge families only:** forward calls (A calls B → A→B), backward calls (reverse traversal of the same edge), and public interface dependency graph export.
- **No code storage.** The persisted graph contains entity metadata (name, kind, LOC range, file path), graph structure (adjacency), and edge metadata (type, direction). Raw source code is never written to the graph store.
- **Tauri desktop app.** Users manage triggering of graph generation, reindexing, and visualization through a native desktop application.
- **OSS dataset generation.** The system must be capable of crawling, parsing, and exporting LLD documentation from thousands of open-source codebases into a training-ready dataset format.

---

## 3. Tree-sitter parsing specification

### 3.1 Parsing hierarchy

The parsing model follows a strict three-level hierarchy: **folder → file → entity**. Each level is an entity in the graph with containment edges flowing downward.

**Folder parsing.** Every directory encountered during traversal is inserted as `entity_type = "folder"` with its absolute or project-relative path as the identifier. Folders form a tree via parent→child containment edges.

**File parsing — unsupported extensions.** If a file's extension does not match any of the nine supported languages, it is inserted as `entity_type = "opaque-file"` with only its filepath and filename recorded. No parsing occurs. This ensures the folder WC (word count) invariant holds — every file contributes to its parent folder's total, even if it cannot be analyzed.

**File parsing — supported extensions.** Files matching supported extensions are inserted as `entity_type = "file"` and fully parsed by the appropriate tree-sitter grammar. The file's total LOC is recorded. The file is then decomposed into MECE entities at exactly one level below the file — **no nesting is allowed**.

### 3.2 The MECE decomposition rule

Every byte in a source file must belong to exactly one entity. Entities are **mutually exclusive** (no byte belongs to two entities) and **cumulatively exhaustive** (every byte belongs to some entity). Entities exist at exactly **one level below the file** — there are no nested entities in the graph.

**ISGL1 (Interface Signature Graph Level 1)** defines each entity as a public-interface-level construct with a LOC range expressed as `start_line:end_line`. The entity's word count is computed from the raw byte range, and the file-level invariant requires that the sum of all entity word counts equals the file's total word count.

**Gap handling.** Blank lines, whitespace, and standalone expressions between top-level nodes become `entity_type = "gap"` entities. The gap-filling algorithm iterates over the root node's named children, creating gap entities for any byte ranges not covered by named children:

```
entities = []
prev_end = 0
for child in root.named_children:
    if child.start_byte > prev_end:
        entities.append(GapEntity(prev_end, child.start_byte))
    entities.append(entity_from_node(child))
    prev_end = child.end_byte
if prev_end < file_size:
    entities.append(GapEntity(prev_end, file_size))  // trailing whitespace/newlines
```

This guarantees the MECE property: every byte is accounted for exactly once.

### 3.3 The flattening rule for nested constructs

Since no nested entities are allowed, constructs that naturally contain sub-definitions must be flattened to file level:

- **Rust `impl` blocks:** An `impl_item` containing methods is decomposed. Each `function_item` inside the `declaration_list` becomes a file-level entity with a qualified name like `StructName::method_name`. The `impl` header (from `impl` keyword to opening brace, plus the closing brace) becomes its own entity capturing the `impl` signature. For trait implementations, the qualified name includes the trait: `<TraitName for StructName>::method_name`.
- **Java inner classes:** A `class_declaration` nested inside another class body is extracted as a file-level entity with qualified name `OuterClass.InnerClass`. Its methods become `OuterClass.InnerClass.methodName`. Anonymous classes are extracted as `OuterClass.$anon_N`.
- **Python nested functions/classes:** A `function_definition` inside another function becomes `outer_func.inner_func`. A class method becomes `ClassName.method_name`. Nested classes become `OuterClass.InnerClass`.
- **JavaScript/TypeScript class methods:** `method_definition` nodes inside `class_body` become `ClassName.methodName`. Arrow functions assigned to variables become entities named after the variable.
- **C++ namespace members:** Functions inside `namespace_definition` become `Namespace::function_name`. Nested namespaces become `Outer::Inner::function_name`.
- **Ruby module/class methods:** Methods inside `class` or `module` nodes become `ClassName#method_name` (instance) or `ClassName.method_name` (class-level).

**[WIP]** The exact byte-range accounting for flattened entities needs careful specification. When an `impl` block is decomposed, the bytes of the `impl` header, the bytes of each method, and any gap bytes (blank lines between methods) must sum to the total byte range of the `impl_item` node.

### 3.4 Entity types per language — tree-sitter node type mapping

The following table maps each language's top-level tree-sitter node types to Parseltongue entity types. These are the **named children of the root node** (e.g., children of `source_file` for Rust, `translation_unit` for C/C++, `program` for Java/JS/TS/Ruby, `module` for Python, `source_file` for Go).

#### Rust (`source_file` root)

| Tree-sitter Node Type | Parseltongue Entity Type | Notes |
|----------------------|------------------------|-------|
| `function_item` | `function` | Free functions at module level |
| `struct_item` | `struct` | |
| `enum_item` | `enum` | |
| `trait_item` | `trait` | |
| `impl_item` | `impl` | Flattened: header + methods extracted separately |
| `type_item` | `type_alias` | `type Foo = Bar;` |
| `const_item` | `constant` | |
| `static_item` | `static` | |
| `mod_item` | `module` | Inline modules; file-based modules detected via filesystem |
| `use_declaration` | `import` | |
| `extern_crate_declaration` | `import` | |
| `macro_definition` | `macro` | `macro_rules!` |
| `macro_invocation` | `macro_call` | Top-level macro invocations |
| `attribute_item` | `attribute` | Outer attributes (`#[...]`) |
| `inner_attribute_item` | `attribute` | Inner attributes (`#![...]`) |
| `line_comment` | `comment` | `//` and `///` (distinguished by content) |
| `block_comment` | `comment` | `/* */` and `/** */` |

**Test detection:** A `function_item` preceded by `attribute_item` containing `(attribute (identifier) @name)` where `@name == "test"` is classified as `entity_type = "test"` instead of `function`. Also detect `#[cfg(test)]` modules: `attribute_item` with path matching `cfg` and `test` argument → the entire `mod_item` and its contents become test entities.

#### C (`translation_unit` root)

| Tree-sitter Node Type | Parseltongue Entity Type |
|----------------------|------------------------|
| `function_definition` | `function` |
| `declaration` | `variable` or `constant` (check for `const` qualifier) |
| `type_definition` | `type_alias` (`typedef`) |
| `struct_specifier` (in declaration) | `struct` |
| `enum_specifier` (in declaration) | `enum` |
| `union_specifier` (in declaration) | `union` |
| `preproc_include` | `import` |
| `preproc_def` | `macro` |
| `preproc_function_def` | `macro` |
| `preproc_if` / `preproc_ifdef` | `preprocessor` |
| `comment` | `comment` |

**Test detection:** Top-level `call_expression` with function identifier matching `TEST`, `TEST_F`, or `TEST_P` (gtest macros) → `entity_type = "test"`.

#### C++ (`translation_unit` root)

Inherits all C node types plus:

| Tree-sitter Node Type | Parseltongue Entity Type |
|----------------------|------------------------|
| `class_specifier` (in declaration) | `class` |
| `namespace_definition` | `namespace` (flattened: members extracted) |
| `template_declaration` | `template` (wraps function/class) |
| `using_declaration` | `import` |
| `alias_declaration` | `type_alias` (`using Foo = Bar;`) |
| `concept_definition` | `concept` (C++20) |

#### Java (`program` root)

| Tree-sitter Node Type | Parseltongue Entity Type |
|----------------------|------------------------|
| `package_declaration` | `package` |
| `import_declaration` | `import` |
| `class_declaration` | `class` (flattened: methods/fields extracted) |
| `interface_declaration` | `interface` (flattened) |
| `enum_declaration` | `enum` |
| `annotation_type_declaration` | `annotation` |
| `line_comment` | `comment` |
| `block_comment` | `comment` (includes `/** */` Javadoc) |

**Test detection:** `method_declaration` with `modifiers` containing `marker_annotation` where `name` identifier equals `Test` → `entity_type = "test"`. Also detect JUnit 5 `@ParameterizedTest`, `@RepeatedTest`.

#### JavaScript (`program` root)

| Tree-sitter Node Type | Parseltongue Entity Type |
|----------------------|------------------------|
| `function_declaration` | `function` |
| `generator_function_declaration` | `function` |
| `class_declaration` | `class` (flattened) |
| `variable_declaration` / `lexical_declaration` | `variable` |
| `expression_statement` | `expression` (or `variable` if assignment) |
| `import_statement` | `import` |
| `export_statement` | `export` |
| `comment` | `comment` |

**Test detection:** `call_expression` with function identifier matching `describe`, `it`, `test`, or `beforeEach`/`afterEach` → `entity_type = "test"`. Also detect Vitest/Jest patterns.

#### TypeScript (`program` root)

Inherits all JavaScript node types plus:

| Tree-sitter Node Type | Parseltongue Entity Type |
|----------------------|------------------------|
| `type_alias_declaration` | `type_alias` |
| `interface_declaration` | `interface` |
| `enum_declaration` | `enum` |
| `abstract_class_declaration` | `class` |
| `module` (namespace) | `namespace` |
| `ambient_declaration` | `declaration` (`declare ...`) |

#### Python (`module` root)

| Tree-sitter Node Type | Parseltongue Entity Type |
|----------------------|------------------------|
| `function_definition` | `function` |
| `class_definition` | `class` (flattened: methods extracted) |
| `decorated_definition` | Unwrap: classify inner node (function/class) |
| `import_statement` | `import` |
| `import_from_statement` | `import` |
| `expression_statement` | `expression` or `variable` (if assignment) |
| `if_statement` (top-level) | `conditional` (e.g., `if __name__ == "__main__"`) |
| `comment` | `comment` (`#` lines) |
| `expression_statement` → `string` (first in body) | `docstring` (Python docstrings) |

**Test detection:** `function_definition` with `name` matching `^test_` → `entity_type = "test"`. `class_definition` with `name` matching `^Test` → all methods within become test entities. Files matching `test_*.py` or `*_test.py` → all top-level functions become test candidates.

#### Go (`source_file` root)

| Tree-sitter Node Type | Parseltongue Entity Type |
|----------------------|------------------------|
| `package_clause` | `package` |
| `import_declaration` | `import` |
| `function_declaration` | `function` |
| `method_declaration` | `method` (has receiver parameter) |
| `type_declaration` → `type_spec` with `struct_type` | `struct` |
| `type_declaration` → `type_spec` with `interface_type` | `interface` |
| `type_declaration` → `type_spec` (other) | `type_alias` |
| `var_declaration` | `variable` |
| `const_declaration` | `constant` |
| `comment` | `comment` |

**Test detection:** `function_declaration` with `name` matching `^Test` AND file path ending in `_test.go` → `entity_type = "test"`. Also detect `^Benchmark` and `^Example` prefixes.

#### Ruby (`program` root)

| Tree-sitter Node Type | Parseltongue Entity Type |
|----------------------|------------------------|
| `method` | `function` |
| `class` | `class` (flattened) |
| `module` | `module` (flattened) |
| `singleton_class` | `class` |
| `assignment` (UPPER_CASE) | `constant` |
| `assignment` (lower_case) | `variable` |
| `call` with `require`/`require_relative` | `import` |
| `comment` | `comment` |

**Test detection:** `call` with method `describe`, `it`, `context` (RSpec) → `entity_type = "test"`. `method` with `name` matching `^test_` (Minitest) → `entity_type = "test"`.

### 3.5 WC (word count) invariants

The system enforces two structural invariants that must hold after every indexing operation:

**Invariant 1 — File decomposition:** For every file `F` parsed into entities `[e₁, e₂, ..., eₙ]`:
```
WC(F) = Σᵢ WC(eᵢ)
```
where WC is computed as the byte length of the entity's source range. **Every byte in the file must belong to exactly one entity** (MECE). Gap entities account for whitespace, blank lines, and any bytes not covered by named tree-sitter children.

**Invariant 2 — Folder aggregation:** For every folder `D` containing files `[f₁, f₂, ..., fₘ]` and subfolders `[d₁, d₂, ..., dₖ]`:
```
WC(D) = Σⱼ WC(fⱼ) + Σₗ WC(dₗ)
```

**Parse failure handling.** If tree-sitter returns `ERROR` or `MISSING` nodes within a file, the file is still decomposed using the partial parse tree. Error nodes become `entity_type = "parse_error"` entities. The system logs parse failures and maintains an accuracy metric:
```
accuracy = (total_bytes_successfully_parsed) / (total_bytes_across_all_files) × 100%
```
This metric is reported per-language and per-project in the indexing summary. A parse accuracy below **95%** triggers a warning. Below **80%** triggers an error with recommendation to check grammar compatibility.

---

## 4. Edge extraction specification

All edges in the Parseltongue graph are **directed**. The direction convention follows established practice from Kythe, Sourcetrail, and SCIP: **containment edges point parent→child** (outward from cluster center), **call edges point caller→callee** (direction of data/control flow), and **inheritance edges point implementor→interface** (direction of dependency).

### 4.1 Containment edges (structural hierarchy)

| Edge Type | Source → Target | Description |
|-----------|----------------|-------------|
| `folder_contains_folder` | parent folder → child folder | Directory nesting |
| `folder_contains_file` | folder → file | File membership |
| `file_contains_entity` | file → entity | Entity membership |

These edges form a strict tree. Every node except the project root has exactly one incoming containment edge.

### 4.2 Sibling edges

Sibling edges connect nodes that share a parent. These are **supplementary edges** that enable efficient lateral traversal without ascending to the parent first.

| Edge Type | Direction Convention | Rationale |
|-----------|---------------------|-----------|
| `folder_sibling` | **Alphabetical by name** (A→B where A < B lexicographically) | Deterministic, reproducible, language-independent |
| `file_sibling` | **Alphabetical by filename** | Same rationale |
| `entity_sibling` (same file) | **Earlier LOC → later LOC** (earlier is cluster center) | Follows reading order; source-level locality |

**Same-line entities.** Tree-sitter provides both `start_position.row` and `start_position.column` for every node. If two entities start on the same line, the **column number** is the tiebreaker: left-to-right order. This can occur in C/C++ (`int a, b;`), Python tuple unpacking, and JS destructuring.

**[WIP] Decision:** Should sibling edges be materialized in the graph, or derived at query time from shared-parent relationships? Materializing them doubles edge count for sibling-heavy directories. Deriving them adds query latency but saves storage. **Leaning toward: derive at query time**, store only containment and call edges.

### 4.3 Call edges (dependency relationships)

Call edges are the core value proposition of Parseltongue. They connect an entity that invokes another entity.

**Direction:** `caller → callee` (forward call). Backward calls are the reverse traversal of the same edge — no separate edge is stored.

#### Call detection by language — tree-sitter node types

| Language | Call Node Type | Method Call Pattern | Constructor | Special |
|----------|---------------|-------------------|-------------|---------|
| **Rust** | `call_expression` | `call_expression` with `field_expression` function | N/A (constructors are `Struct::new()`) | `macro_invocation` |
| **C** | `call_expression` | N/A | N/A | — |
| **C++** | `call_expression` | `call_expression` with `field_expression` | `new_expression` | — |
| **Java** | `method_invocation` | `method_invocation` (object + name) | `object_creation_expression` | Annotations |
| **JavaScript** | `call_expression` | `call_expression` with `member_expression` | `new_expression` | Tagged templates |
| **TypeScript** | `call_expression` | `call_expression` with `member_expression` | `new_expression` | — |
| **Python** | `call` | `call` with `attribute` function | `call` (ClassName()) | Decorators |
| **Go** | `call_expression` | `call_expression` with `selector_expression` | `composite_literal` | — |
| **Ruby** | `call` | `call` (receiver + method) | `call` with method `new` | `define_method` |

#### Cross-file call resolution (heuristic, tree-sitter only)

Without a compiler, linking a call site in file A to a definition in file B is fundamentally heuristic. Parseltongue uses **import-aware name matching**:

1. **Parse imports.** Extract all import/use/require statements with their resolved paths (using filesystem conventions per language: Rust module paths follow directory structure, Python `from X.Y import Z` maps to `X/Y.py`, Go package names match directory names, JS/TS relative imports resolve via filesystem).

2. **Build a global definition index.** Collect all entity definitions across all files with their qualified names built from the nesting context.

3. **Match call sites against definition index.** For each call expression, extract the callee identifier (potentially qualified, e.g., `module::function()` in Rust, `package.Class.method()` in Java). Match against the definition index, **scoped by import visibility** — a call can only resolve to definitions in imported modules or the same file.

4. **Mark ambiguous references.** When multiple definitions match (overloaded methods, wildcard imports, dynamic dispatch), the edge is stored with `confidence = "ambiguous"` and all candidate targets are linked. Sourcetrail pioneered this approach with visually distinct dashed edges for uncertain references.

**Known limitations** (cannot be solved without semantic analysis):
- Overloaded methods in Java/C++ — multiple definitions with same name, different parameter types
- Dynamic dispatch — virtual methods, trait objects, interface implementations at runtime
- Type inference — `let x = get_thing(); x.method()` requires knowing the type of `x`
- Generics/templates — monomorphized calls cannot be resolved
- Wildcard imports — `from module import *` makes resolution ambiguous
- Dynamic imports — `require(variable)` in JS, `importlib.import_module()` in Python

**Expected accuracy** based on research into tree-sitter-tags and GitHub code navigation: **~85–90%** for well-structured codebases with explicit imports and qualified names. Accuracy degrades for dynamic languages (Python, Ruby, JS) and codebases with heavy metaprogramming.

### 4.4 Inheritance and implementation edges

| Language | Syntax | Tree-sitter Node | Direction | Detectable? |
|----------|--------|-------------------|-----------|-------------|
| Rust | `impl Trait for Struct` | `impl_item` with `trait:` and `type:` fields | implementor → trait | ✅ |
| Rust | `impl Struct` (inherent) | `impl_item` with only `type:` field | struct → impl | ✅ |
| Java | `class Foo implements Bar` | `class_declaration` → `interfaces:` → `type_list` | class → interface | ✅ |
| Java | `class Foo extends Bar` | `class_declaration` → `superclass:` | subclass → superclass | ✅ |
| TypeScript | `class Foo implements Bar` | `class_declaration` → `implements_clause` | class → interface | ✅ |
| C++ | `class Foo : public Bar` | `class_specifier` → `base_class_clause` | subclass → superclass | ✅ |
| Go | implicit interface | No syntactic indicator | — | ❌ |
| Python | `class Foo(Bar):` | `class_definition` → `argument_list` (base classes) | subclass → superclass | ⚠️ Partial |
| Ruby | `class Foo < Bar` | `class` → `superclass:` | subclass → superclass | ✅ |

Go's implicit interface satisfaction **cannot be detected** with tree-sitter alone — it requires comparing method sets against interface definitions, which is semantic analysis. This is a documented limitation.

### 4.5 Complete edge type taxonomy

| Edge Type | Direction | Scope | Priority |
|-----------|-----------|-------|----------|
| `contains` | parent → child | folder→folder, folder→file, file→entity | P0 (structural) |
| `calls` | caller → callee | entity→entity (same file or cross-file) | P0 (core value) |
| `imports` | importing file → imported module/file | file→file | P0 |
| `implements` | implementor → interface/trait | entity→entity | P1 |
| `extends` | subclass → superclass | entity→entity | P1 |
| `sibling` | alphabetical/LOC order | same-level nodes sharing parent | P2 (derived) |
| `exports` | file → exported entity | file→entity (for public interface graph) | P1 |

---

## 5. Three architectural options

Each option represents a genuinely different architectural philosophy — not parameter variations. All three share the same tree-sitter parsing and entity extraction front-end; they diverge in persistence, IPC, visualization, and operational characteristics.

### 5.1 Option A — "Monolithic Embedded" (SQLite-backed single binary)

**Philosophy:** Minimize moving parts. One Rust binary handles parsing, storage, querying, and serving the UI. SQLite provides ACID guarantees and a familiar query interface. This is the Sourcetrail model, proven in production.

**Persistence format:** SQLite database (`.ptdb` file). Nodes and edges stored as relational tables with indices on source/target/kind. Recursive CTEs handle graph traversal.

```sql
CREATE TABLE entities (
    id          INTEGER PRIMARY KEY,
    kind        TEXT NOT NULL,       -- 'function', 'class', 'module', etc.
    name        TEXT NOT NULL,
    qualified_name TEXT,
    file_path   TEXT NOT NULL,
    start_line  INTEGER,
    end_line    INTEGER,
    loc         INTEGER,
    wc          INTEGER,
    visibility  TEXT,                -- 'public', 'private', 'protected'
    signature   TEXT,                -- public interface signature
    language    TEXT NOT NULL
);
CREATE TABLE edges (
    source_id   INTEGER REFERENCES entities(id),
    target_id   INTEGER REFERENCES entities(id),
    kind        TEXT NOT NULL,       -- 'calls', 'imports', 'contains', etc.
    confidence  TEXT DEFAULT 'high', -- 'high', 'ambiguous'
    PRIMARY KEY (source_id, target_id, kind)
);
CREATE INDEX idx_edges_source ON edges(source_id, kind);
CREATE INDEX idx_edges_target ON edges(target_id, kind);
CREATE INDEX idx_entities_file ON entities(file_path);
```

**Tauri app architecture:**
- Frontend: **Svelte** (5KB compiled, minimal overhead, excellent Tauri integration)
- Graph visualization: **Cytoscape.js** with ELK.js layout engine (via Web Worker)
- IPC: Tauri commands (`#[tauri::command]`) for all queries. Progress via Channels.
- State: `rusqlite` connection pool managed as Tauri State (`app.manage()`)

**Graph generation pipeline:**
1. `notify` watcher detects file changes → debounced (1s) via `notify-debouncer-full`
2. Rayon parallel iterator walks directory tree, dispatches files to tree-sitter parsers
3. Parser produces entity list per file → INSERT into `entities` table
4. Edge extractor runs call detection queries per entity → INSERT into `edges` table
5. Cross-file edges resolved via import-aware name matching against entity index
6. SQLite FTS5 index built for entity name search

**Incremental reindexing:**
- File-level granularity (Sourcetrail's proven approach)
- On file change: DELETE all entities/edges WHERE file_path = changed file, re-parse, re-INSERT
- Track transitive dependents: if file A imports file B and B changes, re-resolve A's import edges
- Content hash per file to detect no-op changes (whitespace-only edits)

**OSS dataset generation:**
- Batch mode: CLI invocation (`parseltongue index --path /repos --export parquet`)
- Export entities and edges tables as Parquet via `arrow-rs` + `parquet` crates
- SQLite ATTACH for merging multiple project databases into a single dataset

**Public interface export:** SQL query → JSON Graph Format (JGF) or DOT. Filter `visibility = 'public'` entities and their edges.

**Pros:** Simplest deployment. ACID guarantees. Familiar SQL query interface. Excellent tooling (DB Browser for SQLite). Proven by Sourcetrail at scale.

**Cons:** Graph traversal limited by recursive CTE performance (degrades beyond ~100K entities with deep recursion). Single-threaded write path in SQLite. No native graph algorithms. ~300K entities practical ceiling before query latency becomes noticeable.

### 5.2 Option B — "Stream Pipeline" (event-driven with custom binary format)

**Philosophy:** Maximize throughput and query performance for large codebases. Event-driven pipeline with Salsa-style incremental computation. Custom memory-mapped CSR+CSC binary format enables zero-copy graph access.

**Persistence format:** Custom binary `.ptg` (Parseltongue Graph) file, memory-mapped:

```
.ptg File Layout:
┌──────────────────────────────┐
│ Header (64 bytes)            │  magic, version, counts, offsets
├──────────────────────────────┤
│ Entity Metadata Array        │  fixed-size records: kind(u8), name_offset(u32),
│ [EntityRecord; entity_count] │  name_len(u16), file_id(u32), start_line(u32),
│                              │  end_line(u32), loc(u32), wc(u32), flags(u8)
├──────────────────────────────┤
│ CSR — Outgoing Edges         │  row_ptr[entity_count+1], col_idx[edge_count],
│                              │  edge_type[edge_count]
├──────────────────────────────┤
│ CSC — Incoming Edges         │  col_ptr[entity_count+1], row_idx[edge_count]
│ (transpose of CSR)           │
├──────────────────────────────┤
│ String Table                 │  concatenated UTF-8 strings (names, paths, sigs)
├──────────────────────────────┤
│ File Table                   │  [FileRecord; file_count]: path_offset, entity_range
└──────────────────────────────┘
```

The dual CSR+CSC representation enables O(1) out-degree and in-degree lookups, O(degree) neighbor iteration in both directions, and trivial memory-mapping via `memmap2`. Total storage: **O(2(V+E))** integers plus metadata.

**Tauri app architecture:**
- Frontend: **React** (largest ecosystem for graph visualization libraries)
- Graph visualization: **Cytoscape.js** with WebGL renderer (new in v3.31) + ELK.js for hierarchical compound layout
- IPC: Tauri commands for queries. Channels for streaming indexing progress. The Rust backend memory-maps the `.ptg` file and serves subgraph queries.
- State: `memmap2`-mapped `.ptg` file as Tauri State

**Graph generation pipeline:**
1. `notify` watcher → crossbeam channel → debouncer (1s)
2. **Parser stage:** Rayon parallel map over changed files → tree-sitter parse → entity list. CPU-bound, uses `rayon::spawn`.
3. **Entity extraction stage:** Walk AST, extract entities with qualified names, LOC ranges. Concurrent with parsing via bounded channel.
4. **Edge extraction stage:** Detect calls within each file (intra-file edges). Concurrent.
5. **Cross-file resolution stage:** Import-aware name matching against global definition index (built from all entity extraction results). Sequential merge.
6. **Serialization stage:** Sort entities by file, build CSR+CSC arrays, write `.ptg` file. Single-threaded write.

**Incremental reindexing (Salsa-inspired):**
- Model as: `file_content(path) → ast(path) → entities(path) → edges(path)`
- Each stage memoized with content hash. If AST structure unchanged after edit (whitespace-only), downstream stages skip (**early cutoff**).
- On file change: invalidate `file_content(path)`, recompute downstream. Only affected subgraph of the `.ptg` is rebuilt.
- Full `.ptg` rebuild triggered only when >30% of files change (e.g., git branch switch). Otherwise, maintain a **WAL (write-ahead log)** of deltas, periodically compacted into the main `.ptg`.

**OSS dataset generation:**
- Worker pool of 50–100 parallel git clone processes (shallow, `--depth 1`)
- Each repo parsed independently → `.ptg` file generated
- Export pipeline: read `.ptg` → emit Parquet rows (one per entity, one per edge) via `arrow-rs`
- Schema: `(repo_name, entity_id, qualified_name, entity_type, language, loc, signature, file_path, edges_out: list<edge>)`
- Target: **Parquet** with Snappy compression, partitioned by language

**Public interface export:** Traverse CSR, filter entities by `visibility == public`, emit connected subgraph as JGF, DOT, or SARIF.

**Pros:** Maximum query performance (zero-copy mmap). Excellent parallelism. Fine-grained incremental updates. Compact on-disk format. Scales to millions of entities.

**Cons:** Most complex to implement. Custom binary format requires custom debugging tools. No ad-hoc query language — all queries are programmatic. WAL compaction adds operational complexity. Rkyv/bincode dependency for serialization.

### 5.3 Option C — "Graph-Native Server" (protocol-driven with property graph store)

**Philosophy:** Treat the dependency graph as a first-class database with a query protocol. Separate the graph server from the UI via a JSON-RPC protocol (inspired by LSP), enabling multiple clients: Tauri app, CLI, editor plugins, CI pipelines. Use an embedded property graph store optimized for traversal.

**Persistence format:** **redb** (pure Rust embedded KV store) with a property-graph overlay. Entities stored as key-value pairs (`entity_id → EntityRecord`), edges stored in adjacency-list encoding (`(source_id, edge_type, target_id) → EdgeMetadata`). redb provides ACID transactions, copy-on-write B-trees (implicit crash safety), and competitive performance with lmdb.

Alternative: **CozoDB** (Datalog-based embedded relational-graph database) which natively supports graph algorithms (BFS, DFS, PageRank, community detection) via Datalog queries. CozoDB's DataScript-inspired model naturally represents the entity-edge structure.

**Communication protocol — PGSP (Parseltongue Graph Server Protocol):**

```
Transport: JSON-RPC 2.0 over WebSocket (Tauri) or stdio (CLI/editor)

Lifecycle:
  → graph/initialize { projectPath, languages, config }
  ← graph/capabilities { supportedEdgeTypes, maxDepth, languages }
  → graph/index { mode: "full" | "incremental" }
  ← graph/indexProgress { filesTotal, filesDone, entitiesFound, edgesFound }
  ← graph/indexComplete { stats }

Queries:
  → graph/entity { qualifiedName | entityId }
  ← { entity metadata }
  → graph/subgraph { rootId, edgeTypes[], maxDepth, direction: "forward"|"backward"|"both" }
  ← { nodes[], edges[] }
  → graph/blastRadius { entityId, edgeTypes[] }
  ← { affectedEntities[], paths[][] }
  → graph/publicInterface { filePath | folderPath }
  ← { publicEntities[], exportEdges[] }

Mutations:
  ← graph/didChange { uri, changeType: "created"|"modified"|"deleted" }

Export:
  → graph/export { format: "dot"|"jgf"|"sarif"|"parquet", filter }
  ← { data | filePath }
```

**Tauri app architecture:**
- Frontend: **SolidJS** (7KB, fine-grained signals, no VDOM overhead — ideal for frequent graph updates)
- Graph visualization: **Sigma.js + Graphology** for rendering (WebGL, handles 100K+ nodes) with **ELK.js** for hierarchical layout computation in a Web Worker
- IPC: The Tauri Rust backend spawns the PGSP graph server as a managed process. Frontend communicates via Tauri commands that proxy to the PGSP server. The PGSP protocol also enables standalone CLI usage and editor plugin integration.

**Graph generation pipeline:**
Same tree-sitter parsing front-end as Options A/B. The difference is in storage:
1. Parse → extract entities → transactional batch INSERT into redb/CozoDB
2. Edge extraction → batch INSERT edges
3. Cross-file resolution runs as a Datalog query (CozoDB) or programmatic graph walk (redb)
4. Index stored as redb directory or CozoDB `.db` file

**Incremental reindexing:**
- File-level granularity with transactional batch updates
- On file change: begin transaction → delete entities/edges for file → re-parse → insert new entities/edges → commit
- redb's copy-on-write B-trees provide implicit crash safety (no explicit WAL needed)
- CozoDB's Datalog rules can express incremental edge re-derivation declaratively

**OSS dataset generation:**
- PGSP server runs in headless/batch mode
- CLI: `parseltongue serve --batch --repos-file list.txt --export parquet`
- Each repo indexed → PGSP `graph/export` command generates Parquet output
- Arrow Flight protocol (optional) for streaming large exports to remote storage

**Public interface export:** PGSP `graph/publicInterface` query → returns subgraph. Export via `graph/export` in DOT, JGF, SARIF, or Parquet.

**Pros:** Clean separation of concerns. Protocol enables multiple clients (Tauri, CLI, editor, CI). Transactional updates with crash safety. CozoDB option provides native graph algorithms. Extensible to editor plugins.

**Cons:** Protocol overhead for local desktop use. redb's property graph overlay requires manual implementation. CozoDB is less proven at scale than SQLite. More operational complexity (server process management).

### 5.4 Decision matrix

| Dimension | Option A: Monolithic SQLite | Option B: Stream Pipeline CSR | Option C: Graph-Native PGSP |
|-----------|---------------------------|------------------------------|----------------------------|
| **Implementation complexity** | ★☆☆ Low | ★★★ High | ★★☆ Medium |
| **Query performance (10K entities)** | ★★★ Excellent | ★★★ Excellent | ★★★ Excellent |
| **Query performance (1M entities)** | ★☆☆ Degraded | ★★★ Excellent | ★★☆ Good |
| **Incremental update speed** | ★★☆ File-level | ★★★ Entity-level | ★★☆ File-level |
| **Ad-hoc query capability** | ★★★ Full SQL | ★☆☆ Programmatic only | ★★☆ Datalog/Protocol |
| **Multi-client support** | ★☆☆ Tauri only | ★☆☆ Tauri only | ★★★ Protocol-native |
| **Crash safety** | ★★★ SQLite WAL | ★★☆ Custom WAL | ★★★ CoW B-trees |
| **OSS dataset export** | ★★☆ SQL → Parquet | ★★★ Direct mmap → Parquet | ★★☆ Protocol → Parquet |
| **Debugging / inspectability** | ★★★ DB Browser | ★☆☆ Custom tools | ★★☆ Protocol inspection |
| **Precedent** | Sourcetrail | rust-analyzer | LSP ecosystem |
| **Recommended for** | MVP / small-medium projects | Large codebases / performance | Extensible toolchain |

**[WIP] Recommendation:** Start with **Option A** for MVP (fastest time-to-value, proven model). Migrate to **Option B's CSR format** for the query layer once scale demands it, keeping SQLite as the write-path database. Adopt **Option C's PGSP protocol** as the external API when editor plugin and CI integration are prioritized. This staged approach avoids premature optimization while preserving a migration path.

---

## 6. Tauri application design

### 6.1 Technology stack

| Layer | Recommended | Rationale |
|-------|------------|-----------|
| Desktop framework | **Tauri v2** | Rust backend, small binary, cross-platform, active development |
| Frontend framework | **SolidJS** (Option C) or **Svelte** (Option A) | Minimal bundle, fine-grained reactivity, no VDOM overhead |
| Graph visualization | **Cytoscape.js** + **ELK.js** (Web Worker) | Best-in-class compound graph support, hierarchical layout, rich interaction |
| Large graph fallback | **Sigma.js + Graphology** | WebGL rendering handles 100K+ nodes when Cytoscape's Canvas renderer saturates |
| Build tool | **Vite** | Fast HMR, Tauri template support |

### 6.2 IPC patterns

**Commands** (request/response): All graph queries — entity lookup, subgraph extraction, blast radius, public interface export. Registered via `tauri::generate_handler![]`, invoked from JS via `invoke('command_name', { args })`.

**Channels** (streaming): Indexing progress (files parsed, entities found, edges extracted, current file). The Rust backend creates a `Channel<IndexEvent>` parameter; the frontend passes a callback channel.

**Events** (broadcast): File watcher notifications (`graph/fileChanged`), index status updates, error broadcasts. Emitted via `app.emit()`.

**Long-running task pattern:**
```rust
#[tauri::command]
async fn start_indexing(
    path: String,
    on_progress: Channel<IndexEvent>,
    state: State<'_, AppState>,
) -> Result<IndexResult, String> {
    let cancel_token = CancellationToken::new();
    state.set_cancel_token(cancel_token.clone());

    tauri::async_runtime::spawn(async move {
        let walker = WalkDir::new(&path);
        for (i, entry) in walker.enumerate() {
            if cancel_token.is_cancelled() {
                on_progress.send(IndexEvent::Cancelled).ok();
                return;
            }
            // parse file, extract entities/edges...
            on_progress.send(IndexEvent::Progress {
                files_done: i,
                files_total: total,
                current_file: entry.path().display().to_string(),
            }).ok();
        }
        on_progress.send(IndexEvent::Complete { stats }).ok();
    });
    Ok(IndexResult::Started)
}
```

### 6.3 Graph visualization strategy

The graph UI follows Sourcetrail's proven UX pattern: **one-level dependency view by default**, centered on a selected entity. Users see direct dependencies without information overload. Expand/collapse enables drill-down.

**Level-of-detail rendering:**
- Zoomed out (project level): folders as large colored regions, files as dots, no labels
- Mid zoom (folder level): files as labeled nodes, entity count shown, edges between files
- Zoomed in (file level): entities as labeled nodes with kind icons, call edges between entities
- Full zoom (entity level): signature preview, LOC count, edge labels

**ELK.js layout in Web Worker:** Layout computation is CPU-intensive. ELK.js runs in a dedicated Web Worker to avoid blocking the UI thread. The worker receives the graph JSON, computes positions using ELK Layered (Sugiyama-style for hierarchical display) or ELK Force (for organic exploration), and returns positioned nodes/edges.

**Compound graphs:** Cytoscape.js natively supports compound nodes — folders contain files, files contain entities. CoSE-Bilkent and fcose layout algorithms handle compound graphs. This maps directly to Parseltongue's containment hierarchy.

---

## 7. OSS dataset generation pipeline

### 7.1 Existing datasets landscape

| Dataset | Contains Code? | Size | Languages | Access | Best For |
|---------|---------------|------|-----------|--------|----------|
| **The Stack v2** | Yes (full files) | 67.5 TB | 600+ | HuggingFace (agreement required) | Starting point for code |
| **Software Heritage** | Yes (full archive) | 350M+ repos | All | REST API (rate-limited) | Comprehensive archive |
| **BigQuery GitHub** | Yes (file contents) | 1.5+ TB | All | SQL queries (1TB/mo free) | Queryable exploration |
| **GH Archive** | No (events only) | Billions of events | N/A | HTTP / BigQuery | Repo metadata, stars, forks |
| **CodeSearchNet** | Yes (function-level) | ~20 GB | 6 | S3 / HuggingFace | NL-code pairs |

None of these datasets contain **structural dependency graph data**. This is Parseltongue's unique contribution: a dataset of entity-edge graphs, not raw code.

### 7.2 Crawling pipeline

**Repo selection strategy:**
1. Query BigQuery GitHub dataset or GH Archive for repos with >10 stars, recent activity (commits within 2 years), permissive license (MIT, Apache-2.0, BSD), primary language in supported set
2. Deduplicate forks using GitHub's `fork` flag
3. Balance across languages: target ~10K repos per language for initial dataset
4. Stratify by repo size: small (<1K LOC), medium (1K–100K), large (>100K)

**Clone and parse:**
```
for repo in selected_repos:
    git clone --depth 1 --single-branch {repo.url} /tmp/{repo.name}
    parseltongue index --path /tmp/{repo.name} --export parquet --output /data/{repo.name}/
    rm -rf /tmp/{repo.name}
```
Shallow clone (`--depth 1`) minimizes bandwidth. Each repo takes ~5–30 seconds to clone and parse (tree-sitter processes 1000+ files/sec).

**Parallel execution:** 50–100 worker processes, rate-limited to 10–15 git clones/sec to avoid GitHub throttling. Git clone operations are not subject to GitHub API rate limits but can trigger dynamic throttling under heavy load.

### 7.3 Dataset schema

**Format:** Parquet with Snappy compression, partitioned by language.

**Entity table (`entities.parquet`):**

| Column | Type | Description |
|--------|------|-------------|
| `repo_name` | STRING | e.g., `rust-lang/rust` |
| `entity_id` | STRING | SHA-256 of `repo_name + qualified_name + file_path` |
| `qualified_name` | STRING | e.g., `std::collections::HashMap::insert` |
| `entity_type` | STRING | `function`, `class`, `struct`, `trait`, `import`, etc. |
| `language` | STRING | `rust`, `python`, `java`, etc. |
| `loc` | INT32 | Lines of code |
| `wc` | INT32 | Word count (bytes) |
| `start_line` | INT32 | |
| `end_line` | INT32 | |
| `file_path` | STRING | Relative to repo root |
| `visibility` | STRING | `public`, `private`, `protected`, `internal` |
| `signature` | STRING | Public interface signature (first line / declaration) |
| `is_test` | BOOLEAN | Whether this entity is a test |

**Edge table (`edges.parquet`):**

| Column | Type | Description |
|--------|------|-------------|
| `repo_name` | STRING | |
| `source_id` | STRING | Entity ID of source |
| `target_id` | STRING | Entity ID of target |
| `edge_type` | STRING | `calls`, `imports`, `contains`, `implements`, `extends` |
| `confidence` | STRING | `high`, `ambiguous` |

**Storage estimation:**

| Scale | Repos | Est. Entities | Est. Edges | Entity Parquet | Edge Parquet | Total |
|-------|-------|--------------|-----------|----------------|-------------|-------|
| Small | 10K | ~50M | ~200M | ~5 GB | ~8 GB | ~13 GB |
| Medium | 100K | ~500M | ~2B | ~50 GB | ~80 GB | ~130 GB |
| Large | 1M | ~5B | ~20B | ~500 GB | ~800 GB | ~1.3 TB |

### 7.4 Dataset quality controls

- **Parse accuracy threshold:** Repos with <80% parse accuracy (by bytes) are flagged and excluded from the training split
- **WC invariant verification:** Every repo in the dataset must pass the file-level and folder-level WC invariants
- **Deduplication:** Near-duplicate detection across repos using MinHash/LSH on entity signatures
- **License compliance:** Only repos with detected permissive licenses (via ScanCode toolkit). Opt-out mechanism for maintainers.

---

## 8. Public interface dependency graph export

The public interface graph is a **filtered subgraph** containing only entities with `visibility = "public"` (or language-equivalent: `pub` in Rust, `public` in Java/C++/TS, module-level in Python/Go/JS, no leading underscore convention) and the edges between them.

**Export formats supported:**

| Format | Use Case | MIME Type |
|--------|----------|-----------|
| **JSON Graph Format (JGF)** | Web UI, API consumption, Cytoscape.js import | `application/vnd.jgf+json` |
| **DOT (Graphviz)** | Static visualization, documentation | `text/vnd.graphviz` |
| **SARIF** | CI/CD integration, GitHub Code Scanning | `application/sarif+json` |
| **Parquet** | ML training, batch analytics | `application/x-parquet` |

**JGF output structure:**
```json
{
  "graph": {
    "label": "public-interface",
    "directed": true,
    "metadata": { "project": "...", "generated_at": "...", "parseltongue_version": "..." },
    "nodes": {
      "entity_id_1": { "label": "HashMap::insert", "metadata": { "kind": "function", "loc": 45, "file": "src/collections/hash_map.rs" }},
      ...
    },
    "edges": [
      { "source": "entity_id_1", "target": "entity_id_2", "relation": "calls", "metadata": { "confidence": "high" }},
      ...
    ]
  }
}
```

---

## 9. Existing tools — lessons learned

Research into six existing code analysis tools informs Parseltongue's design:

**Sourcetrail** (discontinued 2021) used Clang for C++, Eclipse JDT for Java, and Jedi for Python. Its SQLite-backed `.srctrldb` format and "ambiguous edge" visualization concept (dashed lines for uncertain references) are directly applicable. The one-level dependency view centered on a selected symbol is the gold standard for graph UX. Its key limitation was requiring compiler infrastructure per language.

**GitHub stack-graphs** built on tree-sitter-graph (TSG) to achieve file-incremental name resolution without a compiler. It was **archived in September 2025** after GitHub unshipped Precise Code Navigation. A developer who worked on the project confirmed that TSG was "the wrong choice for stack graphs" due to the difficulty of specifying language semantics in a declarative DSL. The lesson: **don't try to fully replicate compiler semantics in tree-sitter rules**. Simple heuristics with ambiguity markers outperform complex declarative specifications.

**Kythe** (Google) provides the most rigorous graph schema: VNames (5-tuple unique identifiers), anchors (source locations), and a rich edge vocabulary (`defines/binding`, `ref`, `childof`, `extends`, `satisfies`, `overrides`). Parseltongue adopts this edge vocabulary but avoids Kythe's requirement for build-system integration.

**SCIP** (Sourcegraph) demonstrates the value of human-readable symbol identifiers (e.g., `scip-typescript npm @scope/pkg 1.0.0 src/File.ts/className.methodName().`) over opaque numeric IDs. Its "transmission format, not storage format" design philosophy and Protobuf schema are models for Parseltongue's export format.

**Universal Ctags** supports 173+ languages but produces only flat tag lists with no edges or cross-file relationships. Its entity "kind" taxonomy (function, class, method, variable, module, namespace, interface, struct, enum) is a well-established vocabulary that Parseltongue aligns with.

**tree-sitter-tags** provides the most directly useful resource: `tags.scm` query files that define entity extraction patterns (`@definition.function`, `@definition.class`, `@reference.call`) for many languages. Parseltongue uses these patterns as the starting point for entity extraction, layering cross-file resolution and edge construction on top.

---

## 10. Incremental reindexing specification

### 10.1 File change detection

The `notify` crate (v7+, used by rust-analyzer, Zed, Deno, cargo-watch) provides cross-platform file system watching via `inotify` (Linux), `FSEvents` (macOS), and `ReadDirectoryChangesW` (Windows). Events include `Create`, `Modify` (data/metadata/rename), and `Remove`, each with full file paths.

**Debouncing:** `notify-debouncer-full` coalesces events over a configurable window (**1 second** recommended for code editing). Rename events are correlated across from/to paths. Ignore patterns exclude `.git/`, `node_modules/`, `target/`, `build/`, and other build artifact directories.

**Linux `inotify` caveat:** Default `max_user_watches` is 8192. For large monorepos, this must be increased (`sysctl fs.inotify.max_user_watches=524288`). The Tauri app should detect this limit and warn users.

### 10.2 Tree-sitter incremental parsing

Tree-sitter's `ts_tree_edit()` + re-parse API enables sub-millisecond re-parsing of edited files. After applying `TSInputEdit` (describing the byte range of the change), re-parsing with the old tree produces a new tree that **shares structure** with the old tree via copy-on-write. `ts_tree_get_changed_ranges()` returns only the ranges where the AST structure actually changed — if whitespace or comments changed without affecting structure, no entities need updating.

### 10.3 Graph update strategy

**File-level invalidation** (Options A and C):
1. On file change notification → re-parse file with tree-sitter
2. Compare new entity list against old entity list (by qualified name + content hash)
3. DELETE removed entities and their edges
4. INSERT new/modified entities
5. Re-resolve cross-file edges: if the changed file exports different symbols, all files importing from it need edge re-resolution

**Entity-level invalidation with early cutoff** (Option B):
1. On file change → apply `TSInputEdit` → incremental re-parse
2. Get changed ranges → only re-extract entities in changed ranges
3. Hash each entity's AST subtree. If hash unchanged → skip downstream edge re-extraction (early cutoff, Salsa pattern)
4. For changed entities: remove old edges, re-extract calls, re-resolve cross-file references

---

## 11. Open questions and WIP items

| ID | Question | Status | Priority |
|----|----------|--------|----------|
| Q1 | Should sibling edges be materialized or derived at query time? | **Leaning: derived** | P2 |
| Q2 | How to handle Rust `macro_rules!` expansions that generate entities? | **WIP** — tree-sitter sees the macro invocation, not the expansion | P1 |
| Q3 | How to detect Python `@property` decorated methods as public interface? | **WIP** — check `decorated_definition` wrapping `function_definition` | P2 |
| Q4 | Should `entity_type = "gap"` entities be stored or computed? | **Leaning: computed** (stored only for WC invariant verification) | P2 |
| Q5 | What confidence threshold for cross-file edges should trigger "ambiguous" marking? | **WIP** — need empirical data from OSS dataset | P1 |
| Q6 | How to handle C/C++ header files included by multiple `.c` files? | **WIP** — each `#include` creates an import edge; header entities shared | P1 |
| Q7 | How to handle Go's implicit interface satisfaction? | **Decision: out of scope** — cannot detect without type checker | P1 |
| Q8 | Final file extension for graph format — `.ptg` vs `.ptdb` vs other? | `.ptg` for binary, `.ptdb` for SQLite | P3 |
| Q9 | TypeScript `declare module` and `.d.ts` files — entity types? | **WIP** — treat as `ambient_declaration` entities | P2 |
| Q10 | CozoDB vs redb vs SQLite for Option C — which to prototype first? | **WIP** — depends on Datalog query needs | P1 |
| Q11 | Should the PGSP protocol (Option C) be a Tauri plugin or standalone binary? | **WIP** — Tauri plugin reduces deployment complexity | P2 |
| Q12 | How to handle monorepos with mixed languages? | Language detected per-file; graph spans all languages | P1 |

---

## 12. Appendix: tree-sitter query patterns for entity and call extraction

### A.1 Entity extraction queries (representative examples)

**Rust — functions, structs, traits, impl blocks:**
```scheme
;; Functions
(function_item name: (identifier) @name) @definition.function

;; Structs
(struct_item name: (type_identifier) @name) @definition.struct

;; Traits
(trait_item name: (type_identifier) @name) @definition.interface

;; Impl blocks (extract trait and type)
(impl_item trait: (type_identifier) @trait type: (type_identifier) @type) @definition.implementation
(impl_item type: (type_identifier) @type) @definition.implementation

;; Tests
(attribute_item (attribute (identifier) @attr (#eq? @attr "test")))
```

**Python — functions, classes, imports:**
```scheme
(function_definition name: (identifier) @name) @definition.function
(class_definition name: (identifier) @name) @definition.class
(import_statement) @definition.import
(import_from_statement module_name: (dotted_name) @module) @definition.import
(function_definition name: (identifier) @name (#match? @name "^test_")) @definition.test
```

**Java — classes, methods, interfaces:**
```scheme
(class_declaration name: (identifier) @name) @definition.class
(method_declaration name: (identifier) @name) @definition.method
(interface_declaration name: (identifier) @name) @definition.interface
(constructor_declaration name: (identifier) @name) @definition.method
(method_declaration
  (modifiers (marker_annotation name: (identifier) @ann (#eq? @ann "Test")))
  name: (identifier) @name) @definition.test
```

### A.2 Call extraction queries (representative examples)

**Universal call pattern (JS/TS/Rust/C/C++/Go):**
```scheme
;; Direct function call
(call_expression function: (identifier) @callee) @reference.call

;; Method call
(call_expression function: (member_expression property: (property_identifier) @callee)) @reference.call
;; (field_expression in Rust/C++; selector_expression in Go; attribute in Python)

;; Constructor
(new_expression constructor: (identifier) @callee) @reference.call
```

**Python call:**
```scheme
(call function: (identifier) @callee) @reference.call
(call function: (attribute attribute: (identifier) @callee)) @reference.call
```

**Java method invocation:**
```scheme
(method_invocation name: (identifier) @callee) @reference.call
(object_creation_expression type: (type_identifier) @callee) @reference.call
```

---

## 13. Glossary

| Term | Definition |
|------|-----------|
| **ISGL1** | Interface Signature Graph Level 1 — entity representation including public interface signature and LOC range |
| **MECE** | Mutually Exclusive, Cumulatively Exhaustive — every byte belongs to exactly one entity |
| **CSR** | Compressed Sparse Row — graph serialization format using offset and target arrays |
| **CSC** | Compressed Sparse Column — transpose of CSR for efficient incoming-edge queries |
| **PGSP** | Parseltongue Graph Server Protocol — JSON-RPC 2.0 protocol for graph queries |
| **WC** | Word Count — byte-level accounting measure for entity/file/folder sizes |
| **LOC** | Lines of Code — line-level measure of entity size |
| **JGF** | JSON Graph Format — open standard for graph serialization |
| **ELK** | Eclipse Layout Kernel — hierarchical graph layout engine |
| **LLD** | Low-Level Design — detailed technical design documentation |
| **`.ptg`** | Parseltongue Graph — custom binary graph file (Option B) |
| **`.ptdb`** | Parseltongue Database — SQLite-backed graph file (Option A) |

---

*End of document. This PRD is a living document. All sections marked [WIP] require further investigation and team discussion before implementation begins.*

