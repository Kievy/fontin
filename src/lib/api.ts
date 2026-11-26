import { invoke } from "@tauri-apps/api/core";

export type Provider = "local" | "google";

export interface FamilyRow {
  id: string;
  name: string;
  provider: Provider;
  styles: number;
  format: string;
  /** 0 off · 1 partial · 2 on */
  active: 0 | 1 | 2;
  favorite: boolean;
  downloaded: boolean;
  previewFace: string | null;
  previewWeight: number;
  previewItalic: boolean;
  category: string | null;
}

export interface FaceInfo {
  id: string;
  style: string;
  fullName: string;
  postscript: string;
  weight: number;
  italic: boolean;
  width: number;
  variable: boolean;
  format: string;
  path: string;
  size: number;
  active: boolean;
}

export interface FamilyDetail {
  row: FamilyRow;
  faces: FaceInfo[];
  collections: number[];
  designers: string[];
}

export interface FolderNode {
  path: string;
  name: string;
  count: number;
  children: FolderNode[];
}

export interface SourceInfo {
  id: number;
  name: string;
  path: string;
  count: number;
  active: number;
  tree: FolderNode[];
}

export interface CollectionInfo {
  id: number;
  name: string;
  color: string;
  count: number;
  active: number;
}

export interface Counts {
  all: number;
  recent: number;
  favorites: number;
  active: number;
  inactive: number;
  google: number;
  googleActive: number;
}

export interface Overview {
  counts: Counts;
  sources: SourceInfo[];
  collections: CollectionInfo[];
  googleReady: boolean;
}

export type Scope =
  | { kind: "all" }
  | { kind: "recent" }
  | { kind: "favorites" }
  | { kind: "active" }
  | { kind: "inactive" }
  | { kind: "google" }
  | { kind: "collection"; value: number }
  | { kind: "source"; value: number }
  | { kind: "folder"; value: string };

export type Sort = "name" | "recent" | "popular" | "styles";

export interface Query {
  scope: Scope;
  search: string;
  sort: Sort;
  category: string | null;
}

export const api = {
  overview: () => invoke<Overview>("overview"),
  query: (query: Query) => invoke<FamilyRow[]>("query_families", { query }),
  detail: (id: string) => invoke<FamilyDetail | null>("family_detail", { id }),
  setFamilyActive: (id: string, active: boolean) => invoke<void>("set_family_active", { id, active }),
  setFamiliesActive: (ids: string[], active: boolean) => invoke<void>("set_families_active", { ids, active }),
  setFaceActive: (id: string, active: boolean) => invoke<void>("set_face_active", { id, active }),
  toggleFavorite: (id: string) => invoke<boolean>("toggle_favorite", { id }),
  addFolder: (path: string) => invoke<number>("add_folder", { path }),
  rescanSource: (id: number) => invoke<void>("rescan_source", { id }),
  removeSource: (id: number) => invoke<void>("remove_source", { id }),
  createCollection: (name: string, color: string) => invoke<number>("create_collection", { name, color }),
  updateCollection: (id: number, name: string, color: string) =>
    invoke<void>("update_collection", { id, name, color }),
  deleteCollection: (id: number) => invoke<void>("delete_collection", { id }),
  setInCollection: (id: number, familyIds: string[], member: boolean) =>
    invoke<void>("set_in_collection", { id, familyIds, member }),
  refreshGoogle: (force = false) => invoke<number>("refresh_google", { force }),
};

/** Raw bytes of a local face (binary IPC, no filesystem access from JS). */
export const faceData = (faceId: string) => invoke<ArrayBuffer>("face_data", { id: faceId });

export const EVENTS = {
  library: "library-changed",
  catalog: "catalog-changed",
  download: "download-progress",
  scan: "scan-progress",
} as const;

export interface DownloadProgress {
  familyId: string;
  done: number;
  total: number;
  finished: boolean;
  error: string | null;
}

export interface ScanProgress {
  sourceId: number;
  scanning: boolean;
  added: number;
  removed: number;
}
