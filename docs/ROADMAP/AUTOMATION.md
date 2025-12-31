# Roadmap DAG Automation

This repo treats Mermaid DAG styling in `docs/ROADMAP/**/README.md` as **derived output**.

Source of truth is the milestone checklists:
- each milestone’s `## <n>. Task Checklist (Inline)` section, and
- its `- [ ]` / `- [x]` checkbox progress.

The updater script derives:
- milestone status in `docs/ROADMAP/README.md` (nodes `M1..M7`)
- phase status in each milestone README (nodes `P0..Pn` + `Gate`)

## Tools

### Python updater (fast path)

- Update files in-place: `python3 scripts/update_roadmap_dags.py`
- Check for drift (CI-style): `python3 scripts/update_roadmap_dags.py --check`

### Rust `xtask` wrapper (optional)

This repository’s Rust workspace may contain stub crates while milestones are being planned.
To keep tooling runnable, `xtask/` is a standalone Cargo workspace.

- Update files in-place: `cargo run --manifest-path xtask/Cargo.toml -- roadmap-dags`
- Check for drift: `cargo run --manifest-path xtask/Cargo.toml -- roadmap-dags --check`

## Pre-commit hook (recommended)

There is a version-controlled hook at `.githooks/pre-commit` which:
- runs the updater **only when** a `docs/ROADMAP/**/README.md` file is staged, and
- stages any resulting DAG styling updates.

Enable it once per clone:

- `git config core.hooksPath .githooks`

or via `xtask`:

- `cargo run --manifest-path xtask/Cargo.toml -- install-githooks`

## Contracts / assumptions

The updater expects:
- milestone README includes `## <n>. Task Checklist (Inline)`
- phase headings are `### Phase <num> — ...`
- each milestone README contains a Mermaid block with nodes `P0[` and `Gate[`
- `docs/ROADMAP/README.md` contains a Mermaid block with nodes `M1[` and `M2[`

If you change these headings/node IDs, update `scripts/update_roadmap_dags.py` accordingly.

