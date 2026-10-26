import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { type OpenApiValidation, validateOpenApiDocument } from "./openapi-validation";
import { Button } from "./ui-primitives";

type SelectedDocument = { fileName: string; content: string };

export function OpenApiDocumentPage() {
  const [selected, setSelected] = useState<SelectedDocument | null>(null);
  const [validation, setValidation] = useState<OpenApiValidation | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const openDocument = async () => {
    if (busy) return;
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const document = await invoke<SelectedDocument | null>("select_openapi_document");
      if (!document) {
        setNotice("File selection cancelled.");
        return;
      }
      setSelected(document);
      setValidation(validateOpenApiDocument(document.content));
    } catch (reason) {
      setError(
        typeof reason === "string"
          ? reason
          : reason instanceof Error
            ? reason.message
            : "OpenAPI document could not be opened.",
      );
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="tasks-page" aria-labelledby="openapi-title">
      <header className="tasks-page__header">
        <div>
          <p className="eyebrow">TOOLBOX / LOCAL DOCUMENT</p>
          <h2 id="openapi-title">OpenAPI Viewer</h2>
          <p className="muted">
            Review a local OpenAPI 3.0 or 3.1 JSON/YAML document. Moonships does not contact its
            servers.
          </p>
        </div>
      </header>
      <div className="notice-card" role="note">
        Select a UTF-8 document up to 1 MiB. Validation checks syntax, duplicate YAML keys, OpenAPI
        version, info title/version and paths; it does not validate every OpenAPI rule or resolve
        references.
      </div>
      {error ? (
        <div role="alert" className="inline-alert">
          {error}
        </div>
      ) : null}
      {notice ? (
        <p role="status" className="inline-status">
          {notice}
        </p>
      ) : null}
      <Button onClick={() => void openDocument()} disabled={busy}>
        {busy ? "Reading…" : "Choose OpenAPI document"}
      </Button>
      {selected && validation ? (
        <section className="source-form-section" aria-labelledby="openapi-result-title">
          <h3 id="openapi-result-title">{selected.fileName}</h3>
          <p role={validation.valid ? "status" : "alert"}>
            {validation.valid
              ? "Valid OpenAPI document structure"
              : "Document is malformed or unsupported"}
          </p>
          <dl className="intelligence-metadata">
            <div>
              <dt>OpenAPI version</dt>
              <dd>{validation.version ?? "Not recognized"}</dd>
            </div>
            <div>
              <dt>API title</dt>
              <dd>{validation.title ?? "Not provided"}</dd>
            </div>
            <div>
              <dt>API version</dt>
              <dd>{validation.apiVersion ?? "Not provided"}</dd>
            </div>
            <div>
              <dt>Paths</dt>
              <dd>{validation.pathCount}</dd>
            </div>
          </dl>
          {validation.errors.length > 0 ? (
            <ul>
              {validation.errors.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          ) : null}
          <details>
            <summary>View selected document text</summary>
            <pre className="http-inspector-body">{selected.content}</pre>
          </details>
        </section>
      ) : null}
    </section>
  );
}
