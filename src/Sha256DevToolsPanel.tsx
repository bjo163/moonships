import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "./ui-primitives";

const MAX_TEXT_BYTES = 1024 * 1024;

type FileHashResult = {
  cancelled: boolean;
  fileName: string | null;
  bytesHashed: number;
  sha256: string | null;
};

export function Sha256DevToolsPanel() {
  const [text, setText] = useState("");
  const [textHash, setTextHash] = useState<string | null>(null);
  const [textError, setTextError] = useState<string | null>(null);
  const [fileResult, setFileResult] = useState<FileHashResult | null>(null);
  const [fileError, setFileError] = useState<string | null>(null);
  const [fileNotice, setFileNotice] = useState<string | null>(null);
  const [filePending, setFilePending] = useState(false);

  async function hashText() {
    setTextHash(null);
    setTextError(null);
    try {
      setTextHash(await invoke<string>("hash_sha256_text", { text }));
    } catch (error) {
      setTextError(errorMessage(error));
    }
  }

  async function hashFile() {
    setFilePending(true);
    setFileError(null);
    setFileResult(null);
    setFileNotice(null);
    try {
      const result = await invoke<FileHashResult>("hash_sha256_selected_file");
      if (result.cancelled) setFileNotice("File selection cancelled. No file was read.");
      else setFileResult(result);
    } catch (error) {
      setFileError(errorMessage(error));
    } finally {
      setFilePending(false);
    }
  }

  const textBytes =
    text.length > MAX_TEXT_BYTES ? MAX_TEXT_BYTES + 1 : new TextEncoder().encode(text).byteLength;

  return (
    <div className="json-devtools__workspace">
      <section aria-labelledby="sha256-text-title">
        <h3 id="sha256-text-title">Hash text</h3>
        <label htmlFor="sha256-text-input">Text input</label>
        <textarea
          id="sha256-text-input"
          className="json-devtools__editor"
          value={text}
          onChange={(event) => {
            setText(event.target.value);
            setTextHash(null);
            setTextError(null);
          }}
          aria-describedby="sha256-text-limit"
          aria-invalid={textError !== null}
        />
        <div className="json-devtools__actions">
          <span id="sha256-text-limit" className="muted">
            UTF-8 text limited to 1 MiB · local only ({(textBytes / 1024).toFixed(1)} KiB)
          </span>
          <Button onClick={hashText} disabled={textBytes > MAX_TEXT_BYTES}>
            Calculate SHA-256
          </Button>
        </div>
        {textBytes > MAX_TEXT_BYTES ? (
          <p className="json-devtools__error" role="alert">
            Text exceeds the 1 MiB limit.
          </p>
        ) : textHash ? (
          <p className="json-devtools__result" role="status">
            SHA-256 · {textHash}
          </p>
        ) : textError ? (
          <p className="json-devtools__error" role="alert">
            {textError}
          </p>
        ) : (
          <p className="json-devtools__empty" role="status">
            Text digest appears here after calculation.
          </p>
        )}
      </section>

      <section aria-labelledby="sha256-file-title">
        <h3 id="sha256-file-title">Hash a file</h3>
        <p className="muted">
          Choose a file in the native picker. Moonships reads it locally in 64 KiB chunks, up to 2
          GiB. Only the leaf name is displayed; the full selected path is not returned or stored.
        </p>
        <Button onClick={() => void hashFile()} disabled={filePending}>
          {filePending ? "Calculating SHA-256…" : "Choose file and calculate SHA-256"}
        </Button>
        {fileResult?.sha256 ? (
          <p className="json-devtools__result" role="status">
            SHA-256 for {fileResult.fileName ?? "selected file"} ·{" "}
            {fileResult.bytesHashed.toLocaleString()} bytes · {fileResult.sha256}
          </p>
        ) : fileError ? (
          <p className="json-devtools__error" role="alert">
            {fileError}
          </p>
        ) : fileNotice ? (
          <p className="json-devtools__empty" role="status">
            {fileNotice}
          </p>
        ) : (
          <p className="json-devtools__empty" role="status">
            File digest appears here after selection and calculation.
          </p>
        )}
      </section>
    </div>
  );
}

function errorMessage(error: unknown): string {
  const message = typeof error === "string" ? error : error instanceof Error ? error.message : "";
  if (message.includes("VALIDATION_FAILED:")) return message.split("VALIDATION_FAILED:")[1].trim();
  if (message.includes("UNSUPPORTED:")) return message.split("UNSUPPORTED:")[1].trim();
  if (message.includes("IO_ERROR:")) return message.split("IO_ERROR:")[1].trim();
  return "Could not calculate the SHA-256 digest.";
}
