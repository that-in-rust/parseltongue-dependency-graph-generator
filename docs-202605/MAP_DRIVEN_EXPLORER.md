# Map-Driven Parcel Tongue Explorer
## The Ultimate Context Gathering System

**Philosophy**: *"Users should forget grep exists. They navigate maps built by Parcel Tongue."*

---

## 1. Core Vision

### The Problem Today
- Outputs scattered everywhere (no centralized folder)
- Manual database naming for each operation
- No automation for git clones → index → query workflows
- No multi-repo queries
- Agents still think about grep/rg instead of using pre-built maps

### The Solution: Map-Driven Intelligence
```
┌─────────────────────────────────────────────────────────┐
│  USER: "I need to understand payment flows"             │
└─────────────────────────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────┐
│  EXPLORER: Reads map → Clones repos → Indexes → Builds │
│            context documents → Orders by relevance      │
└─────────────────────────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────┐
│  LLM: "Here are 5 documents in this order. Read them."  │
└─────────────────────────────────────────────────────────┘
```

---

## 2. Centralized Folder Structure

### The Default Layout
```
.parseltongue-explorer/
├── maps/                           # Map definitions (YAML)
│   ├── default.yaml                # Auto-created on init
│   ├── payment-systems.yaml        # Example: Multi-repo analysis
│   └── rust-examples.yaml          # Example: Learning from references
│
├── clones/                         # Git repositories
│   ├── stripe-node/
│   ├── payment-backend/
│   └── rust-analyzer/
│
├── databases/                      # One DB per repository
│   ├── stripe-node.db/
│   ├── payment-backend.db/
│   ├── rust-analyzer.db/
│   └── _combined_payment-systems.db/   # Merged DB (optional)
│
├── outputs/                        # All analysis results
│   ├── 2025-11-08_payment-flow/
│   │   ├── metadata.yaml           # Map used, timestamp, query
│   │   ├── edges.json              # Level 0 output
│   │   ├── entities.json           # Level 1 output
│   │   ├── entities.toon           # TOON format (30% token reduction)
│   │   ├── report.md               # Human-readable summary
│   │   └── llm-context.md          # Ordered context for LLM
│   │
│   └── 2025-11-09_dependency-graph/
│       ├── metadata.yaml
│       ├── graph.json
│       └── cycles.txt              # Circular dependency warnings
│
├── web-research/                   # Web search results
│   ├── stripe-api-docs/
│   │   ├── index.md
│   │   └── fetched-2025-11-08.json
│   └── rust-patterns/
│
└── cache/                          # Performance optimization
    ├── tree-sitter-grammars/
    └── query-results/
```

### Key Design Principles
1. **One root**: `.parseltongue-explorer/` contains EVERYTHING
2. **Time-stamped outputs**: Never overwrite previous analyses
3. **Metadata everywhere**: Every output folder has `metadata.yaml`
4. **Reproducible**: Same map + same timestamp = exact same results

---

## 3. Map Format (YAML)

### Example: Multi-Repo Payment Analysis
```yaml
# .parseltongue-explorer/maps/payment-systems.yaml

workspace:
  name: "payment-systems"
  description: "Analyze payment processing across Stripe SDK and backend"
  created: "2025-11-08"
  base_dir: ".parseltongue-explorer"

repositories:
  - name: "stripe-node"
    url: "https://github.com/stripe/stripe-node.git"
    branch: "master"
    database: "stripe-node.db"
    languages: ["javascript", "typescript"]
    exclude_patterns:
      - "test/**"
      - "examples/**"

  - name: "payment-backend"
    url: "https://github.com/myorg/payment-backend.git"
    branch: "main"
    database: "payment-backend.db"
    languages: ["rust"]
    exclude_patterns:
      - "target/**"

# Pre-defined search strategies
strategies:
  - name: "find-payment-flow"
    description: "Trace payment processing from API to database"
    databases: ["stripe-node.db", "payment-backend.db"]
    queries:
      - type: "entity-search"
        where: "entity_name CONTAINS 'payment' OR entity_name CONTAINS 'charge'"
        export_level: "level01"  # Signatures only

      - type: "dependency-graph"
        start_nodes: ["stripe:fn:createCharge", "backend:fn:process_payment"]
        depth: 3
        export_level: "level02"  # Full types

  - name: "find-error-handling"
    databases: ["payment-backend.db"]
    queries:
      - type: "entity-search"
        where: "entity_type = 'function' AND interface_signature CONTAINS 'Result<'"
        export_level: "level01"

# Web research to include
web_research:
  - name: "stripe-docs"
    url: "https://stripe.com/docs/api/charges"
    store_in: "web-research/stripe-api-docs/"

  - name: "payment-security"
    search_query: "PCI DSS compliance payment processing 2025"
    max_results: 5
    store_in: "web-research/payment-security/"

# Output configuration
outputs:
  base_folder: "outputs/{{date}}_{{strategy_name}}/"
  formats: ["json", "toon", "markdown"]
  include_metadata: true

  # LLM context ordering
  llm_context:
    order_by: "relevance"  # Options: relevance | file_path | entity_type
    max_tokens: 100000
    prioritize:
      - "function signatures"
      - "struct definitions"
      - "dependency edges"
```

### Example: Reference Repositories (Learning from Examples)
```yaml
# .parseltongue-explorer/maps/rust-examples.yaml

workspace:
  name: "rust-examples"
  description: "Index 5 reference Rust projects for pattern learning"

repositories:
  - name: "rust-analyzer"
    url: "https://github.com/rust-lang/rust-analyzer.git"
    branch: "master"
    database: "rust-analyzer.db"

  - name: "tokio"
    url: "https://github.com/tokio-rs/tokio.git"
    branch: "master"
    database: "tokio.db"

  - name: "ripgrep"
    url: "https://github.com/BurntSushi/ripgrep.git"
    branch: "master"
    database: "ripgrep.db"

  - name: "serde"
    url: "https://github.com/serde-rs/serde.git"
    branch: "master"
    database: "serde.db"

  - name: "clap"
    url: "https://github.com/clap-rs/clap.git"
    branch: "master"
    database: "clap.db"

strategies:
  - name: "find-cli-patterns"
    databases: ["rust-analyzer.db", "ripgrep.db", "clap.db"]
    queries:
      - type: "entity-search"
        where: "entity_name CONTAINS 'parse' OR entity_name CONTAINS 'cli'"

  - name: "find-async-patterns"
    databases: ["tokio.db", "rust-analyzer.db"]
    queries:
      - type: "entity-search"
        where: "interface_signature CONTAINS 'async'"
```

---

## 4. Database Naming Convention

### Automatic Naming Rules
```
Format: {repository-name}.db

Examples:
  stripe-node        → stripe-node.db
  payment-backend    → payment-backend.db
  rust-analyzer      → rust-analyzer.db

Special databases:
  _combined_{workspace-name}.db    # Merged database (all repos)
  _temp_{timestamp}.db             # Temporary query results
  _cache_{query-hash}.db           # Cached query results
```

### Database Metadata (stored in CozoDB)
```datalog
:create DatabaseMetadata {
    db_name: String =>
    repository_url: String,
    repository_branch: String,
    cloned_at: String,
    indexed_at: String,
    last_updated: String,
    language: String,
    entity_count: Int,
    dependency_count: Int,
}
```

---

## 5. Query Workflows

### Workflow 1: Simple Entity Search
```bash
# User command
parseltongue-explorer query \
  --map payment-systems.yaml \
  --strategy find-payment-flow

# What happens behind the scenes:
# 1. Read map → Identify databases [stripe-node.db, payment-backend.db]
# 2. Execute query on each DB:
#    WHERE: entity_name CONTAINS 'payment' OR entity_name CONTAINS 'charge'
# 3. Export level01 (signatures) to outputs/2025-11-08_find-payment-flow/
# 4. Generate llm-context.md with ordered results
# 5. Print summary: "Found 47 entities across 2 databases"
```

### Workflow 2: Dependency Graph Traversal
```bash
# User command
parseltongue-explorer trace \
  --map payment-systems.yaml \
  --start-node "backend:fn:process_payment" \
  --depth 3

# What happens:
# 1. Find starting node in payment-backend.db
# 2. Query DependencyEdges table (BFS up to depth 3)
# 3. Collect all reachable entities
# 4. Export subgraph as JSON + visualize in terminal (pt07)
# 5. Generate llm-context.md with:
#    - Start node signature
#    - Direct dependencies (depth 1)
#    - Indirect dependencies (depth 2-3)
#    - Ordered by call frequency
```

### Workflow 3: Cross-Repo Pattern Search
```bash
# User command
parseltongue-explorer patterns \
  --map rust-examples.yaml \
  --pattern "async fn.*parse"

# What happens:
# 1. Read map → 5 databases
# 2. Query each DB for interface_signature matching regex
# 3. Cluster similar patterns (pt08 - semantic clustering)
# 4. Export:
#    - patterns.json (all matches)
#    - clusters.json (grouped by similarity)
#    - llm-context.md (best examples from each cluster)
```

---

## 6. Datalog Query Templates

### Template 1: Find All Callers of a Function
```datalog
# Stored in: .parseltongue-explorer/cache/query-templates/find-callers.cozo

?[caller_name, caller_file, caller_signature] :=
  *DependencyEdges{ from_key, to_key, edge_type },
  *CodeGraph{ ISGL1_key: from_key, entity_name: caller_name,
              file_path: caller_file, interface_signature: caller_signature },
  *CodeGraph{ ISGL1_key: to_key, entity_name: $target_function },
  edge_type = "calls"

:order -caller_name
```

### Template 2: Find Circular Dependencies
```datalog
# Stored in: .parseltongue-explorer/cache/query-templates/find-cycles.cozo

# Find all cycles using recursive query
cycle[node1, node2] :=
  *DependencyEdges{ from_key: node1, to_key: node2 }

cycle[node1, node3] :=
  cycle[node1, node2],
  *DependencyEdges{ from_key: node2, to_key: node3 }

?[cycle_path] :=
  cycle[node, node],  # Node calls itself (direct or indirect)
  *CodeGraph{ ISGL1_key: node, entity_name: name, file_path: path },
  cycle_path = [name, path]
```

### Template 3: Find All Public APIs
```datalog
# Stored in: .parseltongue-explorer/cache/query-templates/find-public-apis.cozo

?[api_name, file_path, signature] :=
  *CodeGraph{ entity_name: api_name, file_path, interface_signature: signature },
  interface_signature ~ "pub fn",  # Rust public functions
  entity_class = "CODE"

:order api_name
```

---

## 7. LLM Context Generation

### The Critical Innovation: Ordered Context Documents

**Problem**: LLMs need context in the right order to understand codebases.

**Solution**: Generate `llm-context.md` with strategic ordering.

#### Example Output: `llm-context.md`
```markdown
# Context Document: Payment Flow Analysis
Generated: 2025-11-08 10:30:00
Map: payment-systems.yaml
Strategy: find-payment-flow

---

## Reading Order Recommendation

Read these documents in this order for maximum comprehension:

1. **Public API Surface** (Start here - understand interfaces first)
2. **Core Business Logic** (Main payment processing functions)
3. **Database Layer** (Persistence and state management)
4. **Error Handling** (How failures are managed)
5. **Dependency Graph** (How components connect)

---

## 1. Public API Surface (Stripe Node SDK)

### stripe:fn:createCharge (stripe-node/lib/charges.js:45-78)
```typescript
async function createCharge(params: ChargeParams): Promise<Charge> {
  // Signature only (Level 1 export)
}
```

**Callers**:
- backend:fn:process_payment (payment-backend/src/stripe_integration.rs:120)
- backend:fn:retry_failed_charge (payment-backend/src/retry.rs:45)

---

## 2. Core Business Logic (Payment Backend)

### backend:fn:process_payment (payment-backend/src/stripe_integration.rs:120-180)
```rust
pub async fn process_payment(
    order: Order,
    customer: Customer,
) -> Result<PaymentResult, PaymentError> {
    // Full signature + first 10 lines of implementation
}
```

**Dependencies**:
- Calls: stripe:fn:createCharge
- Calls: backend:fn:save_payment_record
- Calls: backend:fn:send_confirmation_email

---

## 3. Database Layer

[... continues with ordered sections ...]

---

## Summary Statistics

- **Total entities analyzed**: 47
- **Databases queried**: 2 (stripe-node.db, payment-backend.db)
- **Languages**: JavaScript, TypeScript, Rust
- **Dependency depth**: 3 levels
- **Estimated tokens**: ~35,000 (fits in Claude context)

---

## Suggested Follow-up Queries

Based on this analysis, you might want to explore:

1. **Error handling patterns**:
   ```bash
   parseltongue-explorer query --map payment-systems.yaml --strategy find-error-handling
   ```

2. **Database transactions**:
   ```bash
   parseltongue-explorer trace --start-node "backend:fn:save_payment_record"
   ```

3. **Web research on PCI compliance**:
   ```bash
   parseltongue-explorer web-research --map payment-systems.yaml --topic payment-security
   ```
```

---

## 8. CLI Commands

### Core Commands
```bash
# Initialize workspace
parseltongue-explorer init [--map-name NAME]
# Creates: .parseltongue-explorer/ folder + default.yaml

# Clone repositories from map
parseltongue-explorer clone --map payment-systems.yaml
# Clones to: .parseltongue-explorer/clones/

# Index repositories
parseltongue-explorer index --map payment-systems.yaml [--repo REPO_NAME]
# Creates databases in: .parseltongue-explorer/databases/

# Run strategy
parseltongue-explorer query --map MAP --strategy STRATEGY_NAME
# Outputs to: .parseltongue-explorer/outputs/{date}_{strategy}/

# Trace dependencies
parseltongue-explorer trace --map MAP --start-node NODE --depth N
# Outputs dependency graph

# Find patterns
parseltongue-explorer patterns --map MAP --pattern REGEX
# Searches across all DBs in map

# Web research
parseltongue-explorer web-research --map MAP --topic TOPIC
# Fetches web content, stores in: .parseltongue-explorer/web-research/

# Update repositories
parseltongue-explorer update --map MAP [--repo REPO_NAME]
# Git pull + re-index

# Generate LLM context
parseltongue-explorer context --map MAP --strategy STRATEGY --max-tokens N
# Creates llm-context.md

# Visualize
parseltongue-explorer viz --map MAP --output graph.svg
# Uses pt07 for terminal visualizations

# Query template
parseltongue-explorer query-template --template find-callers --target-function "process_payment"
# Uses cached Datalog templates
```

### Batch Operations
```bash
# Clone + Index + Query (one command)
parseltongue-explorer run --map payment-systems.yaml --all-strategies

# Index all repos in maps/ folder
parseltongue-explorer index-all

# Update all repositories and re-index
parseltongue-explorer refresh --all
```

---

## 9. Integration with Agents

### Agent Workflow: Forget Grep, Use Maps

#### Before (Old Way)
```bash
# Agent thinks:
"I need to find payment functions"
→ Use Grep tool: grep -r "payment" src/
→ Too many results (500+ files)
→ Use Glob tool: **/*payment*.rs
→ Read 20 files manually
→ Still confused about dependencies
```

#### After (Map-Driven)
```bash
# Agent thinks:
"I need to find payment functions"
→ Check if map exists: .parseltongue-explorer/maps/payment-systems.yaml
→ Run: parseltongue-explorer query --map payment-systems.yaml --strategy find-payment-flow
→ Read: .parseltongue-explorer/outputs/2025-11-08_find-payment-flow/llm-context.md
→ **DONE** - Context is pre-ordered and ready
```

### Agent Prompt Integration
```markdown
# .claude/agents/parseltongue-ultrathink-isg-explorer.md

When the user asks about code understanding, dependency analysis, or pattern finding:

1. **Check for existing maps** first:
   - Look in: .parseltongue-explorer/maps/
   - If map exists for this domain → Use it
   - If no map → Suggest creating one

2. **Use map-driven queries** instead of grep:
   - parseltongue-explorer query --map X --strategy Y
   - parseltongue-explorer trace --start-node Z
   - parseltongue-explorer patterns --pattern P

3. **Read generated context documents**:
   - .parseltongue-explorer/outputs/{latest}/llm-context.md
   - Context is pre-ordered for maximum comprehension

4. **Never use grep/rg directly** when a map exists.

5. **Suggest map creation** for new domains:
   - "I notice you're analyzing payment flows. Let me create a map for this."
```

---

## 10. Web Research Integration

### Workflow: Combine Code + Docs + Web Research
```yaml
# In map file:
web_research:
  - name: "stripe-docs"
    url: "https://stripe.com/docs/api/charges"
    store_in: "web-research/stripe-api-docs/"

  - name: "payment-security"
    search_query: "PCI DSS compliance 2025"
    max_results: 5
    store_in: "web-research/payment-security/"
```

### Execution
```bash
parseltongue-explorer web-research --map payment-systems.yaml

# Fetches web content → Stores in .parseltongue-explorer/web-research/
# Generates summary → Adds to llm-context.md

# Final llm-context.md includes:
# 1. Code entities (from databases)
# 2. Dependency graphs (from Datalog queries)
# 3. Official docs (from Stripe website)
# 4. Security best practices (from web search)
```

---

## 11. Performance Optimizations

### Caching Strategy
```
.parseltongue-explorer/cache/
├── query-results/
│   ├── {query-hash}.json       # Cached query results
│   └── {query-hash}.metadata   # Timestamp, databases used
│
├── tree-sitter-grammars/       # Pre-compiled grammars
│   ├── rust.so
│   └── javascript.so
│
└── query-templates/            # Datalog templates
    ├── find-callers.cozo
    ├── find-cycles.cozo
    └── find-public-apis.cozo
```

### Incremental Indexing
```bash
# Only index changed files
parseltongue-explorer index --map MAP --incremental

# Uses git diff to find changed files since last index
# Only re-parses those files
# Updates database incrementally
```

### Parallel Processing
```rust
// Index multiple repos in parallel
repositories.par_iter().for_each(|repo| {
    index_repository(repo);
});

// Query multiple databases in parallel
databases.par_iter().map(|db| {
    execute_query(db, query)
}).collect()
```

---

## 12. Minimalistic Design Principles (JIT Philosophy)

### What Makes It Relentless Yet Minimal?

1. **One command does everything**:
   ```bash
   parseltongue-explorer run --map payment-systems.yaml
   # Clone + Index + Query + Generate context - all in one
   ```

2. **Convention over configuration**:
   - Default folder: `.parseltongue-explorer/`
   - Default map: `default.yaml`
   - Default output: `outputs/{date}_{strategy}/`

3. **Progressive disclosure**:
   - Level 0: Edges only (minimal context)
   - Level 1: Signatures (moderate context)
   - Level 2: Full types (maximum context)

4. **Opinionated workflows**:
   ```bash
   # No need to specify output paths - it's automatic
   parseltongue-explorer query --map X --strategy Y
   # Always outputs to: .parseltongue-explorer/outputs/...
   ```

5. **Smart defaults**:
   - Auto-exclude test folders
   - Auto-detect languages
   - Auto-generate database names
   - Auto-order LLM context

6. **Fail fast**:
   - Map validation on init
   - Database health checks
   - Clear error messages with suggestions

---

## 13. Example End-to-End Workflow

### Scenario: Understanding Payment Flows Across 3 Repos

#### Step 1: Create Map
```bash
parseltongue-explorer init --map-name payment-systems

# Edit .parseltongue-explorer/maps/payment-systems.yaml
# Add 3 repositories: stripe-node, payment-backend, database-schema
```

#### Step 2: Clone & Index
```bash
parseltongue-explorer clone --map payment-systems.yaml
parseltongue-explorer index --map payment-systems.yaml

# Output:
# ✓ Cloned stripe-node → .parseltongue-explorer/clones/stripe-node/
# ✓ Cloned payment-backend → .parseltongue-explorer/clones/payment-backend/
# ✓ Cloned database-schema → .parseltongue-explorer/clones/database-schema/
# ✓ Indexed stripe-node.db (347 entities, 892 dependencies)
# ✓ Indexed payment-backend.db (523 entities, 1247 dependencies)
# ✓ Indexed database-schema.db (89 entities, 234 dependencies)
```

#### Step 3: Run Strategy
```bash
parseltongue-explorer query --map payment-systems.yaml --strategy find-payment-flow

# Output:
# ✓ Found 47 entities matching query
# ✓ Generated dependency graph (depth: 3)
# ✓ Exported to: .parseltongue-explorer/outputs/2025-11-08_find-payment-flow/
# ✓ Created llm-context.md (35,000 tokens)
#
# Files created:
#   - edges.json (Level 0)
#   - entities.json (Level 1)
#   - entities.toon (30% smaller)
#   - llm-context.md (ordered for LLM)
#   - metadata.yaml (reproducibility)
```

#### Step 4: Agent Uses Context
```bash
# Agent prompt: "Explain the payment flow"

# Agent reads: .parseltongue-explorer/outputs/2025-11-08_find-payment-flow/llm-context.md

# Agent response:
"Based on the indexed code, here's the payment flow:

1. API Entry Point: stripe:fn:createCharge (stripe-node/lib/charges.js:45)
2. Backend Handler: backend:fn:process_payment (payment-backend/src/stripe_integration.rs:120)
3. Database Persistence: backend:fn:save_payment_record (payment-backend/src/db.rs:67)
4. Notification: backend:fn:send_confirmation_email (payment-backend/src/notifications.rs:34)

Dependencies:
- process_payment → createCharge (external API)
- process_payment → save_payment_record (internal)
- process_payment → send_confirmation_email (internal)

Circular dependencies detected: None
Error handling: Result<PaymentResult, PaymentError> propagates through the stack

Would you like me to explore any specific part in more depth?"
```

#### Step 5: Web Research (Optional)
```bash
parseltongue-explorer web-research --map payment-systems.yaml --topic payment-security

# Fetches:
# - Stripe API docs
# - PCI DSS compliance guides
# - Payment security best practices

# Adds to llm-context.md:
# - Section: "External Documentation"
# - Links to official docs
# - Security checklist
```

---

## 14. Success Metrics

### How Do We Know It's Working?

1. **Agents stop using grep**:
   - Measure: grep/rg tool usage drops to near-zero
   - Target: < 5% of queries use grep when map exists

2. **Context quality improves**:
   - Measure: LLM responses reference correct code locations
   - Target: > 95% accuracy in code references

3. **Time to understanding decreases**:
   - Measure: Time from "I need to understand X" to useful LLM response
   - Target: < 30 seconds (vs. 5+ minutes with grep)

4. **Users create maps proactively**:
   - Measure: Number of maps created per week
   - Target: Growing adoption (1 → 5 → 10+ maps)

5. **Query reuse**:
   - Measure: % of queries using existing maps vs. ad-hoc grep
   - Target: > 80% map-driven queries

---

## 15. Implementation Roadmap

### Phase 1: Core Infrastructure (Week 1-2)
- [ ] Create `.parseltongue-explorer/` folder structure
- [ ] Implement map YAML parser
- [ ] Implement database naming convention
- [ ] Build `init` and `clone` commands

### Phase 2: Indexing & Queries (Week 3-4)
- [ ] Batch indexing from map
- [ ] Multi-database queries
- [ ] Query template system (Datalog)
- [ ] Output folder generation

### Phase 3: LLM Context Generation (Week 5-6)
- [ ] Context ordering algorithm
- [ ] `llm-context.md` generation
- [ ] Metadata tracking
- [ ] Progressive disclosure (L0/L1/L2)

### Phase 4: Web Research Integration (Week 7)
- [ ] Web fetch integration
- [ ] Web search integration
- [ ] Combined context documents

### Phase 5: Agent Integration (Week 8)
- [ ] Update agent prompts
- [ ] Measure grep usage reduction
- [ ] Optimize for common workflows

### Phase 6: Performance & Polish (Week 9-10)
- [ ] Caching system
- [ ] Incremental indexing
- [ ] Parallel processing
- [ ] Error handling & validation

---

## 16. Open Questions

1. **Map composition**: Should maps be composable? (Include other maps?)
   ```yaml
   includes:
     - base-rust-repos.yaml
     - payment-systems.yaml
   ```

2. **Query language**: Should we support natural language queries?
   ```bash
   parseltongue-explorer ask --map X "Show me all async functions that handle payments"
   # Translates to Datalog automatically
   ```

3. **Differential analysis**: Compare two database versions?
   ```bash
   parseltongue-explorer diff \
     --map payment-systems.yaml \
     --before stripe-node.db.2025-11-01 \
     --after stripe-node.db.2025-11-08
   # Shows: New functions, deleted functions, changed signatures
   ```

4. **Visualization**: Should we generate visual graphs (not just terminal)?
   ```bash
   parseltongue-explorer viz --map X --output graph.svg
   # Generates D3.js interactive graph?
   ```

5. **Cloud sync**: Should maps/databases sync across team?
   ```yaml
   workspace:
     sync:
       provider: "s3"
       bucket: "team-parseltongue-maps"
   ```

---

## 17. Comparison to Traditional Approaches

| Approach | Time | Accuracy | Reusability | LLM-Friendly |
|----------|------|----------|-------------|--------------|
| **grep/rg** | 5+ min | 60% | None | No (unordered) |
| **Tree-sitter search** | 2-3 min | 75% | None | No (too granular) |
| **Manual code reading** | 10+ min | 85% | None | No (human-only) |
| **Map-Driven Explorer** | < 30 sec | 95%+ | High | **Yes** (ordered context) |

---

## 18. Future Vision

### The Ultimate Goal
```bash
# User asks agent: "I want to build a payment system like Stripe"

# Agent:
# 1. Checks for map: stripe-reference-architecture.yaml
# 2. If not exists, creates it (clones Stripe SDK, Stripe CLI, etc.)
# 3. Runs: parseltongue-explorer run --map stripe-reference-architecture.yaml
# 4. Reads: .parseltongue-explorer/outputs/{latest}/llm-context.md
# 5. Responds:
#    "Based on Stripe's architecture (indexed from 3 repos + web docs),
#     here's a step-by-step plan with code examples..."
```

**The agent never used grep. It navigated the pre-built map.**

---

## Conclusion

The Map-Driven Parcel Tongue Explorer transforms code understanding from:
- **Manual search** → **Automatic context gathering**
- **Scattered outputs** → **Centralized, organized results**
- **Ad-hoc queries** → **Reusable map strategies**
- **Raw grep results** → **Ordered LLM context documents**

**Users forget grep exists. They navigate maps built by Parcel Tongue.**

---

## Next Steps

1. **Review this document** - Does it capture the vision?
2. **Prioritize features** - Which parts are MVP vs. future?
3. **Start implementation** - Begin with Phase 1 (core infrastructure)
4. **Test with real repos** - Clone 5 Rust repos, build first map
5. **Iterate** - Measure agent grep usage, optimize workflows

**The map is the territory. Let's build it.**
