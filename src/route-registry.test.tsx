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
