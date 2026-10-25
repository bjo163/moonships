# MOONSHIPS WORKBENCH

> **The adaptive developer & infrastructure workbench.**  
> Build. Manage. Observe. Secure. Explore. Automate.

Moonships is planned as a **Windows-first desktop workbench** that brings technical utilities, project and task management, system/network diagnostics, authorized security testing, world/media intelligence, and controlled automation into one modular workspace.

**Project status: blueprint / pre-implementation.** This repository is being established as the product specification and execution source of truth. Features described here are planned, not claims of existing functionality.

<p align="center">
  <img alt="Project status" src="https://img.shields.io/badge/status-blueprint%20phase-64748b">
  <img alt="Primary platform" src="https://img.shields.io/badge/platform-Windows%2011%20primary-0078D4">
  <img alt="Planned core" src="https://img.shields.io/badge/core-Rust%20%2B%20Tauri%202-orange">
  <img alt="Frontend" src="https://img.shields.io/badge/UI-React%20%2B%20TypeScript-3178C6">
  <img alt="Architecture" src="https://img.shields.io/badge/architecture-modular%20monolith-6f42c1">
</p>

---

## Product thesis

Moonships should not become a pile of unrelated utilities. It should connect a user's work from **goal → project/task → tool execution → evidence/result → follow-up action**.

A diagnostic result can become a task. A security finding can become a tracked remediation. A World Monitor signal can become a research note. An automation can run the same registered tool that a user calls from the GUI or command palette. AI may help plan or explain the work, but the same core tools must remain useful without an AI provider.

## Who Moonships is for

- Software developers, programmers, QA/test engineers, architects, technical writers.
- DevOps, SRE, platform, cloud, release, database and data engineers.
- Network engineers, NOC/SOC analysts, sysadmins, helpdesk and IT operations.
- Authorized penetration testers, ethical hackers, red/purple teams, bug bounty researchers, reverse engineers, incident responders, DFIR and OSINT researchers.
- AI/ML/LLM engineers, embedded/firmware developers, homelab operators and technical researchers.
- People who monitor world events, news, media, technology trends and public-source intelligence.

Coverage grows by adding tested modules and integrations. The project must not claim to replace specialist tools that have not been implemented and validated.

## Product domains

| Domain | Purpose | Examples |
|---|---|---|
| **Core** | Shared product foundation | Workspace, projects/tasks, command palette, registry, permissions, history, updates |
| **Build** | Create and validate software/data | DevTools, API/DataLab, Git/runtime checks, dependency and SBOM workflows |
| **Operate** | Diagnose and maintain systems | Cleaner, Storage, SysOps, NetOps, Observe, Connections, Recovery, CostLens |
| **Secure** | Test and protect authorized environments | Recon, Web/API assessment, Security Lab, Blue Team, Threat Intel, DFIR, OSINT |
| **Intelligence** | Monitor and understand public information | World Monitor, Signals & Trends, Media Intelligence, Research Desk, Watchlists |
| **Automate** | Compose repeatable work | Workflow engine, schedules, integrations, AI gateway, optional MCP adapters |
| **Knowledge** | Preserve operational context | Notes, runbooks, evidence, reports, references and activity history |

See [Product Blueprint](docs/PRODUCT_BLUEPRINT.md) for the detailed catalogue and scope. See [Project & Task Management](docs/PROJECT_TASK_MANAGEMENT.md) for the first-class work-tracking specification.

## Planned experience

- **Desktop GUI** powered by Tauri 2 and React/TypeScript.
- **System Tray** for quick access, status and notifications; closing the window must not accidentally quit the app.
- **Command Palette** for keyboard-first navigation and tool discovery.
- **Projects & Tasks** as a first-class feature, not an afterthought.
- **Activity Center** for running jobs, progress, results, errors and approvals.
- **Local-first operation** for core desktop functions; network access is explicit and feature-specific.
- **AI-ready design** through typed tool contracts and provider adapters, not hard coupling to one model/vendor.

Detailed menus, user flows and design rules live in [UX & Design](docs/UX_AND_DESIGN.md).

## Architecture (planned)

- **Desktop/runtime:** Rust + Tauri 2.
- **UI:** React + TypeScript + Vite.
- **Styles:** Tailwind CSS and a consistent, accessible component system.
- **Local persistence:** SQLite with explicit schema migrations.
- **Contracts:** typed command inputs/outputs, stable errors, events and versioned module manifests.
- **Integrations:** provider/CLI/API adapters behind bounded interfaces.
- **Tests and delivery:** Rust tests, UI tests, contract/security tests, CI, signed release updates.

Start with a **modular monolith**, not microservices. Split into separate crates/processes only when a boundary has proven value. Do not build cloud sync, a plugin marketplace, remote agents or multi-agent orchestration into the first release.

Read [Architecture](docs/ARCHITECTURE.md), [Data Model](docs/DATA_MODEL.md), [Project & Task Management](docs/PROJECT_TASK_MANAGEMENT.md), and [Security Model](docs/SECURITY_MODEL.md).

## Delivery plan

| Milestone | Outcome |
|---|---|
| **M0 — Product contract** | Product, UX, architecture, security, data model and acceptance rules agreed in docs |
| **M1 — Desktop foundation** | App shell, navigation, tray lifecycle, settings, database/migrations and CI |
| **M2 — Projects & Tasks** | Local project/task workflow, list/board views, search, links and history |
| **M3 — Daily tools** | Safe cleaner MVP plus offline DevTools |
| **M4 — System operations** | NetOps, SysOps, connections and activity center |
| **M5 — Intelligence** | Source adapters, watchlists, trend detection, timeline and attribution |
| **M6 — Specialist modules** | API/DataLab, observability, security lab and research workflows |
| **M7 — Automation & AI** | Deterministic workflows, AI provider interface, approval gates; MCP as optional adapter |
| **M8 — Stable release** | Security, accessibility, performance, upgrade, packaging and release checks |

The milestones and granular checkboxes are maintained in [Milestones](docs/MILESTONES.md) and [Task Backlog](docs/TASK_BACKLOG.md). An AI coding agent should execute one small task at a time and update the checklist only after verifying the acceptance criteria.

## Start here

1. Read [Product Blueprint](docs/PRODUCT_BLUEPRINT.md).
2. Review [UX & Design](docs/UX_AND_DESIGN.md).
3. Review [Architecture](docs/ARCHITECTURE.md) and [Data Model](docs/DATA_MODEL.md).
4. Understand the [Security Model](docs/SECURITY_MODEL.md).
5. Pick the first unchecked task in [Task Backlog](docs/TASK_BACKLOG.md), respecting its dependencies.
6. Follow [AI Agent Guide](docs/AI_AGENT_GUIDE.md) and root [AGENTS.md](AGENTS.md) before changing code.
7. Review [Research & Decisions](docs/RESEARCH_AND_DECISIONS.md) for licensing and unresolved owner decisions.

## Repository map

```text
moonships-workbench/
├── README.md
├── AGENTS.md
├── CONTRIBUTING.md
├── TODO.md
├── .editorconfig
├── .gitignore
├── .github/
│   ├── PULL_REQUEST_TEMPLATE.md
│   └── ISSUE_TEMPLATE/
└── docs/
    ├── PRODUCT_BLUEPRINT.md
    ├── PROJECT_TASK_MANAGEMENT.md
    ├── UX_AND_DESIGN.md
    ├── ARCHITECTURE.md
    ├── DATA_MODEL.md
    ├── SECURITY_MODEL.md
    ├── MILESTONES.md
    ├── TASK_BACKLOG.md
    ├── AI_AGENT_GUIDE.md
    └── RESEARCH_AND_DECISIONS.md
```

This is the **target documentation layout**. Each file is added as the blueprint is committed; actual implementation may later add or revise source folders.

## Research and design references

The architecture is informed by official and primary references, including:

- [Tauri 2 capabilities](https://v2.tauri.app/security/capabilities/) and [Tauri project](https://github.com/tauri-apps/tauri) for the desktop shell, WebView and permission boundaries.
- [Tauri system tray](https://v2.tauri.app/learn/system-tray/) and [updater plugin](https://v2.tauri.app/plugin/updater/) for desktop lifecycle and release strategy.
- [Model Context Protocol](https://modelcontextprotocol.io/) and the [2026-07-28 specification announcement](https://blog.modelcontextprotocol.io/posts/2026-07-28/) for an optional interoperable tool adapter; pin a tested version and review updates before adoption.
- [OpenTelemetry signals](https://opentelemetry.io/docs/concepts/signals/) for interoperable logs, metrics and traces.
- [OWASP ASVS](https://asvs.owasp.org/) for application-security verification practices.
- [GitHub Projects documentation](https://docs.github.com/en/issues/planning-and-tracking-with-projects/learning-about-projects/about-projects) for issue/project/board/roadmap concepts.
- [Microsoft Windows 10 support lifecycle](https://support.microsoft.com/en-us/windows/deployment/updates-lifecycle/windows-10-support-has-ended-on-october-14-2025) for platform support planning.
- [World Monitor repository](https://github.com/koala73/worldmonitor) as a product-reference example only. Its source code is AGPL-3.0-only; do not copy, embed, or adapt its implementation without a deliberate license review. Separate data-source terms and trademarks must also be respected.

See [Research & Decisions](docs/RESEARCH_AND_DECISIONS.md) for trade-offs, non-goals and architectural decision records.

## Development status

Nothing in this README implies that planned features are already implemented. Use the backlog's acceptance criteria and CI evidence before marking implementation items complete.

## License

No project license has been selected yet. **Do not assume Moonships is MIT, Apache-2.0, AGPL or otherwise open-source until a root `LICENSE` file is intentionally added.** Third-party dependencies and copied assets must be reviewed independently.
