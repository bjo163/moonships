import { describe, expect, it } from "vitest";
import { resolveRoute, ROUTE_REGISTRY } from "./route-registry";

describe("DevTools route", () => {
  it("resolves its stable page ID and hash", () => {
    expect(resolveRoute("#/devtools")).toEqual({ kind: "page", id: "devtools" });
    expect(ROUTE_REGISTRY.devtools.hash).toBe("#/devtools");
  });
});
