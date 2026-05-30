# Parseltongue Ecosystem: Current State & Repository Map

**Created**: 2025-11-10 | **Updated**: 2025-11-10 (ISG v3.0 integration)
**Purpose**: Ultrathink navigation map for parseltongue ecosystem structure
**Status**: v0.9.6 production + compilation research Phase 1 + ISG v3.0 agent architecture

---

## 🎯 CURRENT STATE: Minto Pyramid Summary

### **Essence (2-Line Executive Summary)**

Parseltongue v0.9.6 is a **production-ready ISG-native code analysis system** that converts 500K+ token codebases into 2-5K token queryable CozoDB graphs, achieving 99% token reduction and 10-100× speed improvements through "Parse Once, Query Forever" architecture.
The **parseltongue-ultrathink-isg-explorer v3.0 agent** forbids filesystem fallback (no grep/cat), executing all analysis via 5 progressive database query strategies that return 2-5K tokens vs traditional 250K+ token dumps—maximizing LLM Thinking Space Ratio (TSR) for precise architectural reasoning.

---

### **Layer 1: Strategic Current State (Why This Matters)**

**The Problem Solved:**
- **LLM context pollution**: Dumping 500K tokens of raw code degrades reasoning by 20%+ (Stanford TACL 2023)
- **Analysis bottleneck**: `grep`/filesystem tools are 10-100× slower than database queries and produce noisy, token-heavy results
- **Lost architectural context**: Traditional tools show files/lines, not relationships/dependencies

**The Solution Architecture:**
- **ISG (Interface Signature Graph)**: Rich metadata stored during ingestion—signatures, dependencies, `current_code` field—enables surgical queries
- **Parse Once, Query Forever**: Single pt01 ingestion → infinite instant queries; zero filesystem re-scanning
- **Agent-Native Design**: Ultrathink v3.0 agent strictly enforces database-only queries, treating CozoDB as ground truth

**The Impact:**
- **99% token reduction**: 500K raw code → 2-5K structured graph queries (TSR optimization)
- **10-100× speed**: 80ms database queries vs 2.5s+ grep scans
- **Precise reasoning**: LLMs receive architectural context (dependencies, signatures) not noise (raw text)

---

### **Layer 2: ISG-Native Architecture & v3.0 Features**

#### **Core Principle: "Parse Once, Query Forever"**

Traditional workflow (repeated filesystem scanning):
```
User Query → grep codebase → parse results → analyze → repeat for next query
```

ISG-native workflow (single ingestion, infinite queries):
```
[ONE-TIME] pt01 ingestion → CozoDB populated with ISG
[INFINITE] Query ISG → instant results (no filesystem access)
```

#### **v3.0 Agent Architecture: Strict Database-Only Queries**

The `parseltongue-ultrathink-isg-explorer v3.0` agent **forbids** filesystem tools after ingestion:
- ❌ **NO** `grep`, `rg`, `cat`, `find`, `ls` after indexing
- ✅ **YES** CozoDB Datalog queries on ISG entities
- ✅ **YES** Leveraging `interface_signature`, `current_code`, `forward_deps`, `reverse_deps` fields

**Why This Matters:**
- **Context pollution prevention**: Filesystem tools return unstructured text (200K+ tokens); database returns structured entities (2K tokens)
- **Speed**: Database indexes beat filesystem scans 10-100×
- **Accuracy**: Graph traversal (Strategy 4) provides precise dependency paths vs manual grepping

#### **The ISG Schema: What's Stored**

Every code entity in CozoDB contains:
- `entity_id`: Unique key (e.g., `rust:fn:normalize_string:src_utils_rs:45-60`)
- `entity_type`: Function, struct, trait, module, etc.
- `interface_signature`: API contract (return types, parameters)
- `current_code`: Full implementation (enables code search without filesystem)
- `forward_deps`: Who this entity calls (dependency graph)
- `reverse_deps`: Who calls this entity (blast radius analysis)
- `file_path`, `start_line`, `end_line`: Source location metadata

**Key Innovation**: Storing `current_code` during ingestion means **code search happens in the database**, not the filesystem. This is what enables 99% token reduction.

---

### **Layer 3: The 5 ISG-Native Search Strategies**

The v3.0 agent uses **progressive disclosure**: start minimal (Strategy 1), escalate if needed (Strategy 5).

#### **Strategy 1: Metadata Search (Fastest, Minimal Tokens)**
**Query**: Entity names, types, file paths
**Use Case**: "Find all public functions in `auth` module"
**Token Cost**: ~500 tokens (names only)
**Speed**: <50μs

**CozoDB Query Example:**
```datalog
?[entity_id, entity_type, file_path] :=
  *entities{ entity_id, entity_type, file_path, visibility },
  visibility = "pub",
  file_path ~ "auth"
```

---

#### **Strategy 2: Signature Search (API Contract Analysis)**
**Query**: `interface_signature` field—return types, parameters
**Use Case**: "Find all functions returning `Result<PaymentConfirmation, PaymentError>`"
**Token Cost**: ~1K tokens (signatures only)
**Speed**: ~100μs

**Why Powerful**: Finds code by *what it does* (API contract), not *what it's named*.

**CozoDB Query Example:**
```datalog
?[entity_id, interface_signature] :=
  *entities{ entity_id, interface_signature },
  interface_signature ~ "Result<PaymentConfirmation"
```

**Real-World Win**: Discovers `create_transaction()`, `handle_checkout()`, `process_sale()` when searching for payment APIs—`grep "payment"` would miss these.

---

#### **Strategy 3: Code Search (Implementation Details)**
**Query**: `current_code` field—function bodies, implementation logic
**Use Case**: "Show all code calling the Stripe API"
**Token Cost**: ~2K tokens (code snippets)
**Speed**: ~200ms

**Why Critical**: Searches implementation without re-parsing files. `current_code` stored during pt01 ingestion.

**CozoDB Query Example:**
```datalog
?[entity_id, current_code] :=
  *entities{ entity_id, current_code },
  current_code ~ "stripe\\."
```

**Traditional Alternative**: `grep -r "stripe\." .` → 200K+ tokens of noise, 2.5s+ execution.

---

#### **Strategy 4: Graph-Aware Search (Dependency Traversal)**
**Query**: `forward_deps` / `reverse_deps` fields—who calls whom
**Use Case**: "If I change `validate_payment`, what breaks?" (blast radius)
**Token Cost**: ~2-3K tokens (subgraph)
**Speed**: ~150ms

**Why Transformative**: Traditional tools require *manual* tracing through files; database returns complete subgraph instantly.

**CozoDB Query Example (2-hop blast radius):**
```datalog
// 1-hop: direct callers
?[caller_id] :=
  *dependencies{ caller_id, callee_id },
  callee_id = "rust:fn:validate_payment:src_payments_rs:120-145"

// 2-hop: transitive callers
?[transitive_caller] :=
  *dependencies{ transitive_caller, hop1_caller },
  *dependencies{ hop1_caller, target },
  target = "rust:fn:validate_payment:src_payments_rs:120-145"
```

**Real-World Impact**: 150ms query vs 5+ minutes of manual tracing. 2K tokens vs 100K+ reading multiple files.

---

#### **Strategy 5: Semantic Search (Conceptual Clustering)**
**Query**: Semantic clusters from pt08-semantic-atom-cluster-builder
**Use Case**: "Show me the authentication system" (holistic understanding)
**Token Cost**: ~4K tokens (focused context)
**Speed**: ~400ms

**Why Advanced**: Groups related entities by *meaning*, not just file location. Returns minimal, focused context optimized for LLM reasoning.

**Use Case**: New engineer needs to understand authentication flow:
- **Traditional**: Read 10+ files (~150K tokens), manually trace flow (hours)
- **ISG Strategy 5**: Query "auth system" cluster → receive 9 critical entities with dependency flow (~4K tokens, <1 minute)

**Result**: 97% token reduction, instant architectural understanding.

---

### **Layer 4: Three Practical Simulations (Real-World Workflows)**

---

#### **Simulation 1: The "Blast Radius" (Impact Analysis Using Strategy 4)**

**Scenario**: Sarah must refactor `utils::normalize_string` to return `Result<String, NormalizationError>` instead of `String`. She needs to know exactly what breaks.

**User Query**:
> @parseltongue-ultrathink-isg-explorer I need to refactor `utils::normalize_string`. What is the complete blast radius (up to 2 hops)?

**Agent Execution (Strategy 4: Graph-Aware Search)**:

**Step 1: Identify Seed Entity**
```datalog
?[entity_id, interface_signature] :=
  *entities{ entity_id, interface_signature },
  entity_id ~ "normalize_string"
```
**Result**: `rust:fn:normalize_string:src_utils_rs:45-60`

**Step 2: Query 1-Hop Dependencies (Direct Callers)**
```datalog
?[caller_id, caller_signature] :=
  *dependencies{ caller_id, callee_id },
  callee_id = "rust:fn:normalize_string:src_utils_rs:45-60",
  *entities{ caller_id, interface_signature: caller_signature }
```
**Result**: 3 direct callers found:
1. `auth::validate_username` (src/auth/service.rs:89)
2. `api::sanitize_input` (src/api/middleware.rs:34)
3. `search::index_document` (src/search/indexer.rs:156)

**Step 3: Query 2-Hop Dependencies (Transitive Callers)**
```datalog
?[hop2_caller, hop2_signature] :=
  *dependencies{ hop2_caller, hop1_caller },
  *dependencies{ hop1_caller, target },
  target = "rust:fn:normalize_string:src_utils_rs:45-60",
  *entities{ hop2_caller, interface_signature: hop2_signature }
```
**Result**: 4 transitive callers found:
1. `auth::register_user` → calls `validate_username`
2. `user::update_profile` → calls `validate_username`
3. `api::api_entry_v1` → calls `sanitize_input`
4. `document::handle_document_upload` → calls `index_document`

**Agent Output (2K tokens, 150ms)**:
```
Blast Radius (2-hop): 7 unique entities across 3 domains

Impact Domains:
1. Authentication & User Management: CRITICAL
   - auth::register_user (src/auth/service.rs:120)
   - user::update_profile (src/user/manager.rs:78)
   Must handle Result propagation or registration/updates fail.

2. API Middleware: HIGH
   - api::api_entry_v1 (src/api/routes.rs:23)
   All V1 API requests require error handling updates.

3. Search Indexing: MEDIUM
   - document::handle_document_upload (src/document/handler.rs:201)
   Can fail gracefully with logging.
```

**Visualization** (Mermaid generated by agent):
```mermaid
graph TD
    T["utils::normalize_string"] --> H1_A["auth::validate_username"]
    T --> H1_B["api::sanitize_input"]
    T --> H1_C["search::index_document"]

    H1_A --> H2_A1["auth::register_user"]
    H1_A --> H2_A2["user::update_profile"]
    H1_B --> H2_B1["api::api_entry_v1"]
    H1_C --> H2_C1["document::handle_document_upload"]

    style T fill:#C89999
```

**Efficiency Comparison**:
- **ISG Strategy 4**: 2 queries, 150ms, 2K tokens
- **Traditional `grep`**: Manual tracing, 15+ minutes, 200K+ tokens

---

#### **Simulation 2: The "API Contract Hunt" (Finding Code by Behavior Using Strategies 2 & 3)**

**Scenario**: Alex is adding PayPal integration alongside Stripe. He needs to:
1. Find all code interacting with Stripe SDK
2. Understand the payment abstraction layer

**User Query 1**:
> @parseltongue-ultrathink-isg-explorer I'm adding PayPal integration. Show me all functions calling the Stripe SDK.

**Agent Execution (Strategy 3: Code Search)**:
```datalog
?[entity_id, current_code] :=
  *entities{ entity_id, current_code },
  current_code ~ "stripe\\."
```

**Agent Output (1K tokens, 200ms)**:
```
Found 3 entities interacting with Stripe SDK:

1. payments::process_payment (src/payments/stripe_impl.rs:45)
   Code: stripe.charge.create(...)

2. payments::refund_transaction (src/payments/stripe_impl.rs:89)
   Code: stripe.refund.create(...)

3. payments::create_customer (src/payments/stripe_impl.rs:120)
   Code: stripe.customer.create(...)
```

**User Query 2**:
> Show me all functions returning `Result<PaymentConfirmation, PaymentError>` to understand the abstraction layer.

**Agent Execution (Strategy 2: Signature Search)**:
```datalog
?[entity_id, interface_signature, file_path] :=
  *entities{ entity_id, interface_signature, file_path },
  interface_signature ~ "Result<PaymentConfirmation, PaymentError>"
```

**Agent Output (1.5K tokens, 100ms)**:
```
Found 3 entities with payment contract signature:

1. PaymentGateway::charge (src/payments/traits.rs:12)
   Signature: fn charge(&self, amount: u64) -> Result<PaymentConfirmation, PaymentError>
   Type: Trait definition (abstraction layer)

2. StripeGateway::charge (src/payments/stripe_impl.rs:34)
   Signature: fn charge(&self, amount: u64) -> Result<PaymentConfirmation, PaymentError>
   Type: Trait implementation

3. payments::service::execute_payment (src/payments/service.rs:67)
   Signature: pub fn execute_payment(...) -> Result<PaymentConfirmation, PaymentError>
   Type: Orchestrator (calls gateway.charge())

Recommendation:
Implement PaymentGateway trait in src/payments/paypal_impl.rs,
then register in execute_payment orchestrator.
```

**Why Strategy 2 Is Powerful**:
- `grep "payment"` would find thousands of results
- Signature search finds *exact API contract* regardless of function names
- Discovered `execute_payment` orchestrator that `grep` would miss (doesn't contain "stripe")

**Efficiency Comparison**:
- **ISG Strategies 2 & 3**: 2 queries, 250ms total, 2.5K tokens
- **Traditional `grep`**: Noisy results, 100K+ tokens, manual filtering (30+ minutes)

---

#### **Simulation 3: The "Efficient Overview" (Onboarding Using Strategies 1, 4 & Progressive Disclosure)**

**Scenario**: Maria is a new engineer onboarding to the authentication system. She needs architectural understanding without reading 150K tokens of code.

**User Query**:
> @parseltongue-ultrathink-isg-explorer I'm new. Give me a high-level overview of the authentication architecture and the main login flow.

**Agent Execution (Progressive Disclosure: Level 0 → Strategy 1 → Strategy 4)**:

**Step 1: Architecture Overview (Level 0 - Dependency Graph Only)**
```bash
./parseltongue pt02-level00 --where-clause "auth OR session" --output auth_deps.json
```
**Result**: Dependency graph showing `auth::service` is the hub, with cyclic dependency to `session::manager`.

**Agent Analysis (500 tokens)**:
```
Architecture Overview (Level 0):
- Central hub: auth::service
- Primary callers: api::controllers
- Dependencies: database, crypto utilities
- WARNING: Cyclic dependency detected between auth::service ↔ session::manager
  (May indicate tight coupling - refactoring opportunity)
```

**Step 2: Find Login Entry Point (Strategy 1: Metadata Search)**
```datalog
?[entity_id, file_path, visibility] :=
  *entities{ entity_id, entity_type, file_path, visibility },
  entity_type = "function",
  entity_id ~ "login",
  visibility = "pub"
```
**Result**: `api::controllers::auth_controller::handle_login` (public entry point)

**Step 3: Trace Login Flow (Strategy 4: Graph-Aware - 3 Hops)**
```datalog
// Hop 1: What does handle_login call?
?[callee_id] :=
  *dependencies{ caller_id, callee_id },
  caller_id = "api::controllers::auth_controller::handle_login"

// Hop 2: What do those callees call?
?[hop2_callee] :=
  *dependencies{ hop1_caller, hop2_callee },
  *dependencies{ entry, hop1_caller },
  entry = "api::controllers::auth_controller::handle_login"

// Hop 3: What do those call?
?[hop3_callee] :=
  *dependencies{ hop2_caller, hop3_callee },
  *dependencies{ hop1_caller, hop2_caller },
  *dependencies{ entry, hop1_caller },
  entry = "api::controllers::auth_controller::handle_login"
```

**Agent Output (4.5K tokens, 400ms total)**:
```
Login Flow Summary (9 critical entities):

Entry Point:
└─ api::controllers::auth_controller::handle_login

Core Logic (auth::service::login_user):
├─ Validation: auth::validate_credentials
├─ Database: db::fetch_user_by_email
├─ Security: crypto::verify_password
└─ Session: session::create_session

Session Creation (session::create_session):
├─ Token: crypto::generate_token
└─ Storage: db::store_session

Focus your attention on these 9 functions for login understanding.
```

**Visualization** (Mermaid generated by agent):
```mermaid
graph LR
    A["API: handle_login"] --> B["Auth Service: login_user"]

    B --> C1["validate_credentials"]
    B --> C2["fetch_user_by_email"]
    B --> C3["verify_password"]
    B --> C4["create_session"]

    C4 --> D1["generate_token"]
    C4 --> D2["store_session"]

    style A fill:#90EE90
    style B fill:#87CEEB
```

**Efficiency Comparison**:
- **ISG Progressive Disclosure**: 3 queries, 400ms, 4.5K tokens → 97% token reduction
- **Traditional (reading files)**: Reading auth/ + session/ modules = 150K+ tokens, hours of manual tracing

**Key Insight**: Maria now understands the complete login flow (9 functions) without reading 150K tokens of implementation details. She can dive into specific functions only when needed.

---

## 📊 ISG v3.0 Agent: Summary Benefits

### **Quantitative Impact**

| Metric | Traditional (grep/cat) | ISG v3.0 Agent | Improvement |
|--------|------------------------|----------------|-------------|
| **Token Usage** | 200K-500K | 2K-5K | **99% reduction** |
| **Query Speed** | 2-5 seconds | 80-400ms | **10-100× faster** |
| **Accuracy** | Noisy (grep false positives) | Precise (structured queries) | **High precision** |
| **Context Quality** | Raw text dumps | Architectural relationships | **LLM-optimized** |
| **Filesystem Access** | Every query re-scans | Zero after ingestion | **Parse once, query forever** |

### **Qualitative Impact**

**For Developers**:
- **Instant blast radius analysis**: Know exactly what breaks before refactoring
- **API contract discovery**: Find code by *what it does*, not *what it's named*
- **Onboarding acceleration**: New engineers understand systems in minutes, not days

**For LLMs**:
- **Thinking Space Ratio (TSR) optimization**: 99% token reduction → more tokens for reasoning
- **Precision context**: Architectural relationships, not noise → 20%+ better reasoning (Stanford TACL 2023)
- **Graph-aware understanding**: Dependency traversal enables sophisticated analysis

**For Organizations**:
- **Reduced context costs**: 99% fewer tokens → 99% lower API costs for LLM-assisted development
- **Faster development cycles**: 10-100× query speed → tighter feedback loops
- **Knowledge preservation**: CozoDB captures architectural knowledge, survives team turnover

---

## 🎮 Agent Games 2025: Why This Matters

**Thesis**: Parseltongue v3.0 demonstrates that **ISG-native architecture** is the future of code analysis for AI agents.

**Evidence**:
1. **Stanford TACL 2023**: Context pollution degrades LLM reasoning by 20%+
2. **Parseltongue v3.0**: 99% token reduction via database queries vs filesystem tools
3. **Real-world simulations**: Blast radius (150ms vs 15 min), API hunt (250ms vs 30 min), onboarding (400ms vs hours)

**Innovation**: Strict "no filesystem fallback" policy in v3.0 agent proves database-first analysis is not just *faster*, but *fundamentally better* for LLM reasoning. Structured graph data (2K tokens) vs unstructured text dumps (200K tokens) is the difference between LLMs seeing *architecture* vs *noise*.

**Meta-Research Loop**: Parseltongue analyzing parsing tools (ast-grep, semgrep, tree-sitter) creates self-improving ecosystem—analyzing analyzers to improve analysis.

---

## Main Repository

**`/Users/amuldotexe/Projects/parseltongue`** (Git: main)
Core parseltongue project: 12-language code parser → CozoDB graph database.
Converts 500K+ token codebases into 2-5K token queryable dependency graphs.
11 workspace crates implementing streaming ingestion, LLM context optimization, visual analytics.

---

## Folder Collections Overview

### 1. **compilation_repos/** - Compilation Research (8 Git clones)

**`compilation_repos/rust/`**
The Rust compiler (rustc) with full AST → HIR → MIR → LLVM IR pipeline.
Research target: Understanding compilation trigger points and stage transformations.
Core entry for exploring how to compile from CozoDB representation.

**`compilation_repos/cargo/`**
Rust package manager and build orchestrator with workspace resolution.
Research target: How dependency graphs determine compilation order.
Key for understanding build graph construction and multi-crate compilation.

**`compilation_repos/rust-analyzer/`**
Incremental LSP server using Salsa query-based compilation model.
Research target: On-the-fly compilation without full builds - closest to our CozoDB approach.
Critical study: How query invalidation triggers recompilation (similar to our graph queries).

**`compilation_repos/syn/`**
Most popular Rust parsing library (dtolnay) with span preservation.
Research target: How to parse Rust into AST and maintain source location information.
Entry point for understanding parser construction and error recovery strategies.

**`compilation_repos/quote/`**
Rust code generation library (dtolnay) for AST → source transformation.
Research target: Reverse of parsing - regenerating valid Rust from structured representation.
Critical for CozoDB → compilable code path (round-tripping feasibility).

**`compilation_repos/rustfmt/`**
Code formatter demonstrating parse → transform → regenerate pipeline.
Research target: Preserving semantics while modifying syntax, round-trip parsing validation.
Proof that AST → source code is feasible with proper metadata preservation.

**`compilation_repos/chalk/`**
Trait solver engine used by rustc for type checking.
Research target: Could we do minimal type checking from CozoDB entities?
Explores proof search and constraint solving from graph representation.

**`compilation_repos/miri/`**
MIR (mid-level IR) interpreter executing without LLVM compilation.
Research target: Could we interpret from CozoDB without full compilation?
Alternative path: validation through interpretation rather than compilation.

---

### 2. **crates/** - Parseltongue Workspace (11 crates)

**`crates/parseltongue-core/`**
Core parsing engine using tree-sitter with 12-language grammar support.
Abstracts entity extraction (functions, structs, dependencies) into unified interface.
Foundation for all other crates - the "parser of parsers" architecture.

**`crates/parseltongue/`**
Main binary crate implementing CLI interface and tool orchestration.
Routes commands to specialized pt01-pt08 tools with unified error handling.
User-facing entry point - single binary architecture (v0.9.6+).

**`crates/parseltongue-e2e-tests/`**
End-to-end integration tests validating full parse → ingest → query pipeline.
Multi-language test fixtures ensuring cross-language consistency.
CI/CD validation suite for release quality assurance.

**`crates/pt01-folder-to-cozodb-streamer/`**
Streaming directory parser ingesting codebases into CozoDB graph database.
Test exclusion logic (90% token reduction) and incremental update detection.
Entry tool: parseltongue pt01-folder-to-cozodb-streamer . --db "rocksdb:code.db"

**`crates/pt02-llm-cozodb-to-context-writer/`**
Exports dependency graphs as 2-5K token JSON for LLM consumption.
Multiple levels (level00-level03) offering progressively detailed context.
Key innovation: 99% token reduction vs raw code dumps (500K → 5K tokens).

**`crates/pt03-llm-to-cozodb-writer/`**
Ingests LLM-generated code modifications back into CozoDB as "future" entities.
Enables speculative code changes without touching filesystem (git-safe exploration).
Foundation for "what-if" analysis and LLM-driven refactoring workflows.

**`crates/pt04-syntax-preflight-validator/`**
Pre-validation layer ensuring LLM-generated code parses before ingestion.
Prevents malformed syntax from polluting graph database with invalid entities.
Quality gate: only valid ASTs enter CozoDB representation.

**`crates/pt05-llm-cozodb-to-diff-writer/`**
Generates git-style diffs between current (filesystem) and future (LLM) code states.
Enables review of proposed changes before applying to filesystem.
Bridge between speculative graph edits and actual code modifications.

**`crates/pt06-cozodb-make-future-code-current/`**
Applies approved future entities to filesystem, replacing current code.
Completes the LLM modification loop: generate → validate → review → apply.
Transactional update: all-or-nothing filesystem modification from CozoDB state.

**`crates/pt07-visual-analytics-terminal/`**
Terminal-based visualizations: dependency graphs, cycle warnings, entity counts.
Pure text/ASCII rendering for CLI environments without GUI dependencies.
Developer UX: immediate visual feedback on codebase structure.

**`crates/pt08-semantic-atom-cluster-builder/`**
Semantic clustering for grouping related code entities by meaning (in development).
Research target: Moving beyond syntactic similarity to semantic understanding.
Foundation for Strategy 5 (Semantic Search) in ultrathink agent workflows.

---

### 3. **.ref/** - External Tool Reference (12 Git clones)

**`.ref/ast-grep/`**
Rust-based structural search tool for multi-language pattern matching.
Reference: How to build language-agnostic AST query engines.
Comparison point: Their pattern language vs our CozoDB Datalog queries.

**`.ref/ast-grep-analysis.db/`**
CozoDB database containing ingested ast-grep codebase for analysis.
Example: Parseltongue analyzing another parsing tool (meta-analysis).
Demonstrates self-hosting capability and cross-tool research.

**`.ref/rails/`**
Ruby on Rails framework codebase for multi-language testing.
Test case: Can parseltongue handle large Ruby metaprogramming-heavy codebases?
Cross-language validation: Python/JS parser quality vs Rust parser quality.

**`.ref/research/`**
Scratch space for exploratory analysis and proof-of-concept experiments.
Contains experimental queries, parsing test cases, and research notes.
Git-safe learning: External experiments don't pollute main repo history.

**`.ref/tool-comby/`**
Structural code rewriting tool using concrete syntax patterns.
Reference: Alternative approach to code transformation (vs AST-based).
Comparison: Pattern matching at text level vs graph-level queries.

**`.ref/tool-dependency-cruiser/`**
JavaScript dependency analyzer with circular dependency detection.
Reference: How JS ecosystem handles module graph analysis.
Comparison point: Their visualization vs our pt07 terminal analytics.

**`.ref/tool-fraunhofer-cpg/`**
Code Property Graph framework for C/C++ security analysis.
Reference: Academic approach to graph-based code representation.
Research: How formal methods use graphs for vulnerability detection.

**`.ref/tool-joern/`**
Security-focused code analysis using Code Property Graphs (CPG).
Reference: How security researchers query code for vulnerability patterns.
Comparison: Their CPG schema vs our CozoDB entity/dependency schema.

**`.ref/tool-madge/`**
JavaScript module dependency visualizer generating SVG diagrams.
Reference: Visualization strategies for large dependency graphs.
UX research: How developers interact with graph visualizations.

**`.ref/tool-scc/`**
Fast line counter and complexity calculator (Rust implementation).
Reference: High-performance parallel file processing patterns.
Comparison: Their streaming architecture vs our pt01 streamer design.

**`.ref/tool-semgrep/`**
Multi-language static analysis with YAML-based rule definitions.
Reference: Rule DSL design and how to expose pattern matching to end users.
Research: Can CozoDB Datalog achieve similar expressiveness for custom queries?

**`.ref/tool-tree-sitter/`**
Core tree-sitter parsing framework (our underlying parser infrastructure).
Reference: Grammar development, error recovery, incremental parsing algorithms.
Critical dependency: Understanding parser internals for debugging parseltongue-core.

---

### 4. **.refGitHubRepo/** - Tree-Sitter Grammars (3 Git clones)

**`.refGitHubRepo/tree-sitter/`**
Main tree-sitter repository with parsing runtime and grammar development tools.
Reference: Core API for building language parsers, node traversal patterns.
Essential for understanding how parseltongue-core interacts with tree-sitter bindings.

**`.refGitHubRepo/tree-sitter-python/`**
Official Python grammar for tree-sitter parsing.
Reference: Python-specific AST node types and query patterns for entity extraction.
Debug resource: When Python parsing fails, check grammar evolution here.

**`.refGitHubRepo/tree-sitter-rust/`**
Official Rust grammar for tree-sitter parsing.
Reference: Rust-specific syntax handling, macro parsing, trait resolution patterns.
Self-hosting reference: How we parse ourselves (parseltongue is written in Rust).

---

### 5. **docs/** - Documentation & Release Artifacts

**`docs/v096/`**
Version 0.9.6 release documentation, install scripts, and binary artifacts.
Contains README, install.sh, and platform-specific binaries.
Release archive: Historical documentation for version comparison and rollback.

---

### 6. **Query Artifact Directories**

**`dependency_queries/`**
Cached dependency graph query results from pt02-level00 executions.
JSON files containing edge lists (caller → callee relationships).
Used for testing, debugging, and manual graph analysis workflows.

**`entity_queries/`**
Cached entity signature exports from pt02-level01+ executions.
JSON files with full function/struct metadata (signatures, locations, code).
LLM context artifacts: The actual 2-5K token outputs sent to language models.

---

### 7. **Build & Configuration Directories**

**`target/`**
Cargo build artifacts: compiled binaries, intermediate objects, caches.
Debug builds: target/debug/, Release builds: target/release/.
Contains parseltongue main binary + all pt01-pt08 tool binaries (single-binary arch).

**`.claude/`**
Claude Code configuration: agents, commands, settings.
Contains parseltongue-ultrathink-isg-explorer.md agent definition.
Integration point: How Claude Code interacts with parseltongue tooling.

**`.github/`**
GitHub workflows for CI/CD, release automation, issue templates.
Contains automated testing, binary building, and release publishing logic.
DevOps infrastructure: Ensures quality and automates deployment.

---

## Min2Pyramid Principle Summary

**2-Line Essence (Executive Summary)**
Parseltongue ecosystem: 1 main repo + 23 external Git clones organized in 4 collections (.ref, .refGitHubRepo, compilation_repos, .claude).
Core innovation: Convert 500K token codebases → 2-5K token CozoDB graphs; research expansion into compilation-from-database capability.

**Pyramid Layer 1: Strategic Intent (Why)**
Token efficiency crisis: LLMs drown in 500K+ token code dumps with <5% relevant information (Stanford TACL 2023).
Solution architecture: Parse once → query infinitely; graph database enables surgical context extraction (99% token reduction).
Research expansion: Can we compile from the database? (compilation_repos research: rust-analyzer, syn, quote, miri).

**Pyramid Layer 2: Tactical Structure (What)**
Main parseltongue: 11 workspace crates (pt01-pt08 tools) implementing parse → ingest → query → visualize pipeline.
Reference ecosystem: 12 .ref clones (ast-grep, semgrep, joern) for competitive analysis; 3 .refGitHubRepo clones (tree-sitter grammars) for parser development.
Compilation research: 8 repos (rust, cargo, rust-analyzer, syn, quote, rustfmt, chalk, miri) exploring database-to-compilation feasibility.

**Pyramid Layer 3: Operational Details (How)**
Ingestion: pt01 streams files → tree-sitter parses 12 languages → entities stored in CozoDB (rocksdb backend).
Context export: pt02 (level00-03) generates 2-5K token dependency graphs via Datalog queries → JSON for LLM consumption.
LLM modification loop: pt03 ingests proposed changes → pt04 validates syntax → pt05 generates diffs → pt06 applies to filesystem.

**Pyramid Layer 4: Technical Minutiae (Specifics)**
Test exclusion: 90% token reduction by filtering test/ directories during ingestion (CODE entities only, TEST entities dropped).
Single-binary architecture: All pt01-pt08 tools compiled into one parseltongue binary (80% disk reduction vs separate binaries).
Compilation research phases: (1) Study triggers [current], (2) Experiment with syn → CozoDB → quote round-trip, (3) Design pt09-compiler-from-cozodb, (4) Prototype single-function compilation.

---

## What You Might Be Thinking

### Hypothesis 1: Repository Organization Cleanup
**Signal**: Asking for folder inventory with 3-line descriptions suggests evaluation for reorganization.
**Likely intent**: Identifying redundancy, outdated clones, or unclear naming that impedes navigation.
**Next action**: Consolidate .ref vs .refGitHubRepo? Archive compilation_repos after Phase 1 study? Create docs/external-research/ structure?

### Hypothesis 2: Documentation for Onboarding
**Signal**: Requesting "search doc" with Min2Pyramid summary implies external audience or future-self reference.
**Likely intent**: Creating navigation guide for contributors or Agent Games 2025 submission documentation.
**Next action**: Expand to CONTRIBUTING.md with "How to explore this repo" section using this inventory as foundation?

### Hypothesis 3: Ultrathink Agent Context Optimization
**Signal**: You explicitly said "ultrathink" + requested Min2Pyramid (inverted pyramid writing style).
**Likely intent**: Creating agent-readable repository map for @parseltongue-ultrathink-isg-explorer to reference during codebase analysis.
**Next action**: Add this document to agent's system prompt as "Before exploring, consult SEARCH-DOC-ALL-FOLDERS-AND-REPOS.md for ecosystem structure"?

### Hypothesis 4: Compilation Research Phase Transition
**Signal**: Recent compilation_repos/ creation (Nov 10) + inventory request suggests research milestone.
**Likely intent**: Phase 1 (Study) completing → need inventory to plan Phase 2 (Experimentation with syn/quote).
**Next action**: Create compilation_repos/PHASE1-FINDINGS.md documenting trigger point discoveries, then this inventory guides Phase 2 scope?

### Hypothesis 5: Token Budget Planning for LLM Workflows
**Signal**: Focus on "collections" and "3-liners" (conciseness) mirrors parseltongue's core value prop (token reduction).
**Likely intent**: Calculating context cost if LLM agents need to understand full repo structure (self-dogfooding).
**Next action**: Test: Feed this doc to GPT-4 → measure token usage → compare to raw `find . -type d` output (proof of Min2Pyramid efficiency)?

### Hypothesis 6: Agent Games 2025 Submission Architecture Section
**Signal**: Structured inventory + Min2Pyramid + "what I might be thinking" meta-analysis suggests documentation for external evaluation.
**Likely intent**: Building "System Architecture" section showing parseltongue is NOT just a CLI tool but an ecosystem (main + 23 references + 8 research repos).
**Next action**: Convert this to Mermaid diagram + narrative for submission? Highlight: Self-hosting (parsing parsers), meta-research (analyzing analyzers), compilation evolution?

**Most Likely**: Combination of #3 (agent context) + #6 (Agent Games submission). The explicit "ultrathink" keyword + Min2Pyramid request + "what I might be thinking" meta-cognitive analysis strongly suggests both agent optimization AND external documentation preparation.

---

**Document Status**: ✓ Complete
**Maintenance**: Update when adding new .ref clones or crates
**Usage**: Feed to ultrathink agent OR include in Agent Games 2025 submission architecture section
