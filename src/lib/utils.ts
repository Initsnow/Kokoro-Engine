import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

const isWindows = () => typeof navigator !== "undefined" && navigator.userAgent.includes("Windows");

/**
 * Build a live2d:// URL that works on all platforms.
 * - Windows (WebView2): must use http://live2d.localhost/<path>
 * - macOS / Linux:      live2d://localhost/<path>
 */
export function live2dUrl(path: string): string {
  const cleanPath = path.startsWith("/") ? path.slice(1) : path;
  return isWindows()
    ? `http://live2d.localhost/${cleanPath}`
    : `live2d://localhost/${cleanPath}`;
}

/**
 * Convert a mod:// URL to the correct format for the current platform.
 * - Windows (WebView2): http://mod.localhost/<modId>/<path>
 * - macOS / Linux:      mod://<modId>/<path>  (pass-through)
 */
export function modUrl(src: string): string {
  if (!isWindows()) return src;
  return src.replace(/^mod:\/\//, "http://mod.localhost/");
}
