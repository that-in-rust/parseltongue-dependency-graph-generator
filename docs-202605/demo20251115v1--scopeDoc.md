# Parseltongue v0.9.7 & Scope Document

**Date**: 2025-11-07
**Status**: Draft - Planning Phase
**Previous Version**: v0.9.6 (Test Exclusion + Single Binary)
**Target**: Feature enhancement release

===

# For v097
- [ ] toon export should be a default part of the json export, working for each and every we're looking for each and every command which exports to JSON. As long as it exports to JSON automatically, it should export to.toon as well
- [ ] 








===

# For v300 (long term backlog)


### 📊 Known Gaps (from Recommendation20251107.md)
1. **No Clustering Capability**
   - Current: Users navigate function-by-function or file-by-file
   - Needed: ISGL0.5 semantic grouping (3-20 functions per cluster)
   - Impact: 80-96% time savings in code exploration tasks

2. **No Flow Analysis**
   - Current: Static entity relationships only
   - Needed: Control flow, data flow, temporal flow
   - Impact: 2.3× relevance improvement, taint tracking, bottleneck detection

3. **No Visual Feedback**
   - Current: Text-only output
   - Needed: Terminal visualizations (bar charts, cycle warnings)
   - Note: pt07 binaries exist but not integrated into agent workflows

---

## v0.9.7 Scope Options

### Option A: Quick Win - Visual Integration (2 weeks)
**Goal**: Integrate existing pt07-* visual tools into agent workflows

**Deliverables**:
- Agent automatically invokes pt07-render-entity-count-bar-chart after ingestion
- Agent shows pt07-render-dependency-cycle-warning-list when cycles detected
- Add visual feedback to ultrathink agent workflows
- Update README with visual analytics examples

**Effort**: 2 weeks
**Risk**: Low (tools already exist)
**Value**: Moderate (improved UX, no new capabilities)

**Files to Modify**:
- `.claude/agents/parseltongue-ultrathink-isg-explorer.md` - add visual tool invocations
- `crates/pt07-visual-analytics-terminal/README.md` - update examples
- Root README.md - add Section 4: Visual Analytics

---

### Option B: Foundation - Clustering Phase 1 (4 weeks)
**Goal**: Implement ISGL0.5 semantic clustering (from Recommendation20251107.md Phase 1)

**Deliverables**:
- New crate: `pt09-semantic-clustering`
- pt09-cluster-by-louvain: Group related functions using Louvain algorithm
- pt09-cluster-query: Find which cluster(s) contain specific functions
- Add `clusters` table to CozoDB schema
- Update agent to use clustering in code exploration workflows

**Effort**: 4 weeks
**Risk**: Medium (new graph algorithms, schema changes)
**Value**: High (addresses #1 gap, 80% time savings)

**Files to Create**:
- `crates/pt09-semantic-clustering/Cargo.toml`
- `crates/pt09-semantic-clustering/src/lib.rs`
- `crates/pt09-semantic-clustering/src/algorithms/louvain.rs`
- `crates/pt09-semantic-clustering/src/bin/pt09_cluster_by_louvain.rs`
- `crates/pt09-semantic-clustering/src/bin/pt09_cluster_query.rs`

**Files to Modify**:
- `crates/parseltongue-core/src/schema.rs` - add clusters table
- `.claude/agents/parseltongue-ultrathink-isg-explorer.md` - clustering workflows
- `Cargo.toml` - add pt09 to workspace

---

### Option C: Foundation - Flow Analysis Phase 1 (4 weeks)
**Goal**: Implement control flow analysis (from Recommendation20251107.md Phase 2)

**Deliverables**:
- New crate: `pt10-flow-analysis`
- pt10-control-flow: Trace call chains (who calls whom)
- pt10-find-bottlenecks: Detect functions with high betweenness centrality
- Add `control_flow_edges` table to CozoDB schema
- Update agent to use flow analysis in debugging workflows

**Effort**: 4 weeks
**Risk**: Medium (complex graph traversal, performance concerns)
**Value**: High (addresses #2 gap partially, enables debugging workflows)

**Files to Create**:
- `crates/pt10-flow-analysis/Cargo.toml`
- `crates/pt10-flow-analysis/src/lib.rs`
- `crates/pt10-flow-analysis/src/control_flow.rs`
- `crates/pt10-flow-analysis/src/bin/pt10_control_flow.rs`
- `crates/pt10-flow-analysis/src/bin/pt10_find_bottlenecks.rs`

**Files to Modify**:
- `crates/parseltongue-core/src/schema.rs` - add control_flow_edges table
- `.claude/agents/parseltongue-ultrathink-isg-explorer.md` - flow workflows
- `Cargo.toml` - add pt10 to workspace

---

### Option D: Incremental - TOON Format Enhancements (1 week)
**Goal**: Improve existing TOON format with user-requested features

**Potential Enhancements**:
- TOON compression option (gzip) for large codebases
- TOON streaming mode for real-time ingestion
- TOON validation tool (detect format errors)
- TOON statistics (show token savings per file)

**Effort**: 1 week
**Risk**: Low (incremental improvement)
**Value**: Low-Medium (improves existing feature, no new capabilities)

**Files to Modify**:
- `crates/parseltongue-core/src/serializers/toon.rs` - add enhancements
- `crates/parseltongue-core/src/serializers/mod.rs` - export new features
- Add new bins if needed (pt11-toon-validate, pt11-toon-stats)

---

## Recommendation Matrix

| Option | Effort | Risk | Value | Aligns with Recommendation20251107.md |
|--------|--------|------|-------|--------------------------------------|
| A: Visual Integration | 2 weeks | Low | Medium | Partial (Phase 4: Visual Feedback) |
| B: Clustering Phase 1 | 4 weeks | Medium | High | YES (Phase 1: Clustering) |
| C: Flow Analysis Phase 1 | 4 weeks | Medium | High | YES (Phase 2: Control Flow) |
| D: TOON Enhancements | 1 week | Low | Low-Medium | No |

---

## My Recommendation: **Option B (Clustering Phase 1)**

**Rationale**:
1. **Highest Impact**: Addresses the #1 gap from Recommendation20251107.md research
2. **Foundation for Future**: Clustering is prerequisite for advanced flow analysis (Phase 3)
3. **Clear User Value**: 80% time savings in code exploration (proven in simulations)
4. **Manageable Scope**: 4 weeks for Phase 1, can iterate in v0.9.8, v0.9.9
5. **Follows ONE FEATURE PER INCREMENT**: Single major capability per release

**Follow-up Roadmap**:
- v0.9.7: Clustering Phase 1 (Louvain algorithm, basic queries)
- v0.9.8: Flow Analysis Phase 1 (Control flow, bottleneck detection)
- v0.9.9: Flow Analysis Phase 2 (Data flow, taint tracking)
- v0.10.0: Visual Integration + Multi-flow synthesis (Phase 4)

---

## Questions for User Decision

1. **Which option aligns with your priorities?**
   - Quick win (Option A: Visual)
   - Foundation (Option B: Clustering or Option C: Flow)
   - Incremental (Option D: TOON)

2. **Timeline constraints?**
   - Do you need v0.9.7 within 2 weeks, or can we take 4 weeks for foundation work?

3. **Feature dependencies?**
   - Are there external commitments (Agent Games 2025, demos) that require specific features?

4. **Breaking changes acceptable?**
   - Options B & C require CozoDB schema changes (migration needed)

---

## Current Working Tree Status

```
M .claude/.parseltongue/S01-README-MOSTIMP.md
M .claude/.parseltongue/S02-visual-summary-terminal-guide.md
M .claude/.parseltongue/S05-tone-style-guide.md
M .claude/.parseltongue/S06-design101-tdd-architecture-principles.md
M crates/pt01-folder-to-cozodb-streamer/Cargo.toml
D crates/pt01-folder-to-cozodb-streamer/src/main.rs
M crates/pt01-folder-to-cozodb-streamer/src/streamer.rs
D crates/pt02-llm-cozodb-to-context-writer/src/main.rs
M crates/pt03-llm-to-cozodb-writer/Cargo.toml
D crates/pt03-llm-to-cozodb-writer/src/main.rs
M crates/pt04-syntax-preflight-validator/Cargo.toml
D crates/pt04-syntax-preflight-validator/src/main.rs
M crates/pt05-llm-cozodb-to-diff-writer/Cargo.toml
D crates/pt05-llm-cozodb-to-diff-writer/src/main.rs
M crates/pt06-cozodb-make-future-code-current/Cargo.toml
D crates/pt06-cozodb-make-future-code-current/src/main.rs
M crates/pt07-visual-analytics-terminal/Cargo.toml
D crates/pt07-visual-analytics-terminal/src/bin/pt07_visual_analytics_terminal.rs
D crates/pt07-visual-analytics-terminal/src/bin/render_dependency_cycle_warning_list.rs
D crates/pt07-visual-analytics-terminal/src/bin/render_entity_count_bar_chart.rs
```

**Note**: Uncommitted changes above - should be resolved before starting v0.9.7 work.

---

## Next Steps

1. **User Decision**: Choose option A, B, C, or D (or propose Option E)
2. **Clean Working Tree**: Commit or stash current changes
3. **Create Feature Branch**: `git checkout -b feature/v0.9.7-<selected-option>`
4. **TDD Cycle**: STUB → RED → GREEN → REFACTOR
5. **Update Agent**: Modify `.claude/agents/parseltongue-ultrathink-isg-explorer.md` with new capabilities

---

**This document will be archived to `zzArchive202510/scopeDocs/` after v0.9.7 release.**
