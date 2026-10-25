import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const stylesheet = readFileSync(new URL("./index.css", import.meta.url), "utf8");

describe("semantic design token contract", () => {
  it("defines semantic dark and light palettes without claiming contrast certification", () => {
    expect(stylesheet).toContain("palette is NOT contrast-certified");
    expect(stylesheet).toContain(":root[data-theme=\"light\"]");
    for (const token of [
      "--color-canvas",
      "--color-sidebar",
      "--color-surface",
      "--color-text-primary",
      "--color-text-muted",
      "--color-accent",
      "--color-status-success",
      "--color-status-warning",
      "--color-status-danger",
      "--color-focus",
    ]) {
      expect(stylesheet).toContain(token);
    }
  });

  it("uses defined layout tokens and contains no malformed escaped newlines", () => {
    expect(stylesheet).toContain("--layout-max-width: 1680px;");
    expect(stylesheet).toContain("max-width: var(--layout-max-width);");
    expect(stylesheet).not.toContain("\\n  --");
    expect(stylesheet).not.toContain("--layout-max-width: var(--layout-max-width);");
  });

  it("routes shell colors through semantic variables", () => {
    expect(stylesheet).toContain("background: var(--color-canvas)");
    expect(stylesheet).toContain("background: var(--color-sidebar)");
    expect(stylesheet).toContain("color: var(--color-text-primary)");
    expect(stylesheet).toContain("outline: var(--focus-width) solid var(--color-focus)");
  });
});
