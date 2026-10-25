import { describe, expect, it } from "vitest";
import { APP_ERROR_CODES, presentAppError } from "./app-errors";

describe("stable application error presentation", () => {
  it("provides a user-facing mapping for every stable code", () => {
    for (const code of APP_ERROR_CODES) {
      const presentation = presentAppError({ code });
      expect(presentation.code).toBe(code);
      expect(presentation.message.length).toBeGreaterThan(0);
      expect(typeof presentation.retryable).toBe("boolean");
    }
  });

  it("falls back safely for unknown or malformed errors", () => {
    expect(presentAppError(null).code).toBe("INTERNAL");
    expect(presentAppError({ code: "FUTURE_ERROR" }).code).toBe("INTERNAL");
    expect(presentAppError("permission denied").code).toBe("INTERNAL");
  });

  it("does not expose backend English messages or secret details", () => {
    const presentation = presentAppError({
      code: "PERMISSION_DENIED",
      message: "C:/private/path/token=do-not-show",
    });
    expect(presentation.message).toBe(
      "Moonships does not have permission to complete this action.",
    );
    expect(presentation.message).not.toContain("private");
    expect(presentation.message).not.toContain("token");
  });
});
