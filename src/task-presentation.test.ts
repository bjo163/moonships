import { describe, expect, it } from "vitest";
import { formatTaskDate, isTaskOverdue } from "./task-presentation";

describe("Task date and priority presentation", () => {
  it("compares due instants against the current instant while UTC remains the stored form", () => {
    const dueAt = "2026-10-27T00:30:00Z";
    expect(isTaskOverdue(dueAt, "todo", new Date("2026-10-27T00:29:59Z"))).toBe(false);
    expect(isTaskOverdue(dueAt, "todo", new Date("2026-10-27T00:30:01Z"))).toBe(true);
  });

  it("does not mark completed or cancelled tasks overdue", () => {
    const dueAt = "2020-01-01T00:00:00Z";
    const now = new Date("2026-10-26T12:00:00Z");
    expect(isTaskOverdue(dueAt, "done", now)).toBe(false);
    expect(isTaskOverdue(dueAt, "cancelled", now)).toBe(false);
    expect(isTaskOverdue(null, "todo", now)).toBe(false);
    expect(isTaskOverdue("not a date", "todo", now)).toBe(false);
  });

  it("formats the UTC instant using the user's local date", () => {
    const dueAt = "2026-10-27T00:30:00Z";
    const expected = new Intl.DateTimeFormat(undefined, { dateStyle: "medium" }).format(
      new Date(dueAt),
    );
    expect(formatTaskDate(dueAt)).toBe(expected);
  });
});
