import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import App, { AppShell, NotFoundPage, SettingsDialog } from "./App";
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

  it("renders an accessible Settings dialog with a working theme action and close control", () => {
    const markup = renderToStaticMarkup(
      <SettingsDialog
        theme="dark"
        onToggleTheme={() => {}}
        onClose={() => {}}
        hideToTray={false}
        hideToTrayStatus="Preference loaded from this device."
        onToggleHideToTray={() => {}}
      />,
    );

    expect(markup).toContain('role="dialog" aria-modal="true" aria-labelledby="settings-title"');
    expect(markup).toContain("Settings");
    expect(markup).toContain("Switch to light theme");
    expect(markup).toContain('aria-label="Close settings"');
    expect(markup).toContain('type="checkbox"');
    expect(markup).toContain("Hide to tray on close");
    expect(markup).toContain('role="status" aria-live="polite"');
    expect(markup).toContain("Notification and permission controls");
    expect(markup).toContain("These capabilities are not connected yet.");
    expect(markup).toContain('aria-label="Desktop notifications (Planned)"');
    expect(markup).toContain('aria-label="Background monitoring permission (Planned)"');
    expect(markup).toContain('class="nav-state">Planned</span>');
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

describe("system tray integration contract", () => {
  it("registers Open, Settings and Quit native menu actions", async () => {
    const { readFileSync } = await import("node:fs");
    const rustSource = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
    const cargoManifest = readFileSync(new URL("../src-tauri/Cargo.toml", import.meta.url), "utf8");

    expect(cargoManifest).toContain('features = ["tray-icon"]');
    expect(cargoManifest).toContain('tauri-plugin-single-instance = "2"');
    expect(rustSource).toContain('tauri_plugin_single_instance::init');
    expect(rustSource).toContain('app.get_webview_window("main")');
    expect(rustSource).toContain('window.set_focus()');
    expect(rustSource).toContain('"open"');
    expect(rustSource).toContain('"settings"');
    expect(rustSource).toContain('"hide"');
    expect(rustSource).toContain('"Hide Window"');
    expect(rustSource).toContain('"quit"');
    expect(rustSource).toContain('app.emit("tray-settings", ())');
    expect(rustSource).toContain("app.exit(0)");
    expect(rustSource).toContain("window.hide()");
    expect(rustSource).toContain("shutdown hook must run before this explicit exit");
    expect(rustSource).toContain("struct HideToTrayPreferenceDto");
    expect(rustSource).toContain("#[serde(rename_all = \"camelCase\", deny_unknown_fields)]");
    expect(rustSource).toContain("get_hide_to_tray(app: AppHandle) -> Result<HideToTrayPreferenceDto, String>");
    expect(rustSource).toContain("request: HideToTrayPreferenceDto");
    expect(rustSource).toContain("Result<HideToTrayPreferenceDto, String>");
    expect(rustSource).toContain("CloseRequested");
    expect(rustSource).toContain("window.hide()");
    expect(rustSource).toContain("hide_to_tray_enabled");
  });
});

describe("SQLite storage path contract", () => {
  it("opens the database beneath Tauri's per-user app data directory", async () => {
    const { readFileSync } = await import("node:fs");
    const rustSource = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
    const cargoManifest = readFileSync(new URL("../src-tauri/Cargo.toml", import.meta.url), "utf8");

    expect(cargoManifest).toContain('rusqlite = { version = "0.32", features = ["bundled"] }');
    expect(rustSource).toContain('const DATABASE_FILE_NAME: &str = "moonships.sqlite3";');
    expect(rustSource).toContain("app.path().app_data_dir()?");
    expect(rustSource).toContain("open_database_in(&app_data_dir)");
    expect(rustSource).toContain("Connection::open(path)");
    expect(rustSource).toContain("opening_database_creates_directory_and_file_under_given_path");
  });
});

describe("typed Tauri command DTO contract", () => {
  it("uses a documented request/response DTO and rejects unknown request fields", async () => {
    const { readFileSync } = await import("node:fs");
    const rustSource = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
    const cargoManifest = readFileSync(new URL("../src-tauri/Cargo.toml", import.meta.url), "utf8");
    const appSource = readFileSync(new URL("./App.tsx", import.meta.url), "utf8");

    expect(cargoManifest).toContain('serde = { version = "1", features = ["derive"] }');
    expect(rustSource).toContain("serde::Deserialize, serde::Serialize");
    expect(rustSource).toContain("deny_unknown_fields");
    expect(appSource).toContain('invoke<{ enabled: boolean }>("get_hide_to_tray")');
    expect(appSource).toContain('request: { enabled: nextValue }');
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
