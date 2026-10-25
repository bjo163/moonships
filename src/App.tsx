const foundationChecks = [
  { label: "Desktop runtime", detail: "Tauri 2 · Rust", status: "Foundation" },
  { label: "Interface", detail: "React · TypeScript · Vite", status: "Foundation" },
  { label: "Data and tools", detail: "Not connected yet", status: "Planned" },
];

export default function App() {
  return (
    <main className="app-frame">
      <aside className="rail" aria-label="Primary navigation">
        <a className="brand" href="#home" aria-label="Moonships home">
          <span className="brand-mark" aria-hidden="true">M</span>
          <span className="brand-name">MOONSHIPS<span>WORKBENCH</span></span>
        </a>
        <div className="rail-label">WORKSPACE</div>
        <a className="nav-item nav-item-active" href="#home" aria-current="page">
          <span aria-hidden="true">◈</span> Overview
        </a>
        <div className="rail-note">
          <span className="status-dot" aria-hidden="true" />
          <span>Local-first foundation</span>
        </div>
        <div className="rail-footer">BUILD · MANAGE · OBSERVE</div>
      </aside>

      <section className="main-panel" id="home">
        <header className="topbar">
          <div><span className="eyebrow">DESKTOP FOUNDATION / 00</span><h1>Workbench overview</h1></div>
          <span className="phase-pill"><span className="status-dot" /> FOUNDATION</span>
        </header>

        <section className="hero" aria-labelledby="hero-title">
          <div className="hero-orbit" aria-hidden="true"><span /><i /><b /></div>
          <div className="hero-copy">
            <p className="eyebrow">YOUR SPACE. YOUR SIGNAL.</p>
            <h2 id="hero-title">A clear starting point<br />for complex work.</h2>
            <p className="hero-description">
              Moonships is being built as a local-first technical workbench. This starter screen confirms
              the interface foundation; operational tools are not connected yet.
            </p>
            <span className="quiet-chip">PRE-IMPLEMENTATION SHELL</span>
          </div>
        </section>

        <section className="section-heading">
          <div><p className="eyebrow">SYSTEM MAP / 01</p><h2>Foundation status</h2></div>
          <span className="muted">No external services connected</span>
        </section>

        <div className="status-grid">
          {foundationChecks.map((item, index) => (
            <article className="status-card" key={item.label}>
              <div className="card-index">0{index + 1}</div>
              <h3>{item.label}</h3>
              <p>{item.detail}</p>
              <span className={item.status === "Planned" ? "state state-planned" : "state"}>
                <span className="state-dot" />{item.status}
              </span>
            </article>
          ))}
        </div>

        <footer className="page-footer">
          <span>MOONSHIPS WORKBENCH</span>
          <span>Build the core. Verify every signal.</span>
        </footer>
      </section>
    </main>
  );
}
