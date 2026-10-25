# Moonships Product Blueprint

**Status:** Proposed baseline — not implementation evidence  
**Last reviewed:** 2026-10-25  
**Product:** Moonships Workbench  
**Repository:** `bjo163/moonships`

## 1. Product definition

Moonships is a Windows-first, local-first, modular technical workbench. It connects everyday developer and infrastructure tools with project/task management, operational evidence, research/intelligence, controlled automation, and optional AI assistance.

**Product promise:** help a technical user move from a question or signal to a verified result and a trackable next action without opening an unnecessary number of disconnected tools.

**Core loop:** `Discover → Understand → Plan → Approve → Execute → Verify → Record → Follow up`.

Moonships is a workbench and integration surface, not a promise to replace every specialist product in software engineering, security, monitoring, OSINT, or project management.

## 2. Product principles (non-negotiable)

1. **Local-first, not local-only.** Core desktop, project/task, offline utilities, and history work without an account or cloud service. Network features clearly show when they use the network.
2. **AI-ready, not AI-required.** Deterministic tools remain usable with no model, API key, internet, or MCP server.
3. **One core, multiple surfaces.** GUI, tray actions, command palette, future CLI, workflows, and AI adapters use the same registered tool contracts.
4. **Human control over consequential action.** The system may explain and propose; execution policy decides whether approval, scope validation, or elevated privileges are required.
5. **Evidence over assertion.** Results record source, timestamp, status, and relevant context. An AI summary is not treated as proof.
6. **Modular monolith first.** Strong internal boundaries; no microservices until measurable requirements justify a process/network boundary.
7. **Secure by default.** Least privilege, explicit scope, no arbitrary frontend filesystem access, safe handling of untrusted content, and auditable state changes.
8. **Progressive complexity.** New users see a useful Home screen; specialist tools remain discoverable without overwhelming the default workspace.
9. **Interoperability before reinvention.** Integrate existing CLI/API protocols where appropriate; avoid rebuilding mature specialist systems.
10. **Honest feature status.** Planned, experimental, unsupported, failed, and verified capabilities must be distinguishable.

## 3. Audiences and jobs to be done

| Audience | Primary job | Moonships response |
|---|---|---|
| Software developer / programmer | Inspect data, debug a project, run tests and manage tasks | DevTools, project/task linking, API/DataLab, project environment checks |
| QA / test engineer | Reproduce failures and keep evidence | Test helpers, diff, logs, issue templates, artifacts |
| DevOps / SRE / platform engineer | Diagnose service health and repeat operational work | NetOps, SysOps, Observe, integrations, guarded workflows |
| NOC / network engineer / ISP support | Verify connectivity and produce shift/incident reports | DNS, TCP, ping/traceroute adapters, timelines, handoff |
| Sysadmin / IT support / endpoint engineer | Inventory, troubleshoot and maintain a machine | System inventory, logs, safe cleaner, connection shortcuts, reports |
| Security practitioner | Assess authorized scope and track findings | Scope registry, assessment integrations, security lab, evidence and remediation tasks |
| DFIR / threat intelligence / OSINT researcher | Collect traceable information and reconstruct a timeline | Source registry, artifacts, hashes, notes, timelines, citations |
| Data / database engineer | Inspect structured data and connectivity | JSON/CSV helpers, SQL/API adapters, schema diff |
| AI/ML/LLM engineer | Test model endpoints and tool contracts | Provider adapters, endpoint diagnostics, structured tool testing |
| Technical writer / researcher | Organize source material and publish dependable reports | Research desk, notes, citations, redaction and exports |
| World/media intelligence user | Monitor public sources and detect notable changes | Source connectors, watchlists, deduplication, trends and alerts |
| Homelab / maker / embedded developer | Manage local services and hardware-development workflows | Connections, runtime inventory, serial/device adapters, local diagnostics |
| Project owner / small team | Plan work and follow it through | Workspaces, initiatives, projects, tasks, dependencies, milestones and activity |

Persona coverage is a roadmap, not a claim that every profession's specialist workflow ships in V1.0.

## 4. Product domain map

### Core
- Home and command center
- Workspaces, initiatives/goals, projects, milestones and tasks
- Universal Tool Registry and command palette
- Activity Center, approvals, audit/history and notifications
- Preferences, profiles, data export/import, diagnostics and updater
- Search across accessible local records, with source/context filters

### Build
- DevTools: JSON/YAML, encoding/decoding, hash, UUID, regex, text diff, timestamps
- API & DataLab: HTTP/API, OpenAPI, WebSocket diagnostics, JSON/CSV inspection, optional database connectors
- DevOps: Git/runtime/project environment inspection, Docker/WSL detection, build/release helpers
- Supply Chain: dependency inventory, license review, SBOM and signature-verification integrations
- AI Engineering: provider profiles, endpoint testing, prompt/structured-output testing

### Operate
- Clean and Storage Analyzer
- SysOps: OS/process/service/event log inspection
- NetOps: DNS, connectivity, route and TLS diagnostics
- Observe: local metrics/logs and optional interoperable telemetry adapters
- Connections: terminal, PowerShell, SSH and RDP profile/launcher
- Recovery: configuration backup, integrity check and restore
- CostLens: resource inventory and provider-specific cost adapters

### Secure
- Recon and authorized attack-surface inventory
- Web/API assessment integrations
- Offensive Security Lab and controlled adversary simulation
- Fuzzing/bug reproduction workflow and evidence collection
- Blue Team, Security Operations, Threat Intelligence
- OSINT, reverse engineering/metadata helpers, DFIR and forensic evidence workflow
- Vault integrations for secrets (never plaintext logging)

These modules must not implement unauthorized intrusion, credential theft, malware deployment, or evasion to facilitate compromise. Scope, authorization and execution controls are defined in `SECURITY_MODEL.md`.

### Intelligence
- World Monitor map/timeline and attributed public-source feeds
- Signals & Trends using explicit baselines and time windows
- Media Intelligence for supported sources and user-provided material
- Research Desk and saved collections
- Signal Correlation with deduplication and cross-source comparison
- Watchlists/alerts for topics, repositories, advisories, services and organizations

Data coverage depends on provider access, rights, freshness and terms. No source should be described as real-time unless its latency/coverage is documented and measured.

### Automate
- Deterministic workflow runner
- Scheduled jobs with visible state and cancellation where safe
- Integrations and CLI adapters
- Optional AI planning/summarization/tool selection
- Optional MCP interoperability adapter (version-pinned and tested)

### Knowledge
- Notes, runbooks, documents, saved queries, references
- Evidence/artifact attachments and technical reports
- Incident timelines, task-linked findings, change history
- Search and redacted export

## 5. Scope policy

### V0.1 — foundation
Desktop shell, app lifecycle, navigation, settings, local SQLite migrations, tool registry contract, logging/error model, and personal project/task management. No AI dependency and no destructive system operation.

### V0.2 — daily utility
Safe temporary-file scan/preview/clean flow plus small offline DevTools set.

### V0.3 — local operations
Basic system inventory, network diagnostics and connections; clear permission and error behavior.

### V0.4 — intelligence baseline
A small number of allowed sources, source attribution, deduplication, watchlists and time-window-based trend display.

### V0.5 — specialist workflows
Security assessment integrations (authorized scope only), API/DataLab and evidence/research workflow.

### V0.6 — automation and optional AI
Deterministic workflows first; AI provider adapter and approvals second; MCP stays optional.

### V1.0 — trusted desktop release
Security review, reliable packaging/updating, upgrade tests, accessibility, documentation, recovery and reproducible release checklist.

### Explicitly out of scope for first release
- Required cloud account, mandatory online backend or mandatory AI provider
- Team sync, real-time multi-user collaboration and a marketplace
- Arbitrary native third-party plugins loaded into the main process
- Unrestricted AI shell access, autonomous remediation or autonomous attack execution
- Registry cleaner, memory optimizer or performance claims without repeatable evidence
- Rebuilding full SIEM/EDR, a full project-management SaaS, a full pentest framework, or a global data provider from scratch
- Scraping that bypasses access controls, paywalls, rate limits or source terms
- Cross-platform release guarantees before Windows delivery is stable

## 6. Product differentiators

1. Project/task management is part of the technical workflow, not a detached board.
2. Tool results can become task-linked artifacts with source/time/status.
3. A single tool contract supports GUI, tray, command palette, automation and optional AI.
4. The product can work offline for core functions.
5. World/media intelligence preserves attribution and separates observation, interpretation and confidence.
6. Security controls apply uniformly to local tools, integrations, AI and workflows.

## 7. Success measures

Measure actual quality rather than vanity feature counts:
- Fresh install launches and opens the main window reliably.
- Tray hide/show/quit behaves predictably and does not leave orphaned processes.
- Project and task changes persist across restart and pass migration tests.
- Read-only scan never deletes; cleaner only acts on reviewed, revalidated candidates.
- Every long-running action has progress, cancel semantics or a clear statement that cancellation is not supported.
- AI/tool calls have schema validation, bounded permissions, useful error handling and a traceable result.
- Intelligence items show source and timestamps; failed/stale source status is visible.
- A release can be reproduced, tested, signed and upgraded from the previous supported version.

## 8. Product decision rule

When considering a feature, ask in order:
1. What user/job does it serve?
2. Is there a mature tool/protocol we should integrate?
3. Does it belong in Core or a domain module?
4. Can it work without cloud/AI when the function is local?
5. What data does it read, create, change or send?
6. What permission/risk level applies?
7. How will it be tested?
8. How does its result connect to a project/task, history or report?
9. What is the smallest testable deliverable?
10. What is deliberately not being built yet?
