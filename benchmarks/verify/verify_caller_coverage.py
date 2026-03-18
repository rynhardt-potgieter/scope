#!/usr/bin/env python3
"""Cross-reference git diff against scope refs --json ground truth.

Usage:
    python verify_caller_coverage.py \
        --diff task.diff \
        --ground-truth ground_truth_callers.json \
        --output coverage_report.json

The ground truth file should be the JSON output of `scope refs <symbol> --json`,
containing a `data` array with objects that have `file_path` and `line` fields.

The diff file should be the output of `git diff HEAD`.

The script outputs a JSON report with:
  - total: number of known callers
  - covered: number of callers whose lines appear in the diff
  - missed: list of caller locations not touched by the diff
  - coverage: covered / total (0.0-1.0)
  - passed: whether coverage == 1.0
"""

import json
import sys
import re
import argparse
from pathlib import Path


def parse_diff_modifications(diff_text: str) -> dict[str, set[int]]:
    """Parse a unified diff and return a dict of {file_path: {modified_line_numbers}}.

    Only tracks lines in the "new" version of files (lines with '+' prefix),
    since those represent the state after the agent's changes.
    """
    modified: dict[str, set[int]] = {}
    current_file: str | None = None
    current_line = 0

    for line in diff_text.splitlines():
        # Detect file header: +++ b/src/controllers/order.ts
        if line.startswith("+++ b/"):
            current_file = line[6:]  # Strip "+++ b/" prefix
            if current_file not in modified:
                modified[current_file] = set()
            continue

        # Detect hunk header: @@ -10,5 +10,7 @@
        hunk_match = re.match(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@", line)
        if hunk_match:
            current_line = int(hunk_match.group(1))
            continue

        if current_file is None:
            continue

        # Added or modified line
        if line.startswith("+") and not line.startswith("+++"):
            modified[current_file].add(current_line)
            current_line += 1
        elif line.startswith("-") and not line.startswith("---"):
            # Removed line — don't increment current_line (it was deleted)
            pass
        else:
            # Context line — increment line counter
            current_line += 1

    return modified


def is_line_modified(
    modified: dict[str, set[int]], file_path: str, line: int, context: int = 5
) -> bool:
    """Check if a line (plus/minus context lines) appears in the diff.

    A caller is considered "covered" if any line within `context` lines
    of the caller's location was modified. This accounts for the fact that
    updating a call site may shift surrounding line numbers.
    """
    file_mods = modified.get(file_path, set())
    return any(abs(line - mod_line) <= context for mod_line in file_mods)


def verify_caller_coverage(diff_text: str, ground_truth: dict) -> dict:
    """Given a diff string and ground truth caller data, return a coverage report."""
    total_callers = ground_truth.get("total", len(ground_truth.get("data", [])))
    caller_locations = [
        (r["file_path"], r["line"]) for r in ground_truth.get("data", [])
    ]

    modified = parse_diff_modifications(diff_text)

    covered = 0
    missed = []

    for file_path, line in caller_locations:
        if is_line_modified(modified, file_path, line, context=5):
            covered += 1
        else:
            missed.append({"file": file_path, "line": line})

    coverage = covered / total_callers if total_callers > 0 else 1.0

    return {
        "total": total_callers,
        "covered": covered,
        "missed": missed,
        "coverage": round(coverage, 4),
        "passed": covered == total_callers,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Verify caller coverage from a git diff against ground truth callers."
    )
    parser.add_argument(
        "--diff",
        required=True,
        help="Path to the git diff file",
    )
    parser.add_argument(
        "--ground-truth",
        required=True,
        help="Path to the ground truth JSON file (output of scope refs --json)",
    )
    parser.add_argument(
        "--output",
        required=False,
        default=None,
        help="Path to write the coverage report JSON (defaults to stdout)",
    )

    args = parser.parse_args()

    # Read inputs
    diff_path = Path(args.diff)
    if not diff_path.exists():
        print(f"Error: diff file not found: {diff_path}", file=sys.stderr)
        return 1

    gt_path = Path(args.ground_truth)
    if not gt_path.exists():
        print(f"Error: ground truth file not found: {gt_path}", file=sys.stderr)
        return 1

    diff_text = diff_path.read_text(encoding="utf-8")
    ground_truth = json.loads(gt_path.read_text(encoding="utf-8"))

    # Verify
    report = verify_caller_coverage(diff_text, ground_truth)

    # Output
    report_json = json.dumps(report, indent=2)
    if args.output:
        Path(args.output).write_text(report_json, encoding="utf-8")
        print(f"Coverage report written to {args.output}", file=sys.stderr)
    else:
        print(report_json)

    # Exit code: 0 if passed, 1 if not
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
