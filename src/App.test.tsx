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

  it("keeps shared navigation and header around replaceable page content", () => {
    const markup = renderToStaticMarkup(
      <AppShell>
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
    const markup = renderToStaticMarkup(<AppShell><div>Page</div></AppShell>);

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
