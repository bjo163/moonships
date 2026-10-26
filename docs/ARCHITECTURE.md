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

The native SQLite initializer runs a versioned migration runner before the app finishes startup. The `schema_migrations` table records each applied schema version and UTC application timestamp. Each migration and its version record must commit in one SQLite transaction; an error rolls the transaction back. The runner must reject a database schema newer than the application supports and must never silently reset user data. Version 1 creates the initial `app_metadata` key/value table. Version 2 creates the `workspaces` table and inserts exactly one deterministic Personal workspace for new and upgraded installations. Version 3 creates the workspace-scoped `projects` table (foreign-key checked, canonical project status default `planned`, nonblank names and optional unique project key per workspace), without rewriting any existing workspace or app configuration. Version 4 adds `tasks` with nullable project/parent links, canonical status/priority defaults, and composite foreign keys enforcing workspace isolation. Version 5 adds `activity_events` with workspace-scoped project/task reference integrity and UTC timestamps; entity creation and its corresponding trusted, non-sensitive activity record commit atomically. Versions 6–8 add labels, dependencies and milestones transactionally. Version 9 adds workspace-scoped cleanup report summaries with an optional composite Task link; it stores no candidate paths or IDs and enforces consistent counts and byte totals. Failed writes and mid-migration collisions are covered by disposable SQLite rollback tests. Disposable database tests exercise idempotency, upgrades from prior versions, and intentional mid-migration rollback. `projects` and `tasks` service modules supply transactional, validated UUID-based creates. M2-014 adds typed list/create/update Project IPC commands scoped to the seeded Personal workspace; they resolve the database only through Tauri's per-user application data directory, use named camelCase DTOs with unknown-field rejection, and call validated services rather than arbitrary SQL. The Projects React screen is connected to those commands. M2-015 adds typed project-overview IPC for the same fixed workspace: it reads one active Project, counts its non-archived Tasks, and returns at most ten newest trusted activity summaries for that Project and its currently linked Tasks. It sends no arbitrary SQL, path, or workspace scope from the frontend. The Project overview route displays saved Project fields, task count and recent activity, with explicit zero-task, empty-activity, loading, retry and unavailable-project states. Creating a Task from the overview preselects that Project. M2-016 adds typed Task-create IPC and a Task creation dialog, also fixed to the Personal workspace and using the validated Task service. M2-018 adds typed, read-only Task list IPC fixed to the Personal workspace and returns active records through the existing validated Task service. M2-017 adds typed, scoped `get_task` and `update_task` IPC backed by the existing validated Task services; the detail route edits title, description, status, priority and due time. The UI preserves the saved Project association, renders date/time in local time and sends an RFC 3339 instant for UTC persistence. Loading, missing/archived Task, save error and persisted success are distinct. M2-032 adds a file-backed repository smoke test over real migrations and Project/Task services: it creates a Project and linked Task, updates Task status, closes the SQLite connection, reopens the same disposable app-data database, and verifies the persisted Project and updated Task. This proves native persistence/reopen behavior; it does not replace a manually observed desktop UI restart. The dialog offers the Personal inbox or active Projects, defaults status/priority to Todo/P2, and only shows success after the persisted Task is returned. Task mutations from the list remain later work. This is a forward-only transactional migration; existing `app_metadata` rows remain unchanged and repeated startup does not duplicate the default. Add future schema changes as explicit numbered migrations.

M2-004 migration v6 adds `labels` with a workspace-scoped normalized-name uniqueness constraint and `task_labels` with composite workspace foreign keys to both Tasks and Labels. The native labels service collapses surrounding/internal whitespace, applies Unicode lowercase for duplicate identity, and retains the first trimmed/collapsed display spelling. Repeating the same Task-label link is idempotent. The service does not expose arbitrary paths or SQL; UI/IPC integration and label deletion remain future work.

M2-005 migration v7 adds a directed `task_dependencies` relation with composite workspace foreign keys to both Tasks, a database/service self-dependency guard, and one canonical `blocked_by` row per edge. The reverse `blocks` relation is derived. Dependency edges are not used for scheduling yet; cycle rejection is required before a future scheduler consumes them.

M5-008 and M5-009 add forward-only migrations v11 and v12 for locally persisted connector declarations and topic watchlists, respectively. Both tables retain bounded JSON contracts alongside database-managed creation/update timestamps.

M2-024 migration v8 adds `milestones` and `milestone_tasks`. Composite workspace/Project foreign keys scope each Milestone and association to the same Project; an active-task service check also prevents associating archived Tasks. Typed list/create/update/associate commands accept only a Project ID or named Milestone/task fields and derive the Personal workspace natively. Due dates must be valid calendar dates in `YYYY-MM-DD`; Milestone status is metadata (`planned`, `active`, `completed`). These operations do not change Task status.

M2-025 adds the typed `list_task_activity` command using only a Task ID request; the native service derives Personal workspace scope, verifies that the Task is active, and returns at most 50 newest trusted summaries. The Task update service records `status_changed`, `priority_changed`, and generic `updated` events according to the fields that actually changed, in the same transaction as the Task update. No-op updates create no event; activity summaries contain no Task content or credentials. Task detail presents a retryable history read independently from the saved form.

M2-026 defines artifact references as metadata-only typed targets: validated HTTP(S) URL, namespaced opaque provider ID, or opaque app-managed artifact ID. Native code rejects arbitrary renderer paths and unsafe schemes; an app-managed ID resolves only below app-owned storage after canonical containment checks. Saving a reference never fetches, downloads, opens or executes it. Opening later requires an explicit action through a narrow native adapter that revalidates the target; provider navigation needs a separately reviewed adapter.

## Tauri command DTO conventions

M2-029 adds `export_project_task_data`, a no-path-request native command. It reads the Personal workspace's complete Project/Task records and normal-sensitivity Activity through the application-data database, constructs strict export schema v1 with the Cargo app version and a UTC timestamp, validates and round-trips JSON, then asks the user for a save location through a native Windows dialog. The frontend receives only cancellation/success counts and the selected file's name; it cannot select an arbitrary path or write file data. Errors and cancellation remain distinct from success. Settings, metadata, logs, credentials and restricted activity are not exported.

M2-033 adds `get_home_dashboard`, a no-request, read-only native command fixed to the Personal workspace. SQL-derived counts use a single UTC `asOf` value; returned lists are bounded to five Projects, five overdue Tasks, five upcoming Tasks and ten normal-sensitivity Activity events. The renderer cannot supply a workspace, time, query or SQL, and no polling or write path is introduced.

Every frontend-to-native command must use an explicit, named request/response contract once it crosses the Tauri IPC boundary. Do not expose internal domain structs directly as a long-lived IPC schema.

- Define request and response DTOs in Rust with Serde `Deserialize` / `Serialize`; use `#[serde(rename_all = "camelCase", deny_unknown_fields)]` for predictable JSON naming and rejection of unexpected request fields.
- Use a named command parameter such as `request: SomeRequestDto`, so the frontend payload is explicit: `invoke("command_name", { request: { ... } })`.
- Return `Result<ResponseDto, String>` while the stable application error-code mapping is being generalized. Do not convert failures into success-shaped defaults.
- Validate semantic constraints inside the native command even when deserialization succeeds. DTO deserialization checks shape and types; it does not replace authorization, range checks, or policy enforcement.
- Type the frontend result with the corresponding response shape; keep request and response fields aligned with the Rust DTO.
- Add contract tests for field names, unknown-field policy, failure behavior and the frontend invocation shape. Treat changes to published DTO fields as compatibility changes.

The hide-to-tray preference command is the first reference implementation: both read and write return `HideToTrayPreferenceDto { enabled: bool }`; writes accept it under the `request` argument.

M2-023 applies this boundary to Task relationships: `CreateTaskRequestDto` may include an optional `parentTaskId`, and `TaskDto` returns the persisted parent ID. The command continues to derive the workspace from the fixed Personal workspace rather than accepting renderer scope. The transactional native Task service checks that parent and child share workspace and nullable Project association, and that the parent is active and top-level; relationship integrity is never entrusted to UI-only checks.

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

M4-001 defines the bounded native network-diagnostic contract for one DNS lookup or one TCP host/port test. Targets are required bare ASCII hostnames (IDNs use punycode) or IP literals; URLs, host-embedded ports, wildcards and ranges are rejected. Timeout must be 100–10,000 ms (2,000 ms default); adapters enforce one concurrent run, at most ten runs per minute and at most 32 returned IP addresses. Results identify the requested target/port, operation, UTC observation time as Unix epoch milliseconds, duration, OS resolver/socket source, outcome, stable error code and bounded resolved addresses. M4-002 adds an internal read-only DNS adapter: hostnames use the OS resolver in one bounded worker; requests return within their configured timeout, distinguish validation failure from resolver failure, and never trigger automatic retries. M4-003 adds a single-host/port TCP check to that same worker, so DNS resolution and socket connection share the concurrency, queue, deadline and rate bounds. Success, timeout, connection refusal, and resolver failure have distinct results. M4-004 detects native ICMP capability by opening and closing the Windows IP Helper ICMP handle; this sends no packet and launches no executable. Other platforms return an explicit unsupported capability. M4-005 is explicitly deferred: no traceroute process is launched until a reviewed adapter can enforce hop and duration bounds. M4-006 adds a pure offline IPv4 CIDR calculator; it returns network, masks, broadcast/host range and counts, with explicit `/31` and `/32` semantics, and performs no network access. OS name-resolution calls cannot be cancelled once started, so a stuck resolver can occupy the single worker; no unbounded replacement workers are created. IP literals are returned directly with `IpLiteral` provenance for DNS lookups and do not initiate DNS. No IPC command was registered by the adapter tasks before M4-014.

M4-007 adds a native read-only system inventory DTO. It reports the compiled OS family and process architecture, the logical processor count available to the process, physical RAM totals, and capacity/free bytes for the volume containing the Windows directory. Each field is represented as an optional value with an explicit available, unsupported or unavailable status; no fallback value is fabricated. Non-Windows builds mark Windows-only memory and storage metrics unsupported. The adapter does not return the Windows path or enumerate files/devices/processes, and is not registered as an IPC command.

M4-008 adds a no-argument `list_process_inventory` command backed by a Windows Tool Help process snapshot. The adapter reports only executable name, process ID, parent ID and thread count, caps output at 4,096 records, closes its snapshot handle, and returns explicit unavailable/permission-denied/partial states. It does not open process handles, read command lines/paths or expose mutation commands. The Processes page reads only on mount or an explicit refresh; there is no background polling.

M4-009 adds a no-argument `list_service_inventory` command backed by the local Windows Service Control Manager. It requests only `SC_MANAGER_ENUMERATE_SERVICE`, reads service/display names and current state into a 2,048-record-bounded result, closes the manager handle, and distinguishes permission denial, unavailability and partial results. It never opens individual services or exposes start/stop/configuration commands. The Services page reads once on mount and refreshes only on explicit user action.

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

M3-006 defines cleaner scan updates as serialized progress and terminal events keyed by the Activity Center job UUID. Progress reports stage and observed counts, plus whether cancellation is best-effort or unsupported; it does not invent a percentage when the total is unknown. A terminal event carries one terminal outcome (`succeeded`, `partial`, `failed`, `cancelled` or `timed_out`), final counts and a closed error code. A `cancelled` outcome is emitted only after work actually stops; a cancellation request alone is not completion. This is a DTO contract, not a wired UI or running scan executor.

M4-012 adds schema v10 job history and an Activity Center route. Job status updates enforce allowed transitions, reject invalid progress, record first-start and terminal timestamps, and persist only a bounded title/stage/result summary. The page reads the latest 200 saved records and shows truthful empty/loading/error states. It does not create synthetic jobs or poll in the background. Existing job producers are not yet connected, and interrupted `running` records remain visibly running until a producer reports a terminal result; automatic recovery and cancellation are separate work.

M4-013 keeps Activity Center cancellation disabled and explains why: no connected job runner currently exposes a cancellation handle. A future runner must report its capability and confirm that execution actually stopped before a job may transition to `cancelled`; a request alone is not a terminal outcome.

M4-014 registers a typed one-target DNS/TCP diagnostic command and caches at most 100 result handles in memory. Task creation accepts only a recent result handle, saves to the Personal inbox, and builds a fixed summary from allowlisted target, operation, outcome, timestamp, duration, source and error-code fields. Resolved addresses and raw system messages are omitted. Handles expire on process restart or cache eviction.

M4-015 covers invalid DNS input before resolver work, an already-expired TCP deadline using the loopback literal, TCP success/refusal-or-timeout on loopback, error-code mapping and rate-window limits using injected monotonic times. No test sends DNS or TCP traffic to an arbitrary public host.

M3-007 defines a preview DTO with candidate ID, category, leaf display name, size, modification time, eligible/skipped disposition and a closed skip reason. It excludes absolute paths. The Clean & Storage route renders only the supplied preview, derives counts and estimated bytes from its rows, and has no mutation action. When no scan result exists it explains that scanning is unavailable; it does not fabricate candidates.

M3-008 adds an explicit review dialog for selected eligible rows. Its approval request is bound to the exact preview UUID, unique candidate UUIDs and their estimated byte total. Native request construction rejects empty, duplicate or skipped candidate selections. Approval only records the user's exact selection; it does not start deletion. Revalidation and the bounded executor are separate required gates.

M3-009 captures a platform file identity with the candidate: Windows volume serial plus file index, or Unix device plus inode. Before proceeding, revalidation rejects missing identities, path changes, links/non-files, changed identity or metadata, protected/outside paths and candidates no longer eligible under the current rule. On unsupported platforms it fails closed. The verified path is for immediate use only; a later executor must revalidate immediately before mutation and must not cache it.

M3-010 adds a native-only bounded executor for an exact approved preview and candidate selection (maximum 100 items). It rejects approval/preview total mismatches before mutation, revalidates each item independently and records deleted/skipped/failed outcomes so one item cannot abort the batch. Windows deletion uses a no-follow file handle, rechecks the handle's resolved path, identity and metadata against the validated snapshot, then marks only that handle for deletion; locked or denied items are reported individually. Deletion fails closed on platforms without the handle-safe adapter. Do not expose this executor directly to the renderer; only the native approval flow may call it. Per-item estimated bytes are retained for the follow-up report task; this executor does not claim those bytes were actually freed.

M3-011 keeps the approved estimate separate from `actual_deleted_bytes`, which sums only logical file sizes whose deletion completed and whose path disappeared after the handle closed. Failed or skipped candidates contribute zero. This value describes removed file contents by logical size; it is not a measurement of physical disk space reclaimed, which varies with compression, sparse files and hard links.

M3-013's default rule registry contains only app-owned scratch files older than seven days (64 MiB per item) and regenerable app cache older than thirty days (128 MiB per item). The native app-data roots are `temporary/` and `cache/`; the shared OS user-temp directory has no default rule. Security exclusions and the ownership contract are listed in `SECURITY_MODEL.md`. These rules are definitions only; they do not make scanning or deletion available in the UI.

M3-014 stores only cleanup report summaries in SQLite and exposes a bounded, Personal-workspace read command for the Clean & Storage history. Reports retain outcome counts, estimate, successfully deleted logical file size, completion time and an optional Task ID; the database contains neither candidate paths nor per-file IDs. The native-only `execute_approved_cleanup_and_save` wrapper runs the exact approved executor operation and persists its completed summary; it is not exposed through renderer IPC. The UI loads recent reports with explicit loading, error/retry and empty states and links to a Task only after the user selects the link. The screen still does not start scanning or deletion.

M3-015 adds an in-process JSON formatter/validator with a 1 MiB UTF-8 input limit, 128-level nesting bound and 4 million character formatted-output cap. The built-in JSON parser validates input; formatting changes whitespace only, preserving number and string tokens. It makes no provider, network or filesystem calls and reports empty, invalid, oversized and overly nested input distinctly.

M3-016 uses the explicitly selected, exact-pinned `yaml@2.9.1` parser/stringifier from eemeli/yaml under ISC license (see [third-party notices](THIRD_PARTY_NOTICES.md)). Parsing is YAML 1.2/core, strict, duplicate-key checked and uses `parseDocument` so syntax errors return structured parser codes and positions rather than throwing through the UI. Input is limited to 1 MiB; formatted output is limited to 4 million characters. YAML text is handled locally and never converted to arbitrary application objects or sent to a provider.

M3-017 adds local Base64 conversion for UTF-8 text and explicit hexadecimal byte input, plus strict padded-standard-Base64 decoding. Inputs/decoded bytes are capped at 1 MiB; decoding uses fatal UTF-8 validation, so non-text data is labelled binary and shown as a 64-byte-limited hex preview. Encoding and decoding stay in memory and make no filesystem, network or provider calls.

M3-018 distinguishes standalone URL components from absolute full URLs. Component operations use `encodeURIComponent`/`decodeURIComponent`; full URL operations use the browser URL parser and `decodeURI`, preserving reserved URL delimiters while validating absolute URL structure. UTF-8 input and displayed output are bounded, and malformed percent escapes/URLs return stable local errors.

M3-019 adds a local UUID v4 generator using the browser's Web Crypto `randomUUID()` API, which is specified to use cryptographically secure randomness. It fails closed if the API is unavailable, throws, or returns a malformed value; there is no weak-randomness fallback. The validator accepts canonical UUID versions 1–8 using the RFC variant, plus the Nil UUID. Both operations run in memory without network, provider or filesystem access.

M3-020 provides local SHA-256 text and file hashing with the exact-pinned RustCrypto `sha2` crate. Text is capped at 1 MiB; file selection happens in a native Windows picker and the native reader streams 64 KiB chunks up to 2 GiB, so memory use does not grow with file size. Only the selected leaf filename, byte count and digest return to the UI. Cancellation returns no digest; permission/read errors are reported without disclosing the full path. File hashing is currently unsupported outside Windows, matching the available native picker.

M3-021 compares two local text inputs using exact-pinned `diff@9.0.0` (BSD-3-Clause). Each side is limited to 1 MiB UTF-8 and 50,000 lines; the Myers diff edit distance is capped at 1,000 and the unified result at 4 million characters. Exceeding a bound returns a clear error rather than a partial diff. Text stays in memory and is not sent to a provider or network.

M3-022 converts whole-number Unix timestamps in seconds or milliseconds and strict ISO 8601 date-times with an explicit known timezone offset. It reports normalized UTC ISO, Unix values in both units and a separate device-local display labelled with the resolved timezone. Invalid numbers, unsupported date ranges, malformed calendar dates, missing offsets and unknown `-00:00` offsets return clear errors instead of being silently normalized.

M3-023 exercises all implemented DevTools in one offline acceptance test with browser `fetch` replaced by a throwing stub. JSON, YAML, Base64, URL, UUID, SHA-256 text/file bridge, diff and timestamp flows must still finish, and the test asserts no `fetch` call occurs. This verifies the frontend tools do not depend on HTTP; native SHA-256 operations use Tauri IPC and local filesystem selection only.

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

### Cleaner rule contract (M3-001)

The native cleaner contract is declarative: a rule has a stable ID/version, category, closed root identifier, minimum age and per-item byte ceiling. Roots are app-managed temporary storage, app cache, or the current user's OS-resolved temporary directory; no rule or renderer supplies an absolute path. The native platform adapter must resolve the root and later verify containment before production scanning. Eligibility is a pure function over candidate metadata and a caller-supplied observation time. It accepts only regular files in the rule's bound root that meet the age and size limits; invalid rules, links/directories, future timestamps, mismatched roots, recent files and oversized files are excluded. It performs no filesystem I/O and has no deletion operation. M3-002 adds a test-only read-only traversal helper that collects regular-file metadata and skips links; its disposable fixture test verifies nested fixture bytes are unchanged. It is compiled only in tests and is not exposed through IPC. M3-003 adds a native path validator that canonicalizes an existing allowed root, candidate and native-policy protected roots; it rejects the root itself, any candidate outside the allowed root, protected descendants and any unresolved path. M3-004's Windows junction fixture proves the validator rejects an outside-root target and traversal does not recurse into the junction. M3-005 defines a candidate identity as a per-scan UUID, validated relative locator beneath the rule root and immutable observed metadata (root, kind, modification time and size); absolute paths and display labels are not identity. Later revalidation must compare fresh metadata and platform identity before mutation.

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

M4-010 adds a typed event-log provider seam for fixed Application/System/Security channels, at most 100 recent event summaries, and a 100–5,000 ms requested query deadline (2,000 ms default). A future provider receives both bounds and must enforce them. Results distinguish empty, permission-denied, timed-out, unsupported, unavailable and partial states. Event summaries contain only ID, provider, level and optional time/record ID; raw event messages and payloads are excluded. No Windows Event Log provider, IPC command or UI is wired yet, so this contract performs no OS log query.

M4-011 adds typed Terminal/SSH/RDP profile metadata. Terminal application choices are an enum allowlist; SSH/RDP targets use strict single-host validation and nonzero ports. Credential fields can hold only an opaque UUID reference, never a password or key value. Profile export uses a separate DTO that omits the reference, and safe log summaries omit profile name, host, username and reference. Unknown profile fields fail deserialization. This task adds no persistence, secret-store integration or connection launch; those require separate reviewed adapters.

M6-013 adds a provider-neutral `CredentialStore` boundary with opaque UUID references, store/retrieve/delete operations and closed errors that cannot echo secret values. Its mock tests verify the contract and redaction boundaries. No OS secure-store backend, IPC command, persistence, renderer access or connection launch is implemented; an OS-backed provider requires a separate task and security review.

M6-014 adds a strict versioned export-integrity manifest contract recording artifact format/version, payload byte size and SHA-256. Verification checks manifest shape, size and digest against the supplied bytes and rejects changed or truncated fixtures. This contract is not yet wired into a backup/export UI or file workflow; it detects accidental corruption only when the manifest is trusted separately and does not provide authenticity or encryption.

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

M5-001 implements this as a typed declaration contract only. A source definition records stable ID, content kind, access method, HTTPS source/terms references, reviewed terms and rate notes, minimum poll interval, request timeout, response/item caps, freshness threshold, health and last attempt/success timestamps. `enable()` and enabled-state deserialization validate the entire declaration and fail closed; the contract performs no fetch, polling, persistence or scheduling.

M5-002 adds a strict normalized `IntelligenceItem` DTO with source identity/URL, item canonical URL, external ID, bounded title/summary/category, optional content hash, freshness and verification labels. `published_at`, required `observed_at`, and optional `event_time` are distinct RFC 3339 instants normalized to UTC; missing publication/event times stay absent. The DTO validates bounds and safe HTTPS references but does not persist items or canonicalize links beyond validation.

M5-003 adds a native RSS 2.0 adapter callable for one explicitly enabled user source. It enforces the declared per-source minimum interval in-process, timeout, response and item caps; rejects redirects, proxy use, private/reserved DNS results and unsafe item URLs; parses RSS with a bounded XML parser; and returns stable errors for timeout, transport, HTTP, size and parse failures. Because the HTTP client is blocking, future IPC callers must dispatch it to a bounded blocking worker. The reviewed example source is JPL's documented public CNEOS RSS feed (`https://cneos.jpl.nasa.gov/feed/news.xml`); it is test metadata only and is never enabled or fetched automatically. The adapter has no scheduler, IPC/UI wiring or persistence; those remain follow-up tasks.

M5-004 adds in-memory transient-failure backoff to the RSS request boundary. Retry eligibility is the later of the user-declared minimum poll interval and an exponential delay that starts at 60 seconds and caps at one hour; no automatic retry is performed. Successful fetches reset the failure count, and timeout/oversize errors remain stable typed outcomes. Backoff state is process-local until source state persistence is implemented.

M5-005 adds conservative HTTPS URL canonicalization and a versioned SHA-256 content fingerprint. Item records retain the exact source-provided item URL separately from the canonical identity URL; only URL syntax, fragments and a fixed list of tracking parameters are normalized. Fingerprints preserve case/punctuation, collapse whitespace, and encode title/summary boundaries explicitly. The RSS adapter uses both helpers; no deduplication is performed here.

M5-006 groups a bounded in-memory batch of at most 2,000 items when they share a canonical URL or recomputed content fingerprint. Each group retains every validated item in input order and exposes all unique source IDs in first-seen order; supplied external hashes are not trusted as a grouping key. This creates an aggregation view only, not a persisted merge or deletion.

M5-007 derives a serializable freshness view from connector enablement, health, last-success time and its declared freshness threshold. Only `fresh` sets `isLive`; stale, failed, blocked, rate-limited, disabled, missing, invalid and future timestamps never claim live data. RSS attempts update the caller's in-memory last-attempt/last-success timestamps and health.

M5-008 adds a versioned SQLite table for bounded, user-configured RSS declarations and narrow native list/save/enable commands. The renderer cannot supply storage paths or SQL. New sources are disabled, must be explicitly marked terms-reviewed before enablement, and are reset to disabled when the feed URL changes. The Intelligence Sources page displays persisted settings and freshness, provides cadence and terms controls, and does not fetch on open. Poll cadence is configuration metadata only; there is no background scheduler or startup request, and disabled sources are rejected by the RSS adapter.

M5-009 adds a v12 SQLite table for bounded topic watchlists, with native list/save/pause/delete commands. Keyword and category filters are normalized and bounded; a paused list cannot match an item. Keyword matching uses case-insensitive literal substrings in title/summary, categories are exact IDs, and populated groups combine with AND while alternatives within each group combine with OR. The Topic Watchlists page is explicitly a saved-filter manager: it stores no items and starts no fetch, polling or notification behavior.

Trend calculation is pure and in memory over a bounded batch of validated items and a saved watchlist. The caller declares adjacent equal-length comparison and observation windows and supplies selected/successful/failed source evidence for each. Per-category counts use existing source-preserving deduplication; unavailable counts serialize as `null`, while a successful empty result is a known zero. Coverage labels distinguish no sources, no successful fetch, partial and complete windows. Ratios are emitted only for complete windows with the same selected source scope and a nonzero baseline. Empty category sets produce no synthetic rows, but result-level coverage remains available. Trend observations are not persisted.

M5-012 persists only complete successful results of explicit user-triggered fetches in a bounded app-owned table. Item queries accept only bounded source/category/search filters and a capped page size; item detail reads by validated ID. The Intelligence list exposes source/category/time/freshness metadata and links to validated HTTPS source and original item URLs. No fetch or polling occurs on route load; saved source runtime health is updated after each manual attempt.

M5-013 adds app-owned local research notes with a typed one-to-one intelligence item reference and a source attribution snapshot. Save is idempotent per item and does not duplicate source content. A version-1 JSON export uses a native user-selected save dialog and fails if the chosen path already exists. Notes survive item retention; exports state whether the referenced item is still available. This does not implement general-purpose Notes or Collections CRUD.

M5-014 stores a strict, versioned desktop-notification preference in app-owned metadata and defaults it off. The native fetch path emits a generic OS notification only after a user-triggered fetch succeeds with at least one returned item and the preference is enabled; notification errors do not fail the fetch. Notification settings request OS permission only from the user's enable action. No startup, timer, background, or notification-triggered fetch is registered.

M5-015 adds an official U.S. National Weather Service active-alert Atom preset for one explicitly selected state or DC area. It uses the documented public Atom endpoint and identifies Moonships through an HTTP User-Agent. New presets remain disabled and terms-unreviewed; the minimum interval is 30 minutes, and fetching is still an explicit manual action. The preset has a 10-second timeout, a 2 MiB response limit and a 500-item limit. HTTP 429 and NWS-specific HTTP 403 become `RateLimited`; other non-success responses, including 5xx, remain `HttpStatus`. Existing manual retry backoff applies and no automatic retry or schedule is added. The feed is awareness information only, not an authoritative substitute for NWS warnings or an emergency alerting/EAS system.

M5-017 adds adversarial Atom fixture content that asks the system to ignore rules, invoke shell and export secrets. The existing parser keeps that bounded payload as untrusted title/summary text and the Intelligence detail displays it as text. Ingestion/display invokes no model or tool, and retrieved source content cannot select tool IDs, permissions, network targets or native policy.

M5-018 adds a user-confirmed source export and deletion path. Export uses a native Windows Save dialog, writes a versioned JSON snapshot of source settings and all retained source items (up to the existing 10,000-item storage bound), and refuses to overwrite an existing path. Deletion transactionally removes the declaration and that source's stored Intelligence items; it preserves Research Desk notes and their provenance snapshots plus only a bounded deletion audit summary (source ID/name/feed URL/time/item count). Watchlists remain because they are independent filters. No deletion occurs until the user confirms in the source UI.

M6-001 defines typed `AdapterManifest` metadata and `AdapterHealth` states for future specialist adapters. IDs, three-part numeric versions, descriptions, fixed kind/capability enums and optional health timestamps/diagnostics are bounded and validated. Capability labels describe intended needs only; they grant no permission. The contract is data-only: it does not discover, install, load, update or execute external code, and it is not yet wired to persistence or UI.

M6-002 provides an explicit one-shot HTTPS GET Inspector on port 443 only. The target must be a public DNS/IP address; DNS is resolved once and pinned, private/reserved answers are rejected, redirects and proxies are disabled, the timeout is 10 seconds, response bytes are capped at 256 KiB, and a process-local gate allows only one active request and at least five seconds between starts. No credentials/cookies/custom headers are sent. Response content is returned to the open UI in memory; saved history retains only status, size, safe request/response headers, observation time and a URL with common credential query keys redacted. History is local, capped at 200, and stores no response body.

M6-004 lets the user create a follow-up Task from a saved HTTP inspection by opaque history ID. Native code loads the persisted, redacted record rather than accepting renderer-supplied metadata, enforces the history and Task description size limits, and copies only result ID, redacted URL, status/error, UTC observation time and response byte count into the Task description. Response bodies and headers are not copied. The link remains identifiable by result ID while the bounded 200-entry Inspector history retains the corresponding record.

M6-005 defines a data-only observability provider manifest and health view for logs, metrics and traces. Fixed provider status and freshness enums distinguish fresh, stale, unknown and degraded/unavailable/disabled/unsupported/error states; only ready providers with a valid recent success time are current. IDs, names, signal kinds and freshness thresholds are bounded. The contract has no endpoint, credentials, transport, persistence, queries or polling implementation.

M6-006 exposes a manually refreshed local metrics snapshot from the existing read-only system inventory: logical processor count, physical memory totals/availability and Windows volume capacity. The typed response identifies its fixed local OS API source, sample timestamp and per-field available/unsupported/unavailable state. It does not persist samples, poll in the background, collect process details, or send telemetry remotely.

M6-007 defines a strict security-assessment scope declaration with exact hostname/IP targets, explicit exclusions, a non-empty authorization note and an RFC 3339 expiry. Start validation rejects empty/duplicate/invalid targets, missing authorization, expired scopes and expiry beyond one year, normalizing hostnames and IP literals. This model performs no assessment or request; future executors must revalidate scope and apply tool-specific host/IP matching and approval gates.

M6-008 adds a pre-request URL scope gate. It matches canonical hostnames/IP literals exactly, gives explicit exclusions precedence, rejects unsupported schemes/credentials/fragments, checks expiry at request time, and resolves then revalidates each redirect destination before a caller may follow it. The gate makes no request itself; any future executor must call it before every request and redirect.

M6-009 defines a bounded typed finding with closed severity, likelihood and lifecycle enums, scoped target identity and observation time. Severity and likelihood are independent required fields; there is no derived risk/priority field. A reusable UI summary renders the two dimensions in separate labelled rows; persistence and the assessment findings screen remain future work.

M6-010 registers a native `create_task_from_security_finding` handoff. It validates the finding and still-active assessment scope, requires the finding target to be authorized and not excluded, validates bounded remediation and an HTTP(S) evidence URL, then creates a Personal inbox Task with severity, likelihood, scope, observation time, remediation and evidence URL. Sensitive evidence query values are redacted, and no evidence URL is opened or fetched.

M6-011 adds a Windows native multi-file picker and SHA-256 manifest workflow. Only up to 50 user-selected regular files, 2 GiB aggregate, are read locally; the renderer receives leaf names, byte counts and digests, never paths or content. The versioned JSON manifest includes a self-hash and explicit integrity-only limitation. Save IPC accepts only a bounded opaque handle and retrieves the exact native-cached snapshot (at most 10); it does not trust renderer-supplied entries or digests. Saving uses a native destination picker and create-new semantics to refuse overwrite; no evidence bytes are copied or uploaded.

M6-003 adds a local OpenAPI document viewer. A native file chooser reads at most 1 MiB of UTF-8 text and returns no path; parsing/validation runs locally with YAML 1.2 duplicate-key rejection and aliases disabled. The validator checks OpenAPI 3.0/3.1 version, `info.title`, `info.version` and a bounded `paths` object. The original file text is shown as escaped text only. It neither resolves `$ref`, loads remote schemas, nor contacts API servers; this is a bounded structural validator, not full OpenAPI conformance.

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

The Command Palette is a keyboard-first presentation surface over typed registered command descriptors and local entity search adapters. It performs deterministic case-insensitive substring filtering across title, description, category and keywords; supports Arrow Up/Down and Enter while the search field is focused; and exposes accessible dialog/listbox semantics, entity/category and risk labels, loading and explicit empty states. Selecting a Project or Task opens its existing detail route; selecting a command resolves only an exact registered ID. The palette does not execute tools or grant permissions. Destructive, privileged and remote risk classes are visibly labelled for future preview/approval flows. Ctrl+K lifecycle and focus restoration are tracked separately in M1-033.


### Registered Command Palette actions (M1-032–M1-033)

`src/command-registry.ts` is the allowlist for palette entries. The UI receives descriptors projected from this registry and resolves selection by exact registered ID; unknown IDs fail closed. Navigation to the existing overview and opening Settings are direct UI actions. New Project and New Task open their local persistence forms; New Note remains an explicit preview because note storage is not implemented. Risky tools are not registered as executable palette commands; future execution must use the native policy/approval gate.

Ctrl+K (or Cmd+K on macOS) is ignored when focus is in an editable control, a dialog is already open, or the palette itself is open. Escape closes the palette, and the previously focused element is restored after closing.

### Global search adapters (M2-034)

src/global-search.ts composes independently registered adapters for command descriptors and the scoped list_projects / list_tasks APIs. Those native list commands supply active records from the Personal workspace; the renderer never supplies a workspace ID, SQL, filesystem path or executable action. Results carry an explicit command, project or task entity type and a stable target ID for record navigation. Each source settles independently so an unavailable database read does not hide commands or successful sources; the palette states which source failed and offers retry. Search stays in memory after the user opens the palette; it does not poll or mutate records. Future entity modules can register an adapter in the shared adapter list.
