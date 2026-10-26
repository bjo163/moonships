export type UnixTimestampUnit = "seconds" | "milliseconds";

export type TimestampErrorCode =
  | "invalid_timestamp"
  | "timestamp_out_of_range"
  | "invalid_iso8601"
  | "unknown_timezone_offset";

export type TimestampResult =
  | {
      ok: true;
      epochMilliseconds: number;
      unixSeconds: string;
      unixMilliseconds: string;
      isoUtc: string;
      localTime: string;
      localTimeZone: string;
    }
  | { ok: false; code: TimestampErrorCode; message: string };

function resultForEpoch(epochMilliseconds: number): TimestampResult {
  const date = new Date(epochMilliseconds);
  if (!Number.isFinite(epochMilliseconds) || Number.isNaN(date.getTime())) {
    return {
      ok: false,
      code: "timestamp_out_of_range",
      message: "Timestamp is outside the supported date range.",
    };
  }

  const formatter = new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "long" });
  return {
    ok: true,
    epochMilliseconds,
    unixSeconds: String(epochMilliseconds / 1000),
    unixMilliseconds: String(epochMilliseconds),
    isoUtc: date.toISOString(),
    localTime: formatter.format(date),
    localTimeZone: formatter.resolvedOptions().timeZone,
  };
}

export function convertUnixTimestamp(input: string, unit: UnixTimestampUnit): TimestampResult {
  const trimmed = input.trim();
  if (!/^-?\d+$/.test(trimmed)) {
    return {
      ok: false,
      code: "invalid_timestamp",
      message: "Enter a whole-number Unix timestamp without separators.",
    };
  }
  const value = Number(trimmed);
  if (!Number.isSafeInteger(value)) {
    return {
      ok: false,
      code: "timestamp_out_of_range",
      message: "Timestamp is outside the supported date range.",
    };
  }
  const epochMilliseconds = unit === "seconds" ? value * 1000 : value;
  return resultForEpoch(epochMilliseconds);
}

export function convertIsoTimestamp(input: string): TimestampResult {
  const match = input
    .trim()
    .match(/^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,3}))?(Z|[+-]\d{2}:\d{2})$/i);
  if (!match) {
    return {
      ok: false,
      code: "invalid_iso8601",
      message: "Enter an ISO 8601 date and time with a Z or numeric timezone offset.",
    };
  }

  const [, yearText, monthText, dayText, hourText, minuteText, secondText, fraction, zone] = match;
  if (zone === "-00:00") {
    return {
      ok: false,
      code: "unknown_timezone_offset",
      message: "The -00:00 offset means the timezone is unknown; provide a known offset.",
    };
  }
  const year = Number(yearText);
  const month = Number(monthText);
  const day = Number(dayText);
  const hour = Number(hourText);
  const minute = Number(minuteText);
  const second = Number(secondText);
  const millisecond = Number((fraction ?? "").padEnd(3, "0"));
  if (month < 1 || month > 12 || hour > 23 || minute > 59 || second > 59) {
    return invalidIsoTimestamp();
  }

  const civilDate = new Date(0);
  civilDate.setUTCFullYear(year, month - 1, day);
  civilDate.setUTCHours(hour, minute, second, millisecond);
  if (
    day < 1 ||
    civilDate.getUTCFullYear() !== year ||
    civilDate.getUTCMonth() !== month - 1 ||
    civilDate.getUTCDate() !== day
  ) {
    return invalidIsoTimestamp();
  }

  let offsetMinutes = 0;
  if (zone.toUpperCase() !== "Z") {
    const offset = zone.match(/^([+-])(\d{2}):(\d{2})$/);
    if (!offset) return invalidIsoTimestamp();
    const offsetHours = Number(offset[2]);
    const offsetRemainder = Number(offset[3]);
    if (offsetHours > 23 || offsetRemainder > 59) return invalidIsoTimestamp();
    offsetMinutes = (offsetHours * 60 + offsetRemainder) * (offset[1] === "+" ? 1 : -1);
  }

  return resultForEpoch(civilDate.getTime() - offsetMinutes * 60_000);
}

function invalidIsoTimestamp(): TimestampResult {
  return {
    ok: false,
    code: "invalid_iso8601",
    message: "Enter a real calendar date and time using a valid timezone offset.",
  };
}
