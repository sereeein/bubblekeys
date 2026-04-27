import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
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
    li.addEventListener("click", () => {
      const id = li.dataset.id!;
      // Optimistic UI: update highlight + meta marker before backend round-trip.
      host.querySelectorAll<HTMLLIElement>(".pack-row").forEach(x => {
        x.classList.remove("sel");
        const meta = x.querySelector<HTMLSpanElement>(".meta");
        if (meta) meta.textContent = "";
      });
      li.classList.add("sel");
      const meta = li.querySelector<HTMLSpanElement>(".meta");
      if (meta) meta.textContent = "♪";
      // Fire-and-forget: settings persist + preview both happen async; UI doesn't wait.
      setActivePack(id).catch(e => console.error("setActivePack:", e));
      ipcPreview(id).catch(e => console.error("previewPack:", e));
    });
  });

  const importBtn = host.querySelector<HTMLLIElement>("[data-action='import']");
  if (importBtn) {
    importBtn.addEventListener("click", async () => {
      const path = await open({
        filters: [{ name: "Mechvibes pack", extensions: ["zip"] }],
        multiple: false,
        directory: false,
      });
      if (!path || typeof path !== "string") return;
      try {
        await invoke("import_pack", { archivePath: path });
        await renderPacks(host);
      } catch (e) {
        console.error("import failed:", e);
        alert(`Import failed: ${e}`);
      }
    });
  }
}

