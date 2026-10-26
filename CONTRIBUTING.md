# Contributing to Moonships Workbench

Moonships is currently in blueprint/pre-implementation. Contributions should establish a safe, coherent foundation before expanding module count.

## Before starting
- Read [README](README.md), [Product Blueprint](docs/PRODUCT_BLUEPRINT.md), [Architecture](docs/ARCHITECTURE.md), [Security Model](docs/SECURITY_MODEL.md) and [AI Agent Guide](docs/AI_AGENT_GUIDE.md).
- Pick one unblocked task from [Task Backlog](docs/TASK_BACKLOG.md), or propose a new bounded task.
- For architecture/security/data-flow changes, add or update an ADR before/with the implementation.

## Change expectations
- One logical task per PR where practical.
- Include tests that prove the acceptance criteria and important negative/failure cases.
- Do not change the documented contract just to make a test pass; explain the trade-off and update docs deliberately.
- Keep UI states complete: loading, empty, offline/stale, denied, cancelled, timeout and error where applicable.
- Use typed contracts, stable error codes and bounded execution.
- Avoid broad dependencies and unrelated formatting churn.
- Keep source, dependency and asset license notices intact.
- Never submit credentials, user data, production logs or private artifacts.

## Security-sensitive changes
Any work that changes filesystem access, shell/process execution, network scanning, security testing, AI tool calls, updater behavior or secret storage requires:
- threat/permission review,
- negative tests (denied/out-of-scope/invalid input),
- explicit failure behavior,
- audit/log redaction review.

## Pull request description
Include:
- Task ID and linked issue (if one exists)
- Problem/user goal
- Scope and explicit non-goals
- Behavior changes/screenshots for UI work
- Tests run and exact results
- Permission/data-flow implications
- Known limitations or follow-up work

## Definition of Done
A task is done only when code, tests, docs (where relevant), and its acceptance criteria agree. A successful compile alone does not complete a feature.
## Integration source, data and license review

For any adapter, provider, external data source, third-party package/code, or visual/audio asset, complete [`docs/INTEGRATION_REVIEW_CHECKLIST.md`](docs/INTEGRATION_REVIEW_CHECKLIST.md) in the PR description. The author must identify provenance and applicable terms; a reviewer must record exactly one outcome (`approved`, `blocked`, or `not applicable`), reviewer/date, and evidence. An empty result or a blocked result means the adapter change is not considered complete. `not applicable` requires a reason. Do not merge/release a blocked adapter as available. Re-review when versions, terms, access or use change.

The checklist is an engineering evidence and review gate, not a legal determination. Verify source terms directly, including permitted access, purpose, rate limits, attribution, retention, redistribution, privacy and revocation. Record required notices in [`docs/THIRD_PARTY_NOTICES.md`](docs/THIRD_PARTY_NOTICES.md). User-facing connector terms acknowledgement does not replace the PR review.
