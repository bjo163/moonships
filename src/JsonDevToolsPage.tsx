import { useState } from "react";
import { formatJsonInput, MAX_JSON_INPUT_BYTES } from "./json-formatter";
import { Button } from "./ui-primitives";

export function JsonDevToolsPage() {
  const [input, setInput] = useState("");
  const [result, setResult] = useState<ReturnType<typeof formatJsonInput> | null>(null);

  function format() {
    setResult(formatJsonInput(input));
  }

  return (
    <section className="json-devtools" aria-labelledby="json-devtools-title">
      <header className="json-devtools__header">
        <div>
          <p className="eyebrow">TOOLBOX / OFFLINE DEVTOOLS</p>
          <h2 id="json-devtools-title">JSON formatter &amp; validator</h2>
          <p className="muted">
            Validate and format JSON locally. Input is not sent to a provider or saved by this tool.
          </p>
        </div>
      </header>

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
    </section>
  );
}
