import { useState } from "react";
import { diffTextInputs, MAX_DIFF_INPUT_BYTES } from "./text-diff";
import { Button } from "./ui-primitives";

export function TextDiffDevToolsPanel() {
  const [before, setBefore] = useState("");
  const [after, setAfter] = useState("");
  const [result, setResult] = useState<ReturnType<typeof diffTextInputs> | null>(null);

  const beforeTooLarge = before.length > MAX_DIFF_INPUT_BYTES;
  const afterTooLarge = after.length > MAX_DIFF_INPUT_BYTES;

  function updateBefore(value: string) {
    setBefore(value);
    setResult(null);
  }

  function updateAfter(value: string) {
    setAfter(value);
    setResult(null);
  }

  return (
    <div className="json-devtools__workspace">
      <div className="json-devtools__actions">
        <div>
          <label htmlFor="diff-before-input">Original text</label>
          <textarea
            id="diff-before-input"
            className="json-devtools__editor"
            value={before}
            onChange={(event) => updateBefore(event.target.value)}
            spellCheck={false}
            aria-invalid={beforeTooLarge || undefined}
          />
          <p className="muted">Each input: up to 1 MiB UTF-8 and 50,000 lines.</p>
        </div>
        <div>
          <label htmlFor="diff-after-input">Updated text</label>
          <textarea
            id="diff-after-input"
            className="json-devtools__editor"
            value={after}
            onChange={(event) => updateAfter(event.target.value)}
            spellCheck={false}
            aria-invalid={afterTooLarge || undefined}
          />
          <p className="muted">Compared locally; no provider or network request.</p>
        </div>
      </div>
      <div className="json-devtools__actions">
        <Button
          onClick={() => setResult(diffTextInputs(before, after))}
          disabled={beforeTooLarge || afterTooLarge}
        >
          Compare text
        </Button>
      </div>

      {result === null ? (
        <p className="json-devtools__empty" role="status">
          The unified diff appears here after comparison.
        </p>
      ) : result.ok ? (
        <section className="json-devtools__result" aria-label="Text diff result">
          <h3 role="status">
            {result.changed ? "Differences found" : "Inputs match"} · original{" "}
            {(result.beforeBytes / 1024).toFixed(1)} KiB · updated{" "}
            {(result.afterBytes / 1024).toFixed(1)} KiB
          </h3>
          <textarea
            className="json-devtools__output"
            readOnly
            aria-label="Unified text diff"
            value={result.unified}
          />
        </section>
      ) : (
        <p className="json-devtools__error" role="alert">
          {result.message}
        </p>
      )}
    </div>
  );
}
