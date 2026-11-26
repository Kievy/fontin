import { listen } from "@tauri-apps/api/event";
import {
  api,
  EVENTS,
  type DownloadProgress,
  type FamilyDetail,
  type FamilyRow,
  type Overview,
  type Query,
  type ScanProgress,
  type Scope,
  type Sort,
} from "./api";

export type Section = "library" | "google";
export type View = "list" | "grid";

const PREFS_KEY = "fontin.prefs";

interface Prefs {
  previewText: string;
  size: number;
  view: View;
}

const DEFAULT_PREFS: Prefs = {
  previewText: "Um pequeno jabuti xereta viu dez cegonhas felizes.",
  size: 40,
  view: "list",
};

function readPrefs(): Prefs {
  try {
    return { ...DEFAULT_PREFS, ...JSON.parse(localStorage.getItem(PREFS_KEY) ?? "{}") };
  } catch {
    return DEFAULT_PREFS;
  }
}

export const prefs = $state<Prefs>(readPrefs());

export function savePrefs() {
  try {
    localStorage.setItem(PREFS_KEY, JSON.stringify(prefs));
  } catch {
    /* storage unavailable: prefs stay in memory */
  }
}

export const ui = $state({
  section: "library" as Section,
  scope: { kind: "all" } as Scope,
  search: "",
  sort: "name" as Sort,
  category: null as string | null,
  selected: null as string | null,
  settingsOpen: false,
});

export const data = $state({
  overview: null as Overview | null,
  rows: [] as FamilyRow[],
  loading: true,
  detail: null as FamilyDetail | null,
  downloads: {} as Record<string, DownloadProgress>,
  scanning: {} as Record<number, boolean>,
  pending: {} as Record<string, boolean>,
});

export interface Toast {
  id: number;
  kind: "info" | "error" | "success";
  text: string;
}
export const toasts = $state<Toast[]>([]);
let toastSeq = 0;
export function toast(text: string, kind: Toast["kind"] = "info") {
  const id = ++toastSeq;
  toasts.push({ id, kind, text });
  setTimeout(() => {
    const i = toasts.findIndex((t) => t.id === id);
    if (i >= 0) toasts.splice(i, 1);
  }, kind === "error" ? 6000 : 3200);
}

let querySeq = 0;
export async function refreshRows() {
  const seq = ++querySeq;
  const q: Query = { scope: ui.scope, search: ui.search, sort: ui.sort, category: ui.category };
  try {
    const rows = await api.query(q);
    if (seq === querySeq) {
      data.rows = rows;
      data.loading = false;
    }
  } catch (e) {
    toast(String(e), "error");
  }
}

export async function refreshOverview() {
  data.overview = await api.overview();
}

export async function refreshDetail() {
  data.detail = ui.selected ? await api.detail(ui.selected) : null;
}

let refreshTimer: ReturnType<typeof setTimeout> | undefined;
/** Coalesce bursts of backend events into one refresh. */
export function refreshAll() {
  clearTimeout(refreshTimer);
  refreshTimer = setTimeout(() => {
    refreshOverview();
    refreshRows();
    refreshDetail();
  }, 40);
}

export function setScope(scope: Scope, section: Section = "library") {
  ui.section = section;
  ui.scope = scope;
  if (section === "google") {
    if (ui.sort === "recent") ui.sort = "popular";
  } else {
    ui.category = null;
    if (ui.sort === "popular") ui.sort = "name";
  }
  data.loading = true;
  refreshRows();
}

export function select(id: string | null) {
  ui.selected = ui.selected === id ? null : id;
  refreshDetail();
}

export async function toggleFamily(row: FamilyRow) {
  if (data.pending[row.id]) return;
  const next = row.active !== 2;
  data.pending[row.id] = true;
  // Optimistic: flip immediately, the backend event reconciles.
  const prev = row.active;
  row.active = next ? 2 : 0;
  try {
    await api.setFamilyActive(row.id, next);
  } catch (e) {
    row.active = prev;
    toast(`${row.name}: ${e}`, "error");
  } finally {
    delete data.pending[row.id];
  }
}

export async function toggleFavorite(row: FamilyRow) {
  row.favorite = !row.favorite;
  try {
    await api.toggleFavorite(row.id);
  } catch (e) {
    row.favorite = !row.favorite;
    toast(String(e), "error");
  }
}

export function isScope(a: Scope, b: Scope) {
  return a.kind === b.kind && ("value" in a ? a.value : null) === ("value" in b ? b.value : null);
}

export async function init() {
  await Promise.all([
    listen(EVENTS.library, refreshAll),
    listen(EVENTS.catalog, refreshAll),
    listen<DownloadProgress>(EVENTS.download, (e) => {
      const p = e.payload;
      if (p.finished) {
        delete data.downloads[p.familyId];
        if (p.error) toast(`Falha ao baixar: ${p.error}`, "error");
      } else {
        data.downloads[p.familyId] = p;
      }
    }),
    listen<ScanProgress>(EVENTS.scan, (e) => {
      const p = e.payload;
      if (p.scanning) data.scanning[p.sourceId] = true;
      else delete data.scanning[p.sourceId];
    }),
  ]);
  await Promise.all([refreshOverview(), refreshRows()]);
}
