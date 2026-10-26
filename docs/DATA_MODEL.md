# Data Model & Persistence Blueprint

**Status:** Proposed logical model. Exact SQL migrations are created incrementally in the implementation phase.

## 1. Identity and time conventions

- Use stable UUIDs (or a documented UUID-compatible ID format) for entities; never use list position as identity.
- Store timestamps in UTC ISO-8601 or integer Unix milliseconds consistently; prefer ISO-8601 UTC at JSON/API boundaries.
- Render timestamps in the user's locale and timezone.
- Store created/updated timestamps on mutable records.
- Store soft deletion/archive only where the restore/audit use case justifies it; define retention.
- Define database schema version and execute migrations transactionally.
- SQLite foreign keys must be enabled for every connection.

## 2. Core entities

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
Project key is user-friendly metadata, not the primary ID. Avoid forcing a repository path or Git provider. The M2-002 schema-v3 migration creates a workspace-scoped `projects` table with `project_key` optional and unique when supplied within a workspace; `name` cannot be blank, `status` defaults to `planned`, and workspace references are foreign-key enforced. The M2-007 native service supplies a random UUID v4, validates fields and active workspace membership, and creates the project transactionally. M2-014 adds scoped listing, typed native list/create/update commands, and the Projects list/form for the seeded Personal workspace. The list excludes archived records and sorts by case-insensitive name then stable ID; every command opens SQLite under Tauri's per-user app data directory and reuses the validated services. Project UI fields are name, goal, description, and (on edit) status. Other workspace selection and archive controls remain future work.
Statuses: planned, active, paused, completed, archived.
M2-008 adds transactional scoped update/soft-archive and trusted change events. No-op updates leave timestamps and activity unchanged. Creating a project directly in archived state is rejected; existing linked tasks survive archive.

### Milestone
Fields: id, project_id, title, description?, due_date?, status, sort_order, created_at, updated_at.
Milestone can group tasks by release/outcome. Do not make milestone date a hidden dependency for completing a task.

### Task
Fields: id, workspace_id, project_id?, parent_task_id?, title, description?, status, priority, due_at?, estimate_minutes?, completed_at?, created_at, updated_at, archived_at?.
Canonical statuses: backlog, todo, in_progress, blocked, in_review, done, cancelled.
Canonical priorities: p0, p1, p2, p3. Priority is not security severity or execution risk.
M2-012 initial policy: any explicit transition between the seven canonical statuses is permitted, including reopening a completed/cancelled task; unchanged status is a no-op, unsupported values are rejected. All UI mutations must use the task service rather than a view-specific state machine. M2-010 adds a workspace-scoped transactional Task update service; UI and native IPC integration remain separate tasks.
Task must be creatable with a title alone and defaults. Project is optional for a personal inbox task. The implemented M2-003 schema-v4 migration adds checked `backlog`/`p2` defaults, nonblank task titles, optional project/parent links, and workspace-scoped composite foreign keys. UTC timestamps use the existing SQLite convention. The M2-009 create service validates fields and generates UUID v4 IDs. M2-016 adds a typed, camelCase Tauri create command scoped to Personal and a persisted React dialog. The dialog explicitly supplies `todo`/`p2` as its UX defaults, allows an active Project or no Project, and confirms only the returned saved record. Listing and further Task editing remain future tasks.

M2-013 `due_at` contract: the JSON field is `dueAt` and carries either `null`/omission or an RFC 3339 timestamp with an explicit `Z` or numeric UTC offset. Date-only and timezone-free local strings are invalid. Create/update normalize accepted instants to canonical UTC RFC 3339 with `Z`, preserving fractional-second precision, and SQLite stores that same value. Blank optional input clears the due time. UI adapters render the persisted instant in the user's current local timezone; domain services never guess a timezone or store locale-formatted text.

### TaskDependency
Fields: task_id, depends_on_task_id, dependency_type, created_at.
MVP dependency type: blocks/blocked_by via one canonical directed relation. Enforce no self-dependency and prevent cycles if dependencies affect scheduling. Keep dependency semantics documented.

### TaskLabel / Label
Use a join table for many-to-many task labels. Define normalized label identity within workspace. Avoid storing arrays in SQL if filtering and constraints are required.

### Comment
Fields: id, target_type, target_id, body, created_at, updated_at.
MVP can scope to task/project comments. If polymorphic foreign keys reduce integrity, use separate task_comments/project_comments tables instead.

### ActivityEvent
Fields: id, workspace_id, actor_type, actor_id?, entity_type, entity_id, action, occurred_at, correlation_id?, summary, metadata_json?, sensitivity.
Keep audit/activity distinct from verbose debug logs. Sanitize metadata and never log secret values.

Implemented subset (M2-006, migration v5): `activity_events` stores workspace-scoped entity type/ID, one project or task foreign key, `created` action, UTC timestamp, trusted fixed summary, actor type and sensitivity. Project/task create services append the event in the same transaction as the entity write; events are not fabricated for preexisting records. The additional optional actor ID/correlation ID/metadata fields described above are not persisted by this MVP subset and must be explicitly migrated before future usage. Never place user-supplied descriptions, credentials or arbitrary metadata into event summaries.

### Artifact / Reference
Fields: id, workspace_id, project_id?, task_id?, artifact_type, title, uri_or_relative_path, media_type?, content_hash?, source_id?, created_at, sensitivity, metadata_json?.
For external references save canonical URL/provider ID. For local artifacts validate path against app-owned storage. Avoid arbitrary file embedding by default.

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

### IntelligenceItem
Fields: id, source_id, external_id?, canonical_url, title, summary?, published_at?, observed_at, event_time?, category, content_hash?, normalized_metadata_json?, freshness_state, verification_state.
Preserve source attribution and collection time. Keep the raw payload only when the source terms and privacy policy permit.
Deduplication must not erase source relationships.

### Signal / TrendObservation
Fields: id, subject_key, category, observed_window_start, observed_window_end, count, baseline_count?, delta_ratio?, method_version, source_count, confidence_label, created_at.
A trend requires a named time window and baseline. Avoid calling an item “trending” based only on a global total.

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
