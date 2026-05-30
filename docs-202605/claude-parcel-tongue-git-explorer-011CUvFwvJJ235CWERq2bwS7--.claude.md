# Parseltongue Development Rules

## THE ABSOLUTE TOP: Versioning Philosophy

### ONE FEATURE PER INCREMENT - END TO END - SPIC AND SPAN

**Core Principle**: Each version delivers **EXACTLY ONE complete feature**, fully working end-to-end.

**Version Naming**:
- v0.9.4, v0.9.5, v0.9.6, v0.9.7, v0.9.8, v0.9.9 → v1.0.0 → v1.0.1, v1.0.2...
- **NO v0.10.0** - we go v0.9.9 then v1.0.0
- Triple-digit minor versions before major bump

**What "END TO END" Means**:
- ✅ Feature works in production binary
- ✅ All tests passing (not just the new feature)
- ✅ Documentation updated (README, PRD, scope docs)
- ✅ Shell scripts updated (.sh files work)
- ✅ Agent prompts updated (if feature changes workflow)
- ✅ Integration tested (not just unit tests)
- ✅ Zero TODOs, zero stubs, zero placeholders
- ✅ Pushed to origin/main

**What "SPIC AND SPAN" Means**:
- ✅ Binary compiles: `cargo build --release` succeeds
- ✅ Tests pass: `cargo test --all` → 0 failures
- ✅ No warnings (or explicitly documented if unavoidable)
- ✅ Clean git status after commit
- ✅ Version-wise-scope.md updated
- ✅ User can use the feature IMMEDIATELY

**Forbidden**:
- ❌ Partial features ("LPA foundation but no integration")
- ❌ Stubs for "future work"
- ❌ Breaking existing features to add new ones
- ❌ Documentation that says "will be implemented"
- ❌ Commits with TODO comments in production code

**Example Good Increments**:
- v0.9.4: LPA clustering foundation (7 tests, production-ready)
- v0.9.5: TEST exclusion from ingestion (binary flag, works end-to-end)
- v0.9.6: Louvain algorithm (compares to LPA, exports work)
- v0.9.7: Triple export system (CozoDB + JSON + TOON all work)

**Example Bad Increments**:
- ❌ v0.9.4: "Clustering foundation + export system + visualization"
- ❌ v0.9.5: "Three algorithms but exports don't work yet"
- ❌ v0.9.6: "Integration with PT01 but tests failing"

**Why This Matters**:
- Users get **working features**, not promises
- Each release is **production-ready**
- No accumulation of technical debt
- Clear progress: "We have X complete features"
- LLMs can verify completion objectively

---

## Core Philosophy

### FUNCTIONAL IDIOMATIC RUST
Write **pure functional code** following Rust idioms:
- ✅ Prefer iterators over loops
- ✅ Use `Result<T, E>` and `Option<T>` - never panic in libraries
- ✅ Pure functions (no side effects unless explicit)
- ✅ Immutability by default (`let` not `let mut`)
- ✅ Trait-based abstractions (dependency injection)
- ✅ RAII resource management (Drop trait)

### TDD-FIRST: STUB → RED → GREEN → REFACTOR
**ALWAYS write tests BEFORE implementation:**
1. **STUB**: Write failing test with expected interface
2. **RED**: Run test, verify it fails correctly
3. **GREEN**: Minimal implementation to make test pass
4. **REFACTOR**: Improve code without breaking tests

### EXECUTABLE SPECIFICATIONS OVER NARRATIVES
Traditional user stories fail LLMs. Use **formal contracts**:
- **Preconditions**: What must be true before execution
- **Postconditions**: What will be true after execution
- **Error conditions**: What failures can occur
- **Performance claims**: Measured, not guessed

**Every claim validated by automated tests.**

---

## Rule #1: FOUR-WORD NAMING CONVENTION

**ALL function names: EXACTLY 4 words** (underscores separate)
**ALL crate names: EXACTLY 4 words** (hyphens separate)

**Why**: LLMs parse by tokenizing. 4 words = optimal semantic density.

```rust
✅ filter_implementation_entities_only()
✅ render_box_with_title_unicode()
✅ save_visualization_output_to_file()
✅ pt01-folder-to-cozodb-streamer

❌ filter_entities()                    // Too short (2)
❌ detect_cycles_in_dependency_graph()  // Too long (5)
```

**Naming pattern**: `verb_constraint_target_qualifier()`
- Verb: `filter`, `render`, `detect`, `save`
- Constraint: `implementation`, `box_with_title`, `visualization_output`
- Target: `entities`, `unicode`, `file`
- Qualifier: `only`, `to`, `in`

---

## Rule #2: NEVER LIE - Verify Everything

**Forbidden** (causes hallucination):
- ❌ Claiming "done" when not tested
- ❌ Saying tests pass without showing output
- ❌ Marking todos complete without verification
- ❌ Making up results

**Required**:
- ✅ Run tests → Show actual output
- ✅ Build code → Show actual output
- ✅ Mark work complete ONLY after verification
- ✅ Use explicit status: "✅ VERIFIED" / "⚠️ UNTESTED" / "❌ BROKEN"

---

## Rule #3: NO STUBS IN COMMITS

**Every commit must be production-ready.**

**Forbidden**:
- ❌ Committing code with `TODO:` comments
- ❌ Committing `unimplemented!()` or `panic!()`
- ❌ Shipping releases with stubs
- ❌ Open todos in version control

**Required**:
- ✅ Either implement FULLY or DON'T commit
- ✅ Remove ALL todos before commit
- ✅ If incomplete: Say "Cannot complete because [reason]"

---

## Rule #4: Layered Architecture (L1→L2→L3)

**L1 Core** (no_std compatible):
- Ownership, lifetimes, traits, Result/Option, RAII, newtype pattern

**L2 Standard**:
- Collections, iterators, smart pointers, Send/Sync

**L3 External**:
- Async/await (Tokio), serialization (Serde), databases (CozoDB)

**Never mix layers** - L3 cannot be in L1 core.

---

## Rule #5: Mermaid-Only Diagrams

ALL diagrams must be Mermaid (for GitHub rendering).
- ✅ `graph LR`, `sequenceDiagram`, `classDiagram`
- ❌ No ASCII art, no image files, no external tools

---

## Enforcement Checklist

Before ANY commit:
- [ ] Tests written FIRST (TDD cycle)?
- [ ] Tests passing? (show output: `cargo test`)
- [ ] Build passing? (show output: `cargo build --release`)
- [ ] Zero TODOs/STUBs? (show: `grep -r "TODO\|STUB\|PLACEHOLDER"`)
- [ ] All function names = 4 words? (count: `verb_noun_noun_qualifier`)
- [ ] Follows functional idioms? (iterators, immutability, pure functions)
- [ ] Status verified? (not claimed without proof)

**If ANY checkbox unchecked → STOP. NOT READY.**

---

## Quick Reference

**TDD Cycle**: STUB (write test) → RED (fails) → GREEN (minimal impl) → REFACTOR (improve)

**Naming**: 4 words always (`action_constraint_target_qualifier`)

**Status markers**:
- ✅ VERIFIED (ran tests, saw output)
- ⚠️ UNTESTED (needs testing)
- 🚧 INCOMPLETE (missing parts)
- ❌ BROKEN (not working)

**Architecture**: L1 (core) → L2 (std) → L3 (external)

**Never**: Stubs in commits, lies about status, non-4-word names, panic in libraries

**Always**: Tests first, functional style, immutability, proof over claims
