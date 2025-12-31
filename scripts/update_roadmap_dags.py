#!/usr/bin/env python3
"""
Update Mermaid DAG node styling for the JITOS roadmap based on markdown checkboxes.

This script reads:
  - docs/ROADMAP/README.md (cross-milestone DAG)
  - docs/ROADMAP/M*/README.md (per-milestone DAGs, if present)

and updates Mermaid `classDef` + `class ...;` assignments so nodes render as:
  - done      (green)
  - inprogress (blue)
  - blocked   (red)

Status is derived from checklist checkbox progress:
  - done: all checkboxes in a group are checked
  - inprogress: any checkbox checked, OR it is the earliest not-done item in a sequence
  - blocked: otherwise
"""

from __future__ import annotations

import argparse
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple

RE_MERMAID_BLOCK = re.compile(r"```mermaid\n(?P<body>.*?)\n```", re.DOTALL)
RE_H2_TASK_CHECKLIST = re.compile(
    r"^##\s+\d+\.\s+Task Checklist\s*\(Inline\)\s*$", re.MULTILINE
)
RE_H3_PHASE = re.compile(
    r"^###\s+Phase\s+(?P<num>\d+)\s+—\s+(?P<title>.+?)\s*$", re.MULTILINE
)
RE_H2_ANY = re.compile(r"^##\s+", re.MULTILINE)
RE_CHECKBOX = re.compile(r"^\s*-\s*\[(?P<mark>[ xX])\]\s+", re.MULTILINE)


@dataclass(frozen=True)
class CheckboxStats:
    checked: int
    total: int

    @property
    def any_checked(self) -> bool:
        return self.checked > 0

    @property
    def all_checked(self) -> bool:
        return self.total > 0 and self.checked == self.total


def _slice_section(text: str, start_re: re.Pattern) -> Optional[str]:
    start = start_re.search(text)
    if not start:
        return None
    start_idx = start.start()
    next_h2 = RE_H2_ANY.search(text, start.end())
    end_idx = next_h2.start() if next_h2 else len(text)
    return text[start_idx:end_idx]


def _count_checkboxes(text: str) -> CheckboxStats:
    marks = RE_CHECKBOX.findall(text)
    checked = sum(1 for m in marks if m.strip().lower() == "x")
    return CheckboxStats(checked=checked, total=len(marks))


def parse_phase_checkbox_stats(markdown: str) -> List[Tuple[int, str, CheckboxStats]]:
    """
    Returns a list of (phase_number, phase_title, checkbox_stats) in phase order.
    """
    section = _slice_section(markdown, RE_H2_TASK_CHECKLIST)
    if section is None:
        return []

    phases: List[Tuple[int, str, CheckboxStats]] = []
    matches = list(RE_H3_PHASE.finditer(section))
    for i, m in enumerate(matches):
        phase_num = int(m.group("num"))
        phase_title = m.group("title").strip()
        seg_start = m.end()
        seg_end = matches[i + 1].start() if i + 1 < len(matches) else len(section)
        seg = section[seg_start:seg_end]
        phases.append((phase_num, phase_title, _count_checkboxes(seg)))

    phases.sort(key=lambda x: x[0])
    return phases


def compute_sequenced_statuses(stats: List[CheckboxStats]) -> List[str]:
    """
    Compute statuses for a sequential list of items.

    Rules:
      - done if all checked
      - inprogress if any checked
      - otherwise blocked, except the earliest not-done item becomes inprogress
    """
    if not stats:
        return []

    statuses = []
    for s in stats:
        if s.all_checked:
            statuses.append("done")
        elif s.any_checked:
            statuses.append("inprogress")
        else:
            statuses.append("blocked")

    # Promote the first not-done item to inprogress (so work can "start" without a checkbox tick).
    for i, status in enumerate(statuses):
        if status != "done":
            if status == "blocked":
                statuses[i] = "inprogress"
            break

    return statuses


def _normalize_mermaid(body: str) -> List[str]:
    lines = body.splitlines()
    # Remove existing class lines to allow deterministic rewrite.
    filtered = []
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("classDef "):
            continue
        if stripped.startswith("class "):
            continue
        filtered.append(line.rstrip())
    # Trim trailing blank lines
    while filtered and filtered[-1].strip() == "":
        filtered.pop()
    return filtered


def _append_style_lines(lines: List[str], assignments: Dict[str, str]) -> List[str]:
    out = list(lines)
    out.append("")
    out.append("  classDef done fill:#dcfce7,stroke:#166534,color:#052e16,stroke-width:2px;")
    out.append("  classDef inprogress fill:#dbeafe,stroke:#1d4ed8,color:#1e3a8a,stroke-width:2px;")
    out.append("  classDef blocked fill:#fee2e2,stroke:#b91c1c,color:#7f1d1d,stroke-width:2px;")
    out.append("")
    for node_id, status in assignments.items():
        out.append(f"  class {node_id} {status};")
    return out


def _replace_mermaid_block(text: str, block_index: int, new_body: str) -> str:
    matches = list(RE_MERMAID_BLOCK.finditer(text))
    m = matches[block_index]
    start, end = m.span("body")
    return text[:start] + new_body + text[end:]


def update_cross_milestone_dag_text(original: str, milestone_status: Dict[str, str]) -> Tuple[str, bool]:
    """Update the MOC Mermaid block that contains M1..M7 nodes."""
    blocks = list(RE_MERMAID_BLOCK.finditer(original))
    if not blocks:
        return original, False

    target_index = None
    for i, b in enumerate(blocks):
        body = b.group("body")
        if "M1[" in body and "M2[" in body:
            target_index = i
            break
    if target_index is None:
        return original, False

    body = blocks[target_index].group("body")
    lines = _normalize_mermaid(body)

    assignments: Dict[str, str] = {}
    for node_id in sorted(milestone_status.keys(), key=lambda s: int(s[1:])):
        assignments[node_id] = milestone_status[node_id]

    new_lines = _append_style_lines(lines, assignments)
    new_body = "\n".join(new_lines)
    updated = _replace_mermaid_block(original, target_index, new_body)

    return updated, updated != original


def update_milestone_phase_dag_text(original: str, milestone_status: str) -> Tuple[str, bool]:
    """Update the per-milestone Mermaid block that contains Phase nodes P0..P4."""
    blocks = list(RE_MERMAID_BLOCK.finditer(original))
    if not blocks:
        return original, False

    phases = parse_phase_checkbox_stats(original)
    if not phases:
        return original, False

    phase_stats = [s for _, _, s in phases]
    if milestone_status == "done":
        phase_statuses = ["done"] * len(phase_stats)
    elif milestone_status == "blocked":
        phase_statuses = ["blocked"] * len(phase_stats)
    else:
        phase_statuses = compute_sequenced_statuses(phase_stats)

    # Map Phase N → node id PN
    assignments: Dict[str, str] = {}
    for (num, _title, _stat), status in zip(phases, phase_statuses):
        assignments[f"P{num}"] = status

    assignments["Gate"] = "done" if milestone_status == "done" else "blocked"

    target_index = None
    for i, b in enumerate(blocks):
        body = b.group("body")
        if "P0[" in body and "Gate[" in body:
            target_index = i
            break
    if target_index is None:
        return original, False

    body = blocks[target_index].group("body")
    lines = _normalize_mermaid(body)
    new_lines = _append_style_lines(lines, assignments)
    new_body = "\n".join(new_lines)
    updated = _replace_mermaid_block(original, target_index, new_body)

    return updated, updated != original


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=".", help="Path to repo root (default: .)")
    ap.add_argument("--check", action="store_true", help="Do not write files; exit non-zero if changes needed")
    args = ap.parse_args()

    repo = Path(args.repo).resolve()
    docs_roadmap = repo / "docs" / "ROADMAP"
    moc = docs_roadmap / "README.md"
    if not moc.exists():
        raise SystemExit(f"missing {moc}")

    milestone_dirs = sorted(
        [p for p in docs_roadmap.iterdir() if p.is_dir() and re.match(r"^M\d+-", p.name)],
        key=lambda p: int(p.name.split("-", 1)[0][1:]),
    )

    @dataclass(frozen=True)
    class MilestoneProgress:
        num: int
        readme: Path
        phases: List[Tuple[int, str, CheckboxStats]]
        total_stats: CheckboxStats

        @property
        def any_checked(self) -> bool:
            return self.total_stats.any_checked

        @property
        def all_checked(self) -> bool:
            return self.total_stats.all_checked

    milestones: List[MilestoneProgress] = []
    for d in milestone_dirs:
        readme = d / "README.md"
        if not readme.exists():
            continue
        num = int(d.name.split("-", 1)[0][1:])
        md = readme.read_text(encoding="utf-8")
        phases = parse_phase_checkbox_stats(md)
        # Aggregate checkbox totals across all phases.
        checked = sum(s.checked for _, _, s in phases)
        total = sum(s.total for _, _, s in phases)
        milestones.append(
            MilestoneProgress(
                num=num,
                readme=readme,
                phases=phases,
                total_stats=CheckboxStats(checked=checked, total=total),
            )
        )

    milestones.sort(key=lambda m: m.num)

    # Determine "active" milestone: earliest milestone not done, with all previous done.
    active_num: Optional[int] = None
    all_prev_done = True
    for m in milestones:
        if not all_prev_done:
            break
        if not m.all_checked:
            active_num = m.num
            break
        all_prev_done = all_prev_done and m.all_checked

    milestone_status: Dict[str, str] = {}
    for m in milestones:
        if m.all_checked:
            status = "done"
        elif m.any_checked or (active_num is not None and m.num == active_num):
            status = "inprogress"
        else:
            status = "blocked"
        milestone_status[f"M{m.num}"] = status

    changed_files: List[Path] = []

    # Update per-milestone DAGs.
    for m in milestones:
        r = m.readme
        before = r.read_text(encoding="utf-8")
        status = milestone_status.get(f"M{m.num}", "blocked")
        after, changed = update_milestone_phase_dag_text(before, status)
        if changed:
            changed_files.append(r)
            if not args.check:
                r.write_text(after, encoding="utf-8")

    # Update MOC cross-milestone DAG.
    before_moc = moc.read_text(encoding="utf-8")
    after_moc, changed_moc = update_cross_milestone_dag_text(before_moc, milestone_status)
    if changed_moc:
        changed_files.append(moc)
        if not args.check:
            moc.write_text(after_moc, encoding="utf-8")

    if args.check:
        return 1 if changed_files else 0

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
