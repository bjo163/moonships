import { useEffect, useState, type ReactNode } from "react";
import { resolveRoute, type RouteResolution } from "./route-registry";
import { applyThemePreference, readThemePreference, writeThemePreference, type ThemePreference } from "./theme-preference";

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

const foundationChecks = [
  { label: "Desktop runtime", detail: "Tauri 2 · Rust", status: "Foundation" },
  { label: "Interface", detail: "React · TypeScript · Vite", status: "Foundation" },
  { label: "Data and tools", detail: "Not connected yet", status: "Planned" },
];

type AppShellProps = {
  children: ReactNode;
};

export function AppShell({ children }: AppShellProps) {
  return (
    <main className="app-frame">
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
            <div className="nav-item nav-item-planned" key={label} aria-disabled="true">
              <span>{label}</span>
              <span className="nav-state">Planned</span>
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
              className="theme-toggle"
              type="button"
              onClick={toggleTheme}
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
        <div className="page-content">{children}</div>
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

  useEffect(() => {
    applyThemePreference(theme, document.documentElement);
  }, [theme]);

  const toggleTheme = () => {
    const nextTheme: ThemePreference = theme === "dark" ? "light" : "dark";
    writeThemePreference(nextTheme, getBrowserStorage());
    setTheme(nextTheme);
  };

  useEffect(() => {
    const updateRoute = () => setRoute(resolveRoute(window.location.hash));
    window.addEventListener("hashchange", updateRoute);
    return () => window.removeEventListener("hashchange", updateRoute);
  }, []);

  return (
    <AppShell>
      {route.kind === "page" && route.id === "overview" ? (
        <OverviewPage />
      ) : (
        <NotFoundPage requested={route.kind === "not-found" ? route.requested : ""} />
      )}
    </AppShell>
  );
}
