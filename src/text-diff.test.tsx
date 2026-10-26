import { describe, expect, it } from "vitest";
import { diffTextInputs } from "./text-diff";

describe("M3-021 bounded local text diff", () => {
  it("reports inserts and deletions in the unified result", () => {
    const result = diffTextInputs("keep\nremove\n", "keep\nadd\n");
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.changed).toBe(true);
    expect(result.unified).toContain("-remove\n");
    expect(result.unified).toContain("+add\n");
  });

  it("handles empty inputs and insertion into an empty input", () => {
    expect(diffTextInputs("", "")).toMatchObject({ ok: true, changed: false, unified: "" });
    expect(diffTextInputs("", "first\nsecond\n")).toMatchObject({
      ok: true,
      changed: true,
      unified: "+first\n+second\n",
    });
    expect(diffTextInputs("first\nsecond\n", "")).toMatchObject({
      ok: true,
      changed: true,
      unified: "-first\n-second\n",
    });
  });

  it("reports equal content and preserves a final line without a newline", () => {
    expect(diffTextInputs("same", "same")).toMatchObject({
      ok: true,
      changed: false,
      unified: " same\n",
    });
  });

  it("rejects over-limit inputs and excessive line counts before diffing", () => {
    expect(diffTextInputs("x".repeat(1024 * 1024 + 1), "")).toMatchObject({
      ok: false,
      code: "input_too_large",
    });
    expect(diffTextInputs("\n".repeat(50_000), "")).toMatchObject({
      ok: false,
      code: "too_many_lines",
    });
  });

  it("stops when the edit distance exceeds the configured computation bound", () => {
    const before = Array.from({ length: 550 }, (_, index) => `old-${index}\n`).join("");
    const after = Array.from({ length: 550 }, (_, index) => `new-${index}\n`).join("");
    expect(diffTextInputs(before, after)).toMatchObject({
      ok: false,
      code: "edit_distance_too_large",
    });
  });
});
