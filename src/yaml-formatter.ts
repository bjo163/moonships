import { LineCounter, parseDocument } from "yaml";

export const MAX_YAML_INPUT_BYTES = 1024 * 1024;
export const MAX_YAML_OUTPUT_CHARS = 4 * 1024 * 1024;

export type YamlFormatIssue = { code: string; line?: number; column?: number };
export type YamlFormatResult =
  | { ok: true; formatted: string; inputBytes: number; warnings: YamlFormatIssue[] }
  | {
      ok: false;
      code:
        | "empty_input"
        | "input_too_large"
        | "invalid_yaml"
        | "output_too_large"
        | "parser_error";
      message: string;
      inputBytes: number;
      issues: YamlFormatIssue[];
    };

function measureUtf8Bytes(source: string): number {
  let bytes = 0;
  for (let index = 0; index < source.length; index += 1) {
    const character = source.charCodeAt(index);
    if (character <= 0x7f) bytes += 1;
    else if (character <= 0x7ff) bytes += 2;
    else if (
      character >= 0xd800 &&
      character <= 0xdbff &&
      source.charCodeAt(index + 1) >= 0xdc00 &&
      source.charCodeAt(index + 1) <= 0xdfff
    ) {
      bytes += 4;
      index += 1;
    } else bytes += 3;
    if (bytes > MAX_YAML_INPUT_BYTES) return bytes;
  }
  return bytes;
}

function issuesFrom(
  errors: readonly { code: string; pos: readonly number[] }[],
  lineCounter: LineCounter,
) {
  const seen = new Set<string>();
  return errors
    .map((error) => {
      const location =
        typeof error.pos[0] === "number" ? lineCounter.linePos(error.pos[0]) : undefined;
      return {
        code: error.code,
        ...(location ? { line: location.line, column: location.col } : {}),
      };
    })
    .filter((issue) => {
      const identity = `${issue.code}:${issue.line ?? ""}:${issue.column ?? ""}`;
      if (seen.has(identity)) return false;
      seen.add(identity);
      return true;
    });
}

export function formatYamlInput(source: string): YamlFormatResult {
  const inputBytes = measureUtf8Bytes(source);
  if (inputBytes > MAX_YAML_INPUT_BYTES) {
    return {
      ok: false,
      code: "input_too_large",
      message: "Input exceeds the 1 MiB limit.",
      inputBytes,
      issues: [],
    };
  }
  if (!source.trim()) {
    return {
      ok: false,
      code: "empty_input",
      message: "Enter YAML to validate.",
      inputBytes,
      issues: [],
    };
  }

  try {
    const lineCounter = new LineCounter();
    const document = parseDocument(source, {
      version: "1.2",
      schema: "core",
      strict: true,
      uniqueKeys: true,
      prettyErrors: false,
      logLevel: "error",
      lineCounter,
    });
    const issues = issuesFrom(document.errors, lineCounter);
    if (issues.length > 0) {
      return {
        ok: false,
        code: "invalid_yaml",
        message: "Input is not valid YAML. Review the reported parser codes and positions.",
        inputBytes,
        issues,
      };
    }
    const warnings = issuesFrom(document.warnings, lineCounter);
    const formatted = document.toString({ indent: 2, lineWidth: 0 });
    if (formatted.length > MAX_YAML_OUTPUT_CHARS) {
      return {
        ok: false,
        code: "output_too_large",
        message: "Formatted output exceeds the 4 million character display limit.",
        inputBytes,
        issues: [],
      };
    }
    return { ok: true, formatted, inputBytes, warnings };
  } catch {
    return {
      ok: false,
      code: "parser_error",
      message: "YAML could not be processed within the available local resources.",
      inputBytes,
      issues: [],
    };
  }
}
