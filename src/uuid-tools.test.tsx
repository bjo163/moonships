// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { UuidDevToolsPanel } from "./UuidDevToolsPanel";
import { generateUuidV4, validateUuid } from "./uuid-tools";

afterEach(cleanup);

describe("M3-019 local UUID generator and validator", () => {
  it("generates a canonical UUID v4 using the secure random API", () => {
    const randomUUID = vi.fn(() => "36b8f84d-df4e-4d49-b662-bcde71a8764f");
    expect(generateUuidV4({ randomUUID })).toEqual({
      ok: true,
      value: "36b8f84d-df4e-4d49-b662-bcde71a8764f",
    });
    expect(randomUUID).toHaveBeenCalledOnce();
  });

  it("fails closed when secure randomness is missing, throws, or returns a malformed value", () => {
    expect(generateUuidV4(undefined)).toMatchObject({
      ok: false,
      code: "secure_random_unavailable",
    });
    expect(
      generateUuidV4({
        randomUUID: () => {
          throw new Error("unavailable");
        },
      }),
    ).toMatchObject({
      ok: false,
      code: "secure_random_unavailable",
    });
    expect(generateUuidV4({ randomUUID: () => "not-a-uuid" })).toMatchObject({
      ok: false,
      code: "secure_random_unavailable",
    });
  });

  it("accepts declared RFC UUID versions, RFC variants and Nil UUID", () => {
    for (const uuid of [
      "36b8f84d-df4e-4d49-b662-bcde71a8764f",
      "01890f3e-7c00-7000-8000-000000000000",
      "ffffffff-ffff-8fff-bfff-ffffffffffff",
      "00000000-0000-0000-0000-000000000000",
    ]) {
      expect(validateUuid(uuid)).toBe(true);
    }
  });

  it("rejects malformed UUIDs, unsupported variants and noncanonical separators", () => {
    for (const uuid of [
      "",
      "36b8f84d-df4e-4d49-7662-bcde71a8764f",
      "36b8f84d-df4e-0d49-b662-bcde71a8764f",
      "36b8f84ddf4e4d49b662bcde71a8764f",
      " urn:uuid:36b8f84d-df4e-4d49-b662-bcde71a8764f",
    ]) {
      expect(validateUuid(uuid)).toBe(false);
    }
  });

  it("shows generation failure when secure random UUID is unavailable", () => {
    const originalCrypto = globalThis.crypto;
    Object.defineProperty(globalThis, "crypto", { configurable: true, value: undefined });
    try {
      render(<UuidDevToolsPanel />);
      fireEvent.click(screen.getByRole("button", { name: "Generate UUID" }));
      expect(screen.getByRole("alert").textContent).toContain(
        "Secure UUID generation is unavailable",
      );
    } finally {
      Object.defineProperty(globalThis, "crypto", { configurable: true, value: originalCrypto });
    }
  });
});
