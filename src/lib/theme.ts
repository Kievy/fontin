// Theme resolution. The same logic runs inline in app.html before first paint
// (to avoid a flash), so keep the storage key and attribute in sync with it.
import { getCurrentWindow } from "@tauri-apps/api/window";

export type ThemePref = "system" | "light" | "dark";

const media = typeof window !== "undefined" ? window.matchMedia("(prefers-color-scheme: light)") : null;

export function resolve(pref: ThemePref): "light" | "dark" {
  if (pref === "system") return media?.matches ? "light" : "dark";
  return pref;
}

let current: ThemePref = "system";

export function applyTheme(pref: ThemePref) {
  current = pref;
  document.documentElement.dataset.theme = resolve(pref);
  // Native pieces (file dialogs, title bar) follow the app; null = follow the OS.
  getCurrentWindow()
    .setTheme(pref === "system" ? null : pref)
    .catch(() => {});
}

media?.addEventListener("change", () => {
  if (current === "system") document.documentElement.dataset.theme = resolve("system");
});
