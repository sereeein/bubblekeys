import { open } from "@tauri-apps/plugin-dialog";
import { getState, listPacks, setActivePack, previewPack as ipcPreview, deletePack, importPack } from "../lib/ipc";
import { pixelConfirm, pixelPrompt } from "../lib/modal";
import { t } from "../i18n";

export async function renderPacks(host: HTMLElement) {
  const [packs, state] = await Promise.all([listPacks(), getState()]);

  host.innerHTML = `
    <ul class="pack-list" role="listbox">
      ${packs.map(p => `
        <li class="pack-row ${p.id === state.active_pack ? 'sel' : ''}" role="option"
            data-id="${p.id}" tabindex="0" aria-selected="${p.id === state.active_pack}">
          <span class="pack-name-text">${escapeHtml(p.name)}</span>
          <span class="pack-row-actions">
            <span class="meta">${p.id === state.active_pack ? '♪' : ''}</span>
            ${p.bundled ? '' : `<button class="pack-del" data-act="delete" data-name="${escapeHtml(p.name)}">×</button>`}
          </span>
        </li>
      `).join("")}
      <li class="pack-import" data-action="import">${t("packs.import")}</li>
    </ul>`;

  host.querySelectorAll<HTMLLIElement>(".pack-row").forEach(li => {
    li.addEventListener("click", (e) => {
      const tgt = e.target as HTMLElement;
      if (tgt.classList.contains("pack-del")) return;
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
      setActivePack(id).catch(err => console.error("setActivePack:", err));
      ipcPreview(id).catch(err => console.error("previewPack:", err));
    });
  });

  host.querySelectorAll<HTMLButtonElement>(".pack-del").forEach(btn => {
    btn.addEventListener("click", async (e) => {
      e.stopPropagation();
      const li = btn.closest<HTMLLIElement>(".pack-row")!;
      const id = li.dataset.id!;
      const name = btn.dataset.name ?? id;
      const confirmed = await pixelConfirm({
        title: t("packs.delete_confirm", { name }),
        okLabel: t("common.ok"),
        cancelLabel: t("common.cancel"),
      });
      if (!confirmed) return;
      try {
        await deletePack(id);
        await renderPacks(host);
      } catch (err) {
        console.error("delete failed:", err);
        alert(`Delete failed: ${err}`);
      }
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

      const basename = path.split(/[\\/]/).pop() ?? "";
      const stem = basename.replace(/\.zip$/i, "");

      const customName = await pixelPrompt({
        title: t("packs.name_prompt"),
        defaultValue: stem,
        okLabel: t("common.ok"),
        cancelLabel: t("common.cancel"),
      });
      if (customName === null) return;
      const trimmed = customName.trim();
      const finalName: string | null = trimmed === "" ? null : trimmed;

      try {
        await importPack(path, finalName);
        await renderPacks(host);
      } catch (err) {
        console.error("import failed:", err);
        alert(`Import failed: ${err}`);
      }
    });
  }
}

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, c =>
    ({ "&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;" }[c]!)
  );
}
