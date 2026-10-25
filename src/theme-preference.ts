export type ThemePreference = "dark" | "light";

const THEME_STORAGE_KEY = "moonships.theme";

type ThemeStorageReader = Pick<Storage, "getItem">;
type ThemeStorageWriter = Pick<Storage, "setItem">;
type ThemeRoot = Pick<HTMLElement, "setAttribute" | "removeAttribute">;

export function readThemePreference(storage?: ThemeStorageReader): ThemePreference {
  try {
    return storage?.getItem(THEME_STORAGE_KEY) === "light" ? "light" : "dark";
  } catch {
    return "dark";
  }
}

export function writeThemePreference(
  theme: ThemePreference,
  storage?: ThemeStorageWriter,
): boolean {
  try {
    storage?.setItem(THEME_STORAGE_KEY, theme);
    return Boolean(storage);
  } catch {
    return false;
  }
}

export function applyThemePreference(theme: ThemePreference, root: ThemeRoot): void {
  if (theme === "light") {
    root.setAttribute("data-theme", "light");
    return;
  }
  root.removeAttribute("data-theme");
}
