import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import App, { AppShell, NotFoundPage, SettingsDialog, WorkspaceProfileDialog } from "./App";
import { resolveRoute } from "./route-registry";
import { CommandPalette, filterPaletteCommands, type PaletteCommand } from "./CommandPalette";
import { COMMAND_REGISTRY, getRegisteredCommand, getPaletteCommands } from "./command-registry";
import {
  getWorkspaceProfile,
  readWorkspaceProfile,
  writeWorkspaceProfile,
  WORKSPACE_PROFILES,
} from "./workspace-profile";

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
    expect(rustSource).toContain("tauri_plugin_single_instance::init");
    expect(rustSource).toContain('app.get_webview_window("main")');
    expect(rustSource).toContain("window.set_focus()");
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
    expect(rustSource).toContain('#[serde(rename_all = "camelCase", deny_unknown_fields)]');
    expect(rustSource).toContain(
      "get_hide_to_tray(app: AppHandle) -> Result<HideToTrayPreferenceDto, String>",
    );
    expect(rustSource).toContain("request: HideToTrayPreferenceDto");
    expect(rustSource).toContain("Result<HideToTrayPreferenceDto, String>");
    expect(rustSource).toContain("CloseRequested");
    expect(rustSource).toContain("window.hide()");
    expect(rustSource).toContain("hide_to_tray_enabled");
  });
});

describe("SQLite schema migration contract", () => {
  it("runs versioned migrations before desktop startup completes", async () => {
    const { readFileSync } = await import("node:fs");
    const rustSource = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
    const architecture = readFileSync(new URL("../docs/ARCHITECTURE.md", import.meta.url), "utf8");

    expect(rustSource).toContain("fn run_migrations(connection: &mut Connection)");
    expect(rustSource).toContain("CREATE TABLE IF NOT EXISTS schema_migrations");
    expect(rustSource).toContain("INSERT INTO schema_migrations (version) VALUES (1)");
    expect(rustSource).toContain(".commit()");
    expect(rustSource).toContain("run_migrations(&mut database)");
    expect(rustSource).toContain(
      "fresh_database_migration_persists_schema_version_and_is_idempotent",
    );
    expect(architecture).toContain(
      "Each migration and its version record must commit in one SQLite transaction",
    );
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
    expect(appSource).toContain("request: { enabled: nextValue }");
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

describe("SQLite connection safety contract", () => {
  it("enables foreign keys and configures a bounded busy timeout on every connection", async () => {
    const { readFileSync } = await import("node:fs");
    const rustSource = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
    const architecture = readFileSync(new URL("../docs/ARCHITECTURE.md", import.meta.url), "utf8");

    expect(rustSource).toContain("const DATABASE_BUSY_TIMEOUT_MS: u64 = 5_000;");
    expect(rustSource).toContain("busy_timeout(Duration::from_millis(DATABASE_BUSY_TIMEOUT_MS))");
    expect(rustSource).toContain('execute_batch("PRAGMA foreign_keys = ON;")');
    expect(rustSource).toContain('query_row("PRAGMA foreign_keys"');
    expect(rustSource).toContain("foreign_key_violation_is_rejected_on_opened_connection");
    expect(rustSource).toContain(
      "every_opened_connection_enables_foreign_keys_and_sets_busy_timeout",
    );
    expect(architecture).toContain("Every connection enables SQLite foreign-key enforcement");
    expect(architecture).toContain("5,000 ms busy timeout");
  });
});

describe("M1 desktop foundation contracts", () => {
  it("provides structured local logs with recursive secret-field redaction", async () => {
    const { readFileSync } = await import("node:fs");
    const logger = readFileSync(
      new URL("../src-tauri/src/local_logging.rs", import.meta.url),
      "utf8",
    );
    const rustSource = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

    expect(logger).toContain("pub enum LogLevel");
    expect(logger).toContain("pub enum LogCategory");
    expect(logger).toContain("fn is_sensitive_key");
    expect(logger).toContain("pub fn render_event");
    expect(logger).toContain("pub fn write_event");
    expect(logger).toContain("structured_logs_redact_secret_fields_recursively");
    expect(logger).toContain("local_log_writer_appends_json_lines_to_app_owned_path");
    expect(rustSource).toContain('join("logs").join("moonships.log")');
  });

  it("locks job status names, tool contracts, and native permission policy", async () => {
    const { readFileSync } = await import("node:fs");
    const domain = readFileSync(new URL("../src-tauri/src/domain.rs", import.meta.url), "utf8");

    for (const variant of [
      "Queued",
      "Running",
      "WaitingApproval",
      "Succeeded",
      "Partial",
      "Failed",
      "Cancelled",
      "TimedOut",
    ]) {
      expect(domain).toContain(variant);
    }
    expect(domain).toContain("pub struct ToolDefinition");
    expect(domain).toContain("pub permissions: Vec<String>");
    expect(domain).toContain("pub input_schema: Value");
    expect(domain).toContain("pub output_schema: Value");
    expect(domain).toContain("pub risk_class: RiskClass");
    expect(domain).toContain("pub struct ToolResult");
    expect(domain).toContain("pub struct ToolRegistry");
    expect(domain).toContain("sample_tool_can_be_registered_and_duplicate_ids_are_rejected");
    expect(domain).toContain("pub fn evaluate_policy");
    expect(domain).toContain("permission_policy_covers_allowed_denied_and_approval_required");
  });

  it("exposes explicit module availability states instead of conflating setup and empty results", async () => {
    const { readFileSync } = await import("node:fs");
    const availability = readFileSync(new URL("./module-availability.ts", import.meta.url), "utf8");
    const appSource = readFileSync(new URL("./App.tsx", import.meta.url), "utf8");
    const { describeModuleAvailability } = await import("./module-availability");

    for (const state of [
      "available",
      "disabled",
      "unsupported",
      "not_configured",
      "degraded",
    ] as const) {
      expect(availability).toContain(state);
      expect(describeModuleAvailability(state).label.length).toBeGreaterThan(0);
    }
    expect(describeModuleAvailability("not_configured").isOperational).toBe(false);
    expect(describeModuleAvailability("not_configured").description).toContain(
      "no operation has been attempted",
    );
    expect(describeModuleAvailability("available").isOperational).toBe(true);
    expect(appSource).toContain('availability: "not_configured"');
    expect(appSource).toContain("describeModuleAvailability(item.availability)");
  });

  it("defines versioned non-secret configuration defaults and validation", async () => {
    const { readFileSync } = await import("node:fs");
    const domain = readFileSync(new URL("../src-tauri/src/domain.rs", import.meta.url), "utf8");

    expect(domain).toContain("CURRENT_CONFIG_SCHEMA_VERSION: u32 = 1");
    expect(domain).toContain("impl Default for AppConfig");
    expect(domain).toContain("config_defaults_are_valid_and_schema_mismatch_fails_closed");
    expect(domain).toContain("Unsupported config schema version");
    expect(domain).toContain("deny_unknown_fields");
  });
});

describe("M1-031 Command Palette surface", () => {
  const commands: PaletteCommand[] = [
    {
      id: "workspace.inspect",
      title: "Inspect workspace",
      description: "Review local workspace status",
      category: "Navigation",
      risk: "read-only",
      keywords: ["home", "overview"],
    },
    {
      id: "project.create",
      title: "New Project",
      description: "Prepare a new project",
      category: "Create",
      risk: "state-change",
      keywords: ["project"],
    },
    {
      id: "system.cleanup",
      title: "Clean storage",
      description: "Review cleanup candidates",
      category: "Tool",
      risk: "destructive",
    },
  ];

  it("filters registered palette entries by title, description, category, and keywords", () => {
    expect(filterPaletteCommands(commands, "project").map((item) => item.id)).toEqual([
      "project.create",
    ]);
    expect(filterPaletteCommands(commands, "navigation").map((item) => item.id)).toEqual([
      "workspace.inspect",
    ]);
    expect(filterPaletteCommands(commands, "OVERVIEW").map((item) => item.id)).toEqual([
      "workspace.inspect",
    ]);
    expect(filterPaletteCommands(commands, "not registered")).toEqual([]);
    expect(filterPaletteCommands(commands, "  ")).toEqual(commands);
  });

  it("renders accessible results with category and risk labels without executing a tool", () => {
    const markup = renderToStaticMarkup(
      <CommandPalette open commands={commands} onClose={() => {}} onSelect={() => {}} />,
    );
    expect(markup).toContain('role="dialog"');
    expect(markup).toContain('aria-modal="true"');
    expect(markup).toContain('role="listbox"');
    expect(markup).toContain('role="option"');
    expect(markup).toContain("Navigation");
    expect(markup).toContain("Read-only");
    expect(markup).toContain("Destructive · preview required");
    expect(markup).toContain("Selecting a result does not execute a tool.");
  });

  it("renders honest loading and empty states", () => {
    const loading = renderToStaticMarkup(
      <CommandPalette open commands={[]} loading onClose={() => {}} onSelect={() => {}} />,
    );
    const empty = renderToStaticMarkup(
      <CommandPalette open commands={[]} onClose={() => {}} onSelect={() => {}} />,
    );
    expect(loading).toContain("Loading commands…");
    expect(loading).toContain('aria-busy="true"');
    expect(empty).toContain("No commands are registered yet.");
  });

  it("keeps the palette closed until the user requests it from the shell", () => {
    const markup = renderToStaticMarkup(<App />);
    expect(markup).toContain('aria-label="Open Command Palette"');
    expect(markup).not.toContain('id="command-palette-title"');
  });
});

describe("M1-032 registered command safety", () => {
  it("exposes only allowlisted command IDs and rejects unknown IDs", () => {
    const commands = getPaletteCommands();
    expect(commands.map((command) => command.id)).toEqual(
      COMMAND_REGISTRY.map((command) => command.id),
    );
    expect(getRegisteredCommand("navigation.overview")?.kind).toBe("navigate");
    expect(getRegisteredCommand("settings.open")?.kind).toBe("settings");
    expect(getRegisteredCommand("project.create")?.kind).toBe("preview");
    expect(getRegisteredCommand("system.shell")).toBeUndefined();
    expect(commands.some((command) => command.id === "system.shell")).toBe(false);
  });

  it("routes all create actions to preview and never registers a direct executor", () => {
    for (const id of ["project.create", "task.create", "note.create"]) {
      expect(getRegisteredCommand(id)?.kind).toBe("preview");
      expect(getRegisteredCommand(id)?.risk).toBe("state-change");
    }
    expect(COMMAND_REGISTRY.every((command) => !("execute" in command))).toBe(true);
  });
});

describe("M1-033 Ctrl+K lifecycle contract", () => {
  it("binds the shortcut while guarding editable controls and existing dialogs", async () => {
    const { readFileSync } = await import("node:fs");
    const source = readFileSync(new URL("./App.tsx", import.meta.url), "utf8");
    expect(source).toContain('event.key.toLowerCase() !== "k"');
    expect(source).toContain("isEditableTarget(event.target)");
    expect(source).toContain('document.querySelector("[role=dialog]")');
    expect(source).toContain("paletteTriggerRef.current?.focus()");
    expect(source).toMatch(/paletteTriggerRef\.current\s*=\s*document\.activeElement/);
  });
});

describe("M1-034 workspace profiles", () => {
  it("offers all six profiles and keeps pinned sections separate from permissions", () => {
    expect(WORKSPACE_PROFILES.map((profile) => profile.id)).toEqual([
      "developer",
      "network_noc",
      "sysadmin",
      "security_research",
      "intelligence_research",
      "custom",
    ]);
    expect(WORKSPACE_PROFILES.every((profile) => profile.pinned.length > 0)).toBe(true);
    expect(WORKSPACE_PROFILES.every((profile) => !("permissions" in profile))).toBe(true);
    expect(getWorkspaceProfile("not-a-profile").id).toBe("developer");
  });

  it("persists a selected profile and fails safely when storage throws", () => {
    const values = new Map<string, string>();
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => {
        values.set(key, value);
      },
    };
    expect(readWorkspaceProfile(storage)).toBeNull();
    expect(writeWorkspaceProfile("security_research", storage)).toBe(true);
    expect(readWorkspaceProfile(storage)).toBe("security_research");
    expect(
      writeWorkspaceProfile("network_noc", {
        setItem: () => {
          throw new Error("storage unavailable");
        },
      }),
    ).toBe(false);
    expect(
      readWorkspaceProfile({
        getItem: () => {
          throw new Error("storage unavailable");
        },
      }),
    ).toBeNull();
  });

  it("exposes the profile selector and states that profiles do not grant permissions", () => {
    const markup = renderToStaticMarkup(<App />);
    expect(markup).toContain('aria-label="Change workspace profile"');
    const dialog = renderToStaticMarkup(
      <WorkspaceProfileDialog currentProfile="developer" onSelect={() => {}} onClose={() => {}} />,
    );
    expect(dialog).toContain(
      "Profiles only change suggested pinned sections and layout preferences.",
    );
    expect(dialog).toContain("They never grant permissions or enable tools.");
  });
});

describe("M1-038 desktop viewport and overflow contract", () => {
  it("documents the native minimum window and responsive overflow protections", async () => {
    const { readFileSync } = await import("node:fs");
    const css = readFileSync(new URL("./index.css", import.meta.url), "utf8");
    const tauri = JSON.parse(
      readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
    ) as {
      app: { windows: Array<{ minWidth: number; minHeight: number }> };
    };

    expect(tauri.app.windows[0]?.minWidth).toBe(960);
    expect(tauri.app.windows[0]?.minHeight).toBe(640);
    expect(css).toContain("@media (max-width: 960px)");
    expect(css).toContain("min-width: 0");
    expect(css).toContain("overflow-y: auto");
    expect(css).toContain("@media (max-height: 720px)");
    expect(css).toContain("@media (max-width: 620px)");
  });
});

describe("M1-039 visual and accessibility review baseline", () => {
  it("keeps keyboard focus, reduced-motion and semantic overlay safeguards in the stylesheet", async () => {
    const { readFileSync } = await import("node:fs");
    const css = readFileSync(new URL("./index.css", import.meta.url), "utf8");
    const source = readFileSync(new URL("./App.tsx", import.meta.url), "utf8");

    expect(css).toContain(":focus-visible");
    expect(css).toContain("@media (prefers-reduced-motion: reduce)");
    expect(css).toContain("max-height: calc(100dvh - 3rem)");
    expect(source).toContain('aria-label="Primary navigation"');
    expect(source).toContain('aria-modal="true"');
    expect(source).toContain('className="skip-link"');
  });
});
