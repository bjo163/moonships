import { useState } from "react";
import { Button } from "./ui-primitives";
import { transformUrlInput, type UrlOperation } from "./url-tools";

const operations: { value: UrlOperation; label: string }[] = [
  { value: "encode_component", label: "Encode component" },
  { value: "decode_component", label: "Decode component" },
  { value: "encode_full_url", label: "Encode full URL" },
  { value: "decode_full_url", label: "Decode full URL" },
];

export function UrlDevToolsPanel() {
  const [operation, setOperation] = useState<UrlOperation>("encode_component");
  const [input, setInput] = useState("");
  const [result, setResult] = useState<ReturnType<typeof transformUrlInput> | null>(null);
  const fullUrl = operation.endsWith("full_url");
  const label = fullUrl ? "Absolute URL input" : "URL component input";

  return (
    <div className="json-devtools__workspace">
      <fieldset className="json-devtools__tabs">
        <legend className="visually-hidden">Select a URL operation</legend>
        {operations.map((item) => (
          <Button
            key={item.value}
            variant={operation === item.value ? "primary" : "secondary"}
            aria-pressed={operation === item.value}
            onClick={() => {
              setOperation(item.value);
              setInput("");
              setResult(null);
            }}
          >
            {item.label}
          </Button>
        ))}
      </fieldset>

      <p className="muted">
        {fullUrl
          ? "Full URL mode requires an absolute URL and preserves URL separators such as /, ?, & and #."
          : "Component mode encodes reserved characters as data within one URL component."}
      </p>
      <label htmlFor="url-devtools-input">{label}</label>
      <textarea
        id="url-devtools-input"
        className="json-devtools__editor"
        value={input}
        onChange={(event) => {
          setInput(event.target.value);
          setResult(null);
        }}
        spellCheck={false}
        aria-describedby="url-devtools-limit"
        aria-invalid={result?.ok === false}
      />
      <div className="json-devtools__actions">
        <span id="url-devtools-limit" className="muted">
          UTF-8 input limited to 1 MiB · local only
        </span>
        <Button onClick={() => setResult(transformUrlInput(input, operation))}>
          {operation.startsWith("encode") ? "Encode URL" : "Decode URL"}
        </Button>
      </div>

      {result ? (
        result.ok ? (
          <section className="json-devtools__result" aria-label="URL conversion result">
            <h3 role="status">
              Conversion complete · {(result.inputBytes / 1024).toFixed(1)} KiB input
            </h3>
            <textarea
              className="json-devtools__output"
              readOnly
              aria-label="URL output"
              value={result.value}
            />
          </section>
        ) : (
          <p className="json-devtools__error" role="alert">
            {result.message}
          </p>
        )
      ) : (
        <p className="json-devtools__empty" role="status">
          Results will appear here after conversion.
        </p>
      )}
    </div>
  );
}
