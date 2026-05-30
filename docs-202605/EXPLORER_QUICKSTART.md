# Parcel Tongue Explorer - Quick Start Guide

## TL;DR - The Vision

**Problem**: Agents use grep → Get 500 files → Confused → Slow

**Solution**: Agents use maps → Get ordered context → Fast & accurate

---

## What You Get

### Before (Traditional Approach)
```bash
User: "How do payment flows work?"

Agent:
  1. grep -r "payment" src/          # 500+ results
  2. Read 20 random files            # 5 minutes
  3. Still confused about flow       # Guessing
  4. Context: Unordered, incomplete  # Poor quality
```

### After (Map-Driven)
```bash
User: "How do payment flows work?"

Agent:
  1. Check map: payment-systems.yaml             # Map exists!
  2. Run: parseltongue-explorer query --strategy find-payment-flow
  3. Read: outputs/2025-11-08_find-payment-flow/llm-context.md
  4. Context: Ordered, complete, cross-repo      # High quality

Time: 30 seconds (vs 5 minutes)
Accuracy: 95% (vs 60%)
```

---

## The Three Core Concepts

### 1. **Maps** (YAML configs)
```yaml
# .parseltongue-explorer/maps/payment-systems.yaml

repositories:
  - name: "stripe-node"
    url: "https://github.com/stripe/stripe-node.git"
    database: "stripe-node.db"  # Auto-named!

strategies:
  - name: "find-payment-flow"
    databases: ["stripe-node.db"]
    queries:
      - where: "entity_name CONTAINS 'payment'"
```

**Think of maps as**: Saved search configurations that anyone can reuse.

### 2. **Databases** (Auto-named)
```
.parseltongue-explorer/databases/
├── stripe-node.db       # One DB per repo
├── payment-backend.db
└── rust-analyzer.db
```

**Naming rule**: `{repository-name}.db` (you never have to think about it)

### 3. **LLM Context** (Ordered documents)
```markdown
# llm-context.md (auto-generated)

Read in this order:
1. Public API Surface      ← Start here
2. Core Business Logic     ← Main functions
3. Database Layer          ← Persistence
4. Error Handling          ← Failure modes
5. Dependency Graph        ← How it connects
```

**Purpose**: Give LLM exactly what it needs, in the right order.

---

## Example Workflow

### Scenario: Learn Rust CLI Patterns from 5 Reference Repos

#### Step 1: Use Pre-made Map
```bash
# We've already created: examples/rust-cli-tools.yaml
# It indexes: ripgrep, fd, bat, dust, exa

ls examples/
# rust-cli-tools.yaml ✓
```

#### Step 2: Run the Explorer (When Implemented)
```bash
# Future command (Week 2):
parseltongue-explorer run \
  --map examples/rust-cli-tools.yaml \
  --all-strategies

# What happens:
# 1. Clones 5 repos → .parseltongue-explorer/clones/
# 2. Indexes each → .parseltongue-explorer/databases/{repo}.db
# 3. Runs 5 strategies:
#    - find-cli-parsing
#    - find-main-functions
#    - find-error-handling
#    - find-parallel-processing
#    - find-config-loading
# 4. Generates outputs → .parseltongue-explorer/outputs/{date}_{strategy}/
```

#### Step 3: Agent Reads Context
```bash
# Agent prompt: "How do Rust CLI tools parse arguments?"

# Agent reads:
cat .parseltongue-explorer/outputs/2025-11-08_find-cli-parsing/llm-context.md

# Gets:
# - 47 functions across 5 repos
# - Ordered by: Public API → Core → Helpers
# - With signatures + dependencies
# - Ready to use!
```

---

## Manual Workflow (Current - Week 1)

Since the explorer tool isn't built yet, here's how to achieve similar results manually:

### Step 1: Clone Repos
```bash
mkdir -p .parseltongue-explorer/clones
cd .parseltongue-explorer/clones

git clone https://github.com/BurntSushi/ripgrep.git
git clone https://github.com/sharkdp/fd.git
git clone https://github.com/sharkdp/bat.git
```

### Step 2: Index Each Repo
```bash
cd ../../  # Back to project root
mkdir -p .parseltongue-explorer/databases

# Index ripgrep
./parseltongue pt01-folder-to-cozodb-streamer \
  .parseltongue-explorer/clones/ripgrep \
  --db "rocksdb:.parseltongue-explorer/databases/ripgrep.db"

# Index fd
./parseltongue pt01-folder-to-cozodb-streamer \
  .parseltongue-explorer/clones/fd \
  --db "rocksdb:.parseltongue-explorer/databases/fd.db"

# Index bat
./parseltongue pt01-folder-to-cozodb-streamer \
  .parseltongue-explorer/clones/bat \
  --db "rocksdb:.parseltongue-explorer/databases/bat.db"
```

### Step 3: Query Strategy (Example: Find Main Functions)
```bash
mkdir -p .parseltongue-explorer/outputs/2025-11-08_find-main-functions

# Query ripgrep
./parseltongue pt02-level01 \
  --db "rocksdb:.parseltongue-explorer/databases/ripgrep.db" \
  --where-clause "entity_name = 'main'" \
  --output .parseltongue-explorer/outputs/2025-11-08_find-main-functions/ripgrep-entities.json

# Query fd
./parseltongue pt02-level01 \
  --db "rocksdb:.parseltongue-explorer/databases/fd.db" \
  --where-clause "entity_name = 'main'" \
  --output .parseltongue-explorer/outputs/2025-11-08_find-main-functions/fd-entities.json

# Query bat
./parseltongue pt02-level01 \
  --db "rocksdb:.parseltongue-explorer/databases/bat.db" \
  --where-clause "entity_name = 'main'" \
  --output .parseltongue-explorer/outputs/2025-11-08_find-main-functions/bat-entities.json
```

### Step 4: Manually Combine Results
```bash
# Combine JSON files
cat .parseltongue-explorer/outputs/2025-11-08_find-main-functions/*.json \
  | jq -s 'add' \
  > .parseltongue-explorer/outputs/2025-11-08_find-main-functions/combined.json

# View results
cat .parseltongue-explorer/outputs/2025-11-08_find-main-functions/combined.json | jq .
```

---

## The Automated Future (Week 2+)

Once the explorer tool is built, all of the above becomes:

```bash
# One command replaces 10+ manual steps
parseltongue-explorer run --map rust-cli-tools.yaml --all-strategies

# Output:
# ✓ Cloned 5 repos
# ✓ Indexed 5 databases (2,347 total entities)
# ✓ Ran 5 strategies
# ✓ Generated 5 context documents
# Time: 47 seconds
```

---

## How Agents Should Use This

### Old Agent Behavior (Discouraged)
```python
# Agent: "I need to find payment functions"
Grep("payment", "src/")  # ❌ Slow, unordered
Read("src/payment.rs")   # ❌ Might miss dependencies
```

### New Agent Behavior (Encouraged)
```python
# Agent: "I need to find payment functions"

# 1. Check if map exists
if exists(".parseltongue-explorer/maps/payment-systems.yaml"):
    # 2. Run strategy
    run("parseltongue-explorer query --map payment-systems.yaml --strategy find-payment-flow")

    # 3. Read context
    context = read(".parseltongue-explorer/outputs/{latest}/llm-context.md")

    # 4. Answer user (with accurate references)
    return context
else:
    # 5. Suggest map creation
    suggest("Let me create a map for payment systems analysis")
```

---

## Agent Integration Pattern

### Update `.claude/agents/parseltongue-ultrathink-isg-explorer.md`

```markdown
# Parcel Tongue Explorer Agent

## Priority Rules

1. **Always check for maps first**:
   - Look in: `.parseltongue-explorer/maps/`
   - If map exists for user's domain → Use it
   - If no map → Suggest creating one

2. **Use map-driven queries instead of grep**:
   ```bash
   # ❌ DON'T
   grep -r "payment" src/

   # ✅ DO
   parseltongue-explorer query --map payment-systems.yaml --strategy find-payment-flow
   ```

3. **Read generated context documents**:
   - Location: `.parseltongue-explorer/outputs/{latest}/llm-context.md`
   - Context is pre-ordered for comprehension
   - Includes cross-repo dependencies

4. **Never use grep when a map exists**:
   - Grep usage should drop to < 5%
   - Only use grep for ad-hoc, one-off searches
   - Suggest map creation for repeated searches

5. **Suggest map creation proactively**:
   - User asks about domain repeatedly → Create map
   - User mentions multiple repos → Create map
   - User asks "how does X work?" → Check if map exists
```

---

## File Organization

```
.parseltongue-explorer/          # All outputs here (never scattered)
├── maps/                         # YAML configs
│   ├── default.yaml
│   └── rust-cli-tools.yaml
│
├── clones/                       # Git repos
│   ├── ripgrep/
│   ├── fd/
│   └── bat/
│
├── databases/                    # One DB per repo
│   ├── ripgrep.db/
│   ├── fd.db/
│   └── bat.db/
│
├── outputs/                      # Time-stamped results
│   ├── 2025-11-08_find-cli-parsing/
│   │   ├── metadata.yaml         # Map, strategy, timestamp
│   │   ├── entities.json         # Level 1 export
│   │   ├── entities.toon         # TOON format (30% smaller)
│   │   └── llm-context.md        # ⭐ Ordered context for LLM
│   │
│   └── 2025-11-09_find-main-functions/
│       └── ...
│
└── web-research/                 # Fetched docs (future)
    └── rust-cli-patterns/
```

**Key insight**: Everything in one place, easy to find, time-stamped.

---

## Success Metrics

### How to Know It's Working

1. **Agents stop using grep**:
   - Measure: Count grep vs map usage
   - Target: < 5% grep when map exists

2. **Context quality improves**:
   - Measure: LLM responses reference correct files/functions
   - Target: > 95% accuracy

3. **Time to insight decreases**:
   - Measure: Time from question to answer
   - Target: < 30 seconds (vs 5+ minutes)

4. **Map creation increases**:
   - Measure: Number of maps in repo
   - Target: 1 → 5 → 10+ maps over time

---

## Common Patterns

### Pattern 1: Learning from Reference Repos
```yaml
# Map: rust-examples.yaml
repositories:
  - ripgrep
  - rust-analyzer
  - tokio

strategies:
  - find-async-patterns
  - find-error-handling
  - find-cli-patterns
```

### Pattern 2: Cross-Repo Analysis
```yaml
# Map: payment-systems.yaml
repositories:
  - stripe-node (JavaScript)
  - payment-backend (Rust)
  - database-schema (SQL)

strategies:
  - find-payment-flow
  - find-error-handling
  - trace-charge-creation
```

### Pattern 3: Dependency Mapping
```yaml
# Map: microservices.yaml
repositories:
  - api-gateway
  - auth-service
  - payment-service
  - notification-service

strategies:
  - find-all-apis
  - trace-cross-service-calls
  - find-circular-dependencies
```

---

## Next Steps

### Week 1 (Current)
- [x] Research document (MAP_DRIVEN_EXPLORER.md)
- [x] Implementation plan (EXPLORER_IMPLEMENTATION_PLAN.md)
- [x] Quick start guide (this file)
- [x] Example map (rust-cli-tools.yaml)
- [ ] Manually test the workflow with 3 repos

### Week 2 (Implementation)
- [ ] Build `parseltongue-explorer` CLI
- [ ] Implement core commands (init, clone, index, query)
- [ ] Test with rust-cli-tools.yaml
- [ ] Measure time savings vs grep

### Week 3+ (Polish)
- [ ] LLM context ordering algorithm
- [ ] Web research integration
- [ ] Incremental indexing
- [ ] Agent prompt updates

---

## Questions?

**Q: Why not just use grep?**
A: Grep gives you 500 unordered results. Maps give you 50 ordered, relevant results with dependencies.

**Q: Do I have to create maps manually?**
A: For now, yes. Future: Agents can suggest/create maps automatically.

**Q: Can I query multiple databases at once?**
A: Yes! That's the whole point. One strategy, multiple databases, combined results.

**Q: Where do outputs go?**
A: Always `.parseltongue-explorer/outputs/{date}_{strategy}/` - never scattered.

**Q: How do I share maps with my team?**
A: Commit maps to repo (maps/*.yaml). The databases stay local (gitignored).

**Q: What if a repo updates?**
A: Run `parseltongue-explorer update --map X` (future command) to re-clone & re-index.

---

## The Ultimate Goal

```
User: "I want to build a payment system like Stripe"

Agent:
1. Checks for map: stripe-reference.yaml
2. If not exists, creates it (clones Stripe SDK, Stripe CLI, etc.)
3. Runs strategies:
   - find-api-design-patterns
   - find-error-handling
   - find-retry-logic
   - find-webhook-patterns
4. Reads: .parseltongue-explorer/outputs/{latest}/llm-context.md
5. Responds:
   "Based on Stripe's architecture (indexed from 3 repos + docs),
    here's a step-by-step plan with code examples..."

The agent never used grep. It navigated the pre-built map.
```

---

**Remember**: The map is the territory. Build maps, forget grep, navigate context.
