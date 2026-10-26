import { useState } from "react";
import { generateUuidV4, validateUuid } from "./uuid-tools";
import { Button } from "./ui-primitives";

export function UuidDevToolsPanel() {
  const [generated, setGenerated] = useState<string | null>(null);
  const [input, setInput] = useState("");
  const [valid, setValid] = useState<boolean | null>(null);
  const [generationError, setGenerationError] = useState<string | null>(null);

  function generate() {
    const result = generateUuidV4(globalThis.crypto);
    setGenerationError(result.ok ? null : result.message);
    setGenerated(result.ok ? result.value : null);
  }

  return (
    <div className="json-devtools__workspace">
      <section aria-labelledby="uuid-generate-title">
        <h3 id="uuid-generate-title">Generate UUID v4</h3>
        <p className="muted">
          Uses the browser’s cryptographically secure random UUID API. Local only.
        </p>
        <Button onClick={generate}>Generate UUID</Button>
        {generated ? (
          <p className="json-devtools__result" role="status">
            <label htmlFor="uuid-generated">UUID v4</label>
            <input id="uuid-generated" readOnly value={generated} />
          </p>
        ) : generationError ? (
          <p className="json-devtools__error" role="alert">
            {generationError}
          </p>
        ) : (
          <p className="json-devtools__empty" role="status">
            A UUID appears here after generation.
          </p>
        )}
      </section>

      <section aria-labelledby="uuid-validate-title">
        <h3 id="uuid-validate-title">Validate UUID</h3>
        <p className="muted">
          Accepts canonical UUID versions 1–8 with the RFC variant, or the Nil UUID.
        </p>
        <label htmlFor="uuid-validation-input">UUID input</label>
        <input
          id="uuid-validation-input"
          value={input}
          onChange={(event) => {
            setInput(event.target.value);
            setValid(null);
          }}
          aria-invalid={valid === false}
        />
        <Button onClick={() => setValid(validateUuid(input))}>Validate UUID</Button>
        {valid === null ? (
          <p className="json-devtools__empty" role="status">
            Validation result appears here.
          </p>
        ) : valid ? (
          <p className="json-devtools__result" role="status">
            Valid UUID
          </p>
        ) : (
          <p className="json-devtools__error" role="alert">
            Invalid UUID format or variant.
          </p>
        )}
      </section>
    </div>
  );
}
