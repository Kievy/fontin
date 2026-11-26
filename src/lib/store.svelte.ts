import { listen } from "@tauri-apps/api/event";
import {
  api,
  EVENTS,
  type DownloadProgress,
  type DuplicateFile,
  type FamilyDetail,
  type FamilyRow,
  type Overview,
  type Query,
  type ScanProgress,
  type Scope,
  type Sort,
} from "./api";
import { applyTheme, type ThemePref } from "./theme";
import { applyLocale, t, DEFAULT_PREVIEWS, type LangPref } from "./i18n.svelte";

export type Section = "library" | "google";
export type View = "list" | "grid";

const PREFS_KEY = "fontin.prefs";

interface Prefs {
  previewText: string;
  size: number;
  view: View;
  theme: ThemePref;
}

const DEFAULT_PREFS: Prefs = {
  // Empty = the current language's default sentence (see previewText()).
  previewText: "",
  size: 40,
  view: "list",
  theme: "system",
};

function readPrefs(): Prefs {
  try {
    const saved = { ...DEFAULT_PREFS, ...JSON.parse(localStorage.getItem(PREFS_KEY) ?? "{}") };
    // Older versions stored the default sentence itself; treat it as "not customized".
    if (DEFAULT_PREVIEWS.includes(saved.previewText)) saved.previewText = "";
    return saved;
  } catch {
    return DEFAULT_PREFS;
  }
}

export const prefs = $state<Prefs>(readPrefs());

export function setTheme(theme: ThemePref) {
  prefs.theme = theme;
  applyTheme(theme);
  savePrefs();
}

/** Sample text shown in previews: the user's, or the language default. */
export function previewText(): string {
  return prefs.previewText || t("preview.default");
}

export async function setLanguage(pref: LangPref) {
  applyLocale(pref);
  try {
    await api.setLanguage(pref);
  } catch (e) {
    toast(String(e), "error");
  }
  // Backend-produced labels (e.g. error texts) may now differ.
  refreshAll();
}

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

export interface ToastAction {
  label: string;
  run: () => void | Promise<void>;
}
export interface Toast {
  id: number;
  kind: "info" | "error" | "success";
  text: string;
  detail?: string;
  action?: ToastAction;
}
export const toasts = $state<Toast[]>([]);
let toastSeq = 0;
export function dismissToast(id: number) {
  const i = toasts.findIndex((t) => t.id === id);
  if (i >= 0) toasts.splice(i, 1);
}
export function toast(text: string, kind: Toast["kind"] = "info", opts: { action?: ToastAction; detail?: string } = {}) {
  const id = ++toastSeq;
  toasts.push({ id, kind, text, ...opts });
  // Toasts that ask for a decision stay longer.
  setTimeout(() => dismissToast(id), opts.action ? 12000 : kind === "error" ? 6000 : 3200);
}

/** Identical files were held back: tell the user and let them add them anyway. */
export function offerDuplicates(dups: DuplicateFile[]) {
  if (!dups.length) return;
  const text = dups.length === 1 ? t("dup.one", { family: dups[0].family }) : t("dup.many", { n: dups.length });
  const detail = dups.length === 1 ? t("dup.where", { path: dups[0].existing }) : undefined;
  toast(text, "info", {
    detail,
    action: {
      label: t("dup.addAnyway"),
      run: async () => {
        try {
          await api.allowDuplicates(dups.map((d) => d.path));
          toast(dups.length === 1 ? t("dup.addedOne") : t("dup.addedMany"), "success");
        } catch (e) {
          toast(String(e), "error");
        }
      },
    },
  });
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

/** Family being dragged from the list (drives drop-target highlights). */
export const drag = $state({ row: null as FamilyRow | null });

export async function addToCollection(collectionId: number, row: FamilyRow) {
  const c = data.overview?.collections.find((x) => x.id === collectionId);
  try {
    await api.setInCollection(collectionId, [row.id], true);
    toast(t("toast.addedToCollection", { name: row.name, c: c?.name ?? t("title.collection") }), "success");
  } catch (e) {
    toast(String(e), "error");
  }
}

export async function addToFavorites(row: FamilyRow) {
  if (row.favorite) return;
  await toggleFavorite(row);
  toast(t("toast.addedToFavorites", { name: row.name }), "success");
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
  // Language lives in the backend settings (the tray menu uses it too).
  try {
    const s = await api.getSettings();
    applyLocale((s.language || "system") as LangPref);
  } catch {
    applyLocale("system");
  }
  applyTheme(prefs.theme);
  await Promise.all([
    listen(EVENTS.library, refreshAll),
    listen(EVENTS.catalog, refreshAll),
    listen<string>(EVENTS.notice, (e) => toast(e.payload, "success")),
    listen<DownloadProgress>(EVENTS.download, (e) => {
      const p = e.payload;
      if (p.finished) {
        delete data.downloads[p.familyId];
        if (p.error) toast(t("toast.downloadFail", { e: p.error }), "error");
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
