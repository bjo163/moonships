# Project & Task Management — Feature Specification

**Priority:** P0; part of the first usable Moonships release.  
**Mode:** Personal/local-first in MVP. Team collaboration and cloud sync are future integrations.

## 1. Purpose

Project & Task Management turns Moonships from a collection of utilities into a workbench that can carry work through to completion. Every module may create a task, attach evidence, add a finding, or record a follow-up without inventing its own project-management system.

Core work loop: `Goal → Project → Milestone → Task → Execution/Artifact → Review → Done`.

## 2. MVP user journeys

### Journey A — Create a project
1. Select Projects → New Project.
2. Enter a required name; optionally enter a goal, description and status.
3. Save.
4. Land on Project Overview.
5. Add a task or milestone.
Acceptance: project is persisted; validation is inline and accessible; cancel has no side effects.

### Journey B — Capture a quick task
1. Use “New Task” in Home, My Work or Command Palette.
2. Enter title (required).
3. Optionally choose project, description, priority, due date and labels.
4. Save to project or personal inbox.
5. Task appears in My Work and search.
Acceptance: task can be created with title only and exists after restart.

### Journey C — Move work through statuses
1. Open List or Board view.
2. Change the task status.
3. UI immediately reflects persisted state or presents a recoverable error.
4. Activity records the status transition.
Acceptance: List/Board share one status model; a failed save reverts or clearly indicates unsaved state.

### Journey D — Break work into AI-sized tasks
1. Create an epic/initiative/project or parent task.
2. Add small subtasks with one outcome each.
3. Set explicit dependencies and acceptance criteria.
4. Agent selects one unblocked task and records evidence.
Acceptance: a child task has its own ID, scope, status and completion evidence; parent completion does not silently mark children complete.

### Journey E — Convert a tool result to work
1. A tool finishes and creates a result/artifact reference.
2. User chooses “Create task from result”.
3. Moonships drafts title, summary, source/time and evidence reference.
4. User reviews and saves.
Acceptance: the result is not automatically a completed task; sensitive data is redacted by default.

### Journey F — Plan a milestone
1. Open project → Milestones.
2. Add title, optional target date and status.
3. Associate tasks or deliverables.
4. View the milestone timeline.
Acceptance: milestone is an organizational target; adding it does not silently change task states or deadlines.

## 3. Views

### My Work
- Inbox/unassigned tasks
- Assigned to me (if identity/assignee model exists)
- Today / Upcoming / Overdue
- Blocked
- Waiting for Approval
- Recently updated
- Completed recently

For local-only MVP, “me” is the active local profile. Do not invent organization/team identities.

### Project List
Columns: name, status, task count, open/blocked count, target date, updated at.
Actions: open, create, archive, search/filter/sort.

### Task List
Columns: title, status, priority, project, due date, labels, updated at.
Support sort/filter and keyboard navigation. Do not require inline editing before a robust detail form exists.

### Kanban Board
Groups use canonical task statuses. Drag/drop commits a state transition only once. If persistence fails, restore previous placement and show an error.

### Calendar
Future/optional for MVP; task dates must be modelled so calendar can be added without schema rewrite. Un-dated tasks remain visible in List/Board.

### Roadmap / Timeline
Milestone/project date view. It must disclose that dates are targets, not guarantees. Avoid a pseudo-precise critical path until dependencies and duration estimates are reliable.

### Saved Views
Future after filters are stable. Saved filter schema must be versioned.

## 4. Status, priority and completion

Canonical statuses: Backlog, Todo, In Progress, Blocked, In Review, Done, Cancelled.
Canonical priorities: P0 Critical, P1 High, P2 Normal, P3 Low.
Security severity (e.g. Critical/High/Medium/Low) and action risk (R0–R4) are separate fields and must not be reused as task priority.

A task should include acceptance criteria for work that is more than a quick note. Completion means:
- all mandatory acceptance criteria pass,
- relevant tests/evidence are attached or linked,
- any known limitation is disclosed,
- no forbidden side effect occurred.

For simple personal tasks, a minimal title-only flow stays easy. The app may encourage an acceptance field, but should not make every small task cumbersome.

## 5. Dependencies and hierarchy

- A task may have a parent task.
- A task may be blocked by another task.
- Self-dependency is invalid.
- If scheduling relies on dependencies, cycles must be detected and rejected.
- Deleting/archiving a project must not silently delete its tasks; use archive or explicit preview.
- Parent status does not automatically overwrite children. Any future roll-up status must be computed and labelled as derived.
- Store stable IDs; never use visible task number, array index or title as primary identity.

## 6. Linking modules to work

Supported link types should include:
- tool run / scan / diagnostic result
- security finding / assessment
- intelligence item / source
- code repository / issue / pull request / commit
- document / note / report / external URL
- workflow run
- runbook/incident reference

Links contain a typed reference and minimal summary, not copied secrets or unbounded raw output. The originating module remains the source of detailed data where appropriate.

## 7. Personal-first vs team features

### MVP
- One local profile and Personal workspace
- Local projects, tasks, labels, subtasks and milestones
- List and Kanban
- Basic search/filter/sort
- Activity for material updates
- Link/reference artifacts
- JSON export and import preview (once implemented safely)

### Future, not assumed in MVP
- Multi-user workspaces and roles
- Cloud sync and offline conflict resolution
- Assigning tasks to remote team members
- Sprint/cycle planning with capacity estimates
- Comments/mentions, email/chat notifications
- Integration with GitHub, Linear or issue trackers
- Custom fields and workflow builder
- Gantt/critical path and workload charts

Do not add a required backend just to model future team fields. Keep the model migration-friendly and implement remote identity only alongside a defined auth/sync design.

## 8. UI and accessibility requirements

- Form labels and validation errors are programmatically associated.
- Escape closes an unsaved create form without creating a record.
- Dialog focus is trapped and returned to the invoking control.
- Board drag/drop has a keyboard alternative.
- Status and priority include accessible text, not color only.
- Large task lists have pagination/virtualization when necessary.
- Loading, empty, error and offline states are distinct.
- Search/filter state is deterministic and resettable.
- Optimistic updates must roll back visibly on persistence error.

## 9. Minimum acceptance tests

- Create project → app restart → project remains.
- Create task without project → task appears in personal inbox.
- Create task in project → task appears in project list.
- Change status in Board → same status is shown in List and Task Detail.
- Invalid priority/status is rejected by service layer and UI.
- Archive/restore does not orphan child records.
- Self-dependency and a dependency cycle are rejected if cycle detection is enabled.
- Export/import round trip preserves stable IDs and links or reports explicit conversion.
- Tool result can be linked to task without copying secrets.
- Activity records key state transitions; logs do not store secret field values.
