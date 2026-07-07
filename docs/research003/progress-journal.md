# TDD Progress Journal

- Task: Build five-file Tree-sitter reference corpus from all git-ref-repo repositories for Parseltongue
- Created: 2026-07-06 17:46:57Z
- Updated: 2026-07-06 18:27:53Z
- Current Phase: Refactor
- Status: active

## Sessions

### Session: 2026-07-06 17:47:53Z

#### Current Phase: Refactor

#### Tests Written:
- (none recorded)

#### Implementation Progress:
- docs/research003/repo-manifest.txt: 609 repo paths captured
- docs/research003/repo-shard-1.txt..repo-shard-5.txt: balanced shard files created

#### Current Focus:
Corpus setup: manifest, shard files, CodeGraphContext smoke indexing, and five-agent research design

#### Next Steps:
- Spawn five workers with disjoint write targets tree-sitter-patterns-1.md through tree-sitter-patterns-5.md
- Collect CodeGraphContext smoke result and decide graph-indexing cadence for all shards
- Verify each output file cites concrete repository paths and covers its thematic lens

#### Context Notes:
- Objective is research corpus construction, not code TDD; using Refactor phase as a durable progress journal state per skill schema.
- 609 .git dirs found; git-ref-repo/index.md says 608 populated clones plus one empty clone.
- Initial rg evidence file: /tmp/parseltongue-ts-rg.txt with 330 literal Tree-sitter matches.

#### Performance/Metrics:
- Repo manifest: 609 repos; shards: 122/122/122/122/121

### Session: 2026-07-06 17:53:50Z

#### Current Phase: Refactor

#### Tests Written:
- (none recorded)

#### Implementation Progress:
- docs/research003/repo-feature-summary.tsv: all 609 repos summarized with feature/path counters
- /tmp/parseltongue-api-evidence.txt: 46,908 parser/query/traversal API matches captured
- /tmp/parseltongue-grammar-files.txt: 4,860 grammar/query asset paths captured
- CGC attempted on tree-sitter__tree-sitter and Christoph__treesitter-mcp; both terminated before complete graph output, preserving run dirs under /tmp/codex-code-intel/codegraphcontext

#### Current Focus:
Parallel workers running; local evidence substrate expanded

#### Next Steps:
- Wait for five worker checkpoint files and inspect line counts
- Patch or supplement any weak file with local evidence from Christoph__treesitter-mcp, wrale__mcp-server-tree-sitter, chunkhound, Ataraxy-Labs__sem, CodeGraphContext, Aider/Tabby/difftastic
- Run completion audit for five markdown files, repo coverage manifests, and evidence citations

#### Context Notes:
- Sent checkpoint instruction to all five workers to write current evidence now with explicit gaps.

#### Performance/Metrics:
- CGC status: attempted 2 repos, both incomplete/terminated; source scans are authoritative for all-repo coverage.

### Session: 2026-07-06 18:16:48Z

#### Current Phase: Green

#### Tests Written:
- wc tree-sitter-patterns: passed - 5 files, 5923 total lines
- required section grep: passed - phase scaffolding, Skeptical Systems Engineer, Where found, Rust translation, anti-patterns, and transfer principles found

#### Implementation Progress:
- Created tree-sitter-patterns-1.md covering parser runtime, query execution, traversal, spans, incremental parsing, and runtime anti-patterns
- Created tree-sitter-patterns-2.md covering grammar integration, query layers, capture taxonomy, language packs, injections, locals, textobjects, and multi-language ontology
- Created tree-sitter-patterns-3.md covering repo scanning, ignore modes, entity models, graph edges, chunks, caches, deletes, and golden drift
- Created tree-sitter-patterns-4.md covering LLM companion context, repo-map budgets, token accounting, MCP tools, query guards, watch mode, and context evidence cards
- Created tree-sitter-patterns-5.md covering query/capture tests, corpus/fuzz tests, binding tests, multi-language goldens, benchmarks, config isolation, and anti-patterns

#### Current Focus:
Created five Tree-sitter research corpus files from local repository evidence

#### Next Steps:
- Optional: deepen with per-shard repo-by-repo appendices or rerun CGC with smaller per-repo timeouts if tool stability improves

#### Context Notes:
- CGC evidence-reader scans were attempted earlier but terminated with exit 143 before usable graph output; docs state this caveat
- Initial five spawned worker agents did not produce files and were closed; local synthesis completed the deliverables

#### Performance/Metrics:
- 609 repositories listed in repo-manifest.txt; repo-feature-summary.tsv has 610 lines including header
- Broad SCM scan found 6267 .scm query files; docs use this as broad evidence with curated direct source reads

### Session: 2026-07-06 18:27:53Z

#### Current Phase: Refactor

#### Tests Written:
- completion audit: passed - docs/research003/completion-audit.md lists achieved, partial, and not-achieved requirements
- repo coverage index: passed - repo-evidence-index.tsv has 609 repo rows; coverage index reports 166874 literal Tree-sitter match lines and 497 API-evidence repos
- CGC runner syntax: passed - python3 -m py_compile docs/research003/cgc_batch_runner.py
- CGC runner no-op: passed - --limit 0 exits with attempted=0 and starts no CGC process

#### Implementation Progress:
- docs/research003/completion-audit.md: added requirement-by-requirement audit and explicit not-complete verdict
- docs/research003/repo-evidence-index.tsv and repo-coverage-index.md: added per-repo evidence counters across manifest
- docs/research003/cgc-run-summary.tsv and cgc-bounded-attempts.tsv: recorded existing/current CGC run status and high-signal retry failures
- docs/research003/cgc_batch_runner.py: added resumable process-group-timeout CGC batch runner for all-repo gap

#### Current Focus:
Audited original objective against current research003 artifacts and converted remaining CGC gap into a resumable runner

#### Next Steps:
- Run cgc_batch_runner.py across repo-manifest.txt with conservative timeout and resume, likely by shard/limit batches
- After full runner completion, regenerate cgc-run-summary.tsv and completion-audit.md
- If CGC remains infeasible, document failure mode and ask whether text/path evidence can be accepted as fallback

#### Context Notes:
- Original goal remains active: CGC browsing of each and every repo is not yet proven; five spawned parallel agents did not produce files
- Killed leftover CGC process groups from interrupted/background attempts; current ps check showed no live CGC indexing processes

#### Performance/Metrics:
- repo-evidence-index rows: 609; cgc-run-summary rows: 147; complete CGC artifact repos in coverage index: 17
