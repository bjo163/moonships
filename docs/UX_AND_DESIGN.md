# UX, Navigation & Design Specification

**Status:** Canonical for information architecture, user journeys and low-fidelity layouts; visual tokens/components are governed by [Design System](DESIGN_SYSTEM.md), and sound behavior by [Audio & SFX Specification](AUDIO_SFX_SPEC.md).  
**Target:** Windows desktop with main window, System Tray and keyboard-first controls.

## 1. Experience principles

- Desktop-native behavior with a web-technology UI; Moonships is not a website that must remain open in a browser.
- Fast first paint and low idle CPU/memory. Background behavior must be explicit.
- Keyboard-first, but every action remains discoverable through normal UI controls.
- One global search/command entry, one activity center and consistent task/result linking.
- Default home is useful, not a wall of charts.
- Do not show fake data as live data. Demo/sample content must be visibly labelled.
- Stable layouts, accessible contrast, visible focus and Windows-friendly keyboard behavior.
- Users control startup-at-login, tray behavior, notifications, background schedules and network-enabled features.

## 2. Primary information architecture / menu

### Global application chrome
- App icon + **Moonships Workbench**
- Workspace/profile selector
- Global search / Command Palette (Ctrl+K)
- Activity Center indicator
- Notification center
- Help / keyboard shortcuts
- User/data profile menu (no cloud identity is assumed in MVP)

### Main sidebar (recommended order)
1. **Home**
2. **My Work**
   - Inbox
   - My Tasks
   - Today / Upcoming
   - Waiting for Approval
   - Recent Activity
3. **Projects**
   - All Projects
   - Initiatives / Goals
   - Milestones
   - Roadmap
   - Archived
4. **Tasks**
   - List
   - Board
   - Calendar
   - Saved Views
5. **Toolbox**
   - DevTools
   - API & DataLab (later)
   - Clean & Storage
   - System Info
   - NetOps
   - SysOps
   - Connections
6. **Security**
   - Security Overview
   - Scope Registry
   - Assessment Runs
   - Security Lab
   - Blue Team / Logs
   - Threat Intelligence
   - Evidence & Reports
7. **Intelligence**
   - World Monitor
   - Signals & Trends
   - Media Monitor
   - Watchlists
   - Research Desk
8. **Operations**
   - Observe
   - Activity Center
   - Automation
   - Schedules
   - Recovery
   - CostLens (later)
9. **Knowledge**
   - Notes
   - Runbooks
   - Reports
   - Saved References
10. **Integrations** (show only when useful)
11. **Settings**
   - General
   - Appearance
   - Startup & Tray
   - Notifications
   - Data & Privacy
   - Modules
   - Permissions & Security
   - AI Providers
   - Integrations
   - Updates
   - Diagnostics / Logs
   - About

**Navigation rule:** the IA is a target map, not permission to ship empty pages. Hide unavailable modules or mark them clearly as “Planned”; never present placeholder screens as working tools.

M3-015 through M3-023 expose local JSON, YAML, Base64, URL, UUID, SHA-256, text-diff and timestamp tools under one DevTools navigation item. Each tool has labelled inputs, accessible result/error feedback and bounded output. YAML parser errors include stable parser codes and source positions. Base64 offers UTF-8 text, hexadecimal byte and decode modes; non-UTF-8 decoded data is clearly identified as binary. URL tools distinguish component encoding from full URL handling, which preserves reserved separators and validates absolute URLs. UUID generation identifies the v4 secure random source and visibly fails if unavailable; validation states the canonical RFC format it accepts. SHA-256 text/file limits and native Windows-only file selection are stated alongside the tool. File picker cancellation and file errors are visible. Text diff labels original and updated content, shows a unified read-only result, and reports exceeded limits. Timestamp input clearly distinguishes Unix seconds/milliseconds from ISO 8601 with timezone; output labels UTC and device-local timezone separately and reports invalid values clearly. Offline acceptance exercises each implemented tool with browser network fetch disabled; file content remains on-device. Other listed DevTools remain planned until implemented.

## 3. Home dashboard

### Layout
- Top row: welcome/context, workspace selector, global search.
- Primary action strip: New Task, New Project, Quick Tool, Start Scan (only when scan exists), Add Watchlist (only when intelligence exists).
- “Continue working”: recently opened project/task/tool.
- “My Work”: overdue, due soon, blocked, waiting for approval.
- “Activity”: last task completions, scans, workflows and integration updates.
- Optional health widgets: available only when real data exists and can show freshness/source.
- No default “AI insight” card if no provider is configured.

### Empty states
- New user: explain what can be created; offer Create Project / Try DevTools when implemented.
- No tasks: explain how tasks connect to projects/tools.
- No events: state that no activity is recorded, not “all systems operational”.
- No monitor sources: ask user to choose sources, show update cadence and permissions.
- Offline: local tools remain available; network-only functions display offline state.

M2-033 replaces the Home foundation placeholder with saved Personal workspace data. It shows counts for open, overdue, due-in-seven-days and blocked Tasks; the five most recently updated active Projects; up to five overdue and five upcoming active Tasks; and up to ten most recent normal-sensitivity Activity events. Completed/cancelled and archived Tasks do not affect open/due counts. Empty, loading and retryable read-error states are distinct; a genuinely empty first run offers Project and Task creation. Activity summaries and user-authored titles render as text and link to their saved records. Approval requests remain unavailable and are labelled as such; no synthetic operations or health metrics appear.

M4-012/013 Activity Center shows persisted job state, reported progress, recorded duration and result. Cancel remains disabled with an explanation while no connected runner advertises safe cancellation; the UI does not claim a request stopped work.

M4-014 adds a NetOps page for one bounded DNS or TCP diagnostic. It displays the exact target, fixed two-second timeout, operation, result and observation time. A user can create a Personal inbox Task from the result; the saved summary retains target/result/time while omitting resolved addresses and raw system messages. Validation, rate-limit and unavailable states remain visible.

M6-002 adds **HTTP Inspector** under Toolbox. The user submits one public HTTPS URL and explicitly starts a GET; the page shows status, bounded response headers/body and observation time. It clearly states the 10-second timeout, 256 KiB cap, redirect/private-target rejection, absence of auth/cookie inputs, and that response body remains in memory. The local history shows only status, size, redacted URL and redacted headers, with distinct loading, empty, unavailable and request failure states.

M6-004 adds an explicit **Create follow-up Task** action to a successful Inspector result. The user sees the saved result ID and a notice that only redacted metadata is attached. Success identifies the created Task; failure is shown inline. The Task contains result ID, redacted URL, status, observation time and response size only; body and headers are excluded.

M6-003 adds **OpenAPI Viewer** under Toolbox. A user selects a local JSON/YAML file through the native picker. Loading, cancellation, parse/validation failure and a valid summary are distinct. The screen displays spec version, title, API version and path count, then offers the source text in a collapsed, escaped-text viewer. It states the 1 MiB limit and that validation is structural/local, does not resolve references and never contacts API servers.

## 4. Project & task UX

### Projects list — M2-014 initial slice
The Projects navigation opens records from the local Personal workspace. The list shows active projects only, sorted case-insensitively by name with stable ID tie-breaking, and includes status, goal, description and local update time. Create and edit forms support name, goal, description and status on edit; successful saves are confirmed by re-reading SQLite. Loading, empty, error and retry states are distinct. Archived-project browsing and archive controls are not part of this slice.

### Project detail — M2-015 initial Overview
Opening a Project from the Projects list shows its saved name, status, goal and description, the count of non-archived linked Tasks, a New Task action, and up to ten recent safe activity summaries. Zero linked Tasks and no activity each have explicit empty copy; loading, retryable errors and an unavailable/archived Project have distinct states. New Task opens the shared creation dialog with this Project selected. M2-024 adds a Milestones section: create a milestone with a title and optional due date, review its saved status and associated active Project tasks, associate an unlinked Project task, and update planned/active/completed status. Milestone and task read failures are shown separately with refresh feedback; success reflects saved records. Association and milestone status changes do not change Task status. Other unimplemented detail tabs remain future work.

Future detail tabs may include Overview, Tasks, Roadmap, Notes/Docs, Activity and Settings. Add them only when their data and interactions exist.

### Task detail
The M2-017 initial detail route opens from a Task title in the active Tasks list. It loads the persisted title, description, status, priority, due date/time and Project/inbox association. Title, description, status, priority and optional due date/time can be edited; saving uses the scoped native Task update service and confirms only the returned saved record. The datetime-local control displays local time and sends an RFC 3339 instant; the service stores UTC. Missing/archived Task, loading, recoverable save failure and saved states are explicit. If Project names cannot load, Task details remain usable and the notice distinguishes that failure from an archived/unavailable Project. Do not require advanced fields to create a basic task.

M2-025 adds a persisted Activity history section to Task detail. It shows the newest 50 trusted event summaries with timestamps, distinguishes an empty history from an unavailable read, and offers a history-only retry that preserves unsaved Task form edits. Status and priority changes have their own summaries; other meaningful edits use a generic Task-updated summary. Activity never displays user-authored Task content or credentials.

M2-026 defines linked references for future Project/Task detail: an external HTTP(S) URL, a namespaced provider ID, or an app-managed artifact. Saving a reference does not open or fetch it. Opening is a separate user action and only becomes available through a reviewed native adapter; arbitrary paths and executable content are never launched from a reference.

M2-023 adds one-level subtasks to Task detail. A child links to its active parent, and a root lists active children as stable Task-detail routes. A root can create a child that inherits its Project/inbox association; a child cannot create another level. Relationship reads use the scoped active Task list and fail separately from the Task detail read, leaving the detail form usable with a retry for related tasks. Native validation enforces same-workspace, same-project and top-level-parent rules; a parent with active children cannot be archived until those children are archived, and a child cannot be restored before its parent. Future detail fields may include assignee, labels, estimate, dependencies, comments, linked artifacts/references and activity. Duplicate, archive, link-artifact, dependency and export actions remain scheduled separately.

### Shared Project & Task data states — M2-028
Projects, Project Overview, Task List and Task detail show loading while reads are pending. A successful empty result has purpose-specific empty copy; a failed database read has an error state instead of the empty state. Read-only lists/overviews offer retry. Task detail keeps loaded form values available after a save failure for another attempt. Failure to load optional Project names or the Project picker is shown separately from a Task/database failure, so valid Task data remains usable.

### Default status set
- Backlog
- Todo
- In Progress
- Blocked
- In Review
- Done
- Cancelled

Use a single canonical state model. Views may group or filter states but must not invent incompatible states.

### Default priority set
- P0 Critical
- P1 High
- P2 Normal
- P3 Low

Priority describes importance; it is not the same as security severity or execution risk.

### Create-task form (minimum)
1. Title (required)
2. Project (optional on personal inbox)
3. Status (defaults to Todo)
4. Priority (defaults to P2 Normal)
5. Optional: description, due date, labels, parent task, dependencies
6. Save; keyboard escape cancels without persisting accidental partial data.

### M2-016 persisted Task creation slice
The global New task button, `Ctrl+Alt+N` / `⌘+Option+N` shortcut, and registered New Task palette command open the same accessible dialog. The button displays the shortcut. The direct shortcut is ignored while an editable control has focus or any dialog/overlay is open, so it does not interrupt typing or stack dialogs. It requires a nonblank title, offers active Personal-workspace Projects or the Personal inbox, and defaults status/priority to Todo/P2 Normal. The form saves through typed native IPC; success is shown only after the local Task service returns the created record. Failed Project loading is distinct from an empty project list and leaves inbox creation available with a retry for the optional list. Validation is attached to the title field and announced; Escape and Cancel close without invoking the create command. The form prevents closing during an in-flight save to avoid ambiguity about an operation that may already have persisted.

M2-029 provides an Export Projects & Tasks action on both list screens. It exports saved Personal workspace records, including archived Projects and Tasks, and normal-sensitivity Activity history to a versioned JSON file through a native Save dialog. The UI explains that authored descriptions are included and reminds users to review content before sharing; app settings, stored credentials and restricted Activity are omitted. Choosing Cancel saves nothing; success appears only after the native save completes, while failures have a separate retryable message.

M2-030 adds Preview import beside export on both list screens. A native Open dialog selects a versioned Moonships JSON export. The app rejects malformed, unsupported, invalid or over-50-MiB files and limits documents to 100,000 records. A valid preview shows counts, version metadata and up to five Project names and Task titles; content is displayed as plain text. Cancel and failure are reported distinctly. Preview never writes files or changes the database, and the action explicitly says no records were imported.

### M2-018 persisted Task List
The Tasks navigation and registered Open Tasks command lead to the Personal workspace's persisted active tasks. The table shows title, Project or Personal inbox, canonical status, priority, due date and last update. Column controls expose the active sort direction to assistive technology; sorting uses a copied array and stable ID tie-breaker, with no storage mutation. Initial order is title ascending. Loading, empty and database error states remain distinct with a retry; if optional Project names cannot load, the task records remain visible with that limitation explained. Archived-task browsing and task editing are later slices.

### M2-019 Task Board
The Tasks page switches between List and Board using one shared set of persisted Task records. Board columns follow all seven canonical statuses. A card can move by drag-and-drop or by its labelled status select, which provides keyboard operation. Both paths update through the existing native Task update command and display the returned saved record; a failed update leaves the Task in its prior status and reports the failure. Board and List must not keep separate status state.

### M2-020 Task search
The Tasks toolbar searches active Task titles and descriptions locally, without case sensitivity. Search is debounced, can be cleared, and filters both List and Board without changing persisted records. A no-match result is distinct from the initial no-active-tasks state and from a failed task read; the result count reports matches against the active-task total.

### M2-021 combined Task filters
Project, canonical status and canonical priority filters combine with each other and Task search using AND semantics. The Project selector includes Personal inbox and every Project represented by the loaded Projects or Task records. Filtering is local, uses the same result set in List and Board, and never edits persisted records. Clear filters restores all active tasks; no-match and data-read error states remain distinct.

### M2-022 due-date and priority indicators
Task List rows and Board cards show readable due dates and labelled canonical priorities. Due timestamps remain UTC instants in storage and display in the user's local date. An unfinished task is labelled Overdue only after its due instant has passed; Done and Cancelled tasks are not marked overdue. Invalid or absent due dates never create an overdue signal.

## 5. Command Palette (Ctrl+K)

Search categories:
- Navigation pages
- Create actions (project, task, note)
- Registered tools
- Recent items
- Settings
- Safe quick actions

Behavior:
- Show keyboard navigation and selected item.
- Search active local Project names/descriptions/goals, Task titles/descriptions, and registered command names from this single entry point.
- Identify each result as a Project, Task or Command; preserve the command category and risk indicator.
- Project and Task results open their existing detail routes. Arrow keys and Enter work for all result types.
- Keep registered commands and successful local sources available if another source read fails; show which local source failed and offer retry.
- Distinguish a successful no-match result from an empty command registry.
- Do not execute a destructive/high-impact operation directly from a search result; open its preview/confirmation flow.
- Search result ranking should be deterministic and work offline for local commands.
- Future AI semantic matching must supplement, never replace, literal search.

## 6. System Tray flows

Tray menu (target):
- Open Moonships
- Quick Tools (only implemented/safe tools)
- My Work / Waiting Approvals
- Recent Activity
- Pause/Resume background monitoring (if that feature exists)
- Settings
- Quit

Required behaviors:
- X/close button follows the user's configured behavior: Hide to Tray or Quit. Make first-time behavior clear.
- If hidden, main process stays alive only when configured or required by an active task.
- Tray icon indicates states only if the state is real and distinguishable (idle, task running, approval waiting, error); do not rely on color alone.
- Context menu supports keyboard and mouse.
- Quit is explicit and shuts down worker tasks cleanly.
- Startup-at-login is opt-in and reversible.
- No background network fetches before the user enables/configures them.

## 7. Core user flows

### Flow A — first launch
1. Launch app.
2. Show short welcome and explain local-first behavior.
3. Let user choose a profile: Developer, Network/NOC, Sysadmin, Security Research, Intelligence/Research, or Custom.
4. Profile only changes pinned tools/layout; it must not silently grant permissions.
5. Offer Create Project, Create Task, or Open Toolbox.
6. Explain tray behavior and ask whether to start with Windows (opt-in).
7. No account, cloud registration or API key required.

### Flow B — create and complete a project task
1. Create project with name and goal.
2. Create task with small, clear deliverable and acceptance criteria.
3. Optionally add subtasks/dependencies.
4. Start work; task activity records meaningful status changes.
5. Attach result/artifact or reference.
6. Mark Done only when acceptance criteria are met.
7. History records the final status and time.

### Flow C — safe cleanup
1. User selects categories.
2. Scanner runs read-only and reports progress.
3. Show candidates, paths (appropriately truncated), category, size, last modified where relevant, and skip reasons.
4. Policy filters candidates; unsafe paths are not selectable.
5. User selects items and opens a preview.
6. Revalidate candidate identity/path immediately before action.
7. Execute eligible deletions; skip locked, changed, denied or out-of-policy files.
8. Show bytes actually removed, failures and skipped items.
9. Never claim scan-estimated bytes were actually freed.
10. Keep a clear option to export the report; do not promise restoration after permanent deletion.

### Flow D — network diagnosis
1. Choose a target from a saved profile or enter a target.
2. Accept one hostname/IP for DNS or one hostname/IP and a single port for TCP; do not accept URLs, ranges or wildcards.
3. Show the exact target, requested operation and timeout before running a selected diagnostic within the rate limit.
4. Display raw result, parsed summary, timestamp and failure state.
5. Offer “Create task from result” with the relevant evidence attached.
6. Do not automatically expand the target range or scan unrelated hosts.

### Flow E — World Monitor / trend research
1. User enables a connector or selects built-in public sources.
2. Explain source, cadence, privacy and provider limits.
3. Ingest/normalize records with canonical source URL and timestamps.
4. Deduplicate while preserving source relationships.
5. Show trend baseline and comparison window; disclose missing data.
6. Summary must link back to sources and separate facts, interpretation and uncertainty.
7. Save to Research Desk or create a task/project.
8. Stale or blocked connectors show degraded state, never fabricated live status.

### Flow F — security assessment
1. Create/select assessment and document authorization/scope.
2. Validate each target against the declared scope before execution.
3. Select a low-risk check and show the planned request/actions.
4. Apply policy and any required approval.
5. Execute with rate/time limits and cancel support where safe.
6. Record evidence, versions, timestamps and failures.
7. Turn validated findings into tasks with remediation guidance.
8. Do not perform out-of-scope, destructive, stealth, persistence or credential-theft actions.

### Flow G — AI-assisted task/tool use
1. User enters a natural-language request.
2. AI proposes an interpretation and lists evidence/tools it wants to use.
3. Resolve tool IDs only from the registry; validate every argument against schema.
4. Apply policy independently of model output.
5. Read-only calls run within granted permissions; consequential calls show a preview and request explicit approval.
6. Record provider/model (when appropriate), tool calls, results and citations without storing secrets.
7. Treat web pages, logs, issue text, code and source content as untrusted data, never as policy instructions.
8. If model/network is unavailable, expose normal non-AI workflows.

### Flow H — update and recovery
1. Fetch update metadata from configured trusted endpoint.
2. Verify signature before installing.
3. Explain version, source and release notes.
4. Preserve user data and backup/migrate schema transactionally.
5. On failure, show clear recovery instructions and preserve diagnostics.
6. Never install unsigned/unverified updates silently.

## 8. Visual design system (delegated)

The visual identity, canonical design tokens, component variants, motion rules, accessibility contract, asset manifest and visual QA gates are defined in [Design System](DESIGN_SYSTEM.md). Do not define parallel colors, spacing scales or component behavior here. The UX spec owns the information architecture and page-level intent; implementation must follow the canonical design system.

## 9. UI states that every feature must define

Every data/action screen must document and implement:
- Initial / not configured
- Loading / in progress
- Success / results
- Empty results
- Partial result
- Stale data
- Offline / provider unavailable
- Permission denied
- Invalid input
- Cancelled
- Timeout
- Recoverable error
- Fatal/unexpected error

A screen is not done merely because its happy path renders.

## 10. UX acceptance checklist

- [ ] Sidebar labels and page hierarchy are consistent with the menu spec.
- [ ] No menu entry implies that an unimplemented module works.
- [ ] Global search works on local commands without AI.
- [ ] Focus order and keyboard shortcuts are documented.
- [ ] Dark/light themes share semantic color tokens.
- [ ] Empty, loading, offline, stale, permission-denied and error states are designed.
- [ ] Destructive operations require a preview and separate confirmation.
- [ ] Tray lifecycle follows user settings and supports a genuine Quit path.
- [ ] Long tasks display progress and an honest cancellation capability.
- [ ] All primary interactions have keyboard support and visible focus.


## 11. Low-fidelity screen layouts

These are functional wireframes, not final pixel-perfect mockups. Keep the same global shell and design tokens across domains.

### Main window

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ MOONSHIPS  [Workspace ▾]       [ Search / Ctrl+K................ ] [Activity]│
├──────────────────┬───────────────────────────────────────────────────────────┤
│ Home             │ Page title                        [Primary action]       │
│ My Work          │ Breadcrumb / scope / freshness                           │
│ Projects         ├───────────────────────────────────────────────────────────┤
│ Tasks            │                                                           │
│ Toolbox          │ Main content                                               │
│ Security         │ - Cards/tables/forms appropriate to this tool              │
│ Intelligence     │ - Empty/loading/error states have the same layout          │
│ Operations       │ - Evidence/source/date and risk are visible when relevant  │
│ Knowledge        │                                                           │
│ Integrations     │                                                           │
│ Settings         │                                                           │
├──────────────────┴───────────────────────────────────────────────────────────┤
│ Status: Local workspace · Active jobs: 0 · Data freshness / connectivity     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Home / command center

```text
[Good morning / active profile]                              [New task] [Tools]
[Search everything: projects, tasks, tools, notes, sources]               

[My Work: overdue] [Due soon] [Blocked] [Waiting approval] [Active jobs]

[Continue working: recent project/task/tool]
┌────────────────────────────────┐ ┌─────────────────────────────────────────┐
│ Projects in progress            │ │ Recent activity / active jobs           │
│ project + progress + next step  │ │ status + time + result + open details    │
└────────────────────────────────┘ └─────────────────────────────────────────┘
[Optional system/source status widgets only when they have real data]
```

Do not show fabricated CPU/network/World Monitor data. First-run Home should show next steps, not empty decorative graphs.

### Project detail

```text
Projects / MOONSHIPS
[Name] [Status] [Edit] [More]
Goal / summary / target date / project health with explainable criteria
[Overview] [Tasks] [Roadmap] [Notes & Artifacts] [Activity] [Settings]
Tasks: [List | Board]   [Search] [Filters] [New task]
```

### Task detail

```text
[Task title] [Status] [Priority] [Save]
Project / parent / labels / due date
Description
Acceptance criteria (if work item)
Subtasks / dependencies
Linked artifacts (tool run, issue, source, report, URL)
Activity history (who/what/time; local profile in MVP)
```

Keep basic task creation short. Expand advanced fields only on demand.

### Cleaner result screen

```text
Clean & Storage                                   [Start scan]
[Category checkboxes]  Scan status / last scan / scope disclaimer
[Estimated recoverable space] (estimate, not guaranteed actual freed space)
┌ Select │ Category │ Item summary │ Size │ Status / skip reason ┐
[Preview selected actions] [Clean selected — only after preview/approval]
Results: removed / skipped / failed / actual bytes freed
```

No candidate is actionable until it passes policy; an unsafe candidate should be omitted or visibly blocked, not merely unchecked by default.

M3-007 exposes Clean & Storage as a workspace route and adds a read-only preview view for typed candidate data: eligible/skipped counts, estimated bytes, category, item label, size, modification time and skip reason. The route shows an explicit unavailable state when no scan preview exists; it does not invent candidates or start a scan. M3-008 lets the user select eligible items and opens a separate approval dialog bound to that preview and selection. Skipped items cannot be selected. Confirmation records intent only; cleanup is not wired to this screen and no files are changed. M3-014 adds recent persisted report summaries with estimate, successfully deleted logical file size, item outcomes and an optional Task link; the history never displays or stores candidate paths by default.

### Intelligence monitor

```text
[World Monitor] [Topic / region / source filters] [Freshness indicator]
[Map or timeline view]                         [List of source-attributed items]
[Category / source / published time / observed time / freshness]
[Trend panel: metric + window + baseline + coverage/uncertainty]
[Save to Research] [Create task] [Open source]
```

A map is optional until source location precision, rights, coverage and update cadence are documented. List/timeline remains useful when mapping data is unavailable.

### Intelligence sources (M5-008)

The implemented route is **Intelligence Sources** (`#/intelligence-sources`). It lists only locally saved RSS/Atom declarations with enablement, terms-review state, minimum poll interval, freshness and last-success time. An honest loading, unavailable/retry and empty state are distinct. Add/edit controls collect a display name, HTTPS feed URL, optional terms URL, user-entered access notes, terms-review choice and minimum interval. New sources begin disabled; enabling is a separate action. Changing the URL disables the source and clears its previous freshness. An official NWS active-alert preset lets the user select one state or DC area; adding it only saves a disabled, terms-unreviewed source, and fetching still requires a separate manual action. The 30-minute minimum and provider limitations are visible in the source settings; its information is not emergency alerting.

The interval is not a background schedule. The page does not fetch when opened, and it offers no automatic polling until a later task adds a separately reviewed opt-in scheduler. Enabled means eligible for an explicit manual fetch only. Do not imply fresh/live data without the typed freshness view.

### Topic Watchlists (M5-009)

The **Topic Watchlists** page saves named keyword/category filters with distinct active, paused, empty and unavailable states. Keywords match literal text in an item's title or summary without case sensitivity. Category IDs match exactly; alternatives within a filter group are OR, while populated keyword and category groups combine with AND. Either group can be omitted, but a watchlist must contain at least one filter. Pause stops matching while preserving the saved filter. Delete requires confirmation and removes only the watchlist record.

### Intelligence list and item detail (M5-012)

The **Intelligence** route (`#/intelligence`) lists only locally persisted source-attributed items. Search, source and category filters operate on this bounded saved list. Rows expose source, category, publication time, observation time and freshness status; missing publication or event times remain “Not provided”. Selecting an item opens an ID-scoped detail route with summary, verification state, all distinct timestamps, validated source-feed link and original item link. Initial empty storage, filtered no-results, loading, unavailable/retry, fetch failure, stale source and disabled-source states remain distinct.

Fetching is a separate explicit **Fetch now** action shown beside configured sources. It is available only for enabled, terms-reviewed sources, uses the bounded native RSS policy, persists a complete successful result locally and displays last attempt/success and health. Opening the list or detail never fetches or polls. The list shows at most 200 recent records from a bounded local retention set; data remains on this device.

Desktop notifications (M5-014) are off by default. Enabling them in Settings requests OS permission from that action. Moonships sends a generic notification only after a successful manual fetch returns items; the notice contains no feed or item text. This setting does not start fetching or background monitoring. The OS controls quiet hours and notification presentation; the separate background-monitoring control remains Planned.

### Research Desk source notes (M5-013)

From an Intelligence item detail, **Save to Research** creates or opens one local note with an ID-based item link and source attribution. The Research Desk list links back to the item and offers an explicit **Export note with provenance** action. Versioned export preserves source ID, validated source and original URLs, publication/observation timestamps and verification state without copying the fetched item body. If item retention later removes the linked item, the note remains and displays the reference as unavailable. Export cancellation and failures are reported; an existing file is never silently replaced.

### Settings

```text
Settings navigation               Current section
General                           Startup behavior (opt-in)
Appearance                        Theme and density
Startup & Tray                    Close-to-tray / launch-at-login / tray icon
Notifications                     Categories and quiet behavior
Sound & SFX                       Master toggle, mode, volume and test preview (off by default)
Data & Privacy                    Storage location / export / delete / network usage
Modules                           Enabled / planned / unsupported
Permissions & Security            Capabilities / approvals / active scopes
AI Providers                      Optional provider setup / data handling
Integrations                      Connected state / permission / rate limits
Updates                           Version / channel / signature status
Diagnostics                       Logs / redaction / export
About                             Version / licenses / documentation
```

## 12. Design token source of truth

The proposed palette, semantic token names, typography, spacing, radii, elevation, density and motion tokens are maintained only in [Design System](DESIGN_SYSTEM.md). Values are proposals until implemented and measured for contrast in both themes.


## Workspace profile onboarding (M1-034)

On first launch, offer six local workspace profiles: Developer, Network / NOC, Sysadmin, Security / Research, Intelligence / Research, and Custom. A profile only selects suggested pinned sections/layout hints; it never grants permissions, enables a module, or starts a tool. The profile selector remains available from the top bar so users can change it later. The selected profile is stored in local WebView storage; it does not imply cloud sync or native permission state. If storage is unavailable, keep the selection for the current session and state that it was not persisted.

## Process inventory (M4-008)

The Processes page is a user-requested, read-only snapshot with an explicit **Refresh snapshot** action; it does not poll in the background. Show executable name, process ID, parent process ID, thread count and snapshot time in a semantic, horizontally scrollable table. Do not show a kill/terminate action, command line or executable path. Loading, adapter failure, permission denied, unsupported, empty and partial/capped snapshots have distinct text states. A permission denial never offers elevation. Use the shared page/table tokens and keyboard-visible focus styles from [Design System](DESIGN_SYSTEM.md).

## Windows service inventory (M4-009)

The Services page is a local, read-only snapshot with an explicit **Refresh snapshot** action; it does not poll. Show display name, service name, state and snapshot time in a semantic table. Do not show start, stop, restart, configuration or elevation actions. Permission denied, manager unavailable, unsupported, empty and partial results have distinct messages, while a successful empty list remains distinguishable from a failed query. Use shared table, loading/error and focus patterns from [Design System](DESIGN_SYSTEM.md).
