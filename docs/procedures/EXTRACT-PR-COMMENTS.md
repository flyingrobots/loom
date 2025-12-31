# Procedure: Extract Actionable Comments from CodeRabbitAI PR Reviews

## This Document’s Role in the Submission Workflow (Read This First)

This procedure is **not optional**. It is part of the repo’s expected PR submission workflow.

### Expected Submission Workflow (Branch + PR + Review Loop)

When you finish work:

1. **Do not commit directly to `main`.** Create a branch.
2. **Push the branch** to `origin`.
3. **Open a PR** targeting `main`.
4. **Wait for automated review** (CodeRabbitAI) and CI checks to complete.
5. **Extract actionable review comments** using this document.
6. **Bucket and fix issues** (commit early/often; push updates).
7. **Repeat** steps 4–6 until CodeRabbitAI (or a human reviewer) approves.
8. **Only then merge** the PR (without admin bypass).

### Critical Rules (Enforced by Policy, Even If You Have Admin)

- **No direct-to-main workflow:** even if you *can* push/merge as admin, you should behave like a normal contributor.
- **No admin bypass merges:** do not merge with `--admin` or other override mechanisms to skip required reviews.
- **Don’t “merge because CI is green”:** CI green is necessary, not sufficient. Review approval is a separate gate.

### What “Done” Looks Like for a PR

A PR is mergeable when **all** of the following are true:

- CI checks are green, **AND**
- CodeRabbitAI is satisfied — either approved the PR **OR** has left no unresolved actionable feedback, **AND**
- a human reviewer has approved (if required by repo policy).

If you cannot merge due to branch protection, enable auto-merge (if permitted) and wait:

```bash
gh pr merge <PR_NUMBER> --auto --merge
```

## Purpose
GitHub's review system carries forward comments from earlier commits, making it difficult to identify which issues are truly actionable vs already fixed. This procedure helps extract only the real, unfixed issues.

## Prerequisites
- `gh` CLI installed and authenticated
- `jq` installed for JSON parsing
- Access to the repository and PR

## Procedure

### Step 1: Identify Latest Review

```bash
# Get the latest review ID and commit
gh pr view <PR_NUMBER> --json reviews --jq '.reviews | sort_by(.submittedAt) | last | {id, commit: .commit.oid[0:7], submittedAt}'
```

### Step 2: Fetch All Top-Level Comments

```bash
# Save all comments to temp file for analysis
TMPFILE="/tmp/pr-comments-$(date +%s).json"
gh api repos/<OWNER>/<REPO>/pulls/<PR_NUMBER>/comments > "$TMPFILE"
```

### Step 3: Extract Comments from Latest Commit

```bash
# Get comments associated with latest commit (showing on the current diff)
LATEST_COMMIT="<commit_sha>"
cat "$TMPFILE" | jq --arg commit "$LATEST_COMMIT" '
  .[] |
  select(.in_reply_to_id == null and .commit_id[0:7] == $commit) |
  {
    id,
    line,
    path,
    current_commit: .commit_id[0:7],
    original_commit: .original_commit_id[0:7],
    is_stale: (.commit_id != .original_commit_id),
    created_at,
    body_preview: .body[0:200]
  }
' | jq -s '.' > /tmp/comments-latest.json
```

### Step 4: Identify Stale vs Fresh Comments

```bash
# Group by staleness
cat /tmp/comments-latest.json | jq 'group_by(.is_stale) |
  map({
    category: (if .[0].is_stale then "STALE" else "FRESH" end),
    count: length,
    comments: map({id, line, path, original_commit})
  })'
```

**KEY INSIGHT:** If `is_stale == true`, the comment was created on an earlier commit and **may already be fixed**.

### Step 5: Check for "Already Addressed" Markers

```bash
# Check if comments contain "✅ Addressed in commit" markers
cat "$TMPFILE" | jq '.[] |
  select(.body | contains("✅ Addressed in commit")) |
  {
    id,
    line,
    path,
    fixed_in: (.body | capture("✅ Addressed in commit (?<commit>[a-f0-9]{7})").commit)
  }'
```

**KEY INSIGHT:** CodeRabbitAI sometimes adds "✅ Addressed" markers to its own comments. These are definitely stale.

### Step 6: Categorize by Priority

```bash
# Extract and prioritize actionable comments
cat "$TMPFILE" | jq --arg commit "$LATEST_COMMIT" '
  .[] |
  select(
    .in_reply_to_id == null and
    .commit_id[0:7] == $commit and
    (.body | contains("⚠️") or contains("🧹"))
  ) |
  {
    id,
    line,
    path,
    priority: (
      if (.body | contains("🔴 Critical")) then "P0"
      elif (.body | contains("🟠 Major")) then "P1"
      elif (.body | contains("🟡 Minor")) then "P2"
      else "P3"
      end
    ),
    title: (.body | split("\n")[2:3][0] // "UNTITLED" | gsub("\\*\\*"; "")),
    is_stale: (.commit_id != .original_commit_id),
    body
  }
' | jq -s 'sort_by([
  (.priority | if . == "P0" then 0 elif . == "P1" then 1 elif . == "P2" then 2 else 3 end),
  .path,
  .line
])' > /tmp/prioritized-comments.json
```

### Step 7: Verify Stale Comments (Critical Step)

For each stale comment, verify if it was actually fixed:

```bash
# For each stale comment, check the current code
# Example: Comment claims line 453 is missing something

# 1. Read the current state
git show HEAD:crates/jitos-core/src/events.rs | sed -n '450,460p'

# 2. Search git history for the fix
git log --all --oneline --grep="<issue_keyword>"

# 3. Check if the fix exists
git log -p --all -S"<code_pattern>" -- <file_path>
```

**CRITICAL:** Always verify stale comments by reading the actual current code. Don't trust the "is_stale" flag alone.

### Step 8: Create Issue Report

```bash
cat > /tmp/batch-N-issues.md << 'EOF'
# Batch N - CodeRabbitAI Issues

## Stale (Already Fixed)
- [ ] Line XXX - Issue description (Fixed in: COMMIT_SHA)

## P0 Critical
- [ ] Line XXX - Issue description

## P1 Major
- [ ] Line XXX - Issue description

## P2 Minor
- [ ] Line XXX - Issue description

## P3 Trivial
- [ ] Line XXX - Issue description
EOF
```

### Step 9: Extract Full Comment Bodies for Actionable Issues

```bash
# For each actionable (non-stale) issue, save the full comment
cat /tmp/prioritized-comments.json | jq -r '.[] |
  select(.is_stale == false) |
  "# Comment ID: \(.id) - Line \(.line)\n\(.body)\n\n---\n\n"
' > /tmp/batch-N-full-comments.txt
```

## Common Pitfalls

### Pitfall 1: Trusting GitHub's "Changes Requested" Status
**Issue:** GitHub shows "CHANGES_REQUESTED" even if all issues are fixed.
**Why:** Old reviews remain in "CHANGES_REQUESTED" state; only new approving reviews change the status.
**Solution:** Always check the actual comments on the latest commit, not the PR-level review status.

### Pitfall 2: CodeRabbitAI approves but doesn’t clear “changes requested”

**Issue:** CodeRabbitAI posts an approving review, but the PR remains blocked (e.g., “changes requested” not cleared).
**Why:** Bot status sync occasionally gets stuck or fails to update GitHub’s gate.
**Solution:** Nudge the bot with an explicit unblock request comment:

```text
@coderabbitai here's a carrot 🥕 please lift the 'changes requested', since you approved.
```

### Pitfall 3: Missing Stale Comment Detection
**Issue:** Fixing issues that were already fixed, wasting time.
**Why:** Didn't check `original_commit_id` vs `commit_id`.
**Solution:** Always use Step 4 to identify stale comments.

### Pitfall 4: Not Verifying Code State
**Issue:** Assuming a stale comment means the issue is still present.
**Why:** GitHub carries comments forward even after fixes.
**Solution:** Always use Step 7 to verify the current code state.

### Pitfall 5: Missing "Already Addressed" Markers
**Issue:** Working on issues CodeRabbit already acknowledged as fixed.
**Why:** Didn't search comment bodies for "✅ Addressed" markers.
**Solution:** Always use Step 5 to check for acknowledged fixes.

### Pitfall 6: Confusion Between Line Numbers
**Issue:** Looking at wrong code because line numbers shifted.
**Why:** Line numbers in comments refer to the original commit, not latest.
**Solution:** Use `git show <original_commit>:<file>` to see the exact state being commented on.

## Lessons Learned

### Why Issues Were Missed in Earlier Rounds

1. **Markdown Linting Issues (MD031, MD022):**
   - These were never flagged in earlier rounds because we focused on code issues
   - CodeRabbitAI only started flagging markdown issues after multiple rounds
   - **Prevention:** Include `markdownlint` or similar in local pre-commit hooks

2. **Stale Comments Not Filtered:**
   - We didn't check `original_commit_id` to identify carried-forward comments
   - Wasted time investigating already-fixed issues
   - **Prevention:** Always use Step 4 to separate stale from fresh

3. **"Already Addressed" Markers Ignored:**
   - CodeRabbitAI adds "✅ Addressed in commit XXX" to its own comments
   - We didn't search for these markers
   - **Prevention:** Always use Step 5 before working on any issue

4. **Code Verification Skipped:**
   - Assumed stale comments meant unfixed issues
   - Didn't verify current code state
   - **Prevention:** Always use Step 7 - read the actual code

## Complete Example Workflow

```bash
#!/bin/bash
# Complete extraction workflow

PR_NUMBER=8
OWNER="flyingrobots"
REPO="loom"
LATEST_COMMIT="094334c"

# Step 1: Fetch all comments
TMPFILE="/tmp/pr-${PR_NUMBER}-comments-$(date +%s).json"
gh api "repos/${OWNER}/${REPO}/pulls/${PR_NUMBER}/comments" > "$TMPFILE"

# Step 2: Extract latest commit comments
cat "$TMPFILE" | jq --arg c "$LATEST_COMMIT" '
  .[] | select(.in_reply_to_id == null and .commit_id[0:7] == $c)
' | jq -s '.' > /tmp/latest-comments.json

# Step 3: Separate stale vs fresh
cat /tmp/latest-comments.json | jq '
  map({
    id,
    line,
    path,
    is_stale: (.commit_id != .original_commit_id),
    has_ack: (.body | contains("✅ Addressed")),
    priority: (
      if (.body | contains("🔴 Critical")) then "P0"
      elif (.body | contains("🟠 Major")) then "P1"
      elif (.body | contains("🟡 Minor")) then "P2"
      else "P3"
      end
    ),
    title: (.body | split("\n")[2:3][0] // "UNTITLED" | gsub("\\*\\*"; ""))
  })
' > /tmp/categorized.json

# Step 4: Show summary
echo "=== STALE (likely already fixed) ==="
cat /tmp/categorized.json | jq -r '.[] | select(.is_stale == true or .has_ack == true) | "\(.id) - Line \(.line) - \(.title)"'

echo -e "\n=== ACTIONABLE (need to address) ==="
cat /tmp/categorized.json | jq -r '.[] | select(.is_stale == false and .has_ack == false) | "[\(.priority)] Line \(.line) - \(.title)"'

# Step 5: For each actionable issue, save full comment
cat /tmp/categorized.json | jq -r '.[] | select(.is_stale == false and .has_ack == false) | .id' | while read id; do
  echo "=== Comment $id ===" >> /tmp/actionable-full.txt
  cat "$TMPFILE" | jq -r --arg id "$id" '.[] | select(.id == ($id | tonumber)) | .body' >> /tmp/actionable-full.txt
  echo -e "\n---\n" >> /tmp/actionable-full.txt
done

echo "✅ Extraction complete. Check /tmp/actionable-full.txt for details."
```

## Quick Reference Card

```text
┌─────────────────────────────────────────────────────────────────┐
│ CodeRabbitAI Comment Extraction - Quick Reference              │
├─────────────────────────────────────────────────────────────────┤
│ 1. Fetch:  gh api repos/OWNER/REPO/pulls/PR/comments          │
│ 2. Filter: .[] | select(.commit_id[0:7] == "LATEST")          │
│ 3. Check:  .commit_id != .original_commit_id  => STALE        │
│ 4. Search: .body | contains("✅ Addressed")   => ALREADY FIXED │
│ 5. Verify: git show HEAD:path | sed -n 'X,Yp' => READ CODE    │
│ 6. Prioritize: 🔴=P0, 🟠=P1, 🟡=P2, 🔵=P3                     │
└─────────────────────────────────────────────────────────────────┘
```

## Automation Opportunity

Consider creating a script at `.github/scripts/extract-actionable-comments.sh` that automates this entire workflow and outputs a clean markdown report.
