import { invoke } from "@tauri-apps/api/core";

export type ProjectStatus = "planned" | "active" | "paused" | "completed";

export type ProjectRecord = {
  id: string;
  workspaceId: string;
  name: string;
  projectKey: string | null;
  description: string | null;
  goal: string | null;
  status: ProjectStatus;
  archivedAt: string | null;
  createdAt: string;
  updatedAt: string;
};

export type ProjectInput = {
  name: string;
  description: string | null;
  goal: string | null;
};

export type UpdateProjectInput = ProjectInput & {
  id: string;
  status: ProjectStatus;
};

type ProjectListResponse = { projects: ProjectRecord[] };

export type ProjectActivityRecord = {
  id: string;
  entityType: "project" | "task";
  action: string;
  summary: string;
  occurredAt: string;
};

export type ProjectOverviewRecord = {
  project: ProjectRecord;
  activeTaskCount: number;
  recentActivity: ProjectActivityRecord[];
};

export type MilestoneTaskRecord = { id: string; title: string; status: string };
export type MilestoneRecord = {
  id: string;
  workspaceId: string;
  projectId: string;
  title: string;
  description: string | null;
  dueDate: string | null;
  status: "planned" | "active" | "completed";
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
  tasks: MilestoneTaskRecord[];
};

export function listProjects(): Promise<ProjectRecord[]> {
  return invoke<ProjectListResponse>("list_projects", { request: {} }).then(
    ({ projects }) => projects,
  );
}

export function createProject(request: ProjectInput): Promise<ProjectRecord> {
  return invoke<ProjectRecord>("create_project", { request });
}

export function updateProject(request: UpdateProjectInput): Promise<ProjectRecord> {
  return invoke<ProjectRecord>("update_project", { request });
}

export function getProjectOverview(id: string): Promise<ProjectOverviewRecord> {
  return invoke<ProjectOverviewRecord>("get_project_overview", { request: { id } });
}

export function listMilestones(projectId: string): Promise<MilestoneRecord[]> {
  return invoke<{ milestones: MilestoneRecord[] }>("list_milestones", {
    request: { projectId },
  }).then(({ milestones }) => milestones);
}

export function createMilestone(request: {
  projectId: string;
  title: string;
  description: string | null;
  dueDate: string | null;
}): Promise<MilestoneRecord> {
  return invoke<MilestoneRecord>("create_milestone", { request });
}

export function associateMilestoneTask(request: {
  projectId: string;
  milestoneId: string;
  taskId: string;
}): Promise<MilestoneRecord> {
  return invoke<MilestoneRecord>("associate_milestone_task", { request });
}

export function updateMilestone(request: {
  projectId: string;
  id: string;
  title: string;
  description: string | null;
  dueDate: string | null;
  status: MilestoneRecord["status"];
}): Promise<MilestoneRecord> {
  return invoke<MilestoneRecord>("update_milestone", { request });
}

export function toProjectErrorMessage(error: unknown, action: "load" | "save"): string {
  const code = typeof error === "string" ? error.split(":", 1)[0] : "";
  if (code === "VALIDATION_FAILED") return "Check the project fields and try again.";
  if (code === "CONFLICT") return "A project with these details already exists.";
  if (code === "NOT_FOUND") return "This project is no longer available. Refresh the list.";
  return action === "load"
    ? "Projects could not be loaded. Check the desktop app connection and try again."
    : "The project could not be saved. Your entered details are still here; try again.";
}
