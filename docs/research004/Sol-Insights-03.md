<!-- markdownlint-disable MD013 MD024 MD060 -->

# Solution Insights 03: Public Interface Signature Graphs and Exact Source Accounting

**Date:** 2026-07-10
**Status:** Research thesis and architecture proposal
**Product:** Parseltongue, a Rust LLM companion for trustworthy code understanding
**Primary question:** Should architecture be represented mainly as public interface signatures and their dependency/call graph, while private code is left for an LLM to inspect only when a change requires it?

## 1. Governing Answer

Yes, with one important correction:

> Parseltongue should present architecture as a graph of exposed interface signatures, but it should account for all source bytes and retain enough hidden implementation evidence to prove how a private change reaches, implements, or depends on a public surface.

The default architecture view should not be a giant graph of every function, block, closure, and nested syntax node. It should show the contracts that another meaningful boundary can observe:

- externally callable APIs;
- package, crate, module, or workspace APIs;
- HTTP and RPC routes;
- CLI commands and flags;
- exported types, traits, interfaces, and schemas;
- configuration keys and feature flags;
- public headers and ABI-sensitive declarations;
- events, database objects, and generated contracts when they form a consumer-facing boundary.

The default graph then needs only a small relationship vocabulary:

```text
DEPENDS_ON
CALLS
IMPLEMENTS
EXPORTS_OR_REEXPORTS
```

`CALLS` already supports both questions the user cares about:

```text
forward call  = outgoing CALLS edges
backward call = incoming CALLS edges
```

Containment does not need to dominate the product view. It remains useful as extraction metadata for resolving ownership, visibility, and source attribution, but a reviewer should not be forced to navigate a file -> class -> method -> block hierarchy merely to understand a change.

The proposed product has two coordinated artifacts:

1. **Public Interface Signature Graph (PISG):** the small, visible architecture graph.
2. **Source Accounting Ledger (SAL):** a disjoint byte-span partition proving how every source file was classified, including public signatures, restricted signatures, implementation, comments, imports, errors, and unknown material.

Together they support a stronger claim than either artifact alone:

```text
We show only what matters architecturally,
but we can prove what happened to everything else.
```

## 2. Premise Check

### 2.1 The premise is directionally strong

The product instinct is sound:

- architecture views become unreadable when every implementation symbol is treated as equally important;
- public signatures are more stable and more review-relevant than arbitrary private syntax;
- consumers depend on contracts more directly than they depend on source layout;
- reviewers primarily need to know whether a change altered a shared boundary and who consumes it;
- an LLM can inspect hidden implementation details on demand after the graph identifies the relevant public surface.

This is consistent with the local corpus. [Concept 15 in the Tree-sitter reference](../research002/tree-sitter-ref-202606.md) explicitly concludes that the public interface graph should be the first high-value projection because it answers which surface is exposed, who consumes it, what tests cover it, and how risky a change is.

### 2.2 The literal claim that private code cannot act is false

Private code can act. It can:

- determine the result returned by a public function;
- perform a side effect behind a public method;
- register a route or callback during startup;
- call a new external dependency;
- alter authentication, persistence, retries, caching, or validation;
- be shared by several public interfaces;
- mutate a database schema or serialized representation;
- become reachable through reflection, macros, generated code, tests, or framework conventions;
- change performance or resource behavior without changing any signature.

The correct distinction is therefore not:

```text
public = acts
private = does not act
```

It is:

```text
public = belongs in the default architecture projection
private = hidden implementation whose effects may still need attribution
```

### 2.3 Publicness is not universally knowable from syntax alone

Some languages make visibility relatively explicit. Others make it conventional, boundary-dependent, configuration-dependent, or runtime-dynamic.

Even in strongly specified languages, a local modifier is often insufficient:

- Rust `pub` is not externally reachable if an ancestor module is private, unless re-exported.
- Java `public` types are not externally accessible from a named module unless their package is exported.
- TypeScript `export` can mean file/module visibility while `package.json` determines package entry points.
- C and C++ public surfaces depend on headers, linkage, export attributes, build flags, install rules, and modules.
- Python has conventions and `__all__`, not enforced private instance variables.
- SQL visibility depends on deployed grants and roles, not only DDL text.
- shell functions can become available through sourcing or runtime export.

The model must represent **exposure relative to a boundary**, not force every symbol into a universal Boolean `public` field.

### 2.4 Line spans and content hashes solve different problems

Line or byte spans answer:

```text
Where is this occurrence in this exact file revision?
```

Content hashes answer:

```text
Are these bytes or normalized structures equal?
```

Neither answer is:

```text
Which durable logical symbol is this across revisions?
```

Durable identity, current occurrence, signature version, body version, and matching evidence must be separate fields.

## 3. The Chosen Thesis

The recommended architecture is a hybrid of three ideas:

```text
interface design        -> show contracts, not implementation noise
content-addressed store -> version exact source and derived artifacts
accounting ledger       -> every byte belongs to a known or unknown category
```

The data flow is:

```text
repository revision
        |
        v
immutable file revision identified by exact content hash
        |
        +-------------------------+
        |                         |
        v                         v
disjoint source ledger      syntax/semantic evidence
        |                         |
        +------------+------------+
                     v
        stable symbols + versioned occurrences
                     |
                     v
       boundary-aware exposure resolution
                     |
                     v
       Public Interface Signature Graph
                     |
                     v
      before/after visual Change Proof
```

The central product behavior is:

```text
Default: show exposed signatures and graph changes.
On demand: reveal the exact hidden source spans and evidence paths.
Always: disclose unknown, unresolved, stale, or unparsed regions.
```

## 4. Expert Lenses Used

This synthesis uses five engineering lenses.

### Product and code-review lens

The graph is valuable only if it shortens a real review decision. A small public-surface change map is more useful than a complete graph that requires graph expertise.

### Programming-language and API lens

Visibility is language-specific and boundary-relative. Export resolution, package manifests, build metadata, and compiler facts often matter as much as syntax.

### Code-intelligence and parser lens

Tree-sitter provides concrete syntax, spans, error recovery, and query candidates. It does not provide universal package exposure, type resolution, dynamic dispatch resolution, or framework semantics by itself.

### Storage and identity lens

Stable symbols, immutable file revisions, mutable occurrences, and content fingerprints are distinct entities. Conflating them creates false additions, false deletions, and orphaned graph edges.

### Skeptical systems lens

The adversarial questions are:

- Can the graph silently omit a caller?
- Can two overloads collide?
- Can nested spans make coverage exceed 100 percent?
- Can a private change affect a public contract without appearing in the view?
- Can an explicit `public` modifier still fail to expose the symbol?
- Can line movement or file renaming fabricate architectural change?
- Can the system reconstruct and account for the exact source bytes it claims to understand?

The chosen architecture is shaped around making those failures visible.

## 5. What The Existing Parseltongue Research Already Established

### 5.1 The original visualization thesis already used progressive disclosure

The archived [Interface Signature Graph Visualization thesis](../research000/archive/archive-web-ui/INTERFACE_SIGNATURE_GRAPH_THESIS.md) proposed four levels:

```text
Level 0: modules and trait clusters
Level 1: structs, impl blocks, traits, and enums
Level 2: public methods
Level 3: all entities and signatures
```

That is an early form of the current idea. Public interfaces were already identified as the useful middle layer, while private implementation remained available only at the deepest zoom.

### 5.2 ISGL1 v1 used mutable line positions as identity

The current [README entity-key section](../../README.md#entity-key-format) still documents:

```text
language:entity_type:entity_name:file_path:line_range
```

Example:

```text
rust:fn:authenticate:src_auth_rs:10-50
```

This format is convenient for navigation but structurally unstable. Adding comments above a function changes its key even though the function itself has not changed.

### 5.3 ISGL1 v2 separated permanent identity, matching, and change evidence in its design

The [ISGL1 v2 Stable Entity Identity document](../research001/unclassified/ISGL1-v2-Stable-Entity-Identity.md) proposed three components:

| Component | Intended purpose |
|---|---|
| Birth timestamp | Permanent identity assigned once |
| Semantic path | Candidate lookup and human-readable matching |
| Content hash | Changed versus unchanged classification |

Its key design was:

```text
rust:fn:handle_auth:__src_auth_rs:T1706284800
```

Its own conclusion is explicit: the content hash is stored separately and is not part of the primary key.

### 5.4 The shipped ISGL1 v2 implementation differs from the design

The current implementation in [`isgl1_v2.rs`](../../crates/parseltongue-core/src/isgl1_v2.rs) calls the suffix a birth timestamp, but computes it deterministically from `file_path + entity_name` using Rust's `DefaultHasher`, then maps the result into a timestamp-shaped numeric range.

That means it is not an assigned birth identity. It is a location-and-name-derived fingerprint rendered as a timestamp.

This creates several risks:

- file rename changes the value;
- symbol rename changes the value;
- same-name methods in different owners in one file can collide because owner and entity type are absent from the hash input;
- clean re-index behavior depends on a hasher whose algorithm is not a persistence contract;
- the suffix visually implies chronology even though it encodes no birth time.

The Rust standard library documentation for [`DefaultHasher`](https://doc.rust-lang.org/std/collections/hash_map/struct.DefaultHasher.html) explicitly says its internal algorithm is unspecified and hashes should not be relied upon across releases.

### 5.5 Content hashing also changed meaning between design and implementation

The ISGL1 v2 design document describes whitespace-normalized hashing and a shortened digest. The current [`compute_content_hash`](../../crates/parseltongue-core/src/isgl1_v2.rs) hashes exact source bytes with SHA-256 and returns the full hexadecimal digest.

Neither policy is inherently wrong, but they answer different questions:

| Hash | Question answered |
|---|---|
| Exact source hash | Did any byte change? |
| Normalized source hash | Did non-normalized textual content change? |
| Structural hash | Did the selected syntax structure change? |
| Signature hash | Did the exposed contract change? |
| Body hash | Did implementation change? |

One overloaded `content_hash` field cannot answer all five reliably.

### 5.6 Parseltongue already contains a first source-accounting experiment

The [v1.6.5 Ingestion Diagnostics PRD](../research000/archive/v165/PRD-v165.md) introduced raw and effective word coverage:

```text
raw coverage       = entity words / source words
effective coverage = entity words / (source words - import words - comment words)
```

The feature is implemented in [`streamer.rs`](../../crates/pt01-folder-to-cozodb-streamer/src/streamer.rs), represented by [`FileWordCoverageRow`](../../crates/parseltongue-core/src/entities.rs), and stored in the `FileWordCoverage` relation in [`cozo_client.rs`](../../crates/parseltongue-core/src/storage/cozo_client.rs).

This is highly relevant evidence. It proves the product has already tried to answer:

```text
What did Parseltongue miss in this file?
```

However, it is a heuristic ratio, not an exact partition:

- entity bodies can overlap when entities are nested;
- a parent entity may include a nested child entity's words;
- excluded tests are omitted from `entities_to_insert`;
- comments inside function bodies remain inside entity content while top-level comments are subtracted separately;
- imports can be nested or represented differently by grammar;
- whitespace-based words are not stable LLM tokens;
- coverage can exceed 100 percent when overlapping entities are summed.

The proposed Source Accounting Ledger is the exact successor to this useful experiment.

### 5.7 The newer research already recommends stable symbols plus occurrences

The 2026 [Tree-sitter reference corpus](../research002/tree-sitter-ref-202606.md) recommends replacing line-based identity with a stable symbol plus occurrence model. It points to SCIP as the strongest direct precedent.

The locally inspected [`scip.proto`](../../git-ref-repo/ignore-this-folder-repos/scip-code__scip/scip.proto) separates:

- a structured symbol identifier;
- rich symbol information and signatures;
- occurrences that associate a source range with a symbol;
- enclosing ranges for definitions and expressions;
- position encoding;
- relationships and occurrence roles.

This is the right conceptual correction:

```text
symbol identity != source occurrence
```

### 5.8 The public-interface graph is already the strongest local product thesis

[Concepts 15 through 21](../research002/tree-sitter-ref-202606.md) describe:

- a public interface graph as the first high-value projection;
- TypeScript package exports and routes;
- Rust reachability, `pub use`, Cargo features, CLI and HTTP surfaces;
- C/C++ headers, linkage, export attributes, and ABI risk;
- Python `__all__`, package re-exports, routes, models, and CLI entry points;
- evidence, confidence, freshness, and compiler-backed verification.

The current thesis does not replace that work. It narrows and strengthens it with an exact identity and source-accounting model.

## 6. Candidate Architectures

### Approach A: Store only public symbols

In this model, private entities are discarded during ingestion.

**Advantages:**

- smallest graph;
- easiest visualization;
- fewer identities to reconcile;
- low memory usage;
- direct focus on contract changes.

**Failure modes:**

- a changed private helper cannot be attributed to the public APIs it affects;
- calls that pass through private functions disappear;
- a new external dependency introduced privately is invisible;
- effective visibility cannot always be computed without owner/module context;
- framework registration and generated behavior can be missed;
- no evidence path exists when a reviewer asks why a public impact was reported.

**Judgment:** Too destructive for a trust product.

### Approach B: Store and render the complete syntax graph

In this model, every entity and containment relationship becomes a visible architecture node.

**Advantages:**

- maximum raw detail;
- straightforward drill-down;
- easy to reuse parser output directly;
- private call paths remain explicit.

**Failure modes:**

- visual noise dominates the review question;
- nested entities and syntax nodes are mistaken for architecture;
- identity burden grows dramatically;
- users must understand implementation before seeing the contract;
- graph layout instability makes before/after comparison difficult.

**Judgment:** Useful as a debug projection, wrong as the product default.

### Approach C: Public surface graph plus exact source ledger

In this model:

- all source bytes are accounted for;
- symbols and relationships may be indexed internally;
- only exposed surfaces appear in the default graph;
- private paths are collapsed into evidence-bearing summary edges;
- changed private regions appear as implementation change capsules attached to affected public nodes;
- an LLM can request the exact source and witness path when deeper inspection is needed.

**Advantages:**

- small and stable visual architecture;
- no silent loss of source material;
- public/private uncertainty can be disclosed;
- private behavioral effects remain explainable;
- source coverage becomes an invariant, not a marketing percentage;
- supports both human review and agent exploration.

**Judgment:** Recommended.

### Approach D: Compiler artifacts only

In this model, Parseltongue consumes rustdoc/SCIP/compiler symbols, Java class metadata, TypeScript declarations, link maps, and other semantic artifacts, avoiding syntax-first inference.

**Advantages:**

- highest precision where artifacts exist;
- stronger effective visibility and type resolution;
- closer to actual build configuration.

**Failure modes:**

- builds may fail during an agent edit;
- many repositories cannot produce every artifact locally;
- dynamic frameworks and config contracts still require custom extraction;
- broad language support becomes operationally expensive;
- syntax-error-tolerant review disappears.

**Judgment:** Use as an enrichment and verification tier, not the only ingestion path.

## 7. Structured Challenge

### Product lens

The visible product should show only contracts and their changes. A reviewer should be able to answer within one screen:

```text
What public surface changed?
Who calls or depends on it?
What hidden implementation changed behind it?
What is unknown?
```

### Language lens

The graph cannot use one Boolean visibility rule across languages. It needs language packs plus package/build boundary resolvers.

### Parser lens

Tree-sitter can reliably provide exact source ranges and broad declaration candidates. It should not be promoted into a universal semantic resolver.

### Storage lens

Persist immutable file revisions and stable symbol lineage. Treat spans and hashes as versioned evidence, never as the sole durable identity.

### Skeptical systems challenge

The skeptic rejects the simplistic formulation for five reasons:

1. A private function may implement several public APIs.
2. A `public` modifier may not make an item reachable outside the package.
3. A content hash cannot distinguish identical copies and changes whenever content changes.
4. Summed entity spans cannot provide exact coverage because syntax trees overlap.
5. A graph with hidden private nodes can invent misleading direct calls between public nodes unless it preserves a witness path.

### Countermeasures

The refined design responds as follows:

- retain private implementation facts internally, but hide them by projection;
- model exposure from an observer boundary, with evidence and confidence;
- use a stable opaque symbol ID plus structured descriptors and version hashes;
- partition source into disjoint atomic byte segments;
- label collapsed graph edges as direct or summarized and store the private witness path;
- never answer "no impact" without coverage, freshness, and resolver qualifications.

## 8. What Counts As A Public Interface

"Public interface" should mean an **observable contract**, not merely a declaration containing a `public` token.

### Code-level contracts

- exported functions and methods;
- exported classes, structs, enums, traits, interfaces, protocols, and type aliases;
- public fields, variants, associated types, generic constraints, and error types;
- public constants and macros;
- subclass or implementer contracts such as protected virtual methods.

### Runtime contracts

- HTTP, RPC, GraphQL, and message routes;
- event topics and payloads;
- CLI commands, arguments, flags, exit behavior, and output schemas;
- plugin registrations and extension points;
- FFI symbols and calling conventions.

### Data and operational contracts

- database tables, views, procedures, migration effects, and permissions;
- serialized request, response, event, and persistence schemas;
- environment variables, feature flags, and configuration keys;
- generated OpenAPI, protobuf, GraphQL, JSON Schema, and client surfaces.

### Boundary levels

Publicness should use an exposure scope:

```text
external_public
package_public
workspace_public
module_public
subclass_public
file_private
lexical_private
runtime_conditional
unknown
not_applicable
```

The same declaration may have multiple exposure facts. For example, a Rust `pub(crate)` function is crate-visible but not externally public. A TypeScript export from an internal file is module-visible but may not be reachable through the package's `exports` map.

## 9. Can Parseltongue Know Publicness In Every Language?

The short answer is no, not with equal certainty and not from Tree-sitter alone.

The better question is:

```text
For this language, revision, build configuration, and observer boundary,
what evidence says this contract is exposed?
```

### Cross-language capability matrix

| Language or stack | Strong syntax signal | Additional context required | Realistic confidence |
|---|---|---|---|
| Rust | `pub`, restricted `pub`, item kinds | ancestor module reachability, `pub use`, Cargo target/features, macros, `cfg` | High with resolver/compiler; medium from syntax only |
| Go | uppercase exported identifiers | package boundary, build tags, generated code, embedding | High for ordinary source |
| Java | public/protected/private/package access | package and `module-info.java` exports/opens, generated code, reflection | High statically; runtime reflection qualified |
| C# | public/protected/internal/private combinations | assembly boundary, friend assemblies, partial/generated code | High with project context |
| Swift | open/public/package/internal/fileprivate/private | package and module build boundaries, extensions, generated interfaces | High with build context |
| Kotlin | public default, private/protected/internal | compilation module, JVM interop, `@PublishedApi`, generated code | High with module context |
| Scala | public default, private/protected and qualifiers | package/build target, exports, givens/extensions, macros | Medium-high with compiler support |
| TypeScript | ES exports and class modifiers | re-exports, package `exports`, tsconfig paths, declaration output, type erasure | Medium-high with project resolution |
| JavaScript | ESM exports and private fields | CommonJS mutation, package `exports`, runtime registration, bundler behavior | Medium; lower for dynamic modules |
| C | external/internal linkage, declarations, `static` | installed/public headers, visibility flags, export macros, linker/build metadata | Medium with build artifacts |
| C++ | member access, linkage, modules and exports | public headers, templates, macros, build flags, symbol visibility, module graph | Medium-high with compiler index |
| Python | `__all__`, package re-exports, underscore convention | dynamic imports, monkey patching, decorators, framework registration | Medium-low; evidence must be qualified |
| Ruby | public/protected/private method state | runtime metaprogramming, reopenings, mixins, conditional visibility changes | Medium-low |
| PHP | class member modifiers and interfaces | global/package surface, autoloading, framework routes, dynamic properties | Medium |
| SQL | GRANT/REVOKE and schema declarations | deployed role catalog, migration order, environment-specific privileges | Low from file syntax; high against live catalog |
| Shell | executable files, sourced functions, `export -f` | runtime sourcing, PATH, environment, generated/evaluated code | Low and contextual |

Official language evidence supports this variation:

- [Rust visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html) requires ancestor accessibility and supports restricted visibility and re-exports.
- [Go's specification](https://go.dev/ref/spec) defines exported identifiers through Unicode uppercase naming and declaration scope.
- [Java's module specification](https://docs.oracle.com/javase/specs/jls/se20/html/jls-7.html) makes external accessibility depend on both `public` and an exported package.
- [TypeScript modules](https://www.typescriptlang.org/docs/handbook/2/modules.html) and [Node package entry points](https://nodejs.org/api/packages.html) distinguish module exports from package encapsulation.
- [Python's class documentation](https://docs.python.org/3/tutorial/classes.html#private-variables) states that enforced private instance variables do not exist, while underscores are a non-public convention; [`__all__`](https://docs.python.org/3/tutorial/modules.html#importing-from-a-package) provides an explicit but limited export list.
- [C++ access control](https://eel.is/c++draft/class.access.general), [linkage](https://eel.is/c++draft/basic.link), and [module exports](https://eel.is/c++draft/module) are separate concepts.
- [Swift access control](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/accesscontrol/) is relative to declaration, file, module, and package.
- [Kotlin visibility](https://kotlinlang.org/docs/visibility-modifiers.html) depends on file, class, subclass, and compilation module.
- [Ruby visibility](https://ruby-doc.org/3.4/Module.html) can be applied by method calls that change current or existing method visibility.
- [PostgreSQL privileges](https://www.postgresql.org/docs/current/ddl-priv.html) show that database exposure is role and deployment state, not merely source syntax.
- [Bash functions](https://www.gnu.org/s/bash/manual/html_node/Shell-Functions.html) can be sourced, dynamically scoped, and exported to child shells.

### Current Parseltongue capability is narrower than its type model suggests

[`Language`](../../crates/parseltongue-core/src/entities.rs) enumerates Rust, JavaScript, TypeScript, Python, Java, C, C++, Go, Ruby, PHP, C#, Swift, Kotlin, Scala, and SQL.

However, the current [`QueryBasedExtractor`](../../crates/parseltongue-core/src/query_extractor.rs):

- loads entity and dependency query packs for 12 languages;
- comments Kotlin out because of a grammar-version mismatch;
- does not load Scala or SQL in that path;
- returns a `ParsedEntity` with type, name, language, line range, file, and generic metadata;
- does not populate the richer `Visibility` enum defined elsewhere in `entities.rs`;
- uses query files that capture declarations but generally do not capture access modifiers or package reachability.

Therefore, the current product cannot truthfully claim cross-language public-interface extraction merely because a `Visibility` enum exists.

## 10. Visibility Must Be A Resolved Fact, Not A Boolean Field

The visibility model should be observer-relative.

```rust
pub enum ExposureScope {
    External,
    Package,
    Workspace,
    Module,
    Subclass,
    File,
    Lexical,
    RuntimeConditional,
    Unknown,
}

pub enum ExposureEvidenceKind {
    ExplicitModifier,
    LanguageDefault,
    Reexport,
    PackageManifest,
    BuildManifest,
    PublicHeader,
    LinkerSymbol,
    CompilerIndex,
    FrameworkRegistration,
    NamingConvention,
    RuntimeCatalog,
    Heuristic,
}
```

An exposure record should answer:

```text
symbol
observer boundary
exposure scope
condition or feature set
evidence source
confidence
freshness
unresolved prerequisites
```

Example:

```json
{
  "symbol_id": "sym_01J...",
  "scope": "package_public",
  "observer": "npm:@acme/users",
  "condition": "node/import/default",
  "evidence": [
    "src/index.ts reexports UserClient",
    "package.json exports maps . to dist/index.js"
  ],
  "confidence": "high",
  "freshness": "revision_exact"
}
```

The useful API question becomes:

```text
is_visible(symbol, observer_boundary, build_configuration, revision)
```

not:

```text
symbol.public == true
```

## 11. The Primary-Key Decision

The primary-key problem should be answered separately for each kind of record. There is no single magic key that should identify symbols, source files, occurrences, and versions at once.

### 11.1 Recommended identity layers

| Record | Durable identity | Version or occurrence identity | Must not be used as durable identity |
|---|---|---|---|
| Repository | `repository_id` assigned from configured repository origin/workspace | revision ID or Git commit | absolute local path alone |
| Logical file | `file_id` with rename lineage | `file_revision_id` from exact bytes and revision context | current path alone |
| Logical symbol | opaque `symbol_id` assigned once and reconciled | `symbol_version_id` | line span or content hash alone |
| Public signature | parent `symbol_id` plus signature version | `signature_hash` | body hash |
| Source occurrence | no promise across revisions | `occurrence_id` from file revision, byte span, role, and symbol | line number |
| Source segment | no promise across revisions | `segment_id` from file revision and half-open byte range | word count |
| Graph edge | semantic endpoints and relationship kind | revision-scoped edge fact ID | database row number |

### 11.2 Stable symbol identity

The canonical database primary key should be an opaque durable `symbol_id`, such as a UUIDv7, ULID, or database-assigned 128-bit identifier. Its exact encoding is less important than its semantics:

```text
assigned once
never derived from a mutable line or file position
preserved when a reconciler confidently matches the same symbol
replaced when identity is ambiguous or genuinely new
```

An opaque ID alone is not enough for matching or interoperability. Each version also needs a structured canonical descriptor:

```text
repository identity
package or crate coordinate
language
symbol kind
owner descriptor chain
declared or exported name
overload discriminator
namespace/module descriptor
exposure context
```

This descriptor resembles the structured symbol model in SCIP, whose symbol grammar includes package coordinates and namespace/type/term/method descriptors while keeping source ranges on occurrences.

### 11.3 Signature and body fingerprints

Each symbol version should store multiple fingerprints:

```text
exact_source_hash
canonical_signature_hash
body_hash
structural_hash
documentation_hash
```

They are evidence, cache keys, and diff aids. They are not the stable symbol primary key.

The distinction enables precise review states:

| Signature hash | Body hash | Interpretation |
|---|---|---|
| unchanged | unchanged | no semantic artifact change observed |
| unchanged | changed | implementation changed behind the same contract |
| changed | unchanged or changed | public contract changed |
| new | new | symbol added or unmatched |
| absent | absent | symbol removed or unmatched |

### 11.4 Why content hash cannot be the symbol primary key

Using content as the only identity fails when:

- an implementation changes but is still the same logical function;
- two distinct functions have identical content;
- formatting or comments change under an exact hash;
- normalization erases a change that matters to macros, strings, or formatting-sensitive languages;
- a function is split, merged, or renamed;
- a public signature changes and the product needs to show continuity from old contract to new contract.

Content-addressing is excellent for immutable versions. It is not sufficient for mutable logical identity.

### 11.5 Why a line span cannot be the symbol primary key

Line spans are revision-local coordinates. They change under:

- inserted comments;
- formatting;
- import reordering;
- generated headers;
- CRLF/LF normalization;
- code movement;
- unrelated edits above the symbol.

They remain essential for navigation and source evidence, but they belong on an occurrence row.

### 11.6 Reconciliation across revisions

The reconciler should produce a decision, not silently rewrite identity.

Recommended matching order:

```text
1. Compiler or SCIP stable symbol match, when available.
2. Exact canonical descriptor and exact signature/body evidence.
3. Same owner, kind, exported name, and overload descriptor.
4. Exact subtree/body hash moved within the same repository.
5. Git rename/move context plus structural similarity.
6. Near-position evidence as a weak fallback only.
7. New symbol when no candidate is safe.
```

Every non-exact reconciliation should record:

```text
old_symbol_id
new_occurrence
match_method
confidence
alternatives_considered
evidence
decision: preserved | new | ambiguous
```

An ambiguous match should not be forced. It is better to report a possible rename than to join the history of two unrelated public contracts.

## 12. Source Accounting Ledger

The Source Accounting Ledger turns the existing word-coverage idea into an exact invariant.

### 12.1 Canonical unit: bytes, not lines

The canonical file is an immutable byte sequence:

```text
file_bytes = [0, N)
```

Each source segment is also a half-open range:

```text
[start_byte, end_byte)
```

The ledger must satisfy:

```text
first.start_byte = 0
last.end_byte = file_byte_length
segment[i].end_byte = segment[i + 1].start_byte
no segment overlaps another segment
no byte is omitted
sum(segment byte lengths) = file byte length
concatenate(segment source slices) = exact original file bytes
```

These invariants are exact even for Unicode and mixed content. Line and column coordinates are derived through a versioned line index.

### 12.2 Why line accounting is only a projection

A single line can contain:

- a public function signature;
- an opening brace;
- an implementation expression;
- an inline comment;
- several declarations separated by semicolons.

Therefore, asking the categories to sum to exactly 1,000 whole lines is ambiguous. The exact claim should be byte-based.

Useful line projections can still be reported:

```text
lines_touched_by_public_signature
lines_touched_by_implementation
lines_touched_by_comments
lines_with_unknown_bytes
```

Those values can overlap and should not be advertised as a partition. If the product needs an additive line-equivalent number, it can divide each segment's newline-aware byte contribution fractionally, but bytes remain the source of truth.

### 12.3 Token counts also need a named tokenizer

"Token count" is not universal. Different LLM tokenizers produce different totals.

The ledger should store either:

- raw bytes and compute tokens on demand; or
- token counts keyed by tokenizer name and version.

Example:

```text
tokenizer_id = o200k_base@2026-07
segment_id   = seg_...
token_count  = 42
```

Whitespace-separated word counts can remain a cheap diagnostic, but they should not be confused with model tokens or exact source coverage.

### 12.4 Primary source categories

Each byte receives one primary category:

```text
public_signature
restricted_signature
private_signature
implementation
import_or_export
public_contract_declaration
documentation_comment
non_documentation_comment
generated_or_preprocessor
parse_error
unknown_syntax
whitespace_or_separator
embedded_language
```

The exact vocabulary can evolve, but `unknown_syntax` must never be omitted. An unknown segment is a successful accounting result with incomplete understanding, not a reason to fabricate coverage.

### 12.5 Primary classification and secondary annotations

A byte can participate in several semantic facts. For example, a route decorator may be both syntax and public contract evidence. Exact partitioning still requires one primary category.

Secondary annotations should therefore live separately:

```text
segment primary category: public_signature
annotations:
  - route_registration
  - external_public
  - generated_from_openapi
  - parser_error_nearby
```

This avoids overlapping counts without losing semantic richness.

### 12.6 Partition algorithm

The partitioner can be deterministic:

1. Parse the file and collect all candidate byte intervals.
2. Collect public/restricted/private signature header intervals.
3. Collect body intervals, imports, exports, comments, preprocessor regions, injected languages, ERROR nodes, and MISSING-node diagnostics.
4. Add `0` and `file_byte_length` plus every candidate start and end to a sorted boundary set.
5. Split the file into atomic, non-overlapping intervals between adjacent boundaries.
6. Assign each atomic interval one primary category using a versioned precedence policy and the most-specific valid owner.
7. Classify uncovered gaps as whitespace/separator when lexically safe; otherwise classify them as unknown.
8. Merge adjacent intervals with identical primary class, owner, provenance, and confidence.
9. Validate the no-gap/no-overlap/reconstruction invariants.
10. Persist the candidate ledger only if validation succeeds.

Pseudocode:

```rust
fn partition_file_revision(
    source: &[u8],
    candidates: &[CandidateSpan],
) -> Result<Vec<SourceSegment>, PartitionError> {
    let boundaries = collect_sorted_boundaries(source.len(), candidates)?;
    let mut segments = Vec::new();

    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];
        let classification = classify_atomic_span(source, start..end, candidates);
        segments.push(SourceSegment::new(start, end, classification)?);
    }

    let merged = merge_adjacent_equivalent_segments(segments);
    verify_exact_source_partition(source, &merged)?;
    Ok(merged)
}
```

### 12.7 Tree-sitter's role

Tree-sitter is well suited to candidate interval generation:

- nodes expose byte ranges;
- grammars represent recognizable concrete syntax constructs;
- comments are commonly represented as extras;
- query packs can capture declarations, names, modifiers, bodies, imports, and comments;
- ERROR nodes identify unrecognized text;
- MISSING nodes record zero-width recovery insertions;
- incremental parsing can reuse unchanged structure.

The official [Tree-sitter query syntax](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html) documents both `ERROR` and `MISSING` nodes. The [grammar guide](https://tree-sitter.github.io/tree-sitter/creating-parsers/3-writing-the-grammar.html) explains concrete syntax trees and extras such as comments and whitespace.

Tree-sitter does not itself assign product categories or guarantee public exposure. Parseltongue's query packs and resolvers own that interpretation.

### 12.8 Nested entities must not be summed

Suppose a public function contains a private closure. The function's full AST range contains the closure's range.

If both ranges are independently counted, coverage exceeds the file length. The ledger solves this by splitting into atomic intervals and assigning one primary owner/category per interval.

The hierarchy can still be retained as secondary ownership:

```text
atomic segment primary owner: private closure
enclosing owners: public function -> module -> file
```

No bytes are double-counted.

### 12.9 Suggested file accounting response

```json
{
  "file": "src/auth.rs",
  "file_revision_id": "sha256:...",
  "byte_length": 24118,
  "line_count": 1000,
  "partition": {
    "public_signature_bytes": 1240,
    "restricted_signature_bytes": 510,
    "private_signature_bytes": 930,
    "implementation_bytes": 17640,
    "import_or_export_bytes": 820,
    "comment_bytes": 1840,
    "parse_error_bytes": 0,
    "unknown_bytes": 416,
    "whitespace_or_separator_bytes": 722
  },
  "invariants": {
    "no_gaps": true,
    "no_overlaps": true,
    "sum_equals_file": true,
    "exact_reconstruction": true
  },
  "semantic_coverage": {
    "known_bytes_pct": 98.28,
    "unknown_bytes_pct": 1.72,
    "public_exposure_resolution": "partial"
  }
}
```

The numbers above are illustrative, not measured from this repository.

## 13. Signature Extraction

### 13.1 Preserve exact and canonical forms

Every exposed contract should retain two representations:

```text
signature_source    = exact original bytes
signature_canonical = language-pack normalization for comparison
```

The exact form supports evidence and reconstruction. The canonical form supports stable diffs and hashing.

### 13.2 A signature is more than a name

Depending on language and contract kind, include:

- exported name and aliases;
- kind;
- parameter names and types;
- return or yield type;
- generic parameters and constraints;
- async, unsafe, throws, mutability, const, and effect markers;
- calling convention and linkage;
- visibility and exposure conditions;
- attributes, annotations, decorators, and feature guards that alter the contract;
- public fields and enum variants;
- request/response schemas for routes;
- command flags for CLIs;
- privilege or role constraints for database contracts.

### 13.3 The signature-body boundary is language-specific

Examples:

| Language | Signature boundary |
|---|---|
| Rust | attributes/visibility through return/where clause, excluding block body |
| Python | relevant decorators plus `def`/`async def` header and annotations, excluding suite body |
| TypeScript | export/modifiers/name/type parameters/parameters/return type, excluding method/function body |
| Java/C#/Kotlin/Scala/Swift | declaration modifiers, annotations, generics, parameters, return/throws/constraints, excluding body |
| C/C++ | header declaration or definition declarator including linkage/attributes/templates, excluding function body |
| Ruby | visibility state plus method declaration header; types may require external signature sources |
| SQL | object declaration and parameters plus relevant privilege/exposure metadata |
| Shell | command/function name and argument convention inferred from parser/docs; inherently weak |

This cannot be implemented by taking the first line of every entity.

### 13.4 Documentation is adjacent contract evidence

Doc comments should not be folded blindly into the signature hash. Changing prose should not necessarily imply a binary API change.

Store:

```text
signature_hash
documentation_hash
contract_annotation_hash
```

Then report documentation-only drift separately.

### 13.5 Conditional signatures need variants

One symbol may expose different contracts under:

- Rust Cargo features and `cfg`;
- TypeScript conditional package exports;
- C/C++ preprocessor flags and platforms;
- Java modules or build profiles;
- generated-code targets;
- SQL migration/environment state.

The signature version should carry a configuration predicate. Parseltongue must avoid presenting the union of all conditional variants as one always-valid API.

## 14. The Public Interface Signature Graph

### 14.1 Visible node model

```json
{
  "symbol_id": "sym_01J...",
  "symbol_version_id": "symv_01J...",
  "kind": "function",
  "display_name": "authenticate",
  "canonical_descriptor": "rust crate auth fn authenticate(...) -> Result<User, AuthError>",
  "signature_hash": "sha256:...",
  "exposures": ["package_public"],
  "conditions": ["feature=server"],
  "occurrence": {
    "file": "src/auth.rs",
    "start_byte": 1420,
    "end_byte": 1518,
    "start_line": 51,
    "end_line": 55
  },
  "confidence": "high",
  "freshness": "revision_exact"
}
```

### 14.2 Minimal visible edge model

The user-facing graph can remain deliberately small:

```text
CALLS
DEPENDS_ON
IMPLEMENTS
EXPORTS
```

Each edge should still carry:

```text
directness
evidence
confidence
freshness
configuration
witness path if collapsed
```

### 14.3 Direct and collapsed calls must not be confused

Consider:

```text
Public A -> private helper_1 -> private helper_2 -> Public B
```

The default graph may render:

```text
Public A ----calls through implementation----> Public B
```

But the stored edge must say:

```json
{
  "kind": "calls",
  "directness": "collapsed_private_path",
  "from": "Public A",
  "to": "Public B",
  "private_hops": 2,
  "witness_path": ["helper_1", "helper_2"],
  "confidence": "high"
}
```

Without this distinction, the projection lies about the source-level call graph.

### 14.4 Dependency edges can also be projected

A public signature can depend on another contract through:

- parameter or return types;
- inheritance, trait, interface, or protocol requirements;
- exposed fields or variants;
- route request/response schemas;
- public header includes;
- generated contracts;
- configuration or database schema.

The edge should explain which part of the signature creates the dependency.

### 14.5 Forward and backward calls are sufficient as navigation primitives

No separate reverse edge needs to be stored.

```text
forward_calls(symbol)  = outgoing CALLS
backward_calls(symbol) = incoming CALLS
```

The API may materialize reverse adjacency in memory for speed, but the canonical fact remains one directed edge.

### 14.6 Private implementation appears only when relevant

The default graph should hide private nodes. It should reveal one of three compact markers when private code matters:

```text
implementation changed
implementation path changed
unresolved implementation impact
```

Selecting the marker lets the LLM or reviewer inspect exact source spans and witness paths.

## 15. Is Containment Needed?

### 15.1 It is not needed as a dominant visual relationship

The user is right that a reviewer does not need to see every nesting level merely because parsers produce it.

Avoid making these the default architecture:

```text
repository contains directory
directory contains file
file contains class
class contains method
method contains block
block contains expression
```

That is a syntax outline, not necessarily an architecture.

### 15.2 It remains necessary as internal evidence

Containment or owner metadata is needed to:

- determine effective Rust visibility through ancestor modules;
- determine whether a method belongs to an exported class;
- distinguish same-name methods and overloads;
- associate a call with the smallest enclosing callable;
- identify signature and body boundaries;
- resolve protected/private scope;
- collapse private call paths safely;
- map a changed byte segment to the public contract it may affect;
- preserve SCIP-like owner descriptors for symbol identity.

The design conclusion is:

```text
retain containment internally
do not render it by default
```

It may be stored as an owner ID or descriptor path instead of a graph edge if that is simpler for the backend.

## 16. Change Proof Behavior

### 16.1 Compare contract and implementation separately

For each matched public symbol:

```text
compare signature hash
compare body hash
compare exposure facts
compare dependency edges
compare incoming/outgoing call projections
compare evidence coverage
```

### 16.2 Reviewer-facing change classes

| Change class | Meaning | Default presentation |
|---|---|---|
| Public signature added | New observable contract | new public node |
| Public signature removed | Contract no longer exposed | removed public node |
| Public signature modified | Consumer-visible declaration changed | high-attention node diff |
| Exposure changed | Same declaration became more or less reachable | boundary-change warning |
| Implementation-only change | Contract stable, body changed | implementation capsule on public node |
| Private dependency changed | Hidden code introduced/removed dependency | edge-change capsule with witness |
| Private call path changed | Public-to-public behavior path changed | collapsed path diff |
| Internal-only change | No known public reachability | compact internal-change summary |
| Unknown impact | Coverage or resolution insufficient | explicit unknown warning |

### 16.3 A private change can still be important

If a private helper changes and is reachable from three public endpoints, the product should not create a private architecture node by default. It should show:

```text
3 public contracts retain their signatures
1 shared implementation region changed
affected contracts: A, B, C
witness paths available
```

This honors both parts of the thesis:

- architecture remains public-surface-first;
- private behavior is not falsely treated as inert.

### 16.4 No public reachability is not proof of no effect

The product must distinguish:

```text
verified no public reachability
no public reachability found under current coverage
unknown because resolution is incomplete
```

For dynamic languages or low-coverage files, the last two states may be common.

## 17. Product API

The existing Parseltongue APIs can evolve without exposing storage details.

### Public surface

```text
GET /api/v1/public-surface
GET /api/v1/public-signature?symbol=...
GET /api/v1/public-impact?symbol=...
```

### Calls and dependencies

```text
GET /api/v1/forward-calls?symbol=...
GET /api/v1/backward-calls?symbol=...
GET /api/v1/dependencies?symbol=...
GET /api/v1/explain-edge?edge_id=...
```

### Change proof

```text
GET /api/v1/signature-diff?from=...&to=...
GET /api/v1/public-change-map?from=...&to=...
GET /api/v1/implementation-impact?change_id=...
```

### Source accounting

```text
GET /api/v1/source-accounting?file=...&revision=...
GET /api/v1/source-segments?file=...&category=unknown_syntax
GET /api/v1/index-confidence?scope=...
```

### Example public-impact response

```json
{
  "symbol": "authenticate",
  "observer_boundary": "crate_external_consumer",
  "exposure": "package_public",
  "signature_change": "none",
  "implementation_change": "modified",
  "forward_calls": 2,
  "backward_calls": 7,
  "affected_public_contracts": [
    "POST /login",
    "POST /refresh"
  ],
  "coverage": {
    "source_partition_complete": true,
    "call_resolution": "partial",
    "unknown_bytes": 0
  },
  "confidence": "medium_high",
  "limitations": [
    "trait-object dispatch may add callers not resolved by syntax extraction"
  ]
}
```

### Backend neutrality

The API contract should not reveal whether facts are persisted in SQLite/Turso, Neo4j, or another store, or traversed through `petgraph` or another runtime.

The product semantics must define:

- observer boundary;
- exposure categories;
- direct versus collapsed calls;
- ordering and pagination;
- revision consistency;
- unknown and unresolved behavior;
- evidence and confidence envelopes;
- source range encoding;
- configuration predicates.

## 18. Suggested Backend-Neutral Schema

The following is conceptual SQLite-oriented DDL. Names and column types can change, but the ownership boundaries should remain.

```sql
CREATE TABLE repository (
    repository_id TEXT PRIMARY KEY,
    canonical_origin TEXT,
    workspace_label TEXT NOT NULL
);

CREATE TABLE revision (
    revision_id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL,
    parent_revision_id TEXT,
    git_commit TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE file_identity (
    file_id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL,
    first_seen_revision_id TEXT NOT NULL
);

CREATE TABLE file_revision (
    file_revision_id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    revision_id TEXT NOT NULL,
    path TEXT NOT NULL,
    exact_content_hash TEXT NOT NULL,
    byte_length INTEGER NOT NULL,
    line_count INTEGER NOT NULL,
    language TEXT,
    parse_status TEXT NOT NULL,
    parser_provenance_json TEXT NOT NULL,
    UNIQUE (revision_id, path)
);

CREATE TABLE symbol (
    symbol_id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL,
    first_seen_revision_id TEXT NOT NULL,
    last_seen_revision_id TEXT
);

CREATE TABLE symbol_version (
    symbol_version_id TEXT PRIMARY KEY,
    symbol_id TEXT NOT NULL,
    revision_id TEXT NOT NULL,
    canonical_descriptor TEXT NOT NULL,
    symbol_kind TEXT NOT NULL,
    display_name TEXT NOT NULL,
    owner_symbol_id TEXT,
    canonical_signature_hash TEXT,
    exact_signature_hash TEXT,
    body_hash TEXT,
    structural_hash TEXT,
    UNIQUE (symbol_id, revision_id)
);

CREATE TABLE occurrence (
    occurrence_id TEXT PRIMARY KEY,
    symbol_version_id TEXT,
    file_revision_id TEXT NOT NULL,
    role TEXT NOT NULL,
    start_byte INTEGER NOT NULL,
    end_byte INTEGER NOT NULL,
    start_line INTEGER,
    start_column INTEGER,
    end_line INTEGER,
    end_column INTEGER,
    CHECK (start_byte >= 0),
    CHECK (end_byte >= start_byte)
);

CREATE TABLE source_segment (
    segment_id TEXT PRIMARY KEY,
    file_revision_id TEXT NOT NULL,
    start_byte INTEGER NOT NULL,
    end_byte INTEGER NOT NULL,
    primary_category TEXT NOT NULL,
    primary_symbol_version_id TEXT,
    confidence TEXT NOT NULL,
    provenance_json TEXT NOT NULL,
    CHECK (end_byte > start_byte)
);

CREATE TABLE exposure_fact (
    exposure_id TEXT PRIMARY KEY,
    symbol_version_id TEXT NOT NULL,
    observer_boundary TEXT NOT NULL,
    exposure_scope TEXT NOT NULL,
    condition_json TEXT,
    confidence TEXT NOT NULL,
    evidence_json TEXT NOT NULL
);

CREATE TABLE relationship_fact (
    relationship_id TEXT PRIMARY KEY,
    revision_id TEXT NOT NULL,
    from_symbol_id TEXT NOT NULL,
    to_symbol_id TEXT NOT NULL,
    relationship_kind TEXT NOT NULL,
    directness TEXT NOT NULL,
    condition_json TEXT,
    confidence TEXT NOT NULL,
    evidence_json TEXT NOT NULL,
    witness_path_json TEXT
);

CREATE TABLE reconciliation_decision (
    reconciliation_id TEXT PRIMARY KEY,
    old_symbol_id TEXT,
    new_occurrence_id TEXT NOT NULL,
    decision TEXT NOT NULL,
    method TEXT NOT NULL,
    confidence TEXT NOT NULL,
    evidence_json TEXT NOT NULL
);
```

### Publication invariant

Build all rows for a candidate revision, validate identity and partition invariants, then publish the revision atomically. A failed candidate must not replace the last known-good graph.

### In-memory graph projection

SQLite/Turso can own durable facts. A Rust runtime such as `petgraph` can load only the requested projection:

```text
public nodes for observer boundary X
CALLS and DEPENDS_ON edges valid under configuration Y
reverse adjacency for backward calls
revision Z only
```

This keeps the RAM graph smaller than the complete source ledger.

## 19. Language-Pack Contract

Each language pack should publish a capability manifest.

```yaml
language: rust
grammar_version: "..."
query_pack_version: "..."
capabilities:
  declaration_candidates: exact
  signature_spans: high
  explicit_visibility: high
  effective_package_exposure: resolver_required
  exports_reexports: high
  direct_calls: partial
  dynamic_dispatch: compiler_required
  source_partition: exact_with_unknowns
  malformed_source: tolerant
```

### Required outputs

```text
candidate symbols
exact signature and body spans
owner descriptors
explicit visibility modifiers and defaults
imports/exports/reexports
ERROR and MISSING diagnostics
comments and extras
candidate calls and dependencies
package/build boundary evidence
unknown spans
```

### Capability behavior

Unsupported capability must return:

```text
unsupported
partial
unknown
resolver_required
compiler_required
runtime_required
```

It must not return an empty set that looks like a verified negative.

## 20. Product Requirements

### REQ-PISG-001: Public surface is the default architecture

**WHEN** a user opens a repository architecture or change map
**THEN** Parseltongue SHALL show exposed interface signatures and their contract relationships by default
**AND** SHALL hide private implementation nodes unless they are expanded or changed.

### REQ-BOUNDARY-002: Publicness is observer-relative

**WHEN** Parseltongue classifies a symbol's exposure
**THEN** it SHALL record the observer boundary, exposure scope, evidence, condition, confidence, and revision
**AND** SHALL NOT reduce all languages to an unqualified Boolean public flag.

### REQ-IDENTITY-003: Line spans never define symbol identity

**WHEN** unrelated lines are inserted above an unchanged symbol
**THEN** the symbol SHALL retain its durable identity
**AND** its occurrence coordinates SHALL update independently.

### REQ-HASH-004: Fingerprints remain separate

**WHEN** Parseltongue stores exact source, signature, body, structural, or documentation hashes
**THEN** each hash SHALL declare its normalization policy and purpose
**AND** no content hash SHALL be the only durable logical symbol key.

### REQ-ACCOUNTING-005: Every source byte is accounted for

**WHEN** a text file revision is accepted into an index
**THEN** its source segments SHALL be gap-free, non-overlapping, ordered, and exactly reconstruct the original bytes
**AND** unknown bytes SHALL be explicitly categorized.

### REQ-ERROR-006: Broken code remains visible

**WHEN** Tree-sitter produces ERROR or MISSING nodes
**THEN** Parseltongue SHALL retain safe high-confidence facts where possible
**AND** SHALL classify unreliable or unrecognized regions as parse error or unknown
**AND** SHALL lower confidence instead of declaring the entire file understood.

### REQ-PATH-007: Collapsed paths retain witnesses

**WHEN** a public graph edge summarizes a call path through hidden private symbols
**THEN** the edge SHALL be marked as collapsed
**AND** SHALL retain a witness path or explicit reason why no witness is available.

### REQ-PRIVATE-008: Private changes are attributed

**WHEN** a private implementation region changes
**THEN** Parseltongue SHALL identify known reachable public contracts
**OR** SHALL state that no public reachability was found under named coverage and resolver limits.

### REQ-DIRECTION-009: One call fact supports both traversals

**WHEN** a call edge is stored
**THEN** it SHALL have one canonical caller-to-callee direction
**AND** forward and backward call APIs SHALL traverse outgoing and incoming adjacency respectively.

### REQ-PARITY-010: Incremental and full indexing agree

**WHEN** an incremental update is published
**THEN** its public graph, source partition, symbol identities, and diagnostics SHALL match a clean full re-index under the same versions and configuration.

### REQ-UNKNOWN-011: Empty is not unknown

**WHEN** a language pack or resolver cannot determine visibility, calls, or dependencies
**THEN** the API SHALL return an explicit partial/unknown state
**AND** SHALL NOT return an unqualified empty result.

### REQ-CONFIG-012: Conditional interfaces retain predicates

**WHEN** a signature or exposure depends on a feature, platform, export condition, or build flag
**THEN** the fact SHALL retain that predicate
**AND** the default view SHALL identify which configuration it represents.

### REQ-PROVENANCE-013: Every visible fact is inspectable

**WHEN** a reviewer selects a node or edge
**THEN** Parseltongue SHALL provide source occurrence, extractor, grammar/query version, revision, and confidence evidence.

### REQ-DOCS-014: Documentation drift is distinct

**WHEN** documentation changes without a canonical signature change
**THEN** Parseltongue SHALL report documentation-only drift rather than a binary signature change.

### REQ-BACKEND-015: Product semantics remain backend-neutral

**WHEN** storage or graph execution changes
**THEN** public API behavior, directionality, exposure semantics, unknown states, and evidence envelopes SHALL remain equivalent under a backend parity suite.

## 21. Testing Strategy

### 21.1 Identity fixtures

- add comments above an entity;
- move an unchanged entity within a file;
- move an entity across files;
- rename a file;
- rename a symbol while retaining its body;
- change only the body;
- change only the signature;
- create same-name overloads;
- create identical duplicate bodies;
- create same-name methods in different owners;
- split and merge entities;
- compare clean re-index with incremental re-index.

### 21.2 Source-partition property tests

For every fixture:

```text
segments are sorted
segments begin at byte zero
segments end at file length
adjacent ranges meet exactly
no ranges overlap
sum of byte lengths equals source byte length
concatenated slices equal source bytes
all ranges are valid UTF-8 boundaries when text slicing is requested
unknown regions are retained
```

Test:

- LF and CRLF;
- Unicode identifiers and emoji in comments/strings;
- files without a final newline;
- empty files;
- one-line files containing several categories;
- nested functions and closures;
- comments inside and outside entities;
- malformed syntax with ERROR and MISSING nodes;
- mixed-language files and injections;
- generated and preprocessor-heavy files;
- very large files and cancellation.

### 21.3 Visibility fixtures by language

At minimum:

- Rust private ancestor plus `pub` child, `pub use`, `pub(crate)`, `pub(super)`, feature-gated API;
- TypeScript internal export, barrel re-export, package export, conditional package export, CommonJS mutation;
- Go uppercase/lowercase identifiers and promoted methods;
- Java public class in exported and non-exported module packages;
- C/C++ public/private headers, `static`, export macros, public/protected/private methods;
- Python literal and computed `__all__`, package re-export, underscore convention, dynamic import;
- Ruby visibility changes before and after method definitions;
- SQL grants that differ between migration text and live catalog;
- shell sourced and exported functions.

### 21.4 Public projection tests

- private nodes are absent by default;
- public nodes retain stable positions between snapshots where possible;
- direct and collapsed calls have different labels;
- collapsed calls retain witness paths;
- backward call results exactly invert the selected forward edge set;
- a private shared helper change maps to all known public callers;
- unresolved dynamic calls create limitations, not fabricated edges;
- changing observer boundary changes the projection predictably.

### 21.5 Semantic oracle tests

Where available, compare syntax-derived facts against:

- rust-analyzer/rustdoc/SCIP for Rust;
- TypeScript compiler or SCIP for TypeScript;
- `go list`/compiler information for Go;
- Java/Kotlin/Scala compiler indexes;
- clang/SCIP and link symbols for C/C++;
- runtime/database catalog evidence for SQL.

Tree-sitter remains the broad, tolerant baseline. Stronger tools become evidence enrichers and test oracles.

## 22. Migration From Current Parseltongue

### Phase 0: Freeze and measure current semantics

- preserve current API fixtures;
- document current line-key behavior;
- measure collisions and key churn;
- measure files where word coverage exceeds 100 percent;
- inventory unsupported visibility and language-pack capabilities;
- record full versus incremental drift.

### Phase 1: Introduce immutable file revisions and byte spans

- hash exact source bytes;
- assign `file_revision_id`;
- preserve byte and point ranges;
- stop treating line range as identity;
- retain current APIs through an adapter.

### Phase 2: Build the Source Accounting Ledger

- implement disjoint partitioning;
- store parse errors and unknown spans;
- derive current raw/effective word metrics from the ledger for compatibility;
- add exact invariant tests.

### Phase 3: Introduce stable symbol and occurrence tables

- assign opaque `symbol_id` values;
- move source locations to occurrences;
- add canonical descriptors and separate fingerprints;
- record reconciliation decisions;
- migrate or rebuild existing indexes explicitly.

### Phase 4: Ship two high-confidence public-surface packs

Recommended first pair:

1. Rust, because visibility and crate boundaries are central to Parseltongue itself.
2. TypeScript/JavaScript, because exports, package boundaries, routes, and agent-generated application changes are common.

Use compiler/SCIP comparison fixtures before broad claims.

### Phase 5: Add the public graph projection

- default to exposed signatures;
- collapse private paths;
- expose forward/backward calls and dependencies;
- attach implementation-change capsules;
- disclose observer boundary, confidence, freshness, and unknowns.

### Phase 6: Expand language packs by proven demand

Add C/C++, Python, Go, JVM/.NET, Swift, Ruby/PHP, SQL, and shell according to user workflow pull and available semantic oracles.

Do not claim uniform support merely because a grammar parses declarations.

## 23. Product-Market Judgment

### 23.1 The differentiated object is not a code graph

Many tools can produce symbols, references, calls, and search context.

The differentiated object could be:

> A versioned, visually stable proof of which observable contracts changed, which hidden implementation changed behind them, who depends on them, and which source regions remain unknown.

That is closer to the trust problem described in the preceding discussion than "fastest search."

### 23.2 Public-only display improves trust calibration

A reviewer sees fewer nodes, but each node has stronger meaning. The source ledger prevents simplicity from becoming false confidence.

The combination is important:

```text
small visual surface without accounting = potentially misleading
complete source accounting without a public projection = cognitively expensive
public projection plus exact accounting = focused and auditable
```

### 23.3 The user segment becomes sharper

The strongest initial user is:

```text
a maintainer supervising substantial coding-agent changes in a typed or package-structured repository, who needs to decide whether the agent changed a shared contract in the intended direction
```

The reviewer does not need every private function in the first view. They need proof that the hidden implementation has been accounted for and can be inspected immediately.

## 24. Decisions Reached

### Product decisions

1. Public interface signatures should be the default architecture nodes.
2. Dependency, forward-call, and backward-call views are sufficient first graph workflows.
3. Private code should be hidden by default, not discarded.
4. Changed private code should be attributed to affected public contracts.
5. Unknown and unresolved regions are first-class product output.

### Identity decisions

1. Line spans are mutable occurrence coordinates.
2. Content hashes are fingerprints, not stable symbol IDs.
3. Stable symbols need opaque durable lineage IDs.
4. Canonical descriptors support matching and interoperability.
5. Signature, body, structural, exact source, and documentation hashes remain separate.
6. Ambiguous reconciliation must be recorded, not forced.

### Source-accounting decisions

1. Bytes are the canonical additive unit.
2. Lines are derived views and may overlap by category.
3. LLM tokens require a tokenizer ID and version.
4. Source segments must be disjoint, gap-free, and reconstruct the file exactly.
5. Nested semantic ownership is stored separately from additive accounting.

### Visibility decisions

1. Publicness is relative to an observer boundary.
2. Language syntax is one evidence source, not universal truth.
3. Package manifests, re-exports, build configuration, compiler indexes, headers, runtime catalogs, and framework registration may be required.
4. Every language pack must declare capabilities and limitations.

### Visualization decisions

1. Containment is internal evidence, not the primary visual graph.
2. Private paths can be collapsed only with directness labels and witnesses.
3. Stable layout should prioritize changed public contracts.
4. A reviewer can expand exact source evidence without leaving the change map.

## 25. Open Questions

1. What is the first observer boundary: crate/package consumer, workspace consumer, external network/CLI user, or all as selectable projections?
2. Should tests be canonical private implementation, a separate verification projection, or public contracts when they are externally consumed fixtures?
3. Which exact canonical-signature normalization rules should be version 1 for Rust and TypeScript?
4. Should `symbol_id` use UUIDv7, ULID, a database integer with repository namespace, or another opaque format?
5. How should identity reconcile across branches that independently observe the same new symbol?
6. Should private call nodes be fully persisted or derived on demand from source/semantic indexes?
7. Which private-path length or fan-out should trigger an implementation capsule in the default view?
8. How should generated code and source contracts share identity?
9. Which runtime evidence sources are safe and affordable for Python, Ruby, SQL, and shell?
10. How much compiler-backed enrichment is required before Parseltongue may label a call graph "verified"?
11. Should documentation be part of contract compatibility for APIs whose behavior is defined mainly through prose?
12. What latency budget keeps the Change Map interactive without sacrificing candidate-revision validation?

## 26. Verification Questions And Answers

### Did historical Parseltongue use line spans in entity keys?

Yes. The current README still documents a line-range suffix, and ISGL1 v2 documents the cascading false-change problem caused by line movement.

### Did ISGL1 v2 make content hash the primary key?

No. The design proposed a timestamp-bearing key, semantic path for matching, and a separate content-hash field for change detection.

### Is the current birth timestamp actually assigned at birth?

No. Current code deterministically hashes file path and entity name, then maps that value into a timestamp-shaped range.

### Does current code already model visibility?

Partly at the domain-type level, but not in the principal query extraction path. `InterfaceSignature` has a `Visibility` enum, while `QueryBasedExtractor::ParsedEntity` does not populate it and current `.scm` entity queries mostly capture declarations and names.

### Does current word coverage exactly account for a file?

No. It sums words across extracted entity bodies and subtracts selected imports/comments for an effective ratio. Nested ranges can overlap, and unknown spans are not stored as a disjoint ledger.

### Can a disjoint byte ledger exactly account for the source?

Yes, mechanically. Every byte can be placed in one primary segment, including bytes whose semantic category is unknown. The semantic classification can be imperfect while the accounting remains exact.

### Can publicness be determined equally for all languages?

No. Explicit visibility is strong evidence in some languages, while package reachability, headers, build manifests, runtime registration, naming conventions, privileges, or compiler resolution are necessary in others.

### Can private implementation be removed entirely from the model?

Not safely for a trust-oriented change reviewer. It may be removed from the default projection, but enough implementation facts must remain to attribute changes, calls, dependencies, and evidence to public contracts.

### Is containment unnecessary?

It is unnecessary as the default visual architecture. It remains useful internally for effective visibility, owner identity, source attribution, call containment, and collapsed-path construction.

### Are forward and backward calls separate facts?

No. They are opposing traversals over one directed caller-to-callee fact.

## 27. Local Evidence Map

| Local source | Contribution to this thesis |
|---|---|
| [README.md](../../README.md) | Current API, entity-key format, language claims, coverage endpoints, forward/backward workflows |
| [ISGL1 v2 Stable Entity Identity](../research001/unclassified/ISGL1-v2-Stable-Entity-Identity.md) | Line-shift failure, proposed timestamp identity, semantic path, content hash, matching |
| [Incremental Indexing Architecture](../research000/archive/archive-docs-v2/archive-p2/D04_Incremental_Indexing_Architecture.md) | Primary-key rationale, matching simulations, duplicates, move/change behavior |
| [Interface Signature Graph Visualization](../research000/archive/archive-web-ui/INTERFACE_SIGNATURE_GRAPH_THESIS.md) | Public-method progressive disclosure and visualization jobs |
| [v1.6.5 Ingestion Diagnostics PRD](../research000/archive/v165/PRD-v165.md) | Raw/effective word coverage and the question "what did parsing miss?" |
| [v1.6.1 Ingestion Coverage PRD](../research000/archive/v161/v161-PRD-ingestion-coverage-report.md) | File/folder parse coverage and agent confidence requirements |
| [Tree-sitter reference 2026](../research002/tree-sitter-ref-202606.md) | Public-interface projection, stable symbol/occurrence model, language slices, evidence and confidence |
| [Tree-sitter patterns 1](../research003/tree-sitter-patterns-1.md) | Byte-span provenance, source versioning, structural hashes, parser/query risks |
| [Tree-sitter patterns 5](../research003/tree-sitter-patterns-5.md) | ERROR/MISSING nodes, parse quality, fixture and parity testing |
| [Sol-Observations-01](Sol-Observations-01.md) | Stable identity remains unsettled; identity, matching, and location must separate |
| [Sol-Insights-02](Sol-Insights-02.md) | Architecture Snapshot Format, visual trust, exact graph facts, backend neutrality |
| [`entities.rs`](../../crates/parseltongue-core/src/entities.rs) | Current language, visibility, line-range, entity, and word-coverage models |
| [`isgl1_v2.rs`](../../crates/parseltongue-core/src/isgl1_v2.rs) | Current deterministic pseudo-timestamp, exact SHA-256 content hash, line-tolerance matcher |
| [`query_extractor.rs`](../../crates/parseltongue-core/src/query_extractor.rs) | Current 12-language query loading, declaration extraction, calls, comments, imports, and line containment |
| [`entity_queries`](../../entity_queries) | Current declaration query coverage and absence of a normalized visibility contract |
| [`streamer.rs`](../../crates/pt01-folder-to-cozodb-streamer/src/streamer.rs) | Current word-coverage calculation and ingestion integration |
| [SCIP schema](../../git-ref-repo/ignore-this-folder-repos/scip-code__scip/scip.proto) | Symbols, descriptors, signatures, occurrences, ranges, roles, and position encoding |

## 28. External Primary Sources

| Source | Relevant evidence |
|---|---|
| [Rust Reference: visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html) | restricted visibility, ancestor reachability, re-exports |
| [Rust `DefaultHasher`](https://doc.rust-lang.org/std/collections/hash_map/struct.DefaultHasher.html) | hash algorithm is not a cross-release persistence contract |
| [Go Language Specification](https://go.dev/ref/spec) | exported identifier rules and package scope |
| [Java Language Specification: packages and modules](https://docs.oracle.com/javase/specs/jls/se20/html/jls-7.html) | public type plus exported package requirement |
| [ECMAScript modules](https://tc39.es/ecma262/2025/multipage/ecmascript-language-scripts-and-modules.html) | standard export declarations and module semantics |
| [TypeScript modules](https://www.typescriptlang.org/docs/handbook/2/modules.html) | file/module exports and module-local scope |
| [Node.js packages](https://nodejs.org/api/packages.html) | package `exports` as public entry-point boundary |
| [Python modules](https://docs.python.org/3/tutorial/modules.html) | module namespaces, underscore exclusion, `__all__` |
| [Python private variables](https://docs.python.org/3/tutorial/classes.html#private-variables) | convention rather than enforced instance privacy |
| [C23 working draft](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n3096.pdf) | external, internal, and no-linkage rules for C identifiers |
| [C++ access control](https://eel.is/c++draft/class.access.general) | public/protected/private member semantics |
| [C++ linkage](https://eel.is/c++draft/basic.link) | external, module, internal, and no linkage |
| [C++ modules](https://eel.is/c++draft/module) | interface units and exports |
| [Swift access control](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/accesscontrol/) | open/public/package/internal/file/private scopes |
| [Kotlin visibility](https://kotlinlang.org/docs/visibility-modifiers.html) | public default and module/file/class scopes |
| [C# access modifiers](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/access-modifiers) | assembly, inheritance, file, and type accessibility |
| [Scala access modifiers](https://docs.scala-lang.org/scala3/book/domain-modeling-oop.html#access-modifiers) | public default with private/protected restriction |
| [Ruby Module API](https://ruby-doc.org/3.4/Module.html) | runtime method visibility state and modification |
| [PHP visibility](https://www.php.net/manual/en/language.oop5.visibility.php) | public/protected/private members and defaults |
| [PostgreSQL privileges](https://www.postgresql.org/docs/current/ddl-priv.html) | role- and object-relative database exposure |
| [Bash shell functions](https://www.gnu.org/s/bash/manual/html_node/Shell-Functions.html) | sourcing, dynamic scope, and function export |
| [Tree-sitter grammar guide](https://tree-sitter.github.io/tree-sitter/creating-parsers/3-writing-the-grammar.html) | concrete syntax, fields, hidden rules, extras |
| [Tree-sitter query syntax](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html) | named/anonymous nodes, ERROR and MISSING queries |
| [Tree-sitter advanced parsing](https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html) | incremental edits, byte/point ranges, reused structure |

## 29. Final Thesis

Parseltongue should not try to make every parsed entity look architectural.

Its default architecture should be a **Public Interface Signature Graph**:

```text
observable contracts
dependency edges
forward calls
backward calls
before/after signature and exposure changes
```

Private implementation should not clutter that view. But private implementation is not inert, so Parseltongue must retain enough hidden facts to answer:

```text
Which public contracts does this private change affect?
What new dependency or call path did it introduce?
What exact source evidence supports that answer?
What could the system not resolve?
```

The source side should be governed by a **Source Accounting Ledger** whose disjoint byte ranges sum exactly to the file. Public signatures, restricted signatures, private signatures, implementation, comments, imports, errors, whitespace, generated material, embedded languages, and unknown syntax all receive explicit accounting.

The identity rule is equally clear:

```text
line span        = current location
content hash     = version evidence
signature hash   = contract version
body hash        = implementation version
symbol_id        = durable logical identity
occurrence       = symbol at a source range in one file revision
```

This gives Parseltongue a small architecture that humans can see and a complete evidence substrate that LLMs can explore.

That combination is the product opportunity:

> Show less, account for everything, and make every trust claim inspectable.
