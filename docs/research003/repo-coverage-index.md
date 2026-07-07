# Repo Coverage Index for Research003

Generated from current local evidence. This file is an audit companion for the five `tree-sitter-patterns-*.md` corpus files.

## Scope

- Manifest repositories: 609
- Repositories with literal Tree-sitter matches: 567
- Literal Tree-sitter match lines from `/tmp/parseltongue-ts-rg-v2.txt`: 166874
- Repositories with parser/query/traversal API evidence matches: 497
- Repositories with grammar/query asset paths: 289
- Repositories with `.scm` query assets: 230
- Repositories with `tags.scm` assets: 58
- Repositories with at least one CGC run directory: 86
- Repositories with complete CGC artifact sets: 39
- Repositories with incomplete/error CGC run evidence: 51
- CGC run directories scanned under `/tmp/codex-code-intel/codegraphcontext`: 221
- Latest resumable batch attempts recorded in `cgc-batch-status.tsv`: 27
- Latest batch completions: 11
- Latest batch failures: 1
- Latest batch timeouts: 15

## CGC Run Status Totals

| Status | Count |
|---|---:|
| `complete_artifacts` | 64 |
| `index_error` | 6 |
| `interrupted_or_incomplete_index` | 93 |
| `unknown` | 58 |

## Latest Resumable CGC Batch

| Status | Count |
|---|---:|
| `complete` | 11 |
| `failed` | 1 |
| `timeout_45s` | 15 |

## Evidence Files

- `repo-manifest.txt`: authoritative 609-repo list used for coverage accounting.
- `repo-evidence-index.tsv`: per-repo counters from broad scans and CGC run directories.
- `repo-feature-summary.tsv`: earlier coarse feature/path counter table.
- `cgc-run-summary.tsv`: best-effort summary of CGC run directories under `/tmp/codex-code-intel/codegraphcontext`.
- `cgc-bounded-attempts.tsv`: prior bounded CGC retry outcomes for selected high-signal repos.
- `cgc-batch-status.tsv`: latest resumable bounded batch-run status from `cgc_batch_runner.py`.
- `/tmp/parseltongue-ts-rg-v2.txt`: literal Tree-sitter evidence scan across repository contents.
- `/tmp/parseltongue-api-evidence.txt`: parser/query/traversal API evidence scan.
- `/tmp/parseltongue-grammar-files.txt`: grammar and query asset path scan.

## Important Limitation

This index proves broad text/path coverage across the manifest and records the current state of CGC attempts. It does not prove that every repository was semantically browsed through CodeGraphContext, because CGC runs are incomplete for most repositories and the latest bounded runner produced completions, failures, and timeouts.

## Top Repositories by Literal Tree-sitter matches

| Count | Repository |
|---:|---|
| 18091 | `xberg-io__tree-sitter-language-pack` |
| 7457 | `FosterG4__tree-sitter-mcpsaver` |
| 6563 | `odvcencio__gotreesitter` |
| 6178 | `github__semantic` |
| 5721 | `DeusData__codebase-memory-mcp` |
| 5541 | `win4r__codebase-memory-mcp-pro` |
| 3474 | `biomejs__gritql` |
| 2511 | `GlitterKill__sdl-mcp` |
| 2413 | `vitali87__code-graph-rag` |
| 2259 | `ontograph__ontoindex` |
| 2058 | `abhigyanpatwari__GitNexus` |
| 2052 | `yijunyu__tree-sitter-parsers` |
| 1933 | `bearcove__arborium` |
| 1893 | `tree-sitter__tree-sitter` |
| 1819 | `github__codeql` |
| 1440 | `simonbs__TreeSitterLanguages` |
| 1379 | `chunkhound__chunkhound` |
| 1360 | `Wilfred__difftastic` |
| 1288 | `albfan__rust-tree-sitter-ast-viewer` |
| 1215 | `zed-industries__zed` |
| 1171 | `opengrep__opengrep` |
| 1157 | `semgrep__semgrep` |
| 1107 | `bonede__tree-sitter-ng` |
| 1088 | `smacker__go-tree-sitter` |
| 1057 | `Egonex-AI__Understand-Anything` |

## Top Repositories by API evidence matches

| Count | Repository |
|---:|---|
| 4453 | `CodeBendKit__codeseek` |
| 2695 | `DeusData__codebase-memory-mcp` |
| 2584 | `win4r__codebase-memory-mcp-pro` |
| 2213 | `tree-sitter__tree-sitter` |
| 1977 | `xberg-io__tree-sitter-language-pack` |
| 1763 | `0sec-labs__foxguard` |
| 1727 | `vitali87__code-graph-rag` |
| 1550 | `Jakobeha__type-sitter` |
| 1409 | `jimhester__rtreesitter` |
| 1071 | `bearcove__arborium` |
| 1059 | `zed-industries__zed` |
| 891 | `SonarSource__sonarqube` |
| 818 | `Ataraxy-Labs__sem` |
| 784 | `afnanenayet__diffsitter` |
| 721 | `tree-sitter__go-tree-sitter` |
| 660 | `albfan__rust-tree-sitter-ast-viewer` |
| 631 | `probelabs__probe` |
| 612 | `ViperJuice__Code-Index-MCP` |
| 517 | `smacker__go-tree-sitter` |
| 516 | `biomejs__gritql` |
| 483 | `tree-sitter__py-tree-sitter` |
| 456 | `AryanSaini26__CodeAtlas` |
| 456 | `glyphtrail__glyphtrail` |
| 451 | `cmillstead__codesight-mcp` |
| 433 | `jgravelle__jcodemunch-mcp` |

## Top Repositories by Grammar/query asset paths

| Count | Repository |
|---:|---|
| 793 | `romus204__tree-sitter-manager.nvim` |
| 787 | `arborist-ts__arborist.nvim` |
| 786 | `nvim-treesitter__nvim-treesitter` |
| 298 | `bearcove__arborium` |
| 187 | `FosterG4__tree-sitter-mcpsaver` |
| 115 | `CodeEditApp__CodeEditLanguages` |
| 100 | `biomejs__gritql` |
| 91 | `emacs-tree-sitter__tree-sitter-langs` |
| 90 | `yijunyu__tree-sitter-parsers` |
| 81 | `ontograph__ontoindex` |
| 67 | `simonbs__TreeSitterLanguages` |
| 57 | `tree-sitter__tree-sitter` |
| 55 | `semgrep__ocaml-tree-sitter-semgrep` |
| 47 | `zed-industries__zed` |
| 29 | `intersystems__tree-sitter-objectscript` |
| 22 | `Wilfred__difftastic` |
| 19 | `aheber__tree-sitter-sfapex` |
| 18 | `casey__tree-sitter-just` |
| 18 | `zeta1999__ocaml-tree-sitter` |
| 16 | `semgrep__ocaml-tree-sitter-languages` |
| 14 | `moonbitlang__tree-sitter-moonbit` |
| 12 | `github__codeql` |
| 11 | `acristoffers__tree-sitter-matlab` |
| 11 | `rayliwell__tree-sitter-rstml` |
| 10 | `cathaysia__tree-sitter-jinja` |

## Top Repositories by SCM query paths

| Count | Repository |
|---:|---|
| 793 | `romus204__tree-sitter-manager.nvim` |
| 787 | `arborist-ts__arborist.nvim` |
| 786 | `nvim-treesitter__nvim-treesitter` |
| 184 | `bearcove__arborium` |
| 115 | `CodeEditApp__CodeEditLanguages` |
| 91 | `emacs-tree-sitter__tree-sitter-langs` |
| 67 | `simonbs__TreeSitterLanguages` |
| 55 | `FosterG4__tree-sitter-mcpsaver` |
| 47 | `zed-industries__zed` |
| 43 | `biomejs__gritql` |
| 36 | `ontograph__ontoindex` |
| 35 | `yijunyu__tree-sitter-parsers` |
| 18 | `intersystems__tree-sitter-objectscript` |
| 15 | `casey__tree-sitter-just` |
| 9 | `Wilfred__difftastic` |
| 9 | `xberg-io__tree-sitter-language-pack` |
| 8 | `acristoffers__tree-sitter-matlab` |
| 7 | `moonbitlang__tree-sitter-moonbit` |
| 7 | `rayliwell__tree-sitter-rstml` |
| 6 | `aheber__tree-sitter-sfapex` |
| 6 | `github__codeql` |
| 6 | `simonbs__Runestone` |
| 6 | `zee-editor__zee` |
| 5 | `tlaplus-community__tree-sitter-tlaplus` |
| 4 | `HeytalePazguato__tree-sitter-iec61131-3-st` |

## Top Repositories by Indexing feature files

| Count | Repository |
|---:|---|
| 3 | `jiteshy__backstage-plugin-codeinsight` |
| 2 | `Jakedismo__codegraph-rust` |
| 2 | `Manikanta-Reddy-Pasala__AiForgeMemory` |
| 2 | `chunkhound__chunkhound` |
| 2 | `fl0w1nd__repomap-mcp` |
| 2 | `sdsrss__code-graph-mcp` |
| 1 | `AB498__code-context-provider-mcp` |
| 1 | `AryanSaini26__CodeAtlas` |
| 1 | `Ataraxy-Labs__sem` |
| 1 | `Ataraxy-Labs__weave` |
| 1 | `Bpolat0__atlasmemory` |
| 1 | `Christoph__treesitter-mcp` |
| 1 | `DeusData__codebase-memory-mcp` |
| 1 | `FosterG4__tree-sitter-mcpsaver` |
| 1 | `GlitterKill__sdl-mcp` |
| 1 | `MikeRecognex__mcp-codebase-index` |
| 1 | `Nishant-Chaudhary5338__mcp-code-indexer` |
| 1 | `Regsorm__code-index-mcp` |
| 1 | `ShiftLeftSecurity__codepropertygraph` |
| 1 | `Stoica-Mihai__recast` |
| 1 | `ViperJuice__Code-Index-MCP` |
| 1 | `YPYT1__EverMind` |
| 1 | `adamdelezuch89__repo-map-mcp` |
| 1 | `afnanenayet__diffsitter` |
| 1 | `cUDGk__tree-sitter-mcp` |

## Top Repositories by Testing feature files

| Count | Repository |
|---:|---|
| 2 | `GlitterKill__sdl-mcp` |
| 2 | `openrewrite__rewrite` |
| 2 | `zed-industries__zed` |
| 1 | `0sec-labs__foxguard` |
| 1 | `AKrichevski__Lodebrook` |
| 1 | `Aider-AI__aider` |
| 1 | `Ataraxy-Labs__sem` |
| 1 | `BurntSushi__ripgrep` |
| 1 | `Cranot__roam-code` |
| 1 | `Graphify-Labs__graphify` |
| 1 | `HelgeSverre__tree-sitter-applescript` |
| 1 | `Muvon__octocode` |
| 1 | `Stoica-Mihai__recast` |
| 1 | `ViperJuice__Code-Index-MCP` |
| 1 | `VoidNxSEC__cerebro` |
| 1 | `afnanenayet__diffsitter` |
| 1 | `albfan__rust-tree-sitter-ast-viewer` |
| 1 | `biomejs__biome` |
| 1 | `datwaft__tree-sitter-corpus` |
| 1 | `demirmusa__nanocontext` |
| 1 | `dgraph-io__dgraph` |
| 1 | `facebook__rocksdb` |
| 1 | `greglas75__codesift` |
| 1 | `indradb__indradb` |
| 1 | `jgravelle__jcodemunch-mcp` |

## Top Repositories by Complete CGC runs

| Count | Repository |
|---:|---|
| 5 | `AB498__code-context-provider-mcp` |
| 2 | `BrianHicks__tree-grepper` |
| 2 | `tree-sitter__tree-sitter-html` |
| 2 | `tree-sitter__tree-sitter-json` |
| 1 | `71__vscode-tree-sitter-api` |
| 1 | `AbstractMachinesLab__tree-sitter-sexp` |
| 1 | `Aerijo__tree-sitter-biber` |
| 1 | `Akzestia__tree-sitter-cql` |
| 1 | `AndroidIDEOfficial__tree-sitter-aidl` |
| 1 | `AndroidIDEOfficial__tree-sitter-log` |
| 1 | `AndroidIDEOfficial__tree-sitter-properties` |
| 1 | `AndroidIDEOfficial__tree-sitter-xml` |
| 1 | `Anirudh-030307__CHATBOT` |
| 1 | `Aryan1643__swe-agent` |
| 1 | `Benjamin-Davies__tree-sitter-relview` |
| 1 | `BloopAI__bloop` |
| 1 | `INS-JVidal__code-primer` |
| 1 | `RageLtd__cartographer` |
| 1 | `afnanenayet__diffsitter` |
| 1 | `ast-grep__ast-grep` |
| 1 | `bydecom__graphrag-code` |
| 1 | `bytecodealliance__tree-sitter-wit` |
| 1 | `casey__tree-sitter-just` |
| 1 | `framadhita4__syntax-tree-codebase-mcp` |
| 1 | `indradb__indradb` |
