# Moonships Workbench — Implementation Checklist

This file is the short entry point. The detailed, atomic task source of truth is [docs/TASK_BACKLOG.md](docs/TASK_BACKLOG.md). Execute tasks in dependency order; do not mark a box complete without evidence.

## Before implementation
- [ ] Read [AGENTS.md](AGENTS.md) and [AI Agent Guide](docs/AI_AGENT_GUIDE.md).
- [ ] Review [Product Blueprint](docs/PRODUCT_BLUEPRINT.md), [UX & Design](docs/UX_AND_DESIGN.md), [Architecture](docs/ARCHITECTURE.md), [Data Model](docs/DATA_MODEL.md) and [Security Model](docs/SECURITY_MODEL.md).
- [ ] Verify all internal documentation links and reconcile any conflict.
- [ ] Pick exactly one unblocked task from [Task Backlog](docs/TASK_BACKLOG.md).

## Foundation gate (M1)
- [ ] Bootstrap Tauri 2 + React + TypeScript + Vite using current official template.
- [ ] Establish reproducible local setup instructions.
- [ ] Create design tokens and application shell.
- [ ] Implement navigation and not-implemented module states.
- [ ] Implement tray hide/show/quit behavior and settings.
- [ ] Add local data directory and SQLite migration baseline.
- [ ] Add typed command/error boundary and tool registry contract.
- [ ] Add CI formatting, lint, tests and Windows build.

## First usable product (M2–M3)
- [ ] Implement Projects and Tasks before expanding specialist modules.
- [ ] Add task list and Kanban with stable status/priority semantics.
- [ ] Add activity/history and artifact/reference links.
- [ ] Implement safe Cleaner read-only scan, preview, explicit approval, revalidation and report.
- [ ] Implement offline DevTools utilities with tests.

## Expand only after the foundation passes
- [ ] NetOps / SysOps / Connections.
- [ ] Source connectors / World Monitor / Signals & Trends.
- [ ] API/DataLab / Observe / authorized Security Lab.
- [ ] Deterministic workflows first, optional AI second, MCP adapter later.
- [ ] Signed updates, upgrade tests and stable-release gate.

## Release hygiene
- [ ] No feature is described as implemented before it is tested.
- [ ] No destructive operation is directly exposed as an arbitrary frontend path or model-generated command.
- [ ] No credentials are stored/logged/exported in plaintext.
- [ ] No source is polled without documented access rules and rate limits.
- [ ] All feature screens handle loading, empty, partial, stale, offline, denied, timeout, cancellation and failure states as appropriate.
- [ ] Every completed task has a test or explicit, reviewed reason why a test is not applicable.
