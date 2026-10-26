import { describe, expect, it } from "vitest";
import {
  formatJsonInput,
  MAX_JSON_FORMATTED_OUTPUT_CHARS,
  MAX_JSON_INPUT_BYTES,
  MAX_JSON_NESTING_DEPTH,
} from "./json-formatter";

describe("M3-015 local JSON formatter and validator", () => {
  it("validates and formats valid JSON without changing number or string tokens", () => {
    const result = formatJsonInput('{"large":9007199254740993,"exp":1e400,"text":"a,b: } \\"x"}');
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.formatted).toBe(
      '{\n  "large": 9007199254740993,\n  "exp": 1e400,\n  "text": "a,b: } \\"x"\n}',
    );
    expect(JSON.parse(result.formatted)).toEqual(
      JSON.parse('{"large":9007199254740993,"exp":1e400,"text":"a,b: } \\"x"}'),
    );
  });

  it("preserves compact empty objects and arrays", () => {
    const result = formatJsonInput('{"empty":{},"items":[]}');
    expect(result).toMatchObject({ ok: true, formatted: '{\n  "empty": {},\n  "items": []\n}' });
  });

  it("returns a structured error for malformed JSON", () => {
    expect(formatJsonInput('{"value":,}')).toMatchObject({
      ok: false,
      code: "invalid_json",
      message: "Input is not valid JSON.",
    });
  });

  it("rejects empty input distinctly", () => {
    expect(formatJsonInput("  \n")).toMatchObject({ ok: false, code: "empty_input" });
  });

  it("rejects input over the UTF-8 byte limit", () => {
    const oversized = `"${"a".repeat(MAX_JSON_INPUT_BYTES)}"`;
    expect(formatJsonInput(oversized)).toMatchObject({ ok: false, code: "input_too_large" });
  });

  it("bounds nesting before formatting", () => {
    const nested = `${"[".repeat(MAX_JSON_NESTING_DEPTH + 1)}0${"]".repeat(MAX_JSON_NESTING_DEPTH + 1)}`;
    expect(formatJsonInput(nested)).toMatchObject({ ok: false, code: "nesting_too_deep" });
  });

  it("bounds expanded output even when the source remains under the input cap", () => {
    const nested = `${"[".repeat(20)}${"0,".repeat(120_000)}0${"]".repeat(20)}`;
    expect(nested.length).toBeLessThan(MAX_JSON_INPUT_BYTES);
    expect(formatJsonInput(nested)).toMatchObject({
      ok: false,
      code: "formatted_output_too_large",
    });
    expect(MAX_JSON_FORMATTED_OUTPUT_CHARS).toBe(4 * 1024 * 1024);
  });
});
