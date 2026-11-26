// Auto-update from GitHub Releases (signed; see tauri.conf.json → plugins.updater).
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { toast } from "./store.svelte";

export const updater = $state({
  available: null as { version: string; notes: string } | null,
  checking: false,
  /** 0..1 while downloading, null otherwise */
  progress: null as number | null,
  dismissed: false,
});

let pending: Update | null = null;

export async function checkForUpdates(manual = false) {
  if (updater.checking) return;
  updater.checking = true;
  try {
    pending = await check();
    if (pending) {
      updater.available = { version: pending.version, notes: pending.body ?? "" };
      updater.dismissed = false;
    } else if (manual) {
      toast("Você está na versão mais recente", "success");
    }
  } catch (e) {
    // Silent on startup (offline, or no release published yet).
    if (manual) toast(`Não foi possível verificar atualizações: ${e}`, "error");
  } finally {
    updater.checking = false;
  }
}

export async function installUpdate() {
  if (!pending || updater.progress !== null) return;
  let total = 0;
  let done = 0;
  updater.progress = 0;
  try {
    await pending.downloadAndInstall((ev) => {
      if (ev.event === "Started") total = ev.data.contentLength ?? 0;
      else if (ev.event === "Progress") {
        done += ev.data.chunkLength;
        updater.progress = total ? done / total : 0;
      } else if (ev.event === "Finished") updater.progress = 1;
    });
    await relaunch();
  } catch (e) {
    updater.progress = null;
    toast(`Falha ao atualizar: ${e}`, "error");
  }
}

/** Check shortly after start, then every 6 hours. Skipped in `tauri dev`. */
export function startUpdateChecks() {
  if (import.meta.env.DEV) return;
  setTimeout(() => checkForUpdates(), 5_000);
  setInterval(() => checkForUpdates(), 6 * 60 * 60 * 1000);
}
