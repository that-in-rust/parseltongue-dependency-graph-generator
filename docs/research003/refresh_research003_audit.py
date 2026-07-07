#!/usr/bin/env python3
"""Refresh Research003 coverage and completion-audit artifacts.

This utility is intentionally scoped to the Research003 corpus. It summarizes
local evidence files and CodeGraphContext output directories so continuation
agents can update the audit without retyping a large ad-hoc Python snippet.
"""

from __future__ import annotations

import csv
import re
from collections import Counter, defaultdict
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RESEARCH_DIR = REPO_ROOT / "docs" / "research003"
CGC_ROOT = Path("/tmp/codex-code-intel/codegraphcontext")

MANIFEST_PATH = RESEARCH_DIR / "repo-manifest.txt"
EVIDENCE_PATH = RESEARCH_DIR / "repo-evidence-index.tsv"
COVERAGE_PATH = RESEARCH_DIR / "repo-coverage-index.md"
AUDIT_PATH = RESEARCH_DIR / "completion-audit.md"
CGC_SUMMARY_PATH = RESEARCH_DIR / "cgc-run-summary.tsv"
BATCH_STATUS_PATH = RESEARCH_DIR / "cgc-batch-status.tsv"

CGC_FIELDNAMES = [
    "run_dir",
    "repo_guess",
    "status",
    "has_list",
    "has_stats",
    "has_files_query",
    "has_functions_find",
    "index_tail",
]


def read_lines(path: Path) -> list[str]:
    return [line.strip() for line in path.read_text().splitlines() if line.strip()]


def read_tsv(path: Path) -> list[dict[str, str]]:
    if not path.exists():
        return []
    with path.open(newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def write_tsv(path: Path, fieldnames: list[str], rows: list[dict[str, str]]) -> None:
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames, delimiter="\t")
        writer.writeheader()
        writer.writerows(rows)


def clean_tail(path: Path) -> str:
    if not path.exists():
        return ""
    try:
        lines = [
            line.strip()
            for line in path.read_text(errors="replace").splitlines()
            if line.strip()
        ]
    except OSError as exc:
        return f"<read error: {exc}>"
    if not lines:
        return ""
    return lines[-1].replace("\t", " ").replace("|", "/")[:500]


def repo_guess_from_run(path: Path) -> str:
    match = re.match(r"^(.*)-\d{8}-\d{6}$", path.name)
    return match.group(1) if match else path.name


def classify_cgc_run(run_dir: Path) -> dict[str, str]:
    has_list = (run_dir / "list.txt").exists()
    has_stats = (run_dir / "stats.txt").exists()
    has_files_query = (run_dir / "files_query.txt").exists()
    has_functions_find = (run_dir / "functions_find.txt").exists()
    has_index = (run_dir / "index.txt").exists()
    has_wal = (run_dir / "ladybugdb.sqlite.wal").exists()
    index_tail = clean_tail(run_dir / "index.txt")

    if has_list and has_stats and has_files_query and has_functions_find:
        status = "complete_artifacts"
    elif has_index and (
        "Traceback" in index_tail
        or "NoneType" in index_tail
        or "error" in index_tail.lower()
    ):
        status = "index_error"
    elif has_index and "success" in index_tail.lower():
        status = "index_only"
    elif has_index or has_wal:
        status = "interrupted_or_incomplete_index"
    else:
        status = "unknown"

    return {
        "run_dir": str(run_dir),
        "repo_guess": repo_guess_from_run(run_dir),
        "status": status,
        "has_list": str(has_list),
        "has_stats": str(has_stats),
        "has_files_query": str(has_files_query),
        "has_functions_find": str(has_functions_find),
        "index_tail": index_tail,
    }


def summarize_cgc_runs() -> list[dict[str, str]]:
    if not CGC_ROOT.exists():
        return []
    return [
        classify_cgc_run(path)
        for path in sorted(CGC_ROOT.iterdir(), key=lambda item: str(item))
        if path.is_dir()
    ]


def int_at(row: dict[str, str], key: str) -> int:
    try:
        return int(row.get(key, "0") or 0)
    except ValueError:
        return 0


def short_repo(repo_path: str) -> str:
    return Path(repo_path).name


def top_rows(rows: list[dict[str, str]], metric: str, limit: int = 25) -> list[tuple[int, str]]:
    ranked = sorted(
        ((int_at(row, metric), short_repo(row["repo"])) for row in rows),
        key=lambda item: (-item[0], item[1]),
    )
    return [(count, name) for count, name in ranked if count > 0][:limit]


def md_count_table(title: str, rows: list[tuple[int, str]]) -> str:
    lines = [f"## {title}", "", "| Count | Repository |", "|---:|---|"]
    if rows:
        for count, name in rows:
            lines.append(f"| {count} | `{name}` |")
    else:
        lines.append("| 0 | _none_ |")
    lines.append("")
    return "\n".join(lines)


def update_evidence_cgc_columns(
    evidence_rows: list[dict[str, str]],
    cgc_rows: list[dict[str, str]],
    manifest: list[str],
) -> tuple[list[dict[str, str]], set[str], set[str], set[str]]:
    manifest_by_name = {Path(repo).name: repo for repo in manifest}
    cgc_by_repo: defaultdict[str, Counter[str]] = defaultdict(Counter)

    for row in cgc_rows:
        repo = manifest_by_name.get(row["repo_guess"])
        if not repo:
            continue
        cgc_by_repo[repo]["runs"] += 1
        if row["status"] == "complete_artifacts":
            cgc_by_repo[repo]["complete"] += 1
        elif row["status"] in {"interrupted_or_incomplete_index", "index_error"}:
            cgc_by_repo[repo]["error_or_incomplete"] += 1

    updated = []
    for row in evidence_rows:
        counts = cgc_by_repo.get(row["repo"], Counter())
        row["cgc_runs"] = str(counts.get("runs", 0))
        row["cgc_complete_runs"] = str(counts.get("complete", 0))
        row["cgc_error_or_incomplete_runs"] = str(
            counts.get("error_or_incomplete", 0)
        )
        updated.append(row)

    any_cgc = {repo for repo, counts in cgc_by_repo.items() if counts.get("runs", 0)}
    complete = {
        repo for repo, counts in cgc_by_repo.items() if counts.get("complete", 0)
    }
    incomplete = {
        repo
        for repo, counts in cgc_by_repo.items()
        if counts.get("error_or_incomplete", 0)
    }
    return updated, any_cgc, complete, incomplete


def write_coverage_index(
    evidence_rows: list[dict[str, str]],
    manifest: list[str],
    cgc_rows: list[dict[str, str]],
    batch_rows: list[dict[str, str]],
    any_cgc: set[str],
    complete_cgc: set[str],
    incomplete_cgc: set[str],
) -> None:
    cgc_status = Counter(row["status"] for row in cgc_rows)
    batch_status = Counter(row.get("status", "") for row in batch_rows)
    batch_timeouts = sum(
        count for status, count in batch_status.items() if status.startswith("timeout")
    )

    literal_repos = sum(
        1 for row in evidence_rows if int_at(row, "literal_tree_sitter_matches") > 0
    )
    api_repos = sum(1 for row in evidence_rows if int_at(row, "api_evidence_matches") > 0)
    grammar_repos = sum(1 for row in evidence_rows if int_at(row, "grammar_asset_paths") > 0)
    scm_repos = sum(1 for row in evidence_rows if int_at(row, "scm_paths") > 0)
    tags_repos = sum(1 for row in evidence_rows if int_at(row, "tags_scm_paths") > 0)
    literal_lines = sum(int_at(row, "literal_tree_sitter_matches") for row in evidence_rows)

    lines = [
        "# Repo Coverage Index for Research003",
        "",
        "Generated from current local evidence. This file is an audit companion for the five `tree-sitter-patterns-*.md` corpus files.",
        "",
        "## Scope",
        "",
        f"- Manifest repositories: {len(manifest)}",
        f"- Repositories with literal Tree-sitter matches: {literal_repos}",
        f"- Literal Tree-sitter match lines from `/tmp/parseltongue-ts-rg-v2.txt`: {literal_lines}",
        f"- Repositories with parser/query/traversal API evidence matches: {api_repos}",
        f"- Repositories with grammar/query asset paths: {grammar_repos}",
        f"- Repositories with `.scm` query assets: {scm_repos}",
        f"- Repositories with `tags.scm` assets: {tags_repos}",
        f"- Repositories with at least one CGC run directory: {len(any_cgc)}",
        f"- Repositories with complete CGC artifact sets: {len(complete_cgc)}",
        f"- Repositories with incomplete/error CGC run evidence: {len(incomplete_cgc)}",
        f"- CGC run directories scanned under `/tmp/codex-code-intel/codegraphcontext`: {len(cgc_rows)}",
        f"- Latest resumable batch attempts recorded in `cgc-batch-status.tsv`: {len(batch_rows)}",
        f"- Latest batch completions: {batch_status.get('complete', 0)}",
        f"- Latest batch failures: {batch_status.get('failed', 0)}",
        f"- Latest batch timeouts: {batch_timeouts}",
        "",
        "## CGC Run Status Totals",
        "",
        "| Status | Count |",
        "|---|---:|",
    ]
    for status, count in sorted(cgc_status.items()):
        lines.append(f"| `{status}` | {count} |")

    lines.extend(
        [
            "",
            "## Latest Resumable CGC Batch",
            "",
            "| Status | Count |",
            "|---|---:|",
        ]
    )
    if batch_status:
        for status, count in sorted(batch_status.items()):
            lines.append(f"| `{status}` | {count} |")
    else:
        lines.append("| _none_ | 0 |")

    lines.extend(
        [
            "",
            "## Evidence Files",
            "",
            "- `repo-manifest.txt`: authoritative 609-repo list used for coverage accounting.",
            "- `repo-evidence-index.tsv`: per-repo counters from broad scans and CGC run directories.",
            "- `repo-feature-summary.tsv`: earlier coarse feature/path counter table.",
            "- `cgc-run-summary.tsv`: best-effort summary of CGC run directories under `/tmp/codex-code-intel/codegraphcontext`.",
            "- `cgc-bounded-attempts.tsv`: prior bounded CGC retry outcomes for selected high-signal repos.",
            "- `cgc-batch-status.tsv`: latest resumable bounded batch-run status from `cgc_batch_runner.py`.",
            "- `/tmp/parseltongue-ts-rg-v2.txt`: literal Tree-sitter evidence scan across repository contents.",
            "- `/tmp/parseltongue-api-evidence.txt`: parser/query/traversal API evidence scan.",
            "- `/tmp/parseltongue-grammar-files.txt`: grammar and query asset path scan.",
            "",
            "## Important Limitation",
            "",
            "This index proves broad text/path coverage across the manifest and records the current state of CGC attempts. It does not prove that every repository was semantically browsed through CodeGraphContext, because CGC runs are incomplete for most repositories and the latest bounded runner produced completions, failures, and timeouts.",
            "",
        ]
    )

    for title, metric in [
        ("Top Repositories by Literal Tree-sitter matches", "literal_tree_sitter_matches"),
        ("Top Repositories by API evidence matches", "api_evidence_matches"),
        ("Top Repositories by Grammar/query asset paths", "grammar_asset_paths"),
        ("Top Repositories by SCM query paths", "scm_paths"),
        ("Top Repositories by Indexing feature files", "indexing_feature_files"),
        ("Top Repositories by Testing feature files", "testing_feature_files"),
        ("Top Repositories by Complete CGC runs", "cgc_complete_runs"),
    ]:
        lines.append(md_count_table(title, top_rows(evidence_rows, metric)))

    COVERAGE_PATH.write_text("\n".join(lines).rstrip() + "\n")


def write_completion_audit(
    manifest: list[str],
    cgc_rows: list[dict[str, str]],
    batch_rows: list[dict[str, str]],
    any_cgc: set[str],
    complete_cgc: set[str],
) -> None:
    batch_status = Counter(row.get("status", "") for row in batch_rows)
    batch_timeouts = sum(
        count for status, count in batch_status.items() if status.startswith("timeout")
    )
    batch_note = "No latest batch status file was present."
    if batch_rows:
        batch_note = (
            f"Latest `cgc_batch_runner.py` status file records {len(batch_rows)} repos: "
            f"{batch_status.get('complete', 0)} completed, "
            f"{batch_status.get('failed', 0)} failed, and "
            f"{batch_timeouts} timed out at the configured per-repo boundary."
        )

    lines = [
        "# Completion Audit for Original Research003 Objective",
        "",
        "This audit checks the current worktree against the original pasted objective, using current files and command evidence. It intentionally does not redefine completion around the work already done.",
        "",
        "## Verdict",
        "",
        "Status: partially complete, not yet proven complete. The five requested corpus files exist and are substantial. Broad repository text/path scans cover the 609-repo manifest. However, the explicit requirement to browse each and every repository with `codegraphcontext-evidence-reader` is not satisfied by current evidence, and the five originally spawned parallel agents did not produce the files.",
        "",
        batch_note,
        "",
        "## Requirement Audit Table",
        "",
        "| Requirement | Evidence Inspected | Status | Notes |",
        "|---|---|---|---|",
        "| Read and follow original pasted objective | `/Users/amuldotexe/.codex/attachments/1fd285ac-9a71-4644-9a10-68e78a09b8c7/pasted-text-1.txt`; `progress-journal.md`. | achieved | Objective re-read and tracked through journal checkpoints. |",
        f"| Use `codegraphcontext-evidence-reader` | Skill instructions read; `cgc-run-summary.tsv` has {len(cgc_rows)} run dirs; `cgc-batch-status.tsv` has {len(batch_rows)} latest bounded attempts. | partial | Skill was used/attempted, but not successfully for every repo. |",
        f"| Browse each and every repo inside `git-ref-repo` with CGC | `repo-manifest.txt` has {len(manifest)} repos; `repo-coverage-index.md` shows {len(any_cgc)} repos with any CGC run and {len(complete_cgc)} with complete artifacts. | not achieved | CGC all-repo browsing is the largest remaining gap. |",
        "| Search across all repositories for Tree-sitter/parser/query/indexing/testing/architecture patterns | `repo-evidence-index.tsv`, `/tmp/parseltongue-ts-rg-v2.txt`, `/tmp/parseltongue-api-evidence.txt`, `/tmp/parseltongue-grammar-files.txt`. | partial | Broad scans cover all repos; direct source inspection is curated, not exhaustive per repo. |",
        "| Do not limit evidence to Rust | Five corpus files cite Python, Swift, Rust, TypeScript/JavaScript, editor integrations, graph/indexing systems, benchmarks, tests. | achieved | Cross-language evidence is present. |",
        "| Create exactly five named files in `docs/research003` | `tree-sitter-patterns-1.md` through `tree-sitter-patterns-5.md`. | achieved | All five requested file names exist. |",
        "| Use five parallel agents to write the files | Progress journal notes spawned workers did not produce files; local synthesis completed the deliverables. | not achieved as requested | The final artifacts exist, but provenance differs from request. |",
        "| Each agent/file covers a distinct slice with minimal duplication | File titles and contents split runtime, grammar, repo indexing, LLM companion, and verification/hardening. | achieved | Thematic split is clear. |",
        "| Use `tdd-task-progress-context-retainer` to track progress, evidence, repo coverage, gaps | Progress journal exists and was updated; skill refs read; latest checkpoints record CGC and worker gaps. | achieved | Journal tracks state and gaps. |",
        "| Optimize for high recall, concrete examples, repo evidence, tradeoffs, Rust translations | Each corpus file contains pattern sections with where-found evidence and Rust translations. | substantially achieved | Quality is high but direct per-repo CGC coverage is not exhaustive. |",
        "| For every meaningful pattern, capture pattern metadata and agent guidance where possible | Pattern sections include `Where found`, why it matters, Rust translation, risks, testing, and agent guidance. | substantially achieved | Not every possible field is present on every pattern, but most are represented. |",
        "| Final result helps answer how to build Parseltongue as a reliable multi-language Tree-sitter LLM companion | Five corpus files collectively cover parser runtime, grammar/query layers, indexing, LLM context, and verification. | achieved for current corpus | Primary knowledge artifact is useful now. |",
        "",
        "## Concrete Remaining Work for Full Completion",
        "",
        "1. Continue `cgc_batch_runner.py` across all 609 repos with a conservative timeout and resume enabled; the runner now provides process-group timeout, per-repo status persistence, and skip/resume behavior.",
        "2. Review `cgc-batch-status.tsv` after a full run, or explicitly document that CGC cannot handle the corpus and obtain user acceptance for text/path evidence as fallback.",
        "3. If strict provenance matters, replace the local-synthesis note with actual successful parallel-agent-authored files, or document the deviation as accepted.",
        "4. Optionally deepen direct source citations for top API-evidence repositories not yet represented in the five files.",
    ]
    AUDIT_PATH.write_text("\n".join(lines).rstrip() + "\n")


def main() -> int:
    manifest = read_lines(MANIFEST_PATH)
    cgc_rows = summarize_cgc_runs()
    write_tsv(CGC_SUMMARY_PATH, CGC_FIELDNAMES, cgc_rows)

    evidence_rows = read_tsv(EVIDENCE_PATH)
    if not evidence_rows:
        raise SystemExit(f"missing evidence rows: {EVIDENCE_PATH}")
    fieldnames = list(evidence_rows[0].keys())
    updated_rows, any_cgc, complete_cgc, incomplete_cgc = update_evidence_cgc_columns(
        evidence_rows,
        cgc_rows,
        manifest,
    )
    write_tsv(EVIDENCE_PATH, fieldnames, updated_rows)

    batch_rows = read_tsv(BATCH_STATUS_PATH)
    write_coverage_index(
        updated_rows,
        manifest,
        cgc_rows,
        batch_rows,
        any_cgc,
        complete_cgc,
        incomplete_cgc,
    )
    write_completion_audit(manifest, cgc_rows, batch_rows, any_cgc, complete_cgc)

    batch_counts = Counter(row.get("status", "") for row in batch_rows)
    print(f"cgc_rows={len(cgc_rows)}")
    print(f"manifest_cgc_repos={len(any_cgc)}")
    print(f"manifest_complete_repos={len(complete_cgc)}")
    print(f"batch_rows={len(batch_rows)}")
    print(f"batch_status={dict(sorted(batch_counts.items()))}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
