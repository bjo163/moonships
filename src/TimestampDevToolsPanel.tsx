import { useState } from "react";
import {
  convertIsoTimestamp,
  convertUnixTimestamp,
  type TimestampResult,
  type UnixTimestampUnit,
} from "./timestamp-tools";
import { Button } from "./ui-primitives";

type TimestampInputMode = "unix" | "iso";

export function TimestampDevToolsPanel() {
  const [mode, setMode] = useState<TimestampInputMode>("unix");
  const [unit, setUnit] = useState<UnixTimestampUnit>("seconds");
  const [input, setInput] = useState("");
  const [result, setResult] = useState<TimestampResult | null>(null);

  function changeMode(nextMode: TimestampInputMode) {
    setMode(nextMode);
    setInput("");
    setResult(null);
  }

  function convert() {
    setResult(mode === "unix" ? convertUnixTimestamp(input, unit) : convertIsoTimestamp(input));
  }

  return (
    <div className="json-devtools__workspace">
      <fieldset className="json-devtools__tabs">
        <legend className="visually-hidden">Select timestamp input format</legend>
        <Button
          variant={mode === "unix" ? "primary" : "secondary"}
          aria-pressed={mode === "unix"}
          onClick={() => changeMode("unix")}
        >
          Unix timestamp
        </Button>
        <Button
          variant={mode === "iso" ? "primary" : "secondary"}
          aria-pressed={mode === "iso"}
          onClick={() => changeMode("iso")}
        >
          ISO 8601 date
        </Button>
      </fieldset>

      <p className="muted">
        {mode === "unix"
          ? "Use a whole-number timestamp and choose seconds or milliseconds."
          : "Use a full ISO 8601 date and time with a known Z or numeric timezone offset."}
      </p>
      {mode === "unix" ? (
        <>
          <label htmlFor="timestamp-unit">Unix timestamp unit</label>
          <select
            id="timestamp-unit"
            value={unit}
            onChange={(event) => setUnit(event.target.value as UnixTimestampUnit)}
          >
            <option value="seconds">Seconds</option>
            <option value="milliseconds">Milliseconds</option>
          </select>
          <label htmlFor="timestamp-input">Unix timestamp input</label>
        </>
      ) : (
        <label htmlFor="timestamp-input">ISO 8601 date and time input</label>
      )}
      <input
        id="timestamp-input"
        value={input}
        onChange={(event) => {
          setInput(event.target.value);
          setResult(null);
        }}
        aria-invalid={result?.ok === false}
      />
      <div className="json-devtools__actions">
        <span className="muted">Local conversion · no provider or network request</span>
        <Button onClick={convert}>Convert timestamp</Button>
      </div>

      {result === null ? (
        <p className="json-devtools__empty" role="status">
          Converted values appear here after conversion.
        </p>
      ) : result.ok ? (
        <section className="json-devtools__result" aria-label="Timestamp conversion result">
          <h3 role="status">Timestamp converted</h3>
          <dl>
            <dt>ISO 8601 in UTC</dt>
            <dd>{result.isoUtc}</dd>
            <dt>Local time ({result.localTimeZone})</dt>
            <dd>{result.localTime}</dd>
            <dt>Unix timestamp in seconds</dt>
            <dd>{result.unixSeconds}</dd>
            <dt>Unix timestamp in milliseconds</dt>
            <dd>{result.unixMilliseconds}</dd>
          </dl>
        </section>
      ) : (
        <p className="json-devtools__error" role="alert">
          {result.message}
        </p>
      )}
    </div>
  );
}
