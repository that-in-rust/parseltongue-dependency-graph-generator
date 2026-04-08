# Parseltongue Simulation 01: Tree Sitter Graph Scope

- only tree-sitter dependency graph - no rustc
- only forward calls backward calls and export a sub-graph of public interface dependency graph
- language: rust c cpp java js ts python golang ruby
- Tauri app for desktop for the user to manage triggering of graph generation, reindexing and viewing the graph
- create a dataset of all OSS codebases for LLD documentation which can be used to train the models
- Parseltongue WILL NOT store code
- tree-sitter parses
    - a folder
        - entity is folder
    - a file
        - if extension is not in the list of supported languages with entity type as <wip-name> and insert it as filepath-filename in the persisted codebase graph
        - if extension is in the list of supported languages
            - entity is a file
            - total LOC are measured
            - file is broken down into mutually exclusive cumulatively exhaustive entities WHICH ARE just 1 level below the file ONLY - NO nested entities are allowed
                - <wip list of normal entities found in codebase> like functions, classes, structs, enums, interfaces
                    - includes import statements and use statements
                - entity type for tests is separated out since we may want to exclude them from blast radius since they can affect our results
                - entity type for comments is separated out since we may want to exclude them from blast radius since they can affect our results
    - total wc of each file should match the total wc of the entities in the file
    - total wc of a folder should match the total wc of the files in the folder
    - if failures happen in parsing they should be logged and summarized by the accuracy of the above system
    - entity will be ISGL1 - public interfaces with LOC range as start_line:end_line 
- edges extraction
    - every edge will have a direction
        - calling
            - forward calls
            - backward calls
        - folder to folder relationships
            - if both have common parents it is shared-parent as an edge-type
                - <wip what direction should this edge go for siblings>
            - if one is a parent of the other it is parent-child as an edge-type
                - parent is seen as center of cluster and arrows going out towards children
        - folder to file relationships
            - folder contains a file
                - folder is seen as center of cluster and arrows going out towards files
            - folder and file are siblings
                - <wip what direction should this edge go for siblings>
        - file to file relationships
            - if files are siblings it is an edge-type
                - <wip what direction should this edge go for siblings>
        - file to entity relationships
            - file contains an entity
                - file is seen as center of cluster and arrows going out towards entities
        - entity to entity relationships
            - entity calls another entity
                - normal direction figured out <wip list of what direction what is>
            - entity is in the same file as another entity
                - shared context as an edge-type
                    - direction can be basis LOC - earler one is center of cluster and arrows going out towards later one
                    - <wip what direction if both on same LOC - 2 entities can it even happen>

---

# Parseltongue Simulation 02: Walk Graph Runtime

- repo codename: `Walk-Graph-Runtime`
- selected storage/runtime architecture: `Immutable Dual CSR/CSC Snapshot (mmap)`
- workload target:
    - embedded
    - local
    - code-scale graph storage
    - forward and backward traversal
    - BFS / DFS / reachability / callers / callees / blast radius

## Why this was chosen

- this choice comes directly from the comparison table in [walk-runtime-options-explainer.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/strategic-research/walk-runtime-options-explainer.md)
- in that comparison, `Immutable Dual CSR/CSC Snapshot (mmap)` scored highest overall for the specific job of a **Walk Graph Runtime**
- it is the best match when the main questions are:
    - who points to this node?
    - what does this node point to?
    - what can I reach from here?
    - what breaks if I start from here?

## Why it wins against the alternatives

- versus `CSR Base + Tiny Mutable Overlay`
    - the overlay design is a good second step, but adds complexity too early
    - rebuild-first is acceptable for local code graphs, so simplicity wins right now

- versus `Serialized petgraph Snapshot`
    - good for prototyping, weaker as a product wedge
    - slower cold start and less clear differentiation

- versus `SQLite / LMDB Adjacency Store`
    - better mutation fit, worse walk-first feel
    - this moves the architecture toward a database even though the core workload is graph walking

- versus `Packed / Dynamic CSR`
    - technically interesting, strategically premature
    - too much cleverness before real user pressure for in-place mutation

- versus `Edge Table + Covering Indexes`
    - this behaves more like a database than a map
    - the main job here is traversal, so a traversal-native shape is better

- versus `Matrix-First Sparse Snapshot`
    - strong idea for a future `Rank Graph Runtime`
    - wrong lead architecture for the walk-first problem

## Current architecture thesis

- immutable snapshot files
- page-friendly sectioned binary layout
- forward adjacency by dimension
- backward adjacency by dimension
- `mmap` open path
- zero-copy or near-zero-copy reads
- graph walking is the center of gravity

## What this runtime is for

- forward and backward neighborhood lookup
- call graph walking
- bounded BFS
- reverse traversal
- reachability
- blast radius
- dead code from roots
- shortest-path style local path queries

## What this runtime is not for

- semantic graph delta across snapshots
- architecture simulation across worlds
- public-interface policy
- tree-sitter parsing
- test/comment semantics
- query-language style graph database behavior

## Boundary with higher layers

- `Walk-Graph-Runtime` stores and walks one frozen graph world
- Parseltongue or Parcel10 can sit above it to:
    - build the graph from source code
    - compare multiple snapshots
    - compute semantic deltas
    - explain architectural consequences
