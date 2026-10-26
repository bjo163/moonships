import { describe, expect, it } from "vitest";
import { decodeBase64, encodeHexBytes, encodeUtf8Text, MAX_BASE64_BYTES } from "./base64-tools";

describe("M3-017 Base64 text and binary conversion", () => {
  it("round-trips Unicode text through UTF-8 Base64", () => {
    const encoded = encodeUtf8Text("Moonships 🌕 café");
    expect(encoded.ok).toBe(true);
    if (!encoded.ok) return;
    expect(decodeBase64(encoded.base64)).toMatchObject({
      ok: true,
      kind: "text",
      text: "Moonships 🌕 café",
    });
  });

  it("preserves a UTF-8 byte-order mark as text data", () => {
    const encoded = encodeUtf8Text("\uFEFFpayload");
    expect(encoded.ok).toBe(true);
    if (!encoded.ok) return;
    expect(decodeBase64(encoded.base64)).toMatchObject({
      ok: true,
      kind: "text",
      text: "\uFEFFpayload",
    });
  });

  it("encodes explicit hex bytes without treating them as text", () => {
    expect(encodeHexBytes("00 7f 80 ff")).toEqual({
      ok: true,
      base64: "AH+A/w==",
      byteLength: 4,
    });
    expect(encodeHexBytes("0")).toMatchObject({ ok: false, code: "invalid_hex" });
  });

  it("decodes non-UTF-8 bytes as binary with a bounded hex preview", () => {
    expect(decodeBase64("AH+A/w==")).toEqual({
      ok: true,
      kind: "binary",
      byteLength: 4,
      hexPreview: "00 7f 80 ff",
      truncated: false,
    });
  });

  it("limits binary hex previews without losing the decoded byte count", () => {
    const raw = String.fromCharCode(...new Uint8Array(65).fill(0xff));
    expect(decodeBase64(btoa(raw))).toMatchObject({
      ok: true,
      kind: "binary",
      byteLength: 65,
      truncated: true,
    });
  });

  it("rejects malformed and non-canonical Base64 as structured errors", () => {
    expect(decodeBase64("YQ===")).toMatchObject({ ok: false, code: "invalid_base64" });
    expect(decodeBase64("Zh==")).toMatchObject({ ok: false, code: "invalid_base64" });
  });

  it("enforces the byte limit for encoding and decoded output", () => {
    const oversizedText = "a".repeat(MAX_BASE64_BYTES + 1);
    expect(encodeUtf8Text(oversizedText)).toMatchObject({ ok: false, code: "input_too_large" });
    const oversizedBytesBase64 = btoa("a".repeat(MAX_BASE64_BYTES + 1));
    expect(decodeBase64(oversizedBytesBase64)).toMatchObject({
      ok: false,
      code: "input_too_large",
    });
  });

  it("handles empty data as a valid zero-byte round trip", () => {
    expect(encodeUtf8Text("")).toEqual({ ok: true, base64: "", byteLength: 0 });
    expect(decodeBase64("")).toEqual({ ok: true, kind: "text", text: "", byteLength: 0 });
  });
});
