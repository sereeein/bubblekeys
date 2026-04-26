import { getSettings, updateSettings, type Settings } from "../lib/ipc";

export async function renderSettings(host: HTMLElement) {
  const s = await getSettings();

  host.innerHTML = `
    <ul class="settings-list">
      ${row("HOTKEY", input("hotkey", s.hotkey, "text"))}
      ${row("AUTOSTART", toggle("auto_start", s.auto_start))}
      ${row("PITCH VAR", toggle("pitch_jitter", s.pitch_jitter))}
      ${row("MENU ICON", toggle("menu_icon_visible", s.menu_icon_visible))}
      ${row("OUTPUT", input("output_device", s.output_device, "text"))}
      ${row("LANGUAGE", select("language", s.language, [
        ["auto", "Auto"], ["en", "English"], ["zh-CN","简体中文"],
        ["zh-TW","繁體中文"], ["ja","日本語"], ["ko","한국어"]
      ]))}
    </ul>
  `;

  host.querySelector<HTMLFormElement>(".settings-list")!.addEventListener("change", async (e) => {
    const t = e.target as HTMLInputElement | HTMLSelectElement;
    const next: Settings = await getSettings();
    const key = t.dataset.key as keyof Settings;
    if (t.type === "checkbox") (next as any)[key] = (t as HTMLInputElement).checked;
    else (next as any)[key] = t.value;
    await updateSettings(next);
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
