import { describe, expect, it } from "vitest";
import { parseDocument } from "yaml";
import { formatYamlInput, MAX_YAML_INPUT_BYTES } from "./yaml-formatter";

describe("M3-016 vetted offline YAML formatter and validator", () => {
  it("formats valid YAML using the selected YAML 1.2 parser", () => {
    const result = formatYamlInput("name: Moonships\nitems: [one,two]\n");
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.formatted).toContain("name: Moonships");
    expect(parseDocument(result.formatted, { version: "1.2" }).errors).toHaveLength(0);
  });

  it("returns structured error codes and positions for malformed YAML", () => {
    const result = formatYamlInput("items: [unterminated\n");
    expect(result).toMatchObject({ ok: false, code: "invalid_yaml" });
    if (result.ok) return;
    expect(result.issues.length).toBeGreaterThan(0);
    expect(result.issues[0].code).toBeTruthy();
    expect(result.issues[0].line).toBeGreaterThan(0);
  });

  it("rejects duplicate mapping keys", () => {
    expect(formatYamlInput("value: first\nvalue: second\n")).toMatchObject({
      ok: false,
      code: "invalid_yaml",
    });
  });

  it("distinguishes empty content and oversized input", () => {
    expect(formatYamlInput(" \n")).toMatchObject({ ok: false, code: "empty_input" });
    const oversized = `value: ${"x".repeat(MAX_YAML_INPUT_BYTES)}`;
    expect(formatYamlInput(oversized)).toMatchObject({ ok: false, code: "input_too_large" });
  });

  it("keeps YAML aliases as syntax and does not expand them into objects", () => {
    const result = formatYamlInput("base: &base {x: 1}\ncopy: *base\n");
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.formatted).toContain("&base");
    expect(result.formatted).toContain("*base");
  });
});
