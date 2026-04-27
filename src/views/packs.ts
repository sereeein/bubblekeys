import { getState, listPacks, setActivePack, previewPack as ipcPreview } from "../lib/ipc";
import { t } from "../i18n";

export async function renderPacks(host: HTMLElement) {
  const [packs, state] = await Promise.all([listPacks(), getState()]);

  host.innerHTML = `
    <ul class="pack-list" role="listbox">
      ${packs.map(p => `
        <li class="pack-row ${p.id === state.active_pack ? 'sel' : ''}" role="option"
            data-id="${p.id}" tabindex="0" aria-selected="${p.id === state.active_pack}">
          <span>${p.name}</span><span class="meta">${p.id === state.active_pack ? '♪' : ''}</span>
        </li>
      `).join("")}
      <li class="pack-import" data-action="import">${t("packs.import")}</li>
    </ul>`;

  host.querySelectorAll<HTMLLIElement>(".pack-row").forEach(li => {
    li.addEventListener("click", async () => {
      await setActivePack(li.dataset.id!);
      host.querySelectorAll(".pack-row").forEach(x => x.classList.remove("sel"));
      li.classList.add("sel");
    });
    li.addEventListener("mouseenter", () => previewPack(li.dataset.id!));
  });
}

let lastPreview = 0;
async function previewPack(id: string) {
  // Phase 5: throttle hover; actual preview command added in Phase 10 (Mechvibes import).
  const now = Date.now();
  if (now - lastPreview < 200) return;
  lastPreview = now;
  await ipcPreview(id);
}
