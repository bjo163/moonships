export const MAX_BASE64_BYTES = 1024 * 1024;
export const MAX_BASE64_INPUT_CHARS = Math.ceil((MAX_BASE64_BYTES * 4) / 3) + 4;
const BINARY_HEX_PREVIEW_BYTES = 64;

export type Base64ErrorCode = "input_too_large" | "invalid_base64" | "invalid_hex";
export type Base64EncodeResult =
  | { ok: true; base64: string; byteLength: number }
  | { ok: false; code: Base64ErrorCode; message: string };
export type Base64DecodeResult =
  | { ok: true; kind: "text"; text: string; byteLength: number }
  | {
      ok: true;
      kind: "binary";
      byteLength: number;
      hexPreview: string;
      truncated: boolean;
    }
  | { ok: false; code: Base64ErrorCode; message: string };

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunkSize = 0x8000;
  for (let offset = 0; offset < bytes.length; offset += chunkSize) {
    const chunk = bytes.subarray(offset, Math.min(offset + chunkSize, bytes.length));
    binary += String.fromCharCode(...chunk);
  }
  return btoa(binary);
}

function inputTooLarge(): { ok: false; code: "input_too_large"; message: string } {
  return {
    ok: false,
    code: "input_too_large",
    message: "Decoded or encoded data is limited to 1 MiB.",
  };
}

export function encodeUtf8Text(source: string): Base64EncodeResult {
  if (source.length > MAX_BASE64_BYTES) return inputTooLarge();
  const bytes = new TextEncoder().encode(source);
  if (bytes.byteLength > MAX_BASE64_BYTES) return inputTooLarge();
  return { ok: true, base64: bytesToBase64(bytes), byteLength: bytes.byteLength };
}

export function encodeHexBytes(source: string): Base64EncodeResult {
  if (source.length > MAX_BASE64_BYTES * 3) return inputTooLarge();
  const normalized = source.replace(/\s/g, "");
  if (!/^(?:[0-9a-fA-F]{2})*$/.test(normalized)) {
    return {
      ok: false,
      code: "invalid_hex",
      message: "Enter hexadecimal byte pairs, for example: 00 7F FF.",
    };
  }
  if (normalized.length / 2 > MAX_BASE64_BYTES) return inputTooLarge();
  const bytes = new Uint8Array(normalized.length / 2);
  for (let index = 0; index < bytes.length; index += 1) {
    bytes[index] = Number.parseInt(normalized.slice(index * 2, index * 2 + 2), 16);
  }
  return { ok: true, base64: bytesToBase64(bytes), byteLength: bytes.byteLength };
}

export function decodeBase64(source: string): Base64DecodeResult {
  if (source.length > MAX_BASE64_INPUT_CHARS * 2) return inputTooLarge();
  const normalized = source.replace(/\s/g, "");
  if (
    normalized.length > MAX_BASE64_INPUT_CHARS ||
    !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(normalized)
  ) {
    return {
      ok: false,
      code: normalized.length > MAX_BASE64_INPUT_CHARS ? "input_too_large" : "invalid_base64",
      message:
        normalized.length > MAX_BASE64_INPUT_CHARS
          ? "Base64 input exceeds the 1 MiB decoded-data limit."
          : "Input is not valid padded standard Base64.",
    };
  }

  const expectedBytes =
    Math.floor((normalized.length * 3) / 4) -
    (normalized.endsWith("==") ? 2 : normalized.endsWith("=") ? 1 : 0);
  if (expectedBytes > MAX_BASE64_BYTES) return inputTooLarge();

  let binary: string;
  try {
    binary = atob(normalized);
  } catch {
    return {
      ok: false,
      code: "invalid_base64",
      message: "Input is not valid padded standard Base64.",
    };
  }
  const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
  if (bytesToBase64(bytes) !== normalized) {
    return {
      ok: false,
      code: "invalid_base64",
      message: "Input is not canonical padded standard Base64.",
    };
  }

  try {
    const text = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(bytes);
    return { ok: true, kind: "text", text, byteLength: bytes.byteLength };
  } catch {
    const preview = bytes.subarray(0, BINARY_HEX_PREVIEW_BYTES);
    return {
      ok: true,
      kind: "binary",
      byteLength: bytes.byteLength,
      hexPreview: Array.from(preview, (byte) => byte.toString(16).padStart(2, "0")).join(" "),
      truncated: bytes.byteLength > BINARY_HEX_PREVIEW_BYTES,
    };
  }
}
