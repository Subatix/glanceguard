import type { AppTheme } from "../settings/types";

/** Applies `data-theme` on `<html>`: system clears the attribute for CSS media fallback. */
export function syncDomTheme(theme: AppTheme | undefined): void {
  const root = document.documentElement;
  const t = theme ?? "system";
  if (t === "system") {
    root.removeAttribute("data-theme");
    return;
  }
  root.setAttribute("data-theme", t);
}
