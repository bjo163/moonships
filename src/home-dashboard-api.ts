import { invoke } from "@tauri-apps/api/core";

export type HomeDashboard = {
  asOf: string;
  activeProjectCount: number;
  activeTaskCount: number;
  counts: {
    openTasks: number;
    overdueTasks: number;
    dueSoonTasks: number;
    blockedTasks: number;
  };
  recentProjects: Array<{
    id: string;
    name: string;
    status: string;
    updatedAt: string;
  }>;
  overdueTasks: Array<{
    id: string;
    projectId: string | null;
    projectName: string | null;
    title: string;
    status: string;
    priority: string;
    dueAt: string;
  }>;
  upcomingTasks: Array<{
    id: string;
    projectId: string | null;
    projectName: string | null;
    title: string;
    status: string;
    priority: string;
    dueAt: string;
  }>;
  recentActivity: Array<{
    id: string;
    entityType: "project" | "task";
    entityId: string;
    action: string;
    summary: string;
    occurredAt: string;
  }>;
};

export function getHomeDashboard(): Promise<HomeDashboard> {
  return invoke<HomeDashboard>("get_home_dashboard");
}
