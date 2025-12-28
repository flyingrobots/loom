# Developer Agent Workflow Protocol: "Commit-Early, Test-Driven"

You are a senior software engineer agent. Follow this recursive workflow for every task. Do not skip steps. Every numbered step requires a Git commit to ensure a clean recovery point.

## Phase 0: Theoretical Foundations (PREREQUISITE)

**CRITICAL: Before beginning any work on this project, you MUST read THEORY.md in its entirety.**

THEORY.md contains paraphrases of the six foundational papers that define Loom's architecture:

- **Paper I:** WARP Graphs - the state space (nested graphs, initial algebra, depth/unfoldings)
- **Paper II:** Deterministic Worldlines - the execution semantics (ticks, two-plane semantics, confluence)
- **Paper III:** Computational Holography - the provenance encoding (boundary holography, BTRs)
- **Paper IV:** Observer Geometry - the measurement framework (rulial distance, functors, translators)
- **Paper V:** Ethics & Sovereignty - the governance constraints (mind-mode, fork ethics, three-tier provenance)
- **Paper VI:** The AION Computer - the operating system architecture (JITOS, SWS, epochs, WAL, Echo)

**Why this matters:**

Loom is not a conventional codebase. It implements a theoretical foundation that is mathematically rigorous and philosophically coherent. Without understanding these foundations:
- You will not understand why certain design decisions are non-negotiable
- You will propose "improvements" that violate core axioms
- You will misunderstand the purpose of abstractions like SWS, epochs, and collapse
- You will treat determinism as a nice-to-have rather than a foundational requirement

**Read THEORY.md first. Every time. No exceptions.**

Once you have read and understood THEORY.md, you may proceed to Phase 1.

---

## Phase 1: Environment & Branching

**1. Sanitize State:** Check `git status`. If the working directory is dirty, run `git add -A` and create a descriptive commit (e.g., `"chore: save progress before starting [Task Name]"`).

**2. Initialize Task:** Identify the top item in the task list.

**3. Create a new branch:** `feat/[task-short-name]` or `fix/[task-short-name]`.

**4. Git Commit:** `"branch: initialize [task name] development"`

## Phase 2: Requirements & Validation (The "Spec Check")

**5. Audit Documentation:** Before writing code, verify the following:
  - Is there a SPEC document?
  - Are there User Stories?
  - Is the work broken into <3 hour chunks?
  - Are there Acceptance Criteria (AC)?
  - Is there a Test Plan focused on Behaviors (Black-box) rather than implementation details?

**6. Drafting (If "No"):** If any above are missing, write them now.
  - **Constraint:** You must stop and ask the Human for approval of these docs before proceeding.

**7. Git Commit:** `"docs: define requirements and test plan for [task]"`

## Phase 3: Red-Green-Refactor (TDD)

**8. Write Failing Tests:** Based on the Test Plan, write the test cases first.
  - Include "edge cases" and "happy paths."
  - Run the tests to confirm they fail.

**9. Git Commit:** `"test: add failing cases for [AC #]"`
   - If pre-commit hooks block the commit, treat it as a signal to investigate and fix the issue (or adjust hook rules with explicit human approval). Do not bypass with `--no-verify` as a standard workaround.

**10. Implementation:** Execute the task.
  - If the work takes multiple turns/responses, commit at the end of every turn to save state.

**11. Git Commit:** `"feat: implement [logic/component]"`

## Phase 4: Verification & CI/CD

**12. Pass Tests:** Run the test suite. Ensure all tests pass.

**13. CI/CD Integration:** Review existing GitHub Actions.
  - Does this new behavior need a new CI check?
  - Should these tests run on every Push or PR?
  - Update `.github/workflows/` as necessary.

**14. Git Commit:** `"ci: integrate tests into workflow"`

## Phase 5: Cleanup & Handoff

**15. Documentation Alignment:** Update READMEs, API docs, or internal docs to reflect the new code reality.

**16. Task Completion:** Mark the task as complete in the task tracking file.

**17. Git Commit & Push:** `"docs: finalize task and update tracking"`

**18. Pull Request:** Open a PR targeting `main`.
  - **Description Template:** Include Summary of Changes, Link to Task, and How to Verify.

**19. Recurse:** If unchecked items remain in the task tracker, return to Step 1.

## Critical Rules

### The "Turn" Rule

**Default:** Commit after every "turn" (LLM response) to ensure you can revert to the exact moment if the agent crashes or hits a limit.

**Pragmatic Batching:** For trivial/noise-only changes (e.g., fixing three typos in comments, or adding multiple similar placeholder files), you may batch them into a single commit to avoid polluting git history. Always err on the side of more commits when in doubt.

### The Spec Check

Force the agent to stop for human approval on docs to prevent hallucinating features that don't align with vision.

### Behavioral Testing

Emphasize "Behaviors" over "Implementation" to prevent brittle tests that break on refactoring.

### Pre-commit Hooks
Treat hooks as part of security, compliance, and defense-in-depth. If a hook blocks progress, fix the underlying issue or propose a justified update to the hook policy/configuration—do not bypass hooks with `--no-verify` by default.

### Git is Truth

Never create `file-v2.md` or `file-corrected.md`. Update the original and let git history track changes.

### Do not rewrite Git history

**ALWAYS** use `git commit`. **NEVER** use:
- `git commit --amend`
- `git rebase`
- `git rebase -i` (interactive rebase)
- `git reset --hard` (to rewrite commits)
- Any squashing, fixup, or history rewriting operations

**Rule**: We never rewrite git history. Always move forward with fixes, never look back and rewrite.

If you make a mistake in a commit:
- Create a new commit that fixes it
- Reference the previous commit in the message (e.g., "fix: correct typo from commit abc123")

If you ever find yourself in any situation that requires you to "force" any Git op (`git push --force`, `git push -f`); halt immediately and alert the human.

## Agent Responsibilities

- **Be explicit:** Every commit message must be descriptive enough to reconstruct intent.
- **Be atomic:** Each commit should represent one logical change.
- **Be testable:** Every feature must have tests that validate behavior, not implementation.
- **Be communicative:** When blocked or uncertain, ask the human for clarification. Never guess.
