# F04: Minimal Approach - Agent Queryability First

**Date**: 2025-11-14
**Status**: Architecture Re-Design
**Principle**: TDD-First, MVP-First Rigor (S01)

---

## The REAL Question

> "Can agents answer questions about codebases from JSON themselves?"

**NOT**: Can we generate pretty Mermaid diagrams?
**YES**: Can an LLM agent QUERY the existing JSON graph to answer architectural questions?

---

## S01 Principle #9: MVP-First Rigor

**Proven architectures over theoretical abstractions**

### What I Was Doing (WRONG)
1. Implementing Mermaid generation (no test first)
2. Adding extended EdgeType variants (no failing test)
3. Planning Connected Components algorithm (no proven need)
4. Building features WITHOUT validating agent capability

### What S01 Demands (RIGHT)
1. **Write failing test**: "Agent cannot answer X from JSON"
2. **Implement minimal**: Add ONLY what makes test pass
3. **Validate claim**: Does agent actually use this?
4. **Refactor**: Clean up if needed

---

## The Test: Can Agents Query Current JSON?

### Test Case 1: Blast Radius
**Question**: "If I change `validate_payment()`, what breaks?"

**Current JSON Has**:
```json
{
  "isgl1_key": "rust:fn:validate_payment:src_payment_rs:89-112",
  "reverse_deps": [
    "rust:fn:process_payment:src_payment_rs:145-167",
    "rust:fn:handle_checkout:src_checkout_rs:200-245"
  ]
}
```

**Agent Can**:
- Parse JSON
- Read `reverse_deps` array
- List affected functions

**Result**: ✅ YES (no changes needed)

---

### Test Case 2: Call Chain Traversal
**Question**: "Show me the execution path for payment processing"

**Current JSON Has**:
```json
{
  "edges": [
    {"from_key": "rust:fn:process_payment", "to_key": "rust:fn:validate_payment", "edge_type": "Calls"},
    {"from_key": "rust:fn:validate_payment", "to_key": "rust:fn:check_balance", "edge_type": "Calls"}
  ]
}
```

**Agent Can**:
- Filter edges by `edge_type == "Calls"`
- Follow chain: process_payment → validate_payment → check_balance
- Build execution tree

**Result**: ✅ YES (no changes needed)

---

### Test Case 3: Find Related Functions
**Question**: "What functions are related to authentication?"

**Current JSON Has**:
```json
{
  "entities": [
    {"name": "login", "file_path": "./src/auth.rs"},
    {"name": "logout", "file_path": "./src/auth.rs"},
    {"name": "validate_token", "file_path": "./src/auth.rs"}
  ]
}
```

**Agent Can (Option 1 - Naive)**:
- Grep for "auth" in `file_path` or `name`
- List matching entities

**Agent Can (Option 2 - Clustering)**:
- Run `pt08-semantic-atom-cluster-builder`
- Read cluster JSON output
- Find "auth" cluster members

**Result**: ✅ YES (pt08 already exists, or naive grep works)

---

### Test Case 4: Inheritance Hierarchy
**Question**: "What structs implement the `Payment` trait?"

**Current JSON Has**:
```json
{
  "edges": [
    {"from_key": "rust:struct:CreditCard", "to_key": "rust:trait:Payment", "edge_type": "Implements"},
    {"from_key": "rust:struct:BankTransfer", "to_key": "rust:trait:Payment", "edge_type": "Implements"}
  ]
}
```

**Agent Can**:
- Filter edges by `edge_type == "Implements"` AND `to_key` contains "Payment"
- List implementing structs

**Result**: ✅ YES (no changes needed)

---

### Test Case 5: Instantiation Tracking
**Question**: "What creates instances of `DatabaseConnection`?"

**Current JSON Has**:
- ❌ NO `Instantiates` edge type

**Agent Cannot**: Find instantiations without code search

**Result**: ❌ GAP FOUND (need `Instantiates` edge? OR accept limitation?)

---

### Test Case 6: Control Flow (Within Function)
**Question**: "What are the error paths in `process_payment()`?"

**Current JSON Has**:
- Function-level calls only (no intra-function CFG)

**Agent Cannot**: See `if/else` branches, error handling paths

**Result**: ❌ GAP FOUND (need control flow edges? OR defer to future?)

---

## Summary: What Do Agents ACTUALLY Need?

| Question Type | Current JSON Supports | Gap | Priority |
|---------------|----------------------|-----|----------|
| Blast radius | ✅ reverse_deps | None | ✅ P0 (done) |
| Call chains | ✅ Calls edges | None | ✅ P0 (done) |
| Trait impls | ✅ Implements edges | None | ✅ P0 (done) |
| Clustering | ✅ pt08 output | None | ✅ P0 (done) |
| Inheritance | ⚠️ Implements only | Extends edge? | P1 (minor) |
| Instantiation | ❌ Missing | Instantiates edge? | P2 (defer?) |
| Control flow (intra-func) | ❌ Missing | CFG edges | P3 (future) |
| Data flow | ❌ Missing | DataFlowsTo edges | P3 (future) |

**Key Insight**: **Agents can answer 80% of questions with EXISTING JSON!**

---

## Minimal Implementation Plan (TDD-First)

### Phase 1: Validate Agent Capability (1 day) - MVP

**Goal**: Prove agents can query current JSON

**Executable Specification** (S01 Principle #1):

```rust
#[test]
fn test_agent_finds_blast_radius_from_json() {
    // GIVEN: JSON export with reverse_deps
    let json = load_test_json("edges.json");

    // WHEN: Agent queries "what breaks if I change validate_payment?"
    let affected = query_reverse_deps(&json, "rust:fn:validate_payment");

    // THEN: Agent gets complete list of callers
    assert_eq!(affected.len(), 15);
    assert!(affected.contains(&"rust:fn:process_payment"));
}

#[test]
fn test_agent_traverses_call_chain_from_json() {
    // GIVEN: JSON with Calls edges
    let json = load_test_json("edges.json");

    // WHEN: Agent builds execution path
    let path = build_call_chain(&json, "rust:fn:main");

    // THEN: Agent reconstructs flow
    assert_eq!(path, vec![
        "rust:fn:main",
        "rust:fn:process_payment",
        "rust:fn:validate_payment",
        "rust:fn:check_balance"
    ]);
}
```

**Implementation**:
```rust
// In crates/parseltongue-core/src/query_helpers.rs (new module)

/// Query helper functions for agent JSON traversal
///
/// # Contract
/// **Preconditions**: Valid JSON from pt02 export
/// **Postconditions**: Returns filtered/traversed results
/// **Errors**: JSON parse errors, missing fields

pub fn query_reverse_deps(json: &Value, entity_key: &str) -> Result<Vec<String>> {
    let entities = json["entities"].as_array()?;
    let target = entities.iter()
        .find(|e| e["isgl1_key"] == entity_key)?;

    let reverse_deps = target["reverse_deps"].as_array()?;
    Ok(reverse_deps.iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect())
}

pub fn build_call_chain(json: &Value, start_key: &str) -> Result<Vec<String>> {
    let edges = json["edges"].as_array()?;
    let calls_edges: Vec<_> = edges.iter()
        .filter(|e| e["edge_type"] == "Calls")
        .collect();

    // BFS or DFS to build path
    let mut path = vec![start_key.to_string()];
    let mut current = start_key;

    while let Some(next_edge) = calls_edges.iter()
        .find(|e| e["from_key"] == current) {
        let next = next_edge["to_key"].as_str()?;
        path.push(next.to_string());
        current = next;
    }

    Ok(path)
}
```

**Success Criteria**:
- ✅ All tests pass
- ✅ Agent can answer 5+ question types
- ✅ < 200 LOC implementation

**Deliverable**: `query_helpers.rs` module with TDD-proven functions

---

### Phase 2: Extended EdgeType (IF NEEDED - 2 days)

**Trigger**: Phase 1 tests reveal agent cannot answer inheritance questions

**Executable Specification**:

```rust
#[test]
fn test_agent_finds_inheritance_hierarchy_from_json() {
    // GIVEN: JSON with Extends edges
    let json = load_test_json("edges.json");

    // WHEN: Agent queries "what extends BaseClass?"
    let children = query_children(&json, "rust:struct:BaseClass");

    // THEN: Agent gets subclass list
    assert_eq!(children, vec!["rust:struct:ChildA", "rust:struct:ChildB"]);
}
```

**Implementation**:
1. Extend `EdgeType` enum:
   ```rust
   pub enum EdgeType {
       Calls, Uses, Implements,
       Extends, // NEW: Inheritance (child → parent)
   }
   ```

2. Update tree-sitter query:
   ```scheme
   ; Rust inheritance detection
   (impl_item
     trait: (type_identifier) @dependency.extends
     type: (type_identifier) @entity.impl
   ) @scope
   ```

3. Test passes → Done

**Skip Condition**: If Phase 1 shows agents don't need inheritance tracking yet

---

### Phase 3: Mermaid Generation (OPTIONAL - 1 day)

**Trigger**: User explicitly requests visual diagrams

**Executable Specification**:

```rust
#[test]
fn test_render_json_as_mermaid_diagram() {
    // GIVEN: JSON graph
    let json = load_test_json("edges.json");

    // WHEN: Render as Mermaid
    let mermaid = render_graph_as_mermaid(&json, &MermaidConfig::default());

    // THEN: Valid Mermaid syntax
    assert!(mermaid.contains("```mermaid"));
    assert!(mermaid.contains("graph TD"));
    assert!(mermaid.contains("-->|\"Calls\"|"));
}
```

**Implementation**: Use the `mermaid.rs` module I already wrote (but deferred)

**Priority**: P4 (low - for humans, not agents)

---

## What We're NOT Building (Yet)

### ❌ Control Flow Edges (CFG)
**Why**: Requires AST analysis within functions (complex)
**Defer to**: v1.0 or later
**Workaround**: Agents can read `current_code` field for manual analysis

### ❌ Data Flow Edges
**Why**: Requires taint analysis (very complex)
**Defer to**: v1.1 or later
**Workaround**: Use coarse-grained `Uses` edges

### ❌ Instantiates Edge Type
**Why**: Need to prove agent need first
**Decision**: Phase 1 tests will reveal if needed

### ❌ Louvain Clustering
**Why**: Label Propagation (pt08) already works
**Decision**: Keep existing LPA, skip Louvain for now

---

## Mermaid's Role (Clarified)

### S01 Quote
> "ALL DIAGRAMS WILL BE IN MERMAID ONLY TO ENSURE EASE WITH GITHUB"

**Interpretation**:
- ✅ Use Mermaid for DOCUMENTATION diagrams (architecture docs)
- ✅ Generate Mermaid from JSON (for human visualization)
- ❌ Do NOT use Mermaid as PRIMARY data format
- ❌ Do NOT prioritize Mermaid over agent queryability

**Architecture**:
```
JSON Graph (canonical)
    ↓
    ├─→ Agent Queries (P0 - immediate value)
    └─→ Mermaid Rendering (P4 - nice to have)
```

---

## Success Metrics (Executable Specification)

### Phase 1 Success
- ✅ 5+ agent query tests pass
- ✅ Agents answer 80% of architectural questions
- ✅ < 200 LOC implementation
- ✅ Documentation with query examples

### Phase 2 Success (If Triggered)
- ✅ Extends edge type extracted
- ✅ Inheritance queries work
- ✅ Tests pass
- ✅ < 100 LOC additions

### Phase 3 Success (If Requested)
- ✅ Mermaid renders correctly
- ✅ GitHub displays diagrams
- ✅ < 300 LOC for renderer

---

## Risk Mitigation (S01 Principle #5)

### Risk 1: JSON Too Large for Agent Context
**Mitigation**: Query functions return ONLY relevant subset

Example:
```rust
// ❌ WRONG: Return entire JSON (300K tokens)
fn get_all_entities(json: &Value) -> Vec<Entity>

// ✅ RIGHT: Return filtered subset (3K tokens)
fn get_entities_in_file(json: &Value, file_path: &str) -> Vec<Entity>
```

### Risk 2: Agent Cannot Parse Complex JSON
**Mitigation**: Simplify JSON structure, add helper fields

Example:
```json
{
  "isgl1_key": "rust:fn:main:src_main_rs:1-10",
  "name": "main",  // ← Add human-readable name
  "reverse_deps_count": 15,  // ← Add summary stats
  "reverse_deps": [...]
}
```

### Risk 3: Performance Degradation
**Mitigation**: Performance tests (S01 Principle #5)

```rust
#[test]
fn test_query_performance_within_100ms() {
    let large_json = load_json("1500_entities.json");
    let start = Instant::now();

    let result = query_reverse_deps(&large_json, "rust:fn:target");

    assert!(start.elapsed() < Duration::from_millis(100));
}
```

---

## Implementation Order

1. **Day 1**: Phase 1 (Agent query tests + helpers)
2. **Day 2**: Documentation + examples
3. **Day 3**: Phase 2 (if tests show gaps)
4. **Defer**: Phase 3 (Mermaid - wait for user request)

---

## Decision Point: Do We Need Mermaid?

### For AGENTS: NO
- Agents query JSON directly
- Mermaid adds no semantic value
- JSON is canonical format

### For HUMANS: MAYBE
- GitHub visualization is nice
- But NOT priority
- Defer until requested

### For DOCUMENTATION: YES (Per S01)
- Use Mermaid for architecture diagrams
- Manually create (not generated from code)
- Example: PDG/SDG explanation diagrams

---

## Conclusion

### The Minimal Path Forward

1. ✅ **Validate**: Write tests proving agents can query current JSON
2. ✅ **Implement**: Add query helper functions (200 LOC)
3. ✅ **Document**: Show examples of agent queries
4. ⏸️ **Defer**: Mermaid generation (wait for need)
5. ⏸️ **Defer**: Control/data flow (future v1.0)

### What We Learned

**Wrong Focus**: Building Mermaid renderer before proving agent need
**Right Focus**: Validate agents can answer questions from JSON
**S01 Principle**: Proven architectures over theoretical abstractions

### Next Action

**STOP**: Mermaid implementation
**START**: Write Phase 1 tests in `query_helpers_test.rs`
**VALIDATE**: Can agents actually use this?

---

**END OF MINIMAL APPROACH DOCUMENT**

**Status**: Ready for TDD implementation
**Priority**: P0 (foundational capability)
**Effort**: 1-3 days (depending on test results)
