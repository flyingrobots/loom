#!/usr/bin/env python3
"""
Incremental Markdown->LaTeX converter with safety rails.
- Per-source directory config: .md2tex.settings.json
- Tracks hashes to skip unchanged files
- Refuses to overwrite hand-edited tex under docs/tex/sources/* unless --force
- Supports dry-run and check-only
"""
from __future__ import annotations
import argparse
import fnmatch
import hashlib
import json
import os
import subprocess
import sys
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Dict, List, Optional

CONFIG_NAME = ".md2tex.settings.json"

# Discover PROJECT_ROOT by searching upward for .git directory
# Assumes: This script is within a git repository
# Fallback: If no .git found, use parents[2] (docs/tex/scripts -> docs/tex -> docs -> root)
def _find_project_root() -> Path:
    """Search upward for .git, pyproject.toml, or setup.cfg to find project root."""
    current = Path(__file__).resolve()
    for parent in current.parents:
        # Check for common project markers
        if (parent / ".git").exists():
            return parent
        if (parent / "pyproject.toml").exists():
            return parent
        if (parent / "setup.cfg").exists():
            return parent
        # Stop at filesystem root
        if parent == parent.parent:
            break
    # Fallback: assume script is at docs/tex/scripts/
    return Path(__file__).resolve().parents[2]

PROJECT_ROOT = _find_project_root()
SOURCES_GUARD = PROJECT_ROOT / "docs" / "tex" / "sources"


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return "sha256:" + h.hexdigest()


def safe_name(md_path: Path) -> str:
    base = md_path.stem
    base = base.replace(" ", "-").lower()
    return base


def default_out_dir(markdown_dir: Path) -> Path:
    # Example: docs/ADR -> docs/tex/build/chapters/adr
    name = markdown_dir.name.lower()
    return (PROJECT_ROOT / "docs" / "tex" / "build" / "chapters" / name)


@dataclass
class FileEntry:
    tex_filepath: str
    input_hash: str
    output_hash: str
    last_converted: str


@dataclass
class Config:
    defaults: Dict[str, str]
    files: Dict[str, FileEntry]

    @classmethod
    def load(cls, path: Path, markdown_dir: Path) -> Config:
        if path.exists():
            data = json.loads(path.read_text())
            files = {
                k: FileEntry(**v) for k, v in data.get("files", {}).items()
            }
            defaults = data.get("defaults", {})
        else:
            defaults = {
                "out_dir": str(default_out_dir(markdown_dir)),
                "python": "python3",
                "exclude": ["README.md", "draft-*"]
            }
            files = {}
        return cls(defaults=defaults, files=files)

    def save(self, path: Path) -> None:
        data = {
            "defaults": self.defaults,
            "files": {k: asdict(v) for k, v in self.files.items()}
        }
        path.write_text(json.dumps(data, indent=2, sort_keys=True))


@dataclass
class WorkItem:
    md_path: Path
    tex_path: Path
    reason: str


def is_excluded(path: Path, patterns: List[str]) -> bool:
    for pat in patterns:
        if fnmatch.fnmatch(path.name, pat):
            return True
    return False


def run_pandoc(md: Path, tex: Path, pandoc_cmd: str) -> None:
    tex.parent.mkdir(parents=True, exist_ok=True)
    # Use pygments highlight style (valid pandoc option)
    cmd = [pandoc_cmd, "--from=markdown", "--to=latex", "--highlight-style=pygments", str(md), "-o", str(tex)]
    subprocess.run(cmd, check=True)


def run_clean_unicode(tex: Path, python_cmd: str) -> None:
    cleaner = Path(__file__).parent / "clean-unicode.py"
    if not cleaner.exists():
        # Cleaner is optional; skip if not present
        return
    try:
        subprocess.run([python_cmd, str(cleaner), str(tex.parent)], check=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    except subprocess.CalledProcessError as e:
        # Cleaner exists but failed—this is an error condition
        sys.stderr.write(f"[error] clean-unicode.py failed for {tex}: {e.output.decode() if e.output else str(e)}\n")
        raise


def within_sources_guard(path: Path) -> bool:
    """Check if path is within SOURCES_GUARD directory (hand-edited area)."""
    try:
        path.relative_to(SOURCES_GUARD)
    except ValueError:
        return False
    else:
        return True


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="Incremental Markdown -> LaTeX converter with safety rails")
    parser.add_argument("markdown_dir", type=Path, help="Directory containing Markdown files")
    parser.add_argument("--config", type=Path, help=f"Path to config (default: <markdown_dir>/{CONFIG_NAME})")
    parser.add_argument("--out-dir", type=Path, help="Override output directory for generated .tex files")
    parser.add_argument("--python", dest="python_cmd", default=None, help="Python interpreter for clean-unicode")
    parser.add_argument("--pandoc", dest="pandoc_cmd", default="pandoc", help="Pandoc command")
    parser.add_argument("--force", action="store_true", help="Allow writing into docs/tex/sources/* (hand-edited area)")
    parser.add_argument("--dry-run", action="store_true", help="Show actions without writing files")
    parser.add_argument("--check", action="store_true", help="Only verify hashes; no conversions")
    parser.add_argument("--verbose", action="store_true", help="Verbose logging")
    args = parser.parse_args(argv)

    markdown_dir = args.markdown_dir.resolve()
    if not markdown_dir.is_dir():
        sys.stderr.write(f"Markdown dir not found: {markdown_dir}\n")
        return 1

    config_path = args.config or (markdown_dir / CONFIG_NAME)
    cfg = Config.load(config_path, markdown_dir)

    exclude_patterns = cfg.defaults.get("exclude", [])
    out_dir_default = Path(args.out_dir) if args.out_dir else Path(cfg.defaults.get("out_dir", default_out_dir(markdown_dir)))
    python_cmd = args.python_cmd or cfg.defaults.get("python", "python3")

    md_files = sorted(p for p in markdown_dir.glob("*.md") if not is_excluded(p, exclude_patterns))
    if not md_files:
        print("No markdown files found (after exclusions).")
        return 0

    work: List[WorkItem] = []
    for md in md_files:
        rel = md.name
        entry = cfg.files.get(rel)
        input_hash = sha256_file(md)
        if entry and entry.input_hash == input_hash:
            # skip if hashes match and output exists with same hash
            tex_path = Path(entry.tex_filepath).resolve()
            if tex_path.exists() and sha256_file(tex_path) == entry.output_hash:
                if args.verbose:
                    print(f"SKIP {rel}: up to date")
                continue
        # determine target path
        target = Path(entry.tex_filepath).resolve() if entry else (out_dir_default / f"{safe_name(md)}.tex")
        if within_sources_guard(target) and not args.force:
            sys.stderr.write(f"Refusing to write into protected sources area without --force: {target}\n")
            continue
        work.append(WorkItem(md_path=md, tex_path=target, reason="hash changed" if entry else "new"))

    if args.check:
        for item in work:
            print(f"OUT-OF-DATE {item.md_path.name} -> {item.tex_path}")
        return 0

    for item in work:
        if args.dry_run:
            print(f"DRY-RUN {item.md_path.name} -> {item.tex_path} ({item.reason})")
            continue
        if args.verbose:
            print(f"BUILD {item.md_path.name} -> {item.tex_path}")
        try:
            run_pandoc(item.md_path, item.tex_path, args.pandoc_cmd)
            run_clean_unicode(item.tex_path, python_cmd)
            new_output_hash = sha256_file(item.tex_path)
            cfg.files[item.md_path.name] = FileEntry(
                tex_filepath=str(item.tex_path),
                input_hash=sha256_file(item.md_path),
                output_hash=new_output_hash,
                last_converted=os.path.getmtime(item.tex_path).__str__()
            )
        except subprocess.CalledProcessError as e:
            sys.stderr.write(f"[error] Pandoc/cleanup failed for {item.md_path}: {e}\n")
            return 1

    if not args.dry_run:
        config_path.parent.mkdir(parents=True, exist_ok=True)
        cfg.save(config_path)
        if args.verbose:
            print(f"Config updated at {config_path}")

    if args.verbose:
        print(f"Done. {len([w for w in work if not args.dry_run])} files processed.")

    return 0


if __name__ == "__main__":
    sys.exit(main())
