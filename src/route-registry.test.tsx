import { describe, expect, it } from "vitest";
import { ROUTE_REGISTRY, resolveRoute } from "./route-registry";

describe("DevTools route", () => {
  it("resolves its stable page ID and hash", () => {
    expect(resolveRoute("#/devtools")).toEqual({ kind: "page", id: "devtools" });
    expect(ROUTE_REGISTRY.devtools.hash).toBe("#/devtools");
  });
});

describe("Processes route", () => {
  it("resolves its stable page ID and hash", () => {
    expect(resolveRoute("#/processes")).toEqual({ kind: "page", id: "processes" });
    expect(ROUTE_REGISTRY.processes.hash).toBe("#/processes");
  });
});

describe("Services route", () => {
  it("resolves its stable page ID and hash", () => {
    expect(resolveRoute("#/services")).toEqual({ kind: "page", id: "services" });
    expect(ROUTE_REGISTRY.services.hash).toBe("#/services");
  });
});

describe("Activity Center route", () => {
  it("resolves its stable page ID and hash", () => {
    expect(resolveRoute("#/activity")).toEqual({ kind: "page", id: "activity" });
    expect(ROUTE_REGISTRY.activity.hash).toBe("#/activity");
  });
});

describe("NetOps route", () => {
  it("resolves its stable page ID and hash", () => {
    expect(resolveRoute("#/netops")).toEqual({ kind: "page", id: "netops" });
    expect(ROUTE_REGISTRY.netops.hash).toBe("#/netops");
  });
});

describe("Intelligence routes", () => {
  it("resolves the list and encoded item detail routes", () => {
    expect(resolveRoute("#/intelligence")).toEqual({ kind: "page", id: "intelligence" });
    expect(resolveRoute("#/intelligence/items/item-42")).toEqual({
      kind: "intelligence-item",
      itemId: "item-42",
    });
    expect(ROUTE_REGISTRY.intelligence.hash).toBe("#/intelligence");
  });
});
