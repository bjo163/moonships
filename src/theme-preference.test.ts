import { describe, expect, it } from "vitest";
import {
  applyThemePreference,
  readThemePreference,
  writeThemePreference,
  type ThemePreference,
} from "./theme-preference";

function createStorage(initial: Record<string, string> = {}) {
  const values = new Map(Object.entries(initial));
  return {
    getItem(key: string) {
      return values.get(key) ?? null;
    },
    setItem(key: string, value: string) {
      values.set(key, value);
    },
    values,
  };
}

describe("theme preference", () => {
  it("defaults to dark and restores a persisted light preference", () => {
    expect(readThemePreference()).toBe("dark");
    expect(readThemePreference(createStorage({ "moonships.theme": "light" }))).toBe("light");
    expect(readThemePreference(createStorage({ "moonships.theme": "unknown" }))).toBe("dark");
  });

  it("persists both supported preferences without throwing on storage failures", () => {
    const storage = createStorage();
    const preferences: ThemePreference[] = ["light", "dark"];
    for (const preference of preferences) {
      expect(writeThemePreference(preference, storage)).toBe(true);
      expect(readThemePreference(storage)).toBe(preference);
    }
    expect(writeThemePreference("light")).toBe(false);
    expect(
      writeThemePreference("light", {
        setItem() {
          throw new Error("storage blocked");
        },
      }),
    ).toBe(false);
  });

  it("applies light theme and removes the override for dark theme", () => {
    const attributes = new Map<string, string>();
    const root = {
      setAttribute(name: string, value: string) {
        attributes.set(name, value);
      },
      removeAttribute(name: string) {
        attributes.delete(name);
      },
    };
    applyThemePreference("light", root);
    expect(attributes.get("data-theme")).toBe("light");
    applyThemePreference("dark", root);
    expect(attributes.has("data-theme")).toBe(false);
  });
});
