# Coverage & Overlap Audit

**Status:** Checked against the product, UX, architecture, data model, security, milestone and task-backlog documents on 2026-10-25.

## 1. Gaps found and corrected in this audit

The prior backlog did not have atomic tasks for all visible core UX behaviors even though the UX spec mentioned them. The following tasks have now been added:
- **M1-031–M1-033:** Command Palette surface, safe command registration, Ctrl+K/focus behavior.
- **M1-034:** first-launch onboarding and workspace profiles.
- **M1-035:** idempotent task-backlog-to-GitHub-Issues sync.
- **M2-033:** real-data Home dashboard and honest empty states.
- **M2-034:** global search across currently implemented local entities.

The startup-at-login setting was ambiguous ("implement or defer"). It is now explicitly deferred for MVP; startup remains off by default. The app must not silently register itself to launch at login.

## 2. Design readiness follow-up

The detailed readiness findings are in [Design Readiness Audit](DESIGN_READINESS_AUDIT.md). The [Design System](DESIGN_SYSTEM.md) is the single source of truth for visual tokens/components/motion/accessibility/assets; [Audio & SFX Specification](AUDIO_SFX_SPEC.md) is the single source for sound events, defaults and audio assets. [UX & Design](UX_AND_DESIGN.md) retains information architecture, user journeys and wireframes only. Do not duplicate these specifications. M1-038–M1-045 cover implementation readiness for window behavior, shell visual/accessibility review, usability, components, assets, optional SFX and screenshot evidence; M2-035 covers separate Projects & Tasks usability after that vertical slice exists. These checks must not be marked complete from wireframes alone.

## 3. Intentional boundaries that may look overlapping

| Potential overlap | Canonical responsibility | Explicit non-overlap rule |
|---|---|---|
| Command Palette vs Global Search | Palette finds commands/navigation/actions; Global Search finds project/task/tool/source records | Palette must not duplicate entity search results as executable commands; search results do not auto-execute actions |
| Home vs My Work | Home summarizes projects, task counts and recent activity; My Work is the actionable task inbox/filter view | Home is a summary, not a second independent task store |
| Activity Center vs ActivityEvent vs logs | Activity Center is a job/run UI; ActivityEvent is business/entity history; structured logs are diagnostics | Never store debug log streams as task activity or use task activity as raw diagnostic log |
| Tool Registry vs module registry/plugin system | Tool Registry registers callable tool contracts and policy metadata; compiled-in modules provide implementations | Registry metadata is not executable code; dynamic native plugin loader is out of MVP |
| Project/task priority vs security severity vs execution risk | Priority ranks work; severity describes finding impact; risk class describes action consequence | Separate fields and UI labels; never share one enum across all three |
| NetOps vs Recon | NetOps performs bounded diagnostics to a user-selected endpoint; Recon coordinates authorized assessment of declared scope | NetOps does not silently fan out to a subnet; Recon enforces target scope and rate limits |
| Clean vs Recovery | Clean deletes only narrowly allowlisted, previewed temporary candidates; Recovery backs up/restores app/user-selected configuration where explicitly supported | A cleanup report is not a backup; never promise undelete without tested quarantine/recovery |
| Observe vs NOC/Operations Desk | Observe reads/normalizes health/log/metric data; operations workflows track incidents, handoff and next actions | Observability does not itself own the incident/task lifecycle |
| World Monitor vs Media Intelligence vs Research Desk | Connectors ingest source records; Media Intelligence normalizes supported media/transcripts; Research Desk preserves user-selected source context and analysis | Raw/source collection, interpretation and research conclusions are different records and statuses |
| Signals & Trends vs Signal Correlation | Trends measure a subject over an explicit time window/baseline; correlation links reports/events that may describe the same event | High frequency does not prove truth or causal connection |
| AI Assistant vs Workflow Engine | AI interprets request/proposes plan/summarizes; Workflow Engine executes declared deterministic steps | AI output is untrusted; every tool call is schema-validated and passes independent policy |
| Secure Vault vs App Settings | Vault stores secret material through OS-secure storage; settings store preferences and secret references | No token/password in settings JSON, project data, logs, exports or prompts |
| GitHub issue vs backlog checkbox | Backlog is canonical planned/completed task state; issue is its actionable discussion/execution mirror | Issue closure alone does not mark the checkbox; checked task closes generated issue on the next sync |

## 4. Coverage matrix

| Spec surface/capability | Backlog coverage | Result |
|---|---|---|
| Main window and navigation shell | M1-008–M1-010 | Covered in backlog; implementation open |
| Design readiness, minimum-window behavior and UI QA | Design Readiness Audit; M1-038–M1-039 and M2-035 | Newly tracked; implementation/validation open |
| Theme, keyboard accessibility, tray lifecycle, single instance | M1-011–M1-017 | Covered |
| Startup-at-login | M1-018 | Explicitly deferred and documented |
| Settings skeleton, notification/permission settings | M1-019, M1-030 | Covered for foundation; provider-specific pages later |
| Error codes, command DTOs, SQLite, migrations, logging | M1-020–M1-030 | Covered |
| Command Palette and onboarding | M1-031–M1-034 | Added in this audit |
| Backlog → GitHub issues | M1-035 + workflow/docs | Implemented and exercised: automatic import, stable markers, no-op sync and checked-task closure |
| Workflow supply-chain pinning | M1-036 + Dependabot config | Implemented; full upstream SHAs and weekly update PRs; main-branch sync run passed |
| Sync reconciliation regression tests | M1-037 | Open follow-up; fail-closed guards exist, but edge-case test coverage is not yet complete |
| Projects, tasks, list/board, search, milestones, history, import/export | M2-001–M2-032 | Covered for personal/local-first MVP |
| Home dashboard and global search | M2-033–M2-034 | Added in this audit |
| Cleaner safety and offline DevTools | M3-001–M3-023 | Covered for narrow MVP scope |
| NetOps, SysOps, connections, activity jobs | M4-001–M4-015 | Covered for initial diagnostics |
| Public source connectors, dedupe, trends, watchlists, freshness | M5-001–M5-018 | Covered for initial intelligence baseline |
| API/DataLab, Observe, authorized security, evidence/research | M6-001–M6-014 | Covered at initial integration/workflow level; individual providers are later tasks |
| Workflow/AI/MCP control plane | M7-001–M7-015 | Covered for a bounded first implementation |
| License, SBOM, updater, upgrade, recovery and UX release gates | M8-001–M8-017 | Covered as release gates |
| Full CostLens, full Vault UI, expanded media/video ingestion, dedicated reverse-engineering suite, team sync, plugin marketplace, remote agents | M9–M13 post-V1.0 milestones | Explicitly deferred into M9–M13 task batches; each still needs its own source/security review and acceptance tests before implementation |

## 5. Non-overlap invariants

- One canonical task status and priority model across UI views.
- One policy engine for native tools, workflows and AI-requested calls.
- One job/result contract with tool identity, timestamps, status and error codes.
- One source/provenance model for intelligence items.
- One local persistence authority for each record; adapters do not create shadow copies without a migration/retention plan.
- No feature should implement another module's private behavior. Call a declared contract or link a source result.
- A product domain may contain many modules, but a module needs one owner, one purpose, explicit inputs/outputs and defined permissions.

## 6. Remaining scope caveat

A catalog entry is not a shipped feature. Deferred specialist products now have task batches under M9–M13, but must not be described as supported until their source/provider terms, threat model, UX, isolation/permission design and acceptance tests pass.
