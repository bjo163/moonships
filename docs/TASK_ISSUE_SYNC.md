# Backlog → GitHub Issues Synchronization

## Purpose

`docs/TASK_BACKLOG.md` is the canonical engineering checklist. The GitHub Issues board is a generated execution surface for individual tasks. Root `TODO.md` is only a concise human-readable entry checklist and is **not** parsed into issues, which avoids duplicate high-level summary issues. Add atomic, acceptance-tested task lines to `docs/TASK_BACKLOG.md` to create issue entries. The synchronization is deliberately **idempotent**: a stable task ID such as `M2-014` is stored in an HTML marker in the issue body, so repeat runs do not create duplicates.

## Automatic behavior

The workflow `.github/workflows/sync-backlog-to-issues.yml` runs when `docs/TASK_BACKLOG.md` or `docs/MILESTONES.md` changes on `main`.

For every task line in the backlog:
- Unchecked and no linked issue: create an issue titled `[TASK-ID] Task title`.
- Unchecked and linked issue: update the generated task section/title, estimate, scope, acceptance criteria, dependency IDs and source link; ensure the issue is open.
- Checked and linked open issue: update the generated section and close the issue as completed.
- Checked and no linked issue: do nothing; completed design/meta tasks need not have issues.
- Each generated issue carries its stable task ID marker, estimated size and declared prerequisite IDs. Task IDs are the identity; issue title text is not the identity.
- Existing text outside the generated body markers and issue comments are preserved.
- The workflow ensures labels `task` and `backlog-sync`, and milestone association for active phases. Changes to either the backlog or milestone document trigger sync on `main`.
- M0 does not need a GitHub milestone if all M0 checkboxes are already checked.
- A repeated sync is safe; the verified no-op run returned 0 creates, 0 updates, 0 closes and 0 reopens.
- Duplicate task IDs, multiple issues with the same task marker, title collisions without markers, malformed managed-body markers, or managed IDs missing from the backlog fail closed and require explicit reconciliation. The edge-case regression-test task is tracked as M1-037.

The checklist is the status source of truth for this automation. If an issue is manually closed while its task checkbox remains unchecked, the next sync reopens it. To mark the task complete, check the task in `docs/TASK_BACKLOG.md` on `main`. Do not close a generated issue manually to mean “won't do”; instead resolve the scope with a deliberate backlog edit.

## Initial import and dry-run

The initial push trigger can create issues immediately; it is not automatically a dry-run. A successful bootstrap has been performed for this repository, and new unchecked post-V1 tasks are included in the same synchronization.

For a deliberate preview of a future change, use manual dispatch:

1. Open **Actions → Sync backlog to GitHub Issues**.
2. Choose **Run workflow** on `main`.
3. Keep `dry_run = true` to preview counts and planned updates without writes.
4. Review the result summary; only set `dry_run = false` when the intended changes are understood.

After initial sync, edits to `docs/TASK_BACKLOG.md` or `docs/MILESTONES.md` on `main` synchronize automatically. The workflow is not triggered by pull requests, to avoid creating issues from unmerged or fork-supplied task text.

## Required repository setting

The workflow declares minimal permissions:
- `contents: read`
- `issues: write`

Repository Actions policy must permit GitHub Actions and the configured `GITHUB_TOKEN` must be allowed to write issues. No PAT or secret token is required for same-repository issue/milestone operations when the repository token permission allows it.

The workflow deliberately does not:
- execute arbitrary code/commands from a task line;
- assign an agent or user automatically;
- sync discussion comments into the Markdown file;
- create issues for checked-off tasks with no prior issue;
- import tasks from pull requests;
- delete issues or milestones;
- mark a task checked merely because an issue was closed;
- bypass GitHub's rate limits.

## Validation

`node scripts/validate-backlog.mjs` validates task ID uniqueness, acceptance-criteria presence, milestone mapping, declared dependencies (including ranges), dependency cycles and local Markdown links before the workflow writes issues.

If validation fails, the workflow stops before creating/updating issues.
