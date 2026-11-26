// Lazy font loading for previews. Only rows on screen request their font;
// an LRU keeps the number of registered FontFaces bounded.
import { faceData, type FamilyRow } from "./api";

const MAX_FACES = 320;
const faces = new Map<string, FontFace>(); // insertion order = LRU order
const pending = new Map<string, Promise<boolean>>();
const googleCss = new Map<string, Promise<boolean>>();

function touch(key: string) {
  const f = faces.get(key);
  if (f) {
    faces.delete(key);
    faces.set(key, f);
  }
}

function evict() {
  while (faces.size > MAX_FACES) {
    const [key, face] = faces.entries().next().value as [string, FontFace];
    faces.delete(key);
    document.fonts.delete(face);
  }
}

export const localFamily = (faceId: string) => `fx-${faceId}`;
export const googleFamily = (name: string) => `gx-${name.replace(/[^a-z0-9]+/gi, "-").toLowerCase()}`;

export function loadLocalFace(faceId: string): Promise<boolean> {
  if (faces.has(faceId)) {
    touch(faceId);
    return Promise.resolve(true);
  }
  let p = pending.get(faceId);
  if (!p) {
    let face: FontFace;
    p = faceData(faceId)
      .then((buf) => {
        face = new FontFace(localFamily(faceId), buf, { display: "block" });
        return face.load();
      })
      .then(() => {
        document.fonts.add(face);
        faces.set(faceId, face);
        evict();
        return true;
      })
      .catch(() => false)
      .finally(() => pending.delete(faceId));
    pending.set(faceId, p);
  }
  return p;
}

/** Google previews stream from the Fonts CSS API; nothing is written to disk. */
export function loadGoogleFamily(name: string, weight: number, italic: boolean): Promise<boolean> {
  const key = `${name}:${weight}:${italic}`;
  let p = googleCss.get(key);
  if (!p) {
    const fam = encodeURIComponent(name).replace(/%20/g, "+");
    const axis = italic ? `:ital,wght@1,${weight}` : weight !== 400 ? `:wght@${weight}` : "";
    const alias = googleFamily(name);
    p = fetch(`https://fonts.googleapis.com/css2?family=${fam}${axis}&display=block`)
      .then((r) => (r.ok ? r.text() : Promise.reject(r.status)))
      .then(async (css) => {
        const style = document.createElement("style");
        style.textContent = css
          .replace(/font-family:\s*'[^']+'/g, `font-family: '${alias}'`)
          .replace(/font-weight:\s*\d+/g, "font-weight: 400")
          .replace(/font-style:\s*italic/g, "font-style: normal");
        document.head.appendChild(style);
        await document.fonts.load(`16px "${alias}"`);
        return true;
      })
      .catch(() => {
        googleCss.delete(key);
        return false;
      });
    googleCss.set(key, p);
  }
  return p;
}

/** CSS font-family for a row's preview, or null if it cannot be previewed. */
export function rowFamily(row: FamilyRow): string | null {
  if (row.previewFace) return localFamily(row.previewFace);
  if (row.provider === "google") return googleFamily(row.name);
  return null;
}

export function loadRow(row: FamilyRow): Promise<boolean> {
  if (row.previewFace) return loadLocalFace(row.previewFace);
  if (row.provider === "google") return loadGoogleFamily(row.name, row.previewWeight, row.previewItalic);
  return Promise.resolve(false);
}
