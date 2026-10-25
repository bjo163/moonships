export const APP_ERROR_CODES = [
  "VALIDATION_FAILED",
  "NOT_FOUND",
  "PERMISSION_DENIED",
  "UNAVAILABLE",
  "TIMEOUT",
  "CANCELLED",
  "CONFLICT",
  "IO_ERROR",
  "INTERNAL",
] as const;

export type AppErrorCode = (typeof APP_ERROR_CODES)[number];

export type AppErrorPresentation = {
  code: AppErrorCode;
  message: string;
  retryable: boolean;
};

const PRESENTATION: Record<AppErrorCode, Omit<AppErrorPresentation, "code">> = {
  VALIDATION_FAILED: {
    message: "Some information is missing or invalid. Review the fields and try again.",
    retryable: false,
  },
  NOT_FOUND: {
    message: "The requested item could not be found.",
    retryable: false,
  },
  PERMISSION_DENIED: {
    message: "Moonships does not have permission to complete this action.",
    retryable: false,
  },
  UNAVAILABLE: {
    message: "This capability is not available right now.",
    retryable: true,
  },
  TIMEOUT: {
    message: "The operation took too long to finish.",
    retryable: true,
  },
  CANCELLED: {
    message: "The operation was cancelled.",
    retryable: false,
  },
  CONFLICT: {
    message: "The item changed before the operation could finish. Refresh and try again.",
    retryable: true,
  },
  IO_ERROR: {
    message: "Moonships could not read or write the requested local data.",
    retryable: true,
  },
  INTERNAL: {
    message: "Something unexpected happened. Try again, or inspect the diagnostic details.",
    retryable: true,
  },
};

const VALID_CODES = new Set<string>(APP_ERROR_CODES);

/**
 * Maps stable machine-readable codes to safe UI copy.
 * It deliberately never parses or displays a backend's English error message.
 */
export function presentAppError(input: unknown): AppErrorPresentation {
  if (
    typeof input === "object" &&
    input !== null &&
    "code" in input &&
    typeof input.code === "string" &&
    VALID_CODES.has(input.code)
  ) {
    const code = input.code as AppErrorCode;
    return { code, ...PRESENTATION[code] };
  }

  return { code: "INTERNAL", ...PRESENTATION.INTERNAL };
}
