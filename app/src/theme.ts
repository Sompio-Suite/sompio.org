// Light, dark or whatever the computer uses. Kept on this device only.
export type ThemeChoice = "light" | "dark" | "system";

const KEY = "sompio.theme";
const inDesktopApp = "__TAURI_INTERNALS__" in window;
const systemDark = window.matchMedia("(prefers-color-scheme: dark)");

export function themeChoice(): ThemeChoice {
  try {
    const v = localStorage.getItem(KEY);
    return v === "dark" || v === "system" ? v : "light";
  } catch {
    return "light";
  }
}

/** Colours the page, and the window's title bar in the desktop app. */
export function applyTheme(choice: ThemeChoice = themeChoice()) {
  const dark = choice === "dark" || (choice === "system" && systemDark.matches);
  document.documentElement.dataset.theme = dark ? "dark" : "light";
  if (inDesktopApp) {
    import("@tauri-apps/api/core")
      .then(({ invoke }) => invoke("set_app_theme", { theme: choice }))
      .catch(() => {});
  }
}

export function setThemeChoice(choice: ThemeChoice) {
  try {
    localStorage.setItem(KEY, choice);
  } catch {
    /* not available: it just isn't remembered */
  }
  applyTheme(choice);
}

/** Follows the computer's setting while "system" is chosen, and changes made in another Sompio window. */
export function watchTheme() {
  systemDark.addEventListener("change", () => themeChoice() === "system" && applyTheme());
  window.addEventListener("storage", (e) => e.key === KEY && applyTheme());
}
