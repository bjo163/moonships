import { diffLines } from "diff";

export const MAX_DIFF_INPUT_BYTES = 1024 * 1024;
export const MAX_DIFF_LINES_PER_INPUT = 50_000;
export const MAX_DIFF_OUTPUT_CHARS = 4 * 1024 * 1024;
export const MAX_DIFF_EDIT_DISTANCE = 1_000;

export type TextDiffErrorCode =
  | "input_too_large"
  | "too_many_lines"
  | "edit_distance_too_large"
  | "output_too_large";

export type TextDiffResult =
  | { ok: true; changed: boolean; unified: string; beforeBytes: number; afterBytes: number }
  | {
      ok: false;
      code: TextDiffErrorCode;
      message: string;
      beforeBytes: number;
      afterBytes: number;
    };

function utf8ByteLengthUpToLimit(text: string): number {
  if (text.length > MAX_DIFF_INPUT_BYTES) return MAX_DIFF_INPUT_BYTES + 1;
  return new TextEncoder().encode(text).byteLength;
}

function countLinesUpToLimit(text: string): number {
  let lines = 1;
  for (let index = 0; index < text.length; index += 1) {
    if (text.charCodeAt(index) === 10) {
      lines += 1;
      if (lines > MAX_DIFF_LINES_PER_INPUT) return lines;
    }
  }
  return lines;
}

function error(
  code: TextDiffErrorCode,
  message: string,
  beforeBytes: number,
  afterBytes: number,
): TextDiffResult {
  return { ok: false, code, message, beforeBytes, afterBytes };
}

export function diffTextInputs(before: string, after: string): TextDiffResult {
  const beforeBytes = utf8ByteLengthUpToLimit(before);
  const afterBytes = utf8ByteLengthUpToLimit(after);
  if (beforeBytes > MAX_DIFF_INPUT_BYTES || afterBytes > MAX_DIFF_INPUT_BYTES) {
    return error(
      "input_too_large",
      "Each text input is limited to 1 MiB UTF-8.",
      beforeBytes,
      afterBytes,
    );
  }
  if (
    countLinesUpToLimit(before) > MAX_DIFF_LINES_PER_INPUT ||
    countLinesUpToLimit(after) > MAX_DIFF_LINES_PER_INPUT
  ) {
    return error(
      "too_many_lines",
      `Each input is limited to ${MAX_DIFF_LINES_PER_INPUT.toLocaleString()} lines.`,
      beforeBytes,
      afterBytes,
    );
  }

  const changes = diffLines(before, after, { maxEditLength: MAX_DIFF_EDIT_DISTANCE });
  if (!changes) {
    return error(
      "edit_distance_too_large",
      "The inputs differ too much to calculate within the edit-distance limit.",
      beforeBytes,
      afterBytes,
    );
  }

  let outputLength = 0;
  const output: string[] = [];
  let changed = false;
  for (const change of changes) {
    const prefix = change.added ? "+" : change.removed ? "-" : " ";
    if (change.added || change.removed) changed = true;
    let start = 0;
    while (start < change.value.length) {
      const newline = change.value.indexOf("\n", start);
      const end = newline === -1 ? change.value.length : newline + 1;
      const line = change.value.slice(start, end);
      const formatted = `${prefix}${line}${newline === -1 ? "\n" : ""}`;
      outputLength += formatted.length;
      if (outputLength > MAX_DIFF_OUTPUT_CHARS) {
        return error(
          "output_too_large",
          "Unified diff exceeds the 4 million character display limit.",
          beforeBytes,
          afterBytes,
        );
      }
      output.push(formatted);
      start = end;
    }
  }
  return { ok: true, changed, unified: output.join(""), beforeBytes, afterBytes };
}
