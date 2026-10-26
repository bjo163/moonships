import { useState } from "react";
import { Base64DevToolsPanel } from "./Base64DevToolsPanel";
import { Sha256DevToolsPanel } from "./Sha256DevToolsPanel";
import { TextDiffDevToolsPanel } from "./TextDiffDevToolsPanel";
import { formatJsonInput, MAX_JSON_INPUT_BYTES } from "./json-formatter";
import { UrlDevToolsPanel } from "./UrlDevToolsPanel";
import { UuidDevToolsPanel } from "./UuidDevToolsPanel";
import { Button } from "./ui-primitives";
import { YamlDevToolsPanel } from "./YamlDevToolsPanel";

export function DevToolsPage() {
  const [activeTool, setActiveTool] = useState<
    "json" | "yaml" | "base64" | "url" | "uuid" | "sha256" | "diff"
  >("json");

  return (
    <section className="json-devtools" aria-labelledby="devtools-title">
      <header className="json-devtools__header">
        <div>
          <p className="eyebrow">TOOLBOX / OFFLINE DEVTOOLS</p>
          <h2 id="devtools-title">Offline DevTools</h2>
          <p className="muted">
            Text stays on this device; these formatters make no provider or network requests.
          </p>
        </div>
      </header>
      <fieldset className="json-devtools__tabs">
        <legend className="visually-hidden">Select a local DevTool</legend>
        <Button
          variant={activeTool === "json" ? "primary" : "secondary"}
          aria-pressed={activeTool === "json"}
          onClick={() => setActiveTool("json")}
        >
          JSON formatter
        </Button>
        <Button
          variant={activeTool === "yaml" ? "primary" : "secondary"}
          aria-pressed={activeTool === "yaml"}
          onClick={() => setActiveTool("yaml")}
        >
          YAML formatter
        </Button>
        <Button
          variant={activeTool === "base64" ? "primary" : "secondary"}
          aria-pressed={activeTool === "base64"}
          onClick={() => setActiveTool("base64")}
        >
          Base64
        </Button>
        <Button
          variant={activeTool === "url" ? "primary" : "secondary"}
          aria-pressed={activeTool === "url"}
          onClick={() => setActiveTool("url")}
        >
          URL
        </Button>
        <Button
          variant={activeTool === "uuid" ? "primary" : "secondary"}
          aria-pressed={activeTool === "uuid"}
          onClick={() => setActiveTool("uuid")}
        >
          UUID
        </Button>
        <Button
          variant={activeTool === "sha256" ? "primary" : "secondary"}
          aria-pressed={activeTool === "sha256"}
          onClick={() => setActiveTool("sha256")}
        >
          SHA-256
        </Button>
        <Button
          variant={activeTool === "diff" ? "primary" : "secondary"}
          aria-pressed={activeTool === "diff"}
          onClick={() => setActiveTool("diff")}
        >
          Text diff
        </Button>
      </fieldset>
      {activeTool === "json" ? (
        <JsonFormatterPanel />
      ) : activeTool === "yaml" ? (
        <YamlDevToolsPanel />
      ) : activeTool === "base64" ? (
        <Base64DevToolsPanel />
      ) : activeTool === "url" ? (
        <UrlDevToolsPanel />
      ) : activeTool === "uuid" ? (
        <UuidDevToolsPanel />
      ) : activeTool === "sha256" ? (
        <Sha256DevToolsPanel />
      ) : (
        <TextDiffDevToolsPanel />
      )}
    </section>
  );
}

function JsonFormatterPanel() {
  const [input, setInput] = useState("");
  const [result, setResult] = useState<ReturnType<typeof formatJsonInput> | null>(null);

  function format() {
    setResult(formatJsonInput(input));
  }

  return (
    <div className="json-devtools__workspace">
      <label htmlFor="json-devtools-input">JSON input</label>
      <textarea
        id="json-devtools-input"
        className="json-devtools__editor"
        value={input}
        onChange={(event) => {
          setInput(event.target.value);
          setResult(null);
        }}
        spellCheck={false}
        aria-describedby="json-devtools-limit"
        aria-invalid={result?.ok === false}
      />
      <div className="json-devtools__actions">
        <span id="json-devtools-limit" className="muted">
          Limit: {MAX_JSON_INPUT_BYTES / 1024 / 1024} MiB · local only
        </span>
        <Button onClick={format}>Validate &amp; format</Button>
      </div>

      {result ? (
        result.ok ? (
          <section className="json-devtools__result" aria-label="Formatted JSON result">
            <h3 role="status">Valid JSON · {(result.inputBytes / 1024).toFixed(1)} KiB</h3>
            <textarea
              className="json-devtools__output"
              readOnly
              aria-label="Formatted JSON"
              value={result.formatted}
            />
          </section>
        ) : (
          <p className="json-devtools__error" role="alert">
            {result.message}
          </p>
        )
      ) : (
        <p className="json-devtools__empty" role="status">
          Results will appear here after validation.
        </p>
      )}
    </div>
  );
}
