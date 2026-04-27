import { getSettings, updateSettings, type Settings } from "../lib/ipc";
import { detectLocale, setLocale, t, type Locale } from "../i18n";
import { refreshActiveTab } from "../lib/router";

export async function renderSettings(host: HTMLElement) {
  const s = await getSettings();

  host.innerHTML = `
    <ul class="settings-list">
      ${row(t("settings.hotkey"), input("hotkey", s.hotkey, "text"))}
      ${row(t("settings.autostart"), toggle("auto_start", s.auto_start))}
      ${row(t("settings.pitch_jitter"), toggle("pitch_jitter", s.pitch_jitter))}
      ${row(t("settings.menu_icon"), toggle("menu_icon_visible", s.menu_icon_visible))}
      ${row(t("settings.output"), input("output_device", s.output_device, "text"))}
      ${row(t("settings.language"), select("language", s.language, [
        ["auto", "Auto"], ["en", "English"], ["zh-CN","简体中文"],
        ["zh-TW","繁體中文"], ["ja","日本語"], ["ko","한국어"]
      ]))}
      ${row(t("settings.night_silent"), toggle("night_silent.enabled", s.night_silent.enabled))}
      ${row(t("settings.night_silent.window"),
         `${input("night_silent.start", s.night_silent.start, "time")}<span class="row-sep">–</span>${input("night_silent.end", s.night_silent.end, "time")}`)}
    </ul>
  `;

  host.querySelector<HTMLFormElement>(".settings-list")!.addEventListener("change", async (e) => {
    const tgt = e.target as HTMLInputElement | HTMLSelectElement;
    const next: Settings = await getSettings();
    const path = tgt.dataset.key!.split(".");
    let obj: any = next;
    for (let i = 0; i < path.length - 1; i++) obj = obj[path[i]];
    const last = path[path.length - 1];
    if (tgt.type === "checkbox") obj[last] = (tgt as HTMLInputElement).checked;
    else obj[last] = tgt.value;
    await updateSettings(next);
    if (path[0] === "language") {
      const v = tgt.value;
      setLocale((v === "auto" ? detectLocale() : v) as Locale);
      await refreshActiveTab();
    }
  });
}

function row(label: string, control: string) {
  return `<li class="settings-row"><span class="lbl">${label}</span>${control}</li>`;
}
function input(key: string, val: string, type: string) {
  return `<input type="${type}" class="set-val" data-key="${key}" value="${val}">`;
}
function toggle(key: string, on: boolean) {
  return `<label class="toggle"><input type="checkbox" data-key="${key}" ${on?'checked':''}><span></span></label>`;
}
function select(key: string, val: string, opts: [string,string][]) {
  return `<select data-key="${key}" class="set-val">${
    opts.map(([v,l]) => `<option value="${v}" ${v===val?'selected':''}>${l}</option>`).join("")
  }</select>`;
}
