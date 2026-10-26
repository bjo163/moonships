import { describe, expect, it } from "vitest";
import { MAX_OPENAPI_DOCUMENT_BYTES, validateOpenApiDocument } from "./openapi-validation";

describe("OpenAPI local document validation", () => {
  it("validates a basic OpenAPI 3.1 YAML document", () => {
    const result = validateOpenApiDocument(
      `openapi: 3.1.0\ninfo:\n  title: Example API\n  version: 1.0.0\npaths:\n  /health:\n    get:\n      responses:\n        '200':\n          description: OK\n`,
    );
    expect(result).toMatchObject({
      valid: true,
      version: "3.1.0",
      title: "Example API",
      pathCount: 1,
      errors: [],
    });
  });

  it("rejects malformed YAML, duplicate keys, aliases and invalid OpenAPI shape", () => {
    expect(validateOpenApiDocument("openapi: [broken").valid).toBe(false);
    expect(
      validateOpenApiDocument("openapi: 3.1.0\nopenapi: 3.1.0\n").errors.length,
    ).toBeGreaterThan(0);
    expect(
      validateOpenApiDocument("openapi: 3.1.0\ninfo: &i {title: API, version: 1}\npaths: *i\n")
        .errors,
    ).toContain("Aliases or complex YAML values are not supported.");
    expect(validateOpenApiDocument("notOpenApi: true").valid).toBe(false);
  });

  it("rejects oversized content before parsing", () => {
    const content = `openapi: 3.1.0\n${"x".repeat(MAX_OPENAPI_DOCUMENT_BYTES)}`;
    expect(validateOpenApiDocument(content).errors).toEqual(["Document exceeds the 1 MiB limit."]);
  });
});
