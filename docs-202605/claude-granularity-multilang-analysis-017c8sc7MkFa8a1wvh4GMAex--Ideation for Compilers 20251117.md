``` mermaid

flowchart TD
    A[Parseltongue Dependency Graph Enhancement] --> B{Core Philosophy}
    A --> C{Implementation Strategy}
    A --> D{Architectural Foundation}
    
    B --> B1[Parse Once, Query Forever]
    B --> B2[No jq Anti-Pattern]
    B --> B3[Direct CozoDB Queries Only]
    B --> B4[JSON for LLMs, Not Humans]
    
    C --> C1[Minimal Viable Path]
    C1 --> C1a[TDD-Validate JSON Queryability]
    C1 --> C1b[80% Coverage Goal]
    C1 --> C1c[Defer Visualization]
    C1 --> C1d[Focus on Agent Usability]
    
    D --> D1[PDG/SDG Unification Model]
    D1 --> D1a[Control Dependencies]
    D1 --> D1b[Data Dependencies]
    D1 --> D1c[Semantic Directionality]
    D1 --> D1d[Natural Clustering]
    
    %% Current State
    E[Current Parseltongue] --> E1[Typed Edges<br/>Calls, Uses, Implements]
    E --> E2[Basic Clustering LPA]
    E --> E3[JSON Exports]
    E --> E4[CozoDB Backend]
    
    %% Enhancement Pipeline
    F[Proposed Enhancements] --> F1[Extended Edge Types]
    F1 --> F1a[Extends, Contains]
    F1 --> F1b[ControlDependsOn]
    F1 --> F1c[DataFlowsTo]
    
    F --> F2[Advanced Clustering]
    F2 --> F2a[Connected Components]
    F2 --> F2b[Hierarchical Analysis]
    
    F --> F3[Directional Semantics]
    F3 --> F3a[Upward: Implements/Extends]
    F3 --> F3b[Horizontal: Calls/Uses]
    F3 --> F3c[Downward: Contains/Instantiates]
    
    %% Query Patterns
    G[Agent Query Patterns] --> G1[Blast Radius Analysis]
    G --> G2[Call Chain Tracing]
    G --> G3[Dependency Impact]
    G --> G4[Code Slicing]
    
    %% Relationships
    B1 -.-> C1
    D1 -.-> F
    E -.-> F
    C1 -.-> G
    
    classDef philosophy fill:#e1f5fe
    classDef implementation fill:#f3e5f5
    classDef architecture fill:#e8f5e8
    classDef current fill:#fff3e0
    classDef future fill:#fce4ec
    classDef queries fill:#f1f8e9
    
    class B,B1,B2,B3,B4 philosophy
    class C,C1,C1a,C1b,C1c,C1d implementation
    class D,D1,D1a,D1b,D1c,D1d architecture
    class E,E1,E2,E3,E4 current
    class F,F1,F1a,F1b,F1c,F2,F2a,F2b,F3,F3a,F3b,F3c future
    class G,G1,G2,G3,G4 queries

```
