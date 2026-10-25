import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import App, { AppShell, NotFoundPage } from "./App";
import { resolveRoute } from "./route-registry";

describe("Moonships application shell", () => {
  it("renders the foundation status without claiming planned tools are live", () => {
    const markup = renderToStaticMarkup(<App />);

    expect(markup).toContain("Workbench overview");
    expect(markup).toContain("PRE-IMPLEMENTATION SHELL");
    expect(markup).toContain("Not connected yet");
    expect(markup).toContain("No external services connected");
  });

  it("provides a keyboard skip link, focus target and named theme control", () => {
    const markup = renderToStaticMarkup(
      <AppShell theme="dark" onToggleTheme={() => {}}>
        <section aria-label="Test page">
          <h2>Keyboard target</h2>
        </section>
      </AppShell>,
    );

    expect(markup).toContain('class="skip-link" href="#main-content"');
    expect(markup).toContain('id="main-content" tabindex="-1"');
    expect(markup).toContain('aria-label="Switch to light theme"');
    expect(markup).toContain('type="button"');
  });

  it("keeps shared navigation and header around replaceable page content", () => {
    const markup = renderToStaticMarkup(
      <AppShell theme="dark" onToggleTheme={() => {}}>
        <section aria-label="Test page">
          <h2>Replaceable content</h2>
        </section>
      </AppShell>,
    );

    expect(markup).toContain('aria-label="Primary navigation"');
    expect(markup).toContain("Moonships Workbench");
    expect(markup).toContain("Replaceable content");
  });

  it("exposes only the implemented Home route as a link and labels other menu items Planned", () => {
    const markup = renderToStaticMarkup(
      <AppShell theme="dark" onToggleTheme={() => {}}>
        <div>Page</div>
      </AppShell>,
    );

    expect(markup).toContain('aria-label="Workspace navigation"');
    expect(markup).toContain('href="#/overview"');
    expect(markup).toContain('aria-current="page"');
    for (const label of [
      "My Work",
      "Projects",
      "Tasks",
      "Toolbox",
      "Security",
      "Intelligence",
      "Operations",
      "Knowledge",
      "Settings",
    ]) {
      expect(markup).toContain(label);
    }

    const navigation = markup.split('aria-label="Workspace navigation"')[1]?.split("</nav>")[0];
    expect(navigation).toBeDefined();
    expect(navigation).not.toContain('href="#"');
    expect(navigation?.match(/class="nav-state">Planned<\/span>/g)).toHaveLength(9);
  });

  it("resolves registered route IDs and keeps the legacy home hash compatible", () => {
    expect(resolveRoute("#/overview")).toEqual({ kind: "page", id: "overview" });
    expect(resolveRoute("#home")).toEqual({ kind: "page", id: "overview" });
    expect(resolveRoute("")).toEqual({ kind: "page", id: "overview" });
  });

  it("returns a not-found result for unknown route IDs instead of throwing", () => {
    expect(resolveRoute("#/made-up-page")).toEqual({
      kind: "not-found",
      requested: "#/made-up-page",
    });

    const markup = renderToStaticMarkup(<NotFoundPage requested="#/made-up-page" />);
    expect(markup).toContain("This page is not available.");
    expect(markup).toContain("#/made-up-page");
    expect(markup).toContain('href="#/overview"');
  });
});

describe("keyboard accessibility styles", () => {
  it("keeps a visible focus indicator and reveals the skip link on focus", async () => {
    const { readFileSync } = await import("node:fs");
    const stylesheet = readFileSync(new URL("./index.css", import.meta.url), "utf8");

    expect(stylesheet).toContain(":focus-visible");
    expect(stylesheet).toContain(".skip-link:focus");
    expect(stylesheet).toContain("outline: var(--focus-width) solid var(--color-focus)");
  });

  it("honors reduced-motion preferences for transitions and animation", async () => {
    const { readFileSync } = await import("node:fs");
    const stylesheet = readFileSync(new URL("./index.css", import.meta.url), "utf8");

    expect(stylesheet).toContain("@media (prefers-reduced-motion: reduce)");
    expect(stylesheet).toContain("transition-duration: .01ms !important");
  });
});

describe("semantic design token contract", () => {
  it("defines both theme palettes and keeps the proposed palette explicitly uncertified", async () => {
    const { readFileSync } = await import("node:fs");
    const stylesheet = readFileSync(new URL("./index.css", import.meta.url), "utf8");
    expect(stylesheet).toContain("palette is NOT contrast-certified");
    expect(stylesheet).toContain(':root[data-theme="light"]');
    expect(stylesheet).toContain("--color-canvas");
    expect(stylesheet).toContain("--color-text-primary");
    expect(stylesheet).toContain("--color-status-warning");
    expect(stylesheet).toContain("--space-4");
    expect(stylesheet).toContain("--font-ui");
    expect(stylesheet).toContain("--radius-card");
    expect(stylesheet).toContain("--motion-normal");
  });

  it("uses design tokens for component colors instead of raw color literals", async () => {
    const { readFileSync } = await import("node:fs");
    const stylesheet = readFileSync(new URL("./index.css", import.meta.url), "utf8");
    const componentStyles = stylesheet.slice(stylesheet.indexOf("* { box-sizing"));
    expect(componentStyles).not.toMatch(/#[0-9a-fA-F]{3,8}\b/);
  });
});
