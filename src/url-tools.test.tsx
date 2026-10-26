import { describe, expect, it } from "vitest";
import { MAX_URL_INPUT_BYTES, transformUrlInput } from "./url-tools";

describe("M3-018 URL component and full URL conversion", () => {
  it("encodes reserved characters when text is a single component", () => {
    const source = "a/b?c=d &雪";
    const encoded = transformUrlInput(source, "encode_component");
    expect(encoded).toMatchObject({
      ok: true,
      value: "a%2Fb%3Fc%3Dd%20%26%E9%9B%AA",
    });
    if (!encoded.ok) return;
    expect(transformUrlInput(encoded.value, "decode_component")).toMatchObject({
      ok: true,
      value: source,
    });
  });

  it("preserves full URL delimiters and only encodes needed characters", () => {
    expect(
      transformUrlInput("https://example.com/a b?q=one two&next=/path#section", "encode_full_url"),
    ).toMatchObject({
      ok: true,
      value: "https://example.com/a%20b?q=one%20two&next=/path#section",
    });
  });

  it("does not decode percent-encoded reserved delimiters in full URL mode", () => {
    expect(
      transformUrlInput("https://example.com/a%2Fb?q=one%26two", "decode_full_url"),
    ).toMatchObject({ ok: true, value: "https://example.com/a%2Fb?q=one%26two" });
  });

  it("returns stable errors for malformed component escapes and invalid full URLs", () => {
    expect(transformUrlInput("bad%2", "decode_component")).toMatchObject({
      ok: false,
      code: "invalid_encoding",
    });
    expect(transformUrlInput("/relative/path", "encode_full_url")).toMatchObject({
      ok: false,
      code: "invalid_url",
    });
  });

  it("bounds UTF-8 input size", () => {
    expect(
      transformUrlInput("x".repeat(MAX_URL_INPUT_BYTES + 1), "encode_component"),
    ).toMatchObject({
      ok: false,
      code: "input_too_large",
    });
  });
});
