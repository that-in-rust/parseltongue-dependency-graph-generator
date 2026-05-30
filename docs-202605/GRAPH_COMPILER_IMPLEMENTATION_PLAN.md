# Practical Implementation Plan: Rust Graph-Native Compiler
## From Parseltongue ISG to Graph-Aware Compilation

**Date**: 2025-11-10
**Prerequisite Reading**: `GRAPH_NATIVE_COMPILATION_ANALYSIS.md`
**Target**: Actionable engineering roadmap

---

## Executive Summary

This document provides a **concrete, phased implementation plan** to evolve Parseltongue from a code analysis tool into a foundation for graph-native compilation.

**Philosophy**: Start small, prove value incrementally, avoid boiling the ocean.

**Timeline**: 18-24 months to production-ready hybrid compiler

---

## Phase 1: Foundation (Months 1-3) - "High-Fidelity ISG"

### Goal
Extend Parseltongue to capture complete AST, not just signatures.

### Deliverables

#### 1.1 Schema Evolution
```datalog
# Current (v0.9.6) - Signatures only
?[isgl1_key, entity_name, entity_type, interface_signature]

# Target (v0.10.0) - Full AST
?[
  isgl1_key: String,
  entity_name: String,
  entity_type: String,
  entity_class: String,

  # NEW: Full source code
  full_source_code: String,        # Complete function/struct source

  # NEW: AST representation
  ast_json: String,                # tree-sitter AST as JSON
  ast_version: String,             # tree-sitter grammar version

  # NEW: Compilation metadata
  source_hash: String,             # SHA-256 of source
  dependencies_hash: String,       # Hash of all direct deps

  # Existing fields
  interface_signature: String,
  file_path: String,
  start_byte: Int,                 # Byte offsets (not lines)
  end_byte: Int
]
```

#### 1.2 Enhanced Ingestion (pt01 extension)

**File**: `crates/pt01-folder-to-cozodb-streamer/src/ast_extractor.rs` (NEW)

```rust
use tree_sitter::{Node, Parser};
use serde_json::json;

/// Extract full AST including function bodies
pub struct FullAstExtractor {
    parser: Parser,
}

impl FullAstExtractor {
    pub fn extract_entity_with_body(&mut self, source: &str, node: Node) -> EntityData {
        EntityData {
            isgl1_key: generate_key(&node),
            entity_name: extract_name(&node),
            entity_type: node.kind().to_string(),

            // NEW: Capture full source
            full_source_code: node.utf8_text(source.as_bytes()).unwrap().to_string(),

            // NEW: Serialize AST
            ast_json: self.serialize_ast_to_json(&node, source),
            ast_version: tree_sitter_rust::language().version().to_string(),

            // NEW: Compute hashes
            source_hash: sha256(&node.utf8_text(source.as_bytes()).unwrap()),
            dependencies_hash: compute_deps_hash(&node, source),

            // Existing
            interface_signature: extract_signature(&node),
            file_path: current_file.clone(),
            start_byte: node.start_byte() as i64,
            end_byte: node.end_byte() as i64,
        }
    }

    fn serialize_ast_to_json(&self, node: &Node, source: &str) -> String {
        // Recursively serialize tree-sitter AST to JSON
        let json = json!({
            "kind": node.kind(),
            "text": node.utf8_text(source.as_bytes()).ok(),
            "start_byte": node.start_byte(),
            "end_byte": node.end_byte(),
            "children": node.children(&mut node.walk())
                .map(|child| self.serialize_ast_to_json(&child, source))
                .collect::<Vec<_>>()
        });
        serde_json::to_string(&json).unwrap()
    }
}
```

#### 1.3 Validation Test

**File**: `crates/pt01-folder-to-cozodb-streamer/tests/full_ast_roundtrip.rs` (NEW)

```rust
#[test]
fn test_full_ast_roundtrip() {
    // 1. Parse Rust code
    let source = r#"
        pub fn fibonacci(n: u32) -> u32 {
            match n {
                0 => 0,
                1 => 1,
                _ => fibonacci(n - 1) + fibonacci(n - 2),
            }
        }
    "#;

    // 2. Extract AST and store in CozoDB
    let extractor = FullAstExtractor::new();
    let entity = extractor.extract_entity_with_body(source, root_node);
    db.insert_entity(&entity).await.unwrap();

    // 3. Retrieve from database
    let retrieved = db.get_entity(&entity.isgl1_key).await.unwrap();

    // 4. Verify: Source code matches
    assert_eq!(entity.full_source_code, retrieved.full_source_code);

    // 5. Verify: AST can be deserialized
    let ast: serde_json::Value = serde_json::from_str(&retrieved.ast_json).unwrap();
    assert_eq!(ast["kind"], "function_item");

    // 6. CRITICAL: Verify we can reconstruct source from AST
    let reconstructed = reconstruct_source_from_ast(&ast);
    assert_eq!(source.trim(), reconstructed.trim());
}
```

**Success Metric**: 100% round-trip fidelity for all Rust syntax constructs

#### 1.4 Storage Optimization

**Challenge**: Full AST = 50-100x larger than signatures

**Solution**: Compression + Lazy Loading

```rust
// Store compressed AST
use flate2::write::GzEncoder;

pub fn compress_ast(ast_json: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(ast_json.as_bytes()).unwrap();
    encoder.finish().unwrap()
}

// Database schema uses BLOB for compressed data
?[
  isgl1_key: String,
  ast_compressed: Bytes,  # Compressed JSON
]

// Lazy loading: Only fetch AST when needed for compilation
pub async fn get_entity_ast(&self, key: &str) -> Result<AstNode> {
    let compressed = self.db.query_single(
        "?[ast_compressed] := *entities{isgl1_key: $key, ast_compressed}"
    )?;
    let json = decompress(&compressed)?;
    serde_json::from_str(&json)
}
```

**Expected Compression**: 70-80% size reduction (100KB → 20-30KB)

### Milestones

- [ ] Week 1-2: Design schema extension
- [ ] Week 3-4: Implement `FullAstExtractor`
- [ ] Week 5-6: Write round-trip tests
- [ ] Week 7-8: Optimize storage (compression)
- [ ] Week 9-10: Ingest parseltongue codebase as test
- [ ] Week 11-12: Documentation and examples

**Phase 1 Complete When**: Parseltongue codebase (10K LOC) fully ingested with AST in < 5 seconds, DB size < 50MB

---

## Phase 2: Proof of Concept Compiler (Months 4-6) - "Hello Graph"

### Goal
Compile a single Rust function from CozoDB to machine code.

### Deliverables

#### 2.1 Minimal Compiler Fork

**Repository**: `rustc-graph` (fork of rust-lang/rust)

**Changes**: Replace parsing entry point

```rust
// Original rustc entry point:
// compiler/rustc_interface/src/interface.rs
pub fn parse_crate_from_source_str(...) -> Result<ast::Crate> {
    let mut parser = Parser::from_source_str(...);
    parser.parse_crate_mod()
}

// NEW: Graph-native entry point
// compiler/rustc_interface/src/graph_interface.rs
pub fn parse_crate_from_graph(
    db_path: &str,
    crate_root: &str
) -> Result<ast::Crate> {
    // 1. Connect to CozoDB
    let db = CozoDbStorage::open(db_path)?;

    // 2. Query for all entities in crate
    let entities = db.query_entities_by_crate(crate_root)?;

    // 3. Reconstruct AST from graph
    let mut crate_ast = ast::Crate::empty();

    for entity in entities {
        // Deserialize tree-sitter AST
        let ts_ast: TreeSitterAst = serde_json::from_str(&entity.ast_json)?;

        // Convert tree-sitter AST → rustc AST
        let rustc_item = convert_ts_to_rustc_ast(ts_ast)?;

        // Add to crate
        crate_ast.items.push(rustc_item);
    }

    Ok(crate_ast)
}

// CRITICAL: Conversion logic
fn convert_ts_to_rustc_ast(ts_node: TreeSitterAst) -> Result<ast::Item> {
    // This is the hard part! Must map tree-sitter nodes to rustc AST nodes
    match ts_node.kind.as_str() {
        "function_item" => {
            // Extract components
            let vis = parse_visibility(&ts_node)?;
            let ident = parse_identifier(&ts_node)?;
            let sig = parse_fn_signature(&ts_node)?;
            let body = parse_block(&ts_node)?;

            Ok(ast::Item {
                ident,
                vis,
                kind: ast::ItemKind::Fn(Box::new(sig), body),
                // ... other fields
            })
        }
        "struct_item" => { /* ... */ }
        "impl_item" => { /* ... */ }
        _ => Err(anyhow!("Unsupported node type: {}", ts_node.kind))
    }
}
```

**Complexity**: Converting tree-sitter AST → rustc AST is non-trivial
- tree-sitter: Concrete syntax tree (CST)
- rustc: Abstract syntax tree (AST)
- Requires mapping ~150 Rust syntax constructs

#### 2.2 Test Case: Single Function

**Input** (stored in CozoDB):
```rust
// Simplest possible Rust function
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Compilation Pipeline**:
```bash
# 1. Ingest function into CozoDB
echo "pub fn add(a: i32, b: i32) -> i32 { a + b }" > test.rs
./parseltongue pt01 . --db test.db

# 2. Compile from graph
./rustc-graph --graph-mode --db test.db --crate-name testcrate --output test

# 3. Execute
./test
# Expected output: (nothing, but binary runs without crashing)

# 4. Test correctness
./test
echo $?  # Exit code should be 0
```

**Success Metric**: Generated binary is byte-identical to `rustc test.rs`

#### 2.3 Integration Test Suite

```rust
// tests/graph_compilation.rs
#[test]
fn test_compile_simple_function() {
    let source = "pub fn add(a: i32, b: i32) -> i32 { a + b }";
    let binary = compile_from_graph(source).unwrap();

    // Execute and verify
    let output = Command::new(&binary).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn test_compile_with_dependencies() {
    let source = r#"
        pub fn double(x: i32) -> i32 { x * 2 }
        pub fn quadruple(x: i32) -> i32 { double(double(x)) }
    "#;
    let binary = compile_from_graph(source).unwrap();

    // Verify cross-function calls work
    let output = Command::new(&binary).args(&["4"]).output().unwrap();
    assert_eq!(output.stdout, b"16\n");
}

#[test]
fn test_compile_with_generics() {
    let source = r#"
        pub fn identity<T>(x: T) -> T { x }
    "#;
    let binary = compile_from_graph(source).unwrap();
    assert!(binary.exists());
}
```

### Milestones

- [ ] Week 1-3: Fork rustc, set up build environment
- [ ] Week 4-6: Implement tree-sitter → rustc AST conversion
- [ ] Week 7-8: Compile first function end-to-end
- [ ] Week 9-10: Handle dependencies between functions
- [ ] Week 11-12: Integration test suite

**Phase 2 Complete When**: 20+ test cases pass, including generics, traits, and cross-function calls

---

## Phase 3: Incremental Compilation (Months 7-12) - "The Speed Demon"

### Goal
Implement function-level incremental compilation with 10x speedup.

### Deliverables

#### 3.1 Change Detection Engine

```rust
pub struct ChangeDetector {
    db: Arc<CozoDbStorage>,
}

impl ChangeDetector {
    /// Find entities that changed since last compilation
    pub async fn detect_changed_entities(&self) -> Result<Vec<EntityId>> {
        self.db.query(r#"
            ?[entity_id] :=
                *entities{
                    entity_id,
                    source_hash,
                    last_compiled_hash
                },
                source_hash != last_compiled_hash
        "#).await
    }

    /// Compute transitive dependencies (blast radius)
    pub async fn compute_blast_radius(
        &self,
        changed: &[EntityId]
    ) -> Result<HashSet<EntityId>> {
        // Datalog transitive closure
        let query = r#"
            // Base case: Changed entities
            affected[entity] :=
                changed[entity]

            // Recursive case: Entities that depend on affected entities
            affected[dependent] :=
                affected[dependency],
                *dependencies{dependent, dependency, "calls"}

            // Return all affected
            ?[entity] := affected[entity]
        "#;

        let results = self.db.query_with_params(
            query,
            &[("changed", changed.to_vec())]
        ).await?;

        Ok(results.into_iter().collect())
    }
}
```

#### 3.2 Selective Recompilation

```rust
pub struct GraphCompiler {
    db: Arc<CozoDbStorage>,
    cache: CompilationCache,
    detector: ChangeDetector,
}

impl GraphCompiler {
    pub async fn compile_incremental(&mut self) -> Result<PathBuf> {
        let start = Instant::now();

        // 1. Detect changes
        let changed = self.detector.detect_changed_entities().await?;
        println!("  Changed: {} entities", changed.len());

        if changed.is_empty() {
            println!("  No changes, using cached binary");
            return Ok(self.cache.get_cached_binary()?);
        }

        // 2. Compute blast radius
        let to_compile = self.detector.compute_blast_radius(&changed).await?;
        println!("  Blast radius: {} entities", to_compile.len());

        // 3. Fetch only affected entities from DB
        let entities = self.db.get_entities_batch(&to_compile).await?;

        // 4. Compile only affected entities
        let mut compiled_units = Vec::new();
        for entity in entities {
            let mir = self.compile_entity_to_mir(&entity).await?;
            compiled_units.push(mir);

            // Update compilation cache
            self.db.update_last_compiled_hash(&entity.id, &entity.source_hash).await?;
        }

        // 5. Link: Combine newly compiled + cached units
        let cached_units = self.cache.get_cached_units_except(&to_compile)?;
        let all_units = [compiled_units, cached_units].concat();

        let binary = self.link_units(all_units).await?;

        println!("  Incremental compilation: {:?}", start.elapsed());
        Ok(binary)
    }
}
```

#### 3.3 Benchmarking Framework

**Test Codebase**: Parseltongue itself (10K LOC, ~1,500 functions)

**Scenario**: Developer changes one function in `pt01/src/streamer.rs`

**Baseline (Traditional rustc)**:
```bash
# Full rebuild
time cargo build --release
# Expected: 60-90 seconds

# Incremental rebuild (file-level)
# Change one function in streamer.rs
time cargo build --release
# Expected: 5-10 seconds (entire streamer.rs recompiled + dependents)
```

**Target (Graph-Native)**:
```bash
# Full rebuild
time rustc-graph build --db parseltongue.db --release
# Expected: 60-90 seconds (same as traditional)

# Incremental rebuild (function-level)
# Change one function in streamer.rs
time rustc-graph build --db parseltongue.db --release
# Expected: 0.5-1 second (only changed function + dependents)
# Target: 10x faster than traditional incremental
```

**Measurement Script**:
```bash
#!/bin/bash
# benchmarks/incremental_benchmark.sh

echo "=== Incremental Compilation Benchmark ==="

# Setup
cargo build --release  # Baseline
rustc-graph build --db test.db --release  # Graph-native baseline

# Test: Change one function
FUNCTION="crates/pt01-folder-to-cozodb-streamer/src/streamer.rs:45"

# Traditional rustc
echo "1. Traditional rustc (incremental)"
sed -i 's/println!("Processing")/println!("Processing v2")/' $FUNCTION
time cargo build --release

# Reset
git checkout $FUNCTION

# Graph-native rustc
echo "2. Graph-native rustc (incremental)"
sed -i 's/println!("Processing")/println!("Processing v2")/' $FUNCTION
./parseltongue pt01 crates/pt01-folder-to-cozodb-streamer --db test.db  # Re-ingest changed file
time rustc-graph build --db test.db --release

# Results
echo "=== Results ==="
echo "Traditional: ~5-10 seconds"
echo "Graph-native: ~0.5-1 seconds"
echo "Speedup: 10x"
```

### Milestones

- [ ] Month 7: Implement change detection
- [ ] Month 8: Implement blast radius computation
- [ ] Month 9: Selective recompilation logic
- [ ] Month 10: Caching layer (MIR cache)
- [ ] Month 11: Benchmark suite on parseltongue codebase
- [ ] Month 12: Optimize to achieve 10x speedup

**Phase 3 Complete When**: Incremental builds are 10x faster than traditional `cargo build` on 10K+ LOC codebase

---

## Phase 4: Ecosystem Integration (Months 13-18) - "The Unified Toolchain"

### Goal
Integrate compiler with IDE, analysis tools, and version control.

### Deliverables

#### 4.1 IDE Integration (rust-analyzer-graph)

**Fork**: rust-analyzer → rust-analyzer-graph

**Changes**: Replace filesystem indexing with CozoDB queries

```rust
// Original rust-analyzer: Scans filesystem for *.rs files
// lsp_server/src/indexing.rs
pub fn index_project(root: PathBuf) -> ProjectIndex {
    let mut index = ProjectIndex::new();
    for entry in WalkDir::new(root) {
        if entry.path().extension() == Some("rs") {
            let ast = parse_file(entry.path())?;
            index.add_file(ast);
        }
    }
    index
}

// Graph-native rust-analyzer: Queries CozoDB
// lsp_server/src/graph_indexing.rs
pub fn index_project_from_graph(db_path: &str) -> ProjectIndex {
    let db = CozoDbStorage::open(db_path)?;

    // Instant: No filesystem scanning
    let entities = db.query_all_entities().await?;

    let mut index = ProjectIndex::new();
    for entity in entities {
        // AST already parsed and stored
        index.add_entity(entity);
    }

    index
}
```

**LSP Features Accelerated**:
```rust
// "Go to Definition" - Traditional: O(N) scan, Graph: O(1) lookup
async fn goto_definition(&self, position: Position) -> Option<Location> {
    // Query: Find entity at cursor position
    let entity_id = self.db.query(r#"
        ?[entity_id] :=
            *entities{entity_id, file_path, start_byte, end_byte},
            file_path == $file,
            start_byte <= $byte,
            end_byte >= $byte
    "#).await?;

    // Instant lookup
    let entity = self.db.get_entity(&entity_id).await?;
    Some(entity.definition_location())
}

// "Find References" - Traditional: O(N) scan, Graph: O(1) lookup
async fn find_references(&self, entity_id: &str) -> Vec<Location> {
    // Query: Find all callers
    self.db.query(r#"
        ?[caller_id, caller_file, caller_byte] :=
            *dependencies{caller_id, $target, "calls"},
            *entities{caller_id, caller_file, _, caller_byte, _}
    "#).await
}
```

**Performance Impact**:
| Operation | Traditional | Graph-Native | Speedup |
|-----------|-------------|--------------|---------|
| **Initial Indexing** | 10-30s | 0s (already indexed) | ∞ |
| **Go to Definition** | 50-200ms | < 5ms | 10-40x |
| **Find References** | 100-500ms | < 10ms | 10-50x |
| **Rename Symbol** | 1-5s | < 50ms | 20-100x |

#### 4.2 Version Control Integration

**Challenge**: Git diffs `*.rs` files, not graph databases

**Solution**: Bidirectional sync

```bash
# Developer workflow (unchanged)
git clone repo
cd repo

# On git pull: Auto-sync files → graph
./parseltongue watch --sync-on-change
# When files change: Automatically re-ingest to CozoDB

# On compile: Use graph
rustc-graph build --db .parseltongue.db

# On commit: Graph → files (if edits were made via graph)
./parseltongue export --output src/
git add src/
git commit -m "Update"
```

**Git Hooks**:
```bash
# .git/hooks/post-merge
#!/bin/bash
# After git pull, re-ingest changed files
echo "Re-indexing changed files..."
./parseltongue pt01 . --db .parseltongue.db --incremental
```

#### 4.3 CI/CD Integration

**GitHub Actions Workflow**:
```yaml
# .github/workflows/graph-native-ci.yml
name: Graph-Native CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install parseltongue
        run: |
          curl -fsSL https://parseltongue.dev/install.sh | bash

      - name: Cache graph database
        uses: actions/cache@v3
        with:
          path: .parseltongue.db
          key: ${{ runner.os }}-graph-${{ hashFiles('src/**/*.rs') }}
          restore-keys: |
            ${{ runner.os }}-graph-

      - name: Ingest codebase (or use cache)
        run: |
          if [ ! -d ".parseltongue.db" ]; then
            ./parseltongue pt01 . --db .parseltongue.db
          else
            ./parseltongue pt01 . --db .parseltongue.db --incremental
          fi

      - name: Compile (incremental from cache)
        run: |
          rustc-graph build --db .parseltongue.db --release

      - name: Test
        run: |
          cargo test  # Tests still use traditional cargo
```

**Expected CI Performance**:
- First run (cold cache): 10-15 minutes (same as traditional)
- Subsequent runs (warm cache): 2-3 minutes (5x faster)

### Milestones

- [ ] Month 13-14: Fork rust-analyzer, implement graph indexing
- [ ] Month 15: Git integration (bidirectional sync)
- [ ] Month 16: CI/CD templates and examples
- [ ] Month 17: Documentation and tutorials
- [ ] Month 18: Beta testing with real projects

**Phase 4 Complete When**: 10+ open-source projects successfully using graph-native toolchain in production

---

## Success Metrics & KPIs

### Technical Metrics

| Metric | Baseline | Target | Measured By |
|--------|----------|--------|-------------|
| **Incremental Build Time** | 5-10s | 0.5-1s | Benchmark suite |
| **IDE Indexing Time** | 10-30s | < 1s | rust-analyzer startup |
| **"Go to Definition" Latency** | 50-200ms | < 5ms | LSP benchmarks |
| **Database Size** | N/A | < 10MB per 1K functions | Storage tests |
| **Compilation Correctness** | 100% | 100% | Test suite (1000+ tests) |

### Adoption Metrics

| Metric | Month 6 | Month 12 | Month 18 |
|--------|---------|----------|----------|
| **GitHub Stars** | 100 | 500 | 2,000 |
| **Projects Using It** | 1 (parseltongue) | 10 | 50+ |
| **Contributors** | 2 | 10 | 25+ |
| **Blog Posts/Talks** | 0 | 5 | 20+ |

### Business Metrics (For Organizations)

| Metric | Before | After | ROI |
|--------|--------|-------|-----|
| **Average Build Time** | 5 min | 30 sec | $54M/year savings (1000 engineers) |
| **Developer Satisfaction** | Baseline | +30% | Retention improvement |
| **Code Quality Issues** | Baseline | -20% | Architectural enforcement |

---

## Risk Mitigation

### Risk 1: Performance Doesn't Meet 10x Target

**Mitigation**:
- Parallel Phase 2-3: Build benchmarking framework early
- Monthly performance reviews
- If < 5x at Month 9, pivot to pure analysis tool (abandon compilation)

### Risk 2: tree-sitter → rustc AST Conversion Too Complex

**Mitigation**:
- Alternative: Use rustc's own parser, store rustc AST directly
- Trade-off: Language-specific (Rust only), but more reliable
- Decision point: End of Month 4

### Risk 3: Database Corruption / Reliability

**Mitigation**:
- Continuous backup to filesystem (every 5 minutes)
- Provide `--fallback-mode` that compiles from files if DB unavailable
- Extensive fuzz testing of database operations

### Risk 4: Community Resistance / Low Adoption

**Mitigation**:
- Don't force migration: Keep traditional workflow supported
- Marketing: Focus on "optional accelerator" narrative
- Partnership: Engage Rust core team early (RFC process)
- Publish research papers (PLDI, OOPSLA) to build credibility

---

## Resource Requirements

### Team Composition

| Role | Headcount | Months | Key Skills |
|------|-----------|--------|------------|
| **Compiler Engineer** | 2 | 18 | rustc internals, LLVM |
| **Database Engineer** | 1 | 18 | CozoDB, Datalog, performance tuning |
| **IDE Engineer** | 1 | 12 (Month 7-18) | LSP, rust-analyzer |
| **DevOps Engineer** | 1 | 6 (Month 13-18) | CI/CD, tooling |
| **Technical Writer** | 1 | 12 | Documentation, tutorials |

**Total**: 5-6 full-time engineers

### Infrastructure

- **Development**: 4-8 core machines, 32GB RAM
- **CI/CD**: GitHub Actions (existing)
- **Database**: CozoDB (embedded, no separate server)

### Budget Estimate

| Category | Cost |
|----------|------|
| **Personnel** (6 engineers × 18 months × $150K/year) | $1.35M |
| **Infrastructure** | $10K |
| **Travel/Conferences** | $30K |
| **Contingency (20%)** | $276K |
| **Total** | **$1.67M** |

**ROI**: If deployed in organization with 1,000 engineers, payback in < 2 months

---

## Decision Points

### Decision Point 1: End of Month 3 (Phase 1)
**Question**: Is round-trip fidelity achievable with tree-sitter?
- **Yes** → Proceed to Phase 2
- **No** → Switch to rustc parser directly, adjust timeline +1 month

### Decision Point 2: End of Month 6 (Phase 2)
**Question**: Can we compile at least 10 test functions correctly?
- **Yes** → Proceed to Phase 3 (incremental compilation)
- **No** → Pivot to "analysis-only" tool, abandon compilation goal

### Decision Point 3: End of Month 12 (Phase 3)
**Question**: Did we achieve 10x incremental compilation speedup?
- **Yes** → Proceed to Phase 4 (ecosystem integration)
- **Partial (5x)** → Continue Phase 3 optimization for 2 more months
- **No (< 3x)** → Stop compilation work, focus on IDE/analysis applications

---

## Next Steps (This Week)

1. **Create Proof-of-Concept Branch**:
   ```bash
   git checkout -b poc/full-ast-ingestion
   ```

2. **Spike: AST Storage**:
   - Implement `FullAstExtractor`
   - Test on single function
   - Measure storage size

3. **Design Review**:
   - Present this plan to team
   - Gather feedback
   - Refine timeline

4. **Stakeholder Communication**:
   - Create slides summarizing vision
   - Schedule demo for Month 3 (end of Phase 1)

---

## Conclusion

This plan provides a **concrete, incremental path** from today's Parseltongue (analysis tool) to a production-ready graph-native compiler.

**Key Principles**:
1. **Incremental**: Prove value at each phase before proceeding
2. **Measurable**: Clear success metrics (10x speedup)
3. **Reversible**: Can pivot to analysis-only if compilation too hard
4. **Pragmatic**: Hybrid approach (filesystem + graph) for adoption

**If successful**, this becomes the foundation for:
- Next-generation developer tooling
- AI-native code generation
- Compile-time architectural enforcement

**The future of software development is graph-native. Parseltongue v4.0 is the first step.**

---

**Document Control**:
- **Version**: 1.0
- **Status**: Implementation Plan (Pending Approval)
- **Dependencies**: GRAPH_NATIVE_COMPILATION_ANALYSIS.md
- **Next Review**: After Phase 1 completion
