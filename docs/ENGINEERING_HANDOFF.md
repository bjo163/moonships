# Moonships — Engineering Continuity & Foundation Evidence

## M2-029 Project & Task export — 2026-10-26

Added schema-v1 JSON export for all Personal workspace Projects/Tasks (active and archived) plus normal-sensitivity trusted Activity. The strict manifest carries `schemaVersion`, Cargo `appVersion`, and UTC `exportedAt`; the native validator checks canonical fields, scope, IDs, timestamps and relationships, then serializes and reparses the typed schema before saving. App settings, `app_metadata`, logs, stored credentials and restricted Activity are omitted. A user-visible native Save dialog chooses the destination; no path or file bytes come from the frontend. Cancellation and I/O errors do not report success. Project and Task list screens expose the action and warn users that authored descriptions are included.

**Verification passed:** targeted Rust export tests and disposable SQLite integration test (4/4); targeted UI behavior and Projects/Tasks regression tests (15/15); `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (93/93); Clippy with warnings denied; `cargo fmt -- --check`; `npm run check` (repository tests 8/8, frontend tests 101/101, typecheck and production build); Windows `npm run tauri -- build` produced MSI and NSIS installers; backlog validation and `git diff --check`. Full desktop Save dialog interaction could not be verified because the app launcher surfaced an older installed single-instance shell that lacks these Projects/Tasks screens; no export was saved. CI and issue sync are pending. M2-029 is accepted by automated evidence; M2-030 is next. M2 exit gates remain open.

## M2-027 Task creation shortcut — 2026-10-26

Added `Ctrl+Alt+N` (`⌘+Option+N` on Mac) to open the shared persisted Task creation dialog, and exposed the shortcut on the global New task button. It ignores editable targets and active dialogs/overlays, preserving typing and focus context. The dialog retains its existing focus management and native IPC persistence behavior.

**Verification passed:** targeted `npx vitest run src/command-shortcut.behavior.test.tsx src/App.test.tsx` (39/39); `npm run check` (repository tests 8/8, frontend tests 99/99, typecheck and production build); backlog validation and `git diff --check`. Biome reports the existing CLI/schema mismatch and deprecated configuration notices plus four existing Task List test non-null assertion warnings. CI and issue sync are pending. M2-027 is accepted locally; M2-029 is the next dependency-ready task. M2 exit gates remain open.

## M2-026 Artifact/reference contract — 2026-10-26

References are typed as validated HTTP(S) URL, namespaced provider ID, or opaque app-managed artifact ID. URL validation excludes embedded credentials and unsafe schemes; renderer-provided paths are rejected. App-managed references resolve below application-owned storage after canonical containment checks. Saving never opens, fetches, downloads or executes the target; opening requires a later explicit user action through a reviewed native adapter. The contract is documentation-only; no storage or open UI is claimed live.

**Verification passed:** backlog validation and `git diff --check`. This task changes the product/data/security contract only; no executable behavior or runtime result is claimed. M2-026 is accepted; next dependency-ready task is M2-027 — keyboard shortcut to create a Task. M2 exit gates remain open.

## M2-025 Task Activity history — 2026-10-26

Task updates now record explicit `status_changed` and `priority_changed` events; meaningful title, description, due-date or Project edits use a generic `updated` event. Applicable events commit atomically with the Task update and no-op saves add none. A typed Task-ID-only command returns up to 50 newest records for an active Task in the fixed Personal workspace. Task detail shows real persisted summaries and timestamps, a separate empty/error state, and retries history without resetting unsaved form edits. Summaries stay fixed trusted text and do not expose Task content or credentials.

**Verification passed:** `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (89/89); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; Rust format check; `npm run check` (repository tests 8/8, frontend 97/97, typecheck and production build); targeted Task detail behavior tests (7/7); and Windows `npm run tauri -- build` (MSI and NSIS produced). Biome reports the existing CLI/schema mismatch and deprecated config notices plus preexisting Task List test non-null warnings. M2-025 CI and issue sync remain to verify. M2 exit gates remain open.

## M2-024 Project Milestones — 2026-10-26

Migration v8 adds Project-scoped milestones and task associations enforced by composite SQLite foreign keys. The native list/create/associate/update commands use typed DTOs, derive Personal workspace scope, validate milestone dates and statuses, and reject cross-Project or archived Task links. Project Overview can create dated milestones, display saved status and associated active tasks, associate Project tasks, and update milestone status with distinct read/error feedback. These actions do not mutate Task status. Legacy migration fixtures now explicitly remove v8 additions when reconstructing earlier schema versions.

**Verification passed:** `npm run check` (repository tests 8/8, frontend tests 95/95, typecheck and production build); `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (87/87); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; Biome check of changed frontend files; and Windows `npm run tauri -- build` (MSI and NSIS produced). GitHub CI [40283538988](https://github.com/bjo163/moonships/actions/runs/40283538988) passed 5/5 jobs, and backlog sync [40283538947](https://github.com/bjo163/moonships/actions/runs/40283538947) closed existing issue [#58](https://github.com/bjo163/moonships/issues/58). `git diff --check` and backlog validation passed. Biome reports repository configuration version/deprecation notices and existing non-null assertions in Task List tests. This verifies packaging, not manual desktop runtime/accessibility. **Next task:** M2-025 — Activity history UI. M2 exit gates remain open.

## M2-023 one-level Task subtasks — 2026-10-26

Task create and response records now carry the existing nullable `parent_task_id` through the typed native DTO and frontend API. The validated transactional service allows only active top-level parents in the same workspace and with the same Project/inbox association, preventing cycles and deeper nesting. Active children appear on the parent Task detail page; each child links back to its parent. Root details can create subtasks and show only the native returned saved record. Children cannot create nested tasks. Failed relationship reads are reported separately and can be retried while the detail form remains usable. Archiving a parent with active children, restoring a child before its parent, and Project changes that would break a relationship return conflicts.

**Verification passed:** `npm run check` (repo tests 8/8, frontend tests 94/94, typecheck and production build); targeted Task detail/List tests 14/14; `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (82/82); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; Rust format check; and `git diff --check`. Biome retains the existing CLI/schema mismatch and deprecated config notices, plus existing non-null assertion test warnings. Manual desktop UI restart, Windows runtime/accessibility and M2 milestone exit gates remain open. Existing issue #57 is pending backlog sync from this documentation update. **Next task:** M2-024 — milestones.

## M2-022 due-date and priority indicators — 2026-10-26

Task List rows and Board cards now show labelled priority badges and local due dates, with an Overdue badge only when an unfinished Task's UTC due instant precedes the current instant. Done/Cancelled, missing and invalid due values do not appear overdue. Date formatting uses the user's local timezone while persistence remains an RFC 3339 UTC instant.

**Verification passed:** `npx vitest run src/task-presentation.test.ts src/TaskListPage.behavior.test.tsx` (12/12); `npm run check` (repository tests 8/8, frontend tests 95/95, typecheck and production build); `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; Rust tests 79/79; Clippy with warnings denied; local `npm run tauri -- build` produced MSI and NSIS installers; `git diff --check`. CI [40279779952](https://github.com/bjo163/moonships/actions/runs/40279779952) passed 5/5 jobs including Windows packaging. Existing Biome schema/deprecation notices remain. Issue #56 is pending backlog sync from this documentation update. **Next task:** M2-022 accepted; M2-023 subtask relationship/UI is the next dependency-ready backlog task. Manual Windows runtime/accessibility and M2 exit gates remain open.

## M2-021 combined Task filters — 2026-10-26

The Tasks toolbar now combines Project, canonical status and canonical priority filters with M2-020's local search. Project options include Personal inbox and IDs found on Tasks when a Project name cannot be loaded, keeping those records filterable. One local filtered set drives List and Board; clearing filters restores the active task set without modifying persistence. No-match remains distinct from an empty collection or failed read.

**Verification passed:** targeted Task List/Board behaviors (8/8); `npm run check` (repository tests 8/8, frontend tests 91/91, typecheck and production build passed); `git diff --check`. Repository regression checks CI [40277420116](https://github.com/bjo163/moonships/actions/runs/40277420116) passed 5/5 jobs including Windows package smoke build. Existing Biome schema/deprecation notices and test non-null assertion warnings remain. Issue #55 is pending backlog sync from this documentation update. **Next task:** M2-022 — due-date and priority indicators. Manual Windows runtime/accessibility and M2 milestone gates remain open.

## M2-020 local Task search — 2026-10-26

The Tasks toolbar now searches active Task titles and descriptions locally without case sensitivity. A 150 ms debounce keeps keystrokes responsive; Clear and no-match states work in List and Board. The match count is shown against the active-task total, and an empty result is distinct from no active tasks or a failed read. Search only filters in-memory persisted records and does not mutate them.

**Verification passed:** targeted `npx vitest run src/TaskListPage.behavior.test.tsx` (7/7); `npm run check` (repository tests 8/8, frontend tests 90/90, typecheck and production build passed); `git diff --check`. Biome reports the existing CLI/schema mismatch/deprecated config notices and existing non-null assertion warnings. Issue #54 is pending backlog sync from this documentation update. **Next task:** M2-021 — combined Project/status/priority filters. The M2 milestone remains open, including manual Windows runtime/accessibility and restart evidence.

## M2-019 Task Board — 2026-10-26

The Task page now offers List and Board views over the same persisted Task records. Board groups tasks by all seven canonical statuses. Drag-and-drop and a keyboard-accessible status selector both use the existing task update operation and only show the returned saved record; failures retain the prior status and report the save failure.

**Verification passed:** `npm run check` (89 tests; format/lint, typecheck and production build passed); Repository regression checks CI [40274608802](https://github.com/bjo163/moonships/actions/runs/40274608802) passed all 5/5 jobs, including Rust tests, Clippy, backlog regression and Windows package smoke build. Biome reported the existing CLI/schema version warning. Implementation commit `306a14b` is pushed. Issue #53 is pending backlog sync from this documentation update. The M2 milestone remains open; manual Windows runtime/accessibility and desktop restart evidence remain outstanding. **Next task:** M2-020 — local case-insensitive Task search.

## M2-005 TaskDependency model — 2026-10-26

Migration v7 adds a workspace-scoped directed Task dependency table. A row means `task_id` is `blocked_by` the prerequisite `depends_on_task_id`; the inverse “blocks” relation is derived. The service and database reject self-dependencies and cross-workspace references. Repeating the same edge is idempotent, and dependency edges are not yet used for scheduling. Longer cycle detection remains required before scheduling depends on this graph.

**Verification passed:** targeted self/cross-workspace test; `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (79 passed); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; Rust format check; and `git diff --check`. Implementation commit `33b28e0` passed Repository regression checks CI [40272847358](https://github.com/bjo163/moonships/actions/runs/40272847358), 5/5 jobs including Windows installer packaging. Existing issue #39 closed through backlog-sync run [40273492664](https://github.com/bjo163/moonships/actions/runs/40273492664). M2-019 is accepted in the newer checkpoint above; **next task:** M2-020 — local case-insensitive Task search. Manual M2 UI/runtime and milestone exit gates remain open.

## M2-004 workspace-scoped Labels — 2026-10-26

Schema migration v6 adds workspace-scoped Labels and a Task-label join table with composite foreign keys that reject cross-workspace relationships. The native service creates/lists labels and adds, removes and lists Task-label associations. Duplicate identity is a trimmed name with whitespace collapsed and Unicode lowercase applied; the first normalized display spelling is preserved. Duplicate names conflict within one workspace and are allowed in another. Repeated association writes/removals are deterministic.

**Verification passed:** targeted relationship/duplicate/migration upgrade/rollback tests; `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (76 passed); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; and `git diff --check`. Implementation commit `967b747` passed Repository regression checks CI [40271494784](https://github.com/bjo163/moonships/actions/runs/40271494784), 5/5 jobs including Windows installer packaging. Existing issue #38 closed through backlog-sync run [40272028346](https://github.com/bjo163/moonships/actions/runs/40272028346). Label UI/IPC and deletion are outside M2-004. M2-005 and M2-019 are accepted in newer checkpoints; **next backlog task:** M2-020 — local case-insensitive Task search. M2 exit gates remain open.

## M2-032 Project-to-Task persistence smoke — 2026-10-26

`project_to_task_status_flow_persists_after_database_reopen` uses a disposable file-backed database, real schema migrations, and the validated Project/Task services. It creates a Project and linked Task, updates Task description/status/priority, closes the connection, reopens the same database, and verifies both persisted records and their association. The temporary database is removed afterward. This verifies native persistence across a database reopen; manual Windows UI restart/accessibility evidence remains open.

**Verification passed:** targeted smoke test (1/1); `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (73 passed); `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`; Rust format check; and `git diff --check`. Implementation commit `f9daa55` passed Repository regression checks CI [40270033159](https://github.com/bjo163/moonships/actions/runs/40270033159), all 5/5 jobs including Windows packaging. Documentation/backlog commit `4b2836b` passed docs CI [40270601676](https://github.com/bjo163/moonships/actions/runs/40270601676) (5/5 jobs). Backlog sync [40270601714](https://github.com/bjo163/moonships/actions/runs/40270601714) succeeded and closed existing issue #66.

The requested P0 sequence M2-011, M2-013, M2-031, M2-014, M2-016, M2-018, M2-015, M2-017, M2-028 and M2-032 is complete. Manual Windows runtime/accessibility validation and M2 milestone exit gates remain open. No secondary task is included in this checkpoint.


## M2-028 persisted Project/Task states — 2026-10-26

Explicit state behavior across Projects, Project Overview, Task List, Task detail and Task creation now has behavioral coverage. Loading reads show a loading state; successful zero results show accurate empty copy; read failures show error content rather than an empty list, with retry on safe reads. Project activity's empty result is distinct from activity-read failure. Supplemental Project-name/picker failures are separate and leave Task data usable; failed Task edits retain form values for retry. No code behavior change was needed for this acceptance task: the behaviors landed with M2-014/015/016/017/018 and this task verifies their shared acceptance.

**Verification passed:** `npm run check` (8 repository tests, 86 frontend tests, typecheck and production build); `git diff --check`. The behavior suite includes Projects unavailable-vs-empty/retry, Task List unavailable-vs-empty/retry, Project Overview empty-activity-vs-failure/retry, Task detail read failure/retry and Task creation optional-Project failure. Biome prints the existing CLI/config version warning. Documentation checkpoint CI [40269028326](https://github.com/bjo163/moonships/actions/runs/40269028326) passed 5/5 jobs; issue #51 is closed and issue #62 is closed through backlog-sync run [40269028296](https://github.com/bjo163/moonships/actions/runs/40269028296).

**At this checkpoint, the requested P0 sequence is complete; M2-032 is documented below. Manual Windows runtime/accessibility and M2 exit gates remain open.**

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
## M2-030 Project & Task import preview — 2026-10-26

Added a read-only preview action beside export on Projects and Tasks. The native Open dialog selects the file, native code reads no more than 50 MiB and validates strict export schema v1 and the 100,000-record cap. The UI receives counts, manifest metadata and up to five Project names/Task titles. No database is opened and no records are imported; malformed and unsupported content returns an error, and cancel is distinct. **Verification passed:** Rust tests 94/94, Clippy warnings denied, `cargo fmt -- --check`, targeted UI tests 4/4, `npm run check` (repository 8/8, frontend 103/103, lint/typecheck/production build), Windows MSI/NSIS packaging, backlog validation and `git diff --check`. CI [40288940951](https://github.com/bjo163/moonships/actions/runs/40288940951) passed 5/5 jobs; backlog sync [40288940959](https://github.com/bjo163/moonships/actions/runs/40288940959) closed issue [#64](https://github.com/bjo163/moonships/issues/64). Existing Biome configuration schema/deprecation notices and four existing Task List non-null assertion warnings remain. The separate import/commit workflow remains unimplemented and requires explicit confirmation in a later task.
## M2-033 Home dashboard — 2026-10-26

Replaced the Home placeholder with a local dashboard backed by `get_home_dashboard`, a no-request command scoped to the Personal workspace. It returns a UTC freshness instant, counts from saved active rows, up to five recently updated Projects, five overdue and five due-soon Tasks, and ten normal-sensitivity Activity events. React renders real saved records, localizes timestamps, links to record details, distinguishes first-run/empty/loading/error states, supports retry and reloads after a Task is created. Completed/cancelled tasks do not count as open/due; archived rows and restricted Activity are excluded. Approval requests remain unavailable and are described as such. No polling, writes, synthetic events, or health metrics were added.

**Verification passed:** `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` (96/96), Clippy warnings denied, `cargo fmt -- --check`, `npm run check` (repository 8/8, frontend 105/105, lint/typecheck/production build), Windows MSI (4.02 MiB) and NSIS (2.82 MiB) packages, backlog validation and `git diff --check`. Existing Biome configuration schema/deprecation notices and four existing Task List non-null assertion warnings remain. CI [40291006541](https://github.com/bjo163/moonships/actions/runs/40291006541) passed 5/5 jobs; backlog sync [40291006493](https://github.com/bjo163/moonships/actions/runs/40291006493) passed and closed existing issue [#67](https://github.com/bjo163/moonships/issues/67). Manual Windows UI verification is not claimed.

## M2-034 Global search — 2026-10-26

The Ctrl+K palette now searches registered commands and active local Projects/Tasks from the Personal-scoped APIs. Results are typed, keyboard-navigable and open the matching existing detail route; literal matching includes Project names/descriptions/goals and Task titles/descriptions. Independent adapters preserve successful results and allow retry when a local read fails. Verification passed: targeted frontend behaviors 15/15; `npm run check` (repository 8/8, frontend 108/108, typecheck and production build); backlog validation; and `git diff --check`. CI [40292258131](https://github.com/bjo163/moonships/actions/runs/40292258131) passed 5/5 jobs, including Windows installer packaging. Backlog sync [40292258129](https://github.com/bjo163/moonships/actions/runs/40292258129) closed existing issue [#68](https://github.com/bjo163/moonships/issues/68). This is not manual Windows WebView or usability evidence. **Next task:** M2-035 requires representative participants to complete the specified workflows and provide observed blockers; do not infer or fabricate those observations. The remaining milestone exit gates are still open.

## M3-001 Cleanup rule contract — 2026-10-26

Added a typed, metadata-only native contract in `src-tauri/src/cleaner.rs`: stable rule ID/version, category, closed allowed-root enum (app temporary, app cache, or OS-resolved user temporary), minimum age, per-item size limit and explicit eligibility/exclusion outcomes. Validation rejects malformed IDs, zero limits and category/root mismatches. Eligibility rejects wrong-root candidates, non-regular entries, future timestamps, recent files and oversized items. M3-001 verification passed targeted cleaner tests (6/6), full Rust tests (102/102), Clippy with warnings denied, Rust format, `npm run check` (repository 8/8, frontend 108/108, typecheck/build), backlog validation and `git diff --check`. Its backlog sync closed issue #69; CI [40296048122](https://github.com/bjo163/moonships/actions/runs/40296048122) has Rust tests, format/Clippy and frontend build passing, with Windows packaging still running at this handoff update.

## M3-002 Read-only cleaner traversal — 2026-10-26

Added a test-only metadata traversal helper beneath a disposable fixture root. It gathers regular-file metadata recursively, skips links without recursing, does not open file contents, and has no mutation operation. It is compiled only for tests and has no IPC exposure. The nested-fixture test confirms both files remain byte-identical after scanning. Targeted cleaner tests passed 7/7; full Rust tests passed 103/103; format, Clippy with warnings denied, backlog validation and `git diff --check` all pass. **Next task:** M3-003 protected-path and containment checks; M3-004 adds platform-specific link/junction coverage.

## M3-003 Cleaner path containment — 2026-10-26

Added `validate_cleanup_path`: canonicalizes an existing allowed root, candidate and native-policy protected paths; rejects unresolved paths, invalid roots, the root itself, outside-root traversal and protected descendants. Tests use disposable fixtures for valid paths, `..` escape, protected target, root target and missing-path fail-closed behavior. This is metadata/path resolution only and performs no mutation. Canonicalization does not close link or TOCTOU races; M3-004 and later revalidation remain required. Full Rust tests passed 105/105, Clippy with warnings denied, formatting and `git diff --check` passed. **Next task:** M3-004 symlink/junction escape behavior.

## M3-004 Cleaner junction policy — 2026-10-26

The read-only fixture test now creates a Windows directory junction pointing outside the allowlisted root. Path canonicalization rejects the junction target as outside-root, and traversal sees no candidates because it skips the junction rather than recursing. The external fixture file remains byte-identical. This Windows junction test passed locally. The test uses a fixed `mklink /J` invocation with generated disposable paths; no product code launches a process. Broader checks and backlog validation are being rerun. **Next task:** M3-005 candidate identity contract.

## M3-005 Cleanup candidate identity — 2026-10-26

Added `CleanupCandidateIdentity` with a per-scan UUID, validated relative path locator and immutable cleanup metadata snapshot (root, kind, modified time and size). Empty, absolute and parent-traversal locators are rejected. Tests show identical display locators from separate scans receive distinct IDs and preserve metadata. This is a contract only; mutation still requires fresh metadata and platform identity checks. Targeted cleaner tests passed 11/11; full Rust suite passed 107/107; format, Clippy with warnings denied and `git diff --check` passed. **Next task:** M3-006 scan progress and cancellation contract.

## M3-006 Scan progress and cancellation contract — 2026-10-26

Added serializable cleaner scan progress and terminal event DTOs keyed by the shared job UUID. Progress carries a stable stage, observed counters, and best-effort/unsupported cancellation capability; it does not invent percentages without a known total. Terminal outcomes map to the shared job statuses and use a closed error-code enum. The contract contains no paths or file contents. A `cancelled` terminal state means work stopped, not merely that cancellation was requested. This task does not wire a UI or implement scan execution. Full Rust tests passed 108/108; the final test-only correction passed targeted cleaner tests 12/12. Clippy with warnings denied, Rust formatting and `git diff --check` passed. **Next task:** M3-007 preview DTO and UI.

## M3-007 Cleanup preview DTO and UI — 2026-10-26

Added a serializable native preview row with candidate ID, category, leaf display name, byte size, modified time, eligible/skipped status and a closed skip reason. The frontend has a Clean & Storage route with an explicit no-scan state and a read-only preview table with counts, estimated bytes, status and skip reasons. It contains no delete/clean action; no data is fabricated when a preview is unavailable. Rust tests passed 109/109, focused frontend tests passed 34/34, and `npm run check` passed (repository 8/8, frontend 110/110, typecheck and production build). Rust format, full tests and Clippy also passed. **Next task:** M3-008 explicit clean approval flow.

## M3-008 Explicit cleanup approval — 2026-10-26

Added eligible-only selection and a separate confirmation dialog. Confirmed approval requests bind the preview UUID, unique candidate UUIDs and selected estimated-byte total; native validation rejects empty, duplicate and skipped-row requests. Cancel and pre-confirm states do not emit an approval request. Confirmation records approval intent only, clearly says cleanup has not started, and has no deletion callback. `npm run check` passed (repository 8/8, frontend 111/111, typecheck and production build); full Rust tests passed 110/110, Clippy with warnings denied, Rust formatting and `git diff --check` passed. **Next task:** M3-009 per-candidate revalidation.

## M3-009 Candidate revalidation — 2026-10-26

Added native candidate capture and revalidation. Capture records a platform file identity (Windows volume serial/file index; Unix device/inode) with the relative locator and metadata. Revalidation rejects missing identity, changed identity or metadata, links/non-files, invalid relative locators, containment/protected-path failures and current-rule ineligibility. Tests cover changed size, replacement with same-size content and restored mtime, out-of-policy size, invalid path, missing identity and type change. Unsupported platforms fail closed. The function returns a verified path for immediate use only and does not delete; its path-based TOCTOU limit remains explicit for M3-010. Targeted revalidation tests passed 2/2; the full Rust suite passed 112/112, Clippy with warnings denied, formatting and diff checks passed. **Next task:** M3-010 bounded deletion executor.

## M3-010 Bounded deletion executor — 2026-10-26

Added a native-only executor for the exact preview, approval and candidate set, capped at 100 items. Invalid or altered approval totals are rejected before any mutation. Each selected item is independently revalidated and returns a closed deleted/skipped/failed outcome, allowing subsequent candidates to proceed after a locked or denied file. Windows deletion opens the checked path without following its final reparse point, blocks concurrent writer/name replacement, compares the handle's resolved path, identity and metadata with the validated snapshot, and marks that exact handle for deletion. Other platforms report deletion as unsupported and leave the file untouched. The executor is not registered as a renderer command; wiring it to the native approval authority remains required before product use. A disposable fixture test confirms a locked item reports `RESOURCE_BUSY` and the next item is deleted; a separate test confirms a tampered approval leaves its file untouched. Final verification passed after the handle-path guard was added: Rust full suite 114/114, formatting check, Clippy with warnings denied, and `git diff --check`. **Next task:** M3-011 estimated-versus-actual freed-byte reporting.

## M3-011 Estimated versus actual deleted bytes — 2026-10-26

The execution report now preserves the preview estimate separately from `actual_deleted_bytes`, the sum of logical file sizes for successful deletions only. A file contributes only after the handle operation succeeds, the handle closes, and the validated path no longer exists. Locked/failed items contribute zero. This is not a claim about physical disk space reclaimed, which may differ for compressed, sparse or hard-linked files. The mixed locked/success fixture verifies that estimate exceeds actual when one file is not removed. Verification passed: Rust full suite 114/114, formatting check, Clippy with warnings denied, `git diff --check`, and backlog validation. **Next task:** M3-012 cleaner tests for locked/denied/changed files.

## M3-012 Cleaner failure-case tests — 2026-10-26

Added executor-level disposable fixture tests for a changed candidate (`Skipped(Changed)`), a Windows read-only candidate (`Failed(PermissionDenied)`), and a Windows locked candidate (`Failed(ResourceBusy)`) followed by a successful deletion of the next candidate. Tests assert failed/skipped files remain present, actual deleted bytes exclude them, and one item does not terminate the batch. The locked/denied tests are Windows-gated; the changed-candidate test runs on all platforms. Verification passed: full Rust suite 116/116, formatter, Clippy with warnings denied, `git diff --check`, and backlog validator. **Next task:** M3-013 safe temporary-file rule set.

## M3-013 Safe temporary-file rule set — 2026-10-26

Added a native registry with two valid defaults: `moonships-temp-stale` is limited to app-owned `temporary/` files at least seven days old and no larger than 64 MiB; `moonships-cache-stale` is limited to app-owned regenerable `cache/` files at least thirty days old and no larger than 128 MiB. The shared OS temporary directory is deliberately not in the default set. `SECURITY_MODEL.md` names each target and exclusions, including persistent project/task data, logs, settings, credentials, browser state, downloads and user documents. The rules do not enable scanning or deletion through the UI. The contract test verifies valid roots, age/size boundaries and omission of the shared user-temp root. Verification passed: Rust full suite 117/117, formatting check, Clippy with warnings denied, and `git diff --check`; backlog validation is being rerun before commit. **Next task:** M3-014 cleanup report and optional task link.

## M3-014 Cleanup report and optional Task link — 2026-10-26

Added schema v9 for bounded cleanup report summaries, a native-only wrapper that saves a report after the approved executor finishes, and a Personal-workspace history read command. Stored and displayed records contain outcome totals, estimated bytes, successfully deleted logical bytes, completion time and an optional same-workspace Task link; they omit candidate IDs, names and paths. Clean & Storage has loading, retryable error, empty and populated history states. The history view does not initiate scanning or deletion. Verification passed: Rust full suite 121/121 (including Windows execution-and-report fixture), Clippy with warnings denied, Rust formatting, `npm run check` (repository 8/8, frontend 113/113, typecheck and production build), migration upgrade test, backlog validation and `git diff --check`. Existing Biome schema/deprecation notices and four Task List assertions remain. **Next task:** M3-015 JSON formatter/validator.

## M3-015 JSON formatter/validator — 2026-10-26

Added a DevTools page and pure in-process JSON utility. It validates with the built-in parser and formats by changing whitespace only, preserving large numeric and string tokens. Limits are 1 MiB UTF-8 input, 128 nesting levels and 4 million output characters; empty, malformed, oversized, deeply nested and over-expanded results have distinct structured errors. The screen has a labelled editor, keyboard-accessible read-only output and no provider/network/filesystem call. Verification passed: focused formatter/page/route tests 10/10; `npm run check` passed repository tests 8/8, frontend tests 123/123, lint, typecheck and production build; backlog validation and `git diff --check`. Existing Biome schema/deprecation notices and four Task List non-null assertion warnings remain. **Next task:** M3-016 YAML formatter/validator; parser choice must be reviewed and explicitly vetted before implementation.

## M3-016 YAML formatter/validator — 2026-10-26

Added YAML as a second local tool in DevTools. The explicitly chosen dependency is exact-pinned `yaml@2.9.1` (ISC; upstream eemeli/yaml; no runtime dependencies), recorded in `docs/THIRD_PARTY_NOTICES.md`. Parsing uses strict YAML 1.2/core with duplicate-key checks; syntax errors return stable parser codes and line/column offsets without exposing source excerpts. Input is capped at 1 MiB UTF-8 and formatted output at 4 million characters. Aliases remain syntax and are not expanded into JavaScript objects. Verification passed: YAML/page/route/App focused tests 41/41; `npm run check` passed repository tests 8/8, frontend 129/129, lint, typecheck and production build; backlog validation and `git diff --check`. Existing Biome schema/deprecation notices and four Task List non-null assertion warnings remain. The install command reported three advisories in the dependency tree; `npm audit --omit=dev` could not run because this repository has no npm lockfile. The YAML package itself reports no runtime dependencies. **Next task:** M3-017 Base64 encode/decode.

## M3-017 Base64 encode/decode — 2026-10-26

Added three local modes: UTF-8 text to Base64, hex bytes to Base64, and Base64 to decoded bytes. Decoding requires canonical padded standard Base64, classifies only fatal-valid UTF-8 as text, and otherwise reports binary with byte count and at most a 64-byte hex preview. Byte input/output is limited to 1 MiB; malformed Base64 and hex return structured errors. Verification passed: focused Base64 and DevTools UI tests 12/12; `npm run check` passed repository tests 8/8, frontend tests 138/138, lint, typecheck and production build; backlog validation and `git diff --check`. Existing Biome schema/deprecation notices and four Task List non-null assertion warnings remain. **Next task:** M3-018 URL encode/decode.

## M3-018 URL encode/decode — 2026-10-26

Added distinct component and full URL modes. Components use `encodeURIComponent`/`decodeURIComponent`, so reserved characters become data; full URLs use the URL parser and `decodeURI`, which preserve structural delimiters and require an absolute URL. UTF-8 input and output are bounded; malformed percent escapes and invalid URLs return distinct structured errors. The utility never opens or fetches the target. Verification passed: focused URL/UI tests 10/10; `npm run check` passed repository tests 8/8, frontend tests 144/144, lint, typecheck and production build; backlog validation and `git diff --check`. Existing Biome schema/deprecation notices and four Task List non-null assertion warnings remain. **Next task:** M3-019 UUID generator/validator.

## M3-019 UUID generator/validator — 2026-10-26

Added local UUID v4 generation backed by Web Crypto `randomUUID()` and canonical UUID validation. Generation fails closed when the API is missing, throws or returns malformed output; no insecure random fallback exists. Validation accepts UUID versions 1–8 using the RFC variant, plus the Nil UUID, matching the format stated in the UI. The tool uses no provider, network or filesystem capability. Verification passed: focused UUID/UI tests 11/11; `npm run check` passed repository tests 8/8, frontend 150/150, typecheck and production build; backlog validator and `git diff --check`. Existing Biome schema/deprecation notices and four Task List assertions remain. **Next task:** M3-020 SHA-256 file/text hash utility.

## M3-020 SHA-256 file/text hash utility — 2026-10-26

Added local SHA-256 text and file hashing using exact-pinned RustCrypto `sha2` 0.10.9 (Apache-2.0/MIT; recorded in third-party notices). Text input is bounded at 1 MiB. On Windows, a native file picker selects the file and native code streams up to 2 GiB in a fixed 64 KiB buffer; the renderer cannot pass a path, and only the leaf name, bytes read and digest are returned. Picker cancellation, oversize input, unsupported-platform and read-error states are explicit. Verification passed: SHA-256 Rust known vectors and multi-chunk/size-bound tests 3/3; full Rust suite 124/124; Clippy with warnings denied; Rust formatting; focused UI tests 4/4; `npm run check` (repository 8/8, frontend 154/154, lint, typecheck and production build); backlog validation and `git diff --check`. Existing Biome schema/deprecation notices and four Task List assertion warnings remain. **Next task:** M3-021 text diff utility.

## M3-021 Text diff utility — 2026-10-26

Added local unified text comparison using exact-pinned `diff@9.0.0` (BSD-3-Clause; recorded in third-party notices). Inputs are bounded to 1 MiB UTF-8 and 50,000 lines each; calculation has a maximum edit distance of 1,000 and the displayed output is capped at 4 million characters. Insertions, deletions, identical inputs, empty inputs, oversized input, line limits and excessive edit distance are tested. Verification passed: focused utility/UI tests 7/7; `npm run check` passed repository tests 8/8, frontend tests 161/161, lint, typecheck and production build; backlog validation and `git diff --check`. npm reported three advisories across the existing dependency tree; `diff@9.0.0` itself has no runtime dependencies, and this repository has no npm lockfile for a lockfile-based audit. Existing Biome schema/deprecation notices and four Task List assertion warnings remain. **Next task:** M3-022 timestamp converter.

## M3-022 Timestamp converter — 2026-10-26

Added local Unix timestamp (seconds or milliseconds) and strict ISO 8601 conversion. Inputs must be whole safe integers or real calendar date-times with an explicit known timezone offset. Outputs always label normalized UTC ISO, device-local time with its resolved timezone, and Unix values in both units. Invalid syntax, dates, ranges, missing offsets and unknown `-00:00` offsets produce clear errors. Verification passed: focused conversion/UI tests 6/6; `npm run check` passed repository tests 8/8, frontend tests 167/167, lint, typecheck and production build; backlog validation and `git diff --check`. Existing Biome schema/deprecation notices and four Task List assertion warnings remain. **Next task:** M3-023 DevTools tests and offline acceptance check.

## M3-023 DevTools offline acceptance — 2026-10-26

Added an integrated acceptance test that runs JSON, YAML, Base64, URL, UUID, SHA-256 text/file bridge, text diff and timestamp flows while `fetch` throws as if browser networking were unavailable. The test confirms both local hash IPC calls are the only native commands and no `fetch` call occurs. Verification passed: offline acceptance test 1/1; `npm run check` passed repository tests 8/8, frontend tests 168/168, lint, typecheck and production build; backlog validation and `git diff --check`. A first full run had one existing Task List search assertion race; the repeat passed. Existing Biome schema/deprecation notices and four Task List assertion warnings remain. Limitation: this acceptance test disables browser `fetch` only; it does not turn off the operating system's network adapter. The code paths under test make no network calls. **Next task:** inspect the next dependency-ready backlog task after M3-023.

## M4-001 Bounded network diagnostic contract — 2026-10-26

Added native request validation and typed provenance-bearing result contracts for one DNS lookup or one TCP host/port check. Targets must be bare ASCII DNS hostnames (IDNs as punycode) or IP literals; URLs, embedded ports, wildcard/range targets and malformed names fail validation. Timeouts are 100–10,000 ms (2,000 ms default); the future adapter policy allows one concurrent run, ten diagnostics per minute and at most 32 returned IP addresses. Result provenance records the operation/target, UTC epoch-millisecond observation time, duration, resolver/socket source, outcome and stable error code. No network call or executable IPC command was added. Verification passed: four focused target/port/timeout/limit tests; full Rust suite 128/128; Clippy with warnings denied; Rust formatting; backlog validation and `git diff --check`. **Next task:** M4-002 DNS lookup adapter.

## M4-002 DNS lookup adapter — 2026-10-26

Added an internal read-only system-resolver adapter with a single bounded worker, one queued slot, the shared ten-per-minute limit, request deadlines, and a 32-address deduplicated result cap. Invalid target validation errors remain distinct from resolver failure results; timeout and unavailable worker states are separately typed. IP literals return directly with explicit provenance and do not invoke DNS. No Tauri command exposes the adapter. Limitation: the operating-system resolver API cannot cancel an in-flight lookup; the caller still returns at its deadline, while a hung lookup occupies the sole worker and new worker threads are not spawned. Verification passed: Rust suite 132/132; Clippy with warnings denied; Rust formatting; backlog validation (103/238 checked); and `git diff --check`.

## M4-003 TCP connectivity test — 2026-10-26

Added one-host/one-port TCP checks to the same bounded worker used for DNS, sharing its queue, timeout and ten-per-minute limit. The adapter tries resolved addresses only until the original deadline and distinguishes success, timeout, connection refusal, resolver errors and permission failures. Tests exercise a successful loopback listener and the error-code mappings without contacting external hosts. No IPC command is registered. Verification passed: Rust suite 133/133; Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.

## M4-004 Ping capability detection — 2026-10-26

Added typed ping capability detection. On Windows it probes whether the native IP Helper ICMP handle can be opened, closes it immediately, sends no echo packet, and does not assume a `ping.exe` installation. Unsupported platforms and unavailable APIs carry explicit reasons. Verification passed: Rust suite 135/135; Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.

## M4-005 Traceroute deferred status — 2026-10-26

Recorded traceroute as explicitly deferred because no reviewed adapter yet enforces both hop and wall-clock limits. The native contract returns a typed deferred reason; it does not start `tracert`/`traceroute`, substitute another process, or report a successful result. Verification passed: Rust suite 136/136; Clippy with warnings denied; Rust formatting; backlog validation (106/238 checked); and `git diff --check`.

## M4-006 IPv4 subnet calculator — 2026-10-26

Added strict single-IPv4-CIDR arithmetic for network address, netmask, wildcard mask, traditional broadcast, usable address range and counts. `/31` treats both addresses as usable and `/32` as one address; broadcast is absent for both. Invalid address/family/prefix and malformed slash inputs return typed errors. This utility is offline and is not exposed as an executable IPC command. Verification passed: Rust suite 139/139; Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.

## M4-007 Read-only system inventory — 2026-10-26

Added typed inventory fields for OS family, process architecture, available logical processors, physical memory totals and the Windows directory volume's total/free capacity. Windows uses native `GlobalMemoryStatusEx`, `GetWindowsDirectoryW` and `GetDiskFreeSpaceExW`; no arbitrary path is accepted or returned. Each value has an explicit available/unsupported/unavailable state and unsupported/failing values are null. No file, device or process enumeration or IPC command was added. Verification passed: Rust suite 141/141; Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.

## M4-008 Read-only process inventory — 2026-10-26

Added a Windows Tool Help process snapshot and a Processes page with one-time load and user-triggered refresh. The native command has no input and returns at most 4,096 entries with name, PID, parent PID and thread count; no individual process handle, command line, executable path or mutation action is present. The page has explicit loading, empty, partial, permission, unsupported, unavailable and retry states, and rows contain no action controls. Verification passed: Rust suite 143/143; frontend suite 173/173; `npm run check` (format, lint, repo tests, frontend tests, typecheck and production build); Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`. Biome emitted the repository's existing non-fatal notice that its schema is 2.2 while the installed CLI is 2.6.3.

## M4-009 Windows service inventory — 2026-10-26

Added a local Service Control Manager snapshot and a Services page with user-triggered refresh. The native adapter requests enumeration-only access, returns service/display names and current state, caps results at 2,048, and closes its manager handle. It does not open individual service handles or expose start/stop/restart/configuration actions. Permission denied, unavailable, unsupported, empty and partial states are separate in the UI. Verification passed: Rust suite 146/146; frontend suite 178/178; `npm run check` (format, lint, repo tests, frontend tests, typecheck and production build); Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`. Biome emitted the repository's existing non-fatal notice that its schema is 2.2 while the installed CLI is 2.6.3.

## M4-010 Event log provider seam — 2026-10-26

Added a fixed-channel, 100-record-bounded provider contract with a 100–5,000 ms requested deadline and privacy-minimized event summary DTO. Empty, permission-denied, timed-out, unavailable, unsupported and partial states map distinctly; event message text and payload are omitted. This task deliberately wires no OS provider, IPC command or UI, so it does not read event logs. Verification passed: Rust suite 149/149; Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.

## M4-011 Allowlisted connection profile contract — 2026-10-26

Added tagged Terminal/SSH/RDP profile types, strict host/port/name validation, and allowlisted terminal applications. Credential slots accept only UUID secret references; profile imports reject unknown fields, the export DTO omits references, and the safe log projection omits user content/target details. No secret values, persistence, OS credential store or connection launch are implemented here. Verification passed: Rust suite 154/154; Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.

## M4-012 Activity Center — 2026-10-26

Added schema v10 for bounded local job history, a native transition service, and an Activity Center route that reads the newest 200 records. The UI shows truthful status, reported progress, recorded duration and result, with loading, empty and retryable error states. Terminal states cannot reopen; timestamps are recorded on first run and terminal completion. No sample records are created. No job producers are wired yet, and restart recovery/cancellation remain follow-up work; interrupted jobs are not falsely completed. Verification passed: Rust suite 157/157; frontend suite 181/181; `npm run check` (format, lint, repo tests, frontend tests, typecheck and production build); Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`. Existing Biome schema/CLI compatibility notices are informational.

## M4-013 Job cancellation — 2026-10-26

Activity Center displays a disabled Cancel action for saved jobs and explains that no connected runner currently supports cancellation. No cancellation command or false `cancelled` state is exposed; a future runner must confirm that work stopped before reporting the terminal status. Verification passed: frontend suite and `npm run check`; backlog validation; and `git diff --check`.

## M4-014 Network diagnostic follow-up Task — 2026-10-26

Added a NetOps page for one DNS lookup or one TCP host/port check, calling the existing bounded adapter with a 2-second timeout. The native process caches at most 100 result handles; Task creation takes only a handle and constructs the summary from the stored native result. The inbox Task links target, outcome and observed time while omitting resolved addresses and raw system messages. Invalid, unavailable, rate-limited and expired-result cases are bounded and visible. Verification passed: Rust suite 159/159; frontend suite 184/184; `npm run check` (format, lint, repo tests, frontend tests, typecheck and production build); Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`. Existing Biome schema/CLI compatibility notices are informational.

## M4-015 Network diagnostic regression tests — 2026-10-26

Added a malformed-DNS request test that stops before resolver work and an already-expired TCP deadline test using `127.0.0.1`. The local TCP integration test checks success and accepts the OS-specific refusal or timeout outcome after a loopback listener closes; closed error mapping is asserted independently. The rate window is tested with injected monotonic timestamps at the 60-second boundary. No test contacts a public host. Verification passed: Rust suite 161/161; Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.

## M5-001 Connector/source contract — 2026-10-26

Added typed connector kind, access method, terms review, health and bounded policy fields. RSS/API/authorized-export kinds must match the access method. Enabled connectors require a valid stable ID, reviewed terms, HTTPS-only references without embedded credentials, poll cadence, request timeout, response/item caps and freshness threshold; both `enable()` and enabled-state deserialization fail closed, and blocked sources cannot be re-enabled by resetting health. No source is fetched, scheduled or persisted. Verification passed: Rust suite 168/168; Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.

## M5-002 Normalized intelligence item schema — 2026-10-26

Added a strict typed intelligence item DTO with stable item/source IDs, separate source and item URLs, bounded external ID/title/summary/category, content hash, freshness and verification state. Publication, observation and event timestamps are distinct explicit-zone RFC 3339 values normalized to UTC; missing publication/event times remain absent. URL canonicalization, deduplication, persistence and connector fetching are not included. Verification passed: Rust suite 171/171; Clippy with warnings denied; Rust formatting; targeted item-schema tests; backlog validation; and `git diff --check`.

## M5-003 User-configured RSS connector — 2026-10-26

Added a single-call RSS 2.0 adapter with an in-process poll limiter, one bounded DNS resolver worker, public-address filtering and DNS pinning, HTTPS-only/no-proxy/no-redirect HTTP, request timeout, response and item limits, and strict bounded XML parsing. The normalized items retain feed attribution, source publication time and observation time separately. A local fixture is based on the officially documented JPL CNEOS public RSS feed; no network request is made by tests, and the feed is not enabled or fetched automatically. The adapter is native-callable but has no IPC/UI wiring, scheduler, durable rate-limit state or persistence. Verification passed: Rust suite 179/179; eight focused RSS tests; Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.

## M5-004 HTTP source timeout, size cap and backoff — 2026-10-26

RSS response reads enforce the declared total timeout and byte limit, with stable `RequestTimedOut` and `ResponseTooLarge` failures and no partial parsed result. Per-source failures now apply exponential backoff beginning at 60 seconds and capped at one hour, combined with the source's minimum cadence; no automatic retry occurs and success resets the failure count. Backoff state remains bounded and process-local. Verification passed: Rust suite 181/181; ten focused RSS tests including exponential backoff/reset, timeout, response cap and cadence; Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.

## M5-005 Canonical URL and content fingerprint — 2026-10-26

Added conservative HTTPS URL identity normalization while retaining each exact item URL as `original_url`; validation checks that the stored canonical URL matches the normalization. URL normalization strips fragments and a fixed analytics-parameter allowlist only. Added a versioned SHA-256 fingerprint over title and optional summary with whitespace collapse and unambiguous field boundaries; the RSS adapter now uses it. Fixtures verify equivalent tracked URLs produce the same identity and preserve original URL/source attribution, and that content hashes are stable and field-separated. No deduplication or persistence is added. Verification passed: Rust suite 188/188; identity, item-schema and RSS fixture suites passed (20 tests total); Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.

## M5-006 Source-preserving item deduplication — 2026-10-26

Added a 2,000-record-bounded, in-memory grouping helper keyed by shared canonical URL or recomputed title/summary fingerprint. Every validated record remains in input order within its group, and each group exposes unique source IDs in first-seen order. External hashes are not trusted for identity. No persisted records are deleted or merged. Verification passed: Rust suite 192/192; four focused grouping tests for shared URLs, matching content, transitive groups, invalid/oversized batches and source-reference retention; Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.

## M5-007 Connector freshness and last-success status — 2026-10-26

Added a serializable freshness view with normalized last-success time and an `is_live` flag that is true only for an enabled source inside its freshness window. Missing, invalid and future timestamps, stale records, and blocked/rate-limited/unavailable/error states cannot appear live. RSS fetch attempts now update the supplied connector's in-memory attempt time, last success and health; failures preserve prior success while setting a non-live health state. These values are not persisted or rendered by a source screen yet; M5-008/M5-012 own those follow-ups. Verification passed: Rust suite 197/197; four freshness and eleven RSS tests; Clippy with warnings denied; Rust formatting; backlog validation; and `git diff --check`.
