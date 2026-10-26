# Security Model & Threat Analysis

**Status:** Required design baseline before any system-changing or network/security tool ships.  
**Default posture:** deny unless a capability explicitly permits the requested operation.

## 1. Security objectives

1. Protect the user's files, credentials, project data and system from unintended actions.
2. Prevent untrusted UI content, external source data or model output from bypassing native policy.
3. Limit each module/tool to the least authority needed.
4. Make consequential actions reviewable before execution and traceable afterward.
5. Keep source-data collection transparent and respectful of provider rules.
6. Support authorized offensive-security workflows without enabling arbitrary attacks against out-of-scope systems.
7. Ensure update packages and imported data are verified before trust is granted.

## 2. Assets and trust boundaries

### Assets
- User files and Windows profile
- Project/task database and local artifacts
- Credentials, API tokens, SSH profiles and provider keys
- Tool definitions and execution permissions
- Security assessment targets/scope
- Intelligence source data and research notes
- Update signing/trust configuration
- Local logs, reports and evidence
- User consent/approval decisions

### Trust boundaries
1. React/WebView → Tauri commands
2. Application services → OS/platform APIs
3. Core → domain module/tool executor
4. Moonships → external process/CLI
5. Moonships → internet/source provider
6. Untrusted page/log/document → AI context
7. AI/model output → tool registry/policy
8. Imported archive/plugin/package → local state
9. Update endpoint/package → installed application

Crossing a trust boundary requires validation and explicit data/permission rules. Never treat an origin URL, model output, file name, source article, log line or plugin manifest as trusted merely because it appears in the app.

## 3. Threat model

| Threat | Example | Control |
|---|---|---|
| Destructive path misuse | Path traversal or wrong directory deletion | Allowlisted root, canonicalization, protected-path denylist, revalidation and tests |
| Symlink/junction escape | Cleanup follows a link outside the intended folder | Do not follow links by default; test reparse points and resolved target containment |
| TOCTOU file change | Candidate replaced after scan but before deletion | Revalidate identity, type, path and policy immediately before mutation |
| Excess permission | Frontend/plugin gains broad filesystem/shell access | Minimal Tauri capabilities and native-side authorization |
| Injection through shell | User/AI data becomes a command string | Avoid shell string construction; use explicit executable + argument arrays and allowlisted commands |
| Prompt injection | A news page tells the model to exfiltrate secrets or ignore policy | Treat retrieved content as untrusted data; tool policy independent of model |
| Credential leak | API token appears in logs, exports, prompts or crash dump | OS secure storage, redaction, export exclusions, secret scanning |
| Malicious imported data | ZIP path traversal, oversized payload, malformed JSON | Size caps, schema validation, safe extraction, no auto-overwrite |
| Source abuse | Excessive polling, scraping prohibited pages or bypassing restrictions | Provider policies, documented cadence, rate limits, backoff and source disable controls |
| Rogue update | Tampered package or metadata | Verify signed updates before install; protect signing key; release audit |
| Denial of service | Infinite directory tree, hung CLI, enormous response | Bounded execution, timeouts, size limits, concurrency limits, cancellation |
| Stale/misleading intel | Old item displayed as live/verified | Source timestamps, freshness state, confidence/provenance and visible degradation |
| Unauthorized offensive action | User enters an unrelated public target | Scope declaration, authorization check, policy gate, rate limits and audit |
| Data loss on migration | Schema update partially applied | Transactional migrations, backup/restore test and no silent DB reset |

## 4. Permission model

Use named capabilities instead of one broad “admin” permission. Examples (exact names may be refined by implementation):
- `app.settings.read`, `app.settings.write`
- `projects.read`, `projects.write`, `tasks.read`, `tasks.write`
- `filesystem.scan.approved_roots`
- `filesystem.delete.approved_candidates`
- `system.processes.read`, `system.services.read`, `system.services.change`
- `network.dns.read`, `network.connectivity.test`, `network.scan.authorized_scope`
- `source.fetch.public_configured`
- `process.launch.allowlisted`
- `secrets.reference.read` (do not return secret contents to UI unless purpose requires it)
- `workflow.execute`
- `remote.execute.approved_target`

A capability describes authority; it does not by itself authorize every target or argument. Scope, risk, user consent and environment constraints are checked separately.

### External adapter manifest contract (M6-001)

- A manifest is bounded descriptive metadata with a stable ID, three-part numeric version, name, description, closed adapter-kind enum and a unique list of closed capability labels. Unknown JSON fields and unbounded values are rejected.
- Capability labels are documentation only and never grant a Tauri command, filesystem/network/process permission or approval. Health (`available`, `degraded`, `unavailable`, `disabled`, `unsupported`) and bounded optional diagnostic/time are reported separately from manifest identity.
- This contract does not enumerate installed files, trust a manifest as executable authority, download/update code, load a dynamic library, start a process or create an adapter runtime. Future execution requires a separately scoped task for signature/provenance, isolation, permission mediation, lifecycle and failure handling.

### Credential reference interface (M6-013)

- Domain code depends on the `CredentialStore` interface and opaque UUID references; no plaintext secret field is added to profile/config DTOs, SQLite, exports or logs. Providers return closed error codes without echoing values or provider diagnostics.
- Only the narrow native operation that needs a secret may retrieve it. A future OS-backed implementation must keep bytes within native code, use the OS secure store, and document user consent, replacement/deletion behavior and platform availability. No secret value is returned to the renderer.
- The current interface and mock provider are contract/test infrastructure only. No OS secure-store backend, IPC command, persistence, UI or credential read/write is registered.

### Export integrity manifest (M6-014)

- The strict v1 manifest binds an allowlisted-format identifier and nonzero artifact version to the exact payload byte count and lowercase SHA-256 digest; payloads are capped at 2 GiB and unknown fields/manifest versions fail validation.
- Verification fails on size or digest mismatch. The verifier consumes caller-supplied bytes and does not open paths, restore data, or modify files.
- A checksum detects accidental changes only if the manifest itself is trusted. It is not a signature, provenance proof, encryption or tamper-proof backup; no existing export flow or backup UI is wired to it yet.

### HTTP Inspector (M6-002)

- The only method is an explicit user-submitted HTTPS GET to one URL on port 443. Other ports, embedded credentials and fragments are rejected; the user sends no custom headers, authorization or cookies. Each request resolves one host once, rejects any private/local/reserved address and pins the HTTP connection to the accepted public addresses. HTTPS is required; proxies and redirects are disabled.
- One request is active at a time, with a process-local minimum five-second interval, 10-second total request timeout and 256 KiB response cap. No automatic retry, route-open request or background fetch occurs.
- The response body is shown only in the current UI result and is not saved to history. History is capped at 200 and stores status, byte count, UTC observation time and bounded request/response headers only after case-insensitive redaction of authorization, proxy authorization, cookie, set-cookie, token, API-key, secret and password headers. Sensitive URL query keys (token, API key, key, secret, password, auth, signature) are redacted in history; the original query is still sent only because the user explicitly submitted the URL. Unknown headers/paths are not assumed safe when exported in a future task.

### HTTP inspection Task handoff (M6-004)

- The renderer submits only a UUID result handle. Native code reloads the saved history record; it never accepts a renderer-provided URL, status, header, or body as evidence.
- The Task contains only the result ID, redacted URL, status or closed error code, observation timestamp and bounded response byte count. It never includes response body, request/response headers or original unredacted URL/query values.
- Both stored history JSON (256 KiB) and the Task description (10,000 characters) are bounded. An evicted or malformed history row fails closed; no Task is created.

### Observability provider contract (M6-005)

- Provider declarations contain only a bounded ID, display name and fixed logs/metrics/traces kinds. Health contains only a closed status enum, optional last-success timestamp and bounded freshness threshold.
- No endpoint, credential, query, raw telemetry payload, provider plugin execution, polling or persistence is defined. Unknown fields are rejected by the strict DTO.
- A recent timestamp never overrides degraded, unavailable, disabled, unsupported or error status; missing, invalid or future timestamps cannot be labeled fresh/current.

### Local metrics snapshot (M6-006)

- Read only the existing bounded OS inventory counters. The snapshot exposes no process names, file paths, device identifiers, account data or raw OS messages.
- Refresh occurs only on page entry or explicit user action. No timer, startup collection, remote exporter, persistence or telemetry is added.
- Values carry explicit availability status; unsupported/unavailable values are null and are not represented as zero. The source and sample time are included in the response.

### Assessment scope model (M6-007)

- A future assessment start must validate a non-empty list of exact host or IP targets, explicit exclusions, a non-empty authorization note and a future expiry. Scope and exclusions are capped at 100 entries each; authorization text is capped at 2,000 characters and expiry at 365 days.
- Target values reject URLs, ports, wildcards, malformed names and invalid IP literals. The current declaration model does not imply ownership or prove authorization; the note records the user's stated basis and must be reviewed before execution.
- No network call or assessment is performed by the model. Any future executor must revalidate immediately before each request, enforce exclusions and redirect/out-of-scope rules, apply rate/deadline limits and require approval appropriate to risk.

### Assessment destination gate (M6-008)

- Before each request, recheck that scope has not expired, then compare the canonical destination hostname or IP literal to the validated authorized entries; exact host match only, no implicit subdomain or IP-range expansion. Explicit exclusions always deny.
- Permit only HTTP(S) URLs without embedded credentials or fragments. Resolve relative redirects against the current URL, then apply the same scope check to the resolved destination before following; reject external, excluded or unsupported destinations before network activity.
- The gate performs no DNS lookup or request and is not a complete SSRF control by itself. Future network execution must additionally pin/validate resolved addresses, restrict ports, enforce deadlines/rate limits and disable unchecked redirects.

### Security finding model (M6-009)

- Severity and likelihood are independent required closed enums; do not substitute Task priority for either or present one combined value as if it were both.
- Finding title/summary lengths, canonical UUID, exact target and RFC 3339 observation time are validated. Free-form provider payloads, credentials and evidence blobs are not fields in this model.
- The reusable UI presents Severity and Likelihood as separate named rows. This model does not establish exploitability or automatically compute a risk score.

### Finding-to-Task handoff (M6-010)

- Native code validates the finding and assessment scope at handoff time, rejects an expired scope or target not explicitly authorized (or explicitly excluded), rejects future observation timestamps and enforces remediation/Task payload size bounds.
- Evidence is an explicit HTTP(S) URL only; embedded credentials, fragments and unsafe schemes are rejected, common sensitive query values are redacted before storage, and the URL is never fetched or opened.
- The Task snapshots scope authorization note, targets, exclusions and expiry with the finding timestamp, severity, likelihood, remediation and evidence link. Task priority remains its ordinary independent default and is not derived from severity or likelihood.

### Evidence manifest and file hashing (M6-011)

- Read only files chosen through the native picker. Enforce at most 50 entries and 2 GiB aggregate while streaming SHA-256; return leaf names only, never renderer-visible paths or file bytes.
- The strict v1 manifest validates every digest, entry/aggregate size, leaf-only filename, fixed integrity-only assurance, and its own canonical payload hash. Saving is user-triggered, uses a native Save dialog and create-new semantics, and refuses overwrite.
- Save IPC accepts only the opaque ID of a native-generated snapshot from a ten-entry process-local cache; the renderer cannot supply or alter saved filenames, sizes or digests. An evicted handle requires a new explicit file selection.
- A digest supports later byte-integrity comparison only. Selection and hashing do not establish file origin, acquisition procedure, custody, authenticity, or forensic validity. The manifest must retain this limitation and must not be called a forensic chain-of-custody record.

### OpenAPI document viewer (M6-003)

- A user action opens the native Windows file picker; renderer code cannot supply a path. Native code reads at most 1 MiB, rejects non-UTF-8 and oversized files, and returns only the filename plus bounded text. No database persistence or network access is performed.
- The pinned local `yaml@2.9.1` parser uses YAML 1.2, strict duplicate keys and `maxAliasCount: 0`; parser exceptions are caught, and diagnostics expose stable parser codes rather than source excerpts. Input is capped before parsing. The viewer displays source as escaped text, not HTML.
- Validation only checks a top-level object, OpenAPI 3.0/3.1 version, required `info.title`/`info.version`, and bounded `paths`. It does not dereference `$ref`, access API servers, fetch external schemas or claim full specification conformance. Imported descriptions/examples are untrusted text.

### Risk classes
- **R0 Read-only/local:** local metadata, format conversion, task searches.
- **R1 Network/read-only:** DNS, GET/status checks or source fetch. Still subject to host/provider rules and limits.
- **R2 Reversible state change:** settings, labels, task status, creating files in app-owned data.
- **R3 Destructive/system-changing:** file deletion, service changes, privileged operations, bulk modifications.
- **R4 Remote/high-impact:** remote command execution or high-impact security operations.

Risk class is separate from task priority, vulnerability severity and business impact.

### Approval rules
- R0 may execute without per-action confirmation if feature is configured.
- R1 requires visible target/source and configured rate/timeout rules; external or sensitive sources may require setup consent.
- R2 follows explicit user settings and displays a clear change summary.
- R3 requires a preview and explicit action-specific confirmation, then revalidation.
- R4 requires target scope, authorization context, detailed preview and explicit confirmation; some operations may remain unsupported.
- App startup and background tasks cannot silently approve a consequential action.
- Approval binds to exact arguments, tool version, target, scope and preview hash. Any material change invalidates it.

### Bounded network diagnostics (M4-001 through M4-006)

- A request names one DNS hostname/IP or one TCP hostname/IP plus one explicit port. A URL, embedded port, wildcard, CIDR or target range is rejected; the UI must show the exact target and requested operation before starting a future adapter.
- Hostnames are ASCII DNS labels (IDNs must be punycode); literal IPv4 and IPv6 are supported. User whitespace is not silently trimmed. A target is never accepted from page content, model output or a redirect.
- Timeout is 100–10,000 ms, with a 2,000 ms default. Adapters enforce one concurrent diagnostic and at most ten runs per minute; DNS output is capped at 32 IP addresses. No automatic retries or range expansion are allowed.
- The DNS adapter performs read-only resolution through the OS resolver on one bounded worker and reports timeout to the caller. The OS resolver call cannot be forcibly cancelled; a stuck call occupies the single worker, and the adapter never creates replacement workers. IP literals are returned directly without a resolver request.
- The TCP adapter accepts one host and one nonzero port. DNS resolution and socket connect share the same single worker, queue, deadline and rate bound; each resolved address is tried only within the original request deadline. It does not scan adjacent ports, targets or ranges.
- Ping capability detection opens and closes the Windows IP Helper ICMP handle without sending an echo request or launching an external program. Unsupported platform/API conditions return a typed unsupported reason.
- Traceroute is explicitly deferred until a reviewed adapter can enforce hop and wall-clock limits. There is no executable fallback or silent success-shaped result.
- The IPv4 subnet calculator is pure local arithmetic over one strict address/prefix input. It does not resolve names, query network state or expand to scanned targets.
- Result provenance identifies the exact target/port, operation, UTC observation time as epoch milliseconds, elapsed time, IP-literal source, OS resolver or TCP socket, outcome, stable error code and bounded resolved addresses. Invalid input is a validation error; resolver failures are distinct typed results. M4-002 added no IPC command; M4-014 later registered the bounded one-target command.

### Connector declarations (M5-001)

- Connector declarations require a stable bounded ID, a matching kind/access-method pair, HTTPS-only source and optional terms URLs without embedded credentials, and non-empty terms/rate-limit notes.
- Enabling requires reviewed, non-restricted terms and bounded cadence, timeout, body size, item count and freshness values. Enabled-state deserialization reruns validation; no raw connector can bypass the enable check.
- This task adds no network access, polling, scheduling or persistence. A future connector implementation must apply DNS/IP policy, redirect handling, timeout/size/rate enforcement and provider terms at the actual fetch boundary.

### Intelligence items (M5-002)

- Connector output remains untrusted text. Item IDs, source IDs, HTTPS URLs, timestamps, bounded title/summary/category and optional SHA-256 format are validated; UI consumers must render text as text and never interpret summary markup or instructions.
- `published_at`, `observed_at` and `event_time` are separate values. Missing source dates remain missing; observation time is not substituted as publication or event time.

### RSS fetch boundary (M5-003)

- Fetch occurs only through an explicit native call for a connector whose user-provided declaration is enabled, validated, RSS/Public RSS, and terms-reviewed. There is no startup fetch or scheduler.
- Apply a process-local minimum interval by connector ID before each attempt; a failed attempt also consumes the interval. The limiter tracks at most 256 source IDs and resets on app restart until durable settings/history are added.
- Resolve each hostname once on one bounded worker. A timed-out resolver remains the sole active resolver worker until it returns. Reject the whole result if any address is private, local, reserved or otherwise non-public, cap DNS results at 32, and pin the HTTP client's resolution to the accepted set to prevent a second lookup/rebinding.
- Use HTTPS only, disable proxies and redirects, cap total request time and response bytes, and reject non-success HTTP status. Parse only RSS 2.0 with a pull parser, reject DTDs and malformed/oversized item data, and cap item count. Treat title/summary as untrusted text; do not execute instructions or render markup from a feed.
- The fixture uses JPL's officially documented public CNEOS RSS feed as an attributed source example. The URL is not built-in configuration and tests do not contact it. External pages may change terms or availability; users must review the source terms before enabling it.
- The adapter uses a blocking HTTP client; any future IPC integration must put it on a bounded blocking worker so it cannot block or panic inside the UI async runtime.

### HTTP source failures and backoff (M5-004)

- Enforce the connector timeout and response-byte ceiling while reading the body; a known oversized content length is rejected before reading. Slow reads, transport failures, non-success HTTP status and malformed/oversized RSS return stable error codes and no partial item set.
- Failed attempts count toward per-source exponential backoff (60 seconds doubling to a one-hour cap) and the declared minimum poll interval. The next allowed manual request is the later of those two deadlines. Success clears failure history; the adapter never retries automatically. This rate/backoff state is process-local and bounded to 256 connector IDs.

### Official NWS active-alert Atom preset (M5-015)

- The only built-in provider preset is the documented public `https://api.weather.gov/alerts/active.atom?area=...` feed for one selected U.S. state or DC area. The native parser recognizes the exact NWS host/path for the `weather_alert` category; it does not activate emergency systems or replace official NWS warnings.
- The request identifies Moonships with a descriptive `User-Agent`. The saved preset is disabled and terms-unreviewed until the user reviews source terms and separately enables it. Fetch remains a manual action; there is no startup, page-open, timer or background request. The 30-minute minimum is more conservative than NWS's published 30-second recommendation.
- NWS preset requests are bounded to 10 seconds, 2 MiB and 500 items. Existing HTTPS-only transport, one bounded DNS resolution, public-address rejection, pinning, no proxy/redirect, XML DTD rejection and text-only treatment apply. Atom entry content is untrusted; no page/feed instruction can alter policy or tools.
- HTTP 429 from any source and HTTP 403 from exact host `api.weather.gov` map to `RateLimited`; this accounts for NWS's documented rate limiting/blocking behavior. Other non-success statuses (including 5xx) map to `HttpStatus`. Failures enter the existing bounded exponential backoff (60 seconds to one hour, combined with the source cadence); no automatic retry occurs. No provider response body is surfaced as an instruction or error detail.

### Intelligence identity (M5-005)

- Canonicalization accepts only validated HTTPS URLs without credentials and preserves the exact original item URL. It removes fragments and only the documented analytics parameter allowlist; it does not rewrite other query fields, path semantics or source attribution.
- Content fingerprints are deterministic SHA-256 values over a versioned, length-delimited title/optional-summary representation. Whitespace is collapsed within each field; case and punctuation are unchanged. Fingerprints identify matching content only and do not establish truth, authorship or independent verification.

### Intelligence deduplication (M5-006)

- Deduplication validates each bounded item before grouping and caps a batch at 2,000 records. It groups by exact canonical URL or a fingerprint recomputed from the bounded title/summary, not by an externally supplied hash. All original records and unique source IDs remain in the result; grouping does not delete or merge saved data.
- A group is a navigation/aggregation aid, not evidence that its sources independently verify one another or that matching content is true. Source provenance stays available on every member record.

### Connector freshness (M5-007)

- `isLive` is true only for an enabled source with a valid successful timestamp no later than the configured freshness threshold and no blocked/error/rate-limit/unavailable/stale health state.
- Missing, malformed or future last-success values cannot be presented as current. A failed fetch retains the prior success timestamp but updates health, so an earlier success never masks a current failure as live. UI consumers must render both status and timestamp from the typed freshness view.

### Source settings and controls (M5-008)

- Connector declarations are stored only in the app-owned SQLite database. Native commands accept strict typed payloads, validate the full connector contract, cap the source count, and expose no renderer-supplied path, SQL, host scope, or request limits.
- New sources remain disabled. Enablement requires a user declaration that terms were reviewed; restricted or unreviewed sources cannot be enabled. Changing a feed URL disables the source and clears its old freshness timestamps.
- The displayed polling interval is a minimum-cadence setting, not a scheduler. This milestone adds no startup fetch, page-open fetch, timer, background polling or automatic retry. Any future scheduler requires a separate explicit opt-in and review. A disabled declaration cannot pass the RSS fetch boundary.
- The renderer displays source URLs and terms notes as text and links only to validated HTTPS URLs. Failure to read persisted settings is shown as unavailable rather than as an empty list.

### Topic watchlists (M5-009)

- Watchlists and filter values are stored in the app-owned database, with strict DTOs, 128-list and per-filter count/length caps, normalized category IDs, and literal keyword matching. The renderer cannot submit SQL or storage paths.
- Keywords and categories are filter data only. They are never interpreted as instructions, executable expressions, regular expressions or source URLs. Retrieved item content remains untrusted and is not stored by this task.
- Pause disables matching without deleting the filter. Delete is an explicit confirmation and removes only the selected watchlist. No scheduler, automatic fetch or notification is introduced.

### Trend calculation (M5-010, M5-011)

- The calculation accepts at most 2,000 items, validates each item before using it, and relies on the bounded deduplication helper. Windows must be explicit, adjacent, equal length, non-overlapping, no more than 90 days each, and must not extend into the declared calculation time.
- Window membership uses normalized observation time, not provider publication/event dates. Each window must include bounded source-scope and fetch-outcome evidence, validated for unique IDs and subset relationships. No successful source yields a null count; successful empty results may yield zero. Partial coverage and differing selected scopes suppress relative-change output, which also remains undefined for a zero baseline. No matching categories produce no observation rows, while result-level coverage still distinguishes a successful empty window from unavailable data.
- Trend calculation is pure in-memory processing. It makes no network requests, starts no scheduler, stores no item content and interprets no source text as instructions.

### Stored intelligence items and display (M5-012)

- Only a manual IPC fetch for an enabled and valid saved connector can access the network. The existing RSS boundary enforces public DNS/IP pinning, HTTPS, no proxy/redirect, request/size/item bounds, terms review, cadence and backoff.
- Results are validated and persisted atomically only after the complete RSS document parses successfully. Storage retention is bounded to 10,000 records; native reads cap pages at 200, and source/category/search and item ID inputs are validated. Runtime source health is persisted after attempts so errors cannot leave an old success presented as live.
- Feed title/summary are rendered only as text. URLs are validated HTTPS item/source links opened externally with a safe `rel`; page routes use item IDs and never accept arbitrary paths or SQL. Opening list/detail routes performs no network activity.

### Research note/source linking (M5-013)

- Save accepts only a validated stored item ID; the native service creates an app-generated note ID and a typed source-attribution snapshot. The note body does not duplicate fetched item text, source content remains untrusted, and saves are idempotent per item.
- Export serializes a bounded versioned note/reference document and opens only a native save dialog. The renderer supplies a note ID, never a path. Cancellation writes nothing; an existing selected destination is rejected rather than overwritten. The reference snapshot remains if the bounded item store later evicts its item, and export declares item availability.

### Desktop notifications (M5-014)

- Notification delivery defaults off and is checked in the native fetch command. Only a successful user-triggered fetch with returned items can produce a generic notification; permission or notification API failures never fail the fetch. The notification contains no feed title, item title, URL or source content.
- OS permission is requested only from the Settings enable action. The renderer receives only permission-check/request APIs, not notification-send authority; native code owns sending and reads the strict persisted preference. Disabling the preference suppresses future notifications even if OS permission remains granted.
- Enabling notifications does not enable connectors, start network access, or schedule work. No startup, timer, background, or notification-triggered fetch is implemented.

### Intelligence source export/deletion (M5-018)

- Source export takes only a validated source ID, reads the source declaration and all retained validated items (bounded by the 10,000 item retention ceiling), then opens a native Windows Save dialog. The renderer supplies no output path or item contents. JSON writes use create-new semantics and fail on an existing destination; cancellation writes nothing.
- Source deletion requires an explicit UI confirmation and a native transaction. It removes the source declaration and all stored Intelligence items attributed to that source. Research note text and its source-attribution snapshot remain intact; saved watchlists remain because they are independent filters. A bounded audit retains only source ID, display name, feed URL, deletion timestamp and removed-item count, capped at the newest 1,000 records. Audit rows contain no titles, summaries, secrets or fetched content.
- Export and deletion failures are surfaced; deletion rollback leaves the source and items available. No request is performed by opening the source screen or export dialog.
- A manual fetch rechecks the saved connector's existence, enablement and identity inside the item-write transaction after network I/O. If the source was deleted, disabled or materially changed while a request was running, its response is discarded rather than restoring deleted source content.

### Read-only system inventory (M4-007)

- Inventory reads only OS family/process architecture, available logical processors, physical memory counters and capacity/free bytes on the volume containing the Windows directory.
- Values use explicit available/unsupported/unavailable states. A missing platform API or failed query stays null with status; no estimate is substituted.
- The inventory does not enumerate files, devices or processes, disclose the full system path, or register an unrestricted renderer command.

### Process inventory (M4-008)

- The native adapter takes a Tool Help process snapshot and returns only executable name, process/parent IDs and thread count; output is capped at 4,096 records and the snapshot handle is closed.
- It does not open individual process handles, read command lines or executable paths, launch a process, elevate, terminate or mutate a process. Its registered IPC command takes no renderer arguments.
- The UI requests one snapshot on entry and only refreshes after a user action. Permission denied, unsupported, unavailable, empty and partial results remain distinct.

### Windows service inventory (M4-009)

- The service adapter connects only to the local Service Control Manager and requests `SC_MANAGER_ENUMERATE_SERVICE`; output is capped at 2,048 service records and the manager handle is closed.
- It returns service name, display name and current state only. It does not open service objects or expose start, stop, restart, configuration, remote-host or elevation actions.
- Permission-denied, unavailable, unsupported, empty and partial results remain distinct in the UI; refresh is user-triggered and no background polling occurs.

### Event log provider seam (M4-010)

- Queries are scoped to a fixed channel enum, at most 100 summaries, and a 100–5,000 ms requested deadline. No free-form query string, XPath, remote computer, or unbounded time range is accepted; a future provider must enforce both limits.
- Only event ID, provider name, level, optional timestamp and record ID are represented; raw event messages and payloads are not returned.
- Empty, permission-denied, timed-out, unavailable, unsupported and partial states are distinct. The current seam has no OS provider, IPC command or UI and performs no event-log read.

### Connection profiles (M4-011)

- Profiles accept only an allowlisted local terminal enum or a strict single-host SSH/RDP target with a nonzero port. No shell command string, arbitrary executable path or free-form process arguments are part of the model.
- Credential slots contain only UUID secret references. Deserialization rejects unknown fields, including plaintext password/key fields; the profile export DTO omits the reference, and safe log metadata omits profile name, host, username and reference.
- M4-012 job history is local-only and the renderer has read-only access. Trusted native producers must use bounded, non-sensitive titles/stages/result summaries; credentials, raw provider output and arbitrary log text do not belong in these fields. Secret scrubbing is not inferred or promised by this storage contract.
- M4-014 diagnostics accept one manually entered hostname/IP, or one host and port, with the adapter's shared one-worker, timeout and rate limits. Task creation uses a short-lived native result handle rather than trusting renderer-supplied result fields; the summary omits resolved addresses and raw system messages. The target is shown before the user starts the check.
- M4-015 network regression tests use synthetic error kinds, injected monotonic instants and loopback only; tests do not contact arbitrary public hosts.
- Profile metadata is inert: this task adds no persistence, OS credential-store access, process launch or network connection.

## 5. Cleaner-specific safety

- M3-001 defines rules as typed metadata with closed root IDs only: app-managed temporary storage, app cache, or an OS-resolved user temporary directory. Rules and renderer inputs cannot supply arbitrary paths. The pure eligibility evaluator accepts metadata only and never reads or deletes files. M3-002's test-only traversal helper reads metadata recursively under a disposable fixture, skips links, and performs no content reads or mutations; it is compiled only for tests and is not an IPC entry point. M3-003 canonicalizes the native-resolved allowed root, existing candidate and native-policy protected paths, then rejects candidates equal to/outside the root, beneath a protected path, or any unresolved input. Protected paths must be assembled by native policy, not supplied by renderer data. M3-004 tests a Windows directory junction aimed outside the root: canonical path validation rejects the target and read-only traversal skips the junction. This check does not close time-of-check/time-of-use races; immediate identity revalidation remains required before production mutation.
- M3-005 represents each scan candidate with a per-scan UUID, a validated relative locator, and immutable root/type/mtime/size metadata. It excludes absolute paths and display labels as identity inputs. Future mutation must revalidate this snapshot against fresh filesystem and platform identity.
- M3-006 progress and terminal event DTOs carry only a job UUID, stage, bounded counters, cancellation support and a closed error code. They do not contain paths or file contents. Cancellation is best-effort or unsupported; a request is not reported as cancelled until work stops.
- M3-007 preview DTOs expose only a candidate UUID, leaf display name, category, size, timestamp and eligibility/skip reason. The UI preview performs no mutation and offers no deletion control.
- M3-008 requires a separate confirmation dialog for the selected eligible candidate IDs. The native approval request binds the exact preview UUID, unique IDs and estimated bytes; empty, duplicate or skipped candidates are rejected. Confirmation records intent only and cannot start deletion before later revalidation/executor gates.
- M3-009 captures a platform file identity (Windows volume serial/file index, Unix device/inode) and compares it with a fresh no-follow observation before any later action. Missing identity, changed file identity/metadata, link, path escape, protected path or policy change is a skip. Unsupported platforms fail closed; path-based revalidation still has a race window, so the executor must revalidate immediately before mutation and never cache a checked path.
- M3-010 executes only an exact approval/preview/candidate set, capped at 100 files per run, and rejects changed preview totals before mutation. Each selected file is independently revalidated; one failure does not stop later items. On Windows, the executor opens the checked file without following the final reparse point, denies concurrent writers/name replacement, compares the handle's final resolved path, file identity and metadata with the validated snapshot, then marks that handle for deletion. Locked, denied and changed items have closed per-item outcomes. Other platforms fail closed for deletion until a platform-specific handle-safe adapter is implemented. The executor is native-only and must not be registered as an unrestricted renderer command; its caller must be the native approval flow.
- M3-011 reports the preview estimate and actual logical sizes of files whose deletion completed as separate values. Failed/skipped files add zero to the actual total. This is not a physical free-space measurement; compression, sparse allocation and hard links can make reclaimed storage differ.
- M3-012 exercises executor outcomes with disposable fixtures: Windows locked files report `RESOURCE_BUSY` and do not block a later candidate; read-only files report `PERMISSION_DENIED`; changed files are skipped as `CHANGED` before deletion. These status codes are part of the per-item result, not fatal batch errors.
- M3-013 registers only two default rules, both rooted in Moonships-owned application data: `moonships-temp-stale` targets `app_data_dir/temporary`, requires a 7-day age and caps each item at 64 MiB; `moonships-cache-stale` targets `app_data_dir/cache`, requires a 30-day age and caps each item at 128 MiB. Both include regular files only and exclude links/junctions, directories, protected/outside paths, recent files and oversized files. The scratch root is for closed, disposable outputs; active outputs must remain elsewhere, and open/denied candidates fail individually. The cache root is for regenerable app-owned cache only. Neither rule includes the SQLite database, logs, settings, project/task data, exports, credentials, browser state, Downloads or user documents. The shared OS user-temp directory has no default rule.
- M3-014 stores only report totals, outcome counts, timestamps and an optional same-workspace Task ID. Candidate IDs, names and full paths are omitted by default. The report history command accepts no renderer-supplied scope or path and reads only recent summaries from the native Personal workspace.
- M3-015 JSON formatting and validation operate only on user-entered in-memory text. Input is capped at 1 MiB UTF-8, nesting at 128 levels and formatted output at 4 million characters; formatting changes whitespace while preserving numeric/string tokens. No filesystem, network, provider or shell access is used.
- M3-016 uses the exact-pinned `yaml@2.9.1` parser under ISC license, with YAML 1.2/core strict parsing and duplicate-key rejection. `parseDocument` errors are surfaced by stable code and line/column only; source excerpts are not shown. The formatter caps UTF-8 input at 1 MiB and output at 4 million characters, keeps aliases as YAML syntax without expanding them into JavaScript objects, and performs no filesystem, network, provider or shell access.
- M3-017 Base64 transforms accept bounded in-memory text, hex bytes or standard Base64 only. Encoded/decoded data is limited to 1 MiB; Base64 decoding is strict and UTF-8 detection is fatal, so invalid UTF-8 is never silently replaced. Binary output is labelled and only a 64-byte hex preview is displayed. No file, network, provider or shell capability is used.
- M3-018 URL tools operate only on bounded local text. Component mode encodes reserved separators as data; full URL mode validates an absolute URL and preserves its reserved syntax delimiters. Conversion never opens or fetches the URL and has no filesystem, network, provider or shell capability.
- M3-019 UUID v4 generation uses Web Crypto `randomUUID()` and fails closed when secure randomness is unavailable; it never falls back to `Math.random()`. Validation accepts canonical RFC-variant UUID versions 1–8 and the Nil UUID. No value leaves the local process.
- M3-020 text hashing accepts at most 1 MiB. File hashing opens only a path returned by the native Windows file picker, reads at most 2 GiB using a fixed 64 KiB buffer and exposes only the selected leaf filename, byte count and SHA-256 digest. The renderer cannot submit an arbitrary path, and the feature never modifies or uploads selected content. Errors do not include a full path; other platforms return explicit unsupported status.
- M3-021 compares only in-memory text, with a 1 MiB UTF-8 and 50,000-line cap per input, a 1,000 edit-distance cutoff and a 4 million character output bound. The renderer has no file, network, provider or shell access through the diff tool.
- M3-022 timestamp conversion accepts only bounded JavaScript-safe whole-number Unix values or strict ISO 8601 timestamps with a known timezone. It runs locally and displays UTC separately from device-local time with the timezone label; invalid input/range is rejected.
- M3-023's offline acceptance test makes `fetch` throw and completes each local DevTools workflow, including the native SHA-256 bridge, while asserting no browser fetch. The native selected-file flow opens a native picker and reads locally; no provider or HTTP call is part of these tools.
- Scan is read-only.
- Candidate set comes from registered cleanup rules, never arbitrary UI-provided paths.
- Default targets are narrow and documented; no broad recursive deletion of a user profile.
- Never touch registry, Windows component store, Windows Update, Prefetch, user Downloads/documents, browser cookies/session/auth data in the MVP.
- Do not follow symlinks/junctions outside an approved root.
- Revalidate identity, path, file type and eligibility immediately before deletion.
- Handle locked, changed, denied and inaccessible files as skipped/failed, not fatal crashes.
- Show estimate vs actual bytes removed separately.
- No claim of restoration unless a tested quarantine/backup path exists.
- No admin requirement in the normal flow.

## 6. Offensive security boundaries

Allowed product intent:
- Tools for systems the user owns or is explicitly authorized to assess.
- Scope recording, benign discovery, safe validation, lab simulation, fuzzing of owned/test inputs, evidence collection and remediation tasks.
- Local labs, CTF environments, intentionally vulnerable services and test fixtures.
- Reports, reproduction steps and defensive follow-up.

Disallowed design goals:
- Unauthorized compromise of third-party systems.
- Credential theft, credential harvesting or exfiltration.
- Malware deployment, persistence, destructive payloads or covert access.
- Evasion features designed to conceal unauthorized intrusion.
- Unbounded automatic scanning or attack execution.
- Bypassing authentication, paywalls, platform restrictions or source rate limits.

Before executing an assessment:
1. Require declared target scope.
2. Normalize target identity and reject ambiguous/out-of-scope targets.
3. Enforce tool-specific scope and rate/time limits.
4. Show planned actions and likely impact.
5. Require approval appropriate to risk.
6. Record result and support abort.
7. Offer evidence and remediation task, not an automatic remediation by default.

## 7. AI and untrusted input

### Artifact and reference handling — M2-026
- Treat URL, provider IDs, file names and metadata as untrusted input; validate them in the native adapter before persistence or opening.
- External URLs must be absolute HTTP(S) with a host and no embedded username/password. Reject script, data, file and unknown schemes. Never fetch, download or open a saved reference automatically.
- Provider references remain namespaced opaque IDs; they do not become URLs or trigger network access without a separately reviewed adapter and explicit user action.
- Local artifact references are opaque IDs resolved only under app-owned artifact storage. Do not accept renderer-supplied absolute paths or arbitrary filesystem paths. Canonicalize and revalidate containment before explicit open; reject traversal and symlink/junction escape.
- Opening a link or managed artifact requires a user action and a narrow native platform adapter. Never construct shell commands from reference text or execute referenced content.
- Persist reference metadata only; binary size/retention, import and export require their own contracts. Do not store credentials in URLs or reference metadata.

- Model output never sets permissions or policy.
- Validate every AI-generated tool ID and argument against an allowlisted registry and strict schema.
- Retrieved web pages, feeds, video transcripts, code, comments, logs and task descriptions are untrusted data, even if they contain instructions aimed at the model.
- M5-017's bounded Atom fixture includes text requesting policy override, shell execution and secret export. The parser preserves it only in the ordinary title/summary fields; the Intelligence detail renders it as text, creates no attacker URL/action, and retains source-attributed verification. No model/tool is invoked by ingestion or display, so fixture text cannot change registry, permissions, native policy or fetch behavior.
- Keep user instructions/policy separate from retrieved content in prompt construction.
- Do not automatically execute shell commands copied from sources or generated by models.
- Require a human review step for consequential action.
- Store only necessary AI traces; redact tokens, personal data and local paths where possible.
- Provide local-only/no-AI mode.
- If a model is unavailable or returns invalid output, fail closed for that AI-dependent operation while preserving ordinary non-AI tools.
- M7-001 workflow documents are bounded to 1 MiB, reject unknown fields and value tags, and require explicit tool ID/version plus typed input values. The schema parser does not grant tool authority or perform registry lookup/execution; later workflow validation and the independent policy gate remain mandatory.
- M7-002 rejects workflow graphs above 128 steps, duplicate IDs, missing/duplicate dependency references and dependency cycles before a future runner can consider execution. DAG validity does not authorize a tool; registry version matching, capability checks, target scope and approval are still required at execution time.
- M7-003's runner only invokes an exact registered in-process handler version; no production handler or IPC workflow entry point is added. A future production handler must check capabilities, target scope and approval independently before consequential work.
- M7-004 requires explicit per-step timeouts and stop/continue policy for executable schema-v2 workflows. A handler receives its deadline, and the runner detects a late return, but a synchronous handler cannot be preempted; only bounded/cooperative handlers may be registered. Continue policy may proceed only to independent steps after a failure; dependent steps are skipped. Policy still cannot override capability, target scope or approval checks.
- M7-005 tool metadata defaults to unknown idempotency and `never` retry. A retry policy must name transient failures and a bounded count (1–5), and is accepted only for an explicitly idempotent tool; destructive, privileged and remote risk classes reject automatic retries. These fields do not themselves trigger retries; the future executor must honor the declared policy and recheck safety before each attempt.
- M7-006 workflow history persists only bounded run/step metadata and fixed closed status/error values. Successful output is always replaced by the literal `[REDACTED]`; output values and inputs are not written. History retention is limited to 500 runs and list responses to 200 entries. Renderer-facing native commands are read-only and accept a validated UUID for detail lookup; no execution or workflow-write IPC is exposed.
- M7-007 schedule records are local, bounded and disabled by default. Only explicit enablement creates a due time; cadence is limited to 5 minutes–30 days. The due-claim operation filters on enabled state and pause clears the due time. No startup/background timer or automatic execution is connected, and a future executor must recheck enabled state and all workflow permissions immediately before running.
- M7-008 AI configuration is strict, versioned and disabled by default. It stores only an opaque credential reference, never a secret value; remote endpoints must use HTTPS and cannot contain credentials, query parameters or fragments. Request and response text are size-bounded, provider errors are closed enums, and no production provider or network call is implemented. A future remote adapter must apply endpoint/network scope policy and use the OS credential boundary.
- M7-009 treats structured model output as untrusted JSON. The parser rejects unknown fields, unsupported schema versions, invalid tool IDs, oversized documents, excessive argument counts and nested values. Parsing produces inert request data and cannot invoke the registry; registry matching and policy checks remain independent mandatory steps.
- M7-010 gives AI only the bounded catalog of enabled registered tools. A proposed ID must match an enabled registry entry exactly; invented and disabled IDs are rejected. Discovery returns an inert recommendation and does not dispatch tools or grant capability/approval. The production registry and approval path remain separate.
- M7-011 preview approval binds the exact registered tool/API version, target, arguments and preview ID. Approval requires the displayed preview ID, and verification recomputes the SHA-256 binding before accepting it; changed arguments, target or version fail closed. Approval data is not deserializable from model/renderer data, is not persisted, and does not itself execute or authorize an action.
- M7-012 summaries are parsed into claims with explicit `supported` or `unknown` status. Supported claims require known evidence item IDs, and display URLs/source IDs are resolved from validated local evidence records rather than model-supplied references. Unknown claims cannot carry citations; invented evidence IDs and uncited supported claims fail closed.
- M7-013 uses only synthetic prompt-injection and credential fixtures. A request to invent a secret-export tool fails exact registry matching; the synthetic secret is not included in the provider request context and a rejected response is not returned. This tests local boundaries only; no production model/provider is connected.

## 8. Secrets and privacy

- Use Windows Credential Manager or a reviewed OS credential abstraction for secrets.
- Store non-secret preferences in SQLite; store references/identifiers, not secret values.
- Redact Authorization headers, cookies, tokens, passwords and query parameters known to contain secrets.
- Never attach secrets automatically to tasks, logs, reports, model prompts or bug reports.
- Network providers must disclose what is sent and why.
- Make background fetch intervals visible and configurable.
- Export excludes secrets by default.
- M2-029 Project/Task export is initiated by a visible user action and uses a native Save dialog; the renderer never supplies a filesystem path or writes bytes. Native code derives the Personal workspace, emits only the versioned Project/Task/activity schema, excludes app settings, metadata, logs, credentials and restricted activity, validates the serialized document, and writes only to the path returned by that dialog. The export includes user-authored descriptions, so the UI tells users to review content before sharing. Cancellation saves nothing; an I/O error is reported as a failure, never as success.
- M2-030 import preview is initiated by a visible user action and uses a native Open dialog; the renderer never supplies a filesystem path or file bytes. Native code reads at most 50 MiB, rejects malformed UTF-8/JSON, unsupported schemas, invalid records and documents above 100,000 records, and reports only a bounded summary. Preview does not access the database or write files. Imported text is rendered as text and never interpreted as markup or executable content. There is no import/commit action in this task.
- M2-033 Home uses a no-request native read-only command scoped to the Personal workspace. Parameterized SQL returns counts and bounded previews only; restricted Activity is excluded. No user-controlled SQL, renderer-provided workspace scope, background polling, telemetry or writes are involved. Project/Task titles and Activity summaries are treated as text in the WebView.
- M2-034 global search reuses native active Project and Task list commands scoped to the Personal workspace plus the static registered-command allowlist. The renderer filters the returned in-memory records; it cannot supply SQL, workspace scope, paths or executable instructions. Search result text is rendered as text, record targets are URL-encoded IDs passed through existing detail routes, and selecting a command still resolves its exact registered ID. Reads are initiated only while the user opens the palette; each source failure is surfaced and retryable.
- Local database backup can still contain sensitive data; document this and offer encryption or user-controlled encrypted storage if needed.
- Do not add analytics/telemetry that transmits usage data by default.

## 9. Updates, plugins and supply chain

- Verify updater signatures before installation and keep signing keys out of repository/build artifacts.
- Do not execute downloaded code without explicit trust and verification.
- Built-in modules first; no arbitrary native in-process plugin loader for MVP.
- Third-party extension design requires manifest validation, version compatibility, permissions, source provenance and isolation.
- Record dependency versions and licenses; run vulnerability checks where tooling supports them.
- Use release provenance/SBOM as the project matures.
- Review third-party licenses before copying source, code, assets or branding.

For adapter, provider, external source, third-party code/package or asset changes, the PR must include the record in [INTEGRATION_REVIEW_CHECKLIST.md](INTEGRATION_REVIEW_CHECKLIST.md). The author supplies source/version and evidence; a reviewer/date and exactly one outcome (`approved`, `blocked`, or `not applicable`) are required. `not applicable` needs a reason; blank or `blocked` means the change is incomplete and the adapter must remain unavailable. Review covers provenance/licensing, permitted access and purpose, attribution, limits, caching/retention, redistribution, privacy, revocation/failure, notices and assets as relevant. Re-review material changes. This records an engineering decision and evidence at review time; it is not legal advice or a permanent rights grant.

## 10. Security verification checklist

- [ ] Every Tauri command has input validation and an explicit capability/policy.
- [ ] No command allows arbitrary filesystem deletion from a renderer-supplied path.
- [ ] Shell execution uses explicit allowlisted executable/argument interfaces or is not implemented.
- [ ] Symlinks, junctions, traversal, long paths and race conditions are tested.
- [ ] Secret redaction tests cover logs, exports, errors and AI payloads.
- [ ] Import/export rejects traversal, oversized files and malformed schemas.
- [ ] Approval is invalidated after target/arguments/tool-version change.
- [ ] AI prompt-injection fixtures cannot expand tool permissions.
- [ ] Scope enforcement tests include IPv4/IPv6, hostnames, redirects, DNS changes and out-of-scope cases.
- [ ] External-source rate limits, timeouts and freshness states are tested.
- [ ] Update signature failure is a hard stop.
- [ ] Data migrations and restore procedures are tested against copies.
- [ ] Security review follows an appropriate subset of OWASP ASVS and desktop-specific threat analysis.
