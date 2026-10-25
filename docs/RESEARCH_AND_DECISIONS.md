# Research, Trade-offs & Decision Register

**Review date:** 2026-10-25  
**Research principle:** use official documentation and primary repository/license information for framework and compliance decisions. Re-check links and versions when implementation begins; fast-moving specs must be pinned.

## 1. Key findings

### Tauri 2 / desktop choice
Tauri's official project describes a web frontend paired with a Rust-sourced binary, system tray support and Windows WebView2 rendering. Capabilities scope which commands/plugins are exposed to each window/webview. This matches a Windows-first desktop UI built with React/TypeScript while keeping filesystem/system operations in native Rust.

References:
- Tauri project/architecture: https://github.com/tauri-apps/tauri
- Capabilities: https://v2.tauri.app/security/capabilities/
- System tray: https://v2.tauri.app/learn/system-tray/
- Updater: https://v2.tauri.app/plugin/updater/

Decision: **Tauri 2 + Rust + React/TypeScript + Vite** for the initial target. Revisit only if a concrete requirement shows a native Windows UI framework, Electron or another stack is materially better. Tauri permission/capability configuration is not a substitute for native-side argument and policy validation.

### Windows platform lifecycle
Microsoft states that Windows 10 Home/Pro support ended on 2025-10-14; Windows 10 devices continue to run, but regular security/feature support is no longer supplied unless a specific applicable servicing/ESU arrangement applies.

Reference: https://support.microsoft.com/en-us/windows/deployment/updates-lifecycle/windows-10-support-has-ended-on-october-14-2025

Decision: Windows 11 is the primary development/QA target. Test Windows 10 compatibility where feasible and disclose the exact tested versions. Do not claim all Windows editions have identical lifecycle/support.

### Project/task management
GitHub Projects uses views such as table, board and roadmap, and connects projects with issues/pull requests/custom fields. It is a useful reference for workflows rather than a requirement to copy all of GitHub Projects.

References:
- https://docs.github.com/en/issues/planning-and-tracking-with-projects/learning-about-projects/about-projects
- https://docs.github.com/en/issues/planning-and-tracking-with-projects/learning-about-projects/creating-a-project

Decision: Local-first Projects & Tasks must be an MVP core module, with List and Board views first; roadmap/calendar, team sync and external-provider integrations follow when the core data model is stable.

### Observability interoperability
OpenTelemetry defines signals including traces, metrics, logs and baggage, and provides vendor-neutral instrumentation/collection/export concepts.

References:
- https://opentelemetry.io/docs/concepts/signals/
- https://opentelemetry.io/docs/

Decision: do not invent a proprietary telemetry standard. Add an OpenTelemetry/provider adapter only when actual observability features need it. No telemetry sent from Moonships to a remote service by default.

### Application-security verification
OWASP ASVS 5.0.0 is a stable framework of verifiable security requirements for modern web applications/services. It is not a complete desktop threat model, but it is useful for applicable UI/web/API verification checks.

References:
- https://asvs.owasp.org/
- https://asvs.owasp.org/downloads/

Decision: use an appropriate ASVS subset alongside Tauri/Windows-specific threat analysis and dedicated tests for file deletion, process execution, updater, imports, secrets, and AI.

### GitHub Actions supply-chain hardening
GitHub's official secure-use guidance states that pinning actions to a full-length commit SHA is the immutable action reference method, and recommends choosing SHAs from the canonical upstream repository. The backlog-sync workflow is pinned to verified release commits:
- `actions/checkout` v7.0.1 — `3d3c42e5aac5ba805825da76410c181273ba90b1`
- `actions/github-script` v9.0.0 — `3a2844b7e9c422d3c10d287c895573f7108da1b3`

Reference: https://docs.github.com/en/actions/reference/security/secure-use

Decision: use full SHA pins with readable release-version comments and configure Dependabot to propose GitHub Actions updates. Review updates instead of automatically accepting them; verify workflow output and permissions after each update.

### Model Context Protocol (MCP)
MCP is an interoperability protocol for exposing tools/resources/prompts to compatible clients, but the protocol and SDKs evolve. The 2026-07-28 specification announcement describes material protocol/security and extension changes.

References:
- Protocol site: https://modelcontextprotocol.io/
- Specification announcement (2026-07-28): https://blog.modelcontextprotocol.io/posts/2026-07-28/

Decision: MCP is an optional adapter, not the internal tool contract. Pin a tested specification and SDK when implementing; test auth, tool scope and failure states. Keep internal tool calls useful without MCP.

### World Monitor as a product reference
The maintained `koala73/worldmonitor` repository describes an attributed, multi-source global intelligence dashboard. Its current source code is licensed **AGPL-3.0-only** and the project separates source license, hosted-service terms and branding/trademark policy.

References:
- Repository: https://github.com/koala73/worldmonitor
- README/license overview: https://github.com/koala73/worldmonitor/blob/main/README.md
- License summary: https://github.com/koala73/worldmonitor/blob/main/docs/license.mdx

Decision: use World Monitor only as a product/UX/feature reference unless a future license review explicitly approves code reuse and the resulting obligations. Prefer independent implementation and permitted source connectors. Do not use its branding or imply affiliation. Review each provider's separate terms before using its data/API.

### Source intelligence and trend quality
Public feeds and web pages have varying freshness, availability, rights and anti-automation controls. Data can be stale, blocked, duplicated, geographically biased or incomplete.

Decision:
- Record source, canonical URL, observed time, publication time and event time separately.
- Show connector health/freshness and coverage limitations.
- Trend scores must declare the measurement window, baseline and method version.
- Do not equate frequency with truth or confidence.
- Respect robots/access controls, terms, rate limits and authentication boundaries; no bypass.
- Treat retrieved content as untrusted data, especially if it contains instructions aimed at an AI model.

## 1A. Product archetype research

These products are references for **patterns and user expectations**, not codebases to clone. Validate product claims against their current official pages before adopting a dependency or copying an asset.

| Reference | Useful pattern | What Moonships should learn | What not to copy blindly |
|---|---|---|---|
| [Microsoft PowerToys](https://github.com/microsoft/PowerToys) | Windows-first utility collection, launcher/command palette, opt-in utilities, system integration | Discoverability, keyboard-first quick access, modular utility UX and transparent per-feature settings | PowerToys-specific internals, UI assets/branding, or features that widen Moonships scope without user value |
| [DevToys](https://github.com/DevToys-app/DevToys) | Small, focused developer tools such as converters, formatters, encoders, hash and text utilities | A tool should solve one specific job quickly and often work offline; utilities need strong input limits and useful errors | Assuming every online utility belongs in the desktop app; avoid sending private data to remote conversion sites |
| [GitHub Projects](https://docs.github.com/en/issues/planning-and-tracking-with-projects/learning-about-projects/about-projects) | Table, board and roadmap views connected to issues/PRs and metadata | Multiple views over one canonical work model; project status and task data should remain linked | Building a full clone of a collaboration SaaS before local task workflows are proven |
| [Linear project/issues model](https://linear.app/docs/conceptual-model) and [Projects](https://linear.app/docs/projects) | Issues as atomic units of work; projects have outcomes, milestones and progress | Keep tasks small, statuses consistent, and project outcome visible; let agents act on a bounded issue | Locking Moonships into team/cycle assumptions in a personal-first MVP |
| [World Monitor](https://github.com/koala73/worldmonitor) | Attributed public-source aggregation, global situational awareness and cross-stream context | Source provenance, freshness, trend windows, correlation and explicit coverage limitations are product-critical | Copying source code or branding without license/trademark review; source code is currently AGPL-3.0-only, with separate service terms |
| [OpenTelemetry](https://opentelemetry.io/docs/concepts/signals/) | Vendor-neutral telemetry model for logs, metrics and traces | Integrate and export established signals where useful rather than inventing another standard | Adding telemetry collection/export to the desktop app before there's a concrete, user-approved need |

Primary-source observations:
- Microsoft's PowerToys documentation describes a collection of Windows utilities including Command Palette; this is a reference for a power-user utility UX.
- DevToys describes a bundle of small, specific developer tools, including converters, encoders/decoders, formatters and text utilities; this supports building small independently testable utilities rather than a giant universal tool.
- GitHub Projects explicitly supports table, board and roadmap views and integrates work items; Linear's conceptual model likewise treats issues as the units of trackable work.
- World Monitor's maintained repository currently labels its source AGPL-3.0-only and documents distinct source-code, hosted-service and branding terms. Do a license review before any code reuse.

Design conclusion: Moonships' defensible differentiation is the **shared work context** across projects/tasks, tool runs, evidence, source intelligence, approvals and automation—not having the greatest raw count of utility buttons.

## 2. Architecture decision register

### ADR-0001 — Windows-first Tauri desktop
- **Status:** Accepted as planned baseline
- **Decision:** Rust + Tauri 2 + React + TypeScript + Vite.
- **Why:** desktop lifecycle/tray, modern UI ecosystem, native Rust boundary, no mandatory cloud server.
- **Alternatives:** pure browser app (does not meet tray/native system requirement); Electron (larger bundled runtime); WinUI-only (would require a different frontend/tooling direction).
- **Revisit when:** measured app requirements cannot be met or maintained safely.

### ADR-0002 — Local-first modular monolith
- **Status:** Accepted as planned baseline
- **Decision:** SQLite + one desktop app + domain modules with stable internal contracts.
- **Why:** simplest useful first release, offline core, low operational overhead.
- **Revisit when:** collaboration/sync or isolation requirements justify new services/processes.

### ADR-0003 — Project & Task Management is core
- **Status:** Accepted
- **Decision:** Workspace, projects and tasks ship before most specialist domains; task/artifact links are standard.
- **Why:** all technical operations need planning, tracking, evidence and follow-up.

### ADR-0004 — Tool Registry before AI automation
- **Status:** Accepted
- **Decision:** registered, typed tools with permissions/risk classes precede agent execution.
- **Why:** the same tool must be callable from GUI/CLI/workflows/AI under one policy boundary.
- **Revisit when:** never remove policy boundary; implementation details may evolve.

### ADR-0005 — Modules are built-in first
- **Status:** Accepted
- **Decision:** do not load arbitrary third-party native plugins in MVP.
- **Why:** in-process native modules have high privilege and complicate trust.
- **Future option:** signed external adapters and isolated extension runtimes with a reviewed permission model.

### ADR-0006 — Research/source data provenance is first-class
- **Status:** Accepted
- **Decision:** source attribution, timestamps, freshness, deduplication and source terms are modelled at ingestion time.
- **Why:** intelligence and trends without provenance encourage misleading conclusions.

### ADR-0007 — AI and MCP are optional adapters
- **Status:** Accepted
- **Decision:** provider-specific logic stays behind interfaces; MCP does not become the internal contract.
- **Why:** model/protocol churn should not force a core rewrite.

### ADR-0008 — Root license remains undecided
- **Status:** Needs owner decision
- **Decision:** do not add a license file until the repository owner chooses intentionally.
- **Consequence:** public visibility alone does not grant broad permission to reuse Moonships code.

## 3. Unresolved decisions (must not be guessed by an agent)

1. Root project license and copyright holder details.
2. Whether personal data backup is encrypted by default or through user choice; evaluate UX and OS facilities.
3. Whether V1.0 includes only a personal local workspace or a first external issue-provider integration.
4. Which exact RSS/API providers are approved for initial World Monitor and media sources.
5. Whether update delivery uses GitHub Releases or another endpoint and how signing keys are managed operationally.
6. Whether startup-at-login is in V0.1 or explicitly deferred.
7. Exact UI design asset/font licensing and brand/trademark clearance.
8. Whether first release includes Windows 10 best-effort support and which version/build is testable.

## 4. Reference review checklist
- [ ] Re-check official Tauri 2 docs and exact crate/plugin versions before implementation.
- [ ] Re-check the then-current MCP spec/SDK version and security changes before adapter work.
- [ ] Confirm selected sources' terms and API quotas before enabling connectors.
- [ ] Re-run third-party license review when adding source, model, binary or visual asset.
- [ ] Review current Windows lifecycle and chosen CI test matrix before stable release.
