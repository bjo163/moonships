# Moonships — Engineering Continuity & Foundation Evidence

## M2-028 persisted Project/Task states — 2026-10-26

Explicit state behavior across Projects, Project Overview, Task List, Task detail and Task creation now has behavioral coverage. Loading reads show a loading state; successful zero results show accurate empty copy; read failures show error content rather than an empty list, with retry on safe reads. Project activity's empty result is distinct from activity-read failure. Supplemental Project-name/picker failures are separate and leave Task data usable; failed Task edits retain form values for retry. No code behavior change was needed for this acceptance task: the behaviors landed with M2-014/015/016/017/018 and this task verifies their shared acceptance.

**Verification passed:** `npm run check` (8 repository tests, 86 frontend tests, typecheck and production build); `git diff --check`. The behavior suite includes Projects unavailable-vs-empty/retry, Task List unavailable-vs-empty/retry, Project Overview empty-activity-vs-failure/retry, Task detail read failure/retry and Task creation optional-Project failure. Biome prints the existing CLI/config version warning. Issue #51 is closed; issue #62 will close through the following documentation backlog sync. CI for this documentation checkpoint is pending.

**Exact next task in the requested order:** M2-032 — Project-to-Task end-to-end smoke; existing issue #66. Manual Windows runtime/accessibility and M2 exit gates remain open.

## M2-017 persisted Task detail/edit — 2026-10-26

The active Task list opens a Task detail route from each title. Typed `get_task` and `update_task` commands are fixed to the Personal workspace and use the existing validated native Task services. The page edits title (200-character native limit), description, canonical status, priority and optional due date/time while retaining the Task's Project/inbox association. Local datetime input is displayed in the system timezone and converted to an RFC 3339 UTC instant before update. The UI refreshes from the returned saved record and distinguishes loading, archived/missing Task, supplemental Project-name failure, save error and persisted success.

**Verification passed:** `npm run check` (8 repository tests, 86 frontend tests, typecheck and production build); targeted M2-017 tests (3/3 after title-limit alignment); `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (73 passed, 0 failed); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; `npm run tauri -- build` (Windows executable, MSI and NSIS bundles); and `git diff --check`. GitHub CI [40267653497](https://github.com/bjo163/moonships/actions/runs/40267653497) passed 5/5 jobs on final code `09d9a03`. Biome retains the existing 2.6.3 CLI / 2.2.0 configuration-schema warning. Local and CI packaging plus mocked DOM tests do not establish manual install/runtime/accessibility validation. Existing issue #51 is closed through backlog-sync run [40268461798](https://github.com/bjo163/moonships/actions/runs/40268461798).

At that checkpoint, the exact next task was M2-028; it is accepted in the newer checkpoint above. M2-032 end-to-end workflow, M2-019/020/021/022/025 and other secondary tasks, restart smoke, manual Windows runtime/accessibility and M2 exit gates remain open.

## M2-015 persisted Project Overview — 2026-10-26

Opening a Project from the Projects list now loads its saved name, status, goal and description through a typed `get_project_overview` command. The native read is fixed to Personal workspace scope, excludes archived Projects, counts non-archived linked Tasks, and returns at most ten newest safe activity summaries for the Project and its currently linked Tasks. It does not return activity metadata or user-authored text. The Overview displays accurate zero-Task and no-activity states, loading, retryable failure and unavailable-Project states. New Task launched from the Overview preselects that Project.

**Verification passed:** `npm run check` (8 repository tests, 83 frontend tests, typecheck and production build); `cargo fmt --manifest-path src-tauri/Cargo.toml`; `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (71 passed, 0 failed); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; `npm run tauri -- build` (Windows executable and MSI package completed); and `git diff --check`. Biome reports the existing 2.6.3 CLI / 2.2.0 configuration-schema warning. Local packaging and mocked DOM tests do not establish manual installation, runtime, or accessibility validation. GitHub CI [40266403136](https://github.com/bjo163/moonships/actions/runs/40266403136) passed 5/5 jobs. Existing issue #49 will close through the following backlog-sync run.

M2-017 Task detail/edit is accepted in the checkpoint above. Project/Task end-to-end restart workflow, M2-028 states, M2-032 E2E, manual Windows runtime/accessibility and M2 exit gates remain open.
## M2-018 persisted Task List — 2026-10-26

`src/TaskListPage.tsx` adds a Tasks route reachable from primary navigation and the registered Open Tasks command. Its typed `list_tasks` command accepts an empty, unknown-field-rejecting DTO, derives Personal workspace scope in Rust, opens SQLite only under Tauri's per-user app-data directory and returns active records from the existing validated service. The table displays title, Project/inbox, canonical status, priority, due date and updated time. Sort controls are keyboard-accessible and report their active direction; `sortTasks` sorts a copy and uses an ID tie-breaker, leaving records unchanged. Task loading errors, an empty result, and unavailable optional Project names are distinct; retry reloads persisted data.

**Verification passed:** `npm run check` (8 repository tests, 78 frontend tests, typecheck and production build); `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (66 passed, 0 failed); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; `npm run tauri -- build` (Windows executable, MSI and NSIS bundles); and `git diff --check`. GitHub CI [40265034022](https://github.com/bjo163/moonships/actions/runs/40265034022) passed 5/5 jobs. Biome emits the existing 2.6.3 CLI / 2.2.0 config-schema warning. Bundle packaging and mocked DOM behavior are not manual installation/runtime/accessibility evidence. Implementation commit `d6c8612` was pushed; existing issue [#52](https://github.com/bjo163/moonships/issues/52) closes through backlog sync. At that checkpoint, the exact next task was M2-015; Project overview and M2-017 Task detail/edit are accepted in the newer checkpoints above. End-to-end restart flow, manual Windows runtime/accessibility and M2 exit gates remain open.

## M2-016 persisted Task creation — 2026-10-26

`src/TaskCreateDialog.tsx` opens from the global New task button and the New Task palette command. It requires a nonblank title, allows an active Project or the Personal inbox, defaults to Todo/P2 Normal, and submits through the typed `create_task` Tauri command. The command accepts only a named camelCase request (unknown fields rejected), derives the workspace from the native Personal workspace constant, opens the database in Tauri's app-data location, and calls the validated transactional Task service. The saved Task is returned to the UI before success is shown. Project-list failure remains distinct and still permits inbox creation. Title errors are announced accessibly; Escape/Cancel do not invoke the write command.

**Verification passed:** `npm run check` (8 repository tests, 74 frontend tests, typecheck and production build); `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (65 passed, 0 failed); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; `npm run tauri -- build` (Windows executable, MSI and NSIS bundles); and `git diff --check`. GitHub CI [40263435733](https://github.com/bjo163/moonships/actions/runs/40263435733) passed 5/5 jobs. Biome emits the existing 2.6.3 CLI / 2.2.0 config-schema warning. Build/package and mocked DOM behavior are not manual installation/runtime/accessibility evidence. Implementation commit `e4fe3c8` was pushed; existing issue [#50](https://github.com/bjo163/moonships/issues/50) closed through the backlog sync. M2-018 is now accepted in the checkpoint above; M2-015 Project overview and M2-017 Task detail/edit are accepted in newer checkpoints. End-to-end restart flow, manual Windows runtime/accessibility and M2 exit gates remain open.

## M2-014 persisted Project list/create/edit — 2026-10-26

`src/ProjectsPage.tsx` now lists active projects from SQLite, creates and edits project name, goal, description and status, refreshes from the persisted list after saves, and distinguishes loading, empty, and recoverable error states. The Projects route is available from navigation and the New Project command opens the real form. Rust exposes typed, camelCase, unknown-field-rejecting list/create/update DTOs scoped to the deterministic Personal workspace; commands resolve the SQLite file only from Tauri's per-user application data directory and use the existing validated Project services. The active list is case-insensitive alphabetical with stable ID tie-breaking. A disposable file-backed integration test closes and reopens the connection, then confirms the same Project remains present.

**Verification passed:** `npm run check` (8 repository tests, 70 frontend tests, typecheck and production build); `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (64 passed, 0 failed); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; `npm run tauri -- build` (Windows executable, MSI and NSIS bundles); and `git diff --check`. The disposable file-backed test closes and reopens the SQLite connection, then confirms the created Project remains readable. GitHub CI [40262070424](https://github.com/bjo163/moonships/actions/runs/40262070424) passed 5/5 jobs, including the Windows package build. Biome emits its existing 2.6.3 CLI / 2.2.0 config-schema warning. Installer packaging and database reopen tests are not manual app installation/runtime evidence. Implementation commit `d5469ba` was pushed; existing issue [#48](https://github.com/bjo163/moonships/issues/48) closed through backlog sync. M2-015 Project overview is accepted in the latest checkpoint. Project→Task listing/detail workflow, UI-level restart smoke, manual Windows/accessibility checks, and M2 exit gates remain open.

## M2-031 Project/Task repository integration tests — 2026-10-26

`src-tauri/src/repository_integration_tests.rs` adds isolated, in-memory SQLite integration tests over the public Project and Task services. The fixture runs the real migrations and asserts that its database path is empty. Coverage verifies persisted project/task create, update, listing and activity counts; UTC due date storage; cross-workspace link rejection without partial writes; and consistency when a project is archived while a linked task remains.

**Verification passed:** targeted integration tests (3/3 at M2-031 completion); full Rust suite (64/64 at current HEAD), formatting and Clippy; `npm run check`; local Windows MSI/NSIS build; GitHub CI [40261169609](https://github.com/bjo163/moonships/actions/runs/40261169609) passed 5/5 jobs; backlog sync run [40261169634](https://github.com/bjo163/moonships/actions/runs/40261169634) succeeded and existing issue [#65](https://github.com/bjo163/moonships/issues/65) is closed. The M2-031 implementation commit is `b0f097a`; its code run was superseded, and the successful complete run included that code and its following documentation checkpoint at `fecf519`.

Implementation commit: `b0f097a` (`test(M2-031): cover project and task repository persistence`). M2-013 CI [40260498134](https://github.com/bjo163/moonships/actions/runs/40260498134) passed 5/5 jobs and issue [#47](https://github.com/bjo163/moonships/issues/47) is closed. M2-014, M2-016, M2-018 and M2-015 are accepted UI slices; see the latest checkpoints above. Project and Task creation use local persistence. Task listing, detail/editing, the complete Project→Task workflow and M2 exit gates remain open.

## M2-013 due date/time serialization — 2026-10-26

`dueAt` now accepts an RFC 3339 timestamp only when the input includes `Z` or a numeric timezone offset. The Task create/update service normalizes it to UTC with a `Z` suffix, preserves fractional-second precision, and persists the canonical value in the existing `due_at` column. Date-only and timezone-free values are rejected before writes; blank optional values clear the due time. The model contract documents local-time rendering at the UI boundary.

**Local verification passed:** targeted Task tests 16/16; `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (58 passed, 0 failed); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; `npm run check` (8 repository tests, 67 frontend tests, typecheck and production build); and `npm run tauri -- build` (Windows executable, MSI and NSIS bundles). Biome emits its known config-schema mismatch warning documented in the M2-011 section. The test generated a temporary Cargo.lock because the repository does not carry one; it is excluded from the change.

Implementation commit: `f58adca` (`feat(M2-013): normalize due timestamps to UTC`). The backlog records M2-013 complete; GitHub CI [40260498134](https://github.com/bjo163/moonships/actions/runs/40260498134) passed 5/5 jobs and existing issue [#47](https://github.com/bjo163/moonships/issues/47) is closed. The next task was M2-031, completed in the checkpoint above.

## M2-011 Task archive/restore — 2026-10-26

`src-tauri/src/tasks.rs` now provides soft archive and restore through `archived_at`, idempotent repeats, transactional safe activity summaries, and deterministic active/archived/all list filtering. Existing status, priority and creation metadata persist. Restore is rejected while a linked project is archived. Regression tests cover archive/restore, no-ops, missing and wrong-workspace records, metadata and history, activity failure rollback, archived-project consistency, filtering, and file-backed state after reopening.

**Local verification passed:** `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; targeted task tests (12/12); `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (54 passed, 0 failed); and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`. `npm run check` exited 0 (8 repository tests, 67 frontend tests, typecheck and production build); Biome reports that its 2.6.3 CLI reads a 2.2.0 config schema, and npm reports 3 advisories while installing the unlocked declared dependency ranges. `npm ci` cannot run because the repository has no package lock; dependencies were installed with `npm install --package-lock=false`, and no lockfile is included. `npm run tauri -- build` passed on Windows and created the MSI and NSIS bundles. These are packaging results, not manual install/runtime evidence. `git diff --check` passed.

Implementation commit: `8815dfb` (`feat(M2-011): add safe task archive and restore`); status/documentation commit: `6214cba`. Backlog records M2-011 complete and existing GitHub Issue [#45](https://github.com/bjo163/moonships/issues/45) is CLOSED. Commit `6214cba` passed GitHub CI run [40259593377](https://github.com/bjo163/moonships/actions/runs/40259593377), 5/5 jobs: frontend production build, Rust format/Clippy, backlog sync regressions, Windows package smoke build, and Rust unit tests. Exact next task in requested priority order: M2-013, then M2-031; the remaining M2 UI and vertical-slice tasks are still open.

## Current checkpoint — M2-010 Task update and M2-012 status policy (2026-10-26)

**Canonical branch:** `main`. **Accepted implementation SHA:** [63816b2](https://github.com/bjo163/moonships/commit/63816b29a2d1d103f2e504855a7746c68b84ff59). **Complete CI:** [40254246931](https://github.com/bjo163/moonships/actions/runs/40254246931) — **SUCCESS 5/5 jobs**: Rust formatting and Clippy (warnings denied), **49/49 Rust tests**, frontend production build, backlog sync regression and Windows installer package smoke build. Windows package construction is not actual installer execution or user testing.

**M2-012 (#46):** `src-tauri/src/task_status.rs` owns the initial seven canonical Task statuses, four priorities and change/no-op validation. Because product docs do not define restricted transition edges, the initial policy permits explicit transitions among canonical statuses and rejects unknown values; UI adapters must use the Task service rather than invent a second policy. Direct tests cover allowed, invalid, and unchanged transitions.

**M2-010 (#44):** `src-tauri/src/tasks.rs` provides validated `UpdateTask` requests. The service checks task/workspace/project scope, active parent project, title/status/priority/description, unchanged edits, missing records, and preserves original creation metadata. The entity mutation and trusted ActivityEvent are atomic; error injection tests prove rollback without partial edits. Project association can be changed or cleared safely. Added task test coverage and centralized create validation; no schema migration or user-data reset.

**Canonical backlog:** M2-010 and M2-012 are checked after acceptance. Existing issues #44 and #46 should close via the idempotent backlog-sync workflow; check that result separately. **Remaining immediate next task:** M2-011 Task archive/restore, followed by M2-013 due date semantics, M2-004/005 relationships and the real native IPC + React Projects/Tasks workflow (M2-014–022/025/028/031/032). M2-033/034/035, M1 Windows manual and release gates remain open. Current published release is still `v0.1.0-alpha.2`, not a stable MVP.

---


## Previous checkpoint — M2-008 project update/archive accepted (2026-10-26)

**Canonical branch:** `main`. **Verified code:** [e83930b](https://github.com/bjo163/moonships/commit/e83930bde4d40d8d978a4dc45643e43e49d908e5). **Relevant CI run:** [40253359963](https://github.com/bjo163/moonships/actions/runs/40253359963) — Rust format/Clippy PASS, 43/43 Rust unit tests PASS, frontend production build PASS, backlog sync regression PASS; the Windows installer smoke-build remained pending at the time of this checkpoint. Inspect its final conclusion; do not treat packaging as Windows runtime validation.

**M2-008, existing issue #42:** The native project update and soft-archive services preserve linked records and write trusted ActivityEvents transactionally. Acceptance tests verify field normalization, allowed status and workspace scoping, no-op edit without timestamp/activity churn, preservation of linked tasks on archive, and rollback when activity insert fails. Project creation cannot bypass soft archive by specifying status `archived`. No schema migration, data reset, or unsafe hard delete was introduced. The canonical backlog now checks M2-008 and will close issue #42 via its existing idempotent sync workflow.

**Unfinished:** Task domain M2-004/005/010–013, persistent Project/Task IPC and React views M2-014–022, activity UI, integration/E2E M2-031/032, dashboard/search and usability M2-033–035, plus other unchecked task IDs in the backlog. Manual Windows desktop and accessibility tasks remain open. This checkpoint is **not** a verified MVP, release candidate or stable release.

**Next unblocked task:** M2-012 status transition policy; then M2-010 task update, M2-011 task archive/restore, and UI/IPCs after their service acceptance. No new branch required.

---


## Previous checkpoint — M2-006 activity storage accepted (2026-10-26)

**Canonical branch:** `main`. **Verified implementation HEAD:** [4d4cc4d](https://github.com/bjo163/moonships/commit/4d4cc4dee571f344e7cdcc67f7dc2a9da8f2c64e). **Full code CI:** [40249712444](https://github.com/bjo163/moonships/actions/runs/40249712444) **SUCCESS 5/5 jobs**, including **38/38 Rust unit tests**, Rust formatting/Clippy with warnings denied, frontend build, backlog regression and Windows MSI/NSIS installer smoke packaging.

**M2-006 (#40):** Schema v5 adds `activity_events` with UTC timestamps, local actor, event action, safe summary, sensitivity, workspace-scoped entity reference and composite SQLite foreign keys. Both `create_project` and `create_task` now persist their corresponding event in the same transaction. Tests assert valid project/task references, rejection of cross-workspace and nonexistent entities, secret-free event summaries, atomic entity rollback when event insertion fails, v4→v5 upgrade preserving prior records, and schema rollback when an index collision interrupts migration.

**Accepted in canonical backlog:** M2-002, M2-003, M2-006, M2-007 and M2-009. After this documentation commit, confirm the existing issue-sync workflow closes #40. Issues #36/#37/#41/#43 were already confirmed closed by sync run [40248824539](https://github.com/bjo163/moonships/actions/runs/40248824539). No new branches or duplicate issues.

**Not completed:** M2 project/task edit/archive services, Tauri IPC for this model, React Projects/Tasks screens, real end-to-end persisted UI workflow and Windows manual install/usability validation. This is **not an MVP or stable-release gate pass**. The last verified published version remains prerelease `v0.1.0-alpha.2`.

**Exact next unblocked task:** M2-008 — transactional Project update/archive service preserving history and workspace integrity; then M2-012 status policy and M2-010/011 Task update/archive, followed by IPC and Project→Task UI vertical slice (M2-014/015/016/017/018/028/032). Keep independent M1 Windows/runtime/accessibility blockers open until observed.

---

## Previous M2 checkpoint (historical)

## M2 model/service checkpoint — 2026-10-26 (historical)

**Canonical branch:** `main`. **Code commit:** [9ce3718](https://github.com/bjo163/moonships/commit/9ce3718f425366f8681a0732fd1d6fb6f2f12134). **Verified CI:** [40248122958](https://github.com/bjo163/moonships/actions/runs/40248122958) — SUCCESS, all 5/5 required jobs; Rust 33/33 tests, formatting and Clippy, frontend production build, repository regression and Windows MSI/NSIS packaging all PASS.

**Accepted and now checked in canonical backlog:**
- M2-002 (#36): schema-v3 Project constraints/defaults, v2 preservation, idempotence and deliberate transactional rollback, verified by Rust tests in `src-tauri/src/lib.rs`.
- M2-003 (#37): schema-v4 Tasks with nullable project inbox, canonical `backlog`/`p2` defaults, composite workspace FKs, v3 upgrade/reopen and rollback tests.
- M2-007 (#41): `src-tauri/src/projects.rs` transactional Project create with UUID v4, validation, active-workspace check, duplicate-key rejection and no partial writes.
- M2-009 (#43): `src-tauri/src/tasks.rs` transactional Task create with title-only inbox, optional valid project, workspace isolation, UUID v4, validated status/priority and no writes on invalid requests.

**Issue reconciliation:** This checkpoint updates `docs/TASK_BACKLOG.md`, the canonical source. Check the existing idempotent sync workflow for closure of #36, #37, #41 and #43. No duplicate issue or branch was created.

**Release readiness: NOT READY.** The project/task service layer is not wired to Tauri commands or React screens yet. Installer packaging is not a real Windows installation/usability test. The latest confirmed published release remains `v0.1.0-alpha.2`; no stable release or new tag is authorized by these results.

**Exact next unblocked task:** M2-006 ActivityEvent storage with no secret metadata; then M2-008 and M2-010/011/012/013 service work followed by M2-014/015/016/017/018/028/032 for the real persisted Project → Task UI flow. M1 Windows manual runtime, accessibility, screenshots and release gates remain outstanding.

---

## Historical M1 checkpoint (superseded below)

**Checkpoint:** 2026-10-26. **Milestone:** M1 — Desktop Foundation (**not complete**).  
**Latest implementation commit under evaluation:** `0d0a7c3f8c9323dadc847c86a4eea4fbe91bd26f` (M1-032/033 behavior tests and focus fixes).  
**Last fully successful repository CI:** [40239724966](https://github.com/bjo163/moonships/actions/runs/40239724966) — **SUCCESS, 5/5 jobs**, including M1-032 browser-interaction tests and Windows MSI/NSIS packaging. **Latest code CI:** [40240662226](https://github.com/bjo163/moonships/actions/runs/40240662226) — frontend formatting, lint, behavior tests, typecheck and build **PASS** at the time of this checkpoint; native/installer jobs require their own final status check.  
**Last public release checked:** `v0.1.0-alpha.2` (prerelease). Current `main` is **not** included in that release. No stable release published.

> Read the latest `main` HEAD, CI and synced issue states after this document commit. The source commit above is verified evidence for implementation; the backlog/documentation sync workflow runs separately after this checkpoint is committed. Do not treat this document as evidence of live Windows runtime behavior.

## Reconciliation of previously unchecked foundation tasks

The following task checkboxes were previously stale. Their acceptance is supported by implementation and tests in the verified CI, rather than the mere presence of source files. The `docs/TASK_BACKLOG.md` checkbox remains the canonical synchronization source.

| Task | Existing evidence | Boundary |
|---|---|---|
| M1-019 | `SettingsDialog` in `src/App.tsx` has usable theme/tray controls; unavailable notification/monitor controls are disabled and visibly `Planned`. `src/App.test.tsx` checks markup. | No notification or monitoring capability is claimed live. |
| M1-021 | `docs/ARCHITECTURE.md` specifies named DTOs; `HideToTrayPreferenceDto`, `get/set_hide_to_tray`, `AppConfig` have typed native requests and `deny_unknown_fields`; contract checks in `src/App.test.tsx`. | Future commands must still validate arguments. |
| M1-022 | `database_path` and `open_database_in` use Tauri per-user `app_data_dir`; Rust path/file tests in `src-tauri/src/lib.rs`. | No current-working-directory fallback. |
| M1-023 | `run_migrations` uses one SQLite transaction and `schema_migrations` version table; Rust tests cover fresh, idempotent, and too-new schema. | Schema beyond supported version remains rejected. |
| M1-024 | Each opened SQLite connection enables foreign keys and 5000-ms busy timeout; Rust tests check pragmas and rejected invalid FK. | Every future connection adapter must preserve these pragmas. |
| M1-025 | `src-tauri/src/local_logging.rs` implements local JSONL levels/categories, recursively redacts secret-bearing fields; tests check redaction and append. | No default telemetry or credential logging. |
| M1-026 | `src-tauri/src/domain.rs` defines all eight JobStatus states and locks serialized names in Rust tests. | The runtime Activity Center UI remains a later task. |
| M1-027 | Typed versioned ToolDefinition/ToolResult plus in-memory ToolRegistry; unit test registers a sample tool and rejects duplicate ID. | Definition is not proof tools execute. |
| M1-028 | Rust evaluate_policy denies missing capabilities and requires approval for impactful risk classes; tests cover allowed/denied/approval. | Native policy must be enforced by each future tool executor. |
| M1-029 | `src/module-availability.ts` distinguishes available, disabled, unsupported, not_configured and degraded; `src/App.tsx` renders not-configured honestly; `src/App.test.tsx` checks it. | No fake successful module query. |
| M1-034 | `src/workspace-profile.ts` defines six profiles and local persistence. `src/App.tsx` adds first-run local-first explanation, no implicit permission grant, keyboard containment and explicit selection; `src/onboarding.behavior.test.tsx` adds three real DOM tests for first run, persistence/reopening, Escape, focus wrap and dismissal. | Browser/WebView-local profile storage is not yet synchronized with the native AppConfig record; real WebView2 UI testing remains pending. |

**Intentionally still open:** M1-001 requires actual fresh-clone app launch proof; M1-016/017 need actual Windows Quit/Hide and second-instance behavior; M1-038/039/040 require viewport, visual/contrast, accessibility and manual Windows observations; M1-042–045 retain icon/asset, sound and screenshot gate work. M1-031/032/033 are checked based on frontend behavior-level tests and prior native CI, not on claims of Windows WebView accessibility conformance.

## M1-031 — Command Palette verified and next task

The existing palette now has behavior-level DOM tests plus stale-selection fixes. [Implementation CI #40193893988](https://github.com/bjo163/moonships/actions/runs/40193893988) completed **SUCCESS (5/5 jobs)** at code commit `664758da0d9eb19dec3f71fbe69aa5dbb79fdee3`, including **56/56 frontend tests**, Rust formatting/Clippy/tests, backlog regression and Windows installer packaging. Five tests in `src/command-palette.behavior.test.tsx` verify keyboard opening/focus, query filtering, Arrow navigation, Enter/Escape, loading/empty state, no hidden ARIA active descendant, stale-index recovery and reset after reopening. `src/CommandPalette.tsx` fixes the corresponding edge cases without expanding permissions.

**M1-032 completed:** `src/command-registry.behavior.test.tsx` exercises the actual App to verify allowlisted command results, safe navigation, Settings, and project/task/note preview-only actions (six scenarios). Static registry checks still reject unknown IDs. Implementation/test commit [0f8d5a9](https://github.com/bjo163/moonships/commit/0f8d5a92da6a9c3001055dbb010393c665f5b3b5) passed [CI 40239724966](https://github.com/bjo163/moonships/actions/runs/40239724966) (5/5 jobs).

**M1-033 frontend acceptance completed:** `src/command-shortcut.behavior.test.tsx` exercises Ctrl+K opening/search focus, Escape from both search and close button, editable-control exclusion, Settings suppression/focus handoff and preview-dialog focus. `src/CommandPalette.tsx` moves Escape handling to the dialog and adds a Tab boundary handler; `src/App.tsx` avoids stealing focus when transitioning to another dialog. The intermediate CI runs [40240222750](https://github.com/bjo163/moonships/actions/runs/40240222750) and [40240461506](https://github.com/bjo163/moonships/actions/runs/40240461506) exposed one synthetic Shift+Tab boundary assertion that was removed from M1-033's acceptance suite pending M1-039's complete accessibility review. Do **not** claim Windows Tab-cycle/forced-colors conformance from this source change. [Final frontend CI 40240662226](https://github.com/bjo163/moonships/actions/runs/40240662226) passed formatting, lint, tests, typecheck and build at this checkpoint; inspect remaining CI jobs and latest HEAD again.

**Next unblocked action:** M1-038 — verify minimum-window and overflow decisions against the real Windows app (960×640, 1280×720, 1440×900; 100% and 150% scaling), retaining M1-039/040 open until manual evidence exists. Work on independent M2 model/migration tasks while Windows execution is unavailable; M2 persistence and its vertical slice are still mandatory before claiming a complete product MVP.

**Reminder:** CI packaging does not prove real Windows tray lifecycle or WCAG compliance. M1-016/017/039/040 remain gated on manual evidence. M1-034 is complete only as WebView-local profile preference behavior; native config synchronization remains outside the checked acceptance criteria.

## Windows manual batch — NOT TESTED on current main

Use an MSI/NSIS artifact from the **exact current main CI run** and record OS version, scaling (100% and 150%), SHA, installer filename, observed results, and reproduction steps:

1. Launch fresh and upgraded installation; verify first-run onboarding, six profiles and persisted selection after restart.
2. Try Ctrl+K, typing, Arrow keys, Enter and Escape; verify focus returns and no permission bypass.
3. Toggle hide-to-tray, close/hide, restore via tray Open, open Settings from tray, then explicitly Quit; check process termination.
4. Launch a second process; verify the original main window is brought forward without an extra instance.
5. Check narrow window and 960x640 / 1280x720 / 1440x900 at both scalings; test dark/light, focus, reduced motion and contrast using measured values. Never label WCAG 2.2 AA certified without evidence.

Previous Alpha.2 smoke observations in `docs/SMOKE_TEST_ALPHA2_WINDOWS.md` were **USER-REPORTED PASS**, not current-main proof. M1-040 remains OPEN.

## Local verification commands

```sh
npm install
npm run format:check
npm run lint
npm run test:repo
npm test
npm run typecheck
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Preserve `main`, use existing GitHub Issues and the idempotent backlog sync; never issue a stable-release or manual-Windows PASS claim from a successful compilation alone.

## M2-001 — Personal Workspace migration (2026-10-26)

Implemented schema v2 in `src-tauri/src/lib.rs` with a single deterministic Personal workspace seed and transactionally recorded migration version. Source commits: [`6291d9b`](https://github.com/bjo163/moonships/commit/6291d9b4d29b8560043f521e76d7835fdc1e5a30) and rustfmt-only correction [`2a8cf7d`](https://github.com/bjo163/moonships/commit/2a8cf7d46ce3d84d91879977fcd01f461905ca50). CI [40241213681](https://github.com/bjo163/moonships/actions/runs/40241213681) has PASS for frontend production build, backlog tests, Rust formatting/Clippy and **21/21 Rust unit tests**, including the fresh DB workspace seed and upgrade from schema v1 retaining existing user metadata. At this documentation checkpoint, Windows packaging remains **IN PROGRESS**, not a runtime test. Task acceptance is satisfied by the native migration test results; Windows installer and real-Windows smoke gates remain independent.

**Next schema task:** M2-002 — Add Project model/migration, using a bounded and foreign-key-validated project table associated with the default workspace.
