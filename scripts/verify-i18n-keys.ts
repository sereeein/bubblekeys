import en from "../src/i18n/locales/en.json";
import zhCN from "../src/i18n/locales/zh-CN.json";
import zhTW from "../src/i18n/locales/zh-TW.json";
import ja from "../src/i18n/locales/ja.json";
import ko from "../src/i18n/locales/ko.json";

const locales = { "zh-CN": zhCN, "zh-TW": zhTW, ja, ko };
const enKeys = new Set(Object.keys(en));

let ok = true;
for (const [name, dict] of Object.entries(locales)) {
  const missing = [...enKeys].filter(k => !(k in dict));
  const extra   = Object.keys(dict).filter(k => !enKeys.has(k));
  if (missing.length || extra.length) {
    ok = false;
    console.error(`[${name}] missing: ${missing.join(", ") || "—"}`);
    console.error(`[${name}] extra:   ${extra.join(", ")   || "—"}`);
  }
}
if (!ok) {
  console.error("i18n verification failed.");
  process.exit(1);
}
console.log("i18n keys consistent across all locales ✓");
