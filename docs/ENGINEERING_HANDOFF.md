# Moonships — Engineering Continuity & Foundation Evidence

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
