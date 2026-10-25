import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";

export type PaletteRisk = "read-only" | "state-change" | "destructive" | "privileged" | "remote";

export type PaletteCommand = {
  id: string;
  title: string;
  description: string;
  category: string;
  risk: PaletteRisk;
  keywords?: string[];
};

type CommandPaletteProps = {
  commands: PaletteCommand[];
  open: boolean;
  loading?: boolean;
  onClose: () => void;
  onSelect: (command: PaletteCommand) => void;
};

const riskLabels: Record<PaletteRisk, string> = {
  "read-only": "Read-only",
  "state-change": "State change",
  destructive: "Destructive · preview required",
  privileged: "Privileged · approval required",
  remote: "Remote · approval required",
};

export function filterPaletteCommands(commands: PaletteCommand[], query: string): PaletteCommand[] {
  const normalized = query.trim().toLowerCase();
  if (!normalized) return commands;
  return commands.filter((command) =>
    [command.title, command.description, command.category, ...(command.keywords ?? [])].some(
      (value) => value.toLowerCase().includes(normalized),
    ),
  );
}

export function CommandPalette({
  commands,
  open,
  loading = false,
  onClose,
  onSelect,
}: CommandPaletteProps) {
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const results = useMemo(() => filterPaletteCommands(commands, query), [commands, query]);

  useEffect(() => {
    if (open) searchInputRef.current?.focus();
  }, [open]);

  if (!open) return null;

  const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    } else if (event.key === "ArrowDown" && results.length > 0) {
      event.preventDefault();
      setActiveIndex((index) => (index + 1) % results.length);
    } else if (event.key === "ArrowUp" && results.length > 0) {
      event.preventDefault();
      setActiveIndex((index) => (index - 1 + results.length) % results.length);
    } else if (event.key === "Enter" && results[activeIndex]) {
      event.preventDefault();
      onSelect(results[activeIndex]);
    }
  };

  return (
    <div className="command-palette-backdrop" role="presentation">
      <section
        className="command-palette"
        role="dialog"
        aria-modal="true"
        aria-labelledby="command-palette-title"
        aria-describedby="command-palette-help"
      >
        <div className="command-palette-heading">
          <div>
            <p className="eyebrow">QUICK ACCESS / LOCAL COMMANDS</p>
            <h2 id="command-palette-title">Command Palette</h2>
          </div>
          <button
            className="settings-close"
            type="button"
            onClick={onClose}
            aria-label="Close Command Palette"
          >
            Close
          </button>
        </div>
        <label className="command-palette-search-label" htmlFor="command-palette-search">
          Search commands
        </label>
        <input
          ref={searchInputRef}
          id="command-palette-search"
          className="command-palette-search"
          type="search"
          placeholder="Search pages and registered commands…"
          value={query}
          onChange={(event) => {
            setQuery(event.target.value);
            setActiveIndex(0);
          }}
          onKeyDown={handleKeyDown}
          aria-controls="command-palette-results"
          aria-activedescendant={
            results[activeIndex] ? `palette-command-${results[activeIndex].id}` : undefined
          }
        />
        <p id="command-palette-help" className="command-palette-help">
          Use ↑ and ↓ to navigate, Enter to select, and Esc to close. Selecting a result does not
          execute a tool.
        </p>
        <div
          id="command-palette-results"
          className="command-palette-results"
          role="listbox"
          aria-label="Command results"
          aria-busy={loading}
        >
          {loading ? (
            <p className="command-palette-state" role="status">
              Loading commands…
            </p>
          ) : results.length === 0 ? (
            <p className="command-palette-state" role="status">
              {query.trim() ? "No commands match this search." : "No commands are registered yet."}
            </p>
          ) : (
            results.map((command, index) => (
              <button
                id={`palette-command-${command.id}`}
                className={`command-palette-result${index === activeIndex ? " command-palette-result-active" : ""}`}
                key={command.id}
                type="button"
                role="option"
                aria-selected={index === activeIndex}
                onMouseEnter={() => setActiveIndex(index)}
                onClick={() => onSelect(command)}
              >
                <span className="command-palette-result-copy">
                  <strong>{command.title}</strong>
                  <span>{command.description}</span>
                </span>
                <span className="command-palette-result-meta">
                  <span className="command-palette-category">{command.category}</span>
                  <span className={`command-palette-risk command-palette-risk-${command.risk}`}>
                    {riskLabels[command.risk]}
                  </span>
                </span>
              </button>
            ))
          )}
        </div>
        <div className="command-palette-footer">
          <span>LOCAL-FIRST · DETERMINISTIC SEARCH</span>
          <span>Esc to close</span>
        </div>
      </section>
    </div>
  );
}
