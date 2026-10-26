import { useState } from "react";
import {
  type Base64DecodeResult,
  type Base64EncodeResult,
  decodeBase64,
  encodeHexBytes,
  encodeUtf8Text,
  MAX_BASE64_BYTES,
} from "./base64-tools";
import { Button } from "./ui-primitives";

type Base64Mode = "text" | "hex" | "decode";
type Base64Result = Base64EncodeResult | Base64DecodeResult;

export function Base64DevToolsPanel() {
  const [mode, setMode] = useState<Base64Mode>("text");
  const [input, setInput] = useState("");
  const [result, setResult] = useState<Base64Result | null>(null);
  const label =
    mode === "text"
      ? "UTF-8 text input"
      : mode === "hex"
        ? "Hexadecimal byte input"
        : "Base64 input";

  function run() {
    setResult(
      mode === "text"
        ? encodeUtf8Text(input)
        : mode === "hex"
          ? encodeHexBytes(input)
          : decodeBase64(input),
    );
  }

  return (
    <div className="json-devtools__workspace">
      <fieldset className="json-devtools__tabs">
        <legend className="visually-hidden">Select a Base64 operation</legend>
        {(
          [
            ["text", "Text → Base64"],
            ["hex", "Hex → Base64"],
            ["decode", "Base64 → bytes"],
          ] as const
        ).map(([value, title]) => (
          <Button
            key={value}
            variant={mode === value ? "primary" : "secondary"}
            aria-pressed={mode === value}
            onClick={() => {
              setMode(value);
              setInput("");
              setResult(null);
            }}
          >
            {title}
          </Button>
        ))}
      </fieldset>

      <label htmlFor="base64-devtools-input">{label}</label>
      <textarea
        id="base64-devtools-input"
        className="json-devtools__editor"
        value={input}
        onChange={(event) => {
          setInput(event.target.value);
          setResult(null);
        }}
        spellCheck={false}
        aria-describedby="base64-devtools-limit"
        aria-invalid={result?.ok === false}
      />
      <div className="json-devtools__actions">
        <span id="base64-devtools-limit" className="muted">
          UTF-8 or decoded bytes limited to {MAX_BASE64_BYTES / 1024 / 1024} MiB · local only
        </span>
        <Button onClick={run}>{mode === "decode" ? "Decode Base64" : "Encode Base64"}</Button>
      </div>

      {result ? (
        result.ok ? (
          <section className="json-devtools__result" aria-label="Base64 result">
            {"base64" in result ? (
              <>
                <h3 role="status">Encoded {result.byteLength} bytes</h3>
                <textarea
                  className="json-devtools__output"
                  readOnly
                  aria-label="Base64 output"
                  value={result.base64}
                />
              </>
            ) : result.kind === "text" ? (
              <>
                <h3 role="status">Decoded as UTF-8 text · {result.byteLength} bytes</h3>
                <textarea
                  className="json-devtools__output"
                  readOnly
                  aria-label="Decoded UTF-8 text"
                  value={result.text}
                />
              </>
            ) : (
              <div role="status" className="base64-devtools__binary">
                <h3>Decoded as binary · {result.byteLength} bytes</h3>
                <p>
                  Not valid UTF-8. Hex preview: <code>{result.hexPreview || "(empty)"}</code>
                  {result.truncated ? " … (preview limited to 64 bytes)" : ""}
                </p>
              </div>
            )}
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
