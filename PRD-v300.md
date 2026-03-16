# Parseltongue: Rust LLM Companion

> Grep returns files. Parseltongue returns understanding.
> A minimalistic, well-verified proof of Rust craft.

# Primary Key Format

Uniform for ALL entities: `path:start_line:end_line`

    src/auth/:-1:-1                    → folder
    src/auth/service.rs:0:0            → file
    src/auth/service.rs:8:25           → code span (fn login)

Sentinels: `-1:-1` = folder, `0:0` = file, `N:M` (N >= 1) = code span.

# Coverage Model (grounded in apache/iggy: 2712 files, 775 dirs, 379K lines)

Every byte in every file is accounted for. Zero gaps. 100% coverage by construction.

## The Rule: Every Entity Has a `wc` (Word Count)

Every entity — searchable or not — stores a `wc` field (word count of its source text).
For any parsable file: `file.total_wc = sum(entity.wc for all entities in that file)`.
No gaps. No unaccounted words. This is how we track coverage and compute token economics.

Tree-sitter's root node children cover the entire file. We classify ALL of them (not just
code declarations). Whitespace gaps between root children are computed and accounted for.

## File Categories (what happens to each file after .gitignore)

    Category              iggy count    What Parseltongue does
    --------              ----------    ----------------------
    CODE (tree-sitter)    2143 files    Parse ALL root children → entities with wc. Searchable.
      .rs                 1237            208K lines (Rust — also gets Layer 3 enrichment)
      .java               320             36K lines
      .ts                 224             18K lines
      .cs                 220             25K lines
      .go                 127             17K lines
      .py                 11              2K lines
      .js                 4               177 lines
      .svelte             70              7K lines (if we add tree-sitter-svelte)

    RUST CONFIG           83 files      Parse as TOML → dependency/package_meta/config_section entities
      Cargo.toml          83              5K lines
      (build.rs)          2               (counted in .rs above)

    DATA/CONFIG           132 files     File entity + hash + total_wc. NOT parsed.
      .toml (non-Cargo)   0
      .yml/.yaml          65              12K lines
      .json               63              14K lines
      .xml                3               238 lines
      .proto              1               41 lines

    DOCUMENTATION         56 files      File entity + hash + total_wc. NOT parsed.
      .md                 52              8K lines
      .txt                4               61 lines

    SCRIPTS/TOOLING       66 files      File entity + hash + total_wc. NOT parsed.
      .sh                 31              4K lines
      Dockerfile          8
      justfile            2
      .http               3               665 lines
      .editorconfig       3
      .gitignore          10
      .dockerignore       6
      other (no ext)      ~20

    BUILD SYSTEM          49 files      File entity + hash + total_wc. NOT parsed.
      .csproj/.sln/.props 23
      .kts                15
      .properties         7
      .bazel              2
      other               2

    BINARY/OPAQUE         56 files      File entity + hash. No wc (binary).
      .png                34
      .svg                12
      .pem                3
      .lock               7               (16K lines but generated, not useful)

## The Coverage Equation

For any parsable file, tree-sitter gives us ALL root node children. We classify every one:

    file.total_wc = sum(entity.wc) + whitespace_wc

    Where entities include ALL root children:
      code entities   (function, struct, impl, ...) → searchable, snippet stored
      imports         (use_declaration, ...)         → wc counted, drives edges
      doc_comments    (///, //!, /** */)             → wc counted, folded into adjacent entity FTS
      comments        (// plain, /* block */)        → wc counted only
      whitespace      (gaps between root children)   → wc counted only

This gives us per-file, per-folder, and per-repo breakdowns:

    Per file:
      SELECT entity_type, SUM(wc) as words,
             ROUND(SUM(wc) * 100.0 / file.total_wc, 1) as pct
      FROM entities WHERE file = 'src/auth/service.rs'
      GROUP BY entity_type;

      -- function     340 words   56.7%    ← searchable
      -- struct       120 words   20.0%    ← searchable
      -- import        30 words    5.0%    ← graph edges
      -- doc_comment   50 words    8.3%    ← searchable via code spans
      -- comment       30 words    5.0%    ← coverage only
      -- whitespace    30 words    5.0%    ← coverage only
      -- TOTAL        600 words  100.0%

    Per repo (apache/iggy scale):
      Searchable code:    ~800K words (~1M tokens)    ← FTS indexes this
      Doc comments:       ~120K words (~156K tokens)   ← searchable via code spans
      Imports:             ~80K words                   ← drives graph edges only
      Plain comments:     ~100K words                   ← coverage only
      Whitespace:         ~100K words                   ← coverage only
      Total:             ~1.2M words                    ← 100% accounted for

# Searchability Rule

- SEARCHABLE: Code entities (function, struct, impl, ...) go into FTS. Name + signature + snippet + doc_comment.
- SEARCHABLE VIA CODE SPANS: Doc comments (///, //!, /** */) folded into adjacent code span's
  `doc_comment` field. Searchable through FTS but not separate blobs.
- NOT SEARCHABLE: Folders, files, imports, plain comments, whitespace — graph/coverage only.
- NOT STORED: Full file content is NEVER stored. Only parsed snippets.
- ALL COUNTED: Every entity has `wc`. Sum of all entity wc = file total wc. Zero gaps.

# Entity Taxonomy (`entity_type` column — 100% file coverage)

Every entity has: `pk` (path:start_line:end_line), `entity_type`, `wc` (word count).
For parsable files, ALL tree-sitter root children get an entity_type. No bytes left uncounted.
Only module-level declarations become entities. Nested items (closures, inner fns, structs
inside function bodies) are part of their parent entity's snippet — not separate entities.

## A. Structural Entities (graph-only, not searchable)

    entity_type       Example PK                     Description
    -----------       ----------                     -----------
    folder            src/auth/:-1:-1                Every directory in the tree
    file_parsable     src/auth/service.rs:0:0        Tree-sitter can parse. Stores total_wc + hash.
    file_unparsable   README.md:0:0                  Can't parse. Stores total_wc + hash.
    file_config       Cargo.toml:0:0                 Rust config only. Parsed as TOML.

All files store: file_hash (SHA-256), total_wc.
For parsable files: total_wc = sum(child entity wc). Verified on save.
Other languages' config files (package.json, pyproject.toml) = file_unparsable.

## B. Code Entities (searchable, FTS-indexed: name + signature + doc_comment)

Module-level declarations extracted by tree-sitter. Each stores snippet + wc.

    entity_type   Example PK                     tree-sitter node types
    -----------   ----------                     ----------------------
    function      src/main.rs:10:25              function_item, function_definition, function_declaration
    method        src/auth.rs:30:45              (inside impl/class via splitNodes)
    struct        src/model.rs:5:15              struct_item, struct_specifier
    class         src/app.py:1:50                class_definition, class_declaration
    enum          src/status.rs:3:12             enum_item, enum_declaration, enum_specifier
    trait         src/auth.rs:1:20               trait_item, trait_definition, trait_declaration
    interface     src/api.ts:5:30                interface_declaration
    impl          src/auth.rs:22:60              impl_item
    type_alias    src/types.rs:3:3               type_item, type_alias_declaration, type_definition
    constant      src/config.rs:1:1              const_item, const_declaration
    static        src/global.rs:5:5              static_item
    macro         src/macros.rs:1:20             macro_definition, preproc_def, preproc_function_def
    module        src/lib.rs:1:1                 mod_item, module
    variable      src/app.js:1:1                 lexical_declaration, variable_declaration, val_definition
    constructor   src/App.java:10:20             constructor_declaration
    namespace     src/lib.cpp:1:50               namespace_definition, namespace_declaration
    record        src/User.java:1:10             record_declaration
    object        src/App.scala:1:20             object_definition

Tests: not a separate entity_type — `is_test=true` flag on function/method entities.

## C. Non-Code Entities (not searchable, but counted for 100% wc coverage)

These are tree-sitter root children that are NOT code declarations.
They exist so that sum(entity.wc) = file.total_wc with zero gaps.

    entity_type     Example PK                     tree-sitter node types
    -----------     ----------                     ----------------------
    import          src/main.rs:1:3                use_declaration, import_statement, extern_crate_item
    doc_comment     src/auth.rs:7:9                line_comment (///), block_comment (/** */), inner docs (//!)
    comment         src/main.rs:1:1                line_comment (//), block_comment (/* */)
    attribute       src/auth.rs:6:6                attribute_item (#[...]), decorator (@...)
    whitespace      (computed, not a TS node)       gaps between root children

    import:       wc counted + drives dependency graph edges. Not FTS-indexed.
    doc_comment:  wc counted + text folded into adjacent code entity's `doc_comment` FTS field.
                  Module doc comments (//!) folded into file_parsable entity's `doc_comment`.
    comment:      wc counted only. Not indexed. Not stored as blob.
    attribute:    wc counted + attached to next code entity (for is_test detection, etc.)
    whitespace:   wc counted only. Computed as: file.total_wc - sum(all other entity wc).

## D. Rust Config Span Entities (Cargo.toml only)

    entity_type     Example PK                     Description
    -----------     ----------                     -----------
    dependency      Cargo.toml:5:5                 A crate dependency declaration
    package_meta    Cargo.toml:1:4                 Package name, version, edition
    config_section  Cargo.toml:10:15               Named section ([features], [workspace], etc.)

## E. Rust Compiler Enrichment (Layer 3 — extra columns, not new entity_types)

For .rs code entities, same pk, same row, more columns filled in:

    rustc_scope     tcx.def_path_str()     "crate::auth::service::login"
    rustc_sig       tcx.fn_sig()           "fn(&Credentials) -> Result<Token>"
    visibility      tcx.visibility()       "pub(crate)"
    mir_calls       tcx.optimized_mir()    ["crate::db::lookup", ...]
    trait_impls     tcx.all_impls()        [...]

## Entity Type Summary

    Searchable (FTS):      18 code entity_types (function through object)
    Graph-only:            4 structural (folder, file_parsable, file_unparsable, file_config)
                           1 import (drives edges)
                           1 attribute (attached to next entity)
    Coverage-only:         3 (doc_comment, comment, whitespace)
    Rust config:           3 (dependency, package_meta, config_section)
    Rust enrichment:       0 new types (extra columns on existing code entities)
    -------
    Total distinct:        30 entity_types

---

# Tree-Sitter API Reference

## Core Node API (what we get per node)

Every tree-sitter node exposes these properties. This is the raw material for entity extraction.
Source: tree-sitter 0.25 C/Rust API (verified via Context7 + cargo cache node-types.json).

    Property/Method              Returns              Used for
    ---------------              -------              --------
    node.kind()                  &str                 entity_type classification
    node.start_byte()            u32                  wc = end_byte - start_byte (→ byte count)
    node.end_byte()              u32                  wc calculation
    node.start_position()        { row, column }      start_line (row + 1, 1-based)
    node.end_position()          { row, column }      end_line (row + 1, 1-based)
    node.child_by_field_name()   Option<Node>         extract "name", "type", "trait" fields
    node.children()              Iterator<Node>       walk all children
    node.named_children()        Iterator<Node>       skip anonymous nodes (punctuation)
    node.parent()                Option<Node>         walk upward
    node.next_sibling()          Option<Node>         find adjacent doc_comments
    node.prev_sibling()          Option<Node>         find adjacent doc_comments
    node.is_named()              bool                 skip anonymous (keywords, brackets)
    node.text                    &str (via bytes)     snippet extraction

## Language Enumeration API (runtime node type discovery)

We do NOT need to hardcode node types. The Language API lets us enumerate at runtime:

    language.node_kind_count()            → total number of node kinds
    language.node_kind_for_id(id: u16)    → name string for each id
    language.node_kind_is_named(id: u16)  → skip anonymous nodes
    language.field_count()                → number of named fields
    language.field_name_for_id(id: u16)   → field name by id

This means: at build time or first-run, we can generate the complete mapping table
for every grammar version we ship. No manual maintenance.

## Root Node Names Per Language

    Language       Root node type         Grammar crate in Cargo.toml
    --------       --------------         ---------------------------
    Rust           source_file            tree-sitter-rust 0.23
    Python         module                 tree-sitter-python 0.25
    JavaScript     program                tree-sitter-javascript 0.25
    TypeScript     program                tree-sitter-typescript 0.23
    Java           program                tree-sitter-java 0.23
    Go             source_file            tree-sitter-go 0.25
    C              translation_unit       tree-sitter-c 0.24
    C++            translation_unit       tree-sitter-cpp 0.23
    C#             compilation_unit       tree-sitter-c-sharp 0.23
    Ruby           program                tree-sitter-ruby 0.23
    Scala          compilation_unit       tree-sitter-scala 0.24
    PHP            program                tree-sitter-php 0.24
    Swift          source_file            tree-sitter-swift 0.7
    Kotlin         source_file            tree-sitter-kotlin 0.3

---

# Tree-Sitter Node Type → entity_type Mapping (per language)

Every concrete node type that can appear as a direct child of the root node,
mapped to our entity_type. Extracted from node-types.json files in cargo cache.

Key: S = searchable (FTS), G = graph edges, C = coverage only, A = attach to next entity.

## Rust (root: source_file)

    tree-sitter node type        entity_type    S/G/C/A   notes
    -------------------------    -----------    -------   -----
    function_item                function       S
    function_signature_item      function       S         trait fn signatures
    struct_item                  struct         S
    enum_item                    enum           S
    impl_item                   impl           S         splitNodes → methods
    trait_item                   trait          S         splitNodes → methods
    type_item                    type_alias     S
    const_item                   constant       S
    static_item                  static         S
    macro_definition             macro          S
    mod_item                     module         S         splitNodes
    union_item                   struct         S         treat as struct
    foreign_mod_item             module         S         extern blocks
    associated_type              type_alias     S
    macro_invocation             macro          S         top-level macro calls
    use_declaration              import         G         dependency edges
    extern_crate_declaration     import         G         dependency edges
    attribute_item               attribute      A         #[...], attach to next
    inner_attribute_item         attribute      A         #![...], attach to file
    line_comment (///)           doc_comment    C         fold into next entity FTS
    line_comment (//)            comment        C         wc only
    block_comment (/** */)       doc_comment    C         fold into next entity FTS
    block_comment (/* */)        comment        C         wc only
    empty_statement              whitespace     C
    expression_statement         variable       C         rare at top-level
    let_declaration              variable       C         rare at top-level
    shebang                      comment        C

    Comment detection: both /// and // are `line_comment` — inspect first chars to classify.
    Doc markers: //! and /*! are module-level doc_comments → fold into file entity.

## Python (root: module)

    tree-sitter node type        entity_type    S/G/C/A   notes
    -------------------------    -----------    -------   -----
    function_definition          function       S
    class_definition             class          S         splitNodes → methods
    decorated_definition         function/class S         unwrap to inner definition
    type_alias_statement         type_alias     S
    expression_statement         variable       S         top-level X = 5 assignments
    import_statement             import         G
    import_from_statement        import         G
    future_import_statement      import         G
    if_statement                 comment        C         rare at module level
    for_statement                comment        C
    while_statement              comment        C
    try_statement                comment        C
    with_statement               comment        C
    match_statement              comment        C
    assert_statement             comment        C
    pass_statement               comment        C
    return_statement             comment        C
    break_statement              comment        C
    continue_statement           comment        C
    raise_statement              comment        C
    delete_statement             comment        C
    exec_statement               comment        C
    print_statement              comment        C
    global_statement             comment        C
    nonlocal_statement           comment        C
    comment                      comment/doc    C         # vs docstring position

    is_test detection: function name starts with test_ or file in tests/.

## JavaScript (root: program)

    tree-sitter node type            entity_type    S/G/C/A   notes
    -------------------------        -----------    -------   -----
    function_declaration             function       S
    generator_function_declaration   function       S
    class_declaration                class          S         splitNodes → methods
    lexical_declaration              variable       S         const/let at top level
    variable_declaration             variable       S         var at top level
    export_statement                 (unwrap)       S         unwrap to inner declaration
    expression_statement             variable       S         module.exports = ...
    import_statement                 import         G
    if_statement                     comment        C
    for_statement                    comment        C
    for_in_statement                 comment        C
    while_statement                  comment        C
    do_statement                     comment        C
    switch_statement                 comment        C
    try_statement                    comment        C
    with_statement                   comment        C
    return_statement                 comment        C
    throw_statement                  comment        C
    break_statement                  comment        C
    continue_statement               comment        C
    debugger_statement               comment        C
    labeled_statement                comment        C
    statement_block                  comment        C
    empty_statement                  whitespace     C
    comment (/** */)                 doc_comment    C         JSDoc → fold into next entity
    comment (//)                     comment        C
    hash_bang_line                   comment        C         #!/usr/bin/env node

    is_test: inside describe()/it()/test() blocks, or file matches *.test.* / *.spec.*.

## TypeScript (root: program)

    Same as JavaScript, plus:

    tree-sitter node type            entity_type    S/G/C/A   notes
    -------------------------        -----------    -------   -----
    interface_declaration            interface      S
    type_alias_declaration           type_alias     S
    enum_declaration                 enum           S
    abstract_class_declaration       class          S         splitNodes
    using_declaration                variable       S

## Java (root: program)

    tree-sitter node type                entity_type    S/G/C/A   notes
    -------------------------            -----------    -------   -----
    class_declaration                    class          S         splitNodes → methods
    interface_declaration                interface      S         splitNodes
    enum_declaration                     enum           S         splitNodes
    record_declaration                   record         S
    annotation_interface_declaration     interface      S
    import_declaration                   import         G
    package_declaration                  module         S
    block_comment (/** */)               doc_comment    C         Javadoc → fold into next
    block_comment (/* */)                comment        C
    line_comment                         comment        C

    is_test: @Test annotation on method.

## Go (root: source_file)

    tree-sitter node type        entity_type    S/G/C/A   notes
    -------------------------    -----------    -------   -----
    function_declaration         function       S
    method_declaration           method         S         receiver.Type.Name
    type_declaration             (inspect)      S         contains struct/interface/type_alias
    const_declaration            constant       S
    var_declaration              variable       S
    import_declaration           import         G
    package_clause               module         S
    comment                      comment/doc    C         // before func = doc_comment

    type_declaration unwrapping: inspect child type_spec to determine struct vs interface vs type_alias.
    is_test: function starts with Test in *_test.go files.

## C (root: translation_unit)

    tree-sitter node type        entity_type    S/G/C/A   notes
    -------------------------    -----------    -------   -----
    function_definition          function       S
    declaration                  variable       S         top-level vars, externs
    type_definition              type_alias     S         typedef
    struct_specifier             struct         S
    enum_specifier               enum           S
    union_specifier              struct         S         treat as struct
    preproc_def                  macro          S         #define VALUE
    preproc_function_def         macro          S         #define FUNC(x)
    preproc_include              import         G         #include
    preproc_if                   attribute      C
    preproc_ifdef                attribute      C
    preproc_call                 macro          C
    linkage_specification        module         S         extern "C" { }
    comment                      comment        C

## C++ (root: translation_unit)

    Same as C, plus:

    tree-sitter node type            entity_type    S/G/C/A   notes
    -------------------------        -----------    -------   -----
    class_specifier                  class          S         splitNodes → methods
    namespace_definition             namespace      S         splitNodes
    template_declaration             (unwrap)       S         unwrap to inner class/fn
    using_declaration                import         G
    namespace_alias_definition       type_alias     S
    concept_definition               trait          S         C++20 concepts ≈ traits
    alias_declaration                type_alias     S         using X = Y
    static_assert_declaration        comment        C

## C# (root: compilation_unit)

    tree-sitter node type                entity_type    S/G/C/A   notes
    -------------------------            -----------    -------   -----
    class_declaration                    class          S         splitNodes
    interface_declaration                interface      S
    struct_declaration                   struct         S
    enum_declaration                     enum           S
    record_declaration                   record         S
    namespace_declaration                namespace      S         splitNodes
    file_scoped_namespace_declaration    namespace      S
    delegate_declaration                 type_alias     S
    method_declaration                   method         S
    constructor_declaration              constructor    S
    destructor_declaration               method         S
    property_declaration                 variable       S
    field_declaration                    variable       S
    event_declaration                    variable       S
    event_field_declaration              variable       S
    indexer_declaration                  method         S
    operator_declaration                 method         S
    conversion_operator_declaration      method         S
    using_directive                      import         G
    extern_alias_directive               import         G
    global_attribute                     attribute      A
    comment                              comment        C

    is_test: [Test] or [TestMethod] attribute.

## Ruby (root: program)

    tree-sitter node type        entity_type    S/G/C/A   notes
    -------------------------    -----------    -------   -----
    module                       module         S         splitNodes
    class                        class          S         splitNodes
    method                       function       S
    singleton_method             function       S         self.method
    assignment                   variable       S         top-level CONST = ...
    alias                        type_alias     S
    call                         variable       C         top-level method calls (rare)
    begin_block                  comment        C
    end_block                    comment        C
    undef                        comment        C
    comment                      comment/doc    C         # comment (RDoc before def = doc)

## Scala (root: compilation_unit)

    tree-sitter node type        entity_type    S/G/C/A   notes
    -------------------------    -----------    -------   -----
    class_definition             class          S         splitNodes
    object_definition            object         S         splitNodes
    trait_definition             trait          S         splitNodes
    function_definition          function       S
    function_declaration         function       S
    val_definition               constant       S
    val_declaration              constant       S
    var_definition               variable       S
    var_declaration              variable       S
    type_definition              type_alias     S
    enum_definition              enum           S
    given_definition             impl           S         Scala 3 given ≈ Rust impl
    extension_definition         impl           S         Scala 3 extension ≈ Rust impl
    import_declaration           import         G
    export_declaration           import         G
    package_clause               module         S
    package_object               module         S
    block_comment                comment        C
    comment                      comment        C         Scaladoc (/** */) = doc_comment

---

# Tree-Sitter Implementation Notes

## 1. Comment Detection Requires Text Inspection

Tree-sitter does NOT distinguish doc comments from plain comments at the node type level.
Both `///` and `//` parse as `line_comment` in Rust. Both `/** */` and `/* */` parse as
`block_comment`. We must inspect the first characters of the comment text:

    Rust:    /// or //! → doc_comment. // → comment. /** */ or /*! */ → doc_comment.
    Python:  Docstrings are expression_statement containing a string, not comment nodes.
    JS/TS:   /** */ → JSDoc (doc_comment). // and /* */ → comment.
    Java:    /** */ → Javadoc (doc_comment). // and /* */ → comment.
    Go:      // comment immediately before a declaration → doc_comment (by convention).
    Ruby:    # comment before def/class → RDoc (doc_comment, by convention).

## 2. Nodes That Need Unwrapping

Some root children wrap the real entity. We unwrap before classifying:

    export_statement (JS/TS)       → inner is class/function/variable declaration
    decorated_definition (Python)  → inner is function_definition or class_definition
    template_declaration (C++)     → inner is class/function/struct
    type_declaration (Go)          → inner type_spec reveals struct vs interface vs alias

Codemogger already implements all four unwrapping patterns in treesitter.ts.

## 3. splitNodes: One Level Deeper for Large Containers

When an entity exceeds ~150 lines, we split into sub-items (methods within class/impl).
Only one level deep — never recurse into function bodies.

    Language    Split targets
    --------    -------------
    Rust        impl_item, trait_item, mod_item
    Python      class_definition
    JS          class_declaration
    TS          class_declaration, abstract_class_declaration, interface_declaration
    Java        class_declaration, interface_declaration, enum_declaration
    Go          (none — Go has flat top-level declarations)
    C           (none)
    C++         class_specifier, struct_specifier, namespace_definition
    C#          class_declaration, interface_declaration, struct_declaration, namespace_declaration
    Ruby        module, class
    Scala       class_definition, object_definition, trait_definition

Body wrapper nodes to walk into: class_body, declaration_list, field_declaration_list,
body_statement, block (varies by language).

## 4. Module-Level Only Rule

We only extract entities from root node children (+ one splitNodes level).
Anything nested inside a function body is an implementation detail:

    YES: top-level fn, struct, class, impl, trait, module
    YES: methods inside impl/class (via splitNodes — one level deep)
    YES: items inside mod tests { } (module-level within test module)
    NO:  closure inside function body
    NO:  fn nested inside another fn
    NO:  struct/enum inside function body
    NO:  block expression items

This matches codemogger's approach: processNode() only walks tree.rootNode.children,
and splitLargeNode() only goes one level into body wrappers.

## 5. is_test Detection (per language)

    Rust:      #[test] or #[cfg(test)] attribute on function
    Python:    function name starts with test_ or file in tests/
    JS/TS:     inside describe()/it()/test(), or file matches *.test.* / *.spec.*
    Go:        function starts with Test in *_test.go files
    Java:      @Test annotation
    C#:        [Test] or [TestMethod] attribute
    Ruby:      method inside RSpec describe block, or file in spec/
    Scala:     extends FunSuite/FlatSpec, or method annotated with test

---

# v1.6.1 Retrospective: What We Learned

## Two Approaches Compared

v1.6.1 used declarative `.scm` tree-sitter query files (12 languages, ~15 lines each).
Codemogger uses imperative AST walking with data-driven LanguageConfig (~587 lines total).

    Approach              v1.6.1 (.scm queries)       Codemogger (imperative walk)
    --------              ---------------------        ----------------------------
    Code per language     ~15 lines .scm + Rust glue  ~50 lines config (shared walker)
    Total code            12 .scm files + glue         587 lines for 14 languages
    splitNodes            NOT implemented              Built-in (>150 lines → split)
    Comment detection     NOT implemented              Manual text inspection
    Export unwrapping     Nested .scm patterns         Explicit code
    Fuzzy node matching   Exact node type names        type.includes("function")
    Compile-time embed    include_str!() → zero I/O    N/A (WASM runtime)
    Production tested     Internal only                Shipped by Turso team

## What v1.6.1 Got Right (steal these)

1. **FileWordCoverage schema** — had source_word_count, entity_word_count, import_word_count,
   comment_word_count, raw_coverage_pct, effective_coverage_pct. Validates our wc model.
2. **8 dependency edge types** — calls, uses, implements, type_refs, field_access,
   async_await, iterators, generics (from dependency_queries/rust.scm, 180 lines).
3. **Deduplication** — HashSet<(name, line_range)> to handle overlapping query matches.
4. **include_str!() embedding** — compile-time config embedding, zero runtime I/O.

## What v1.6.1 Got Wrong (avoid these)

1. **Key format** — rust:fn:name:__path:T170... breaks on renames. Our path:line:line is stable.
2. **No splitNodes** — large impl blocks became single giant entities.
3. **No doc comment handling** — known gap, never addressed.
4. **CozoDB underutilized** — stored data but didn't use graph engine.
5. **.scm queries are fragile** — grammar updates break exact node type patterns.

## Decision: Data-Driven Approach (like codemogger, in Rust)

Follow codemogger's architecture, not v1.6.1's .scm approach:

    struct LanguageConfig {
        name: &'static str,
        extensions: &'static [&'static str],
        top_level_nodes: &'static [&'static str],
        split_nodes: &'static [&'static str],
    }

Walk root_node.children(), classify via node.kind(), normalize to entity_type.
Add our v3.0 innovations on top: wc tracking, doc_comment folding, 100% coverage.

Reasons:
- ONE shared walker for all languages (not 12 separate .scm files + glue)
- splitNodes, comment detection, export unwrapping need imperative code anyway
- Data-driven config is auditable and extensible
- Proven in production by Turso team
- Fuzzy matching (kind.contains("function")) survives grammar updates

## Reindexing Speed (from codemogger benchmarks)

Codemogger benchmarks on Apple M2:

    Project         Files     Keyword search    Semantic search    ripgrep
    -------         -----     --------------    ---------------    -------
    Turso (Rust)    748       1 ms              35 ms              25 ms
    Bun (Zig)       9,255     2 ms              137 ms             166 ms
    TypeScript      39,298    4 ms              242 ms             1,500 ms
    Kubernetes (Go) 16,668    12 ms             617 ms             731 ms

Key insight: embedding is 97% of codemogger's indexing time. We skip embedding entirely.

    For Parseltongue (no embedding), single-file reindex estimate:
      SHA-256 hash         ~instant
      Tree-sitter parse    10-20ms
      Delete old entities  <5ms (single SQL DELETE)
      Insert new entities  <5ms (batch INSERT)
      Update FTS           <5ms (incremental)
      ─────────────────────────────
      Total:               <50ms per changed file

    Full initial index (748-file Rust project like Turso):
      Without embedding:   ~5-15 seconds (tree-sitter only)
      With embedding:      ~60-120 seconds (97% embedding time)

---

# The Screens

Everything follows from the screens. The screens ARE the product.

---

## Screen 1: First Launch (Empty State)

User downloads .dmg or `brew install parseltongue`. Opens the app.
No login. No account. No server. Privacy-first.

```
┌──────────────────────────────────────────┐
│  Parseltongue                        [—] │
│──────────────────────────────────────────│
│                                          │
│  No codebases yet.                       │
│                                          │
│  [ Browse Folder ]  or drag & drop here  │
│                                          │
│  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │
│  Parseltongue analyzes Rust codebases    │
│  so you (and your LLM) can understand    │
│  them without reading every file.        │
│                                          │
└──────────────────────────────────────────┘
```

User clicks Browse Folder. Native macOS file dialog opens.
They pick ~/code/my-rust-project.

**What happens behind the screen:**
- Tauri native file dialog (already researched in docs/tauri-research)
- No database exists yet. Nothing to configure.

---

## Screen 2: Ingestion Progress

```
┌──────────────────────────────────────────┐
│  Parseltongue                        [—] │
│──────────────────────────────────────────│
│                                          │
│  Analyzing: my-rust-project              │
│  /Users/dev/code/my-rust-project         │
│                                          │
│  [████████░░░░░░░░░░] 42%               │
│                                          │
│  Tree-sitter parsing...     142 files    │
│  Rust compiler analysis...  in progress  │
│  Building graph...          pending      │
│                                          │
│  Found so far:                           │
│    387 entities  ·  1,204 edges          │
│                                          │
└──────────────────────────────────────────┘
```

Two passes:
1. Fast pass (tree-sitter): entities + basic edges for all languages. Seconds.
2. Deep pass (rustc_private): compiler-verified types, real call graph, trait impls. Rust only.

macOS notification when done: "my-rust-project analyzed. 387 entities, 1,204 edges."

**What happens behind the screen:**
- Walk folder tree → create folder entities (Layer 0) and file entities (Layer 1)
- For each parsable file → tree-sitter extracts line-range entities (Layer 2)
- For .rs files → rustc_private adds compiler truth (Layer 3)
- All stored in Turso/libSQL database managed by the app
- Database location is invisible to the user

---

## Screen 3: Home Screen (With Data)

```
┌──────────────────────────────────────────┐
│  Parseltongue                        [—] │
│──────────────────────────────────────────│
│                                          │
│  YOUR CODEBASES                          │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │ my-rust-project           FRESH    │  │
│  │ ~/code/my-rust-project             │  │
│  │ 387 entities · 1,204 edges         │  │
│  │ Last analyzed: 2 min ago           │  │
│  │                                    │  │
│  │ HTTP: localhost:7777  [ Copy URL ] │  │
│  │ [ Re-analyze ]  [ Open Terminal ]  │  │
│  └────────────────────────────────────┘  │
│                                          │
│  [ + Add Another Codebase ]              │
│                                          │
└──────────────────────────────────────────┘
```

- FRESH / STALE badge — compares DB timestamp to git HEAD or file mod times
- Copy URL — one click to get the HTTP endpoint for LLM tools
- DB location is invisible — Tauri manages it
- Each workspace = one codebase + one DB + one HTTP server instance

**What happens behind the screen:**
- Tauri spawns/manages an HTTP server per workspace
- Staleness check: compare stored file hashes to current files on disk
- The HTTP URL is the interface for LLMs and CLI users

---

## Screen 4: The HTTP Query (7-Event Journey begins)

User copies localhost:7777 into Claude Code / Cursor / terminal.

```
curl "http://localhost:7777/query?q=authentication+flow"
```

This is NOT a Tauri screen. This is an HTTP response.
The 7-event journey runs server-side:

**Event 1: QUERY** — the ~7 words arrive

**Event 2: SEARCH** — RRF fusion finds 4 candidates (<10ms)
  - Symbol trie (exact matches)
  - Trigram index (fuzzy matches)
  - Git history (recent edits)

**Event 3: ANCHOR** — BFS upward to public API boundary (<50ms)
  - For private entities: walk callers until a public fn/trait is found
  - For public entities: anchor is itself

**Event 4: CLUSTER** — ego network 1-hop for each anchor (<100ms)
  - Cluster = anchor + callers + callees + implementations
  - Each cluster compressed to ~3000 tokens

---

## Screen 5: The HTTP Response (Cluster Selection)

The HTTP response presents 4 candidate clusters (~200 tokens):

```json
{
  "query": "authentication flow",
  "clusters": [
    {
      "id": 1,
      "label": "API HANDLER",
      "anchor": "src/api/handlers.rs:45:78",
      "name": "login_route",
      "summary": "HTTP endpoint, calls auth::login",
      "entity_count": 5,
      "edge_count": 8
    },
    {
      "id": 2,
      "label": "AUTH TRAIT",
      "anchor": "src/auth/provider.rs:12:35",
      "name": "AuthProvider",
      "summary": "Abstraction, 2 impls: JWT, OAuth",
      "entity_count": 7,
      "edge_count": 12
    },
    {
      "id": 3,
      "label": "MODULE",
      "anchor": "src/auth/",
      "name": "authentication",
      "summary": "Folder with 12 files",
      "entity_count": 24,
      "edge_count": 45
    },
    {
      "id": 4,
      "label": "EXTERNAL",
      "anchor": "src/oauth/client.rs:8:42",
      "name": "oauth",
      "summary": "Third-party integration",
      "entity_count": 3,
      "edge_count": 5
    }
  ],
  "prompt": "Which cluster? [1] [2] [3] [4] [none]"
}
```

LLM or human picks one. Token cost to decide: ~200 tokens.

---

## Screen 6: The Deep Dive Response

User/LLM chose [1]. Full context returned (up to 20k tokens):

```json
{
  "cluster_id": 1,
  "anchor": {
    "location": "src/api/handlers.rs:45:78",
    "name": "login_route",
    "kind": "function",
    "signature": "pub async fn login_route(req: Request) -> Result<Response>",
    "visibility": "pub",
    "code": "pub async fn login_route(req: Request) -> Result<Response> {\n    ...\n}"
  },
  "callers": [ ... ],
  "callees": [ ... ],
  "type_signatures": { ... },
  "control_flow": { ... },
  "git_history": [ ... ],
  "next_queries": [
    "blast_radius(src/api/handlers.rs:45:78)",
    "type_flow(src/api/handlers.rs:45:78)",
    "call_slice(src/api/handlers.rs:45:78)"
  ]
}
```

Note: anchors are referenced by physical location (file:line:line).
Rust entities get compiler-verified signatures, types, control flow.
Other languages get tree-sitter-level information.

---

## Screen 7: Coming Back Tomorrow

User opens Parseltongue the next day.

```
┌──────────────────────────────────────────┐
│  Parseltongue                        [—] │
│──────────────────────────────────────────│
│                                          │
│  YOUR CODEBASES                          │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │ my-rust-project           STALE    │  │
│  │ ~/code/my-rust-project             │  │
│  │ 387 entities · 1,204 edges         │  │
│  │ Last analyzed: yesterday           │  │
│  │ 3 commits behind                   │  │
│  │                                    │  │
│  │ HTTP: stopped     [ Start Server ] │  │
│  │ [ Re-analyze ]  [ Open Terminal ]  │  │
│  └────────────────────────────────────┘  │
│                                          │
└──────────────────────────────────────────┘
```

One click to re-analyze. Server can restart with old data while re-ingestion runs.

---

# Token Economics

## The wc-to-Token Bridge

Every entity stores `wc` (word count). Tokens ≈ wc * 1.3 (rough average for code).
This means we can compute exact token economics from the database itself:

    SELECT
      SUM(CASE WHEN entity_type IN ('function','method','struct',...) THEN wc END) as searchable_wc,
      SUM(CASE WHEN entity_type = 'doc_comment' THEN wc END) as doc_wc,
      SUM(CASE WHEN entity_type = 'import' THEN wc END) as import_wc,
      SUM(CASE WHEN entity_type IN ('comment','whitespace') THEN wc END) as overhead_wc,
      SUM(wc) as total_wc
    FROM entities WHERE file_path LIKE 'src/%';

    -- "Your codebase: 1.2M words (1.56M tokens).
    --  Searchable: 920K words (1.2M tokens). Overhead: 280K words.
    --  You queried with 7 words. We returned 200 tokens to decide,
    --  then 20K tokens of understanding. 99.7% token reduction."

## Per-Query Token Flow

| Screen | Tokens (Internal) | Tokens (to LLM) |
|--------|-------------------|-----------------|
| Screen 4: Query arrives | 0 | ~7 words |
| Event 2: Search | 30 | - |
| Event 3: Anchor | 100 | - |
| Event 4: Cluster | 12,000 | - |
| Screen 5: Cluster selection | - | ~200 |
| Screen 6: Deep dive | - | Up to 20,000 |

LLM pays ~200 tokens to choose, then up to 20k for ONE deep dive (not 80k for all 4).

---

# Big Rocks

- Big-Rock-01: the scope and dependencies
    - language Rust 21
    - treesitter for
        - C C++ Javascript Typescript Python Java Go
    - rustcompiler enrichment for Rust code

- Big-Rock-02: the primary-key and entity_type
    - Uniform PK: `path:start_line:end_line` — see Entity Taxonomy above
    - Sentinels: -1:-1 = folder, 0:0 = file, N:M = code span
    - ISG_L1_V3 (language|||kind|||scope|||name|||file_path|||discriminator) is DERIVED, not the key
    - Every entity has: pk, entity_type, wc (word count)
    - 30 entity_types for 100% file coverage — see Entity Taxonomy above
    - Module-level only: nested items (closures, inner fns) are part of parent snippet
    - Validated by codemogger (uses same `file:line:line` chunk key, same module-level-only rule)

- Big-Rock-03: code-graph-building
    - .gitignore-driven walk (simplified: directory names only, no globs)
    - Hardcoded ALWAYS_IGNORE: .git, node_modules, target, build, dist, __pycache__, .venv, .cargo, .rustup
    - SHA-256 hash per file for incremental indexing (skip unchanged files on re-analyze)
    - Folder → folder edges (parent/child)
    - File → folder edges (belongs_to)
    - Code span → file edges (part_of)
    - Code span → code span edges (calls, imports, implements — from tree-sitter + rustc)
    - Rust files (.rs) → Layer 2 (tree-sitter) + Layer 3 (rustc_private enrichment)
    - Rust config (Cargo.toml) → parsed as TOML, yields dependency/package_meta/config_section entities
    - Other parsable languages (py, js, ts, go, java, c, cpp) → Layer 2 only
    - Unparsable files → Layer 1 only (just the address + hash)
    - Tests → same entities, flagged with is_test=true

---

# Decisions (2026-03-16 brainstorm session)

## D1: Tauri App is Priority One
- Mac-first. Workspace management. File picker, ingestion status, settings, logs.
- Tauri does NOT do: graph algorithms, compiler analysis, database ops, search logic.
- Queries happen over HTTP. Tauri manages the lifecycle.

## D2: Rust Gets rustc_private Enrichment
- Pin nightly toolchain. Extract: resolved types, real call graphs, trait impls, visibility, MIR.
- Proven by: Miri, Flowistry, Aquascope, Prusti, Kani, Rudra.

## D3: Other Languages Get Basic Tree-Sitter Only
- Entity extraction + basic edges. No deep analysis.

## D4: HTTP-Only for LLM Integration
- Ship HTTP REST. MCP can be a thin wrapper added later.

## D5: Algorithm Breadth is Minimal
- Only what the 7-event journey needs: RRF, BFS, ego network, deep dive.

## D6: Audience is Both Humans and LLMs
- OSS contributors + LLM coding agents. Same journey for both.

## D7: Database is Turso/libSQL
- Replacing CozoDB. Single file. FTS5 built-in.

## D8: Primary Key is Physical Location
- file_path + optional start_line:end_line. ISG_L1_V3 is derived, not identity.

## D9: Entity Taxonomy is 30 Types with `entity_type` Column (2026-03-17)
- Every entity has: pk, entity_type, wc (word count).
- 4 structural (folder, file_parsable, file_unparsable, file_config)
- 18 searchable code entities (function, method, struct, class, enum, trait, interface, impl,
  type_alias, constant, static, macro, module, variable, constructor, namespace, record, object)
  + is_test flag on function/method
- 5 non-code entities (import, doc_comment, comment, attribute, whitespace) for 100% coverage
- 3 Rust config spans (dependency, package_meta, config_section)
- Doc comments (///, //!, /** */) folded into adjacent code entity's `doc_comment` FTS field.
- Module doc comments (//!) folded into file_parsable entity's `doc_comment` field.
- Plain comments (//), whitespace counted for coverage only.
- Imports drive dependency graph edges. Attributes attach to next code entity.
- Module-level only: nested items are part of parent entity's snippet, not separate entities.
- Only code entities are FTS-searchable. Everything else is graph/coverage only.

## D10: Coverage via Word Count (2026-03-17, grounded in apache/iggy)
- Every entity stores `wc` (word count). File stores `total_wc`.
- For parsable files: sum(entity.wc) = file.total_wc. Verified on save. Zero gaps.
- Coverage computable at file, folder, and repo level via SQL GROUP BY entity_type.
- Token economics derived from wc: tokens ≈ wc * 1.3.
- Expected breakdown: ~65% searchable code, ~10% doc comments (also searchable), ~25% overhead.
- Lock files, binaries, generated files are file entities with total_wc but no child entities.
- .svelte could be added if tree-sitter-svelte grammar is included (adds 70 files in iggy).

## D11: Data-Driven Tree-Sitter Walker (2026-03-17, codemogger-validated)
- Follow codemogger's imperative AST walking, not v1.6.1's .scm query files.
- LanguageConfig struct: name, extensions, top_level_nodes, split_nodes (const, compile-time).
- Shared walker for all languages. Classify via node.kind() → entity_type.
- splitNodes for large containers (>150 lines → extract methods).
- Comment detection via text inspection (/// vs // are same node type).
- Export/decorator/template unwrapping in shared code.
- v1.6.1's FileWordCoverage schema validates our wc model.
- v1.6.1's 8 dependency edge types are the right set to extract.
- Reindexing: <50ms per changed file (no embedding bottleneck).

---

# CPU-Only Guarantee

No GPU. No embedding model. No LLM in the middle.
Symbol trie lookup, trigram index, graph traversal, rustc type info.
Full transparency: logs show exactly why each result ranked.
