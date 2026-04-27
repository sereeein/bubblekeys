import en from "./locales/en.json";
import zhCN from "./locales/zh-CN.json";
import zhTW from "./locales/zh-TW.json";
import ja from "./locales/ja.json";
import ko from "./locales/ko.json";

export type Locale = "en" | "zh-CN" | "zh-TW" | "ja" | "ko";
type Dict = Record<string, string>;
const DICTS: Record<Locale, Dict> = { en, "zh-CN": zhCN, "zh-TW": zhTW, ja, ko };

let current: Locale = "en";

export function setLocale(l: Locale) { current = l; }
export function getLocale(): Locale { return current; }

export function detectLocale(): Locale {
  const sys = navigator.language || "en";
  if (sys.startsWith("zh-CN") || sys === "zh") return "zh-CN";
  if (sys.startsWith("zh-TW") || sys.startsWith("zh-HK")) return "zh-TW";
  if (sys.startsWith("ja")) return "ja";
  if (sys.startsWith("ko")) return "ko";
  return "en";
}

export function t(key: string, vars?: Record<string, string>): string {
  const raw = DICTS[current][key] ?? DICTS.en[key] ?? key;
  if (!vars) return raw;
  return raw.replace(/\{(\w+)\}/g, (_, k) => vars[k] ?? `{${k}}`);
}
