// Tiny reactive i18n: `t("key", { n: 3 })`, `tn("styles", n)` for plurals.
import { ptBR, type Dict, type Key } from "./i18n/pt-BR";
import { en } from "./i18n/en";
import { es } from "./i18n/es";

export type Locale = "pt-BR" | "en" | "es";
export type LangPref = "system" | Locale;

export const LOCALES: { id: Locale; label: string }[] = [
  { id: "pt-BR", label: "Português (Brasil)" },
  { id: "en", label: "English" },
  { id: "es", label: "Español" },
];

const dicts: Record<Locale, Dict> = { "pt-BR": ptBR, en, es };

export function resolveLocale(pref: LangPref | string | undefined): Locale {
  const tag = (!pref || pref === "system" ? navigator.language : pref).toLowerCase();
  if (tag.startsWith("pt")) return "pt-BR";
  if (tag.startsWith("es")) return "es";
  return "en";
}

export const i18n = $state({ pref: "system" as LangPref, locale: resolveLocale("system") as Locale });

export function applyLocale(pref: LangPref) {
  i18n.pref = pref;
  i18n.locale = resolveLocale(pref);
  document.documentElement.lang = i18n.locale;
}

export function t(key: Key, params?: Record<string, string | number>): string {
  let s = dicts[i18n.locale][key] ?? ptBR[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      s = s.replaceAll(`{${k}}`, typeof v === "number" ? fmt(v) : v);
    }
  }
  return s;
}

type PluralBase = "families" | "styles";
export function tn(base: PluralBase, n: number): string {
  return t(`${base}_${n === 1 ? "one" : "other"}` as Key, { n });
}

export const fmt = (n: number) => n.toLocaleString(i18n.locale);

/** All built-in preview sentences, to know whether the user customized theirs. */
export const DEFAULT_PREVIEWS = Object.values(dicts).map((d) => d["preview.default"]);
