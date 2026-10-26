import { parseDocument } from "yaml";

export const MAX_OPENAPI_DOCUMENT_BYTES = 1024 * 1024;

export type OpenApiValidation = {
  valid: boolean;
  version: string | null;
  title: string | null;
  apiVersion: string | null;
  pathCount: number;
  errors: string[];
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function validateOpenApiDocument(content: string): OpenApiValidation {
  if (new TextEncoder().encode(content).byteLength > MAX_OPENAPI_DOCUMENT_BYTES) {
    return {
      valid: false,
      version: null,
      title: null,
      apiVersion: null,
      pathCount: 0,
      errors: ["Document exceeds the 1 MiB limit."],
    };
  }
  let document: ReturnType<typeof parseDocument>;
  try {
    document = parseDocument(content, { version: "1.2", uniqueKeys: true, prettyErrors: false });
  } catch {
    return {
      valid: false,
      version: null,
      title: null,
      apiVersion: null,
      pathCount: 0,
      errors: ["Document could not be parsed."],
    };
  }
  if (document.errors.length > 0) {
    return {
      valid: false,
      version: null,
      title: null,
      apiVersion: null,
      pathCount: 0,
      errors: document.errors.slice(0, 5).map((error) => error.code),
    };
  }
  let value: unknown;
  try {
    value = document.toJS({ maxAliasCount: 0 });
  } catch {
    return {
      valid: false,
      version: null,
      title: null,
      apiVersion: null,
      pathCount: 0,
      errors: ["Aliases or complex YAML values are not supported."],
    };
  }
  if (!isRecord(value)) {
    return {
      valid: false,
      version: null,
      title: null,
      apiVersion: null,
      pathCount: 0,
      errors: ["Root value must be an OpenAPI object."],
    };
  }
  const errors: string[] = [];
  const version = typeof value.openapi === "string" ? value.openapi : null;
  if (!version || !/^3\.(0|1)\.\d+$/.test(version))
    errors.push("Only OpenAPI 3.0.x and 3.1.x documents are supported.");
  const info = isRecord(value.info) ? value.info : null;
  const title = typeof info?.title === "string" ? info.title.slice(0, 300) : null;
  const apiVersion = typeof info?.version === "string" ? info.version.slice(0, 80) : null;
  if (!title) errors.push("info.title is required.");
  if (!apiVersion) errors.push("info.version is required.");
  if (!isRecord(value.paths)) errors.push("paths must be an object.");
  const pathCount = isRecord(value.paths) ? Object.keys(value.paths).length : 0;
  if (pathCount > 5_000) errors.push("Document has more than 5,000 paths.");
  return { valid: errors.length === 0, version, title, apiVersion, pathCount, errors };
}
