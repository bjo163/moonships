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
      id: "settings.open";
      kind: "settings";
      title: string;
      description: string;
      category: string;
      risk: "read-only";
      keywords: string[];
    }
  | {
      id: "project.create" | "task.create" | "note.create";
      kind: "preview";
      entity: "project" | "task" | "note";
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
    kind: "preview",
    entity: "project",
    title: "New Project",
    description: "Preview creating a project (storage is not connected yet)",
    category: "Create",
    risk: "state-change",
    keywords: ["project", "new"],
  },
  {
    id: "task.create",
    kind: "preview",
    entity: "task",
    title: "New Task",
    description: "Preview creating a task (storage is not connected yet)",
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
