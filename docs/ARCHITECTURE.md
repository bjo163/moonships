# Technical Architecture

**Status:** Proposed architecture baseline. Revise through documented decision records.

## 1. Technology decisions

| Concern | Decision for initial implementation | Reason / guardrail |
|---|---|---|
| Desktop shell | Tauri 2 | Native desktop lifecycle, tray and Rust integration |
| Native engine | Rust stable | Memory safety, explicit native/system boundaries |
| UI | React + TypeScript + Vite | Productive desktop UI ecosystem |
| Styling | Tailwind CSS + semantic design tokens | Consistent UI without coupling visual design to business logic |
| Persistence | SQLite | Local-first persistence, migrations, easy backup/export |
| Serialization | Serde with explicit DTOs | Stable data contracts between layers |
| Async work | Rust async or bounded worker threads chosen per workload | Avoid blocking UI/IPC; bound concurrency |
| Tests | Rust unit/integration tests, frontend tests, contract tests, end-to-end smoke tests | Test policy and failure states, not just visuals |
| CI | GitHub Actions | Repeatable formatting, lint, tests, Windows build |
| Telemetry | Local structured logs first; optional OpenTelemetry adapter later | No default data exfiltration |
| AI | Optional provider abstraction, added after tool contracts are stable | No model lock-in and no AI requirement |
| MCP | Optional version-pinned integration | Interoperability without coupling core to evolving protocol |

Use current official docs at implementation time; pin dependency versions and review breaking changes. Tauri capabilities constrain which commands/plugins are exposed to each webview, but capability configuration does not replace validation in native code. Record architecture changes using the [ADR template](ADR_TEMPLATE.md) and update the decision register when relevant.

## 2. Runtime layers

1. **Experience layer:** React screens, command palette, tray-facing UI, status and progress.
2. **Tauri boundary:** small command handlers, typed inputs/outputs, event transport, capabilities.
3. **Application services:** projects/tasks, tool registry, activity/history, settings, approvals.
4. **Domain modules:** clean, devtools, netops, sysops, intelligence, security, etc.
5. **Policy layer:** validation, permission checks, scope checks, risk classification and approval state.
6. **Platform adapters:** Windows filesystem, process/service inventory, network and credential storage.
7. **Persistence:** SQLite, cache, structured logs, local artifacts.
8. **External adapters:** provider APIs, CLI programs, source feeds, model APIs and MCP.

Dependencies flow inward toward contracts/core. Domain modules do not import UI components; UI does not manipulate the filesystem directly. A domain module must not call another module's private implementation; use declared service interfaces/events.

## 3. Monolith and module boundaries

Start with one desktop process and a modular Rust codebase. A module may be a folder/crate depending on size; a separate process is not required just because the product has a named domain.

Extract a process or service only when there is a concrete reason, such as isolation of untrusted code, independent scaling, a separate lifecycle, or an established external protocol. Do not create microservices, a mandatory cloud API or a plugin marketplace for the first release.

Target internal layout (not all directories must exist on day one):

```text
apps/desktop/
  src/                         # React UI
  src-tauri/
    capabilities/
    src/
      main.rs
      lib.rs
      commands/
      core/
        ids.rs
        errors.rs
        events.rs
        tool_registry.rs
        tool_contract.rs
        activity.rs
      policy/
      modules/
        projects/
        tasks/
        cleaner/
        devtools/
        netops/
        sysops/
        intelligence/
        security/
        automation/
      platform/windows/
      storage/
      integrations/
crates/                        # Split stable reusable parts only when justified
schemas/
tests/
docs/
```

## Desktop process lifecycle

The desktop shell enforces a single-instance policy. When a second launch occurs, it must bring the existing `main` window to the foreground rather than create a confusing duplicate workspace. The tray's explicit Quit action exits the process; Hide only hides the window. If background workers are introduced, their shutdown hooks must run before explicit process exit.

## Local SQLite storage location

The desktop app must store its SQLite database beneath Tauri's platform-specific per-user application data directory, never relative to the process current working directory. The initial database filename is `moonships.sqlite3`. The native storage adapter creates the application data directory if needed and opens SQLite through the Rust `rusqlite` adapter. Every connection enables SQLite foreign-key enforcement and configures a bounded 5,000 ms busy timeout before it is returned to callers.

## Transactional schema migrations

The native SQLite initializer runs a versioned migration runner before the app finishes startup. The `schema_migrations` table records each applied schema version and UTC application timestamp. Each migration and its version record must commit in one SQLite transaction; an error rolls the transaction back. The runner must reject a database schema newer than the application supports and must never silently reset user data. Version 1 creates the initial `app_metadata` key/value table. Version 2 creates the `workspaces` table and inserts exactly one deterministic Personal workspace for new and upgraded installations. Version 3 creates the workspace-scoped `projects` table (foreign-key checked, canonical project status default `planned`, nonblank names and optional unique project key per workspace), without rewriting any existing workspace or app configuration. Version 4 adds `tasks` with nullable project/parent links, canonical status/priority defaults, and composite foreign keys enforcing workspace isolation. Version 5 adds `activity_events` with workspace-scoped project/task reference integrity and UTC timestamps; entity creation and its corresponding trusted, non-sensitive activity record commit atomically. Failed writes and mid-migration collisions are covered by disposable SQLite rollback tests. Disposable database tests exercise idempotency, upgrades from prior versions, and intentional mid-migration rollback. `projects` and `tasks` service modules supply transactional, validated UUID-based creates. M2-014 adds typed list/create/update Project IPC commands scoped to the seeded Personal workspace; they resolve the database only through Tauri's per-user application data directory, use named camelCase DTOs with unknown-field rejection, and call validated services rather than arbitrary SQL. The Projects React screen is connected to those commands. M2-016 adds typed Task-create IPC and a Task creation dialog, also fixed to the Personal workspace and using the validated Task service. M2-018 adds typed, read-only Task list IPC fixed to the Personal workspace and returns active records through the existing validated Task service. The Tasks route renders that returned data and applies stable, in-memory sorting; the list never writes task values. The dialog offers the Personal inbox or active Projects, defaults status/priority to Todo/P2, and only shows success after the persisted Task is returned. Task editing and other list mutations remain later work. This is a forward-only transactional migration; existing `app_metadata` rows remain unchanged and repeated startup does not duplicate the default. Add future schema changes as explicit numbered migrations.

## Tauri command DTO conventions

Every frontend-to-native command must use an explicit, named request/response contract once it crosses the Tauri IPC boundary. Do not expose internal domain structs directly as a long-lived IPC schema.

- Define request and response DTOs in Rust with Serde `Deserialize` / `Serialize`; use `#[serde(rename_all = "camelCase", deny_unknown_fields)]` for predictable JSON naming and rejection of unexpected request fields.
- Use a named command parameter such as `request: SomeRequestDto`, so the frontend payload is explicit: `invoke("command_name", { request: { ... } })`.
- Return `Result<ResponseDto, String>` while the stable application error-code mapping is being generalized. Do not convert failures into success-shaped defaults.
- Validate semantic constraints inside the native command even when deserialization succeeds. DTO deserialization checks shape and types; it does not replace authorization, range checks, or policy enforcement.
- Type the frontend result with the corresponding response shape; keep request and response fields aligned with the Rust DTO.
- Add contract tests for field names, unknown-field policy, failure behavior and the frontend invocation shape. Treat changes to published DTO fields as compatibility changes.

The hide-to-tray preference command is the first reference implementation: both read and write return `HideToTrayPreferenceDto { enabled: bool }`; writes accept it under the `request` argument.

## 4. Tool contract

A registered tool defines:
- Stable namespaced ID (e.g. `moonships.netops.dns_lookup`)
- Human-readable name, description, category and documentation
- Tool contract/API version and input/output schema version
- Supported platform and availability/health state
- Required permission capabilities
- Risk class: read-only, low-impact change, destructive, privileged, remote/high-impact
- Timeout, maximum input/output sizes and concurrency policy
- Whether cancellation is supported and how it behaves
- Audit/result metadata rules and data sensitivity
- Test fixture and contract-test reference

A tool call returns structured status, data, warnings, errors and provenance metadata. Do not treat a missing field, provider outage, parser failure or stale result as success.

## 5. Command and event flow

1. UI or workflow requests a registered tool by ID with typed arguments.
2. Application API validates schema, caller context, tool availability and limits.
3. Policy layer resolves capabilities, scope and required approval.
4. If approval is required, create a pending approval record and return a plan/preview.
5. After consent, revalidate policy and arguments; never trust the earlier preview blindly.
6. Run through a bounded executor with timeout/cancellation semantics.
7. Emit typed progress/state events and capture structured result.
8. Persist run metadata/result references; redact sensitive fields.
9. UI renders the result and may create/link a task, artifact or report.

UI-to-Rust IPC is not a security boundary by itself. Every command must validate data and enforce permissions inside the native process.

## 6. Long-running jobs

Use a job record with:
- job ID
- tool ID and version
- status (queued, running, waiting_approval, succeeded, partial, failed, cancelled, timed_out)
- created/started/finished timestamps
- progress stage and optional numeric progress
- cancellation capability/status
- result/error reference
- initiator and approval reference

Rules:
- Do not block the UI thread or a command until a large scan/ingestion completes.
- Limit per-tool and global concurrency.
- Treat cancellation as best effort unless the underlying operation guarantees cancellation.
- Persist enough state to recover from app restart; do not falsely mark an interrupted job as success.
- No automatic retry for destructive or non-idempotent actions without policy.

## 7. Error model

Use stable error codes/categories:
- INVALID_ARGUMENT
- UNSUPPORTED_PLATFORM
- PERMISSION_DENIED
- OUT_OF_SCOPE
- APPROVAL_REQUIRED
- NOT_CONFIGURED
- OFFLINE
- RATE_LIMITED
- TIMEOUT
- CANCELLED
- RESOURCE_BUSY
- NOT_FOUND
- CONFLICT
- PROVIDER_ERROR
- PARSE_ERROR
- POLICY_BLOCKED
- INTERNAL_ERROR

Human-facing messages should explain the next safe step. Technical details go to logs with secrets/PII redacted. Frontend logic must depend on error codes, not parsing English error strings.

## 8. Platform and filesystem adapters

- Keep Windows-specific APIs behind a platform adapter.
- Feature-detect unsupported APIs rather than assuming every Windows edition/build behaves identically.
- Resolve special directories using platform APIs/environment contracts, then validate canonical paths.
- Reject protected paths and path traversal; avoid following symlinks/junctions outside a permitted root.
- Avoid time-of-check/time-of-use gaps by revalidating the candidate immediately before mutation.
- Handle locked files, access denied, races and long paths without crashing.
- Core project management and offline DevTools must not depend on elevated privileges.

## 9. Persistence and transactions

- Store app-owned data under an OS-appropriate per-user application data directory.
- Apply schema migrations transactionally and record the schema version.
- Enable foreign-key enforcement and sensible busy timeout; use parameterized SQL.
- Use UTC timestamps for persisted events and render local time in UI.
- Use UUID/ULID-compatible stable IDs, not array indices.
- Keep large artifacts out of relational rows; store files separately with metadata/hash.
- Treat imported/exported data as untrusted; validate size, schema and paths.
- Backups must include a manifest/version and integrity check; test restore, not just backup creation.
- Do not write secrets into history, task descriptions, debug logs or model prompts.

## Local logging, job and tool contracts

Native local logs are newline-delimited JSON at the per-user application data path `logs/moonships.log`. Log events use stable levels and categories, and redact values for known secret-bearing field names recursively before serialization. Log only operational summaries and allowlisted metadata; do not put raw tool input, credentials, authorization headers or untrusted source text in event messages. Logging failure must not prevent the app from opening, and logs remain local by default.

The domain contract defines stable job states (`queued`, `running`, `waiting_approval`, `succeeded`, `partial`, `failed`, `cancelled`, `timed_out`), versioned tool definitions, structured tool outcomes and a native capability policy. Tool metadata is not executable code. The policy denies missing capabilities and returns an approval-required decision for state-changing, destructive, privileged or remote operations unless approval is explicitly supplied. The UI must not be the enforcement boundary.

Non-secret application configuration carries a schema version and validated enum values for theme, workspace profile and log level. Defaults are dark theme, Developer profile, Info logging and hide-to-tray disabled. Unknown config fields and unsupported schema versions fail closed; credentials belong in secure storage, not this config object.

M1-030 native implementation: `AppConfig` is a fixed typed JSON object stored under key `app.config.v1` in the existing per-user SQLite `app_metadata` table (database schema version and config schema version are distinct). `get_app_config` and `set_app_config` use the documented named DTO pattern; no arbitrary configuration key, secret field or file path may be supplied. Reads return the defaults only when the row is absent, and reject corrupt/unknown/future-version rows without overwriting them. Updates validate before one atomic SQLite UPSERT so failed validation preserves the previous record. The existing `get_hide_to_tray` / `set_hide_to_tray` commands now use the same native record. A legacy `hide-to-tray` file is read only as a fallback if no versioned record exists; it is never deleted or overwritten. Existing WebView-local theme/profile preferences remain under their current adapters until a separately reviewed UI migration—do not claim they are already synchronized with the native record.

## Module availability semantics

Module availability is distinct from operation results. `available`, `disabled`, `unsupported`, `not_configured` and `degraded` communicate readiness; none of these means a successful query returned zero results. The UI must explain setup/health state without fabricating operational data.

## 10. External source and provider adapters

Each adapter must expose:
- provider/source ID and adapter version
- required user configuration and permission
- data access method and terms/rate-limit notes
- refresh cadence, timeout and last-success timestamp
- freshness/stale policy
- normalized records and source attribution
- errors, circuit-breaker/backoff behavior and health status
- retention/redaction settings

Never assume every website allows automated fetching. Prefer public documented APIs, official status feeds, RSS where permitted and user-authorized exports. Do not bypass auth, paywalls, access controls or rate limits.

## 11. AI adapter boundary

AI consumes a limited context assembled by Moonships and can request only registered tools. Keep provider request/response types behind an internal interface. Persist prompt/output only when configured and necessary; redact secrets and minimize sensitive data.

Model output is untrusted input:
- parse against schema
- validate tool ID/arguments
- independently enforce policy
- require explicit approval for consequential operations
- never allow model output to redefine permissions
- provide source references for claims derived from retrieved documents

When AI provider is unavailable, disable only AI-dependent workflows, not core local tools.

## 12. Performance and resilience

- Lazy-load large UI feature areas where it improves startup.
- Bound directory walks, network calls, ingestion batches and result sizes.
- Virtualize large tables.
- Use cancellation and backpressure for long-running tasks.
- Debounce high-frequency UI searches.
- Never perform heavy scanning automatically at app startup.
- Show source freshness and last successful run; do not spin forever on a failing provider.
- Test idle resource use and behavior while hidden in the tray.

## 13. Architectural fitness tests

- No frontend command allows arbitrary deletion or arbitrary shell execution.
- Every registered mutating tool has a risk classification and policy tests.
- A failed/offline provider cannot be represented as healthy/live.
- A tool contract change fails contract tests until UI/automation adapters are reconciled.
- DB migrations work from empty DB and from the previous schema version.
- App restart cannot silently resume a destructive action.
- Optional AI/MCP integrations can be disabled without breaking core.


### Command Palette surface (M1-031)

The Command Palette is a keyboard-first presentation surface over an explicitly supplied list of registered command descriptors. It performs deterministic local substring filtering across title, description, category and keywords; supports Arrow Up/Down and Enter while the search field is focused; and exposes accessible dialog/listbox semantics, category labels, risk labels, loading and empty states. The current shell intentionally supplies an empty command list until the typed registry and safe action wiring task is implemented. Selecting a result only calls the UI selection callback; the palette does not execute tools or grant permissions. Destructive, privileged and remote risk classes are visibly labelled for future preview/approval flows. Ctrl+K lifecycle and focus restoration are tracked separately in M1-033.


### Registered Command Palette actions (M1-032–M1-033)

`src/command-registry.ts` is the allowlist for palette entries. The UI receives descriptors projected from this registry and resolves selection by exact registered ID; unknown IDs fail closed. Navigation to the existing overview and opening Settings are direct UI actions. New Project and New Task open their local persistence forms; New Note remains an explicit preview because note storage is not implemented. Risky tools are not registered as executable palette commands; future execution must use the native policy/approval gate.

Ctrl+K (or Cmd+K on macOS) is ignored when focus is in an editable control, a dialog is already open, or the palette itself is open. Escape closes the palette, and the previously focused element is restored after closing.
