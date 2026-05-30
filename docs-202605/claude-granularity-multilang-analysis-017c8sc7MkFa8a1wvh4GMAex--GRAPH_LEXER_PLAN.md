# Graph-Based Rust Lexer Implementation Plan

## Mission Overview
Build a revolutionary Rust compiler using CozoDB as the foundational graph database, starting with the Lexing phase. This approach transforms traditional ephemeral token streams into persistent, queryable graph artifacts.

## Repositories Cloned

All relevant repositories have been cloned to `external_repos/`:

1. **CozoDB** (`external_repos/cozo/`)
   - Rust-native graph database using Datalog
   - Supports RocksDB for persistence
   - ~100K QPS for transactional queries
   - Location: https://github.com/cozodb/cozo

2. **rustc** (`external_repos/rustc/`)
   - Official Rust compiler source
   - Key component: `rustc_lexer` crate at `compiler/rustc_lexer/`
   - Provides reference implementation of lexing
   - Location: https://github.com/rust-lang/rust

3. **tree-sitter** (`external_repos/tree-sitter/`)
   - Parser generator for creating syntax trees
   - Used by parseltongue for multi-language support
   - Location: https://github.com/tree-sitter/tree-sitter

4. **rust-analyzer** (`external_repos/rust-analyzer/`)
   - Rust language server implementation
   - Shows how to use dependency graphs for analysis
   - Location: https://github.com/rust-lang/rust-analyzer

## Key Insights from Research

### 1. rustc_lexer Architecture (`external_repos/rustc/compiler/rustc_lexer/`)

**Current Implementation:**
```rust
// Token structure from rustc_lexer
pub struct Token {
    pub kind: TokenKind,  // Type of token (Ident, Keyword, Operator, etc.)
    pub len: u32,         // Length in bytes
}
```

**Token Types (TokenKind enum):**
- Comments: `LineComment`, `BlockComment`
- Identifiers: `Ident`, `RawIdent`, `InvalidIdent`
- Literals: `Literal { kind: LiteralKind, suffix_start: u32 }`
- Symbols: `Semi`, `Comma`, `Dot`, `OpenParen`, `CloseParen`, etc.
- Whitespace: `Whitespace`
- Keywords: Handled at parser level, lexer treats as `Ident`

**Key Design Principles:**
1. **No error reporting** - errors stored as flags on tokens
2. **Operates on `&str`** - no spans, pure text processing
3. **Stateless** - can be called incrementally
4. **Fast** - optimized for O(n) scanning

### 2. CozoDB Architecture (`external_repos/cozo/`)

**Basic Usage Pattern:**
```rust
use cozo::{DbInstance, ScriptMutability};

// Create database (in-memory or RocksDB-backed)
let db = DbInstance::new("rocksdb", "path/to/db", Default::default()).unwrap();

// Define schema with Datalog
let schema = r#"
    :create token {
        id: Uuid =>
        kind: String,
        text: String,
        span_start: Int,
        span_end: Int
    }
"#;
db.run_script(schema, Default::default(), ScriptMutability::Mutable).unwrap();

// Insert data
let insert = r#"
    ?[id, kind, text, span_start, span_end] <- [[gen_uuid(), "Ident", "foo", 0, 3]]
    :put token { => * }
"#;
db.run_script(insert, Default::default(), ScriptMutability::Mutable).unwrap();

// Query data
let query = r#"
    ?[id, kind, text] := *token{id, kind, text}
"#;
let result = db.run_script(query, Default::default(), ScriptMutability::Immutable).unwrap();
```

**Performance Characteristics:**
- 100K+ QPS for transactional queries
- 250K+ QPS for read-only queries
- ~50MB memory for 1.6M rows
- Sub-millisecond graph traversals

### 3. Integration Strategy

**Why This Approach is Revolutionary:**

Traditional compiler pipeline:
```
Source Code → Lexer → [Ephemeral Token Stream] → Parser → AST
```

Graph-based pipeline:
```
Source Code → Lexer → [CozoDB Graph] → Parser Query → AST Graph
                           ↓
                    Queryable, Persistent,
                    Incremental Updates
```

**Benefits:**
1. **Incremental Compilation**: Cache tokenized forms, avoid re-lexing unchanged files
2. **Queryable Tokens**: "Find all identifiers shadowing variables" via Datalog
3. **Multi-Language Support**: Same schema for all languages
4. **Real-time Analysis**: IDE features without re-parsing
5. **Distributed Compilation**: Share graph database across machines

## Graph Schema Design for Lexer

### Relations (Tables)

#### 1. `file` relation
```datalog
:create file {
    id: Uuid =>
    path: String,
    hash: String,           // SHA256 of content for change detection
    last_modified: Int,     // Unix timestamp
    language: String        // "rust", "python", etc.
}
```

#### 2. `token` relation
```datalog
:create token {
    id: Uuid =>
    kind: String,           // "Ident", "Keyword", "Literal", etc.
    text: String,           // Raw lexeme
    span_start: Int,        // Start byte offset
    span_end: Int,          // End byte offset
    line: Int,              // Line number (1-indexed)
    column: Int,            // Column number (1-indexed)
    file_id: Uuid,          // Foreign key to file
    created_at: Int         // Unix timestamp
}
```

#### 3. `token_sequence` relation (for ordering)
```datalog
:create token_sequence {
    from_token: Uuid,       // Previous token ID
    to_token: Uuid,         // Next token ID
    =>
    order_index: Int        // Sequence number
}
```

#### 4. `token_metadata` relation (optional, for errors/warnings)
```datalog
:create token_metadata {
    token_id: Uuid =>
    error_type: String?,     // "InvalidIdent", "UnterminatedString", etc.
    warning_type: String?,   // Optional warnings
    suggestion: String?      // Fix suggestions
}
```

### Indexes for Performance

```datalog
::index create file:path { path }
::index create token:file { file_id }
::index create token:position { file_id, span_start }
::index create token_sequence:from { from_token }
```

## Implementation Roadmap

### Phase 1: Proof of Concept (Week 1)
- [ ] Create new crate: `parseltongue-lexer-cozo`
- [ ] Implement basic Rust lexer using `rustc_lexer` as reference
- [ ] Store tokens in CozoDB with schema above
- [ ] Benchmark: Compare speed vs in-memory lexer
- [ ] Target: <2x slowdown for initial lex, >10x speedup for re-query

### Phase 2: Incremental Lexing (Week 2)
- [ ] Implement file hash checking
- [ ] Skip lexing for unchanged files
- [ ] Implement partial re-lexing for changed regions
- [ ] Target: <100ms for 100K LOC codebase (after initial index)

### Phase 3: Query Interface (Week 3)
- [ ] Design Datalog queries for common patterns:
  - Find all tokens of type X
  - Get tokens in span range
  - Find identifiers by name
  - Detect suspicious patterns (e.g., very long identifiers)
- [ ] Create Rust API wrapper for queries
- [ ] Integration tests with parseltongue

### Phase 4: Multi-Language Support (Week 4)
- [ ] Extend to Python, JavaScript, TypeScript
- [ ] Unified token schema across languages
- [ ] Language-specific metadata handling

### Phase 5: Advanced Features (Month 2)
- [ ] Parallel lexing for large codebases
- [ ] Distributed database for multi-machine compilation
- [ ] LLM integration: Token-level code suggestions
- [ ] IDE protocol: LSP integration for real-time feedback

## Example: Lexing a Simple Rust File

**Input (`example.rs`):**
```rust
fn main() {
    let x = 42;
}
```

**Generated Graph in CozoDB:**

**file table:**
| id | path | hash | last_modified | language |
|----|------|------|---------------|----------|
| uuid1 | "example.rs" | "sha256..." | 1700000000 | "rust" |

**token table:**
| id | kind | text | span_start | span_end | line | col | file_id |
|----|------|------|------------|----------|------|-----|---------|
| t1 | "Keyword" | "fn" | 0 | 2 | 1 | 1 | uuid1 |
| t2 | "Whitespace" | " " | 2 | 3 | 1 | 3 | uuid1 |
| t3 | "Ident" | "main" | 3 | 7 | 1 | 4 | uuid1 |
| t4 | "OpenParen" | "(" | 7 | 8 | 1 | 8 | uuid1 |
| t5 | "CloseParen" | ")" | 8 | 9 | 1 | 9 | uuid1 |
| t6 | "Whitespace" | " " | 9 | 10 | 1 | 10 | uuid1 |
| t7 | "OpenBrace" | "{" | 10 | 11 | 1 | 11 | uuid1 |
| ... | ... | ... | ... | ... | ... | ... | ... |

**token_sequence table:**
| from_token | to_token | order_index |
|------------|----------|-------------|
| t1 | t2 | 1 |
| t2 | t3 | 2 |
| t3 | t4 | 3 |
| ... | ... | ... |

**Query Examples:**

1. **Get all tokens in order:**
```datalog
?[kind, text] :=
    *token_sequence{from: prev, to: next, order_index},
    *token{id: next, kind, text},
    sorted by order_index
```

2. **Find all identifiers:**
```datalog
?[text, line, col] :=
    *token{kind: "Ident", text, line, column: col}
```

3. **Count tokens by type:**
```datalog
?[kind, count(id)] :=
    *token{id, kind},
    grouped by kind
```

## Next Steps

1. **Create the crate structure** in `crates/parseltongue-lexer-cozo/`
2. **Implement minimal lexer** using rustc_lexer + CozoDB
3. **Write benchmarks** comparing with traditional approach
4. **Document API** for other parseltongue components
5. **Integrate with existing parseltongue tools** (pt01, pt02, etc.)

## Questions to Resolve

1. **Storage overhead**: How much disk space per token? (Estimate: ~100 bytes)
2. **Transaction boundaries**: Batch inserts vs. per-token inserts?
3. **Concurrency**: Multiple files lexed in parallel?
4. **Backwards compatibility**: Keep existing parseltongue API?
5. **Migration path**: How to move from current tree-sitter to this?

## Success Metrics

- **Performance**: <2x slowdown vs traditional lexer on first pass
- **Incremental**: >10x speedup on re-lex of unchanged files
- **Query speed**: <10ms for common queries on 100K token database
- **Memory**: <100MB for 1M token database
- **Scalability**: Support 10M+ tokens (entire Linux kernel)

---

**Status**: Planning phase complete, ready to implement Phase 1.

**Next Action**: Create `crates/parseltongue-lexer-cozo/` and implement basic lexer.
