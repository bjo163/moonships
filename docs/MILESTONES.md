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

> 2026-10-26 follow-up: M2-029 adds strict schema-v1 JSON export for all Personal Projects/Tasks, including archived records, and normal-sensitivity trusted Activity. The manifest includes schema/app versions and UTC export time; settings, metadata, logs, stored credentials and restricted Activity are excluded. Native code validates and reparses the export, and a user-selected native Save dialog writes it; no renderer path/filesystem write is accepted. Local checks passed Rust 93/93, Clippy, Rust format, `npm run check` (repo 8/8, frontend 101/101, typecheck/build), targeted export UI/repository tests 15/15, and Windows MSI/NSIS packaging. Backlog validation passed. CI and issue sync are pending; Save-dialog desktop interaction could not be observed because the launcher surfaced an older single-instance shell without this UI. M2-030 is next; M2 remains open.

> 2026-10-26 follow-up: M2-030 adds read-only Project/Task import preview through the native Open dialog. Strict schema validation rejects malformed/unsupported documents; input is capped at 50 MiB and 100,000 records. Preview returns counts, version metadata and up to five names/titles; no database is opened or changed. Local verification passed Rust 94/94, Clippy, Rust format, targeted preview UI 4/4, `npm run check` (repository 8/8, frontend 103/103, typecheck/build), Windows MSI/NSIS packaging and backlog validation. CI [40288940951](https://github.com/bjo163/moonships/actions/runs/40288940951) passed 5/5 jobs; backlog sync [40288940959](https://github.com/bjo163/moonships/actions/runs/40288940959) closed issue [#64](https://github.com/bjo163/moonships/issues/64). Import/commit remains a separate user-confirmed task. M2 exit gates remain open.

> 2026-10-26 follow-up: M2-033 replaces the Home placeholder with a fixed-Personal-workspace, read-only dashboard command. It derives open/overdue/due-soon/blocked counts from active SQLite records, and returns bounded recent Projects, overdue/upcoming Tasks and normal-sensitivity Activity. The UI has honest first-run, empty, loading and retryable error states, links to saved records and refreshes after Task creation; approval work is explicitly unavailable. No polling, synthetic events or fake health metrics were added. Local verification passed Rust 96/96, Clippy, Rust format, `npm run check` (repository 8/8, frontend 105/105, typecheck/build), Windows MSI/NSIS packaging and backlog validation. CI [40291006541](https://github.com/bjo163/moonships/actions/runs/40291006541) passed 5/5 jobs; backlog sync [40291006493](https://github.com/bjo163/moonships/actions/runs/40291006493) passed and closed existing issue [#67](https://github.com/bjo163/moonships/issues/67). M2-034 and M2-035 remain; M2 exit gates remain open. Manual Windows UI verification is not claimed.

> 2026-10-26 follow-up: M2-034 global search combines the registered command catalog with active local Projects and Tasks through the existing Personal-scoped APIs. Typed results open record details, matching includes Project/Task descriptions, keyboard arrows and Enter select every result type, and independent source failures preserve available matches with retry. Local validation passed targeted search/palette/registry tests (15/15), `npm run check` (8 repository tests, 108 frontend tests, typecheck and production build), `git diff --check`, and backlog validation. CI [40292258131](https://github.com/bjo163/moonships/actions/runs/40292258131) passed 5/5 jobs, including Windows installer packaging. Backlog sync [40292258129](https://github.com/bjo163/moonships/actions/runs/40292258129) passed and closed existing issue [#68](https://github.com/bjo163/moonships/issues/68). Manual Windows WebView behavior is not claimed. M2-035 is blocked pending a representative-user session per the test record; its issue and milestone exit gate remain open.

> 2026-10-26 follow-up: M2-027 adds discoverable `Ctrl+Alt+N` / `⌘+Option+N` access to the shared persisted Task creation dialog. The shortcut ignores editable controls and active dialogs; targeted behavior and App contract tests passed 39/39. CI [40285806600](https://github.com/bjo163/moonships/actions/runs/40285806600) passed 5/5 jobs, including Windows package smoke build; backlog sync [40285806580](https://github.com/bjo163/moonships/actions/runs/40285806580) closed existing issue [#61](https://github.com/bjo163/moonships/issues/61). M2-029 is next and is accepted in the newer checkpoint above; M2 remains open.

> 2026-10-26 follow-up: M2-026 defines safe Project/Task references: validated credential-free HTTP(S), namespaced opaque provider IDs, or opaque app-managed artifact IDs. Renderer paths and unsafe URL schemes are rejected; saving does not open, fetch, download or execute targets. Explicit opening requires a reviewed native adapter. Backlog validator passed; this is a documentation contract, not a shipped UI/storage feature. M2-027 was next at that checkpoint and is accepted in a newer checkpoint; M2 remains open.

> 2026-10-26 follow-up: M2-025 adds persisted per-Task Activity history and distinct safe summaries for status changes, priority changes and other meaningful edits. Event writes share the Task update transaction; no-op saves add no history. A scoped typed read lists up to 50 newest events for active Tasks, and Task detail has independent empty/error/retry behavior that preserves unsaved form values. Local checks passed Rust 89/89, Clippy, Rust format, `npm run check` (repo 8/8, frontend 97/97, typecheck/build), targeted detail behaviors 7/7, and Windows MSI/NSIS packaging. CI [40284707581](https://github.com/bjo163/moonships/actions/runs/40284707581) passed all 5/5 jobs including Windows installers; backlog sync [40284707524](https://github.com/bjo163/moonships/actions/runs/40284707524) closed existing issue #59. Manual desktop/accessibility/restart and other M2 gates remain open.

> 2026-10-26 follow-up: M2-024 adds migration v8 for Project-scoped Milestones and active Task associations, typed native commands and Project Overview create/list/associate/status interactions. Dates are validated as calendar dates; Milestone status is metadata and association/status updates leave Task status untouched. Local verification passed `npm run check` (repo 8/8, frontend 95/95, typecheck/build), Rust 87/87, Clippy, Rust format, changed-file Biome, backlog validation, and Windows MSI/NSIS packaging. CI [40283538988](https://github.com/bjo163/moonships/actions/runs/40283538988) passed 5/5 jobs; existing issue [#58](https://github.com/bjo163/moonships/issues/58) closed through backlog sync [40283538947](https://github.com/bjo163/moonships/actions/runs/40283538947). Manual desktop interaction, restart, accessibility, and other M2 release gates remain open. M2-025 is next; the M2 milestone remains open.

> 2026-10-26 implementation checkpoint: M2-008 Project update/archive, M2-010/M2-012 Task update/status-policy, M2-011 Task archive/restore, M2-013 due date/time serialization, M2-031 repository integration tests, M2-014 Projects list/create/edit, M2-016 Task creation, M2-018 Task List, M2-015 Project Overview, M2-017 Task detail/edit, M2-028 Project/Task state handling and M2-032 Project-to-Task persistence smoke are accepted. Code CI runs [40259593377](https://github.com/bjo163/moonships/actions/runs/40259593377), [40260498134](https://github.com/bjo163/moonships/actions/runs/40260498134), [40261169609](https://github.com/bjo163/moonships/actions/runs/40261169609), [40262070424](https://github.com/bjo163/moonships/actions/runs/40262070424), [40263435733](https://github.com/bjo163/moonships/actions/runs/40263435733), [40265034022](https://github.com/bjo163/moonships/actions/runs/40265034022), [40266403136](https://github.com/bjo163/moonships/actions/runs/40266403136) and [40267653497](https://github.com/bjo163/moonships/actions/runs/40267653497) passed 5/5 jobs. M2-028 checkpoint CI [40269028326](https://github.com/bjo163/moonships/actions/runs/40269028326) passed 5/5. Issues #45, #47, #48, #50, #52, #65, #49, #51 and #62 are closed. M2-017 passed 86 frontend tests, 73 Rust tests, local Windows MSI/NSIS packaging and full CI. M2-028's state tests are included in the frontend suite. M2-032 extends the isolated repository test to cover create Project, create linked Task, update status, close and reopen SQLite, then verify both records. Manual desktop UI restart, activity UI, Windows runtime/accessibility and other M2 exit gates remain open; see docs/ENGINEERING_HANDOFF.md.

> 2026-10-26 follow-up: M2-004 adds migration v6 for workspace-scoped labels and Task-label links. Duplicate identity trims and collapses whitespace, then applies Unicode lowercase; duplicates conflict within a workspace and are allowed across workspaces. Local Rust suite passed 76 tests; Repository regression checks CI [40271494784](https://github.com/bjo163/moonships/actions/runs/40271494784) passed 5/5 jobs including Windows packaging. Existing issue #38 closed through backlog-sync run [40272028346](https://github.com/bjo163/moonships/actions/runs/40272028346). M2-005 (issue #39) is accepted in the newer checkpoint below. The M2 milestone remains open.

> 2026-10-26 follow-up: M2-005 adds migration v7 and a workspace-scoped directed TaskDependency service. `task_id` is stored as blocked by `depends_on_task_id`; the inverse “blocks” view is derived. Self-dependencies and cross-workspace references fail, and duplicate edge insertion is idempotent. Local Rust suite passed 79 tests; CI [40272847358](https://github.com/bjo163/moonships/actions/runs/40272847358) passed 5/5 jobs, including Windows packaging. Existing issue #39 closed through backlog-sync run [40273492664](https://github.com/bjo163/moonships/actions/runs/40273492664). M2-019 is accepted in the newer checkpoint; M2-020 is next. The M2 milestone remains open; longer cycle detection is required before dependency edges are used for scheduling.

> 2026-10-26 follow-up: M2-019 adds a persisted Task Board grouped by canonical status. Drag-and-drop and keyboard status selection share the existing update operation and reflect only saved records; a failed update preserves the previous status. Local `npm run check` passed 89 tests, formatting/lint, typecheck and production build. CI [40274608802](https://github.com/bjo163/moonships/actions/runs/40274608802) passed 5/5 jobs including Windows packaging. Existing issue #53 closes through backlog sync from the documentation update. M2-020 is next; the milestone remains open with manual Windows runtime/accessibility and restart evidence outstanding.

> 2026-10-26 follow-up: M2-020 adds debounced case-insensitive local search over active Task titles and descriptions in both List and Board. Clear restores all tasks, no matches has a distinct empty state, and loading/read failures remain separate. Targeted behavior tests passed 7/7; `npm run check` passed repository tests 8/8, frontend tests 90/90, typecheck and production build. Existing issue #54 closes through backlog sync from this documentation update. M2-021 is next; manual Windows runtime/accessibility and restart evidence remain outstanding.

> 2026-10-26 follow-up: M2-021 adds combined Project, status and priority filters that compose with Task search using AND semantics and share results across List and Board. Personal inbox and all represented Project IDs remain selectable. Clearing filters restores the full active collection. Targeted behavior tests passed 8/8; `npm run check` passed repository tests 8/8, frontend tests 91/91, typecheck and production build. CI [40277420116](https://github.com/bjo163/moonships/actions/runs/40277420116) passed 5/5 jobs including Windows packaging. Existing issue #55 closes through backlog sync from this documentation update. M2-022 is next; manual Windows runtime/accessibility and restart evidence remain outstanding.

> 2026-10-26 follow-up: M2-022 adds labelled overdue and priority indicators to Task List and Board. Overdue compares the stored UTC due instant to the current instant; local date formatting remains timezone-aware, and Done/Cancelled tasks are excluded. Targeted behavior tests passed 12/12; `npm run check` passed repository tests 8/8, frontend tests 95/95, typecheck and production build. Rust formatting, 79 Rust tests, Clippy and local MSI/NSIS packaging passed. CI [40279779952](https://github.com/bjo163/moonships/actions/runs/40279779952) passed 5/5 jobs. Existing issue #56 closed through backlog sync run [40280478740](https://github.com/bjo163/moonships/actions/runs/40280478740). M2-023 is accepted below; manual Windows runtime/accessibility and restart evidence remain open.

> 2026-10-26 follow-up: M2-023 exposes the existing `parent_task_id` through typed Task IPC and Task detail. The native service permits one-level active parent/child links only when workspace and nullable Project match; this prevents cycles, and relationship-breaking archive/restore or Project changes return conflicts. Parent and child details link to each other; only roots can add subtasks. Local checks passed: `npm run check` (repo 8/8, frontend 94/94, typecheck/build), Rust 82/82, Clippy and Rust format. Existing Biome schema/deprecation and test non-null warnings remain. M2-024 is next; Windows runtime/accessibility and M2 exit criteria remain open.

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
- [x] Home dashboard counts are derived from persisted data and have honest empty/error states.
- [x] Global search finds local entities by type and is keyboard-navigable.
- [ ] Task can exist without project (personal inbox).
- [ ] Canonical statuses and priorities are consistent across views.
- [ ] Filters/sorts do not mutate source records accidentally.
- [ ] First desktop vertical slice has task-based usability findings and linked follow-up tasks (M2-035).
- [ ] Activity is created for meaningful state transitions.
- [ ] No secrets are logged or exported.
- [ ] Empty/loading/error states and keyboard navigation work.
- [ ] DB migration and repository tests pass.

## M3 — Daily Tools

> M3-001 defines a typed native cleanup rule. M3-002 adds test-only read-only traversal; fixture bytes remain unchanged. M3-003 adds canonical path containment and protected-path validation. M3-004's Windows junction fixture confirms outside targets are rejected and traversal skips the junction. M3-005 defines candidate UUID, relative locator and metadata. M3-006 defines typed progress/terminal events and cancellation semantics. M3-007 adds a serializable preview DTO and Clean & Storage route; no scan data produces an explicit unavailable state. M3-008 adds eligible-item selection and explicit approval tied to one preview. M3-009 adds platform identity capture (Windows volume/file ID or Unix device/inode) and rechecks identity, metadata, path and policy, failing closed on changed or unsupported candidates. M3-010 adds a native-only executor capped at 100 approved files per run, with independent outcomes; Windows deletion rechecks the exact handle identity and fails closed on unsupported platforms. M3-011 separates estimated bytes from logical bytes in files successfully deleted and makes no claim about physical disk space reclaimed. M3-012 tests locked, denied and changed file outcomes; failures remain individual item results. M3-013 defines two conservative app-owned rules for stale scratch and regenerable cache; it does not wire scan execution to the UI. M3-014 persists path-free summaries, lists recent results in Clean & Storage and supports an optional Task link; execution remains native-only and is not wired to the UI.

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
## M2-015 Project Overview — 2026-10-26


The Projects list now opens persisted Project details. A typed native read returns the active Personal Project, non-archived linked Task count and up to ten newest trusted activity summaries for that Project and its currently linked Tasks. Summary text comes from the event allowlist; event metadata and user-authored text are not exposed. The UI displays Project goal/status/description, an accurate zero-Task state, separate no-activity/loading/error/retry/unavailable states, and New Task with the current Project preselected. No health score or placeholder activity is shown.

**Verification:** `npm run check` passed with 8 repository tests and 83 frontend tests, plus typecheck and production build. Rust formatting passed; `cargo test --all-targets` passed 71/71; Clippy with warnings denied passed; local Windows executable/MSI packaging passed; `git diff --check` passed. Biome prints its existing CLI/config schema version warning. Code CI [40266403136](https://github.com/bjo163/moonships/actions/runs/40266403136) passed 5/5 jobs; implementation commit `e5bb0ef` was pushed and existing issue #49 is closed through sync run [40266996207](https://github.com/bjo163/moonships/actions/runs/40266996207). These checks do not replace manual Windows runtime/accessibility validation.

M2-017 Task detail/edit and M2-028 shared Project/Task states are accepted in the newer checkpoints below. **Next requested task:** M2-032 — Project-to-Task end-to-end smoke in the requested P0 order.

## M2-017 Task detail/edit — 2026-10-26

Task titles in the persisted Tasks list now open a detail route. A scoped `get_task` IPC reads active Personal Tasks; `update_task` uses the existing validated transactional Task service. The page edits title, description, canonical status, priority and optional due date/time while retaining the saved Project association. Datetime-local input displays local time and sends an RFC 3339 UTC instant. The form reloads the returned persisted Task after save and distinguishes loading, archived/missing Task, unavailable Project names, save failure and success. No activity metadata or user-authored content is added to trusted activity summaries.

**Verification passed:** `npm run check` (8 repository tests, 86 frontend tests, typecheck and production build); targeted M2-017 tests (3/3 after native title-limit alignment); `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (73 passed, 0 failed); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; final-code `npm run tauri -- build` (Windows executable, MSI and NSIS bundles); and `git diff --check`. GitHub CI [40267653497](https://github.com/bjo163/moonships/actions/runs/40267653497) passed 5/5 jobs on final code `09d9a03`. Biome retains the existing CLI/config warning; bundle packaging and mocked DOM behavior are not manual install/runtime/accessibility evidence. Existing issue #51 is closed through backlog-sync run [40268461798](https://github.com/bjo163/moonships/actions/runs/40268461798).

## M2-028 Project/Task empty/loading/error states — 2026-10-26

Behavior coverage verifies the state contract across the persisted workflow pages: Projects and Task List distinguish failed reads from successful empty results and retry; Project Overview distinguishes no recorded activity from read failure and retries; Task detail shows a retryable error for missing/archived reads; Task creation distinguishes an unavailable optional Project list and preserves inbox creation. Loading states are present while reads are pending. No code change was needed in M2-028 because the behavior shipped with the screen tasks and its acceptance was verified together here.

**Verification passed:** `npm run check` (8 repository regression tests, 86 frontend tests, typecheck and production build). Biome emits the existing 2.6.3 CLI / 2.2.0 config-schema warning. Documentation backlog sync and CI remain pending. **Next requested task:** M2-032 — Project-to-Task end-to-end smoke; existing issue #66.

## M2-032 Project-to-Task persistence smoke — 2026-10-26

`project_to_task_status_flow_persists_after_database_reopen` in `src-tauri/src/repository_integration_tests.rs` uses a unique temporary app-data directory and the real schema migrations. It creates a Project and linked Task, updates Task description/status/priority, drops the SQLite connection, reopens that same database and reruns idempotent migrations, then verifies the Project and updated linked Task from the reopened file. The test removes its disposable directory. This completes the automated persistence/reopen acceptance; it does not claim that the Windows UI was manually restarted.

**Local verification passed:** targeted smoke test (1/1), full Rust suite (73 passed), Clippy with warnings denied, Rust format check and `git diff --check`. CI, backlog sync and existing issue #66 closure are pending for this checkpoint. **Next work after requested P0:** M2-004 — Label relationship; existing issue #38. Manual Windows runtime/accessibility, M2-033/034/035 and milestone exit gates remain open.
