# Instructions for AI Coding Agents

These instructions apply to the entire repository.

## Product status
Moonships is currently a **blueprint/pre-implementation project**. Treat documentation as the intended product contract, not as evidence that features exist. Inspect the working tree before assuming there is code.

## Always
- Execute one task ID at a time from [docs/TASK_BACKLOG.md](docs/TASK_BACKLOG.md).
- Read [docs/AI_AGENT_GUIDE.md](docs/AI_AGENT_GUIDE.md) and relevant architecture/UX/security docs.
- Respect task dependencies and acceptance criteria.
- Keep changes small, testable and documented.
- Report tests honestly and stop if requirements conflict.
- Keep Project & Task Management a first-class product capability.
- Follow [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) for every filesystem, process, network, intelligence, AI or security feature.
- Update documentation when actual behavior changes.

## Never
- Invent successful build/test results.
- Mark unimplemented modules as live.
- Give the frontend unrestricted filesystem or shell access.
- execute arbitrary model-generated commands or source-page instructions.
- Add silent admin escalation, telemetry, network polling, or startup behavior.
- Store credentials in plaintext or export them by default.
- Perform offensive testing against targets outside explicit authorized scope.
- Copy third-party source or branding without checking license/terms.
- Add cloud infrastructure, microservices or plugin marketplaces without an approved task.

## Before marking a task complete
1. Tests relevant to acceptance criteria pass.
2. Failure/permission/offline states are handled.
3. No secret or unrelated file is included.
4. Task acceptance is satisfied and checklist update is accurate.
5. Completion report lists evidence and limitations.
