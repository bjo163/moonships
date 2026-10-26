import type { PaletteCommand } from "./CommandPalette";
import type { RouteId } from "./route-registry";

export type RegisteredCommand =
  | {
      id: "navigation.overview";
      kind: "navigate";
      route: RouteId;
      title: string;
      description: string;
      category: string;
      risk: "read-only";
      keywords: string[];
    }
  | {
      id: "navigation.projects";
      kind: "navigate";
      route: RouteId;
      title: string;
      description: string;
      category: string;
      risk: "read-only";
      keywords: string[];
    }
  | {
      id: "navigation.tasks";
      kind: "navigate";
      route: RouteId;
      title: string;
      description: string;
      category: string;
      risk: "read-only";
      keywords: string[];
    }
  | {
      id: "navigation.intelligence-sources";
      kind: "navigate";
      route: RouteId;
      title: string;
      description: string;
      category: string;
      risk: "read-only";
      keywords: string[];
    }
  | {
      id: "navigation.watchlists";
      kind: "navigate";
      route: RouteId;
      title: string;
      description: string;
      category: string;
      risk: "read-only";
      keywords: string[];
    }
  | {
      id: "settings.open";
      kind: "settings";
      title: string;
      description: string;
      category: string;
      risk: "read-only";
      keywords: string[];
    }
  | {
      id: "project.create";
      kind: "create-project";
      title: string;
      description: string;
      category: string;
      risk: "state-change";
      keywords: string[];
    }
  | {
      id: "task.create";
      kind: "create-task";
      title: string;
      description: string;
      category: string;
      risk: "state-change";
      keywords: string[];
    }
  | {
      id: "note.create";
      kind: "preview";
      entity: "note";
      title: string;
      description: string;
      category: string;
      risk: "state-change";
      keywords: string[];
    };

export const COMMAND_REGISTRY: readonly RegisteredCommand[] = [
  {
    id: "navigation.overview",
    kind: "navigate",
    route: "overview",
    title: "Go to Home",
    description: "Open the registered workspace overview",
    category: "Navigation",
    risk: "read-only",
    keywords: ["home", "overview", "dashboard"],
  },
  {
    id: "navigation.projects",
    kind: "navigate",
    route: "projects",
    title: "Open Projects",
    description: "Browse and manage locally stored projects",
    category: "Navigation",
    risk: "read-only",
    keywords: ["projects", "work", "portfolio"],
  },
  {
    id: "navigation.tasks",
    kind: "navigate",
    route: "tasks",
    title: "Open Tasks",
    description: "Browse active tasks saved in the local workspace",
    category: "Navigation",
    risk: "read-only",
    keywords: ["tasks", "todo", "work"],
  },
  {
    id: "navigation.intelligence-sources",
    kind: "navigate",
    route: "intelligence-sources",
    title: "Open Intelligence Sources",
    description: "Manage locally saved RSS source settings",
    category: "Navigation",
    risk: "read-only",
    keywords: ["intelligence", "sources", "rss", "feeds"],
  },
  {
    id: "navigation.watchlists",
    kind: "navigate",
    route: "watchlists",
    title: "Open Topic Watchlists",
    description: "Manage saved keyword and category filters",
    category: "Navigation",
    risk: "read-only",
    keywords: ["watchlist", "topic", "keywords", "categories"],
  },
  {
    id: "settings.open",
    kind: "settings",
    title: "Open Settings",
    description: "Open workspace preferences",
    category: "Navigation",
    risk: "read-only",
    keywords: ["preferences", "theme", "tray"],
  },
  {
    id: "project.create",
    kind: "create-project",
    title: "New Project",
    description: "Create a project in the local workspace",
    category: "Create",
    risk: "state-change",
    keywords: ["project", "new"],
  },
  {
    id: "task.create",
    kind: "create-task",
    title: "New Task",
    description: "Create a task in the local workspace or Personal inbox",
    category: "Create",
    risk: "state-change",
    keywords: ["task", "todo", "new"],
  },
  {
    id: "note.create",
    kind: "preview",
    entity: "note",
    title: "New Note",
    description: "Preview creating a note (storage is not connected yet)",
    category: "Create",
    risk: "state-change",
    keywords: ["note", "knowledge", "new"],
  },
] as const;

export type PaletteCommandId = (typeof COMMAND_REGISTRY)[number]["id"];

export function getRegisteredCommand(id: string): RegisteredCommand | undefined {
  return COMMAND_REGISTRY.find((command) => command.id === id);
}

export function isRegisteredCommandId(id: string): id is PaletteCommandId {
  return getRegisteredCommand(id) !== undefined;
}

export function getPaletteCommands(): PaletteCommand[] {
  return COMMAND_REGISTRY.map(({ id, title, description, category, risk, keywords }) => ({
    id,
    title,
    description,
    category,
    risk,
    keywords,
  }));
}
