# Moonships — Engineering Continuity & Foundation Evidence

## Current checkpoint — M2-008 project update/archive accepted (2026-10-26)

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
