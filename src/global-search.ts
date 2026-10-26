import type { PaletteCommand } from "./CommandPalette";
import { getPaletteCommands } from "./command-registry";
import { listProjects } from "./projects-api";
import { listTasks } from "./tasks-api";

export type SearchEntityType = "command" | "project" | "task";

export type GlobalSearchResult = PaletteCommand & {
  entityType: SearchEntityType;
  targetId?: string;
};

export type SearchAdapter = {
  id: SearchEntityType;
  load: () => Promise<GlobalSearchResult[]>;
};

export const GLOBAL_SEARCH_ADAPTERS: readonly SearchAdapter[] = [
  {
    id: "command",
    load: async () =>
      getPaletteCommands().map((command) => ({
        ...command,
        category: `Command · ${command.category}`,
        entityType: "command" as const,
      })),
  },
  {
    id: "project",
    load: async () =>
      (await listProjects()).map((project) => ({
        id: `project:${project.id}`,
        targetId: project.id,
        entityType: "project" as const,
        title: project.name,
        description: [project.status, project.description, project.goal]
          .filter(Boolean)
          .join(" · "),
        category: "Project",
        risk: "read-only" as const,
        keywords: [project.projectKey ?? "", "project"],
      })),
  },
  {
    id: "task",
    load: async () =>
      (await listTasks()).map((task) => ({
        id: `task:${task.id}`,
        targetId: task.id,
        entityType: "task" as const,
        title: task.title,
        description: [task.status.replaceAll("_", " "), task.description]
          .filter(Boolean)
          .join(" · "),
        category: "Task",
        risk: "read-only" as const,
        keywords: [task.priority, "task", "inbox"],
      })),
  },
];

export type GlobalSearchLoad = {
  results: GlobalSearchResult[];
  failedAdapters: SearchEntityType[];
};

export async function loadGlobalSearch(): Promise<GlobalSearchLoad> {
  const loaded = await Promise.allSettled(GLOBAL_SEARCH_ADAPTERS.map((adapter) => adapter.load()));
  const results: GlobalSearchResult[] = [];
  const failedAdapters: SearchEntityType[] = [];
  loaded.forEach((entry, index) => {
    if (entry.status === "fulfilled") results.push(...entry.value);
    else failedAdapters.push(GLOBAL_SEARCH_ADAPTERS[index].id);
  });
  return { results, failedAdapters };
}
