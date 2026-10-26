# Data Model & Persistence Blueprint

**Status:** Logical model with implementation notes updated as migrations are accepted.

## 1. Identity and time conventions

- Use stable UUIDs (or a documented UUID-compatible ID format) for entities; never use list position as identity.
- Store timestamps in UTC ISO-8601 or integer Unix milliseconds consistently; prefer ISO-8601 UTC at JSON/API boundaries.
- Render timestamps in the user's locale and timezone.
- Store created/updated timestamps on mutable records.
- Store soft deletion/archive only where the restore/audit use case justifies it; define retention.
- Define database schema version and execute migrations transactionally.
- SQLite foreign keys must be enabled for every connection.

## 2. Core entities

### External adapter manifest (M6-001)
`AdapterManifest` is a data-only contract: stable ID, name, numeric `major.minor.patch` version, closed kind, bounded description and unique closed capability labels. `AdapterHealth` associates an adapter ID with explicit availability, optional RFC 3339 check time and bounded diagnostic text. Neither object is persisted or executable in this task; capabilities are descriptive and grant no authority. Future install/runtime lifecycle remains separate.

M6-002 schema v16 adds a bounded local HTTP inspection history of at most 200 rows. Each versionless strict JSON row contains an ID, observation time, redacted URL, status, byte count, fixed request-header metadata, redacted response headers and an optional stable error code. Persistence redacts sensitive header names again at the storage boundary and redacts common credential query keys. Response bodies and request credentials/cookies are never persisted. The current GET adapter does not accept custom request headers, non-GET methods or non-443 ports.

M6-003 creates no database records. A native picker returns a bounded UTF-8 document and leaf filename for the current view only. The parsed OpenAPI metadata and original text are in-memory UI state and are discarded when the page is left; no paths, source contents or resolved references are persisted.

M6-004 stores no new link table: a follow-up Task description records the opaque HTTP history ID and a bounded metadata snapshot (redacted URL, status/error, observation time and response byte count). The source history remains separately capped at 200 rows; if it is evicted, the ID remains in the Task as provenance, but its history detail is no longer available. Response bodies and headers are excluded.

M6-005 observability provider metadata is a strict in-memory DTO: stable bounded ID/name, one or more fixed signal kinds (logs, metrics, traces), fixed health status, optional last-success RFC 3339 time and bounded freshness threshold. Its derived view separates provider status from freshness and sets `is_current` only for a ready provider with a valid non-future timestamp within threshold. No endpoint, account, token, event payload or raw telemetry field exists in the contract.

M6-006 local metrics are an in-memory snapshot only: fixed source label, RFC 3339 sample time, and fixed numeric fields for logical processor count, physical memory total/available and Windows volume total/free. Each field carries an available/unsupported/unavailable state; unavailable values are null rather than fabricated zeroes. No snapshot table, remote telemetry payload or background series is stored.

M6-007 assessment scope is a strict declaration containing up to 100 exact hostname/IP targets, up to 100 exclusions, a non-empty authorization note capped at 2,000 characters, and an RFC 3339 expiry no more than 365 days ahead. Hostnames normalize to lowercase ASCII DNS labels; IP literals normalize through the standard IP parser. Starting validation fails for empty scope, invalid/duplicate entries, missing authorization or invalid/expired/overlong expiry. This DTO is not yet persisted and has no executor.

### Workspace
Purpose: local grouping of projects, settings and views.
Fields: id, name, description?, profile_type?, created_at, updated_at, archived_at?.
MVP default: one Personal workspace, with support for future multiple workspaces. A workspace does not imply cloud sync or team collaboration. The M2-001 schema-v2 migration seeds `Personal` with stable local UUID `00000000-0000-4000-8000-000000000001` and stores optional description/profile/archived fields plus SQLite-generated UTC creation/update timestamps. The seed is applied once; no cloud identity or access privileges are inferred.

### Initiative / Goal
Purpose: a broad desired outcome grouping projects.
Fields: id, workspace_id, title, description?, status, target_date?, created_at, updated_at.
Optional in MVP if it would delay basic project/task delivery.

### Project
Fields: id, workspace_id, key?, name, description?, goal?, status, color_token?, start_date?, target_date?, archived_at?, created_at, updated_at.
Project key is user-friendly metadata, not the primary ID. Avoid forcing a repository path or Git provider. The M2-002 schema-v3 migration creates a workspace-scoped `projects` table with `project_key` optional and unique when supplied within a workspace; `name` cannot be blank, `status` defaults to `planned`, and workspace references are foreign-key enforced. The M2-007 native service supplies a random UUID v4, validates fields and active workspace membership, and creates the project transactionally. M2-014 adds scoped listing, typed native list/create/update commands, and the Projects list/form for the seeded Personal workspace. The list excludes archived records and sorts by case-insensitive name then stable ID; every command opens SQLite under Tauri's per-user app data directory and reuses the validated services. M2-015 adds a scoped overview read that returns the active Project, count of non-archived linked Tasks, and up to ten newest safe ActivityEvent summaries for the Project and currently linked Tasks. It returns no event metadata or user-authored text. The overview UI links to Task creation with the Project preselected and represents zero Tasks accurately. Project UI fields are name, goal, description, and (on edit) status. Other workspace selection and archive controls remain future work.
Statuses: planned, active, paused, completed, archived.
M2-008 adds transactional scoped update/soft-archive and trusted change events. No-op updates leave timestamps and activity unchanged. Creating a project directly in archived state is rejected; existing linked tasks survive archive.

### Milestone
Fields: id, project_id, title, description?, due_date?, status, sort_order, created_at, updated_at.
Milestone can group tasks by release/outcome. Do not make milestone date a hidden dependency for completing a task. M2-024 implements migration v8 with a workspace- and Project-scoped `milestones` table and a `milestone_tasks` association table. Composite foreign keys prevent cross-Project links. Due dates are validated as real `YYYY-MM-DD` calendar dates; status is `planned`, `active` or `completed`; stable project ordering uses `sort_order`. The typed list/create/update/associate commands derive the Personal workspace natively. Associating or updating a Milestone never mutates Task status, and archived Tasks are excluded from the Project view.

### Task
Fields: id, workspace_id, project_id?, parent_task_id?, title, description?, status, priority, due_at?, estimate_minutes?, completed_at?, created_at, updated_at, archived_at?.
Canonical statuses: backlog, todo, in_progress, blocked, in_review, done, cancelled.
Canonical priorities: p0, p1, p2, p3. Priority is not security severity or execution risk.
M2-012 initial policy: any explicit transition between the seven canonical statuses is permitted, including reopening a completed/cancelled task; unchanged status is a no-op, unsupported values are rejected. All UI mutations must use the task service rather than a view-specific state machine. M2-010 adds a workspace-scoped transactional Task update service; UI and native IPC integration remain separate tasks.
Task must be creatable with a title alone and defaults. Project is optional for a personal inbox task. The implemented M2-003 schema-v4 migration adds checked `backlog`/`p2` defaults, nonblank task titles, optional project/parent links, and workspace-scoped composite foreign keys. UTC timestamps use the existing SQLite convention. The M2-009 create service validates fields and generates UUID v4 IDs. M2-016 adds a typed, camelCase Tauri create command scoped to Personal and a persisted React dialog. The dialog explicitly supplies `todo`/`p2` as its UX defaults, allows an active Project or no Project, and confirms only the returned saved record. M2-018 adds typed, read-only Task list IPC fixed to Personal and requests active records only; the Tasks route displays returned records and sorts a copy deterministically without changing storage. M2-015 overview count uses the same native non-archived scope. M2-017 adds scoped Task detail read and update IPC. The detail form edits title, description, canonical status/priority and optional due date/time while retaining the saved Project association. The local datetime control converts the user's local choice to an RFC 3339 UTC instant; the validated service normalizes and persists it. Task mutations from the list remain future tasks.

M2-013 `due_at` contract: the JSON field is `dueAt` and carries either `null`/omission or an RFC 3339 timestamp with an explicit `Z` or numeric UTC offset. Date-only and timezone-free local strings are invalid. Create/update normalize accepted instants to canonical UTC RFC 3339 with `Z`, preserving fractional-second precision, and SQLite stores that same value. Blank optional input clears the due time. UI adapters render the persisted instant in the user's current local timezone; domain services never guess a timezone or store locale-formatted text.

M2-023 uses the existing nullable `parent_task_id` column and workspace-scoped composite foreign key. A Task may reference only an active top-level Task in the same workspace with the same nullable Project association; this gives one-level nesting and prevents cycles through the only create path. Parent links are immutable after creation in this slice. A root with active children cannot be archived, a child cannot be restored while its parent is archived, a child cannot change Project independently, and a root with any children cannot change Project. The typed `parentTaskId` field is included in Task create requests and Task responses; workspace scope still comes from the native Personal workspace boundary, never the renderer.

### TaskDependency
Fields: task_id, depends_on_task_id, dependency_type, created_at.
M2-005 migration v7 stores one directed relation: `task_id` is `blocked_by` its prerequisite `depends_on_task_id`; the inverse “blocks” wording is derived and is not stored as a second edge. Both Task references are constrained to the same workspace, the self-edge is rejected by both the service and SQLite, and repeating the same edge is idempotent. The current model does not use dependency edges for scheduling or reject longer cycles. If scheduling starts consuming the graph, cycle rejection must be added before that behavior ships.

### TaskLabel / Label
Labels have a workspace-scoped identity and many-to-many Task associations through `task_labels`; do not store label arrays in Task rows. M2-004 migration v6 uses a composite workspace foreign key for both sides of the relationship. A label's duplicate identity is its trimmed display name after internal whitespace is collapsed to one ASCII space and Unicode lowercase conversion is applied. Thus `Needs Review`, ` needs   review ` and `NEEDS REVIEW` conflict within the same workspace. The first label's trimmed, whitespace-collapsed spelling is retained for display. The same normalized name is allowed in another workspace. This rule does not perform Unicode canonical-equivalence normalization. Repeating the same Task-label association is idempotent.

The native `labels` service validates nonblank names up to 64 characters, creates and lists workspace labels, and adds, removes and lists Task-label relationships. Cross-workspace associations are rejected by composite foreign keys. Label deletion and UI/IPC exposure are not part of M2-004.

### Comment
Fields: id, target_type, target_id, body, created_at, updated_at.
MVP can scope to task/project comments. If polymorphic foreign keys reduce integrity, use separate task_comments/project_comments tables instead.

### ActivityEvent
Fields: id, workspace_id, actor_type, actor_id?, entity_type, entity_id, action, occurred_at, correlation_id?, summary, metadata_json?, sensitivity.
Keep audit/activity distinct from verbose debug logs. Sanitize metadata and never log secret values.

Implemented subset (M2-006, migration v5): `activity_events` stores workspace-scoped entity type/ID, one project or task foreign key, `created` action, UTC timestamp, trusted fixed summary, actor type and sensitivity. Project/task create services append the event in the same transaction as the entity write; events are not fabricated for preexisting records. The additional optional actor ID/correlation ID/metadata fields described above are not persisted by this MVP subset and must be explicitly migrated before future usage. Never place user-supplied descriptions, credentials or arbitrary metadata into event summaries.

M2-025 extends Task update events without changing the schema: status changes use `status_changed`, priority changes use `priority_changed`, and meaningful title/description/due-date/Project edits use `updated`. A single edit may record multiple applicable event types within the Task update transaction; no-op saves create none. Task history reads are workspace-scoped to an active Task, ordered newest-first with stable insertion ordering for tied timestamps, and bounded to 50 events. Summaries are fixed trusted text and never copy Task titles, descriptions, credentials or arbitrary user data.

### Job / Activity Center

Schema v10 stores a UUID, bounded title, execution status, UTC creation/start/finish timestamps, optional bounded stage/progress and a bounded result summary. Status transitions are validated in the native service; terminal states cannot be reopened. The renderer can only read the latest 200 jobs. This is execution history, distinct from entity activity events and diagnostic logs. Job producers, interrupted-job recovery and cancellation are not yet wired; no sample records are inserted.

M4-014 keeps network result handles in a process-local cache capped at 100 entries. A handle references the actual native result rather than renderer-supplied summary text. Creating a follow-up Task builds its description from the bounded target, operation, outcome, UTC observation time, duration, source and closed error code. Resolved address lists and raw OS messages are omitted; handles expire on restart or eviction.

M2-029 export schema v1 is a strict JSON document with `format: "moonships.project-task-export"`, `schemaVersion`, `appVersion`, `exportedAt` (UTC RFC 3339), and `projects`, `tasks`, and `activityEvents` arrays. It exports all Personal workspace Project/Task columns, including dates, estimates, completion/archive metadata, stable identifiers and relationships, and archived records. Only normal-sensitivity trusted activity is included. App settings, `app_metadata`, logs, credentials, and restricted activity are excluded. Native validation checks the manifest, canonical Project/Task values, workspace scope, unique record IDs and in-export relationships, then serializes and parses against the strict typed schema before save.

M2-030 reuses the strict schema-v1 validator for a read-only import preview. Native code bounds input to 50 MiB and 100,000 total records before returning manifest metadata, counts, and at most five names/titles. This preview has no database connection or write path; any future import/commit flow requires a separate task and explicit user confirmation.

M2-033 Home reads only the fixed Personal workspace through a native read-only command. Task counts are derived from active SQLite rows at a UTC `asOf` instant: open excludes Done/Cancelled, overdue is before `asOf`, due soon is from `asOf` through seven days, and blocked uses the canonical `blocked` status. It returns at most five recently updated active Projects, five Tasks for each due list, and ten newest normal-sensitivity Activity events. Archived rows and restricted Activity are omitted; there is no dashboard cache or synthetic event source.

### Artifact / Reference
M2-026 reference contract fields: id, workspace_id, project_id?, task_id?, reference_kind, title, canonical_url?, provider_id?, provider_key?, managed_artifact_id?, media_type?, content_hash?, source_id?, created_at, sensitivity. Exactly one target is present: URL, provider identity, or opaque app-managed artifact ID. The reference row stores metadata only; binary payloads belong in app-owned artifact storage, not relational rows. External URLs must parse as absolute HTTP(S) URLs with a host and no embedded credentials; reject `file:`, `javascript:`, `data:` and unknown schemes. Provider references retain a stable provider namespace and opaque ID; they do not construct or fetch a URL unless a future reviewed provider adapter defines that behavior. Managed artifact references use an opaque ID resolved by the native storage layer beneath the app-owned artifacts directory; renderer-supplied absolute paths are not accepted. No target is automatically opened, fetched, downloaded or executed. Opening is a separate explicit user action routed through a native allowlisted adapter after target revalidation. Do not embed arbitrary files or credentials in URLs/references by default.

### CleanupReport (M3-014)
Version 9 stores a path-free cleanup summary: id, workspace_id, optional task_id, preview_id, status, completed_at, selected/deleted/skipped/failed counts, estimated_bytes and actual_deleted_bytes. A composite foreign key keeps an optional Task link in the same workspace. Database checks enforce the 100-item cap, consistent outcome counts and actual bytes no greater than the estimate. No candidate IDs, filenames or paths are retained. `actual_deleted_bytes` is the logical size of successfully removed files, not physical disk space reclaimed.

### SavedView
Fields: id, workspace_id, view_type, name, filter_json, sort_json?, layout_json?, created_at, updated_at.
Validate saved view schema and migration version.

### AppSetting
Fields: key, value_json, schema_version, updated_at.
Only non-secret preferences; secrets use OS credential storage or a dedicated secure abstraction. The initial typed config schema is version 1 with dark theme, Developer profile, Info logging and hide-to-tray disabled by default. Reject unknown fields and unsupported schema versions rather than silently coercing them.

### ToolDefinition (registered tool metadata)
Fields: id, version, api_version, name, description, category, platform_support, input_schema_json, output_schema_json, permissions_json, risk_class, enabled, source_type, created_at.
Do not store executable code in the registry. MVP tool definitions are compiled-in. The native contract includes ID, semantic version, API version, input/output schemas, risk class, required permissions and enabled state.

### ToolRun / Job
Fields: id, tool_id, tool_version, input_redacted_json, status, started_at?, finished_at?, duration_ms?, output_summary?, error_code?, error_redacted?, initiated_by, approval_id?, correlation_id?, cancellability, created_at.
Canonical job statuses are queued, running, waiting_approval, succeeded, partial, failed, cancelled and timed_out. Sensitive arguments and raw secrets must not be persisted. Large outputs go to artifacts with bounded retention.

### ApprovalRequest
Fields: id, action_id, tool_id, target_summary, risk_class, preview_json, requested_at, expires_at?, decision, decided_at?, decision_context?.
Approval must be bound to the exact action/arguments/target. If target or arguments change, invalidate the approval and create a new preview.

### Connector / Source
Fields: id, kind, display_name, enabled, config_reference?, polling_interval_seconds?, last_attempt_at?, last_success_at?, freshness_threshold_seconds?, health_status, terms_url?, created_at, updated_at.
Secrets are referenced, not embedded in config JSON.

M5-001 adds a typed in-memory declaration with an explicit access method and terms review state, HTTPS-only source/reference URLs, bounded minimum cadence (60 seconds to 30 days), request timeout (100–30,000 ms), response size (at most 10 MiB), item count (at most 500), and freshness threshold no shorter than the poll interval. RSS/API/authorized-export kinds must match their access method. Enabling or deserializing an enabled connector rejects missing/invalid IDs, limits, terms review or health. No connector record is persisted and no source is enabled by default.

M5-002 implements the item shape as a strict typed DTO. It keeps `source_id`, source `source_url`, item `canonical_url`, optional `published_at`, required `observed_at`, and optional `event_time` separate. Supplied timestamps must include an explicit timezone and are normalized to UTC; the validator never fills a missing publication/event time with observation time. The record keeps attribution, freshness and verification states, and bounds external IDs, text/category, and a lowercase SHA-256 content hash. Persistence and URL canonicalization/deduplication remain later tasks.

M5-003 reads one user-enabled RSS 2.0 feed on an explicit native call and returns normalized `IntelligenceItem` values in memory. The feed's stable connector ID and source URL remain attached; RSS `pubDate` is normalized to UTC when valid, `observed_at` is recorded separately, and event time is left absent. No feed or item is stored, scheduled, or enabled by default. The process-local rate limiter is reset when the app restarts; durable last-attempt state and source settings are later work.

M5-004 tracks per-connector failure count and retry eligibility in memory. After a failed fetch, retry is blocked until both the declared minimum poll interval and an exponential delay have elapsed; delay starts at 60 seconds and caps at one hour. Success clears the failure count. No retry is started automatically, and the state resets on app restart until connector persistence is added.

M5-005 stores each item's exact `original_url` alongside its `canonical_url`; validation checks that canonical URL is the conservative normalization of the original. Canonicalization lowercases/normalizes URL syntax and removes fragments plus known analytics parameters (`utm_*`, `fbclid`, `gclid`, `dclid`, `msclkid`, `mc_cid`, `mc_eid`) while retaining path and all other query pairs. A version-tagged SHA-256 fingerprint is computed from title and optional summary after Unicode whitespace is collapsed; letter case and punctuation are retained. Deduplication and source relationship grouping remain separate work.

M5-006 groups at most 2,000 validated item records by shared canonical URL or recomputed content fingerprint. The output keeps all original item records (including source ID, feed URL, original item URL and timestamps) and lists each distinct source ID once. It does not overwrite, delete or persist any source item; the first record in each group is only the display representative.

M5-007 exposes a `ConnectorFreshnessView` with status, normalized `lastSuccessAt`, and an `isLive` guard. Freshness is computed against the connector's configured threshold; future/invalid/missing timestamps fail closed. A successful RSS call updates the supplied connector's in-memory health and last-success time; a failed request updates health/last-attempt while retaining the previous last success.

M5-008 persists user-configured RSS source declarations in the app-owned SQLite database. Each source has a stable generated ID, bounded HTTPS URL/terms metadata, user-declared terms review, minimum polling interval, and bounded request limits. New sources start disabled; enabling requires reviewed terms. Editing a source URL disables it and clears previous freshness. The source list reads a typed connector/freshness view. Settings persistence is local; item records and fetch results are not persisted by this task. The cadence is a declared minimum interval only: there is no background scheduler, startup fetch, or fetch when the source screen opens. The RSS adapter still rejects disabled connectors on any future manual fetch path.

M5-009 persists up to 128 local topic watchlists. Each bounded watchlist requires at least one keyword or category filter; up to 20 of each can be saved. Keywords match case-insensitively as literal substrings in title or summary; categories match exact normalized category IDs. When both groups are populated, a match must satisfy at least one value in each group. Paused watchlists never match. Deletion removes only the saved filter. No intelligence item records, fetches, schedules or notifications are added by this task.

### TopicWatchlist
Fields: id, name, keywords[], categories[], paused, created_at, updated_at. IDs are generated locally; filter counts and string lengths are bounded. The record contains no connector credentials, source content, or executable matching expression.

### Connection profile contract (M4-011)
Profiles have a UUID ID, bounded display name and one tagged allowlisted transport: a known local terminal enum, SSH or RDP. Remote transports use a strict single host, nonzero port, optional username and optional opaque UUID secret reference. No password, private key, command line or arbitrary executable path field exists. The dedicated export DTO omits secret references; the safe log projection omits user content and target details. M4-011 defines the contract only: saved profile persistence, OS secret-store integration and transport execution remain future reviewed work.

### IntelligenceItem
Fields: id, source_id, external_id?, canonical_url, title, summary?, published_at?, observed_at, event_time?, category, content_hash?, normalized_metadata_json?, freshness_state, verification_state.
Preserve source attribution and collection time. Keep the raw payload only when the source terms and privacy policy permit.
Deduplication must not erase source relationships.

### Signal / TrendObservation
Fields: id, subject_key, category, observed_window_start/end, comparison_window_start/end, count, baseline_count?, delta_ratio?, method_version, source_count, confidence_label, created_at.
A trend requires declared observation and comparison windows. Avoid calling an item “trending” based only on a global total.

The pure `WindowTrend` result compares adjacent equal-length half-open windows (maximum 90 days each) per watchlist/category, using validated `observed_at` timestamps. It counts deduplicated attributed items, preserves current-window source count, and records both window boundaries and the method version. Each window requires explicit selected/successful/failed source evidence. Coverage is labeled `no_sources`, `no_successful_fetch`, `partial`, or `complete`, with unattempted sources counted separately. A count is `null` when no source fetch succeeded and may be `0` only when at least one source succeeded but no matching item was observed. Relative change is present only for complete windows with identical selected source scopes and a nonzero baseline. Categories with no item in either supplied window produce no row, while result-level coverage still distinguishes a complete empty observation from unavailable data. These rules prevent missing data from appearing as zero without a caveat.

M5-012 stores validated `IntelligenceItem` JSON with source ID, category and normalized observation timestamp indexes. Retention is bounded to the most recent 10,000 items. Native list queries return at most 200 rows and support bounded source/category/search filters; detail is read by item ID. Records are written transactionally only after an explicit RSS fetch returns a complete valid result. No partial feed results are persisted.

### WorkflowDefinition
Fields: id, workspace_id, name, version, definition_json, enabled, permissions_json, created_at, updated_at.
Definitions must be schema-validated and versioned.

### WorkflowRun
Fields: id, workflow_id, workflow_version, status, started_at?, finished_at?, trigger_type, result_summary?, error_code?, correlation_id, created_at.
Each step execution records the tool ID/version, input redaction, approval, result status and duration.

### ResearchCollection / Note
Collection fields: id, workspace_id, name, description?, created_at, updated_at.
Note fields: id, workspace_id, collection_id?, title, body, created_at, updated_at.
Link notes to sources/artifacts through a join table to preserve provenance.

M5-013 adds a bounded local ResearchNote linked to one IntelligenceItem ID and a typed reference snapshot of source ID, source/original URLs, publication/observation times and verification state. The note does not copy the item body. Version-1 research export contains the note plus that attribution snapshot; an `itemAvailable` flag reflects retention without deleting the note/reference. Collections and general Notes CRUD remain future M10 work.

M5-018 adds schema v15 `intelligence_source_deletions`, a bounded last-1,000-entry audit table containing only source ID, display name, feed URL, deletion time and count of removed items. Explicit source deletion transactionally removes its connector declaration and retained Intelligence items, while Research Desk notes and their independent source-attribution snapshots survive. Export is a user-selected version-1 JSON file containing the declaration and up to the full 10,000-item retention bound; the native Save dialog writes with create-new semantics and refuses overwrite. Watchlists are independent filters and remain. No source content is copied into the deletion audit.

M5-014 stores `app.notifications.v1` as strict non-secret app metadata with its own schema version and an `enabled` flag that defaults to `false`. It contains no source content or permission tokens; system permission remains owned by the operating system.

M5-015 accepts Atom alongside RSS 2.0 in the existing bounded source adapter. Atom entries normalize to the same `IntelligenceItem` fields; published time is parsed separately from observation time, and `updated` is only a fallback when the entry has no publication date. Source-specific UUIDv5 item IDs remain stable across repeated fetches and include source ID/URL plus provider GUID or canonical item URL. NWS alert-feed items use category `weather_alert`. The selected-area NWS preset persists as an ordinary disabled/unreviewed RSS-kind source with bounded provider limits (10 seconds, 2 MiB, 500 items); no separate alert database is introduced.

## 3. Relationship summary

- Workspace 1:N Projects, Tasks, SavedViews, Notes, Settings (app-wide settings may be separate).
- Project 1:N Milestones and Tasks.
- Task 1:N Subtasks (via parent_task_id) and N:N dependencies/labels/artifacts.
- Task/Project/ToolRun/WorkflowRun can have ActivityEvents.
- ToolDefinition 1:N ToolRuns.
- ApprovalRequest binds to a single proposed action; the execution revalidates it.
- Connector 1:N IntelligenceItems; IntelligenceItems can be linked to research notes, tasks or evidence.
- WorkflowDefinition 1:N WorkflowRuns.

## 4. SQLite rules

- Use parameterized statements only.
- Enable foreign keys.
- Use transactions for multi-record state changes and migrations.
- Keep database access behind a repository/service abstraction where that reduces coupling; do not create generic ORM layers without need.
- Add indexes for workspace/project/task status/priority/due dates, activity timestamps, source+published dates and stable IDs.
- Use FTS5 only when available and useful; provide a normal-search fallback or state the supported SQLite build.
- Cap JSON input/output size and validate schema before writing.
- Use restrictive file permissions as provided by the platform and store under per-user app data.
- Handle busy/locked/corrupt DB errors visibly; never silently reset/delete user data.
- Backup before risky migrations and test restore on copies.

## 5. Import/export and backup

- Export selected project/task/notes/config using a versioned archive manifest.
- Import is a preview step; validate schema, record count, size, paths and version.
- Reject path traversal, absolute extraction paths and duplicate-ID overwrite unless the user chooses a supported conflict strategy.
- Never export credentials or secret material by default.
- Include schema version, app version, created timestamp and content hashes where appropriate.
- Test a round trip (export → fresh DB import → compare records).

## 6. MVP minimum schema

MVP should begin with Workspace, Project, Task, TaskDependency (if needed for UI), TaskLabel/Label, ActivityEvent, AppSetting, ToolDefinition and ToolRun. Do not implement all intelligence/automation entities before their features are scheduled. Add entity tables in the same milestone that introduces and tests the corresponding feature.

## 7. Data acceptance checklist

- [ ] Empty database initializes to the correct schema version.
- [ ] Migrations are transactional and tested from the previous version.
- [ ] Foreign-key violations fail tests.
- [ ] Task create/update/archive survives restart.
- [ ] IDs are stable and independent of display order.
- [ ] Time display covers UTC/local time and daylight-saving transitions where relevant.
- [ ] Import/export rejects unsafe paths and malformed/oversized input.
- [ ] Activity and tool results redact secrets.
- [ ] Export omits secrets by default.
- [ ] Restore is tested on a disposable copy before release.
