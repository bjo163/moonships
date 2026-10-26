export const MAX_JSON_INPUT_BYTES = 1024 * 1024;
export const MAX_JSON_NESTING_DEPTH = 128;
export const MAX_JSON_FORMATTED_OUTPUT_CHARS = 4 * 1024 * 1024;

export type JsonFormatErrorCode =
  | "empty_input"
  | "input_too_large"
  | "nesting_too_deep"
  | "formatted_output_too_large"
  | "invalid_json";

export type JsonFormatResult =
  | { ok: true; formatted: string; inputBytes: number }
  | { ok: false; code: JsonFormatErrorCode; message: string; inputBytes: number };

function nestingDepthExceedsLimit(source: string): boolean {
  let depth = 0;
  let inString = false;
  let escaped = false;
  for (const character of source) {
    if (inString) {
      if (escaped) escaped = false;
      else if (character === "\\") escaped = true;
      else if (character === '"') inString = false;
      continue;
    }
    if (character === '"') inString = true;
    else if (character === "{" || character === "[") {
      depth += 1;
      if (depth > MAX_JSON_NESTING_DEPTH) return true;
    } else if (character === "}" || character === "]") depth -= 1;
  }
  return false;
}

function measureUtf8BytesUpToLimit(source: string): number {
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
    if (bytes > MAX_JSON_INPUT_BYTES) return bytes;
  }
  return bytes;
}

function prettyPrintPreservingTokens(source: string): string | null {
  const input = source.trim();
  const output: string[] = [];
  let outputLength = 0;
  let depth = 0;
  let inString = false;
  let escaped = false;
  const append = (text: string): boolean => {
    outputLength += text.length;
    if (outputLength > MAX_JSON_FORMATTED_OUTPUT_CHARS) return false;
    output.push(text);
    return true;
  };

  for (let index = 0; index < input.length; index += 1) {
    const character = input[index];
    if (inString) {
      if (!append(character)) return null;
      if (escaped) escaped = false;
      else if (character === "\\") escaped = true;
      else if (character === '"') inString = false;
      continue;
    }

    if (character === '"') {
      inString = true;
      if (!append(character)) return null;
    } else if (character === "{" || character === "[") {
      let next = index + 1;
      while (/\s/.test(input[next] ?? "")) next += 1;
      const matchingClose = character === "{" ? "}" : "]";
      if (input[next] === matchingClose) {
        if (!append(`${character}${matchingClose}`)) return null;
        index = next;
      } else {
        depth += 1;
        if (!append(`${character}\n${"  ".repeat(depth)}`)) return null;
      }
    } else if (character === "}" || character === "]") {
      depth -= 1;
      if (!append(`\n${"  ".repeat(depth)}${character}`)) return null;
    } else if (character === ",") {
      if (!append(`,\n${"  ".repeat(depth)}`)) return null;
    } else if (character === ":") {
      if (!append(": ")) return null;
    } else if (!/\s/.test(character)) {
      if (!append(character)) return null;
    }
  }

  return output.join("");
}

export function formatJsonInput(source: string): JsonFormatResult {
  const inputBytes = measureUtf8BytesUpToLimit(source);
  if (inputBytes > MAX_JSON_INPUT_BYTES) {
    return {
      ok: false,
      code: "input_too_large",
      message: `Input exceeds the ${MAX_JSON_INPUT_BYTES / 1024 / 1024} MiB limit.`,
      inputBytes,
    };
  }
  if (!source.trim()) {
    return { ok: false, code: "empty_input", message: "Enter JSON to validate.", inputBytes };
  }
  if (nestingDepthExceedsLimit(source)) {
    return {
      ok: false,
      code: "nesting_too_deep",
      message: `JSON nesting is limited to ${MAX_JSON_NESTING_DEPTH} levels.`,
      inputBytes,
    };
  }

  try {
    JSON.parse(source);
  } catch {
    return { ok: false, code: "invalid_json", message: "Input is not valid JSON.", inputBytes };
  }

  const formatted = prettyPrintPreservingTokens(source);
  if (formatted === null) {
    return {
      ok: false,
      code: "formatted_output_too_large",
      message: "Formatted output exceeds the 4 million character display limit.",
      inputBytes,
    };
  }
  return { ok: true, formatted, inputBytes };
}
