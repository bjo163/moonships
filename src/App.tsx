import { useEffect, useRef, useState, type ReactNode } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { resolveRoute, type RouteResolution } from "./route-registry";
import { CommandPalette } from "./CommandPalette";
import { getPaletteCommands, getRegisteredCommand } from "./command-registry";
import {
  getWorkspaceProfile,
  readWorkspaceProfile,
  writeWorkspaceProfile,
  WORKSPACE_PROFILES,
  type WorkspaceProfileId,
} from "./workspace-profile";
import { describeModuleAvailability, type ModuleAvailability } from "./module-availability";
import {
  applyThemePreference,
  readThemePreference,
  writeThemePreference,
  type ThemePreference,
} from "./theme-preference";

const plannedNavigationItems = [
  "My Work",
  "Projects",
  "Tasks",
  "Toolbox",
  "Security",
  "Intelligence",
  "Operations",
  "Knowledge",
  "Settings",
] as const;

const foundationChecks: Array<{
  label: string;
  detail: string;
  status: "Foundation" | "Planned";
  availability?: ModuleAvailability;
}> = [
  { label: "Desktop runtime", detail: "Tauri 2 · Rust", status: "Foundation" },
  { label: "Interface", detail: "React · TypeScript · Vite", status: "Foundation" },
  {
    label: "Data and tools",
    detail: "Not connected yet",
    status: "Planned",
    availability: "not_configured",
  },
];

type AppShellProps = {
  children: ReactNode;
  theme: ThemePreference;
  onToggleTheme: () => void;
  onOpenCommandPalette?: () => void;
  profileLabel?: string;
  pinnedItems?: readonly string[];
  onOpenProfile?: () => void;
};

export function AppShell({
  children,
  theme,
  onToggleTheme,
  onOpenCommandPalette,
  profileLabel = "Developer",
  pinnedItems = [],
  onOpenProfile,
}: AppShellProps) {
  return (
    <main className="app-frame">
      <a className="skip-link" href="#main-content">
        Skip to main content
      </a>
      <aside className="rail" aria-label="Primary navigation">
        <a className="brand" href="#/overview" aria-label="Moonships home">
          <span className="brand-mark" aria-hidden="true">
            M
          </span>
          <span className="brand-name">
            MOONSHIPS<span>WORKBENCH</span>
          </span>
        </a>
        <div className="rail-label">WORKSPACE</div>
        <nav className="navigation-list" aria-label="Workspace navigation">
          <a className="nav-item nav-item-active" href="#/overview" aria-current="page">
            <span>Home</span>
          </a>
          {plannedNavigationItems.map((label) => (
            <div
              className={`nav-item nav-item-planned${pinnedItems.includes(label) ? " nav-item-profile-pinned" : ""}`}
              key={label}
              aria-disabled="true"
            >
              <span>{label}</span>
              <span className="nav-state">
                {pinnedItems.includes(label) ? "Pinned · Planned" : "Planned"}
              </span>
            </div>
          ))}
        </nav>
        <div className="rail-note">
          <span className="status-dot" aria-hidden="true" />
          <span>Local-first foundation</span>
        </div>
        <div className="rail-footer">BUILD · MANAGE · OBSERVE</div>
      </aside>

      <section className="main-panel">
        <header className="topbar">
          <div>
            <span className="eyebrow">MOONSHIPS / WORKSPACE</span>
            <h1>Moonships Workbench</h1>
          </div>
          <div className="topbar-actions">
            <button
              className="theme-toggle profile-trigger"
              type="button"
              onClick={onOpenProfile}
              aria-label="Change workspace profile"
              title="Change workspace profile"
            >
              Profile: {profileLabel}
            </button>
            <button
              className="theme-toggle command-palette-trigger"
              type="button"
              onClick={onOpenCommandPalette}
              aria-haspopup="dialog"
              aria-label="Open Command Palette"
              title="Open Command Palette"
            >
              Search commands
            </button>
            <button
              className="theme-toggle"
              type="button"
              onClick={onToggleTheme}
              aria-label={`Switch to ${theme === "dark" ? "light" : "dark"} theme`}
              title={`Switch to ${theme === "dark" ? "light" : "dark"} theme`}
            >
              {theme === "dark" ? "Light theme" : "Dark theme"}
            </button>
            <span className="phase-pill">
              <span className="status-dot" /> LOCAL-FIRST
            </span>
          </div>
        </header>
        <div className="page-content" id="main-content" tabIndex={-1}>
          {children}
        </div>
      </section>
    </main>
  );
}

function OverviewPage() {
  return (
    <div id="home">
      <section className="hero" aria-labelledby="hero-title">
        <div className="hero-orbit" aria-hidden="true">
          <span />
          <i />
          <b />
        </div>
        <div className="hero-copy">
          <p className="eyebrow">YOUR SPACE. YOUR SIGNAL.</p>
          <h2 id="hero-title">
            A clear starting point
            <br />
            for complex work.
          </h2>
          <p className="hero-description">
            Moonships is being built as a local-first technical workbench. This starter screen
            confirms the interface foundation; operational tools are not connected yet.
          </p>
          <span className="quiet-chip">PRE-IMPLEMENTATION SHELL</span>
        </div>
      </section>

      <section className="section-heading">
        <div>
          <p className="eyebrow">SYSTEM MAP / 01</p>
          <h2>Workbench overview</h2>
        </div>
        <span className="muted">No external services connected</span>
      </section>

      <div className="status-grid">
        {foundationChecks.map((item, index) => (
          <article className="status-card" key={item.label}>
            <div className="card-index">0{index + 1}</div>
            <h3>{item.label}</h3>
            <p>{item.detail}</p>
            {item.availability ? (
              <p className="availability-note">
                {describeModuleAvailability(item.availability).label}:{" "}
                {describeModuleAvailability(item.availability).description}
              </p>
            ) : null}
            <span className={item.status === "Planned" ? "state state-planned" : "state"}>
              <span className="state-dot" />
              {item.status}
            </span>
          </article>
        ))}
      </div>

      <footer className="page-footer">
        <span>MOONSHIPS WORKBENCH</span>
        <span>Build the core. Verify every signal.</span>
      </footer>
    </div>
  );
}

export function SettingsDialog({
  theme,
  onToggleTheme,
  onClose,
  hideToTray,
  hideToTrayStatus,
  onToggleHideToTray,
}: {
  theme: ThemePreference;
  onToggleTheme: () => void;
  onClose: () => void;
  hideToTray: boolean;
  hideToTrayStatus: string;
  onToggleHideToTray: () => void;
}) {
  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [onClose]);

  return (
    <div className="settings-backdrop" role="presentation">
      <section
        className="settings-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
      >
        <div className="settings-dialog-header">
          <div>
            <p className="eyebrow">WORKSPACE / PREFERENCES</p>
            <h2 id="settings-title">Settings</h2>
          </div>
          <button
            className="settings-close"
            type="button"
            onClick={onClose}
            aria-label="Close settings"
          >
            Close
          </button>
        </div>
        <div className="settings-row">
          <div>
            <h3>Appearance</h3>
            <p>Choose the theme used by this workspace.</p>
          </div>
          <button className="theme-toggle" type="button" onClick={onToggleTheme}>
            Switch to {theme === "dark" ? "light" : "dark"} theme
          </button>
        </div>
        <div className="settings-row">
          <div>
            <h3>Window behavior</h3>
            <p>Keep Moonships running in the system tray when you close the window.</p>
          </div>
          <label className="settings-checkbox">
            <input type="checkbox" checked={hideToTray} onChange={onToggleHideToTray} />
            <span>Hide to tray on close</span>
          </label>
        </div>
        <p className="settings-note" role="status" aria-live="polite">
          {hideToTrayStatus}
        </p>
        <section className="settings-planned" aria-labelledby="notifications-title">
          <div className="settings-planned-heading">
            <div>
              <p className="eyebrow">NOTIFICATIONS / PERMISSIONS</p>
              <h3 id="notifications-title">Notification and permission controls</h3>
            </div>
            <span className="nav-state">Planned</span>
          </div>
          <p className="settings-note">
            These capabilities are not connected yet. Controls stay disabled until a real permission
            flow and user-visible behavior are implemented.
          </p>
          <label className="settings-checkbox settings-checkbox-disabled">
            <input type="checkbox" disabled aria-label="Desktop notifications (Planned)" />
            <span>
              Desktop notifications <span className="nav-state">Planned</span>
            </span>
          </label>
          <label className="settings-checkbox settings-checkbox-disabled">
            <input
              type="checkbox"
              disabled
              aria-label="Background monitoring permission (Planned)"
            />
            <span>
              Background monitoring <span className="nav-state">Planned</span>
            </span>
          </label>
        </section>
      </section>
    </div>
  );
}

export function WorkspaceProfileDialog({
  currentProfile,
  onSelect,
  onClose,
  firstRun = false,
}: {
  currentProfile: WorkspaceProfileId;
  onSelect: (profile: WorkspaceProfileId) => void;
  onClose: () => void;
  firstRun?: boolean;
}) {
  const dialogRef = useRef<HTMLElement>(null);
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;

  useEffect(() => {
    const previouslyFocused =
      document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const dialog = dialogRef.current;
    const focusable = () =>
      Array.from(dialog?.querySelectorAll<HTMLElement>("*") ?? []).filter(
        (element) =>
          element.matches(
            'button:not(:disabled), input:not(:disabled), a[href], [tabindex]:not([tabindex="-1"])',
          ) && element.getAttribute("aria-hidden") !== "true",
      );
    focusable()[0]?.focus();

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onCloseRef.current();
        return;
      }
      if (event.key !== "Tab") return;
      const elements = focusable();
      if (elements.length === 0) {
        event.preventDefault();
        dialog?.focus();
        return;
      }
      const first = elements[0];
      const last = elements[elements.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    const onFocusIn = (event: FocusEvent) => {
      if (dialog && !dialog.contains(event.target as Node)) focusable()[0]?.focus();
    };
    document.addEventListener("keydown", onKeyDown);
    document.addEventListener("focusin", onFocusIn);
    return () => {
      document.removeEventListener("keydown", onKeyDown);
      document.removeEventListener("focusin", onFocusIn);
      if (previouslyFocused?.isConnected) previouslyFocused.focus();
    };
  }, []);

  return (
    <div className="settings-backdrop" role="presentation">
      <section
        ref={dialogRef}
        className="settings-dialog profile-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="profile-dialog-title"
        aria-describedby="profile-dialog-description"
        tabIndex={-1}
      >
        <div className="settings-dialog-header">
          <div>
            <p className="eyebrow">{firstRun ? "FIRST RUN / WELCOME" : "WORKSPACE / PROFILE"}</p>
            <h2 id="profile-dialog-title">
              {firstRun ? "Welcome to Moonships Workbench" : "Choose your workspace profile"}
            </h2>
          </div>
          <button
            className="settings-close"
            type="button"
            onClick={onClose}
            aria-label="Close workspace profile selector"
          >
            Close
          </button>
        </div>
        {firstRun ? (
          <p className="settings-note">
            Your workbench starts local-first and offline. No account, cloud connection, or API key
            is required. Tools and project creation that are not implemented stay marked Planned.
            Background monitoring and startup with Windows are not enabled automatically. You can
            change tray behavior later in Settings.
          </p>
        ) : null}
        <p className="settings-note" id="profile-dialog-description">
          Profiles only change suggested pinned sections and layout preferences. They never grant
          permissions or enable tools. Background tasks stay disabled unless separately configured.
          You can change your selection at any time.
        </p>
        <div className="profile-options">
          {WORKSPACE_PROFILES.map((profile) => (
            <button
              className={`profile-option${profile.id === currentProfile ? " profile-option-selected" : ""}`}
              key={profile.id}
              type="button"
              aria-pressed={profile.id === currentProfile}
              onClick={() => onSelect(profile.id)}
            >
              <span className="profile-option-copy">
                <strong>{profile.label}</strong>
                <span>{profile.description}</span>
                <small>Pinned: {profile.pinned.join(" · ")}</small>
              </span>
              {profile.id === currentProfile ? <span className="nav-state">Selected</span> : null}
            </button>
          ))}
        </div>
        <p className="settings-note">
          Saved locally in this WebView profile. No cloud sync is used.
        </p>
      </section>
    </div>
  );
}

export function CommandPreviewDialog({
  title,
  description,
  onClose,
}: {
  title: string;
  description: string;
  onClose: () => void;
}) {
  return (
    <div className="settings-backdrop" role="presentation">
      <section
        className="settings-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="command-preview-title"
      >
        <div className="settings-dialog-header">
          <div>
            <p className="eyebrow">REGISTERED COMMAND / PREVIEW</p>
            <h2 id="command-preview-title">{title}</h2>
          </div>
          <button
            className="settings-close"
            type="button"
            onClick={onClose}
            aria-label="Close command preview"
          >
            Close
          </button>
        </div>
        <p className="settings-note">{description}</p>
        <p className="settings-note" role="status">
          No changes have been made.
        </p>
        <div className="settings-dialog-actions">
          <button className="theme-toggle" type="button" onClick={onClose}>
            Back to workspace
          </button>
        </div>
      </section>
    </div>
  );
}

export function NotFoundPage({ requested }: { requested: string }) {
  return (
    <section className="hero" aria-labelledby="not-found-title">
      <div className="hero-copy">
        <p className="eyebrow">ROUTE / NOT FOUND</p>
        <h2 id="not-found-title">This page is not available.</h2>
        <p className="hero-description">
          The route <code>{requested || "(empty)"}</code> is not registered. No tools or data were
          changed.
        </p>
        <a className="nav-item nav-item-active" href="#/overview">
          Return to Home
        </a>
      </div>
    </section>
  );
}

function getInitialRoute(): RouteResolution {
  return resolveRoute(typeof window === "undefined" ? "#/overview" : window.location.hash);
}

function getBrowserStorage(): Storage | undefined {
  try {
    return window.localStorage;
  } catch {
    return undefined;
  }
}

export default function App() {
  const [route, setRoute] = useState<RouteResolution>(getInitialRoute);
  const [theme, setTheme] = useState<ThemePreference>(() =>
    readThemePreference(getBrowserStorage()),
  );
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [profileId, setProfileId] = useState<WorkspaceProfileId>(
    () => readWorkspaceProfile(getBrowserStorage()) ?? "developer",
  );
  const [firstRun, setFirstRun] = useState(
    () => typeof window !== "undefined" && readWorkspaceProfile(getBrowserStorage()) === null,
  );
  const [profileDialogOpen, setProfileDialogOpen] = useState(firstRun);
  const [commandPreview, setCommandPreview] = useState<{
    title: string;
    description: string;
  } | null>(null);
  const paletteTriggerRef = useRef<HTMLElement | null>(null);
  const [hideToTray, setHideToTray] = useState(false);
  const [hideToTrayStatus, setHideToTrayStatus] = useState("Loading preference…");

  useEffect(() => {
    applyThemePreference(theme, document.documentElement);
  }, [theme]);

  useEffect(() => {
    if (!commandPaletteOpen) paletteTriggerRef.current?.focus();
  }, [commandPaletteOpen]);

  useEffect(() => {
    const isEditableTarget = (target: EventTarget | null) => {
      if (!(target instanceof HTMLElement)) return false;
      return (
        target.isContentEditable ||
        Boolean(target.closest("input, textarea, select, [contenteditable=true], [role=textbox]"))
      );
    };
    const onGlobalKeyDown = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey) || event.key.toLowerCase() !== "k") return;
      if (event.defaultPrevented || isEditableTarget(event.target) || commandPaletteOpen) return;
      if (
        document.querySelector("[role=dialog]") ||
        settingsOpen ||
        profileDialogOpen ||
        commandPreview
      )
        return;
      event.preventDefault();
      paletteTriggerRef.current =
        document.activeElement instanceof HTMLElement ? document.activeElement : null;
      setCommandPaletteOpen(true);
    };
    window.addEventListener("keydown", onGlobalKeyDown);
    return () => window.removeEventListener("keydown", onGlobalKeyDown);
  }, [commandPaletteOpen, settingsOpen, profileDialogOpen, commandPreview]);

  const selectWorkspaceProfile = (nextProfile: WorkspaceProfileId) => {
    const saved = writeWorkspaceProfile(nextProfile, getBrowserStorage());
    setProfileId(nextProfile);
    setProfileDialogOpen(false);
    setFirstRun(false);
    if (!saved)
      setCommandPreview({
        title: "Profile not persisted",
        description: "Local storage is unavailable. This selection applies for this session only.",
      });
  };

  const selectPaletteCommand = (id: string) => {
    const command = getRegisteredCommand(id);
    setCommandPaletteOpen(false);
    if (!command) {
      setCommandPreview({
        title: "Command not registered",
        description: "This command ID is not allowlisted. Nothing was executed.",
      });
      return;
    }
    if (command.kind === "navigate") {
      window.location.hash = `#/${command.route}`;
      return;
    }
    if (command.kind === "settings") {
      setSettingsOpen(true);
      return;
    }
    setCommandPreview({
      title: `Preview: ${command.title}`,
      description:
        "This registered action would create a " +
        command.entity +
        ". Local persistence is not implemented yet, so no data has been created. This preview does not grant permissions or execute a tool.",
    });
  };

  const toggleTheme = () => {
    const nextTheme: ThemePreference = theme === "dark" ? "light" : "dark";
    writeThemePreference(nextTheme, getBrowserStorage());
    setTheme(nextTheme);
  };

  useEffect(() => {
    let cancelled = false;
    void invoke<{ enabled: boolean }>("get_hide_to_tray")
      .then(({ enabled }) => {
        if (!cancelled) {
          setHideToTray(enabled);
          setHideToTrayStatus("Preference loaded from this device.");
        }
      })
      .catch(() => {
        if (!cancelled) {
          setHideToTrayStatus("System tray preferences require the Moonships desktop app.");
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const toggleHideToTray = () => {
    const nextValue = !hideToTray;
    setHideToTray(nextValue);
    setHideToTrayStatus("Saving preference…");
    void invoke<{ enabled: boolean }>("set_hide_to_tray", {
      request: { enabled: nextValue },
    })
      .then(({ enabled }) => {
        setHideToTray(enabled);
        setHideToTrayStatus("Preference saved on this device.");
      })
      .catch(() => {
        setHideToTray(!nextValue);
        setHideToTrayStatus("Could not save. Open Settings in the Moonships desktop app.");
      });
  };

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    void listen("tray-settings", () => setSettingsOpen(true))
      .then((stopListening) => {
        if (cancelled) stopListening();
        else unlisten = stopListening;
      })
      .catch(() => {
        // Browser preview does not provide Tauri events; the app remains usable.
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    const updateRoute = () => setRoute(resolveRoute(window.location.hash));
    window.addEventListener("hashchange", updateRoute);
    return () => window.removeEventListener("hashchange", updateRoute);
  }, []);

  return (
    <AppShell
      theme={theme}
      onToggleTheme={toggleTheme}
      onOpenCommandPalette={() => {
        paletteTriggerRef.current =
          document.activeElement instanceof HTMLElement ? document.activeElement : null;
        setCommandPaletteOpen(true);
      }}
      profileLabel={getWorkspaceProfile(profileId).label}
      pinnedItems={getWorkspaceProfile(profileId).pinned}
      onOpenProfile={() => setProfileDialogOpen(true)}
    >
      {route.kind === "page" && route.id === "overview" ? (
        <OverviewPage />
      ) : (
        <NotFoundPage requested={route.kind === "not-found" ? route.requested : ""} />
      )}
      <CommandPalette
        open={commandPaletteOpen}
        commands={getPaletteCommands()}
        onClose={() => setCommandPaletteOpen(false)}
        onSelect={(command) => selectPaletteCommand(command.id)}
      />
      {profileDialogOpen ? (
        <WorkspaceProfileDialog
          currentProfile={profileId}
          onSelect={selectWorkspaceProfile}
          onClose={() => setProfileDialogOpen(false)}
          firstRun={firstRun}
        />
      ) : null}
      {commandPreview ? (
        <CommandPreviewDialog
          title={commandPreview.title}
          description={commandPreview.description}
          onClose={() => setCommandPreview(null)}
        />
      ) : null}
      {settingsOpen ? (
        <SettingsDialog
          theme={theme}
          onToggleTheme={toggleTheme}
          onClose={() => setSettingsOpen(false)}
          hideToTray={hideToTray}
          hideToTrayStatus={hideToTrayStatus}
          onToggleHideToTray={toggleHideToTray}
        />
      ) : null}
    </AppShell>
  );
}
