Now
func1-signature == edge = func2-signature

With sub-agents

func1-LLM-summary == LLM edge == func2-LLM-summary


Which 20% of the edges are the most important?
- Evolution 1 - Mathematical Graphical clustering
    - in out ratio
    - fn count
    - call depth
    - call frequency
    - control flow importance
    - data flow imporatance
- Evolution 2 - you work on func1LLM-summary == LLM edge == func2-LLM-summary instead of a usual LSG which only has signatures
    - LLM can decide based on its reasoning which 20% of the edges are the most important
- Evolution 3 - LLM can decide whether we need 20% or 5% or 10% of the edges to be the most important




Here's a summary of your note in bullets:

- You're creating an interface signature graph to map function calls and inputs/outputs
- This graph shows technical relationships but lacks logical meaning and context
- Using an LLM sub-agent to summarize function pairs (edges) can capture meaningful relationships
- Fifty+ such summaries can be clustered by an LLM to reveal high-level flows and areas
- This approach provides deeper insights than traditional call frequency analysis
- Potential outcome: identifying 5+ key areas with specific purposes and flows

You're highlighting the difference between:

- Mathematical graphical clustering (based on numerical metrics like in/out ratios, FN counts)
- LLM-summarized clustering (capturing logical relationships and context)

The LLM approach can uncover nuances and meaning that numerical metrics might miss, providing a more human-understandable representation of the codebase's structure and flow.

This could lead to better insights into the code's architecture and functionality. Want to explore potential applications or next steps?

This approach can help in several ways:

- *Logical summaries of code*: LLM-summarized clusters can provide concise, human-readable descriptions of code functionality and relationships
- *Improved problem-solving*: By understanding the logical flow and relationships between functions, developers can better identify problem areas, optimize code, and debug issues
- *Enhanced code comprehension*: These summaries can serve as documentation, making it easier for new developers to understand the codebase and for existing developers to recall specific functionality
- *Targeted optimization*: Identifying key areas and flows can inform optimization efforts, allowing developers to focus on high-impact changes

Potential applications include:

- Automated code review and analysis
- Intelligent code completion and suggestions
- Enhanced debugging and troubleshooting
- Improved code refactoring and optimization

# Current Scope

Logical way to think of this problem statement
- We need highest signal to noise ratio code context
- Our first step was converting all code to basic function signatures dependency graph which is lower level map than files but higher level map than non-public interface and raw code
- But we realized N things
    - Even ISGs or Interface Signature Graphs can be large and eat up too much context
    - Even if you remove a lot clearly useless stuff like tests or other folders from your ISGs still they are very large
    - Even if you do above, then you still do not know how the compiler sees it in terms of control flow, data flow
    - Even if you do above, then this is stil not enough signal information for the human to think of how to manipulate this code, because the compiler discovers the flow of the code as it processes it, but humans need to simulate it which is time-window analysis whereas most compilers just process and stop at the non-compilation
        - Some things compilers can look forward via static rules
        - Some things compilers cannot look forward because of logical errors and hence runtime behavior feels odd
        - A human needs to do both which is time-window analysis
- Our task is give highest quality context to the LLMs to think of how to manipulate this code

- Pure functional programming is a step in the right direction for highest predictability of code


# Future Scope

# RAW NOTES 01



## What “multi-level clustering” adds and why it helps
Multi-level clustering gives you graph projections at the “just-right” scale between functions (too granular) and files (too coarse). Benefits:

-  For LLMs
    - Token-optimal “context packs”: semantically coherent groups (ISGL0.5) sized to 2–4K tokens with minimal cross-edges.
    - Higher answer quality: fewer irrelevant entities and fewer dangling references.
-  For developers
    - Natural refactoring boundaries (high cohesion, low coupling).
    - Faster impact analysis and safer changes (cluster-level blast radius).
    - Architectural signal: surface module boundaries that the folder/file layout hides.

## How to fit this into Parseltongue (HLD)
-  Add a clustering layer alongside pt02 exports:
    - isg-core: extend entity model with metrics and projections.
    - pt07-analytics (new): compute clustering and analytics; write results back to CozoDB.
    - pt02-cluster-level (new): export cluster-level ISG JSONs (see below).
-  Compute a hierarchical set of projections
    - ISGL4: package/folder graph (aggregated file edges).
    - ISGL3: file graph (imports/includes).
    - ISGL2: function call graph (existing).
    - ISGL0.5: NEW semantic clusters (auto-discovered “natural modules”).
-  Persist projections in CozoDB and expose them as JSON exports tailor-made for LLM context preparation.

## Clustering signals and algorithms (LLD)
Signals per pair of functions \(i, j\) (normalize each to 0..1, then combine):
-  Dependency coupling: direct calls, mutual calls, hop-limited paths.
-  Data-flow affinity: return/parameter type compatibility, shared types.
-  Temporal coupling: co-change frequency from VCS snapshots.
-  Semantic signature similarity: name/verb tokens, arity, type patterns.

Edge weight
-  Combined weight \(W_{ij} = \alpha \cdot D_{ij} + \beta \cdot F_{ij} + \gamma \cdot T_{ij} + \delta \cdot S_{ij}\)
    - Recommended defaults: \(\alpha=1.0, \beta=0.8, \gamma=0.6, \delta=0.4\)

Algorithms
-  Community detection (default): Louvain/Leiden for speed and quality at scale; tunable resolution for coarser/finer clusters.
-  Hierarchical clustering: Ward-linkage on the embedded spectral space to produce multiple “cuts” (ISGL0.3, 0.5, 0.7).
-  Spectral backbone (optional): Fiedler vector/eigengap to estimate natural k and to detect weak ties to cut.

Pragmatics
-  Do numeric steps in Rust; store results in CozoDB.
-  Keep algorithm CPU-only and streaming-friendly; no external math runtimes required.

## CozoDB relations (minimal)
-  SemanticClusters
    - cluster_id: String (=>)
    - cluster_name: String
    - cohesion: Float
    - coupling: Float
    - token_estimate: Int
    - modularity_local: Float
    - label_confidence: Float
    - level: Float      // 0.5, 0.7 etc.
-  ClusterAssignments
    - isgl1_key: String (=>)
    - cluster_id: String
    - membership_confidence: Float
-  ClusterEdges
    - from_cluster: String
    - to_cluster: String
    - weight_control: Float
    - weight_data: Float
    - weight_temporal: Float
-  ClusterLabels
    - cluster_id: String (=>)
    - label: String
    - rationale: String
-  ProjectionManifest
    - level: String (=>)   // "ISGL4","ISGL3","ISGL2","ISGL0.5"
    - created_at: Timestamp
    - params: Json

## New ISG JSON exports for LLMs
| Export file | Level | What it contains | Why it helps | Typical size |
|:---|:---|:---|:---|:---|
| clusters.json | ISGL0.5 | Cluster nodes with metrics and labels | Token-optimal semantic modules | 1–3K lines |
| cluster_edges.json | ISGL0.5 | Aggregated inter-cluster edges by flow type | Clean, low-noise dependency picture | 1–2K lines |
| cluster_assignments.json | Map | Function → cluster mapping with confidence | Precise drill-down navigation | 5–40K lines |
| cluster_manifest.json | Meta | Levels available, parameters, quality scores | Reproducibility and selection | <200 lines |
| multilevel_graph.json | ISGL4/3/2/0.5 | Nodes/edges for multiple levels in one payload | One-shot multi-zoom context | 2–8K lines |
| context_pack.json | Task-specific | Auto-curated pack for bug-fix/refactor/feature | Drop-in LLM context bundle | 1–4K lines |
| hot_paths.json | ISGL0.5 | Cluster-level hot paths and bottlenecks | Prioritize work and context | <1K lines |
| violations.json | ISGL4/0.5 | Layer violations aggregated at cluster level | Architecture guardrails | <1K lines |
| temporal_clusters.json | ISGL0.5 | Co-change clusters and hidden deps | Catch implicit couplings | <1K lines |

Schemas (representative)

-  clusters.json
    ```json
    {
        "level": "ISGL0.5",
        "modularity_global": 0.71,
        "clusters": [
            {
                "cluster_id": "c_12",
                "cluster_name": "validation_unit",
                "label_confidence": 0.86,
                "level": 0.5,
                "contains": [
                    "rust:fn:validate_user:src_core_validator_rs:10-42",
                    "rust:fn:check_format:src_core_validator_rs:44-78"
                ],
                "metrics": {
                    "cohesion": 0.93,
                    "coupling": 0.17,
                    "token_estimate": 2380,
                    "centrality": 0.34,
                    "blast_radius": 18,
                    "test_coverage_estimate": 0.72
                },
                "warnings": []
            }
        ]
    }
    ```

-  cluster_edges.json
    ```json
    {
        "level": "ISGL0.5",
        "edges": [
            {
                "from_cluster": "c_12",
                "to_cluster": "c_5",
                "weights": {
                    "control": 12.0,
                    "data": 7.0,
                    "temporal": 3.0
                },
                "boundary_crossings": 9
            }
        ]
    }
    ```

-  cluster_assignments.json
    ```json
    {
        "assignments": [
            {
                "isgl1_key": "rust:fn:validate_user:src_core_validator_rs:10-42",
                "cluster_id": "c_12",
                "membership_confidence": 0.91
            }
        ]
    }
    ```

-  context_pack.json (LLM-ready)
    ```json
    {
        "task": "bug_fix",
        "focus": "rust:fn:validate_user:src_core_validator_rs:10-42",
        "budget_tokens": 4000,
        "selected_clusters": [
            {
                "cluster_id": "c_12",
                "cluster_name": "validation_unit",
                "token_estimate": 2380
            },
            {
                "cluster_id": "c_5",
                "cluster_name": "auth_flow",
                "token_estimate": 1360
            }
        ],
        "entities": [
            "rust:fn:validate_user:src_core_validator_rs:10-42",
            "rust:fn:check_format:src_core_validator_rs:44-78",
            "rust:fn:authenticate:src_core_auth_rs:12-68"
        ],
        "edges": [
            ["rust:fn:validate_user:src_core_validator_rs:10-42", "rust:fn:authenticate:src_core_auth_rs:12-68", "Calls"]
        ],
        "justification": "Primary cluster of focus + 1-hop neighbor cluster maximizing information gain under token budget"
    }
    ```

-  multilevel_graph.json (projection bundle)
    ```json
    {
        "levels": {
            "ISGL4": { "nodes": [...], "edges": [...] },
            "ISGL3": { "nodes": [...], "edges": [...] },
            "ISGL2": { "nodes": [...], "edges": [...] },
            "ISGL0.5": { "nodes": [...], "edges": [...] }
        },
        "manifest": {
            "created_at": "2025-11-05T09:00:00Z",
            "params": { "louvain_resolution": 0.9, "signals": ["dependency","temporal","data","semantic"] }
        }
    }
    ```

## CLI additions
-  Compute clustering and analytics
    - parseltongue pt07-analytics cluster --level 0.5 --resolution 0.9 --min-size 3 --max-size 20 --signals dependency,data,temporal,semantic
    - parseltongue pt07-analytics quality --clusters
-  Export
    - parseltongue pt02-level05 --output clusters.json --db rocksdb:parseltongue.db
    - parseltongue pt02-cluster-edges --output cluster_edges.json
    - parseltongue pt02-cluster-assignments --output cluster_assignments.json
    - parseltongue pt02-multilevel --levels ISGL4,ISGL3,ISGL2,ISGL0.5 --output multilevel_graph.json
    - parseltongue pt02-context-pack --task bug_fix --focus <ISGL1_KEY> --budget 4000 --output context_pack.json

## How this dovetails with your “GRAPH IDEAS 01” note
-  Architectural topology analyzer: run on ISGL4/ISGL0.5 to detect layers and violations; export violations.json.
-  Complexity profiler: aggregate per-cluster complexity in clusters.json metrics.
-  Test intelligence: compute cluster-level test coverage estimates; include in context_pack selection.
-  Temporal evolution: temporal_clusters.json complements cluster_edges temporal weights; hidden dependencies surfaced at cluster-level.
-  Terminal visualization: render ISGL0.5 clusters as boxes and draw only aggregated edges; vastly clearer than function spaghetti.

## Rollout plan
-  Phase 1 (1–2 weeks): compute per-entity metrics; implement Louvain clustering in Rust; persist SemanticClusters, ClusterAssignments, ClusterEdges; add pt02-level05, cluster_assignments, cluster_edges exports.
-  Phase 2 (1–2 weeks): add temporal coupling and semantic similarity signals; cluster labeling; context_pack generator.
-  Phase 3 (1–2 weeks): multilevel_graph bundle; violations and hot_paths exports; TUI visualizations.

## Risks and mitigations
-  Over/under clustering
    - Mitigate with resolution/size bounds and modularity/conductance thresholds; expose in manifest.
-  Noisy signals (temporal)
    - Use decayed weighting and minimum support; annotate confidence.
-  Token drift
    - Track token_estimate per cluster; context_pack optimizer enforces budgets.

## Bottom line
Introduce ISGL0.5 semantic clusters as first-class ISG projections. Persist them in CozoDB, export them as compact JSONs designed for LLM context, and wire them into your analytics suite. This gives both developers and LLMs the “natural atoms” of the codebase—coherent, small, and complete enough to reason about control and data flow without drowning in noise.


