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
- Menus, user flows and design system
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
- Idempotent backlog-to-GitHub-Issues workflow and dependency/link validator
- Single-instance behavior or documented intentional alternative
- General settings and per-user local data location
- SQLite connection, migration runner, structured app error/logging
- Typed Tauri command boundary, initial tool registry interface
- Formatting/lint/tests/build CI

Exit criteria:
- [ ] Fresh clone installs using documented steps.
- [ ] Development app and Windows packaging build work in CI.
- [ ] Main window opens, hides/shows from tray and exits cleanly.
- [ ] Ctrl+K opens the Command Palette; navigation and safe create actions work without AI.
- [ ] First-run profile selection persists and does not grant new permissions.
- [ ] Backlog validator passes; task sync dry-run is reviewed and repeat sync creates no duplicates.
- [ ] No background monitoring starts without explicit configuration.
- [ ] Database initializes/migrates without destructive reset.
- [ ] All commands validate arguments and expose stable error codes.
- [ ] CI has clear required checks and no fake status badges.
- [ ] App can be used without cloud account or model provider.

## M2 — Projects & Tasks

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

## Change control

If scope changes:
1. Add the user/job and reason to Product Blueprint.
2. Add an architecture decision to Research & Decisions if it changes technology/security/data flow.
3. Add a bounded task with acceptance criteria.
4. Re-evaluate milestone dependencies and release claims.
5. Do not silently move a task to Done or change an acceptance criterion after failing it.
