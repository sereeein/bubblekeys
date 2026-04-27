import { open } from "@tauri-apps/plugin-dialog";
import { getState, listPacks, setActivePack, previewPack as ipcPreview, deletePack, importPack } from "../lib/ipc";
import { pixelConfirm, pixelPrompt } from "../lib/modal";
import { t } from "../i18n";

const PAGE_SIZE = 8;
let currentPage = 0;

export async function renderPacks(host: HTMLElement): Promise<() => void> {
  const [initialPacks, state] = await Promise.all([listPacks(), getState()]);
  const packs = [...initialPacks];

  function totalPages() {
    return Math.max(1, Math.ceil(packs.length / PAGE_SIZE));
  }

  // Clamp page if pack list shrank since last render (e.g. via delete then tab away).
  if (currentPage >= totalPages()) currentPage = Math.max(0, totalPages() - 1);

  function paint() {
    const total = totalPages();
    const startIdx = currentPage * PAGE_SIZE;
    const slice = packs.slice(startIdx, startIdx + PAGE_SIZE);
    const isLastPage = currentPage === total - 1;

    const pagerHtml = total > 1 ? `
      <div class="pack-pager">
        <button class="pixel-btn off" data-pg="prev" ${currentPage === 0 ? 'disabled' : ''}>◀</button>
        <span>${t("packs.page", { n: String(currentPage + 1), total: String(total) })}</span>
        <button class="pixel-btn off" data-pg="next" ${currentPage >= total - 1 ? 'disabled' : ''}>▶</button>
      </div>` : '';

    host.innerHTML = `
      ${pagerHtml}
      <ul class="pack-list" role="listbox">
        ${slice.map(p => `
          <li class="pack-row ${p.id === state.active_pack ? 'sel' : ''}" role="option"
              data-id="${p.id}" tabindex="0" aria-selected="${p.id === state.active_pack}">
            <span class="pack-name-text">${escapeHtml(p.name)}</span>
            <span class="pack-row-actions">
              <span class="meta">${p.id === state.active_pack ? '♪' : ''}</span>
              ${p.bundled ? '' : `<button class="pack-del" data-act="delete" data-name="${escapeHtml(p.name)}">×</button>`}
            </span>
          </li>
        `).join("")}
        ${isLastPage ? `<li class="pack-import" data-action="import">${t("packs.import")}</li>` : ''}
      </ul>`;

    bindRowClicks();
    bindDeleteButtons();
    if (isLastPage) bindImportButton();
    bindPagerButtons();
  }

  function bindRowClicks() {
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
  }

  function bindDeleteButtons() {
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
          await reload();
        } catch (err) {
          console.error("delete failed:", err);
          alert(`Delete failed: ${err}`);
        }
      });
    });
  }

  function bindImportButton() {
    const importBtn = host.querySelector<HTMLLIElement>("[data-action='import']");
    if (!importBtn) return;
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
        // Jump to last page so the user sees the just-imported pack.
        await reload({ jumpToLastPage: true });
      } catch (err) {
        console.error("import failed:", err);
        alert(`Import failed: ${err}`);
      }
    });
  }

  function bindPagerButtons() {
    host.querySelectorAll<HTMLButtonElement>("[data-pg]").forEach(btn => {
      btn.addEventListener("click", () => {
        const dir = btn.dataset.pg;
        const total = totalPages();
        if (dir === "prev" && currentPage > 0) { currentPage--; paint(); }
        if (dir === "next" && currentPage < total - 1) { currentPage++; paint(); }
      });
    });
  }

  // Re-fetch packs from backend (used after delete/import) and re-paint in place.
  async function reload(opts?: { jumpToLastPage?: boolean }) {
    const fresh = await listPacks();
    packs.length = 0;
    packs.push(...fresh);
    if (opts?.jumpToLastPage) currentPage = Math.max(0, totalPages() - 1);
    if (currentPage >= totalPages()) currentPage = Math.max(0, totalPages() - 1);
    paint();
  }

  function onKey(e: KeyboardEvent) {
    // Let router's Cmd/Ctrl+arrow tab-switch fall through.
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    // Don't hijack typing in form fields (e.g. modal input).
    const tgt = e.target as HTMLElement | null;
    const tag = tgt?.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
    // Ignore arrows when a modal is open (its own handlers should win).
    if (document.querySelector(".modal-overlay")) return;

    const total = totalPages();
    if (e.key === "ArrowLeft" && currentPage > 0) {
      currentPage--; paint();
    } else if (e.key === "ArrowRight" && currentPage < total - 1) {
      currentPage++; paint();
    }
  }
  document.addEventListener("keydown", onKey);

  paint();

  return () => {
    document.removeEventListener("keydown", onKey);
  };
}

function escapeHtml(s: string): string {
  return s.replace(/[&<>"']/g, c =>
    ({ "&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;" }[c]!)
  );
}
