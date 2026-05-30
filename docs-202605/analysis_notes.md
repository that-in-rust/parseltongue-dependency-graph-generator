# ISG Analysis: Parseltongue Codebase

**Date**: November 15, 2025, 01:25 AM  
**Workspace**: parseltongue20251115012556/

---

## 🎯 ELI5: What Is Parseltongue?

### The Simple Version (For a 5-Year-Old)

Imagine you have a HUGE box of LEGO blocks (your code), and you want to build something new or fix something broken. But first, you need to know:
- **What blocks do you have?** (functions, structs, enums)
- **Which blocks connect to which?** (dependencies)
- **Where are all the red blocks?** (find specific code patterns)

**Parseltongue is your magical LEGO sorter!** 🪄

**Step 1**: It looks through ALL your LEGO blocks and puts them in a special organized box (database) - but it only looks ONCE!

**Step 2**: Now you can ask questions super fast:
- "Show me all the red blocks!" ✅
- "Which blocks are connected to the blue one?" ✅
- "Are any blocks stuck together in circles?" ✅

The magic trick: Instead of dumping out ALL your LEGO blocks every time (slow and messy), you just ask the organized box questions (super fast)!

---

## 🧙 The Parseltongue Architecture

Parseltongue is a **7-tool CLI toolkit** for code analysis:

```
📊 Tool Chain (7 Spells):
├─ pt01: 🗂️  Folder → Database (Parse once, query forever)
├─ pt02: 📤 Database → Exports (Level 0, 1, 2 - increasing detail)
├─ pt03: ✍️  LLM → Database (Write proposed changes)
├─ pt04: ✅ Syntax Validator (Check before applying)
├─ pt05: 🔄 Database → Diff (Generate CodeDiff.json)
├─ pt06: ⏭️  Make Future Current (Apply changes)
└─ pt07: 📊 Visual Analytics (Pretty charts and graphs)
```

---

## 📈 Codebase Statistics

### Entity Breakdown (142 Total CODE Entities)
```
╔═══════════════════════════════════════════╗
║     Entity Count by Type (Impl Only)      ║
╠═══════════════════════════════════════════╣
║ Method     [█████░░░░░░░░░]  58  (40%)  ║
║ Module     [████░░░░░░░░░░]  43  (30%)  ║
║ ImplBlock  [█░░░░░░░░░░░░░]  13  ( 9%)  ║
║ Function   [░░░░░░░░░░░░░░]  11  ( 7%)  ║
║ Struct     [░░░░░░░░░░░░░░]  10  ( 7%)  ║
║ Enum       [░░░░░░░░░░░░░░]   7  ( 4%)  ║
╚═══════════════════════════════════════════╝
```

### Dependency Graph
- **Total Edges**: 4,576 dependencies
- **Architecture Quality**: ✅ **0 circular dependencies** (clean design!)
- **Token Cost**: ~5K tokens (edge list), ~30K tokens (full entities)

---

## 🔍 Key Insights

### 1. **Parse Once, Query Forever** 🚀
The core philosophy: 
- **Traditional approach**: Read source files every time → 150K+ tokens
- **Parseltongue approach**: Parse once → Query database → 5-30K tokens
- **Token savings**: 93-97% reduction!

### 2. **Three Export Levels** 📊
```
Level 0: Pure edges only           → ~5K tokens   (architecture view)
Level 1: Entities + ISG + temporal → ~30K tokens  (detailed view)
Level 2: Full type system          → ~60K tokens  (complete view)
```

Pick the right level for your task!

### 3. **Temporal State System** ⏰
Parseltongue supports "future" code states:
- `current_ind = 1`: Current code
- `future_ind = 1`: Proposed changes by LLM
- Switch between states without modifying files!

### 4. **Modular Tool Chain** 🔧
Each `pt01` through `pt07` is a standalone binary that can be composed:
```bash
# Parse code
pt01 → database

# Export different views
pt02-level00 → edges.json (architecture)
pt02-level01 → entities.json (detailed)

# Visualize
pt07 entity-count → bar chart
pt07 cycles → circular dependency warnings
```

---

## 📦 Workspace Contents Summary

```
📁 Workspace: parseltongue20251115012556/

Database & Exports:
├── analysis.db/              (RocksDB database)
├── edges.json                (1.0 MB, 4576 edges)
├── edges_test.json           (1.0 MB)
├── public_api.json           (240 KB, 142 entities)
├── private_funcs.json        (240 KB, 142 entities)

Analysis Artifacts:
├── ingestion.log             (477 B)
├── entity_counts.txt         (1.0 KB)
├── cycles.txt                (649 B)
└── analysis_notes.md         (this file)

Ingestion Stats:
- Total files found: 500
- Files processed: 111
- CODE entities: 142
- TEST entities: 1198 (excluded)
- Duration: 1.43s
```

---

## 💡 What Makes Parseltongue Special?

### 1. **ISG (Indexed Symbolic Graph)** 
Every entity gets a unique key like:
```
rust:fn:build_call_chain_from_root:__zzArchive20251114_crates_parseltongue-core_src_query_json_graph_helpers_rs:34-56
```

This enables:
- Precise entity identification
- Cross-language support
- Temporal versioning

### 2. **Workspace Isolation**
Every analysis session creates a timestamped folder:
```
parseltongue20251115012556/
```
- Self-contained
- Replayable
- No conflicts between sessions

### 3. **Test Exclusion Intelligence**
Automatically excludes TEST entities (1198 in this case) to keep LLM context focused on production code.

### 4. **Token Efficiency Visualization**
Shows you exactly how much token budget you're using:
```
ISG Method: 8K tokens (4%) → 192K thinking space
vs
Grep Method: 150K tokens (75%) → 50K thinking space

Thinking Space Gain: +284%
```

---

## 🎓 The Three-Layer Architecture

```
┌─────────────────────────────────────┐
│  CLI Binaries (pt01-pt07)          │  ← User Interface
├─────────────────────────────────────┤
│  parseltongue-core Library          │  ← Core Logic
│  - Entity extraction                │
│  - Database operations (CozoDB)     │
│  - Query system (Datalog)           │
│  - ISG key generation                │
├─────────────────────────────────────┤
│  Storage Layer                       │  ← Persistence
│  - RocksDB (embedded)                │
│  - JSON exports                      │
│  - .toon format (compressed)         │
└─────────────────────────────────────┘
```

---

## 🌟 Use Cases

### For Developers:
1. **Understand unfamiliar codebase** - Query public API, dependencies
2. **Find refactoring candidates** - High complexity, god objects
3. **Detect code smells** - Circular dependencies, dead code
4. **Impact analysis** - "What will break if I change X?"

### For AI/LLM Agents:
1. **Token-efficient context** - 93% smaller than grep/cat
2. **Structured queries** - SQL-like WHERE clauses
3. **Incremental updates** - Parse once, query many times
4. **Temporal changes** - Track LLM-proposed modifications

---

## 🧪 Next Questions You Can Ask

1. "Show me entities in the pt01 crate" (module-specific query)
2. "What are the most-depended-on functions?" (hub analysis)
3. "Find all public error types" (type + visibility filtering)
4. "Show me functions with >50 lines" (complexity heuristics)
5. "Export with code included" (pt02 with --include-code 1)

---

## 🏆 Final Verdict

**Parseltongue is a "smart index for code"** - like a search engine index for websites, but for your codebase.

**Three superpowers**:
1. ⚡ **Speed**: Parse once, query infinitely
2. 🧠 **Token Efficiency**: 93-97% reduction vs traditional methods
3. 🎯 **Precision**: Structured queries, not pattern matching

**Perfect for**: Large codebases, LLM-assisted development, architecture analysis, refactoring

---

*Analysis complete! All artifacts preserved in: `parseltongue20251115012556/`*
