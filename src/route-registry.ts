export const ROUTE_REGISTRY = {
  overview: {
    id: "overview",
    label: "Overview",
    hash: "#/overview",
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

  return { kind: "not-found", requested: input };
}
