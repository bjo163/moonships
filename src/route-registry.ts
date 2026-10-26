export const ROUTE_REGISTRY = {
  overview: {
    id: "overview",
    label: "Overview",
    hash: "#/overview",
  },
  projects: {
    id: "projects",
    label: "Projects",
    hash: "#/projects",
  },
  tasks: {
    id: "tasks",
    label: "Tasks",
    hash: "#/tasks",
  },
} as const;

export type RouteId = keyof typeof ROUTE_REGISTRY;

export type RouteResolution =
  | { kind: "page"; id: RouteId }
  | { kind: "not-found"; requested: string };

export function resolveRoute(input: string): RouteResolution {
  const normalized = input.trim().replace(/^#/, "").replace(/^\/+/, "").toLowerCase();

  // Keep the existing home anchor compatible while establishing stable route IDs.
  if (normalized === "" || normalized === "home" || normalized === "overview") {
    return { kind: "page", id: "overview" };
  }
  if (normalized === "projects") return { kind: "page", id: "projects" };
  if (normalized === "tasks") return { kind: "page", id: "tasks" };

  return { kind: "not-found", requested: input };
}
