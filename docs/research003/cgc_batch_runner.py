#!/usr/bin/env python3
"""Run CodeGraphContext smoke indexing across a repo manifest.

This is a research003 utility, not production Parseltongue code.

Why this exists:
- The original research objective asked for CGC browsing of every repo.
- Ad-hoc CGC runs left incomplete processes and partial output.
- This runner records status after each repo, supports resume, and kills the
  whole process group when a timeout is reached.

Example:
    python3 docs/research003/cgc_batch_runner.py \
      --manifest docs/research003/repo-manifest.txt \
      --status docs/research003/cgc-batch-status.tsv \
      --timeout-seconds 90 \
      --limit 5
"""

from __future__ import annotations

import argparse
import csv
import os
import re
import signal
import subprocess
import sys
import time
from pathlib import Path


DEFAULT_WRAPPER = Path(
    "/Users/amuldotexe/.codex/skills/codegraphcontext-evidence-reader/scripts/"
    "scan_current_repo_only.sh"
)

FIELDNAMES = [
    "repo",
    "status",
    "returncode",
    "seconds",
    "outdir",
    "stdout_tail",
    "stderr_tail",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--manifest",
        type=Path,
        required=True,
        help="Path to repo-manifest.txt",
    )
    parser.add_argument(
        "--status",
        type=Path,
        required=True,
        help="TSV status file to append/update.",
    )
    parser.add_argument(
        "--wrapper",
        type=Path,
        default=DEFAULT_WRAPPER,
        help="CGC repo-scoped wrapper script.",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=90.0,
        help="Per-repo wall-clock timeout.",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Optional max number of repos to attempt this invocation.",
    )
    parser.add_argument(
        "--retry-failed",
        action="store_true",
        help="Retry repos already recorded as failed/timeout/interrupted.",
    )
    return parser.parse_args()


def read_manifest(path: Path) -> list[str]:
    return [line.strip() for line in path.read_text().splitlines() if line.strip()]


def read_existing_status(path: Path) -> dict[str, dict[str, str]]:
    if not path.exists():
        return {}

    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        return {row["repo"]: row for row in reader if row.get("repo")}


def write_all_status(path: Path, rows: dict[str, dict[str, str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = path.with_suffix(path.suffix + ".tmp")
    with tmp_path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=FIELDNAMES, delimiter="\t")
        writer.writeheader()
        for repo in sorted(rows):
            writer.writerow(rows[repo])
    tmp_path.replace(path)


def extract_outdir(text: str) -> str:
    matches = re.findall(r"/tmp/codex-code-intel/codegraphcontext/[^\s]+", text)
    return matches[-1] if matches else ""


def tail_for_tsv(text: str, limit: int = 700) -> str:
    return text.replace("\t", " ").replace("\n", " | ")[-limit:]


def terminate_process_group(process: subprocess.Popen[str]) -> None:
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            return
        process.wait(timeout=5)


def run_one_repo(wrapper: Path, repo: str, timeout_seconds: float) -> dict[str, str]:
    started = time.monotonic()
    process = subprocess.Popen(
        [str(wrapper), repo],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        start_new_session=True,
    )

    stdout = ""
    stderr = ""
    status = "unknown"
    returncode = ""
    try:
        stdout, stderr = process.communicate(timeout=timeout_seconds)
        returncode = str(process.returncode)
        status = "complete" if process.returncode == 0 else "failed"
    except subprocess.TimeoutExpired:
        terminate_process_group(process)
        stdout, stderr = process.communicate()
        returncode = "timeout"
        status = f"timeout_{int(timeout_seconds)}s"
    except KeyboardInterrupt:
        terminate_process_group(process)
        stdout, stderr = process.communicate()
        returncode = "interrupted"
        status = "interrupted"
        raise

    combined = stdout + "\n" + stderr
    return {
        "repo": repo,
        "status": status,
        "returncode": returncode,
        "seconds": f"{time.monotonic() - started:.1f}",
        "outdir": extract_outdir(combined),
        "stdout_tail": tail_for_tsv(stdout),
        "stderr_tail": tail_for_tsv(stderr),
    }


def should_skip(
    repo: str,
    existing: dict[str, dict[str, str]],
    retry_failed: bool,
) -> bool:
    prior = existing.get(repo)
    if not prior:
        return False
    if prior.get("status") == "complete":
        return True
    return not retry_failed


def main() -> int:
    args = parse_args()
    if not args.wrapper.exists():
        print(f"wrapper not found: {args.wrapper}", file=sys.stderr)
        return 2

    repos = read_manifest(args.manifest)
    status_rows = read_existing_status(args.status)
    attempted = 0

    for repo in repos:
        if should_skip(repo, status_rows, args.retry_failed):
            continue
        if args.limit is not None and attempted >= args.limit:
            break

        attempted += 1
        print(f"[{attempted}] CGC indexing {repo}", flush=True)
        try:
            row = run_one_repo(args.wrapper, repo, args.timeout_seconds)
        except KeyboardInterrupt:
            write_all_status(args.status, status_rows)
            raise
        status_rows[repo] = row
        write_all_status(args.status, status_rows)
        print(
            f"    {row['status']} rc={row['returncode']} "
            f"seconds={row['seconds']} outdir={row['outdir']}",
            flush=True,
        )

    print(f"attempted={attempted} status_file={args.status}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
