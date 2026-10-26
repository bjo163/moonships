# Atomic Task Backlog

**Purpose:** executable source of truth for coding agents, including small/low-context agents.  
**Status:** all implementation tasks below are open until code and acceptance evidence exist.  
**Do not treat the roadmap or a task title as proof that a feature exists.**

## Working rules

- Pick **one task ID** at a time. Read its dependencies and linked design sections before editing.
- Prefer tasks marked **S** (one small behavior/file/contract, ideally 15–45 minutes). **M** means a bounded task that may need splitting before implementation; split it if one agent cannot implement and test it independently.
- Do not include unrelated cleanup, speculative abstractions, or dependency upgrades in a task.
- Write/adjust tests with the behavior. Run the smallest relevant check, then wider checks where available.
- Do not silently change acceptance criteria. If a design conflict is discovered, stop and propose a doc/ADR change.
- Mark an item checked only after the acceptance criteria pass and the implementation is committed/reviewable.
- When finishing, report task ID, files changed, tests run/result, limitations, and the next unblocked task.
- Keep actual GitHub issue state synchronized through the idempotent workflow in [TASK_ISSUE_SYNC.md](TASK_ISSUE_SYNC.md); this checklist remains the canonical task-status source. Generated issue descriptions include task scope, acceptance criteria, estimated size, and declared dependency IDs.
- Treat task IDs as permanent sync keys: do not renumber or remove a published task ID casually. Duplicate IDs, title collisions without a marker, malformed managed bodies, or managed issue IDs absent from the backlog must stop sync for explicit reconciliation. A no-op sync must leave issue count, issue IDs, and issue states unchanged.

**Estimate:** S = 1 small independently reviewable change; M = split when practical. Estimates are planning hints, not deadlines.

---

## M0 — Product contract and repository hygiene

- [x] **M0-001 — Verify the documentation index** [S] — Check all README links resolve to repository files. **AC:** no broken internal Markdown links.
- [x] **M0-002 — Check menu names against module catalogue** [S] — Compare UX menu and Product Blueprint. **AC:** every listed module has a domain/roadmap status; no menu implies an unbuilt feature is live.
- [x] **M0-003 — Choose project license or explicitly defer it** [S] — Record owner decision; do not invent a license. **AC:** decision and implications are recorded; no misleading license badge exists.
- [x] **M0-004 — Create ADR template** [S] — Add a short template for status, context, decision, alternatives and consequences. **AC:** one example/template is linked from architecture docs.
- [x] **M0-005 — Record desktop-stack decision** [S] — Create ADR for Tauri 2 + Rust + React/TS + Vite. **AC:** trade-offs, alternatives and revisit triggers are written down.
- [x] **M0-006 — Record local-first and cloud-sync non-goals** [S] — Make data-flow assumptions explicit. **AC:** docs say core works without account/required backend.
- [x] **M0-007 — Record World Monitor licensing boundary** [S] — Document that its source implementation is not copied without legal review. **AC:** source link and AGPL-3.0-only caution are present.
- [x] **M0-008 — Define supported Windows test matrix** [S] — Set primary/recommended OS and best-effort OS. **AC:** Windows 11 primary and Windows 10 lifecycle note documented.
- [x] **M0-009 — Define task completion evidence rule** [S] — Add PR/task template language. **AC:** every code task requires tests or a reason tests do not apply.

## M1 — Desktop foundation

### Bootstrap and developer workflow
- [ ] **M1-001 — Create Tauri 2 + React + TypeScript + Vite skeleton** [S] [depends: M0-005] — Use official current template. **AC:** dev app launches from a clean clone.
- [x] **M1-002 — Add package-manager and runtime version policy** [S] — Document supported Node/package manager and Rust stable policy. **AC:** fresh setup instructions name prerequisites and verification commands.
- [x] **M1-003 — Add root development scripts** [S] — Provide consistent dev, lint, test and build commands. **AC:** script names are documented and invoke real tools only.
- [x] **M1-004 — Add Rust formatting/lint baseline** [S] — Configure rustfmt and clippy commands. **AC:** CI/local documented command fails on format/lint errors.
- [x] **M1-005 — Add frontend formatting/typecheck baseline** [S] — Configure the chosen formatter/linter and TypeScript check. **AC:** one documented command runs these checks.
- [x] **M1-006 — Add Windows CI smoke build** [S] — Build app in GitHub Actions on Windows. **AC:** workflow runs on pushes and pull requests and fails on build failure.
- [x] **M1-007 — Add minimal test commands to CI** [S] — Run starter Rust and frontend tests. **AC:** CI shows separate, understandable test steps.

### App shell and system tray
- [x] **M1-008 — Implement main AppShell layout** [S] — Sidebar, top bar and content slot only. **AC:** route/page content can be swapped without duplicating shell.
- [x] **M1-009 — Add route registry for implemented pages** [S] — Use typed route IDs. **AC:** invalid route falls back to a not-found page instead of crashing.
- [x] **M1-010 — Add navigation menu from UX spec** [S] — Show only implemented items or explicit Planned state. **AC:** no dead menu links.
- [x] **M1-011 — Implement foundational semantic design tokens** [S] — Implement the initial theme, spacing and typography tokens from DESIGN_SYSTEM.md. **AC:** shell uses semantic variables rather than scattered literal values; dark/light preference works; proposed colors are explicitly not marked contrast-certified until measured.
- [x] **M1-012 — Add dark and light theme toggle** [S] — Persist non-secret preference. **AC:** theme remains selected after restart.
- [x] **M1-013 — Add accessible focus and keyboard navigation** [S] — Sidebar, buttons, dialogs and search. **AC:** primary shell can be operated without a mouse.
- [x] **M1-014 — Implement tray icon and context menu** [S] — Open, Settings and Quit for first pass. **AC:** menu actions execute correctly in a Windows dev build.
- [x] **M1-015 — Implement hide-to-tray preference** [S] — Avoid implicit app exit on X when configured to hide. **AC:** window hides and can be restored; preference is persisted.
- [ ] **M1-016 — Implement explicit Quit lifecycle** [S] — Stop workers and exit cleanly. **AC:** Quit ends process; ordinary Hide does not.
- [ ] **M1-017 — Add single-instance behavior or document/test alternative** [S] — Avoid confusing duplicate windows. **AC:** second launch follows the chosen documented behavior.
- [x] **M1-018 — Defer startup-at-login for the MVP** [S] — Keep automatic startup disabled; revisit only as an explicit later feature. **AC:** startup-at-login is documented as deferred and no automatic startup behavior is enabled in the MVP.
- [x] **M1-019 — Add notification/permission settings skeleton** [S] — Define controls without implying unimplemented monitoring. **AC:** each control has a real effect or clear Planned state.

### Core contracts and persistence
- [x] **M1-020 — Define stable application error codes** [S] — Implement typed errors and UI mapping for the initial app. **AC:** UI logic does not parse English error strings.
- [x] **M1-021 — Add typed Tauri command DTO conventions** [S] — Document request/response shape and validation. **AC:** first command demonstrates the pattern.
- [x] **M1-022 — Add SQLite connection and per-user storage path** [S] — Use a platform-appropriate app data directory. **AC:** database is not created in arbitrary current working directory.
- [x] **M1-023 — Add transactional schema migration runner** [S] — Start with initial migration/version table. **AC:** fresh database migration passes and schema version is stored.
- [x] **M1-024 — Enable SQLite foreign keys and busy timeout** [S] — Apply on every connection. **AC:** a test proves a foreign-key violation is rejected.
- [x] **M1-025 — Add structured local logging with redaction seam** [S] — Stable log levels/categories. **AC:** logs do not include known secret fields in tests.
- [x] **M1-026 — Define Activity Center/job status types** [S] — Queued, running, waiting_approval, succeeded, partial, failed, cancelled, timed_out. **AC:** serialization tests lock the names/shape.
- [x] **M1-027 — Define ToolDefinition and ToolResult types** [S] — Include ID/version/schema/risk/permissions and structured outcome. **AC:** sample tool can be registered in a unit test.
- [x] **M1-028 — Add capability/permission policy abstraction** [S] — Policy decision independent from frontend. **AC:** tests cover allowed, denied and approval-required results.
- [x] **M1-029 — Add module availability states** [S] — Available, disabled, unsupported, not_configured, degraded. **AC:** UI can distinguish unavailable from empty success.
- [x] **M1-030 — Add config schema version and defaults** [S] — Versioned non-secret settings. **AC:** invalid setting values fail validation and defaults are tested.

- [x] **M1-031 — Implement the Command Palette surface** [S] — Add an accessible keyboard-first overlay, query field, result list, category and risk labels. **AC:** opens, filters, navigates results with the keyboard, and has loading/empty states without executing a tool.
- [x] **M1-032 — Register safe navigation and create commands** [S] [depends: M1-009,M1-027,M1-031] — Register page navigation and safe create actions such as New Project, New Task and New Note as typed commands. **AC:** only registered commands appear; unknown IDs are rejected; risky tools open a preview instead of executing from the palette.
- [x] **M1-033 — Bind Ctrl+K and palette focus lifecycle** [S] [depends: M1-031] — Open the palette from the main window, focus search, support Escape and restore focus. **AC:** shortcut does not interfere with typing in editable controls or nested dialogs.
- [x] **M1-034 — Implement first-run onboarding and workspace profile selection** [S] [depends: M1-030] — Offer Developer, Network/NOC, Sysadmin, Security/Research, Intelligence/Research and Custom profiles that change pinned tools/layout only. **AC:** profile selection persists, can be changed later, and never silently grants permissions.
- [x] **M1-035 — Add idempotent backlog-to-GitHub-Issues automation** [S] — Add a GitHub Actions workflow that mirrors unchecked backlog tasks to issues by stable task ID. **AC:** unchecked tasks create one issue keyed by stable ID; a repeated no-op run creates/updates/closes/reopens zero issues; checked tasks close their generated issue; the dry-run mode gates all write calls; only main backlog/milestone pushes or manual dispatch trigger sync.
- [x] **M1-036 — Pin backlog-sync actions and automate update review** [S] — Pin each used GitHub Action to a verified full commit SHA from its official upstream repository and configure Dependabot for GitHub Actions. **AC:** action refs are full 40-character SHAs with documented release tags; the Dependabot config schedules update PRs; backlog validation and the pinned workflow complete successfully on main.
- [x] **M1-037 — Add regression tests for sync reconciliation edge cases** [M] [depends: M1-035] — Add a testable decision layer or mock harness for the backlog-to-issue sync. **AC:** tests cover duplicate task IDs/markers, orphaned managed issue IDs, unmarked title collisions, malformed managed body markers, dry-run zero-write behavior, and a stable no-op plan; invalid cases stop before issue create/update calls.

- [ ] **M1-038 — Define and test minimum-window and overflow behavior** [S] [depends: M1-008,M1-010] — Turn the UX viewport guidance into an explicit minimum window size, resize rules, sidebar collapse behavior, and overflow behavior for tables/detail panels. **AC:** the decision is documented; the shell remains usable at the chosen minimum size and at 1280×720/1440×900; no primary action is clipped or unreachable by keyboard.
- [ ] **M1-039 — Add repeatable visual and accessibility review for the desktop shell** [S] [depends: M1-008,M1-011,M1-013] — Establish a review matrix for shell, onboarding, Command Palette, settings and dialogs; check token usage, focus, keyboard behavior, semantic labels, theme contrast and reduced motion. **AC:** review steps and actual results are recorded; contrast is measured against WCAG 2.2 AA targets where applicable; no conformance claim is made without evidence.
- [ ] **M1-040 — Validate the desktop shell with task-based usability checks** [S] [depends: M1-008,M1-031,M1-034] — Check first launch, primary navigation, opening/closing Command Palette, changing profile/layout preferences, and tray hide/restore/quit behavior. **AC:** a short findings note records the test environment, observed blockers and follow-up task IDs; no user data is collected unnecessarily.
- [x] **M1-041 — Implement shared component variants and interaction states** [S] [depends: M1-008,M1-011,M1-013] — Implement the core button, input, badge, dialog, toast, empty/loading/error and progress patterns from DESIGN_SYSTEM.md. **AC:** components use semantic tokens; keyboard/focus/disabled/loading/invalid states are consistent; tests cover primary interaction and a failure/edge case; no module creates a parallel component system.
- [ ] **M1-042 — Create original app/tray identity and asset manifest** [S] [depends: M1-008,M1-011] — Define a simple original Moonships mark and app/tray icon variants; add asset provenance/license/status records. **AC:** icon is legible at 16×16 and normal app sizes in both themes; all assets have documented rights/source or remain explicitly proposed; no third-party brand assets are copied.
- [ ] **M1-043 — Add sound preferences and central SoundService contract** [S] [depends: M1-030,M1-041] — Add locally persisted master-off default, mode, volume and user-triggered preview; route semantic event requests through one service. **AC:** master-off suppresses every event; missing audio never fails an operation; no audio plays on first launch; unit tests cover settings, dedupe and fallback.
- [ ] **M1-044 — Produce and validate the initial licensed SFX pack** [S] [depends: M1-043] — Add only a small set of original/licensed semantic cues and an audio manifest after format, rights and loudness review. **AC:** every shipped file has provenance/license metadata; packaged Windows tests cover opt-in, throttling, duplicate events, mute, missing assets and visible/text alternatives; otherwise events remain no-op.
- [ ] **M1-045 — Add reproducible UI screenshot review workflow** [S] [depends: M1-008,M1-011,M1-039] — Define screenshot capture/review for the real Windows build across theme, viewport and core shell states. **AC:** evidence records commit, OS/scaling, window size, theme and flow; known defects cannot be silently accepted as new baselines; screenshots are not treated as behavior/accessibility tests.

## M2 — Projects & Tasks

### Data and domain services
- [x] **M2-001 — Add Workspace model/migration** [S] [depends: M1-023] — Seed one Personal workspace. **AC:** fresh DB includes exactly one documented default workspace.
- [x] **M2-002 — Add Project model/migration** [S] [depends: M2-001] — Name, goal/description, status and timestamps. **AC:** constraints and defaults are tested.
- [x] **M2-003 — Add Task model/migration** [S] [depends: M2-001] — Project optional; title required; canonical status/priority. **AC:** task can exist in the personal inbox without a project.
- [x] **M2-004 — Add Label and task-label relationship** [S] [depends: M2-003] — Workspace-scoped labels. **AC:** duplicate label rules are documented and tested.
- [x] **M2-005 — Add TaskDependency model** [S] [depends: M2-003] — Prevent self-dependency. **AC:** self-dependency is rejected and tested.
- [x] **M2-006 — Add ActivityEvent storage** [S] [depends: M1-023] — Link events to project/task updates. **AC:** event timestamp and entity reference are persisted.
- [x] **M2-007 — Add repository/service for creating a project** [S] [depends: M2-002] — No UI coupling. **AC:** unit test covers create and validation.
- [x] **M2-008 — Add repository/service for updating/archiving a project** [S] [depends: M2-007] — Avoid accidental hard deletion. **AC:** update/archive behavior is tested.
- [x] **M2-009 — Add repository/service for creating a task** [S] [depends: M2-003] — Apply canonical defaults. **AC:** minimal title-only create works.
- [x] **M2-010 — Add repository/service for updating a task** [S] [depends: M2-009] — Validate state transitions. **AC:** invalid status/priority is rejected.
- [x] **M2-011 — Add task archive/restore behavior** [S] [depends: M2-010] — Keep history. **AC:** archived items can be filtered and restored.
- [x] **M2-012 — Add task status transition helper** [S] [depends: M2-003] — One canonical transition policy. **AC:** views cannot define their own incompatible status set.
- [x] **M2-013 — Add due date/time serialization rules** [S] [depends: M2-003] — UTC persistence, local display. **AC:** tests cover round trip.
- [x] **M2-014 — Add project list/create/edit UI** [S] [depends: M1-008,M2-007] — Basic fields only. **AC:** project persists after app restart.
- [x] **M2-015 — Add project overview page** [S] [depends: M2-014] — Show goal, status, count and recent activity. **AC:** zero-task state is meaningful and accurate.
- [x] **M2-016 — Add task creation dialog/form** [S] [depends: M2-009] — Title required, project optional. **AC:** validation errors are accessible and no accidental save happens on Escape.
- [x] **M2-017 — Add task detail panel/page** [S] [depends: M2-016] — Description, status, priority and dates. **AC:** changes persist and are reflected in the UI.
- [x] **M2-018 — Add task List view** [S] [depends: M2-016] — Sortable columns. **AC:** sort state is deterministic and does not mutate record data.
- [x] **M2-019 — Add task Kanban view** [S] [depends: M2-012,M2-016] — Group by canonical status. **AC:** moving a task updates the same persisted state as List.
- [x] **M2-020 — Add search by title/description** [S] [depends: M2-018] — Case-insensitive local search. **AC:** results can be cleared and empty state is distinct from error.
- [x] **M2-021 — Add filters for project/status/priority** [S] [depends: M2-018] — Combine filters. **AC:** filter combinations return expected fixture data.
- [x] **M2-022 — Add due date and priority indicators** [S] [depends: M2-017] — Include labels, not color alone. **AC:** overdue state uses comparison with local display/UTC storage correctly.
- [x] **M2-023 — Add subtask relationship/UI** [S] [depends: M2-003,M2-017] — One-level nesting first. **AC:** parent/child navigation is stable and cycle-safe.
- [x] **M2-024 — Add milestone model and project view** [S] [depends: M2-002] — Basic due date/status. **AC:** milestone can group/associate tasks without silent status changes.
- [x] **M2-025 — Add activity history to task detail** [S] [depends: M2-006,M2-017] — Status/priority/edit events. **AC:** meaningful edits create events without logging secrets.
- [x] **M2-026 — Add artifact/reference linking contract** [S] [depends: M1-027,M2-003] — URL/provider ID or app-managed file. **AC:** arbitrary paths are not auto-opened/executed.
- [x] **M2-027 — Add keyboard shortcut to create a task** [S] [depends: M2-016] — Make it discoverable. **AC:** shortcut opens form and respects focus/dialog state.
- [x] **M2-028 — Add project/task empty/loading/error states** [S] [depends: M2-014,M2-018] — Include retry where safe. **AC:** screens do not present DB failures as empty content.
- [x] **M2-029 — Add project/task export to a versioned JSON format** [S] [depends: M2-003,M2-006] — Exclude secrets. **AC:** exported file includes schema/app version and passes schema validation.
- [x] **M2-030 — Add project/task import preview** [S] [depends: M2-029] — Do not commit import until user confirms. **AC:** malformed/oversized input is rejected with no partial writes.
- [x] **M2-031 — Add project/task repository integration tests** [S] [depends: M2-007,M2-009] — Use disposable test DB. **AC:** tests are isolated and do not touch user DB.
- [x] **M2-032 — Add end-to-end smoke flow for project-to-task** [S] [depends: M2-014,M2-016,M2-018] — Create project, task, status update, restart/read. **AC:** automated or documented reproducible smoke test passes.

- [x] **M2-033 — Build the Home dashboard from real local records** [S] [depends: M2-014,M2-018,M2-025] — Show recent projects, My Work counts, upcoming/overdue tasks and recent activity. **AC:** counts derive from persisted data; first-run/empty/error states are honest; no synthetic telemetry or fake activity is shown.
- [x] **M2-034 — Implement global search across available local entities** [S] [depends: M1-031,M2-020,M2-014] — Search project names, task titles/descriptions and registered tool names from one entry point. **AC:** results identify their entity type, are keyboard-navigable, respect local scope, and return an explicit empty state; future entities can register search adapters without changing existing searches.
- [ ] **M2-035 — Validate the Projects & Tasks vertical slice with task-based usability checks** [S] [depends: M2-014,M2-016,M2-018,M2-019] — Observe representative users completing create-project, create-task, find-task, change-status and return-to-work flows; record friction and prioritized fixes without expanding scope. **AC:** a short findings note records the test environment, observed blockers and follow-up task IDs; no user data is collected unnecessarily.

## M3 — Cleaner & offline DevTools

### Cleaner safety
- [x] **M3-001 — Define cleanup rule contract** [S] [depends: M1-027,M1-028] — Category, allowed root, eligibility rules. **AC:** rule can be unit tested without deleting files.
- [x] **M3-002 — Implement read-only traversal on a test directory** [S] [depends: M3-001] — Do not call delete. **AC:** fixture files remain byte-identical after scan.
- [x] **M3-003 — Implement protected-path and path-containment checks** [S] [depends: M3-001] — Reject traversal/outside root. **AC:** malicious path fixtures are denied.
- [x] **M3-004 — Add symlink/junction handling policy** [S] [depends: M3-003] — Do not follow links outside allowed root. **AC:** Windows-capable test or documented platform test proves escape is rejected.
- [x] **M3-005 — Define cleanup candidate identity** [S] [depends: M3-002] — ID plus metadata needed for revalidation. **AC:** candidate is not identified only by display path text.
- [x] **M3-006 — Add scan progress and cancellation contract** [S] [depends: M1-026,M3-002] — Honest cancellation semantics. **AC:** UI receives progress and terminal state.
- [x] **M3-007 — Add preview DTO and UI** [S] [depends: M3-005] — Category/count/size/skips. **AC:** preview performs no mutation.
- [x] **M3-008 — Add explicit clean approval flow** [S] [depends: M3-007,M1-028] — Summary before action. **AC:** no execution begins before approval.
- [x] **M3-009 — Revalidate each candidate before deletion** [S] [depends: M3-005,M3-008] — Re-check target, type, identity and policy. **AC:** changed or out-of-policy candidate is skipped.
- [x] **M3-010 — Add bounded deletion executor** [S] [depends: M3-009] — Handle locked/denied files individually. **AC:** one failed file does not crash whole job.
- [x] **M3-011 — Report estimated versus actual freed bytes** [S] [depends: M3-010] — Separate measurements. **AC:** report does not claim all estimated bytes were freed.
- [x] **M3-012 — Add cleaner tests for locked/denied/changed files** [S] [depends: M3-010] — Platform-appropriate tests. **AC:** each case produces a documented skipped/failed status.
- [x] **M3-013 — Implement safe temporary-file rule set** [S] [depends: M3-001..M3-012] — Narrow allowlist only. **AC:** each rule names target and exclusions in docs.
- [x] **M3-014 — Add cleanup report and optional task link** [S] [depends: M2-026,M3-011] — Save summary, not sensitive full path list by default. **AC:** result can be inspected after operation.

### Offline DevTools
- [x] **M3-015 — Add JSON formatter/validator** [S] — Pure local transformation. **AC:** valid, invalid and oversized input are tested.
- [x] **M3-016 — Add YAML formatter/validator only with vetted parser** [S] — Keep parser selection explicit. **AC:** invalid YAML returns structured error.
- [x] **M3-017 — Add Base64 encode/decode** [S] — Handle UTF-8 and binary-vs-text clearly. **AC:** round-trip test passes.
- [x] **M3-018 — Add URL encode/decode** [S] — Distinguish components from full URLs. **AC:** reserved-character fixtures pass.
- [x] **M3-019 — Add UUID generator/validator** [S] — Use vetted random source/library. **AC:** output conforms to declared UUID format.
- [x] **M3-020 — Add SHA-256 file/text hash utility** [S] — Stream large files. **AC:** known test vectors pass and memory use is bounded.
- [x] **M3-021 — Add text diff utility** [S] — Side-by-side or unified result. **AC:** inserts/deletes/empty files are tested.
- [x] **M3-022 — Add timestamp converter** [S] — Unix/ISO conversion. **AC:** invalid values and timezone display are clear.
- [x] **M3-023 — Add DevTools tests and offline acceptance check** [S] — Verify no provider calls. **AC:** tool works with networking disabled.

## M4 — NetOps, SysOps, connections and jobs

- [x] **M4-001 — Define bounded network diagnostic contract** [S] — Target, timeout, limits, result provenance. **AC:** required target validation exists.
- [x] **M4-002 — Implement DNS lookup adapter** [S] [depends: M4-001] — Read-only resolution. **AC:** invalid host and resolver error are distinct.
- [x] **M4-003 — Implement TCP connectivity test** [S] [depends: M4-001] — One host/port only. **AC:** timeout/refused/success have distinct results.
- [x] **M4-004 — Implement ping adapter or capability detection** [S] — Do not assume external executable availability. **AC:** unsupported is explicit and tested.
- [x] **M4-005 — Implement traceroute adapter or deferred status** [S] — Bounded duration/hops. **AC:** no unbounded process or silent fallback.
- [x] **M4-006 — Add subnet calculator** [S] — Offline arithmetic utility. **AC:** IPv4 cases and invalid CIDR are tested.
- [x] **M4-007 — Add system inventory read-only adapter** [S] — OS, CPU/RAM/storage metadata available via vetted API. **AC:** unsupported fields are null/status, not fabricated.
- [x] **M4-008 — Add process inventory view** [S] — Read-only initially. **AC:** no kill action is exposed accidentally.
- [x] **M4-009 — Add Windows service inventory view** [S] — Read-only initially. **AC:** user permissions/errors are visible.
- [x] **M4-010 — Add event log integration seam** [S] — Read-only and bounded. **AC:** unavailable/denied/log-empty states differ.
- [x] **M4-011 — Add allowlisted terminal/SSH/RDP profiles** [S] — Store secret references only. **AC:** secrets never appear in exported profile/log.
- [x] **M4-012 — Add Activity Center for long-running jobs** [S] [depends: M1-026] — Display state, progress, duration and result. **AC:** status transitions are tested. Native job history is persisted and guarded; producers, cancellation and interrupted-job recovery remain follow-up work.
- [x] **M4-013 — Add cancel job action where supported** [S] — Truthful best-effort semantics. **AC:** unsupported cancellation is disabled and explained. No connected runner currently provides a cancellation handle, so the action remains disabled with its reason visible.
- [x] **M4-014 — Add task creation from diagnostic result** [S] [depends: M2-016] — Attach redacted summary. **AC:** target/result/time are linked without secret data. NetOps runs bounded one-target checks; native result handles create a Personal inbox Task with target, result and timestamp only, omitting resolved addresses and raw system messages.
- [x] **M4-015 — Add network timeout/rate-limit/error tests** [S] — Test fixtures/local loopback. **AC:** tests do not target arbitrary public hosts. Coverage includes malformed DNS input, expired local TCP deadlines, loopback success/refusal behavior, and deterministic injected-clock rate-window limits; no tests contact public hosts.

## M5 — World Monitor, trends and media intelligence

- [x] **M5-001 — Define connector/source contract** [S] — Access method, terms, cadence, freshness and health. **AC:** a connector cannot be enabled without declared ID and limits. Typed declarations validate ID, kind/access pairing, reviewed terms, HTTPS references, bounded polling/timeout/size/item limits, freshness and health before activation; no connector is fetched or persisted by this task.
- [x] **M5-002 — Add normalized intelligence item schema** [S] — Source URL and timestamps. **AC:** published/observed/event time are separate fields. A strict typed DTO preserves source attribution and separate timestamps, validates explicit-zone RFC 3339 values and bounds, and never infers missing publication/event times.
- [x] **M5-003 — Implement one user-configured RSS connector** [S] — Start with a test fixture and a permitted source. **AC:** rate/timeout and parsing errors are handled. A single-call native RSS 2.0 adapter uses JPL's officially documented public CNEOS feed as a local fixture; it enforces process-local cadence, timeout/size/item bounds, DNS/IP pinning, no redirects/proxies, and typed parsing/transport failures. It does not schedule, persist, auto-enable or fetch the example source.
- [x] **M5-004 — Add HTTP fetch timeout, size cap and backoff** [S] — No unbounded content load. **AC:** oversized/slow source produces controlled failure. RSS request reads have a total configured deadline and hard byte ceiling; timeout and oversized bodies map to typed failures. Repeated source failures add a manual-retry backoff from 60 seconds to a one-hour cap, bounded by the source cadence; successful fetch clears it and no automatic retry runs.
- [x] **M5-005 — Add canonical URL and content fingerprint helpers** [S] — Preserve original URL. **AC:** test fixtures show stable identity without losing source reference. Items now retain exact `originalUrl` and a validated canonical URL pair; URL normalization removes fragments/allowlisted tracking parameters only. Versioned SHA-256 fingerprints normalize whitespace while preserving field boundaries, case and punctuation; no deduplication occurs in this task.
- [x] **M5-006 — Add dedupe while preserving source relationships** [S] — Do not erase attribution. **AC:** duplicate records retain all source IDs. A 2,000-item-bounded grouping helper joins items with a shared canonical URL or recomputed content fingerprint, retains every original item and reports all unique source IDs; it does not persist, delete or claim independent verification.
- [x] **M5-007 — Add freshness state and last-success display** [S] — Fresh/stale/blocked/error. **AC:** stale feeds are never displayed as live. A serializable view exposes freshness, normalized last-success time and an `isLive` guard that is true only for an enabled fresh source; future/malformed/missing times and failure health states fail closed. RSS calls update the caller's in-memory attempt/success status; M5-008 persists declarations and renders saved freshness, while item freshness remains in M5-012.
- [x] **M5-008 — Add source list and connector settings UI** [S] — Enable/disable and cadence. **AC:** disabled connector performs no scheduled fetch. User-configured RSS declarations persist locally and appear with terms review, enablement, minimum interval, freshness and last-success state. New sources start disabled and cannot be enabled until terms are marked reviewed; URL changes disable and reset prior freshness. Minimum cadence is saved as connector settings only: no scheduler, startup fetch or source-screen fetch was added. Native RSS boundary continues to reject disabled connectors.
- [x] **M5-009 — Add topic watchlists** [S] — Keyword/category filters. **AC:** watchlist can be paused/deleted. Named local filters persist across database reopen, match case-insensitive literal keywords in title/summary and exact category IDs, combine populated groups with AND, support explicit pause/resume and confirmed deletion, and do not start fetching or notifications.
- [x] **M5-010 — Implement baseline/window trend calculation** [S] — Declare window and comparison window. **AC:** unit fixtures prove output and edge cases. A pure bounded calculation compares adjacent equal-length half-open windows using validated observed time, watchlist matching and per-category deduplicated counts; it includes both windows and method version, leaves the ratio undefined for a zero baseline, and emits no fabricated rows for categories absent from both windows.
- [x] **M5-011 — Add trend uncertainty/coverage labels** [S] — Make low source coverage visible. **AC:** missing data cannot be represented as zero with no caveat. Each window now requires explicit source selection/fetch evidence; `null` means no successful fetch, `0` means successful empty evidence, partial/complete labels show coverage and ratios require identical complete source scopes.
- [x] **M5-012 — Add Intelligence list and item detail UI** [S] — Show source, published/observed time and status. **AC:** source link is visible. The bounded local list/detail reads source-attributed items, preserves published/observed/event timestamps and verification/freshness states, and exposes validated HTTPS feed/item links. Users explicitly trigger a policy-bounded source fetch; route loads do not fetch or poll.
- [x] **M5-013 — Add Research Desk note/source linking** [S] [depends: M2-026] — Save item to a collection/note. **AC:** source attribution survives export. The Intelligence detail saves an id-linked local research note; export is versioned JSON containing source ID/URLs, publication/observation timestamps and verification state. Note data does not clone item content, and export uses a user-selected path without overwriting an existing file.
- [x] **M5-014 — Add safe notification rules** [S] — Tray notifications controlled by preferences. **AC:** no fetch/notification before the user enables it. The strict local preference defaults off; OS permission is requested only by the explicit Settings opt-in. The native sender checks the preference and notifies only after a successful manual fetch returns items; enabling it schedules no startup, timer or background fetch.
- [x] **M5-015 — Add one official status/advisory connector** [S] — Use documented access. **AC:** provider-specific errors/rate limits are documented. The Intelligence Sources page can add an official NWS active-alert Atom feed for one U.S. state or DC. It starts disabled/unreviewed, uses a 30-minute minimum interval, and fetches only by manual user action. The bounded parser labels NWS alert items `weather_alert`; NWS 403 and HTTP 429 map to `RateLimited`, with other HTTP errors retained as `HttpStatus` and existing manual backoff. It is informational only, not emergency alerting.
- [x] **M5-016 — Add World Monitor map decision spike** [S] — Evaluate geospatial sources and licensing without copying code. **AC:** ADR records data availability, UX benefit and legal review needs. ADR-0009 defers map rendering pending a concrete journey and legal/provider review; it evaluates official USGS/NWS coordinate data, basemap terms, OSM attribution/cache/no-prefetch constraints, privacy, coverage, UX benefit and alternatives. No code, branding or map assets are adopted.
- [x] **M5-017 — Add prompt-injection fixture for source content** [S] — Ensure source text cannot direct tools/policy. **AC:** fixture is treated as untrusted input. An adversarial Atom item asks for rule override, shell execution and secret export; native parser preserves it only as source-attributed text, while the Intelligence detail renders text without turning its URL into an action/link. Ingestion/display invokes no model/tool or changes to permissions/policy.
- [x] **M5-018 — Add source deletion/export controls** [S] — User can remove stored source content. **AC:** deletion behavior and retained audit metadata are documented. Sources can be explicitly exported to user-selected versioned JSON (source settings plus all retained items, capped by existing 10,000 item retention) or deleted after confirmation. A transaction removes declaration and source items, retains Research Desk attribution snapshots/watchlists, and records only source ID/name/feed URL/time/item count in a last-1,000-entry audit table.

## M6 — Specialist workflows

- [x] **M6-001 — Define external adapter manifest and health state** [S] — No dynamic plugin execution yet. **AC:** adapter metadata/version/availability is testable. Added bounded typed manifest and health DTOs with closed kind/capability/availability enums, strict JSON shape and tests for identity/version/text/capability and timestamp validation. Capability labels grant no authority; no persistence, discovery or dynamic execution is added.
- [x] **M6-002 — Add HTTP request/response inspection tool** [S] — Safe defaults and redaction. **AC:** auth/cookie headers are redacted from saved history. Added a manual HTTPS GET-only Inspector with public-address DNS pinning, no proxy/redirect, 10-second/256-KiB limits and single-flight/5-second cadence. No auth/cookie/custom headers are sent; history is local and bounded to 200, persists only metadata and sanitized headers, redacts credential-like query keys and never saves response bodies.
- [x] **M6-003 — Add OpenAPI document viewer/validator** [S] — Validate local/imported schema. **AC:** oversized/malformed document is safely rejected. Native file selection reads only UTF-8 JSON/YAML up to 1 MiB. Local YAML 1.2 parsing rejects duplicate keys and aliases; malformed/unsupported OpenAPI version, missing info fields, non-object paths and oversized content show bounded failures. The viewer escapes source text and performs no `$ref` or server network access; it is explicitly a structural, not complete conformance, validator.
- [x] **M6-004 — Add API result-to-task artifact link** [S] [depends: M2-026] — Save redacted request/response metadata. **AC:** payload size and secret controls apply. HTTP Inspector can explicitly create a follow-up Task from a persisted result ID; native code re-redacts the URL and saves only bounded result ID, URL, status/failure, timestamp and response byte count. Response body and headers are excluded; missing, malformed or oversized history fails without creating a Task.
- [x] **M6-005 — Define observability provider adapter** [S] — Logs/metrics/traces source metadata. **AC:** provider status and freshness represented. Added strict data-only provider manifest/health DTOs with fixed signal/status/freshness enums, bounded identifiers and threshold, and derived freshness that distinguishes stale/unknown/degraded states; no endpoint, credential, persistence, polling or execution behavior is present.
- [x] **M6-006 — Add local metrics snapshot view** [S] — No remote telemetry by default. **AC:** source and sample time are displayed. Added a manually refreshed Local Metrics page backed by read-only local system inventory for logical processors, physical memory and Windows volume capacity. The snapshot shows its local OS API source, sample time and per-field availability; no polling, persistence or remote telemetry is added.
- [x] **M6-007 — Define security assessment scope model** [S] — Target entries, authorization note, exclusions, expiry. **AC:** assessment cannot start with empty scope. Added a strict scope declaration and start validator for exact host/IP targets, exclusions, required authorization note and expiry; empty, malformed, duplicate, expired and overlong scopes fail closed. No assessment or network request is executed.
- [x] **M6-008 — Add out-of-scope validation tests** [S] — Host/IP matching and redirects. **AC:** out-of-scope targets are rejected before requests. Added a pre-request scope gate that matches exact normalized host/IP targets, prioritizes exclusions, rejects unsupported schemes, checks expiry for every request, and resolves/rechecks redirects. Tests cover in-scope/out-of-scope host and IP, external redirects, exclusions and expiry; no network requests are issued.
- [x] **M6-009 — Add security finding model and severity/likelihood fields** [S] — Separate severity from risk/priority. **AC:** fields are not conflated in UI. Added a bounded strict finding DTO with independent closed severity and likelihood enums, lifecycle state, target and timestamp validation. A reusable UI fragment renders Severity and Likelihood in separate labelled rows; no combined risk score or Task-priority substitution is introduced. Persistence and findings screen remain future work.
- [x] **M6-010 — Convert finding into task with evidence** [S] [depends: M2-026] — Include remediation and provenance. **AC:** task preserves scope, timestamp and evidence link. Added a strict native handoff that requires a valid finding and unexpired scope containing its target, validates remediation and evidence URL, redacts sensitive evidence query values, and creates a Personal inbox Task with finding dimensions, scope/authorization snapshot, observation time, remediation and evidence link. Unsafe/out-of-scope inputs fail without creating a Task.
- [x] **M6-011 — Add evidence manifest/hash workflow** [S] — Integrity metadata, no claim of forensic validity without procedure. **AC:** hash known vectors pass. Added a native Windows workflow for up to 50 explicitly selected files and 2 GiB aggregate. It creates a versioned self-hashing manifest with leaf filenames, sizes, SHA-256 digests and explicit integrity-only limits; save accepts only a native-cached handle and refuses overwrite. No file paths/content leave native code, no bytes are uploaded, and the manifest does not claim chain-of-custody or forensic validity.
- [x] **M6-012 — Add source/license review checklist for integrations** [S] — Review code and data terms. **AC:** adapter PR cannot be considered done without review result. Added a reusable provenance, code/data terms, permitted access, attribution, privacy/retention and redistribution checklist. The PR template requires a reviewer/date, evidence and exactly one outcome; blank or blocked outcomes leave an adapter incomplete/unavailable, while not-applicable requires rationale. Re-review is required when material source/terms/use changes.
- [x] **M6-013 — Add credential reference interface** [S] — OS secure store abstraction, no actual vault UI required. **AC:** unit tests mock provider and verify secret values are redacted. Added a provider-neutral native trait for store/retrieve/delete using opaque UUID references and closed errors, with a safe operation summary that cannot contain secrets. Mock tests verify round-trip/delete, input bounds, reference shape and redaction. No OS backend, IPC, persistence, renderer access or vault UI is implemented.
- [x] **M6-014 — Add backup/export integrity manifest** [S] — Version/size/hash. **AC:** corrupted fixture fails verification. Added a strict v1 manifest binding artifact format/version, byte size and SHA-256 with a 2 GiB bound. Verification rejects malformed/unsupported manifests, changed payloads and truncated payloads. This integrity contract is not yet wired to export UI or file I/O and does not authenticate or encrypt an artifact.

## M7 — Automation, AI and MCP

- [x] **M7-001 — Define versioned workflow schema** [S] — Steps reference tool ID/version and typed inputs. **AC:** malformed/unknown tool schema is rejected. Added strict workflow schema v1 with bounded IDs, non-empty steps, explicit three-part tool versions and closed typed JSON inputs. Parsing rejects malformed/unknown fields and types, unsupported schema versions, invalid IDs/versions and oversized/deep values. Registry matching, DAG/step bounds and execution remain later tasks.
- [x] **M7-002 — Add workflow validator** [S] — Validate DAG/sequence and bounded step count. **AC:** cyclic/oversized workflow is rejected. Added explicit step dependencies and a validator that rejects dependency cycles, unknown/duplicate dependencies, duplicate step IDs and more than 128 steps. Tests cover valid DAGs and each rejection. It does not execute tools or grant permissions.
- [x] **M7-003 — Add deterministic sequential workflow runner** [S] — No AI dependency. **AC:** fixture workflow produces expected step order/results. Added an in-memory runner over registered exact tool ID/version handlers. It executes a stable topological order, captures typed outputs and stops after the first failure/unavailable handler, marking later steps skipped. Fixture tests verify order/results and mismatched-version failure. No production handler, IPC, persistence, timeout or approval flow is added.
- [x] **M7-004 — Add per-step timeout and error policy** [S] — Stop/continue choices explicit. **AC:** failed step follows declared policy. Workflow schema v2 requires each step to specify a bounded 100–300,000 ms timeout and explicit stop/continue policy; v1 remains parseable but is not executable without the policy. The runner passes a deadline, reports late returns as timed out, stops on stop, and under continue runs independent steps while skipping dependents. Tests cover bounds, timeout, stop and continue. Timeouts are cooperative because synchronous handlers cannot be safely preempted; no production handler or IPC is registered.
- [x] **M7-005 — Add idempotency/retry declaration per tool** [S] — No blanket retries. **AC:** destructive tools default to no automatic retry. Tool definitions now default to unknown idempotency and `never` retry. Explicit 1–5 transient retries are accepted only for declared-idempotent tools and rejected for destructive, privileged or remote risk classes. Registration validates the policy; no retry loop or blanket retry behavior is added.
- [x] **M7-006 — Add workflow run history** [S] — Step outputs are redacted. **AC:** run shows status/duration/correlation ID. Added transactional local SQLite history for run and step status/durations, timestamps and correlation IDs, with output values omitted and a fixed `[REDACTED]` marker for successful steps. Native read-only list/detail commands are bounded (500 retained runs, 200 summaries per list); tests verify status/duration/correlation and synthetic-secret redaction. This is a native data/API contract only: no history UI, production workflow handlers or workflow execution IPC is connected.
- [ ] **M7-007 — Add scheduled workflow prototype behind opt-in** [S] — Explicit cadence and pause. **AC:** disabling schedule prevents future runs.
- [ ] **M7-008 — Define AI provider interface and config schema** [S] — Keep providers behind interface. **AC:** fake test provider exercises core without network.
- [ ] **M7-009 — Add AI structured-output parser/validator** [S] — Strict schema and size limit. **AC:** malformed response cannot call a tool.
- [ ] **M7-010 — Connect AI to registry discovery only** [S] — Read-only first. **AC:** AI can list tools but cannot invent tool IDs.
- [ ] **M7-011 — Add AI-generated action preview and approval binding** [S] — Bind args/tool/target/version. **AC:** changed args invalidate approval.
- [ ] **M7-012 — Add source attribution to AI summaries** [S] — Attach evidence IDs/URLs. **AC:** unsupported claims are labelled unknown, not asserted as facts.
- [ ] **M7-013 — Add prompt-injection and secret-exfiltration tests** [S] — Use synthetic secrets/fixtures. **AC:** retrieved content cannot grant new permission or expose test secret.
- [ ] **M7-014 — Add MCP adapter spike behind feature flag** [S] — Pin tested specification/SDK and scope tools. **AC:** conformance/contract tests run; disabling MCP does not break app.
- [ ] **M7-015 — Add user-visible AI disable/provider failure path** [S] — No silent fallback sending data to another provider. **AC:** provider failures preserve non-AI operations.

## M8 — Stable release, maintainability and UX quality

- [ ] **M8-001 — Define application versioning/release notes format** [S] — Version, changes, known issues, migrations. **AC:** sample release notes pass documentation review.
- [ ] **M8-002 — Select and document root license** [S] — Owner/legal decision required. **AC:** root LICENSE and third-party notice policy agree.
- [ ] **M8-003 — Add dependency/license inventory job** [S] — Review false positives. **AC:** CI produces an artifact/report with dependency versions.
- [ ] **M8-004 — Add SBOM generation where supported** [S] — Identify format/version. **AC:** release pipeline stores SBOM artifact.
- [ ] **M8-005 — Configure signed updater** [S] — Protect private key outside repository. **AC:** bad signature is rejected in test/release procedure.
- [ ] **M8-006 — Add migration upgrade test from previous schema** [S] — Use fixture DB copy. **AC:** data survives and schema version advances.
- [ ] **M8-007 — Add backup/restore integration test** [S] — Use disposable data. **AC:** record counts/hashes match expected output.
- [ ] **M8-008 — Add keyboard/accessibility smoke checklist** [S] — Focus, dialog, status announcements. **AC:** documented critical flows pass.
- [ ] **M8-009 — Measure cold launch and idle/tray resource use** [S] — Record baseline, not a fabricated threshold. **AC:** measurements and environment are documented.
- [ ] **M8-010 — Add app crash/error report redaction check** [S] — No secret or personal path leakage. **AC:** synthetic secret remains absent from artifacts.
- [ ] **M8-011 — Test clean install and uninstall** [S] — No user-data deletion unless user explicitly chooses it. **AC:** documented Windows test passes.
- [ ] **M8-012 — Test upgrade and rollback/recovery messaging** [S] — Do not promise rollback that isn't supported. **AC:** failure state and recovery instructions are exercised.
- [ ] **M8-013 — Add release checklist and artifact checksum** [S] — Ensure build provenance. **AC:** every stable release uses checklist.
- [ ] **M8-014 — Run security review against Security Model** [S] — Trace each requirement to evidence. **AC:** findings are fixed or explicitly accepted with rationale.
- [ ] **M8-015 — Audit README claims against shipped features** [S] — Remove incorrect badges/claims. **AC:** every “available” claim is backed by tested behavior.
- [ ] **M8-016 — Run Windows support matrix smoke tests** [S] — Report actual tested OS/build. **AC:** release notes accurately disclose coverage.
- [ ] **M8-017 — Finalize user guide and troubleshooting FAQ** [S] — Install, data location, tray, backup, permissions and diagnostics. **AC:** a new user can complete setup without repository archaeology.

## M9 — Recovery & Data Protection (post-V1.0)

- [ ] **M9-001 — Define recoverable data classes and backup scope** [S] [depends: M1-023,M2-001] — Document which app-owned records/artifacts may be backed up and which data must be excluded. **AC:** scope, exclusions, sensitive-data rules and restore assumptions are documented; secrets are excluded by default.
- [ ] **M9-002 — Define a versioned backup manifest and archive contract** [S] [depends: M9-001] — Include format/schema version, app version, created time, record types, hashes and counts. **AC:** manifest validates against a schema and rejects unknown unsupported versions.
- [ ] **M9-003 — Implement manual backup export for selected Moonships data** [S] [depends: M9-002] — Export supported projects/tasks/settings/notes and explicitly selected app-owned artifacts. **AC:** backup is created in a user-selected location, credentials are not exported, and the output is not silently overwritten.
- [ ] **M9-004 — Implement backup integrity verification** [S] [depends: M9-003] — Verify archive manifest, per-entry hashes and size limits before restore. **AC:** altered/truncated archives fail verification with a clear error and no data mutation.
- [ ] **M9-005 — Add restore preview and conflict policy** [S] [depends: M9-002,M9-004] — Show data types/counts and conflict options before restoring. **AC:** restore does not begin before confirmation; duplicate IDs and newer local records have an explicit policy.
- [ ] **M9-006 — Implement staged restore with recovery on failure** [S] [depends: M9-004,M9-005] — Validate into staging and apply through a transaction or safe database swap/backup strategy. **AC:** injected failure leaves current data intact or reports a tested recovery path.
- [ ] **M9-007 — Add opt-in scheduled backup and pause control** [S] [depends: M9-006] — Add configurable cadence only after manual backup/restore is reliable. **AC:** scheduling is off by default, visible in Activity Center, pausable and never exports secrets.
- [ ] **M9-008 — Add backup/restore round-trip and adversarial archive tests** [S] [depends: M9-003,M9-004,M9-006] — Cover clean restore, corrupted archive, path traversal, oversized payload and conflict cases. **AC:** test results are reproducible and documented.

## M10 — Knowledge & Operations Desk (post-V1.0)

- [ ] **M10-001 — Define Notes, Collections and Runbook data models** [S] [depends: M1-023,M2-001] — Add versioned schemas/migrations for local knowledge records. **AC:** relationships, timestamps, archive behavior and data-retention rules are documented and tested.
- [ ] **M10-002 — Implement local Notes CRUD with safe Markdown preview** [S] [depends: M10-001] — Create/edit/search/archive notes without external service dependency. **AC:** scripts/unsafe HTML are not executed, preview handles malformed content safely, and notes persist across restart.
- [ ] **M10-003 — Link notes to projects, tasks, tool runs and source records** [S] [depends: M10-002,M2-026] — Use typed references rather than copied opaque payloads. **AC:** references can be opened from both sides and do not duplicate secrets or large result bodies.
- [ ] **M10-004 — Add versioned Runbook templates and step checklists** [S] [depends: M10-001] — Store repeatable procedures and operator notes. **AC:** runbooks have stable IDs/versions and never execute embedded commands automatically.
- [ ] **M10-005 — Define Incident model and lifecycle** [S] [depends: M2-006,M4-012] — Track incident status, severity, timestamps, affected services and linked tasks. **AC:** incident severity is distinct from task priority and action risk.
- [ ] **M10-006 — Add incident timeline and shift handoff view** [S] [depends: M10-005] — Combine selected events and operator notes into a reviewable timeline. **AC:** events retain source/time, sensitive content is redacted, and handoff can be exported.
- [ ] **M10-007 — Generate provenance-aware technical reports** [S] [depends: M10-003,M10-006] — Export project, incident, tool and research outcomes with references. **AC:** report declares its generation time, sources, omissions and redaction behavior.
- [ ] **M10-008 — Add a Knowledge search adapter to global search** [S] [depends: M2-034,M10-002,M10-004] — Make notes and runbooks searchable when implemented. **AC:** results identify type and context, support opening the record and respect archived/deleted state.

## M11 — Advanced Security & Research Integrations (post-V1.0)

- [ ] **M11-001 — Build Security Assessment Runs list and detail UI** [S] [depends: M6-007,M6-009,M4-012] — Show scope, run status, selected checks, evidence and resulting findings. **AC:** a run cannot start without scope, and run history is read-only after completion except for annotations.
- [ ] **M11-002 — Implement a permitted public OSINT source adapter** [S] [depends: M5-001,M6-012] — Start with one approved public source/API and source attribution. **AC:** access terms, rate limits, timestamps and connector health are displayed; no authentication/access controls are bypassed.
- [ ] **M11-003 — Add IOC parsing and optional threat-intelligence enrichment** [S] [depends: M11-002] — Parse IP/domain/URL/hash indicators and enrich only through configured providers. **AC:** each enriched field identifies its source/time and provider failure is not represented as “no threat”.
- [ ] **M11-004 — Add safe local binary metadata inspection** [S] [depends: M3-020] — Read file type/magic bytes, size, hashes and selected strings using vetted parsers. **AC:** input is treated as data, file size is bounded, and no sample is executed.
- [ ] **M11-005 — Add binary/firmware comparison report** [S] [depends: M11-004] — Compare hashes, metadata and selected byte/string differences. **AC:** report identifies tool/version and inputs; it does not claim full reverse-engineering coverage.
- [ ] **M11-006 — Define isolated Security Lab adapter** [S] [depends: M6-001,M6-007,M6-008] — Specify integration with a local VM/container lab. **AC:** lab identity, network isolation, lifecycle and cleanup responsibilities are documented before execution support is implemented.
- [ ] **M11-007 — Implement allowlisted out-of-process specialist-tool adapter** [S] [depends: M11-006] — Run only configured tools with explicit argument arrays and bounded output/time. **AC:** no shell-string concatenation, arbitrary executable path or silent privilege escalation is allowed.
- [ ] **M11-008 — Add security assessment negative and scope regression matrix** [S] [depends: M11-001,M11-007] — Cover redirects, hostname changes, IPv4/IPv6, exclusions, timeouts and abort behavior. **AC:** out-of-scope actions are rejected before request/command execution.
- [ ] **M11-009 — Add authorized CTF/training lab integration profile** [S] [depends: M11-006] — Save local lab target, reset notes and exercise steps. **AC:** profiles are clearly labelled as training, never expand beyond declared lab range, and do not include real third-party targets by default.

## M12 — Advanced World Monitor & Media Intelligence (post-V1.0)

- [ ] **M12-001 — Review provider terms and define advanced media connector policy** [S] [depends: M5-001,M5-002] — Compare official API, RSS, user-provided source and permitted browser automation paths. **AC:** each source class has allowed access, rate, retention, attribution and failure rules before an adapter is selected.
- [ ] **M12-002 — Add a permitted video/channel metadata connector** [S] [depends: M12-001] — Support one approved provider/interface for public metadata and update discovery. **AC:** connector documents API/quotas, records canonical source and timestamps, and does not bypass login or platform restrictions.
- [ ] **M12-003 — Add transcript import for user-provided or permitted sources** [S] [depends: M12-001] — Normalize transcript text and segments with timestamps where provided. **AC:** every transcript retains source, acquisition method, language when known and provenance; unsupported sources fail clearly.
- [ ] **M12-004 — Add compliant page-change monitoring adapter** [S] [depends: M12-001] — Track changes on configured pages using permitted access and bounded fetches. **AC:** respect robots/terms where applicable, rate limits, conditional requests, size caps and opt-out; store no session cookies by default.
- [ ] **M12-005 — Isolate optional browser automation connector** [S] [depends: M6-001,M12-001] — Define a separate adapter/process for browser-driven source workflows. **AC:** browser automation has its own permissions, profile isolation and timeouts and cannot bypass access controls or restrictions.
- [ ] **M12-006 — Add geospatial event normalization with precision metadata** [S] [depends: M5-002,M5-016] — Normalize coordinates, place names and uncertainty from permitted sources. **AC:** provenance and location precision are visible; uncertain locations are not presented as exact.
- [ ] **M12-007 — Add cross-source entity/event correlation** [S] [depends: M5-006,M5-010] — Group related items without discarding original sources. **AC:** output explains matching evidence, time window and confidence; it does not infer causation from correlation alone.
- [ ] **M12-008 — Add user-configured digest and alert schedule** [S] [depends: M5-014,M5-008] — Produce daily/weekly digests from enabled watchlists. **AC:** schedule is opt-in, pauseable, source links are retained, and no notification claims live data from a stale connector.
- [ ] **M12-009 — Add advanced connector freshness/coverage dashboard** [S] [depends: M5-007,M12-002,M12-003] — Summarize last success, coverage, lag, quota/errors and source availability. **AC:** missing data is distinguishable from zero activity and failed sources are visible.

## M13 — CLI, Vault, CostLens, Extensibility & Collaboration (post-V1.0)

- [ ] **M13-001 — Define a CLI interface over the Tool Registry** [S] [depends: M1-027,M7-001] — Specify command IDs, argument schemas and JSON/human output. **AC:** CLI calls registered tools rather than implementing duplicate business logic.
- [ ] **M13-002 — Implement read-only CLI invocation for a small tool set** [S] [depends: M13-001] — Start with safe DevTools and diagnostics. **AC:** structured output is versioned, errors use stable codes, and unsupported tools fail explicitly.
- [ ] **M13-003 — Enforce CLI policy parity with GUI** [S] [depends: M13-002,M1-028] — Apply the same permissions, target scope, limits and approval decisions. **AC:** CLI cannot bypass GUI policy or run destructive commands without required preview/approval.
- [ ] **M13-004 — Implement Windows Credential Manager adapter for secret references** [S] [depends: M6-013] — Store/retrieve credentials through a reviewed OS-secure abstraction. **AC:** secrets never enter settings JSON, logs, issue bodies, exports or test snapshots.
- [ ] **M13-005 — Define CostLens provider and cost-normalization contract** [S] [depends: M1-027] — Record provider, account/profile reference, currency, resource, period and data freshness. **AC:** currency/time window are mandatory for comparisons and missing-cost data is explicit.
- [ ] **M13-006 — Add one user-configured CostLens provider adapter** [S] [depends: M13-005] — Implement the first approved provider only after credential and API-terms review. **AC:** provider access is opt-in, read-only, rate-limited and costs are attributable to a time window.
- [ ] **M13-007 — Decide third-party extension isolation model** [S] [depends: M1-027,M1-028] — Compare compiled-in, out-of-process and sandboxed/WASM options. **AC:** ADR documents threat model, supported capabilities, limits, revocation and why in-process native plugins are not default.
- [ ] **M13-008 — Define signed extension manifest and compatibility checks** [S] [depends: M13-007] — Specify identity, version, API contract, signer, permissions and supported platform. **AC:** malformed, unsigned, incompatible or over-privileged manifests are rejected in fixtures.
- [ ] **M13-009 — Prototype isolated extension execution** [S] [depends: M13-007,M13-008] — Run a harmless test extension in the selected boundary. **AC:** extension cannot access undeclared files/network/tools and can be stopped/revoked.
- [ ] **M13-010 — Define GitHub project/issue integration contract** [S] [depends: M2-026,M13-001] — Link local tasks to external issue/PR identities without duplicating secrets or source-of-truth assumptions. **AC:** direction of sync, permissions, mapping and conflict policy are documented before mutating integrations.
- [ ] **M13-011 — Write team-sync architecture spike** [S] [depends: M2-001,M2-003] — Evaluate identity, authorization, offline edits, conflict resolution and deletion propagation. **AC:** ADR defines data ownership and a tested conflict strategy before a sync backend is built.
- [ ] **M13-012 — Write remote-agent threat model and protocol spike** [S] [depends: M1-028,M7-003] — Evaluate authentication, target binding, command allowlists, replay protection and revocation. **AC:** no remote execution implementation is enabled by this task; a separate security review is required before implementation.
- [ ] **M13-013 — Assess cross-platform portability after Windows release** [S] [depends: M8-016] — Inventory Windows-specific APIs and define macOS/Linux support gaps. **AC:** report names per-module feasibility, blockers, test matrix and estimated maintenance cost; no platform-support claim changes automatically.

## Task handoff template

When a task is picked, the agent should write a short work note (PR body or task comment):

- **Task ID:**
- **Scope:** one sentence
- **Acceptance criteria:** copied without broadening
- **Files likely affected:**
- **Tests to run:**
- **Out of scope:**
- **Result:** implemented / blocked / needs design decision
- **Evidence:** commands and pass/fail results
- **Next unblocked task:**

## Backlog governance

- GitHub Issues are generated from unchecked task lines; see [TASK_ISSUE_SYNC.md](TASK_ISSUE_SYNC.md). Keep each task ID stable and unique.
- Do not mark a task done because code compiles if acceptance criteria include behavior or tests.
- If blocked, record the blocker and leave unchecked.
- If one task requires unrelated edits in more than a few modules, split it and record dependency IDs.
- Add future features only after defining user/job, permission, data-flow, test plan and smallest task.
- Keep this backlog free of dates until effort/capacity is known; milestone gates provide order and exit criteria.
