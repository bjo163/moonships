export const MAX_URL_INPUT_BYTES = 1024 * 1024;
export const MAX_URL_OUTPUT_CHARS = 4 * 1024 * 1024;

export type UrlOperation =
  | "encode_component"
  | "decode_component"
  | "encode_full_url"
  | "decode_full_url";
export type UrlTransformResult =
  | { ok: true; value: string; inputBytes: number }
  | {
      ok: false;
      code: "input_too_large" | "output_too_large" | "invalid_encoding" | "invalid_url";
      message: string;
      inputBytes: number;
    };

function byteLength(source: string): number {
  if (source.length > MAX_URL_INPUT_BYTES) return MAX_URL_INPUT_BYTES + 1;
  return new TextEncoder().encode(source).byteLength;
}

export function transformUrlInput(source: string, operation: UrlOperation): UrlTransformResult {
  const inputBytes = byteLength(source);
  if (inputBytes > MAX_URL_INPUT_BYTES) {
    return {
      ok: false,
      code: "input_too_large",
      message: "Input exceeds the 1 MiB limit.",
      inputBytes,
    };
  }

  try {
    let value: string;
    switch (operation) {
      case "encode_component":
        value = encodeURIComponent(source);
        break;
      case "decode_component":
        value = decodeURIComponent(source);
        break;
      case "encode_full_url":
        value = new URL(source).href;
        break;
      case "decode_full_url":
        value = new URL(decodeURI(source)).href;
        break;
    }
    if (value.length > MAX_URL_OUTPUT_CHARS) {
      return {
        ok: false,
        code: "output_too_large",
        message: "Output exceeds the 4 million character display limit.",
        inputBytes,
      };
    }
    return { ok: true, value, inputBytes };
  } catch (error) {
    const fullUrlOperation = operation.endsWith("full_url");
    const invalidEncoding = error instanceof URIError;
    return {
      ok: false,
      code: invalidEncoding || !fullUrlOperation ? "invalid_encoding" : "invalid_url",
      message:
        invalidEncoding || !fullUrlOperation
          ? "Input contains an invalid percent escape or Unicode sequence."
          : "Enter a valid absolute URL.",
      inputBytes,
    };
  }
}
