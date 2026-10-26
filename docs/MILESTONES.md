# Milestones & Release Gates

**Status:** Proposed; dates intentionally omitted until task capacity and initial implementation speed are known.  
**Rule:** A milestone is complete only when its acceptance gates pass. Merging code alone is not completion.

## Milestone map

| ID | Name | Priority | Depends on | Outcome |
|---|---|---|---|---|
| M0 | Product Contract | P0 | None | Consistent blueprint, UX, architecture, safety and task contracts |
| M1 | Desktop Foundation | P0 | M0 | Reliable Tauri desktop shell and developer workflow |
| M2 | Projects & Tasks | P0 | M1 | Local-first personal project/task workflow |
| M3 | Daily Tools | P1 | M1; can overlap with M2 only after tool contract is stable | Safe Cleaner + offline DevTools |
| M4 | System Operations | P1 | M1, M3 | NetOps, SysOps, connections and activity jobs |
| M5 | Intelligence | P1/P2 | M1, source policy, persistence and activity model | Public-source World Monitor/trends baseline |
| M6 | Specialist Workflows | P2 | M2, M4, M5 as applicable | API/DataLab, Observe, Security Lab, evidence/research |
| M7 | Automation & AI | P2 | Stable registry, policy and job contracts | Deterministic workflows, optional AI gateway and optional MCP adapter |
| M8 | Stable Release | P0 release gate | Required features chosen for v1.0 | Tested, documented, packaged and safely updated release |

## M0 — Product Contract

Deliverables:
- README and product positioning
- Product blueprint and audience/domain catalogue
- Menus/user flows, canonical Design System and Audio & SFX Specification
- Architecture, data model and threat/security model
- Milestone and small-task backlog
- AI agent rules and contributing guide
- Source/research decisions and unresolved questions

Exit criteria:
- [x] Planned vs implemented status is unambiguous.
- [x] Primary personas and non-goals are documented.
- [x] Project/task management is explicitly an MVP capability.
- [x] Navigation covers all domains without pretending planned pages are implemented.
- [x] Each implementation task has ID, dependency, bounded scope and verifiable acceptance criteria.
- [x] No unresolved licensing assumption permits copying third-party code without review.
- [x] Windows target/support policy and security baseline are documented.
- [x] Documentation links and paths are checked.

## M1 — Desktop Foundation

Deliverables:
- Tauri 2 + React + TypeScript + Vite app skeleton
- Main window, navigation shell, Command Palette and proper tray lifecycle
- First-run onboarding/profile selector that changes layout only
- Idempotent backlog-to-GitHub-Issues workflow, dependency/link validator, pinned action references and automated dependency update review
- Single-instance behavior or documented intentional alternative
- General settings and per-user local data location
- SQLite connection, migration runner, structured app error/logging
- Typed Tauri command boundary, initial tool registry interface
- Formatting/lint/tests/build CI
- Design-to-product review: minimum-window/overflow behavior, component states, visual/accessibility checks and reproducible screenshots
- Original app/tray asset manifest and optional SFX service with explicit opt-in

Exit criteria:
- [ ] Fresh clone installs using documented steps.
- [ ] Development app and Windows packaging build work in CI.
- [ ] Main window opens, hides/shows from tray and exits cleanly.
- [ ] Ctrl+K opens the Command Palette; navigation and safe create actions work without AI.
- [ ] First-run profile selection persists and does not grant new permissions.
- [ ] Backlog validator passes; sync dry-run path is available; repeat sync creates no duplicates.
- [ ] Workflow dependencies are pinned to verified full SHAs and update review is automated.
- [ ] Sync reconciliation edge cases have regression coverage (M1-037).
- [ ] No background monitoring starts without explicit configuration.
- [ ] Database initializes/migrates without destructive reset.
- [ ] All commands validate arguments and expose stable error codes.
- [ ] CI has clear required checks and no fake status badges.
- [ ] Minimum-window/overflow behavior is documented and verified (M1-038).
- [ ] Core shell visual/accessibility review has recorded evidence (M1-039).
- [ ] Desktop shell task-based usability findings and linked follow-up tasks are recorded (M1-040).
- [x] Shared component variants and interaction states pass tests (M1-041).
- [ ] Original app/tray identity has documented rights and size/theme review (M1-042).
- [ ] Sound preferences default off and central SoundService passes tests (M1-043).
- [ ] Any shipped SFX has approved provenance and packaged-Windows validation (M1-044); otherwise SFX remains a no-op.
- [ ] Reproducible screenshot review evidence is recorded for core shell states (M1-045).
- [ ] App can be used without cloud account or model provider.

## M2 — Projects & Tasks

> 2026-10-26 implementation checkpoint: M2-008 Project update/archive, M2-010/M2-012 Task update/status-policy, M2-011 Task archive/restore, M2-013 due date/time serialization, M2-031 isolated Project/Task repository integration tests, M2-014 Project list/create/edit, M2-016 persisted Task creation and M2-018 persisted Task List are accepted. CI [40259593377](https://github.com/bjo163/moonships/actions/runs/40259593377), [40260498134](https://github.com/bjo163/moonships/actions/runs/40260498134), [40261169609](https://github.com/bjo163/moonships/actions/runs/40261169609), [40262070424](https://github.com/bjo163/moonships/actions/runs/40262070424), [40263435733](https://github.com/bjo163/moonships/actions/runs/40263435733), and [40265034022](https://github.com/bjo163/moonships/actions/runs/40265034022) passed 5/5 jobs each; issues #45, #47, #48, #50, #52 and #65 are closed. M2-018 commit `d6c8612` passed 78 frontend tests, 66 Rust tests, format/lint/typecheck/build, Clippy, local Windows MSI/NSIS packaging and GitHub CI. Task list IPC returns active Personal records; the Tasks route provides deterministic column sorting over a copy of records, with loading/empty/error states. Task detail/edit, Project overview, vertical-slice restart smoke, manual Windows runtime/accessibility and M2 exit gates remain open; see docs/ENGINEERING_HANDOFF.md.

Deliverables:
- Workspace and personal work inbox
- Home dashboard based only on real local project/task/activity records
- Global search across currently available local projects, tasks and registered tools
- Project create/read/update/archive
- Task create/read/update/archive and status/priority
- List and Kanban views
- Search/filter/sort, due dates, labels and optional subtasks
- Activity events, links/references and project overview
- Schema migrations, tests, export/back-up decision

Exit criteria:
- [ ] Create/edit/archive project and task; persist across restart.
- [ ] Home dashboard counts are derived from persisted data and have honest empty/error states.
- [ ] Global search finds local entities by type and is keyboard-navigable.
- [ ] Task can exist without project (personal inbox).
- [ ] Canonical statuses and priorities are consistent across views.
- [ ] Filters/sorts do not mutate source records accidentally.
- [ ] First desktop vertical slice has task-based usability findings and linked follow-up tasks (M2-035).
- [ ] Activity is created for meaningful state transitions.
- [ ] No secrets are logged or exported.
- [ ] Empty/loading/error states and keyboard navigation work.
- [ ] DB migration and repository tests pass.

## M3 — Daily Tools

Deliverables:
- Narrow safe temporary cleaner with read-only scan/preview/revalidation
- Scan progress, cancellation if supported, skipped/failure statuses
- Actual-clean report distinct from estimated recoverable bytes
- Offline DevTools utilities: JSON/YAML validation, Base64, URL encoding, UUID, hash, diff/timestamp helpers
- Task/artifact link for a result, where available

Exit criteria:
- [ ] Scan cannot delete.
- [ ] Cleaner is restricted to allowlisted rules/directories.
- [ ] Path traversal, symlink/junction, locked file, permission denial and file-change cases are tested.
- [ ] Destructive action requires explicit preview and confirmation.
- [ ] No registry cleaner or broad user-profile deletion is present.
- [ ] DevTools work with no AI/network when the function is local.
- [ ] Utilities have unit tests and accessible error/empty states.

## M4 — System Operations

Deliverables:
- Read-only system inventory
- Network diagnostics with timeouts/target display
- Process/service visibility limited to supported platform APIs
- Terminal/SSH/RDP launch profiles without exposing secrets
- Activity Center and progress/status model

Exit criteria:
- [ ] Network tool shows the target and request/action.
- [ ] All calls have timeout, bounded output and clear failure states.
- [ ] App never silently escalates to administrator.
- [ ] UI does not claim successful connectivity when provider/operation fails.
- [ ] Long-running tasks do not block UI; cancellation semantics are truthful.
- [ ] Target and results can be linked to a task.

## M5 — Intelligence

Deliverables:
- Connector/source registry
- Small curated set of permitted feeds or public APIs
- Normalized item schema, canonical URL, timestamps and source attribution
- Dedupe, watchlists, refresh history/freshness, keyword trend baselines
- Research Desk and notifications
- Optional map only after geospatial source/provenance requirements pass

Exit criteria:
- [ ] All displayed items have provenance.
- [ ] Published/observed/event timestamps are distinct.
- [ ] Stale, blocked, rate-limited and offline source states are visible.
- [ ] Trend metric declares time window, baseline and limitations.
- [ ] Deduplication preserves relationships to original sources.
- [ ] Source terms/rate limits are documented; no access-control bypass.
- [ ] Retrieved content is treated as untrusted input.
- [ ] Users can disable a connector and export/delete stored content.

## M6 — Specialist Workflows

Deliverables can be released separately:
- API/DataLab
- Observe/logs/metrics integration
- authorized Security Assessment / Lab
- DFIR evidence and Research Desk improvements
- Recovery and supply-chain checks

Exit criteria per feature:
- [ ] Feature scope/persona defined.
- [ ] Integration and external tools are versioned and optional.
- [ ] Permission/risk class and failure modes are documented.
- [ ] Evidence has timestamp and provenance.
- [ ] Source license/terms reviewed.
- [ ] Unit, integration and security tests pass.
- [ ] Task/artifact/report linkage works or is explicitly deferred.

## M7 — Automation & AI

Deliverables:
- Workflow schema/runner for deterministic steps
- bounded scheduler and job state
- AI provider interface and configuration
- structured output, tool registry integration and provenance
- explicit approval gate for consequential action
- optional version-pinned MCP adapter

Exit criteria:
- [ ] Workflow schema validates before execution.
- [ ] Each step uses registry tool ID/version, not arbitrary free-form command.
- [ ] timeout, retry, idempotency and cancellation rules are explicit per step.
- [ ] Destructive/remote action requires policy approval.
- [ ] AI unavailable/invalid output cannot break non-AI features.
- [ ] Prompt injection test cases cannot modify permissions or expand scope.
- [ ] AI outputs are distinguishable from verified tool results.
- [ ] MCP version and protocol conformance tests are pinned.

## M8 — Stable Release

Deliverables:
- Windows installer(s), versioned release notes and signed update path
- upgrade/migration/restore checks
- user documentation, privacy/security statement and support instructions
- license and third-party notices decision
- reproducible release workflow and SBOM where practical
- usability/accessibility/performance review

Exit criteria:
- [ ] Clean install and upgrade from last supported release tested.
- [ ] Signatures verified; invalid package fails closed.
- [ ] Backup/restore or recovery expectations documented and tested.
- [ ] Critical security tests pass; known issues disclosed.
- [ ] Accessibility and keyboard-flow smoke tests pass.
- [ ] Idle/minimized/tray resource behavior measured.
- [ ] No false claims about features/data freshness.
- [ ] Root LICENSE is intentionally chosen or the repository clearly states no license is granted.
- [ ] Release artifacts and checksums/provenance are available.


## M9 — Recovery & Data Protection (post-V1.0)

Deliverables:
- Explicit data classification and backup scope
- Versioned backup manifest and integrity verification
- Safe backup/restore preview, conflict rules and tested recovery
- Optional scheduled backup only after user opt-in

Exit criteria:
- [ ] Backup scope excludes credentials and other secrets by default.
- [ ] Restore has a preview and does not overwrite current data without a clear conflict decision.
- [ ] Corrupted, oversized and path-traversal archives are rejected.
- [ ] Backup/restore round-trip and failed-restore recovery are tested.
- [ ] Scheduling is opt-in, visible, pausable and logged.
- [ ] User documentation states what can and cannot be restored.

## M10 — Knowledge & Operations Desk (post-V1.0)

Deliverables:
- Notes, collections, runbooks and source-linked knowledge
- Incident/operations timeline and handoff workflow
- Reports with provenance and redaction
- Search adapters for knowledge records

Exit criteria:
- [ ] Notes/runbooks use versioned schemas and safe Markdown rendering.
- [ ] Notes can link to projects, tasks, tool runs and intelligence items.
- [ ] Incident severity, task priority and action risk remain separate fields.
- [ ] Handoff/report exports preserve provenance and redact secrets.
- [ ] Knowledge search has freshness/empty/error states and respects local scope.

## M11 — Advanced Security & Research Integrations (post-V1.0)

Deliverables:
- Assessment run list/detail UI and evidence navigation
- Permitted OSINT/threat-intelligence source adapters
- Local binary/firmware inspection utilities
- Isolated security-lab and external-tool integration boundary

Exit criteria:
- [ ] Every active assessment has a declared authorized scope and expiry/stop rule.
- [ ] Public-source adapters document terms, attribution, rate limits and freshness.
- [ ] Binary inspection treats samples as data and never executes them implicitly.
- [ ] External security tools run through an explicit, bounded adapter contract.
- [ ] Scope, timeouts, limits, evidence integrity and abort paths have negative tests.

## M12 — Advanced World Monitor & Media Intelligence (post-V1.0)

Deliverables:
- Additional permitted media/video/transcript connectors
- Optional browser automation adapter, isolated from the desktop core
- Geospatial event layers and cross-source correlation
- Personalized digests and more advanced trend baselines

Exit criteria:
- [ ] Each connector's terms, access method, rate limits and retention policy are recorded.
- [ ] Browser automation does not bypass authentication, access controls or source restrictions.
- [ ] Transcript segments retain source URL and timestamps where provided.
- [ ] Geospatial confidence/precision is visible; uncertain locations are not shown as exact.
- [ ] Correlation output distinguishes observed facts from interpretation and does not assert causation without evidence.
- [ ] Alert/digest scheduling is opt-in, pausable and observable.

## M13 — CLI, Vault, CostLens, Extensibility & Collaboration (post-V1.0)

Deliverables:
- CLI sharing Tool Registry contracts with GUI
- Secure Credential Manager-backed vault UI
- CostLens provider adapter
- Reviewed extension isolation/signing design
- Team-sync and remote-agent architecture studies before implementation

Exit criteria:
- [ ] CLI and GUI enforce the same policy and tool argument schema.
- [ ] CLI structured output has versioned schemas and secret redaction.
- [ ] Secret values are stored only through the reviewed OS-secure abstraction.
- [ ] Cost results identify provider, currency, time window and missing-data caveats.
- [ ] Extension model defines isolation, signing, permissions, version compatibility and revocation before third-party code is run.
- [ ] Team sync has an explicit identity, authorization, conflict-resolution and deletion model.
- [ ] Remote agents remain unimplemented until a separate threat model and approval architecture are accepted.

## Change control

If scope changes:
1. Add the user/job and reason to Product Blueprint.
2. Add an architecture decision to Research & Decisions if it changes technology/security/data flow.
3. Add a bounded task with acceptance criteria.
4. Re-evaluate milestone dependencies and release claims.
5. Do not silently move a task to Done or change an acceptance criterion after failing it.
