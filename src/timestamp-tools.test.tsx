import { describe, expect, it } from "vitest";
import { convertIsoTimestamp, convertUnixTimestamp } from "./timestamp-tools";

describe("M3-022 timestamp conversion", () => {
  it("converts whole Unix seconds to UTC and local display values", () => {
    const result = convertUnixTimestamp("0", "seconds");
    expect(result).toMatchObject({
      ok: true,
      epochMilliseconds: 0,
      unixSeconds: "0",
      unixMilliseconds: "0",
      isoUtc: "1970-01-01T00:00:00.000Z",
    });
    if (result.ok) {
      expect(result.localTime.length).toBeGreaterThan(0);
      expect(result.localTimeZone.length).toBeGreaterThan(0);
    }
  });

  it("handles negative Unix milliseconds and validates timestamp syntax/range", () => {
    expect(convertUnixTimestamp("-1", "milliseconds")).toMatchObject({
      ok: true,
      epochMilliseconds: -1,
      isoUtc: "1969-12-31T23:59:59.999Z",
      unixSeconds: "-0.001",
    });
    expect(convertUnixTimestamp("1.25", "seconds")).toMatchObject({
      ok: false,
      code: "invalid_timestamp",
    });
    expect(convertUnixTimestamp("not-a-number", "seconds")).toMatchObject({
      ok: false,
      code: "invalid_timestamp",
    });
    expect(convertUnixTimestamp("9007199254740992", "milliseconds")).toMatchObject({
      ok: false,
      code: "timestamp_out_of_range",
    });
    expect(convertUnixTimestamp("9000000000000", "seconds")).toMatchObject({
      ok: false,
      code: "timestamp_out_of_range",
    });
  });

  it("uses the numeric ISO timezone offset and retains millisecond precision", () => {
    expect(convertIsoTimestamp("1970-01-01T07:00:00+07:00")).toMatchObject({
      ok: true,
      epochMilliseconds: 0,
      isoUtc: "1970-01-01T00:00:00.000Z",
    });
    expect(convertIsoTimestamp("1970-01-01T00:00:00.123Z")).toMatchObject({
      ok: true,
      epochMilliseconds: 123,
      unixSeconds: "0.123",
      unixMilliseconds: "123",
    });
  });

  it("clearly rejects missing timezone, invalid calendar dates and unknown offsets", () => {
    expect(convertIsoTimestamp("2026-10-26T10:00:00")).toMatchObject({
      ok: false,
      code: "invalid_iso8601",
    });
    expect(convertIsoTimestamp("2025-02-29T10:00:00Z")).toMatchObject({
      ok: false,
      code: "invalid_iso8601",
    });
    expect(convertIsoTimestamp("2026-10-26T10:00:00-00:00")).toMatchObject({
      ok: false,
      code: "unknown_timezone_offset",
    });
  });
});
