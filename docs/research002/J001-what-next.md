# Origin Branches

Source: `git ls-remote --heads origin` and fetched `origin/*` refs on 2026-07-05.

| Branch | Last committed date | Latest commit |
| --- | --- | --- |
| main | 2026-07-05 | 410c3d29a |
| v200-prd | 2026-05-30 | 9b863b548 |
| v301 | 2026-04-16 | 157139939 |
| codex/v200 | 2026-03-21 | 1fc9e31ce |
| vorflux/parseltongue-v300-build | 2026-03-19 | f08a75c28 |
| v195-prep-for-v200 | 2026-03-07 | 678ccf24b |
| vorflux/ai-native-skills | 2026-03-07 | bcc8d73a8 |
| research/compiler-endpoints-v216 | 2026-03-04 | a7230d53e |
| v173 | 2026-02-21 | 64e5f366d |
| windows-mem-backup | 2026-02-12 | 73352418a |
| v160 | 2026-02-11 | 4d5c002af |
| v151 | 2026-02-07 | b415e1534 |
| v148-language-check-20260203.md | 2026-02-07 | 38520dc07 |
| feature/analysis-endpoints-v1.5 | 2026-01-31 | d82221d3c |
| interview-docs | 2026-01-26 | 8b20b2f5f |
| apwbd20260122 | 2026-01-26 | fe8cdff8a |
| exp20260118 | 2026-01-21 | e7ddedb39 |
| research/visualization-improvements-20260110-1914 | 2026-01-17 | afdc06275 |
| CPP202512ErrorSolving | 2025-12-03 | 70236ad9f |
| v097Part1 | 2025-11-25 | fa45e23d3 |
| claude/rust-cozo-graph-compiler-011CUyY2ajL61iJPm7tXLhmz | 2025-11-21 | 1bcc3c672 |
| claude/granularity-multilang-analysis-017c8sc7MkFa8a1wvh4GMAex | 2025-11-20 | b599fc138 |
| claude/clone-all-relationships-017c8sc7MkFa8a1wvh4GMAex | 2025-11-19 | 5d7ed31c1 |
| demo20251115v1 | 2025-11-18 | ba02817aa |
| backup-mistakes-20251115 | 2025-11-15 | d1c51ea1c |
| ssr-prd-research | 2025-11-09 | 8bd2b8768 |
| claude/cpp-compiler-research-011CUvFgidQ7y6ak7RvmWR5Q | 2025-11-08 | 482d79fa7 |
| claude/parcel-tongue-git-explorer-011CUvFwvJJ235CWERq2bwS7 | 2025-11-08 | ac49aed9c |
| claude/rust-compiler-coso-db-research-011CUvFWUtt7bVBJmF4Rdhri | 2025-11-08 | fd7dde976 |
| claude/reference-codebase-indexing-011CUvCMZFUEf3mdREgVyhLR | 2025-11-08 | ef2086956 |

# Related GitHub Repositories for Parseltongue

Source: attached repo-research prompt, local `docs/` research context, and live GitHub metadata from `gh repo view` / `gh search repos` on 2026-07-05.

Notes:
- `Last push` is GitHub `pushedAt`; it is a freshness signal, not a quality claim.
- `Stars/Forks` is current GitHub metadata at capture time.
- `Score` is relevance to Parseltongue from 0-5, where 5 means direct overlap with code-intelligence, code-graph, or LLM-agent context workflows.
- `Shreyas-style usefulness` is a JTBD / PMF / workflow commentary lens inspired by Shreyas Doshi product thinking. It is not a quote from Shreyas Doshi.
- Clarity disambiguation: targeted GitHub searches for `Clarity` plus code assistant, code intelligence, code search, codebase, and LLM terms did not produce a meaningfully relevant repo. The only clear hit was a tiny unrelated coding-assignment repo, so it is excluded until the exact intended Clarity repo is known.

| Tier | Repo | Last push | Stars/Forks | License | Lang | Evidence signal | Interface | Storage/index | Score | Shreyas-style usefulness |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| T1 | [MrDawell/atrium](https://github.com/MrDawell/atrium) | 2026-06-28 | 2/0 | MIT | Rust | Tree-sitter code graph plus agent memory | gRPC plus daemon | SQLite facts | 5 | High PMF because it attacks the same job: stop agents wasting tokens while preserving verified project memory. |
| T1 | [Xattaus/claude-brain](https://github.com/Xattaus/claude-brain) | 2026-06-12 | 0/0 | MIT | JavaScript | Tree-sitter code graph exposed as MCP tools | MCP plus Claude Code | Persistent memory | 5 | Study as a direct agent-context packaging competitor: same buyer pain, different surface area. |
| T1 | [FaizaanAlFaisal/code-search](https://github.com/FaizaanAlFaisal/code-search) | 2026-06-08 | 0/0 | MIT | Python | Repo to tree-sitter graph plus semantic and exact search | CLI or library | Local index | 5 | Tiny but strategically sharp: validates the exact positioning of semantic code graph for AI agents. |
| T1 | [Nishant-Chaudhary5338/mcp-code-indexer](https://github.com/Nishant-Chaudhary5338/mcp-code-indexer) | 2026-07-03 | 1/0 | MIT | TypeScript | Queryable TS React code graph, blast radius, cycles, dead code | CLI HTTP WS MCP 3D viewer | Code graph index | 5 | Very high workflow overlap; useful for endpoint names, agent tools, and demo journey design. |
| T1 | [sourcebot-dev/sourcebot](https://github.com/sourcebot-dev/sourcebot) | 2026-07-02 | 3562/315 | OTHER | TypeScript | Self-hosted codebase understanding for humans and agents | Web UI plus search | Search index | 4 | Good PMF signal for multi-repo code intelligence as a shared team utility, not only a local CLI. |
| T1 | [BloopAI/bloop](https://github.com/BloopAI/bloop) | 2024-12-04 | 9506/598 | APACHE-2.0 | Rust | Fast Rust code search engine | Desktop or service | Search index | 4 | Archived but important prior art for code search UX and the limits of search without graph-native workflows. |
| T1 | [TabbyML/tabby](https://github.com/TabbyML/tabby) | 2026-06-30 | 33678/1759 | OTHER | Rust | Self-hosted AI coding assistant | Server IDE integrations | Model and code context services | 4 | Useful for deployment and trust model: teams may want private code assistance before graph sophistication. |
| T1 | [continuedev/continue](https://github.com/continuedev/continue) | 2026-07-03 | 34685/4953 | APACHE-2.0 | TypeScript | Open-source coding agent with context providers and MCP awareness | IDE extension | Configurable context providers | 4 | High adoption signal; Parseltongue should feel like a context provider Continue would naturally call. |
| T1 | [Aider-AI/aider](https://github.com/Aider-AI/aider) | 2026-05-22 | 47053/4697 | APACHE-2.0 | Python | Terminal AI pair programmer with repo map concepts | CLI | Git working tree plus repo map | 4 | Aider shows the core JTBD: compress repo understanding into an action loop developers actually use. |
| T1 | [openai/codex](https://github.com/openai/codex) | 2026-07-05 | 95539/14179 | APACHE-2.0 | Rust | Terminal coding agent | CLI | Workspace and tools | 4 | Useful benchmark for how a coding agent wants context, verification, and shell-native workflows. |
| T1 | [google-gemini/gemini-cli](https://github.com/google-gemini/gemini-cli) | 2026-07-05 | 105749/14206 | APACHE-2.0 | TypeScript | Open-source terminal AI agent | CLI | Workspace and tools | 4 | Study for onboarding and command ergonomics; huge adoption suggests low-friction terminal UX matters. |
| T1 | [cline/cline](https://github.com/cline/cline) | 2026-07-05 | 64283/6843 | APACHE-2.0 | TypeScript | Autonomous coding agent SDK, IDE extension, CLI | IDE SDK CLI | Workspace and tools | 4 | Strong buyer pull around agent autonomy; Parseltongue can become the evidence layer behind such agents. |
| T1 | [RooCodeInc/Roo-Code](https://github.com/RooCodeInc/Roo-Code) | 2026-05-15 | 24303/3353 | APACHE-2.0 | TypeScript | Multi-agent IDE coding assistant | IDE extension | Workspace and tools | 4 | Useful for multi-agent workflow patterns and how graph context could be routed by role. |
| T1 | [OpenHands/OpenHands](https://github.com/OpenHands/OpenHands) | 2026-07-05 | 79452/10122 | OTHER | Python | AI-driven development agent platform | Web app plus runtime | Workspace containers | 4 | Study for end-to-end task execution loops; graph context can reduce expensive exploratory actions. |
| T1 | [yamadashy/repomix](https://github.com/yamadashy/repomix) | 2026-07-05 | 26844/1415 | MIT | TypeScript | Packs repositories into AI-friendly files | CLI | Packed prompt artifacts | 4 | High PMF despite low structure: proves users pay attention to token packaging before deeper graphs. |
| T1 | [coderamp-labs/gitingest](https://github.com/coderamp-labs/gitingest) | 2026-07-02 | 15009/1119 | MIT | Python | Prompt-friendly extract of GitHub repos | Web plus CLI style flow | Packed text artifacts | 4 | Great baseline competitor: Parseltongue must clearly beat raw repo ingestion on selectivity and evidence. |
| T1 | [mufeedvh/code2prompt](https://github.com/mufeedvh/code2prompt) | 2026-06-29 | 7459/425 | MIT | Rust | Turns codebase into prompt with token counting | CLI | Packed prompt artifacts | 4 | Useful for token-budget UX; Parseltongue can convert from bulk packing to queryable retrieval. |
| T1 | [The-PR-Agent/pr-agent](https://github.com/The-PR-Agent/pr-agent) | 2026-07-05 | 11964/1602 | APACHE-2.0 | Python | AI PR reviewer | GitHub PR bot | PR diff context | 3 | Good Shreyas workflow anchor: code review is frequent, painful, and needs blast-radius context. |
| T2 | [tree-sitter/tree-sitter](https://github.com/tree-sitter/tree-sitter) | 2026-07-02 | 26118/2751 | MIT | Rust | Incremental parsing system | Library CLI | Parse trees | 5 | Foundational dependency; the key PMF unlock is robust multi-language extraction without compiler setup. |
| T2 | [ast-grep/ast-grep](https://github.com/ast-grep/ast-grep) | 2026-07-04 | 14924/405 | MIT | Rust | Structural search, lint, rewrite using AST ideas | CLI library | AST pattern engine | 5 | High transfer value for query language ergonomics and user-authored structural rules. |
| T2 | [semgrep/semgrep](https://github.com/semgrep/semgrep) | 2026-07-03 | 15769/988 | LGPL-2.1 | OCaml | Multi-language static analysis with source-like rules | CLI CI SaaS | Rule engine | 5 | PMF proof for pattern-based analysis; borrow rule authoring and coverage reporting, not just parsing. |
| T2 | [github/semantic](https://github.com/github/semantic) | 2025-04-01 | 9044/458 | UNKNOWN | Haskell | Parsing, analyzing, comparing source across languages | Library | AST analysis | 4 | Archived prior art for language abstraction and semantic diff primitives. |
| T2 | [github/codeql](https://github.com/github/codeql) | 2026-07-04 | 9794/2011 | MIT | CodeQL | Code scanning queries and security analysis | CLI GitHub code scanning | CodeQL database | 5 | Gold standard for queryable code facts; study diagnostics, database creation, and query ecosystem. |
| T2 | [github/stack-graphs](https://github.com/github/stack-graphs) | 2025-09-09 | 875/166 | APACHE-2.0 | Rust | Rust implementation of stack graphs | Library | Scope and name graph | 4 | Important for resolving names beyond naive tree-sitter captures. |
| T2 | [scip-code/scip](https://github.com/scip-code/scip) | 2026-07-02 | 677/62 | APACHE-2.0 | Go | Code Intelligence Protocol | Protocol plus tools | SCIP index | 5 | Direct export/import target; validates stable symbols and precise navigation as a product surface. |
| T2 | [sourcegraph/scip-typescript](https://github.com/sourcegraph/scip-typescript) | 2026-07-03 | 102/36 | APACHE-2.0 | TypeScript | SCIP indexer for TS and JS | Indexer | SCIP index | 4 | Useful model for language-specific indexers feeding a common protocol. |
| T2 | [sourcegraph/scip-python](https://github.com/sourcegraph/scip-python) | 2026-07-03 | 93/52 | OTHER | Python | SCIP indexer for Python | Indexer | SCIP index | 4 | Study dynamic-language symbol representation and fallback behavior. |
| T2 | [scip-code/scip-java](https://github.com/scip-code/scip-java) | 2026-07-02 | 126/46 | APACHE-2.0 | Java | SCIP generator for Java | Indexer | SCIP index | 4 | Useful for JVM language precision and package/class identity schemes. |
| T2 | [sourcegraph/scip-ruby](https://github.com/sourcegraph/scip-ruby) | 2026-07-03 | 21/4 | APACHE-2.0 | Ruby | SCIP indexer for Ruby powered by Sorbet | Indexer | SCIP index | 3 | Useful precedent for dynamic language support through external type systems. |
| T2 | [sourcegraph/scip-clang](https://github.com/sourcegraph/scip-clang) | 2026-07-04 | 88/14 | APACHE-2.0 | C++ | SCIP indexer for C and C++ | Indexer | SCIP index | 4 | Relevant to C CPP pain in docs; learn compile-db and header strategies. |
| T2 | [scip-code/scip-rust](https://github.com/scip-code/scip-rust) | 2026-07-02 | 10/7 | APACHE-2.0 | Nix | SCIP support for Rust | Indexer | SCIP index | 3 | Small but relevant for Rust symbol identity and possible export compatibility. |
| T2 | [kythe/kythe](https://github.com/kythe/kythe) | 2026-06-23 | 2139/272 | APACHE-2.0 | Go | Language-agnostic ecosystem for code tools | Indexers plus graph services | Kythe graph | 5 | Major prior art for stable code graph schemas and cross-language facts. |
| T2 | [facebookincubator/Glean](https://github.com/facebookincubator/Glean) | 2026-07-04 | 1359/88 | OTHER | Hack | Collecting and deriving facts about source code | Fact DB | Glean DB | 5 | Direct architectural inspiration: immutable fact layers, derivations, and scalable code intelligence. |
| T2 | [joernio/joern](https://github.com/joernio/joern) | 2026-07-04 | 3303/425 | APACHE-2.0 | Scala | Code analysis platform based on code property graphs | CLI server queries | CPG graph | 5 | Study CPG schema, graph traversals, and vulnerability path workflows. |
| T2 | [ShiftLeftSecurity/codepropertygraph](https://github.com/ShiftLeftSecurity/codepropertygraph) | 2026-06-02 | 589/84 | APACHE-2.0 | Scala | CPG specification and utilities | Spec plus utilities | CPG graph | 4 | Prior-art schema reference for code property graphs without inheriting Joern's whole stack. |
| T2 | [CoatiSoftware/Sourcetrail](https://github.com/CoatiSoftware/Sourcetrail) | 2021-12-13 | 16478/1668 | GPL-3.0 | C++ | Interactive source explorer | Desktop UI | Code index graph | 4 | Archived but excellent UX prior art for visual code navigation and onboarding. |
| T2 | [afnanenayet/diffsitter](https://github.com/afnanenayet/diffsitter) | 2026-07-04 | 2385/53 | MIT | Rust | Tree-sitter semantic diffs | CLI | AST diff | 4 | Useful for entity identity, incremental reindexing, and avoiding line-number false positives. |
| T2 | [Wilfred/difftastic](https://github.com/Wilfred/difftastic) | 2026-07-02 | 25596/494 | MIT | Rust | Structural syntax-aware diff | CLI | Syntax trees | 4 | High-quality semantic diff UX; helps Parseltongue explain changed entities in PRs. |
| T2 | [comby-tools/comby](https://github.com/comby-tools/comby) | 2026-06-08 | 2663/74 | APACHE-2.0 | OCaml | Structural search and replace across many languages | CLI | Template matcher | 3 | Useful for refactoring workflows where graph detects target and structural rewrite applies fix. |
| T2 | [rust-lang/rust-analyzer](https://github.com/rust-lang/rust-analyzer) | 2026-07-05 | 16637/2120 | APACHE-2.0 | Rust | Rust compiler front-end for IDEs | LSP | Incremental semantic DB | 5 | Best-in-class incremental query architecture and IDE-grade symbol UX. |
| T2 | [salsa-rs/salsa](https://github.com/salsa-rs/salsa) | 2026-07-04 | 2894/215 | APACHE-2.0 | Rust | Incremental computation framework | Library | Query cache | 4 | Study for live reindex internals: memoized dependency-aware recomputation. |
| T2 | [openrewrite/rewrite](https://github.com/openrewrite/rewrite) | 2026-07-04 | 3583/533 | APACHE-2.0 | Java | Automated mass refactoring | CLI build plugins | Recipe engine | 4 | Strong precedent for turning analysis into safe code transformation recipes. |
| T2 | [opengrep/opengrep](https://github.com/opengrep/opengrep) | 2026-07-02 | 2764/227 | LGPL-2.1 | OCaml | Static code analysis engine | CLI CI | Rule engine | 4 | Semgrep-family alternative; useful for open governance and parse coverage practices. |
| T2 | [pmd/pmd](https://github.com/pmd/pmd) | 2026-07-02 | 5444/1566 | OTHER | Java | Extensible multi-language static analyzer | CLI CI | Rule engine | 3 | Prior art for language coverage, rule taxonomy, and noisy-signal management. |
| T2 | [SonarSource/sonarqube](https://github.com/SonarSource/sonarqube) | 2026-07-03 | 10757/2203 | LGPL-3.0 | Java | Continuous inspection platform | Server UI CI | Analysis DB | 3 | Useful for dashboards, quality gates, and how teams operationalize static findings. |
| T2 | [oracle/opengrok](https://github.com/oracle/opengrok) | 2026-06-29 | 4877/824 | OTHER | Java | Source search and cross-reference engine | Web UI | Search and xref index | 3 | Older but durable PMF around source navigation at scale. |
| T2 | [hound-search/hound](https://github.com/hound-search/hound) | 2026-05-10 | 5855/599 | MIT | JavaScript | Fast code searching service | Web UI | Search index | 2 | Baseline for simple multi-repo code search; useful as a lower-bound comparator. |
| T2 | [livegrep/livegrep](https://github.com/livegrep/livegrep) | 2026-02-10 | 2221/202 | OTHER | C++ | Interactive source grep | Web UI | Search index | 2 | Shows latency expectations for code search; Parseltongue should stay similarly snappy. |
| T2 | [BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep) | 2026-07-01 | 65809/2622 | UNLICENSE | Rust | Fast recursive grep | CLI | Text search | 2 | The baseline every agent uses; Parseltongue must explain when graph beats grep. |
| T2 | [quickwit-oss/tantivy](https://github.com/quickwit-oss/tantivy) | 2026-07-03 | 15500/936 | MIT | Rust | Rust full-text search engine library | Library | Inverted index | 3 | Useful if Parseltongue adds local text or hybrid search beside graph queries. |
| T2 | [microsoft/language-server-protocol](https://github.com/microsoft/language-server-protocol) | 2026-06-26 | 12912/975 | CC-BY-4.0 | HTML | Common protocol for language servers | Protocol | LSP messages | 4 | Interoperability anchor; Parseltongue can consume or complement LSP rather than replace it. |
| T2 | [microsoft/pyright](https://github.com/microsoft/pyright) | 2026-06-30 | 15499/1792 | OTHER | Python | Python static type checker | CLI LSP | Type analysis | 3 | Useful for Python precision where tree-sitter alone lacks semantic typing. |
| T2 | [clangd/clangd](https://github.com/clangd/clangd) | 2026-06-22 | 2232/94 | APACHE-2.0 | Shell | C C++ language server | LSP | Compiler index | 3 | Study C CPP compilation database and robust header-aware indexing. |
| T2 | [llvm/llvm-project](https://github.com/llvm/llvm-project) | 2026-07-05 | 39097/17715 | OTHER | LLVM | Compiler and toolchain technologies | Libraries tools | Compiler IR | 3 | Deep prior art for AST, indexing, and C-family semantic analysis. |
| T2 | [JetBrains/intellij-community](https://github.com/JetBrains/intellij-community) | 2026-07-04 | 20313/5978 | OTHER | Java | IDE platform | IDE platform | PSI indexes | 3 | Important for mature code intelligence UX and persistent index design. |
| T2 | [redhat-developer/vscode-java](https://github.com/redhat-developer/vscode-java) | 2026-06-29 | 2293/543 | EPL-2.0 | TypeScript | Java language support for VS Code | LSP extension | JDT index | 2 | Useful for Java LSP integration and editor workflow expectations. |
| T2 | [microsoft/TypeScript](https://github.com/microsoft/TypeScript) | 2026-06-29 | 109470/13461 | APACHE-2.0 | TypeScript | TypeScript compiler | Compiler API | Program graph | 3 | Useful for TS AST, symbol, and project graph handling. |
| T2 | [babel/babel](https://github.com/babel/babel) | 2026-07-04 | 43991/5827 | MIT | TypeScript | JavaScript compiler | Compiler plugins | AST pipeline | 2 | Useful for JS/TS transform architecture and plugin APIs. |
| T2 | [swc-project/swc](https://github.com/swc-project/swc) | 2026-07-04 | 34125/1431 | APACHE-2.0 | Rust | Rust-based web compiler platform | Compiler library | AST pipeline | 2 | Rust implementation reference for fast JS/TS parsing and transformation. |
| T2 | [biomejs/biome](https://github.com/biomejs/biome) | 2026-07-04 | 25208/1053 | APACHE-2.0 | Rust | Formatter linter CLI and LSP | CLI LSP | Parser and analyzer | 3 | Good Rust example of modern language tooling distribution and diagnostics UX. |
| T2 | [biomejs/gritql](https://github.com/biomejs/gritql) | 2026-06-12 | 4542/123 | MIT | Rust | Query language for searching, linting, modifying code | CLI query language | Code pattern engine | 4 | Very relevant for user-facing graph or AST query design. |
| T3 | [cozodb/cozo](https://github.com/cozodb/cozo) | 2024-12-04 | 4047/158 | MPL-2.0 | Rust | Relational graph vector DB with Datalog | Embedded DB | Graph Datalog vector | 5 | Current architectural backbone; study roadmap and vector graph hybrid options. |
| T3 | [facebook/rocksdb](https://github.com/facebook/rocksdb) | 2026-07-04 | 31833/6864 | GPL-2.0 | C++ | Embeddable persistent key-value store | Library | LSM KV store | 4 | Operational dependency lesson: write stalls, compaction, and platform tuning matter for PMF. |
| T3 | [dgraph-io/dgraph](https://github.com/dgraph-io/dgraph) | 2026-07-04 | 21722/1596 | APACHE-2.0 | Go | Distributed graph database | Server | Graph DB | 3 | Alternative if embedded graph gives way to team-scale server deployment. |
| T3 | [neo4j/neo4j](https://github.com/neo4j/neo4j) | 2026-07-01 | 16826/2650 | GPL-3.0 | Java | Property graph database | Server | Graph DB | 3 | Study Cypher UX and graph ecosystem even if not chosen as backend. |
| T3 | [kuzudb/kuzu](https://github.com/kuzudb/kuzu) | 2025-10-10 | 3997/498 | MIT | C++ | Embedded property graph database with Cypher | Embedded DB | Property graph plus vector/full-text | 4 | Strong alternative backend to monitor for local graph workloads. |
| T3 | [indradb/indradb](https://github.com/indradb/indradb) | 2025-08-16 | 2453/132 | MPL-2.0 | Rust | Rust graph database | Library DB | Graph DB | 2 | Rust-native alternative worth knowing, though lower ecosystem maturity. |
| T3 | [petgraph/petgraph](https://github.com/petgraph/petgraph) | 2026-04-04 | 3950/451 | APACHE-2.0 | Rust | Rust graph data structure library | Library | In-memory graph | 3 | Good for algorithm prototypes before committing to DB-backed queries. |
| T3 | [networkx/networkx](https://github.com/networkx/networkx) | 2026-07-03 | 17069/3538 | OTHER | Python | Network analysis in Python | Library | In-memory graph | 3 | Algorithm reference for centrality, clustering, and metrics before Rust ports. |
| T3 | [run-llama/llama_index](https://github.com/run-llama/llama_index) | 2026-07-02 | 50647/7684 | MIT | Python | Document agent and RAG platform | Library framework | Indexes and retrievers | 3 | Use as RAG comparison: Parseltongue's advantage is code-structured retrieval, not generic chunks. |
| T3 | [langchain-ai/langchain](https://github.com/langchain-ai/langchain) | 2026-07-05 | 140937/23407 | MIT | Python | Agent engineering platform | Library framework | Tools retrievers agents | 3 | Study tool abstractions and retriever integration points for agent ecosystems. |
| T3 | [microsoft/graphrag](https://github.com/microsoft/graphrag) | 2026-06-22 | 34183/3615 | MIT | Python | Graph-based RAG system | Library pipeline | Knowledge graph RAG | 3 | Useful for graph retrieval patterns, but Parseltongue's graph is deterministic code facts. |
| T3 | [github/github-mcp-server](https://github.com/github/github-mcp-server) | 2026-07-03 | 31195/4508 | MIT | Go | Official GitHub MCP server | MCP | GitHub API | 3 | Model for tool grouping, auth, and MCP ergonomics if Parseltongue re-enters MCP. |
| T3 | [JetBrains/mcp-jetbrains](https://github.com/JetBrains/mcp-jetbrains) | 2026-01-07 | 961/77 | APACHE-2.0 | JavaScript | MCP server for JetBrains IDEs | MCP | IDE context | 3 | Useful bridge pattern from agent to IDE state. |
| T3 | [disler/aider-mcp-server](https://github.com/disler/aider-mcp-server) | 2025-05-21 | 301/61 | UNKNOWN | Python | Minimal MCP server for Aider | MCP | Aider bridge | 2 | Small example of delegating agent work through MCP without overbuilding. |
| T3 | [zxfgds/mcp-code-indexer](https://github.com/zxfgds/mcp-code-indexer) | 2025-03-04 | 36/9 | UNKNOWN | Python | MCP code indexer | MCP | Code index | 3 | Useful weak lead for MCP packaging and agent-facing query naming. |
| T3 | [fluffypony/mcp-code-indexer](https://github.com/fluffypony/mcp-code-indexer) | 2026-02-04 | 18/5 | MIT | Python | Tracks file descriptions across codebases | MCP | Summaries index | 3 | Useful for token-aware summaries and gradual indexing despite weaker graph depth. |
| T3 | [zackyalgiffari/fast-index](https://github.com/zackyalgiffari/fast-index) | 2026-06-19 | 0/0 | MIT | Rust | Fast local MCP code indexer for Claude Code | MCP | Local index | 3 | Study low-friction Claude Code setup and latency expectations. |
| T3 | [lkwslm/tree-sitter-mcp-code-analyzer](https://github.com/lkwslm/tree-sitter-mcp-code-analyzer) | 2025-10-12 | 0/0 | UNKNOWN | Python | MCP and tree-sitter code analyzer | MCP | Tree-sitter analysis | 3 | Tiny but directly aligned; useful as a minimal design contrast. |

## Reusable Ideas for Parseltongue

1. Treat Parseltongue as an agent context provider, not only a code analyzer. The strongest T1 repos win by plugging into real coding loops: terminal agents, IDE agents, PR bots, MCP servers, and prompt-packers.
2. Keep the deterministic code graph as the moat. Glean, Kythe, SCIP, CodeQL, Joern, stack-graphs, and rust-analyzer all reinforce that stable symbol identity and queryable facts are the hard part.
3. Make grep and prompt-packers the baseline competitor. Ripgrep, Repomix, Gitingest, and code2prompt define the user expectation: fast, simple, explainable output. Parseltongue must beat them with precision, not ceremony.
4. Invest in incremental identity and semantic diff. Diffsitter, Difftastic, Salsa, rust-analyzer, SCIP, and Glean point toward stable entity identity, cached facts, and change-aware recomputation.
5. Separate storage from workflows. CozoDB can stay the embedded graph core, but Kuzu, Neo4j, Dgraph, Tantivy, and GraphRAG suggest optional backends or hybrid retrieval paths.
6. Design endpoint/tool names around jobs, not internals. The highest-PMF jobs in the table are code review risk, blast radius, onboarding, root-cause diagnosis, refactoring planning, and token-budgeted context.

## Ranked Study List

Study in table order. T1 repos are direct product/workflow comparisons. T2 repos are subsystem and prior-art foundations. T3 repos are backend, RAG, MCP, and ecosystem adapters.

Highest-priority first pass: `MrDawell/atrium`, `Xattaus/claude-brain`, `FaizaanAlFaisal/code-search`, `Nishant-Chaudhary5338/mcp-code-indexer`, `sourcebot-dev/sourcebot`, `Aider-AI/aider`, `yamadashy/repomix`, `tree-sitter/tree-sitter`, `ast-grep/ast-grep`, `github/codeql`, `scip-code/scip`, `facebookincubator/Glean`, `joernio/joern`, `rust-lang/rust-analyzer`, `salsa-rs/salsa`, `afnanenayet/diffsitter`, `Wilfred/difftastic`, `cozodb/cozo`, and `kuzudb/kuzu`.
