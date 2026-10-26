export type UuidGenerationResult =
  | { ok: true; value: string }
  | { ok: false; code: "secure_random_unavailable"; message: string };

const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
const NIL_UUID = "00000000-0000-0000-0000-000000000000";
type SecureUuidSource = { randomUUID: () => string };

export function generateUuidV4(cryptoSource: SecureUuidSource | undefined): UuidGenerationResult {
  if (!cryptoSource || typeof cryptoSource.randomUUID !== "function") {
    return {
      ok: false,
      code: "secure_random_unavailable",
      message: "Secure UUID generation is unavailable in this environment.",
    };
  }

  try {
    const value = cryptoSource.randomUUID();
    if (!/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value)) {
      return {
        ok: false,
        code: "secure_random_unavailable",
        message: "Secure UUID generation returned an invalid result.",
      };
    }
    return { ok: true, value };
  } catch {
    return {
      ok: false,
      code: "secure_random_unavailable",
      message: "Secure UUID generation is unavailable in this environment.",
    };
  }
}

export function validateUuid(value: string): boolean {
  return value.toLowerCase() === NIL_UUID || UUID_PATTERN.test(value);
}
