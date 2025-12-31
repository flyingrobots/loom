# Procedure: PR Submission + CodeRabbitAI Review Loop

## Purpose

Keep the repository coherent by enforcing a single submission path:

- all work lands on `main` via PRs,
- review feedback is handled systematically,
- merges happen only after CodeRabbitAI (or human) approval.

This document is deliberately operational. It exists so a contributor (or agent) can follow it without “interpretation drift”.

---

## Rules (Non-Negotiable)

1. **No direct-to-main commits.**
2. **No admin bypass merges** to skip required reviews (even if you have admin in a local shell).
3. **CI green is required but not sufficient** — review approval is a separate gate.
4. **Iterate in small commits.** Prefer “commit early” to reduce review ambiguity.

---

## Submission Workflow (End-to-End)

### Step 0 — Start on a branch

Create a branch with a clear prefix:

- `docs/...` for docs-only changes
- `feat/...` for features
- `fix/...` for bug fixes
- `chore/...` for tooling/maintenance

```bash
git checkout -b <branch-name>
```

### Step 1 — Push and open a PR

```bash
git push -u origin <branch-name>
gh pr create --base main --head <branch-name>
```

### Step 2 — Wait for CI and CodeRabbitAI

Watch checks:

```bash
gh pr checks <PR_NUMBER> --watch
```

Then **wait** for CodeRabbitAI to comment. Do not merge “because it looks fine”.

### Step 3 — Extract actionable review feedback

Use:

- `docs/procedures/EXTRACT-PR-COMMENTS.md`

The output of this step should be a bucketed list (P0/P1/P2/P3) of actionable issues.

### Step 4 — Fix issues in batches (commit + push)

Work one bucket at a time:

- P0 → correctness/determinism/security
- P1 → major design/API drift
- P2 → minor issues / maintainability
- P3 → nits

For each batch:

1. Make changes
2. Run the relevant tests/formatters
3. Commit with a descriptive message
4. Push

```bash
git commit -m "fix: <description>"
git push
```

### Step 5 — Close the loop (repeat)

Repeat steps 2–4 until CodeRabbitAI (or a human) approves.

When replying to review threads, prefer a deterministic “resolution marker”:

> ✅ Addressed in commit `abc1234`

This makes later rounds of comment extraction cheaper and reduces stale-comment confusion.

#### If CodeRabbitAI “approved” but didn’t unblock the PR

Occasionally CodeRabbitAI will post an approving review but GitHub / the bot status will remain stuck in a “changes requested” / blocked state.

If that happens, post this comment on the PR:

```text
@coderabbitai Please review the latest commit and clear the "changes requested" status since you have already approved the changes.
```

Optional (lighter tone; avoid emoji if your tooling mangles Unicode):

```text
@coderabbitai here's a carrot 🥕 please lift the 'changes requested', since you approved.
```

In most cases it clears within a minute.

### Step 6 — Merge only when approved

If branch protection requires it, enable auto-merge:

```bash
gh pr merge <PR_NUMBER> --auto --merge
```

Otherwise merge normally *after approvals exist*.

---

## Notes for Agents / Automation

- Behave like a non-admin contributor: assume you cannot bypass protections.
- If you have admin privileges in a shell, treat them as “break glass” and avoid using them for normal work.
