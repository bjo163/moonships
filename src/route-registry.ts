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
  cleaner: {
    id: "cleaner",
    label: "Clean & Storage",
    hash: "#/cleaner",
  },
  devtools: {
    id: "devtools",
    label: "DevTools",
    hash: "#/devtools",
  },
  processes: {
    id: "processes",
    label: "Processes",
    hash: "#/processes",
  },
  services: {
    id: "services",
    label: "Services",
    hash: "#/services",
  },
  activity: { id: "activity", label: "Activity Center", hash: "#/activity" },
  netops: { id: "netops", label: "NetOps", hash: "#/netops" },
  "intelligence-sources": {
    id: "intelligence-sources",
    label: "Intelligence Sources",
    hash: "#/intelligence-sources",
  },
} as const;

export type RouteId = keyof typeof ROUTE_REGISTRY;

export type RouteResolution =
  | { kind: "page"; id: RouteId }
  | { kind: "project-overview"; projectId: string }
  | { kind: "task-detail"; taskId: string }
  | { kind: "not-found"; requested: string };

export function resolveRoute(input: string): RouteResolution {
  const path = input.trim().replace(/^#/, "").replace(/^\/+/, "");
  const projectMatch = path.match(/^projects\/([^/]+)$/i);
  if (projectMatch) {
    try {
      const projectId = decodeURIComponent(projectMatch[1]);
      if (projectId && projectId.length <= 128) return { kind: "project-overview", projectId };
    } catch {
      return { kind: "not-found", requested: input };
    }
  }
  const taskMatch = path.match(/^tasks\/([^/]+)$/i);
  if (taskMatch) {
    try {
      const taskId = decodeURIComponent(taskMatch[1]);
      if (taskId && taskId.length <= 128) return { kind: "task-detail", taskId };
    } catch {
      return { kind: "not-found", requested: input };
    }
  }
  const normalized = input.trim().replace(/^#/, "").replace(/^\/+/, "").toLowerCase();

  // Keep the existing home anchor compatible while establishing stable route IDs.
  if (normalized === "" || normalized === "home" || normalized === "overview") {
    return { kind: "page", id: "overview" };
  }
  if (normalized === "projects") return { kind: "page", id: "projects" };
  if (normalized === "tasks") return { kind: "page", id: "tasks" };
  if (normalized === "cleaner") return { kind: "page", id: "cleaner" };
  if (normalized === "devtools") return { kind: "page", id: "devtools" };
  if (normalized === "processes") return { kind: "page", id: "processes" };
  if (normalized === "services") return { kind: "page", id: "services" };
  if (normalized === "activity") return { kind: "page", id: "activity" };
  if (normalized === "netops") return { kind: "page", id: "netops" };
  if (normalized === "intelligence-sources") return { kind: "page", id: "intelligence-sources" };

  return { kind: "not-found", requested: input };
}
