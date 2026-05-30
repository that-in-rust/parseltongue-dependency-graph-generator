# Graph Analytics Framework for Parseltongue

**Document Version:** 1.0
**Date:** 2025-11-05
**Status:** Design & Analysis

---

## Executive Summary

This document presents a comprehensive analytics framework for Parseltongue that leverages the dependency graph and Interface Signature Graph (ISG) to provide deep code intelligence. The framework introduces three revolutionary capabilities:

1. **Multi-Level Graph Abstraction** - View code at 6 different zoom levels (System → Package → File → Function → Block → Line)
2. **Automatic Semantic Clustering (ISGL0.5)** - Mathematically discover natural code boundaries between files and functions
3. **Flow-Aware Analytics** - Separate analysis of control flow, data flow, and temporal coupling

The core insight: **The dependency graph + ISG metadata can automatically discover the "natural molecules" of code - semantic clusters that provide optimal context for both LLMs and developers.**

---

## Table of Contents

- [Part 1: High-Level Overview](#part-1-high-level-overview)
  - [The Three Revolutionary Capabilities](#the-three-revolutionary-capabilities)
  - [Why This Matters](#why-this-matters)
  - [Key Benefits](#key-benefits)
- [Part 2: Detailed Technical Design](#part-2-detailed-technical-design)
  - [Multi-Level Graph Abstraction](#multi-level-graph-abstraction)
  - [Automatic Semantic Clustering (ISGL0.5)](#automatic-semantic-clustering-isgl05)
  - [Flow-Aware Analytics](#flow-aware-analytics)
  - [LLM Context Optimization](#llm-context-optimization)
  - [Developer Tools](#developer-tools)
- [Part 3: Implementation Roadmap](#part-3-implementation-roadmap)
- [Part 4: Mathematical Foundations](#part-4-mathematical-foundations)
- [Appendix: References and Further Reading](#appendix-references-and-further-reading)

---

# Part 1: High-Level Overview

## The Three Revolutionary Capabilities

### 1. Multi-Level Graph Abstraction

**The Problem:** Currently, we analyze code at only two levels:
- **File level**: "auth.rs depends on user.rs" - too vague!
- **Function level**: "validate() calls check_email() calls sanitize()..." - too detailed!

**The Solution:** Create 6 distinct graph projections from the same ISG data:

```
LEVEL 5: System Graph     - Microservices, APIs, external dependencies
LEVEL 4: Package Graph    - src/api, src/core, src/db (folder structure)
LEVEL 3: File Graph       - auth.rs, user.rs, session.rs
LEVEL 2: Function Graph   - validate(), authenticate(), hash_password()
LEVEL 1: Block Graph      - if/else blocks, loops, match arms
LEVEL 0: Line Graph       - Individual variable data flow
```

**Analogy:** Like looking at a city from different altitudes:
- From space: You see countries (System level)
- From a plane: You see cities (Package level)
- From a drone: You see neighborhoods (File level)
- From a building: You see apartments (Function level)
- Inside apartment: You see rooms (Block level)
- Inside room: You see furniture (Line level)

**Each level reveals different problems!**

### 2. Automatic Semantic Clustering (ISGL0.5)

**The Problem:**
- Functions are too granular (45 tokens each, missing context)
- Files are arbitrary human boundaries (2,400 tokens, too much irrelevant code)

**The Solution:** Automatically discover "natural semantic units" using graph theory.

**Example:**
```
FILE LEVEL (Too coarse):
  auth.rs - 2,400 tokens, 23 functions ❌

FUNCTION LEVEL (Too granular):
  validate_email() - 45 tokens ❌
  check_format() - 32 tokens ❌
  sanitize_input() - 67 tokens ❌
  verify_domain() - 54 tokens ❌

ISGL0.5 LEVEL (Just right):
  "input_validation_cluster" - 820 tokens ✅
    ├─ validate_email()
    ├─ check_format()
    ├─ sanitize_input()
    └─ verify_domain()

  Cohesion: 0.94 (highly related)
  Coupling: 0.18 (loosely coupled to others)
```

**Why Together?**
- Call each other frequently (dependency coupling = 0.94)
- Change together historically (temporal coupling = 0.87)
- Share similar signatures (semantic similarity = 0.79)
- Fit perfectly in LLM context (820 tokens vs 4K limit)

### 3. Flow-Aware Analytics

**The Insight:** There are 3 different types of connections between code:

1. **Control Flow**: "Function A calls Function B"
   - Reveals: Who's in charge (boss-employee relationship)
   - Finds: Bottlenecks (one function called by many)

2. **Data Flow**: "Function A's output → Function B's input"
   - Reveals: Where data travels (assembly line)
   - Finds: Security holes (untrusted data → sensitive operations)

3. **Temporal Flow**: "Function A and B always change together"
   - Reveals: Hidden dependencies not visible in code
   - Finds: Missing abstractions (shared concerns)

**Example - Security Violation:**
```
Data Flow Analysis:
  user_input (⚠️ TAINTED)
    → validate_input()
    → sanitize()
    → db_query() (⚠️ SENSITIVE SINK)

Cross-Level Tracking:
  Level 3 (File):     http_handler.rs receives input
  Level 2 (Function): validate_input() processes it
  Level 1 (Block):    if sanitized { ... } else { return }
  Level 0 (Line):     query = "SELECT * WHERE id = " + input

🚨 ALERT: Tainted data reached SQL without proper sanitization!
```

---

## Why This Matters

### For LLMs: Surgical Context Selection

**Problem:** LLMs have fixed context windows. What code do you include?

**Current Approach (Inefficient):**
```
Include the whole file + nearby files
  auth.rs:     2,400 tokens
  user.rs:     3,100 tokens
  session.rs:  1,800 tokens
  ─────────────────────────
  Total:       7,300 tokens

Waste: ~60% irrelevant code
```

**ISGL0.5 Approach (Optimal):**
```
Task: "Fix the email validation bug"

1. Primary cluster:      "input_validation_cluster"    820 tokens ✓
2. Dependencies:         "error_handling_cluster"      340 tokens ✓
3. Related:              "logging_cluster"             220 tokens ✓
4. Temporal coupling:    "user_model_cluster"          560 tokens ✓
   ─────────────────────────────────────────────────────────────
   Total:                                            1,940 tokens

Efficiency: 4× better (75% less waste)
Relevance: 95% of code is directly useful
```

### For Developers: X-Ray Vision

**1. Architectural Violations**
```
Package Graph Analysis:

src/ui/button.rs ──calls──> src/database/connection.rs ❌

VIOLATION: UI layer jumping directly to Database!
  Expected: UI → API → Business → Database
  Actual:   UI ─────────────────> Database (skipped 2 layers)

Suggestion: Route through src/api/handler.rs
```

**2. Change Risk Prediction**
```
Modifying: delete_user()

IMPACT ANALYSIS:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Direct callers:       ████████░░  8 functions
Indirect callers:     ██████████ 39 functions (3-hop)
Tests affected:       ████░░░░░░ 23 tests
Temporal coupling:    ███░░░░░░░ 12 files (change together)

OVERALL RISK: 🔴 HIGH (87/100)

Recommendations:
  1. Add deprecation warning first
  2. Update 23 tests before merging
  3. Consider feature flag for gradual rollout
```

**3. Hidden Module Discovery**
```
Spectral Clustering Analysis:

Found 4 natural modules in "user_service.rs" monolith:

Module 1: "auth_operations" (cohesion: 0.92)
  ├─ login()
  ├─ logout()
  ├─ validate_token()
  └─ refresh_session()

Module 2: "user_management" (cohesion: 0.88)
  ├─ create_user()
  ├─ update_profile()
  └─ delete_user()

Module 3: "email_system" (cohesion: 0.85)
  ├─ send_welcome_email()
  ├─ send_password_reset()
  └─ queue_email()

Module 4: "audit_logging" (cohesion: 0.79)
  ├─ log_event()
  ├─ flush_logs()
  └─ rotate_files()

REFACTORING SUGGESTION:
  Extract these into separate files
  Min-cut cost: Only 12 inter-module dependencies
  Expected maintainability improvement: +45%
```

---

## Key Benefits

### Immediate Value

1. **Better Code Understanding** - See architecture at the right granularity
2. **Safer Refactoring** - Know the blast radius before changing code
3. **Smarter Testing** - Identify critical paths that need tests
4. **Security Insights** - Track data flow from sources to sinks

### LLM Enhancement

1. **4× Context Efficiency** - Include only relevant code
2. **Better Code Generation** - LLM understands semantic boundaries
3. **Accurate Impact Analysis** - LLM knows what else changes
4. **Pattern Recognition** - LLM learns from discovered clusters

### Developer Productivity

1. **Faster Debugging** - Multi-level search (find issue at right level)
2. **Guided Refactoring** - System suggests natural module boundaries
3. **Risk Assessment** - Know before you code
4. **Architectural Compliance** - Continuous validation of design rules

---

# Part 2: Detailed Technical Design

## Multi-Level Graph Abstraction

### Conceptual Model

Code is a **fractal structure** - different zoom levels reveal different patterns. We create graph projections by mathematically collapsing nodes based on their properties.

### The Six Levels

#### Level 5: System Graph
**Granularity:** Microservices, external APIs, system boundaries

**Use Cases:**
- Distributed system architecture
- Service dependency mapping
- Cross-service data flow
- API contract validation

**Example Query:**
```datalog
# Find all external service dependencies
?[service, external_api, call_count] :=
    *DependencyEdges{from_key, to_key, edge_type},
    service = extract_service(from_key),
    is_external(to_key),
    external_api = to_key,
    call_count = count(from_key)
```

#### Level 4: Package Graph
**Granularity:** Folder structure (src/api, src/core, src/db)

**Use Cases:**
- Architectural layer validation
- Module coupling analysis
- Dependency inversion detection
- Package cohesion metrics

**Visualization:**
```
┌─────────────────────────────────────┐
│        PACKAGE ARCHITECTURE         │
├─────────────────────────────────────┤
│ src/api     ══════> src/core        │
│      ║               ║    ║         │
│      ╚════> src/auth ║    ║         │
│                      ║    ║         │
│ src/core    <════════╝    ║         │
│      ║                    ▼         │
│      ╚═══════════> src/database     │
└─────────────────────────────────────┘

Legend:
  ═══> Strong coupling (>10 dependencies)
  ───> Weak coupling (1-10 dependencies)
```

**CozoDB Implementation:**
```datalog
# Project function-level dependencies to package level
package_deps[from_pkg, to_pkg, weight] :=
    *DependencyEdges{from_key, to_key, edge_type},
    from_pkg = extract_package(from_key),
    to_pkg = extract_package(to_key),
    from_pkg != to_pkg,
    weight = count(from_key)

# Detect architectural layers using betweenness centrality
layer_classification[package, layer] :=
    package_deps[package, _, outgoing],
    package_deps[_, package, incoming],
    ratio = outgoing / (incoming + 1),
    layer = case
        when ratio > 10 then 'presentation'
        when ratio > 1 then 'business'
        when ratio > 0.1 then 'domain'
        else 'infrastructure'
    end
```

#### Level 3: File Graph
**Granularity:** Individual files (auth.rs, user.rs)

**Use Cases:**
- File coupling analysis
- Circular dependency detection
- File change impact
- Module boundary identification

**Metrics:**
- **Afferent Coupling (Ca)**: Number of files that depend on this file
- **Efferent Coupling (Ce)**: Number of files this file depends on
- **Instability (I)**: Ce / (Ca + Ce) - ranges from 0 (stable) to 1 (unstable)

**CozoDB Implementation:**
```datalog
# Calculate file-level coupling metrics
file_coupling[file, Ca, Ce, instability] :=
    file_functions[file, funcs],
    Ca = count(*DependencyEdges{to_key} where file(to_key) == file),
    Ce = count(*DependencyEdges{from_key} where file(from_key) == file),
    instability = Ce / (Ca + Ce + 1)

# Find tightly coupled file pairs
tight_coupling[file1, file2, bidirectional_count] :=
    *DependencyEdges{from_key: f1, to_key: f2},
    file(f1) == file1,
    file(f2) == file2,
    *DependencyEdges{from_key: f2_func, to_key: f1_func},
    file(f2_func) == file2,
    file(f1_func) == file1,
    bidirectional_count = count(f1, f2)
```

#### Level 2: Function Graph
**Granularity:** Functions and methods (your current ISG level)

**Use Cases:**
- Call graph analysis
- Function importance ranking (PageRank)
- Critical path identification
- Blast radius calculation

**Algorithms:**
- **PageRank**: Identify most important functions
- **Betweenness Centrality**: Find bottleneck functions
- **Strongly Connected Components**: Detect circular dependencies
- **Shortest Path**: Find call chains

#### Level 1: Block Graph
**Granularity:** Control flow blocks (if/else, loops, match arms)

**Use Cases:**
- Cyclomatic complexity calculation
- Control flow visualization
- Dead code detection
- Branch coverage analysis

**Visualization:**
```
Function: process_request()
┌─────────┐
│ entry   │
└────┬────┘
     ▼
┌─────────┐
│if auth? │──No──→ ┌────────┐
└────┬────┘        │ error  │
    Yes            └───┬────┘
     ▼                 ▼
┌─────────┐       ┌────────┐
│ loop    │←────┐ │ return │
└────┬────┘     │ └────────┘
     ▼          │
┌─────────┐     │
│if done? │──No─┘
└────┬────┘
    Yes
     ▼
┌─────────┐
│ return  │
└─────────┘

Cyclomatic Complexity: 3 (decision points)
```

#### Level 0: Line Graph
**Granularity:** Individual statements and variable data flow

**Use Cases:**
- Data flow analysis
- Taint tracking
- Use-def chains
- Variable liveness

**Example - Taint Analysis:**
```rust
Line 10: let input = request.body();        // ⚠️ TAINTED SOURCE
Line 11: let cleaned = sanitize(input);     // Sanitizer
Line 12: let validated = validate(cleaned); // Validator
Line 13: let query = format!("... {}", validated); // Safe
Line 14: db.execute(&query);                // ✓ SAFE SINK
```

### Implementation Strategy

```rust
// crates/pt-analytics/src/multi_graph.rs

pub enum GraphLevel {
    System,
    Package,
    File,
    Function,
    Block,
    Line,
}

pub struct MultiGraphEngine {
    db: Arc<CozoDbStorage>,
}

impl MultiGraphEngine {
    /// Project dependency graph to specified level
    pub async fn project_to_level(&self, level: GraphLevel) -> Result<ProjectedGraph> {
        let query = self.build_projection_query(level);
        let results = self.db.run_query(&query).await?;

        Ok(ProjectedGraph {
            level,
            nodes: self.extract_nodes(&results)?,
            edges: self.extract_edges(&results)?,
        })
    }

    /// Cross-level analysis: trace path through multiple levels
    pub async fn trace_cross_level(
        &self,
        start: &str,
        end: &str,
    ) -> Result<MultiLevelPath> {
        // Example: Track user input (System) → API endpoint (Package)
        // → Handler function (Function) → SQL query (Line)
    }
}

pub struct ProjectedGraph {
    pub level: GraphLevel,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

// Helper: Extract package from ISGL1 key
// "rust:fn:process:src_core_processor_rs:45" → "src/core"
fn extract_package(isgl_key: &str) -> String {
    let parts: Vec<&str> = isgl_key.split(':').collect();
    let file_path = parts[3].replace('_', "/");
    file_path.split('/').take(2).collect::<Vec<_>>().join("/")
}

// Helper: Extract file from ISGL1 key
// "rust:fn:process:src_core_processor_rs:45" → "src/core/processor.rs"
fn extract_file(isgl_key: &str) -> String {
    let parts: Vec<&str> = isgl_key.split(':').collect();
    parts[3].replace('_', "/") + ".rs"
}
```

---

## Automatic Semantic Clustering (ISGL0.5)

### The Core Problem

**Files are arbitrary boundaries:**
- Developers create files based on conventions, not mathematics
- A file might contain unrelated functions
- Related functions might be split across files

**Functions are too granular:**
- Understanding one function requires context of related functions
- LLMs waste tokens on function boundaries
- Developers jump between many small functions

**We need the "Goldilocks level"** - not too big, not too small, just right!

### The ISGL0.5 Solution

Automatically discover **semantic clusters** - groups of 3-20 functions that:
1. Work together (high cohesion)
2. Are loosely coupled to other groups
3. Fit well in LLM context windows (500-4000 tokens)
4. Represent natural conceptual units

### Mathematical Foundation

We use **graph clustering algorithms** that operate on a weighted graph where:
- **Nodes** = Functions (ISGL1 entities)
- **Edge weights** = Affinity score combining 4 signals

#### The Four Affinity Signals

**1. Dependency Coupling (Weight: 1.0)**
```
Do functions call each other?

Example:
  validate_email() calls check_format()
  validate_email() calls verify_domain()

Dependency coupling = 2 direct calls
```

**2. Data Flow Coupling (Weight: 0.8)**
```
Do functions pass data between each other?

Example:
  sanitize() returns String
  validate() accepts String parameter
  control_flow(sanitize, validate) exists

Data flow coupling = 1
```

**3. Temporal Coupling (Weight: 0.6)**
```
Do functions change together in version history?

Example:
  Last 10 commits:
    - validate_email() changed in commits: [1, 3, 5, 7]
    - check_format() changed in commits: [1, 3, 5, 8]
    - Overlap: 3/4 = 0.75

Temporal coupling = 0.75
```

**4. Semantic Similarity (Weight: 0.4)**
```
Do functions have similar signatures/names?

Example:
  validate_email(input: String) -> Result<Email>
  validate_phone(input: String) -> Result<Phone>

Shared patterns:
  - Common prefix: "validate_"
  - Same parameter type: String
  - Same return pattern: Result<T>

Semantic similarity = 0.68
```

**Combined Affinity:**
```
affinity(func1, func2) =
    1.0 × dependency_coupling(func1, func2) +
    0.8 × data_flow_coupling(func1, func2) +
    0.6 × temporal_coupling(func1, func2) +
    0.4 × semantic_similarity(func1, func2)
```

### Four Clustering Algorithms

#### 1. Spectral Clustering (Recommended)

**Intuition:** Find the "natural modes of vibration" of the function graph.

**Algorithm:**
1. Build affinity matrix W (n×n matrix of pairwise affinities)
2. Compute degree matrix D (diagonal matrix with row sums of W)
3. Compute normalized Laplacian: L = I - D^(-1/2) × W × D^(-1/2)
4. Calculate eigenvalues λ₁, λ₂, ..., λₙ of L
5. Find optimal k using **eigengap heuristic**: k where (λₖ₊₁ - λₖ) is maximum
6. Use first k eigenvectors to embed functions in k-dimensional space
7. Run k-means in this space to get clusters

**Why it works:**
- Eigenvectors capture graph structure
- Points close in spectral space are strongly connected in graph
- Mathematically proven to minimize normalized cut

**Complexity:** O(n³) for eigendecomposition, but can use sparse methods for O(n² log n)

**CozoDB Implementation:**
```datalog
# Build affinity matrix
affinity[f1, f2, score] :=
    dep_coupling[f1, f2, d],
    data_coupling[f1, f2, dt],
    temp_coupling[f1, f2, t],
    sem_similarity[f1, f2, s],
    score = 1.0*d + 0.8*dt + 0.6*t + 0.4*s

# Note: Eigendecomposition done in Rust, then clusters stored back
```

#### 2. Louvain Method (Fast and Scalable)

**Intuition:** Greedily optimize modularity - maximize internal connections, minimize external ones.

**Modularity Formula:**
```
Q = (1/2m) Σ [Aᵢⱼ - (kᵢkⱼ/2m)] δ(cᵢ, cⱼ)

Where:
  m = total edge weight
  Aᵢⱼ = affinity between i and j
  kᵢ = sum of weights of edges attached to i
  δ(cᵢ, cⱼ) = 1 if i and j in same cluster, else 0
```

**Algorithm:**
1. Start: each function in its own cluster
2. For each function, try moving it to neighbor's cluster
3. Keep move if it increases modularity Q
4. Repeat until no improvement
5. Build super-graph where each cluster becomes a node
6. Recurse on super-graph for hierarchical clustering

**Why it works:**
- Directly optimizes a well-defined quality metric
- Hierarchical structure gives multiple granularities (ISGL0.3, ISGL0.5, ISGL0.7)

**Complexity:** O(n log n) - very fast!

**Rust Implementation:**
```rust
fn louvain_cluster(affinity: &AffinityMatrix) -> Vec<Cluster> {
    let mut clusters = initialize_singleton_clusters();
    let mut improved = true;

    while improved {
        improved = false;
        for node in nodes {
            let best_cluster = find_best_cluster(node, &clusters, affinity);
            if best_cluster != node.cluster {
                move_node(node, best_cluster);
                improved = true;
            }
        }
    }

    clusters
}

fn modularity_gain(
    node: &Node,
    target_cluster: &Cluster,
    affinity: &AffinityMatrix
) -> f64 {
    let k_i = node.total_affinity();
    let sum_in = target_cluster.internal_affinity();
    let sum_tot = target_cluster.total_affinity();
    let m = affinity.total_weight();

    (sum_in + affinity(node, target_cluster)) / (2.0 * m)
        - ((sum_tot + k_i) / (2.0 * m)).powi(2)
}
```

#### 3. InfoMap (Information-Theoretic)

**Intuition:** A random walker tends to stay within semantic clusters. Minimize the description length of the walk.

**Algorithm:**
1. Simulate random walks on the function graph
2. Measure how often walker stays within vs leaves clusters
3. Calculate code length: bits needed to describe walk path
4. Optimize partition to minimize code length

**Why it works:**
- Information flow reveals natural boundaries
- Clusters are "information traps" - walker gets stuck inside
- Theoretical foundation in information theory

**Complexity:** O(n log n)

#### 4. Hierarchical Agglomerative Clustering

**Intuition:** Start with individual functions, progressively merge closest pairs.

**Algorithm:**
1. Start: each function is a cluster
2. Compute pairwise distances between all clusters
3. Merge the two closest clusters
4. Recompute distances involving the merged cluster
5. Repeat until desired number of clusters

**Linkage Methods:**
- **Single linkage**: distance = min distance between any two members
- **Complete linkage**: distance = max distance between any two members
- **Average linkage**: distance = average distance
- **Ward linkage**: distance = increase in within-cluster variance (recommended)

**Why it works:**
- Creates a dendrogram showing hierarchy
- Can "cut" at different heights for different granularities
- Ward linkage minimizes variance (creates compact clusters)

**Complexity:** O(n² log n) with efficient priority queue

**Result:** Multiple ISGL levels!
```
Cut at height 10.0 → ISGL0.3 (coarse: 3-5 large clusters)
Cut at height 5.0  → ISGL0.5 (medium: 8-12 clusters) ← Sweet spot
Cut at height 2.0  → ISGL0.7 (fine: 20-30 small clusters)
```

### Cluster Refinement

After initial clustering, refine to optimize for LLM context:

**Constraint: Token Budget**
```
Target: 500-4000 tokens per cluster

If cluster > 4000 tokens:
  - Split using secondary clustering

If cluster < 500 tokens:
  - Merge with most similar neighbor
```

**Optimization: Minimum Description Length (MDL)**
```
MDL(clustering) =
    Description_Length(cluster_structure) +
    Description_Length(cross_cluster_edges)

Goal: Minimize MDL

Where:
  Cluster structure cost = Σ log(cluster_size)
  Cross-cluster edge cost = Σ affinity(e) for edges crossing clusters
```

**Algorithm:**
```rust
fn refine_clusters(mut clusters: Vec<Cluster>) -> Vec<Cluster> {
    loop {
        let mut improved = false;

        // Try moving boundary nodes
        for cluster in &mut clusters {
            for boundary_node in cluster.boundary_nodes() {
                let current_mdl = calculate_mdl(&clusters);

                // Try each neighbor cluster
                for neighbor in boundary_node.neighbor_clusters() {
                    let new_mdl = calculate_mdl_if_moved(
                        &clusters,
                        boundary_node,
                        neighbor
                    );

                    if new_mdl < current_mdl {
                        move_node(boundary_node, neighbor);
                        improved = true;
                        break;
                    }
                }
            }
        }

        if !improved { break; }
    }

    // Ensure token constraints
    enforce_token_constraints(&mut clusters);

    clusters
}
```

### Automatic Cluster Naming

Generate meaningful names automatically:

**Strategy 1: Common Prefix**
```
Functions: validate_email, validate_phone, validate_address
Common prefix: "validate_"
Base name: "validation"
```

**Strategy 2: Dominant Operation**
```
Verbs extracted: [validate, check, verify, sanitize, validate, check]
Frequency: {validate: 2, check: 2, verify: 1, sanitize: 1}
Dominant: "validation"
```

**Strategy 3: Data Type Analysis**
```
Parameters: [Email, Phone, Address, UserInput]
Return types: [Result<Email>, Result<Phone>, Result<Address>]
Common theme: "user input validation"
```

**Combination:**
```rust
fn generate_cluster_name(cluster: &Cluster) -> String {
    let prefix = longest_common_prefix(&cluster.function_names());
    let operations = extract_verbs(&cluster.function_names());
    let dominant_op = operations.most_common();
    let data_types = analyze_signatures(&cluster.functions());

    format!("{}_{}_cluster", dominant_op, data_types.primary())
}
```

**Example Results:**
- `input_validation_cluster`
- `auth_flow_cluster`
- `database_operations_cluster`
- `error_handling_cluster`
- `user_management_cluster`

### Storing ISGL0.5 in CozoDB

```datalog
# Create relation for semantic clusters
:create SemanticCluster {
    cluster_id: String =>
    cluster_name: String,
    functions: [String],        # ISGL1 keys
    cohesion_score: Float,      # 0-1, internal connectivity
    coupling_score: Float,      # 0-1, external connectivity
    token_count: Int,
    centroid_function: String,  # Most representative function
    cluster_level: Float,       # 0.5 for ISGL0.5
    created_at: Timestamp
}

# Index for fast lookup
cluster_membership[function, cluster_id] :=
    *SemanticCluster{cluster_id, functions},
    function in functions

# Metrics
cluster_quality[cluster_id, modularity, mdl] :=
    *SemanticCluster{cluster_id, functions, cohesion_score, coupling_score},
    modularity = cohesion_score - coupling_score,
    mdl = calculate_mdl(cluster_id)
```

---

## Flow-Aware Analytics

### Three Types of Flow

#### 1. Control Flow

**Definition:** The sequence of function calls - who calls whom.

**Use Cases:**
- Call graph visualization
- Bottleneck detection (high betweenness centrality)
- Critical path analysis
- Stack trace prediction

**CozoDB Query:**
```datalog
# Direct control flow
control_flow[caller, callee] :=
    *DependencyEdges{from_key: caller, to_key: callee, edge_type: 'Calls'}

# Transitive control flow (reachability)
reachable[start, end] :=
    control_flow[start, end]

reachable[start, end] :=
    reachable[start, mid],
    control_flow[mid, end]

# Find all paths between two functions
paths[start, end, path, length] :=
    control_flow[start, end],
    path = [start, end],
    length = 1

paths[start, end, path, length] :=
    control_flow[start, mid],
    paths[mid, end, subpath, sublen],
    path = prepend(start, subpath),
    length = sublen + 1
```

**Bottleneck Detection:**
```datalog
# Functions with high betweenness centrality
betweenness[func, centrality] :=
    paths[src, dst, path, _],
    func in path,
    func != src,
    func != dst,
    centrality = count(distinct paths through func)

# Top bottlenecks
?[func, centrality] :=
    betweenness[func, centrality],
    :order -centrality,
    :limit 10
```

#### 2. Data Flow

**Definition:** How data moves through the system - what data does function A produce that function B consumes?

**Use Cases:**
- Security: Taint analysis (user input → SQL query)
- Performance: Hot data paths
- Privacy: PII tracking
- Debugging: Variable provenance

**Type-Based Data Flow:**
```datalog
# Extract return types from signatures
returns_type[func, type] :=
    *CodeGraph{ISGL1_key: func, interface_signature: sig},
    type = extract_return_type(sig)

# Extract parameter types
accepts_type[func, type, param_index] :=
    *CodeGraph{ISGL1_key: func, interface_signature: sig},
    type = extract_param_type(sig, param_index)

# Data flows from A to B if:
# - A returns type T
# - B accepts type T
# - There's a control flow path from A to B
data_flow[source, sink, data_type] :=
    returns_type[source, data_type],
    accepts_type[sink, data_type, _],
    reachable[source, sink]
```

**Taint Analysis:**
```datalog
# Define sources and sinks
taint_source[func] :=
    *CodeGraph{ISGL1_key: func},
    func ~ "request" | func ~ "input" | func ~ "user_data"

taint_sink[func, sink_type] :=
    *CodeGraph{ISGL1_key: func},
    func ~ "execute" | func ~ "query",
    sink_type = 'database'

taint_sink[func, sink_type] :=
    func ~ "eval" | func ~ "exec",
    sink_type = 'code_execution'

# Track tainted data
tainted[func, source, path] :=
    taint_source[source],
    data_flow[source, func, _],
    path = [source, func]

tainted[func, source, path] :=
    tainted[prev, source, prev_path],
    data_flow[prev, func, _],
    path = append(prev_path, func)

# Find vulnerabilities
vulnerability[source, sink, sink_type, path] :=
    tainted[sink, source, path],
    taint_sink[sink, sink_type],
    not sanitized(path)  # Check if sanitizer in path
```

**Example Output:**
```
🚨 SECURITY VULNERABILITY DETECTED

Source: http_request_handler::get_body() [TAINTED]
  ↓ data_flow
validate_input::process()
  ↓ data_flow
build_query::construct()
  ↓ data_flow
Sink: database::execute_raw() [SQL_INJECTION_RISK]

Path: request → validate → build_query → execute
Missing: SQL sanitization/parameterization

Recommendation: Use prepared statements in build_query()
```

#### 3. Temporal Flow (Evolutionary Coupling)

**Definition:** Functions that change together over time, even without code dependencies.

**Use Cases:**
- Hidden dependencies detection
- Missing abstraction identification
- Change prediction
- Team coordination (Conway's Law)

**CozoDB Query:**
```datalog
# Assuming you track changes in CodeGraph temporal states
change_event[entity, timestamp] :=
    *CodeGraph{ISGL1_key: entity, last_modified: timestamp}

# Co-change: entities modified within time window
co_change[e1, e2, time_window] :=
    change_event[e1, t1],
    change_event[e2, t2],
    e1 != e2,
    time_diff = abs(t1 - t2),
    time_diff < 3600,  # 1 hour window
    time_window = time_diff

# Temporal coupling strength
temporal_coupling[e1, e2, correlation] :=
    co_change[e1, e2, _],
    total_changes_e1 = count(change_event[e1, _]),
    total_changes_e2 = count(change_event[e2, _]),
    co_changes = count(co_change[e1, e2, _]),
    correlation = co_changes / min(total_changes_e1, total_changes_e2)

# Hidden dependencies (high temporal coupling but no code dependency)
hidden_dependency[e1, e2, correlation] :=
    temporal_coupling[e1, e2, correlation],
    correlation > 0.7,
    not exists(*DependencyEdges{from_key: e1, to_key: e2}),
    not exists(*DependencyEdges{from_key: e2, to_key: e1})
```

**Example Output:**
```
🔍 HIDDEN DEPENDENCIES DETECTED

auth.rs::validate_token() <──93%──> session.rs::get_user()

Temporal coupling: 0.93 (changed together 28/30 times)
Code coupling: None (no direct calls)

Analysis:
  - Both functions modified in same commits
  - Likely share implicit state (Redis session store?)

Recommendation:
  - Extract shared SessionStore abstraction
  - Make dependency explicit
  - Consider introducing interface
```

### Cross-Flow Analysis

Combine all three flows for deep insights:

```datalog
# Entities critical in ALL three dimensions
critical_entity[entity, control_score, data_score, temporal_score] :=
    betweenness[entity, control_score],
    data_flow_centrality[entity, data_score],
    change_frequency[entity, temporal_score],
    control_score > 0.8,
    data_score > 0.8,
    temporal_score > 0.8
```

**Example:**
```
⚠️ CRITICAL ENTITY ALERT

Function: auth::validate_token()

Control Flow:    ██████████ 95% (called by 47 functions)
Data Flow:       ████████░░ 87% (token used in 34 places)
Temporal Flow:   ███████░░░ 73% (changes weekly)

RISK ASSESSMENT: 🔴 CRITICAL
  - Single point of failure
  - High blast radius
  - Frequent changes increase bug risk

Recommendations:
  1. Increase test coverage (currently 67%)
  2. Add integration tests
  3. Consider circuit breaker pattern
  4. Extract into separate service
```

---

## LLM Context Optimization

### The Problem

LLMs have fixed context windows (e.g., GPT-4: 128K tokens, Claude: 200K tokens). For a large codebase:
- Need to select relevant code
- Too much context → slower, more expensive
- Too little context → wrong answers

**Goal:** Maximize information density in the context window.

### The Solution: Information-Theoretic Context Selection

**Core Idea:** Select clusters that maximize information gain per token.

```
Information Gain = H(Task) - H(Task | Cluster)

Where H = Shannon entropy (uncertainty)
```

**Algorithm:**

```rust
pub struct ContextOptimizer {
    db: Arc<CozoDbStorage>,
    max_tokens: usize,
    min_coherence: f64,
}

impl ContextOptimizer {
    pub async fn build_optimal_context(
        &self,
        focus_entity: &str,
        task_type: TaskType,
    ) -> Result<OptimalContext> {
        // Step 1: Find primary cluster containing focus entity
        let primary = self.find_cluster_containing(focus_entity).await?;

        let mut context = OptimalContext::new(primary.clone());
        let mut remaining_tokens = self.max_tokens - primary.token_count;

        // Step 2: Determine search radius based on task
        let radius = match task_type {
            TaskType::BugFix => 2,      // Need nearby code
            TaskType::Feature => 3,      // Need broader context
            TaskType::Refactor => 1,     // Just direct dependencies
            TaskType::Understand => 4,   // Comprehensive view
        };

        // Step 3: Get candidate clusters within radius
        let candidates = self.get_clusters_within_radius(
            &primary,
            radius
        ).await?;

        // Step 4: Rank by information gain per token
        let mut ranked: Vec<_> = candidates
            .into_iter()
            .map(|cluster| {
                let info_gain = self.calculate_info_gain(&context, &cluster);
                let efficiency = info_gain / cluster.token_count as f64;
                (cluster, info_gain, efficiency)
            })
            .collect();

        ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Step 5: Greedily add clusters until budget exhausted
        for (cluster, info_gain, efficiency) in ranked {
            if cluster.token_count <= remaining_tokens {
                if efficiency > self.min_coherence {
                    context.add_cluster(cluster.clone());
                    remaining_tokens -= cluster.token_count;
                } else {
                    break;  // No more valuable clusters
                }
            }
        }

        Ok(context)
    }

    fn calculate_info_gain(
        &self,
        current_context: &OptimalContext,
        new_cluster: &SemanticCluster,
    ) -> f64 {
        // Measure: How much does adding this cluster reduce uncertainty?

        // Information gain components:
        // 1. Dependency coverage: Does it complete dependency chains?
        let dep_gain = self.dependency_coverage_gain(current_context, new_cluster);

        // 2. Type coverage: Does it provide type definitions we reference?
        let type_gain = self.type_coverage_gain(current_context, new_cluster);

        // 3. Control flow coverage: Does it show callers/callees?
        let control_gain = self.control_flow_gain(current_context, new_cluster);

        // 4. Temporal relevance: Does it change with current focus?
        let temporal_gain = self.temporal_relevance(current_context, new_cluster);

        // Weighted sum
        0.4 * dep_gain +
        0.3 * type_gain +
        0.2 * control_gain +
        0.1 * temporal_gain
    }
}

pub struct OptimalContext {
    pub clusters: Vec<SemanticCluster>,
    pub total_tokens: usize,
    pub coverage_score: f64,
    pub coherence_score: f64,
}

impl OptimalContext {
    pub fn to_llm_context(&self) -> String {
        // Format clusters for LLM consumption
        let mut output = String::new();

        output.push_str("# Code Context\n\n");

        for cluster in &self.clusters {
            output.push_str(&format!("## {}\n\n", cluster.name));
            output.push_str(&format!("Cohesion: {:.2} | ", cluster.cohesion_score));
            output.push_str(&format!("Functions: {}\n\n", cluster.functions.len()));

            for func_key in &cluster.functions {
                // Fetch actual code from CodeGraph
                let code = self.get_code(func_key);
                output.push_str(&format!("```rust\n{}\n```\n\n", code));
            }
        }

        output
    }
}
```

### Task-Specific Context Strategies

**Bug Fix:**
```
Priority:
  1. Function with bug (100%)
  2. Direct callers (80%)
  3. Direct callees (70%)
  4. Test files (60%)
  5. Recently changed together (50%)

Radius: 2 hops
```

**Feature Implementation:**
```
Priority:
  1. Similar existing features (90%)
  2. Interfaces to implement (80%)
  3. Related domain models (70%)
  4. Integration points (60%)
  5. Test examples (50%)

Radius: 3 hops
```

**Code Understanding:**
```
Priority:
  1. Entry points (90%)
  2. Core abstractions (85%)
  3. Public APIs (80%)
  4. Common utilities (70%)
  5. Documentation (60%)

Radius: 4 hops (comprehensive)
```

**Refactoring:**
```
Priority:
  1. Functions to refactor (100%)
  2. Direct dependencies (90%)
  3. Tests (85%)
  4. Interface contracts (75%)
  5. Callers (50%)

Radius: 1 hop (focused)
```

### Metrics

**Context Quality Metrics:**

```rust
pub struct ContextMetrics {
    pub token_efficiency: f64,      // info_gain / tokens
    pub dependency_coverage: f64,   // % of deps included
    pub type_completeness: f64,     // % of types defined
    pub test_coverage: f64,         // % of tests included
    pub temporal_relevance: f64,    // recency score
}

impl ContextMetrics {
    pub fn calculate(context: &OptimalContext) -> Self {
        // Measure quality of context selection
    }
}
```

**Example Output:**
```
📊 CONTEXT OPTIMIZATION RESULTS

Task: Fix authentication bug in validate_token()
Budget: 8,000 tokens
Used: 7,842 tokens (98% utilization)

Selected Clusters:
  1. auth_flow_cluster           1,234 tokens [Primary]
  2. session_management_cluster  1,456 tokens [Dependency]
  3. error_handling_cluster        567 tokens [Error paths]
  4. user_model_cluster            890 tokens [Data types]
  5. auth_tests_cluster          2,134 tokens [Tests]
  6. logging_cluster               561 tokens [Debugging]

Metrics:
  Token Efficiency:      ████████░░ 87%
  Dependency Coverage:   ██████████ 95%
  Type Completeness:     ████████░░ 89%
  Test Coverage:         ███████░░░ 76%
  Temporal Relevance:    █████████░ 92%

Estimated Quality: 🟢 EXCELLENT

Context includes:
  ✓ Primary functions (5)
  ✓ All direct dependencies (12)
  ✓ Relevant type definitions (8)
  ✓ Test examples (15)
  ✓ Error handling paths (4)
  ✓ Recent changes (last 7 days)
```

---

## Developer Tools

### 1. Change Impact Analysis

**Purpose:** Before making a change, understand the blast radius.

**Implementation:**
```rust
pub struct ChangeImpactAnalyzer {
    db: Arc<CozoDbStorage>,
}

impl ChangeImpactAnalyzer {
    pub async fn analyze_impact(
        &self,
        changed_entities: Vec<String>,
    ) -> Result<ImpactReport> {
        // Multi-dimensional impact analysis

        // 1. Control flow impact (who calls this?)
        let control_impact = self.analyze_control_flow_impact(&changed_entities).await?;

        // 2. Data flow impact (who uses the data?)
        let data_impact = self.analyze_data_flow_impact(&changed_entities).await?;

        // 3. Temporal impact (what else changes with this?)
        let temporal_impact = self.analyze_temporal_impact(&changed_entities).await?;

        // 4. Test impact (which tests break?)
        let test_impact = self.analyze_test_impact(&changed_entities).await?;

        Ok(ImpactReport {
            changed_entities,
            control_flow_affected: control_impact,
            data_flow_affected: data_impact,
            temporal_coupled: temporal_impact,
            tests_affected: test_impact,
            risk_score: self.calculate_risk_score(&control_impact, &data_impact, &temporal_impact),
        })
    }
}

pub struct ImpactReport {
    pub changed_entities: Vec<String>,
    pub control_flow_affected: Vec<AffectedEntity>,
    pub data_flow_affected: Vec<AffectedEntity>,
    pub temporal_coupled: Vec<AffectedEntity>,
    pub tests_affected: Vec<String>,
    pub risk_score: f64,  // 0-100
}

pub struct AffectedEntity {
    pub entity: String,
    pub hop_distance: usize,
    pub impact_probability: f64,
    pub impact_type: ImpactType,
}
```

**CozoDB Queries:**
```datalog
# Find all entities affected by changing target (control flow)
control_impact[target, affected, distance, probability] :=
    changed_entity[target],
    paths[_, target, path, distance],
    affected in path,
    affected != target,
    probability = 1.0 / distance  # Closer = higher probability

# Find data flow impact
data_impact[target, affected, data_type] :=
    changed_entity[target],
    data_flow[target, affected, data_type]

data_impact[target, affected, data_type] :=
    data_impact[target, mid, data_type],
    data_flow[mid, affected, data_type]

# Find temporal impact
temporal_impact[target, coupled] :=
    changed_entity[target],
    temporal_coupling[target, coupled, correlation],
    correlation > 0.7
```

**Terminal Output:**
```
🔍 CHANGE IMPACT ANALYSIS

Modifying: auth::validate_token()

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CONTROL FLOW IMPACT (Function Calls)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Direct (1-hop):     ████████░░  8 callers (100% probability)
  • api::handler::authenticate()
  • middleware::auth_check()
  • ...

Indirect (2-hop):   ██████░░░░ 23 callers (50% probability)
  • api::routes::protected_route()
  • ...

Deep (3+ hop):      ███░░░░░░░ 39 callers (25% probability)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
DATA FLOW IMPACT (Data Dependencies)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Token data flows to:
  • session::verify_session() → database::check_active()
  • logger::log_auth_event() → audit::record()

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TEMPORAL COUPLING (Change Together)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Frequently changes with:
  • session::create_session()    [87% correlation]
  • user::get_permissions()      [72% correlation]

⚠️  These have NO code dependency but change together!

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TEST IMPACT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Tests affected: 23 test functions
  • test_valid_token()
  • test_expired_token()
  • test_invalid_signature()
  • ... (20 more)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
OVERALL RISK ASSESSMENT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Risk Score: 🔴 87/100 (HIGH)

Factors:
  • High call frequency         [Critical]
  • Many dependent systems      [High]
  • Recent production incidents [Medium]
  • Moderate test coverage      [Medium]

Recommendations:
  1. ✓ Add integration tests for auth flow
  2. ✓ Update session and user modules together
  3. ✓ Feature flag the change
  4. ✓ Monitor error rates post-deploy
  5. ✓ Have rollback plan ready
```

### 2. Architectural Constraint Checker

**Purpose:** Continuously validate that code follows architectural rules.

**Implementation:**
```rust
pub struct ArchitectureConstraints {
    db: Arc<CozoDbStorage>,
    rules: Vec<ArchRule>,
}

pub enum ArchRule {
    LayerDependency {
        from_layer: String,
        allowed_deps: Vec<String>,
    },
    ForbiddenDependency {
        from_pattern: String,
        to_pattern: String,
    },
    MaxComplexity {
        package: String,
        max_cyclomatic: u32,
    },
    NamingConvention {
        pattern: String,
        required_prefix: String,
    },
}

impl ArchitectureConstraints {
    pub async fn check_violations(&self) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();

        for rule in &self.rules {
            violations.extend(self.check_rule(rule).await?);
        }

        Ok(violations)
    }
}
```

**Example Rules:**
```rust
// Define layers
let layers = vec![
    Layer::new("presentation", "src/api"),
    Layer::new("business", "src/core"),
    Layer::new("domain", "src/domain"),
    Layer::new("infrastructure", "src/db"),
];

// Rule: Presentation can only depend on Business
let rule1 = ArchRule::LayerDependency {
    from_layer: "presentation".to_string(),
    allowed_deps: vec!["business".to_string()],
};

// Rule: Domain cannot depend on Infrastructure
let rule2 = ArchRule::ForbiddenDependency {
    from_pattern: "src/domain/*".to_string(),
    to_pattern: "src/db/*".to_string(),
};

// Rule: Core complexity must be manageable
let rule3 = ArchRule::MaxComplexity {
    package: "src/core".to_string(),
    max_cyclomatic: 50,
};
```

**CozoDB Constraints:**
```datalog
# Layer violation detection
layer_violation[from_entity, to_entity, from_layer, to_layer] :=
    *DependencyEdges{from_key: from_entity, to_key: to_entity},
    entity_layer[from_entity, from_layer],
    entity_layer[to_entity, to_layer],
    not allowed_dependency[from_layer, to_layer]

# Cyclic dependency detection
cycle[entity] :=
    reachable[entity, entity]

# Complexity violations
complexity_violation[entity, complexity, max_allowed] :=
    *CodeGraph{ISGL1_key: entity, cyclomatic_complexity: complexity},
    complexity_limit[package, max_allowed],
    entity_package[entity, package],
    complexity > max_allowed
```

**Terminal Output:**
```
🏛️  ARCHITECTURAL CONSTRAINT VIOLATIONS

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
LAYER VIOLATIONS (3)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

❌ Domain → Infrastructure dependency
   src/domain/user.rs::User::save()
     └──> src/db/connection.rs::execute()

   Rule: Domain layer must not depend on Infrastructure
   Fix: Introduce Repository pattern

❌ UI → Database dependency
   src/api/handler.rs::get_user()
     └──> src/db/users.rs::query()

   Rule: Skip Business layer
   Fix: Route through src/core/user_service.rs

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CYCLIC DEPENDENCIES (1)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

❌ Circular dependency detected
   auth.rs → session.rs → user.rs → auth.rs

   Cycle length: 3
   Fix: Extract shared interface

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
COMPLEXITY VIOLATIONS (2)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

❌ parser.rs::parse() - Cyclomatic: 67 (limit: 50)
   Fix: Extract sub-parsers

❌ validator.rs::validate_all() - Cyclomatic: 58 (limit: 50)
   Fix: Use validation pipeline pattern

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
SUMMARY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Total violations: 6
Critical: 3 (layer violations)
Warning: 2 (complexity)
Info: 1 (cycle)

Architectural Debt Score: 24/100 (needs improvement)
```

### 3. Test Intelligence

**Purpose:** Identify testing gaps and prioritize test creation.

**Implementation:**
```rust
pub struct TestIntelligence {
    db: Arc<CozoDbStorage>,
}

impl TestIntelligence {
    pub async fn analyze_test_effectiveness(&self) -> Result<TestReport> {
        // 1. Find critical paths without tests
        let untested_critical = self.find_untested_critical_paths().await?;

        // 2. Calculate test leverage (how much code each test covers)
        let test_leverage = self.calculate_test_leverage().await?;

        // 3. Identify redundant tests (testing same things)
        let redundant_tests = self.find_redundant_tests().await?;

        // 4. Suggest optimal test placement
        let suggestions = self.suggest_test_targets().await?;

        Ok(TestReport {
            untested_critical,
            test_leverage,
            redundant_tests,
            suggestions,
        })
    }
}
```

**CozoDB Queries:**
```datalog
# Find test entities
test_entity[entity] :=
    *CodeGraph{ISGL1_key: entity, entity_class: 'TestImplementation'}

# Test coverage relationships
test_covers[test, code] :=
    test_entity[test],
    *DependencyEdges{from_key: test, to_key: code, edge_type: 'Calls'},
    not test_entity[code]

# Transitive coverage
test_covers_transitive[test, code] :=
    test_covers[test, code]

test_covers_transitive[test, code] :=
    test_covers[test, mid],
    test_covers_transitive[mid, code]

# Code entities with no tests
untested[entity] :=
    *CodeGraph{ISGL1_key: entity, entity_class: 'Implementation'},
    not exists(test_covers[_, entity])

# Critical untested paths (high importance + no tests)
critical_untested[entity, importance] :=
    untested[entity],
    betweenness[entity, importance],
    importance > 0.7
```

**Terminal Output:**
```
🧪 TEST INTELLIGENCE REPORT

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CRITICAL UNTESTED PATHS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔴 auth::validate_token()
   Importance: ████████░░ 89%
   Called by: 47 functions
   Test coverage: NONE
   Risk: CRITICAL

   Recommendation: Add unit + integration tests
   Estimated effort: 4 hours
   Expected bug reduction: 67%

🔴 payment::process_charge()
   Importance: ██████████ 95%
   Called by: 12 functions
   Test coverage: NONE
   Risk: CRITICAL

   Recommendation: Add comprehensive tests + mocks
   Estimated effort: 8 hours
   Expected bug reduction: 89%

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TEST LEVERAGE ANALYSIS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

High-leverage tests (cover many functions):
✅ test_auth_flow()           Covers: 23 functions
✅ test_payment_integration()  Covers: 18 functions

Low-leverage tests (cover few functions):
⚠️  test_string_trim()        Covers: 1 function
⚠️  test_add_numbers()        Covers: 1 function
   Consider: Delete or merge into broader tests

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
REDUNDANT TESTS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⚠️  test_user_creation() and test_create_user_success()
   Coverage overlap: 95%
   Recommendation: Merge tests

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
RECOMMENDED TESTING PRIORITIES
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. 🔴 payment::process_charge()     Priority: CRITICAL
2. 🔴 auth::validate_token()        Priority: CRITICAL
3. 🟡 database::transaction()       Priority: HIGH
4. 🟡 api::rate_limiter()           Priority: HIGH
5. 🟢 utils::format_date()          Priority: LOW

Total testing debt: 12 critical functions
Estimated effort: 48 hours
Expected defect reduction: 73%
```

### 4. Module Boundary Suggester

**Purpose:** Automatically suggest where to split files or extract modules.

**Implementation:**
```rust
pub struct ModuleBoundarySuggester {
    db: Arc<CozoDbStorage>,
}

impl ModuleBoundarySuggester {
    pub async fn suggest_module_boundaries(&self) -> Result<Vec<ModuleSuggestion>> {
        // Use spectral clustering to find natural boundaries
        let clusters = self.discover_semantic_clusters().await?;

        // Identify clusters that span multiple files (refactoring opportunities)
        let cross_file_clusters = clusters
            .into_iter()
            .filter(|c| self.spans_multiple_files(c))
            .collect();

        // Generate suggestions
        let suggestions = cross_file_clusters
            .into_iter()
            .map(|cluster| self.create_suggestion(cluster))
            .collect();

        Ok(suggestions)
    }
}
```

**Terminal Output:**
```
📦 MODULE BOUNDARY SUGGESTIONS

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
EXTRACTION OPPORTUNITY #1
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Current: "auth_operations" cluster scattered across files
  • auth.rs (4 functions)
  • session.rs (3 functions)
  • middleware.rs (2 functions)

Cohesion: 0.92 (very high!)
Coupling: 0.15 (very low!)

Recommendation: Extract to new file "auth_service.rs"

Benefits:
  ✓ Improved maintainability (+34%)
  ✓ Clearer responsibilities
  ✓ Easier testing
  ✓ Better for LLM context

Min-cut cost: 3 external dependencies (easy to refactor)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
EXTRACTION OPPORTUNITY #2
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Current: "user_management" hidden in monolith "services.rs"

Functions:
  • create_user()
  • update_user()
  • delete_user()
  • find_user()
  • list_users()

Cohesion: 0.89
Lines of code: 450

Recommendation: Extract to "user_service.rs"

This cluster is a natural "bounded context" in DDD terms!
```

---

# Part 3: Implementation Roadmap

## Phase 1: Foundation (Weeks 1-2)

### Sprint 1.1: Multi-Level Graph Projection

**Goals:**
- Implement graph projection engine
- Support Package, File, and Function levels
- Store projections in CozoDB

**Deliverables:**
- `crates/pt-analytics/src/multi_graph.rs`
- CozoDB queries for projection
- Basic CLI: `pt-analytics project --level package`

**Success Criteria:**
- Can visualize dependency graph at 3 levels
- Projection queries run in <100ms for typical codebases

### Sprint 1.2: Basic Metrics

**Goals:**
- Calculate core graph metrics
- Implement betweenness centrality
- Implement PageRank

**Deliverables:**
- `crates/pt-analytics/src/metrics.rs`
- CLI: `pt-analytics metrics --type centrality`

**Success Criteria:**
- Identify top 10 most important functions
- Detect bottleneck functions

## Phase 2: Semantic Clustering (Weeks 3-4)

### Sprint 2.1: Affinity Matrix

**Goals:**
- Implement 4 affinity signals
- Build weighted affinity matrix
- Store in CozoDB

**Deliverables:**
- Dependency coupling calculator
- Data flow coupling analyzer
- Temporal coupling extractor
- Semantic similarity scorer

**Success Criteria:**
- Affinity matrix computed for entire codebase
- CozoDB query returns affinity in <50ms

### Sprint 2.2: Clustering Algorithms

**Goals:**
- Implement Louvain clustering
- Implement spectral clustering (optional)
- Cluster refinement for token constraints

**Deliverables:**
- `crates/pt-analytics/src/clustering.rs`
- Automatic cluster naming
- CLI: `pt-analytics cluster --method louvain`

**Success Criteria:**
- Discover 8-15 semantic clusters for typical files
- Clusters are 500-4000 tokens each
- Modularity score > 0.6

### Sprint 2.3: ISGL0.5 Storage

**Goals:**
- Store clusters in CozoDB
- Create cluster membership index
- Enable cluster-based queries

**Deliverables:**
- SemanticCluster relation in CozoDB
- Query API for cluster lookup

**Success Criteria:**
- Fast cluster lookup (<10ms)
- Persistent across runs

## Phase 3: Flow Analysis (Weeks 5-6)

### Sprint 3.1: Control Flow

**Goals:**
- Implement transitive closure
- Compute shortest paths
- Find strongly connected components

**Deliverables:**
- Control flow analyzer
- Cycle detector
- CLI: `pt-analytics flow --type control`

**Success Criteria:**
- Detect all cycles
- Compute 3-hop blast radius

### Sprint 3.2: Data Flow

**Goals:**
- Extract type information from ISG
- Build data flow graph
- Implement taint analysis

**Deliverables:**
- Data flow tracker
- Security vulnerability detector

**Success Criteria:**
- Track data from input to database
- Identify potential injection vulnerabilities

### Sprint 3.3: Temporal Flow

**Goals:**
- Extract change history
- Compute temporal coupling
- Find hidden dependencies

**Deliverables:**
- Temporal coupling analyzer
- Hidden dependency finder

**Success Criteria:**
- Identify files that change together >70% of the time
- Find at least 3 hidden dependencies in real codebases

## Phase 4: LLM Optimization (Weeks 7-8)

### Sprint 4.1: Context Optimizer

**Goals:**
- Implement information gain calculator
- Build greedy context selection
- Support task-specific strategies

**Deliverables:**
- `crates/pt-analytics/src/llm_context.rs`
- Context optimizer
- CLI: `pt-analytics context --focus auth::validate --task bug-fix`

**Success Criteria:**
- 3-4× better token efficiency vs file-based
- High relevance (>85%) user evaluation

### Sprint 4.2: Context Export

**Goals:**
- Format context for different LLM APIs
- Add metadata (cluster info, metrics)
- Streaming support for large contexts

**Deliverables:**
- Export formatters (Markdown, JSON)
- API integration helpers

**Success Criteria:**
- Easy integration with GPT-4, Claude
- Context fits in token limits

## Phase 5: Developer Tools (Weeks 9-10)

### Sprint 5.1: Change Impact Analyzer

**Goals:**
- Implement multi-dimensional impact analysis
- Risk scoring
- Beautiful terminal visualization

**Deliverables:**
- Impact analyzer
- CLI: `pt-analytics impact --entity auth::validate`

**Success Criteria:**
- Accurate blast radius calculation
- Actionable recommendations

### Sprint 5.2: Architecture Checker

**Goals:**
- Define architecture rules DSL
- Implement constraint checking
- Continuous validation

**Deliverables:**
- Architecture constraints engine
- CLI: `pt-analytics arch-check`

**Success Criteria:**
- Detect layer violations
- Zero false positives

### Sprint 5.3: Test Intelligence

**Goals:**
- Identify test gaps
- Calculate test effectiveness
- Prioritize test creation

**Deliverables:**
- Test intelligence analyzer
- CLI: `pt-analytics test-gaps`

**Success Criteria:**
- Find critical untested paths
- Prioritization matches manual analysis

## Phase 6: Visualization & Polish (Weeks 11-12)

### Sprint 6.1: Terminal Visualizations

**Goals:**
- ASCII art graph rendering
- Heat maps
- Sparklines
- Interactive exploration

**Deliverables:**
- `crates/pt-analytics/src/visualize.rs`
- Rich terminal output
- Color schemes

**Success Criteria:**
- Beautiful, readable output
- Fits in 80-column terminals

### Sprint 6.2: Dashboard

**Goals:**
- Real-time metrics dashboard
- Multiple views (overview, details)
- Navigation

**Deliverables:**
- TUI dashboard using `ratatui`
- CLI: `pt-analytics dashboard`

**Success Criteria:**
- Responsive UI
- Useful at-a-glance metrics

### Sprint 6.3: Documentation

**Goals:**
- User guide
- API documentation
- Examples

**Deliverables:**
- README with screenshots
- Tutorial
- API docs

**Success Criteria:**
- New users can get started in <5 minutes
- All features documented

## Milestones

**M1: Foundation Complete (End of Week 2)**
- Multi-level graph projection working
- Basic metrics computed
- CLI functional

**M2: Clustering Complete (End of Week 4)**
- ISGL0.5 clusters discovered
- Stored in CozoDB
- Automatic naming works

**M3: Flow Analysis Complete (End of Week 6)**
- Control, data, temporal flow analyzed
- Vulnerabilities detected
- Hidden dependencies found

**M4: LLM Integration Complete (End of Week 8)**
- Context optimizer working
- 4× efficiency demonstrated
- Export to LLM APIs

**M5: Developer Tools Complete (End of Week 10)**
- Impact analyzer
- Architecture checker
- Test intelligence

**M6: Production Ready (End of Week 12)**
- Beautiful visualizations
- Dashboard
- Full documentation
- Ready for release

---

# Part 4: Mathematical Foundations

## Graph Theory Fundamentals

### Basic Definitions

**Graph G = (V, E)**
- V: Set of vertices (functions, files, packages)
- E: Set of edges (dependencies, calls, data flow)

**Directed Graph:**
- Edges have direction: (u, v) means u → v

**Weighted Graph:**
- Each edge has weight w(u,v) representing strength

### Key Metrics

#### Degree Centrality
```
deg(v) = number of edges connected to v

In-degree:  incoming edges (who calls this?)
Out-degree: outgoing edges (who does this call?)
```

#### Betweenness Centrality
```
C_B(v) = Σ (σ_st(v) / σ_st)

Where:
  σ_st = number of shortest paths from s to t
  σ_st(v) = number of those paths passing through v

Interpretation: How often is v on the shortest path between other nodes?
High betweenness = bottleneck
```

**Algorithm:** Brandes' algorithm - O(VE)

#### PageRank
```
PR(v) = (1-d)/N + d × Σ(PR(u) / outdeg(u))

Where:
  d = damping factor (typically 0.85)
  N = number of nodes
  sum over all u that link to v

Interpretation: Importance based on incoming links
Functions called by important functions are important
```

**Algorithm:** Power iteration - O(V + E) per iteration

#### Closeness Centrality
```
C_C(v) = 1 / Σ dist(v, u)

Where dist(v, u) = shortest path length

Interpretation: How close is v to all other nodes?
High closeness = easy to reach from anywhere
```

### Graph Algorithms

#### Shortest Path (Dijkstra's)
```
Time: O((V + E) log V) with priority queue

Use: Find call chains, impact radius
```

#### Strongly Connected Components (Tarjan's)
```
Time: O(V + E)

Use: Find circular dependencies
SCC = set of nodes where every pair is mutually reachable
```

#### Minimum Cut
```
Time: O(V²E) - Ford-Fulkerson

Use: Find minimal refactoring boundaries
Min-cut = minimum number of edges to remove to disconnect graph
```

## Clustering Theory

### Spectral Clustering Mathematics

**Goal:** Partition graph to minimize normalized cut.

**Normalized Cut:**
```
Ncut(A, B) = cut(A, B) / vol(A) + cut(A, B) / vol(B)

Where:
  cut(A, B) = Σ w(u,v) for u∈A, v∈B
  vol(A) = Σ deg(u) for u∈A
```

**Graph Laplacian:**
```
L = D - W

Where:
  D = diagonal degree matrix
  W = weighted adjacency matrix

Normalized Laplacian:
L_norm = I - D^(-1/2) W D^(-1/2)
```

**Spectral Theorem:**
```
The eigenvector corresponding to the second smallest eigenvalue
of L_norm (Fiedler vector) gives optimal 2-way partition.

For k-way partition:
  Use first k eigenvectors
  Embed nodes in k-dimensional space
  Run k-means
```

**Eigengap Heuristic:**
```
Optimal k = argmax_i (λ_{i+1} - λ_i)

Large eigengap indicates natural number of clusters
```

### Modularity

**Modularity Q:**
```
Q = (1/2m) Σ [A_ij - k_i k_j / 2m] δ(c_i, c_j)

Where:
  m = total edge weight
  A_ij = adjacency matrix
  k_i = degree of node i
  δ(c_i, c_j) = 1 if i, j in same cluster, else 0

Interpretation:
  Q = (edges within clusters) - (expected edges)
  Range: -1 to 1
  Q > 0.3: significant community structure
  Q > 0.7: strong community structure
```

### Information Theory

**Shannon Entropy:**
```
H(X) = -Σ p(x) log p(x)

Interpretation: Uncertainty in random variable X
Higher entropy = more uncertainty
```

**Mutual Information:**
```
I(X; Y) = H(X) + H(Y) - H(X,Y)

Interpretation: How much knowing Y reduces uncertainty about X
Used to measure coupling between clusters
```

**Conditional Entropy:**
```
H(X|Y) = H(X,Y) - H(Y)

Interpretation: Remaining uncertainty in X after observing Y
```

**Information Gain:**
```
IG(X, Y) = H(X) - H(X|Y)

Used for context optimization:
  How much does adding cluster Y reduce uncertainty about task X?
```

### Minimum Description Length (MDL)

**Principle:** Best model minimizes total description length

```
MDL(model, data) = L(model) + L(data | model)

For clustering:
  L(model) = cost to describe cluster structure
  L(data|model) = cost to describe cross-cluster edges

Optimal clustering = argmin MDL
```

## Data Flow Analysis

### Taint Analysis

**Lattice:**
```
        TAINTED
           |
         UNKNOWN
           |
          SAFE

Rules:
  TAINTED ⊔ SAFE = TAINTED
  UNKNOWN ⊔ x = UNKNOWN
```

**Transfer Functions:**
```
f_sanitize(TAINTED) = SAFE
f_concat(TAINTED, SAFE) = TAINTED
f_copy(x) = x
```

**Fixed-Point Iteration:**
```
Start: taint[source] = TAINTED, all others = SAFE
Repeat:
  For each statement:
    taint_out = transfer_function(taint_in)
Until convergence (no changes)
```

### Use-Def Chains

**Definition:**
```
def(v, s): variable v defined at statement s
use(v, s): variable v used at statement s

use-def chain: links each use to all possible definitions
```

**Algorithm:** Reaching definitions
```
Time: O(n²) where n = number of statements
```

## Complexity Metrics

### Cyclomatic Complexity

**Definition:**
```
M = E - N + 2P

Where:
  E = number of edges in control flow graph
  N = number of nodes
  P = number of connected components (usually 1)

Alternative:
M = number of decision points + 1
```

**Interpretation:**
```
M ≤ 10:  Simple, low risk
10 < M ≤ 20: Moderate complexity
20 < M ≤ 50: High complexity, test carefully
M > 50: Very high risk, refactor
```

### Cognitive Complexity

**Principle:** Measure how hard code is for humans to understand

**Increments:**
```
+1 for: if, for, while, case, catch, &&, ||, ?:
+1 for each nesting level
+0 for else if (better than nested if-else)
```

### Coupling Metrics

**Afferent Coupling (Ca):**
```
Ca = number of classes that depend on this class
High Ca = stable (many depend on it)
```

**Efferent Coupling (Ce):**
```
Ce = number of classes this class depends on
High Ce = unstable (depends on many)
```

**Instability:**
```
I = Ce / (Ca + Ce)

Range: 0 to 1
I = 0: maximally stable (pure library)
I = 1: maximally unstable (pure client)
```

**Abstractness:**
```
A = abstract_classes / total_classes

Range: 0 to 1
```

**Distance from Main Sequence:**
```
D = |A + I - 1|

Ideal: D ≈ 0 (on main sequence)
D > 0.5: problematic
```

## Statistical Methods

### Correlation

**Pearson Correlation:**
```
r = Σ((x_i - x̄)(y_i - ȳ)) / sqrt(Σ(x_i - x̄)² Σ(y_i - ȳ)²)

Range: -1 to 1
|r| > 0.7: strong correlation
```

**For temporal coupling:**
```
x_i = 1 if entity X changed in commit i, else 0
y_i = 1 if entity Y changed in commit i, else 0

High r → entities change together
```

### Time Series

**Moving Average:**
```
MA(t) = (1/k) Σ_{i=0}^{k-1} value(t-i)

Smooths out short-term fluctuations
```

**Exponential Smoothing:**
```
S_t = α × value_t + (1-α) × S_{t-1}

α = smoothing factor (0 < α < 1)
Recent values weighted more heavily
```

## Optimization

### Greedy Algorithms

**Context Selection:**
```
Input: Budget B, clusters C with costs and gains
Output: Subset S maximizing total gain

Algorithm:
  S = {primary_cluster}
  remaining = B - cost(primary)

  while remaining > 0:
    best = argmax_{c ∈ C\S} gain(c) / cost(c)
    if cost(best) ≤ remaining:
      S = S ∪ {best}
      remaining -= cost(best)
    else:
      break

  return S
```

**Approximation:** For submodular gain functions, greedy achieves (1 - 1/e) ≈ 63% of optimal

### Dynamic Programming

**Optimal Clustering:**
```
For some problems (e.g., 1D clustering), DP gives optimal solution

dp[i][k] = min cost to cluster first i items into k clusters

Recurrence:
dp[i][k] = min_{j<i} (dp[j][k-1] + cost(j+1, i))

Time: O(n² k)
```

---

# Appendix: References and Further Reading

## Graph Theory

1. **Introduction to Graph Theory** - Douglas B. West
   - Comprehensive introduction to graph algorithms

2. **Networks** - Mark Newman
   - Modern treatment of network analysis
   - Chapter 7: Measures and Metrics
   - Chapter 11: Community Detection

3. **Spectral Graph Theory** - Fan Chung
   - Deep dive into spectral methods
   - Laplacian eigenvalues and eigenvectors

## Clustering

4. **A Tutorial on Spectral Clustering** - Ulrike von Luxburg
   - Excellent mathematical introduction
   - Proves optimality guarantees

5. **Finding Community Structure in Very Large Networks** - Clauset, Newman, Moore
   - Original Louvain algorithm paper
   - Fast modularity optimization

6. **The Map Equation** - Rosvall & Bergstrom
   - InfoMap algorithm
   - Information-theoretic clustering

## Software Metrics

7. **Object-Oriented Metrics in Practice** - Michele Lanza & Radu Marinescu
   - Software quality metrics
   - Refactoring patterns

8. **Evaluating Software Architectures** - Clements, Kazman, Klein
   - Architecture evaluation methods
   - Quality attributes

9. **Code Complete** - Steve McConnell
   - Chapter on complexity metrics
   - Practical guidelines

## Information Theory

10. **Elements of Information Theory** - Cover & Thomas
    - Shannon entropy, mutual information
    - Chapter 2: Entropy and Mutual Information

11. **Model Selection and Multimodel Inference** - Burnham & Anderson
    - MDL principle
    - Model comparison

## Data Flow Analysis

12. **Compilers: Principles, Techniques, and Tools** - Aho, Sethi, Ullman (Dragon Book)
    - Chapter 9: Data Flow Analysis
    - Reaching definitions, use-def chains

13. **Secure Programming with Static Analysis** - Chess & West
    - Taint analysis
    - Security vulnerabilities

## Applied Papers

14. **Software Clustering: A Comprehensive Study** - Anquetil & Lethbridge (1999)
    - Survey of software clustering techniques

15. **Temporal Coupling in Software Engineering** - D'Ambros et al. (2009)
    - Change coupling patterns
    - Historical data mining

16. **PageRank Beyond the Web** - Gleich (2015)
    - Applications of PageRank to other domains

17. **Using Machine Learning to Improve Suggestions in IDEs** - Bruch et al. (2009)
    - Context-aware code completion

## Tools & Libraries

18. **CozoDB Documentation** - https://github.com/cozodb/cozo
    - Datalog syntax
    - Graph algorithms

19. **NetworkX Documentation** - Python graph library
    - Reference implementations of algorithms

20. **Ratatui** - https://ratatui.rs
    - Terminal UI framework for Rust

---

## Glossary

**Affinity Matrix**: Weighted graph representation where w[i,j] = similarity/coupling between nodes i and j

**Betweenness Centrality**: Measure of how often a node lies on shortest paths between other nodes

**Blast Radius**: Set of entities affected by changing a given entity

**Cohesion**: How strongly related elements within a module are (high is good)

**Coupling**: How strongly a module depends on other modules (low is good)

**Control Flow**: Sequence of function calls through a program

**Data Flow**: Movement of data through transformations in a program

**Eigengap**: Difference between consecutive eigenvalues; large gaps indicate natural cluster boundaries

**ISGL0.5**: Automatically discovered semantic clusters between file and function granularity

**Laplacian Matrix**: Graph matrix L = D - W encoding structural properties

**Modularity**: Quality metric for graph partitions; measures excess internal edges vs expected

**Normalized Cut**: Graph partition metric balancing cut size against cluster volumes

**PageRank**: Importance measure based on incoming connections from important nodes

**Spectral Clustering**: Clustering via eigendecomposition of graph Laplacian

**Taint Analysis**: Tracking potentially dangerous data from sources to sinks

**Temporal Coupling**: Pattern where code entities frequently change together

---

**End of Document**

This framework provides a comprehensive foundation for building advanced analytics capabilities in Parseltongue. The multi-level approach combined with automatic clustering and flow-aware analysis will enable both better developer tools and more effective LLM integration.

For questions or contributions, please refer to the Parseltongue repository.