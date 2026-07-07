# TDD Progress Journal

- Task: Build five-file Tree-sitter reference corpus from all git-ref-repo repositories for Parseltongue
- Created: 2026-07-06 17:46:57Z
- Updated: 2026-07-07 02:27:45Z
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

### Session: 2026-07-07 01:15:33Z

#### Current Phase: Refactor

#### Tests Written:
- (none recorded)

#### Implementation Progress:
- Ran cgc_batch_runner.py bounded sample across 10 manifest repos; 4 completed and 6 timed out at 45 seconds
- Regenerated cgc-run-summary.tsv from /tmp/codex-code-intel/codegraphcontext and updated repo-evidence-index.tsv CGC counters
- Refreshed repo-coverage-index.md and completion-audit.md with current CGC totals and explicit incomplete verdict
- Terminated stale current-workspace analysis process groups for tree-sitter-cpp and tree-sitter-bash

#### Current Focus:
Research003 CGC coverage audit refresh after bounded batch

#### Next Steps:
- Continue cgc_batch_runner.py by resumable batches until all 609 manifest repos have attempted status, or document CGC infeasibility for user acceptance
- Optionally deepen direct source citations for top API-evidence repos not already represented in the five corpus files

#### Context Notes:
- Goal remains active because each-and-every-repo CGC browsing is not proven and original parallel-agent provenance remains unmet

#### Performance/Metrics:
- CGC run dirs scanned: 166; manifest repos with any CGC run: 52; manifest repos with complete CGC artifacts: 23
- Latest cgc-batch-status.tsv rows: 10 attempts, 4 complete, 6 timeout_45s

### Session: 2026-07-07 01:34:28Z

#### Current Phase: Refactor

#### Tests Written:
- (none recorded)

#### Implementation Progress:
- Ran second resumable cgc_batch_runner.py batch across 15 additional unattempted manifest repos
- Added docs/research003/refresh_research003_audit.py to regenerate cgc-run-summary.tsv, repo-evidence-index.tsv, repo-coverage-index.md, and completion-audit.md reproducibly
- Refreshed coverage artifacts after the second batch

#### Current Focus:
Continue all-repo CGC coverage for Research003

#### Next Steps:
- Continue cgc_batch_runner.py by resumable batches until cgc-batch-status.tsv records all 609 manifest repos
- Investigate or document CGC NoneType split failure for AndyInternet__indexer and related failed repos
- After all repos are attempted, decide whether timed-out repos need longer retry windows or documented CGC infeasibility

#### Context Notes:
- Goal remains active; each-and-every-repo CGC browsing is still not proven
- AndyInternet__indexer failed with CGC runtime error: 'NoneType' object has no attribute 'split'

#### Performance/Metrics:
- cgc-batch-status.tsv rows: 25 attempts; 10 complete; 1 failed; 14 timeout_45s
- cgc-run-summary.tsv rows: 190 run dirs; manifest repos with any CGC run: 68; manifest repos with complete CGC artifacts: 31

### Session: 2026-07-07 02:24:02Z

#### Current Phase: Refactor

#### Tests Written:
- (none recorded)

#### Implementation Progress:
- Terminated stray current-workspace CGC scan process group for Christoph__treesitter-mcp after process audit
- Ran refresh_research003_audit.py again after process cleanup so cgc-run-summary.tsv and coverage docs match current /tmp CGC output directories

#### Current Focus:
Finalize continuation after second CGC batch and audit refresh

#### Next Steps:
- Resume cgc_batch_runner.py from cgc-batch-status.tsv row 26 and continue toward all 609 manifest repos
- Keep using refresh_research003_audit.py after each bounded batch to update audit artifacts
- At full attempted coverage, retry selected failed/timeouts with longer per-repo timeout or document CGC infeasibility

#### Context Notes:
- Filtered process check after TERM showed no remaining current-workspace CGC runner or scan wrapper except the check command itself
- Goal remains active; cgc-batch-status.tsv covers 25 of 609 manifest repos

#### Performance/Metrics:
- Latest refresh: cgc-run-summary.tsv rows=212; manifest repos with any CGC run=80; complete CGC artifact repos=38
- Durable cgc-batch-status.tsv rows=25; complete=10; failed=1; timeout_45s=14

### Session: 2026-07-07 02:27:45Z

#### Current Phase: Refactor

#### Tests Written:
- (none recorded)

#### Implementation Progress:
- Ran resumable `cgc_batch_runner.py` across the next manifest slice, capturing `Ataraxy-Labs__weave` (timeout), `Benjamin-Davies__tree-sitter-relview` (complete), and entering `BloopAI__bloop` before interruption
- Refreshed audit evidence after the interrupted run with `refresh_research003_audit.py`
- Checked all generated docs and evidence artifacts for consistency before checkpointing

#### Current Focus:
Provide a checkpoint commit of all current research artifacts.

#### Next Steps:
- Resume CGC scanning for remaining manifest repositories when user wants full-complete coverage

#### Context Notes:
- User requested quick closure; this checkpoint reflects the current run state

#### Performance/Metrics:
- cgc-batch-status.tsv rows: 27 attempts; 11 complete; 1 failed; 15 timeout_45s
- cgc-run-summary.tsv rows: 221; manifest repos with any CGC run: 86; complete CGC artifact repos: 39
