// Which OS the webview runs on (WebView2 → Windows, WKWebView → macOS).
const ua = typeof navigator !== "undefined" ? navigator.userAgent : "";

export const isWindows = ua.includes("Windows");
export const isMac = ua.includes("Macintosh") || ua.includes("Mac OS X");

/**
 * Windows and macOS draw no native title bar (see tauri.windows.conf.json /
 * tauri.macos.conf.json), so the app renders its own. On Linux the window
 * manager keeps its decorations.
 */
export const customTitleBar = isWindows || isMac;
