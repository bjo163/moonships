import { useState } from "react";
import { Button } from "./ui-primitives";
import { formatYamlInput, MAX_YAML_INPUT_BYTES } from "./yaml-formatter";

export function YamlDevToolsPanel() {
  const [input, setInput] = useState("");
  const [result, setResult] = useState<ReturnType<typeof formatYamlInput> | null>(null);

  return (
    <div className="json-devtools__workspace">
      <label htmlFor="yaml-devtools-input">YAML input</label>
      <textarea
        id="yaml-devtools-input"
        className="json-devtools__editor"
        value={input}
        onChange={(event) => {
          setInput(event.target.value);
          setResult(null);
        }}
        spellCheck={false}
        aria-describedby="yaml-devtools-limit"
        aria-invalid={result?.ok === false}
      />
      <div className="json-devtools__actions">
        <span id="yaml-devtools-limit" className="muted">
          YAML 1.2 · {MAX_YAML_INPUT_BYTES / 1024 / 1024} MiB limit · local only
        </span>
        <Button onClick={() => setResult(formatYamlInput(input))}>Validate &amp; format</Button>
      </div>
      {result ? (
        result.ok ? (
          <section className="json-devtools__result" aria-label="Formatted YAML result">
            <h3 role="status">
              Valid YAML · {(result.inputBytes / 1024).toFixed(1)} KiB
              {result.warnings.length > 0 ? ` · ${result.warnings.length} parser warning(s)` : ""}
            </h3>
            <textarea
              className="json-devtools__output"
              readOnly
              aria-label="Formatted YAML"
              value={result.formatted}
            />
          </section>
        ) : (
          <div className="json-devtools__error" role="alert">
            <p>{result.message}</p>
            {result.issues.length > 0 ? (
              <ul aria-label="YAML parser issues">
                {result.issues.map((issue) => (
                  <li key={`${issue.code}-${issue.line ?? "none"}-${issue.column ?? "none"}`}>
                    {issue.code}
                    {issue.line ? ` · line ${issue.line}, column ${issue.column ?? 1}` : ""}
                  </li>
                ))}
              </ul>
            ) : null}
          </div>
        )
      ) : (
        <p className="json-devtools__empty" role="status">
          Results will appear here after validation.
        </p>
      )}
    </div>
  );
}
