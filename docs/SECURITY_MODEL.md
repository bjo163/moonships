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

### Bounded network diagnostics (M4-001, M4-002)

- A request names one DNS hostname/IP or one TCP hostname/IP plus one explicit port. A URL, embedded port, wildcard, CIDR or target range is rejected; the UI must show the exact target and requested operation before starting a future adapter.
- Hostnames are ASCII DNS labels (IDNs must be punycode); literal IPv4 and IPv6 are supported. User whitespace is not silently trimmed. A target is never accepted from page content, model output or a redirect.
- Timeout is 100–10,000 ms, with a 2,000 ms default. Adapters enforce one concurrent diagnostic and at most ten runs per minute; DNS output is capped at 32 IP addresses. No automatic retries or range expansion are allowed.
- The DNS adapter performs read-only resolution through the OS resolver on one bounded worker and reports timeout to the caller. The OS resolver call cannot be forcibly cancelled; a stuck call occupies the single worker, and the adapter never creates replacement workers. IP literals are returned directly without a resolver request.
- Result provenance identifies the exact target/port, operation, UTC observation time as epoch milliseconds, elapsed time, IP-literal source, OS resolver or TCP socket, outcome, stable error code and bounded resolved addresses. Invalid input is a validation error; resolver failures are distinct typed results. No executable IPC command is registered by M4-002.

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
- Keep user instructions/policy separate from retrieved content in prompt construction.
- Do not automatically execute shell commands copied from sources or generated by models.
- Require a human review step for consequential action.
- Store only necessary AI traces; redact tokens, personal data and local paths where possible.
- Provide local-only/no-AI mode.
- If a model is unavailable or returns invalid output, fail closed for that AI-dependent operation while preserving ordinary non-AI tools.

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
