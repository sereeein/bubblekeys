import { getSettings, listPacks, setActivePack, setMuted, setVolume, showMain, quitApp } from "./lib/ipc";

const root = document.getElementById("tray-root")!;
async function paint() {
  const [s, packs] = await Promise.all([getSettings(), listPacks()]);
  const recent = packs.slice(0, 4);

  root.innerHTML = `
    <div class="tray">
      <header>
        <span>🫧 BubbleKeys</span>
        <button class="pixel-btn ${s.muted ? 'off' : ''}" id="t-toggle">
          ${s.muted ? "OFF" : "ON"}
        </button>
      </header>
      <p class="tray-label">CURRENT PACK</p>
      <ul class="tray-pack-list">
        ${recent.map(p => `
          <li class="${p.id === s.active_pack ? 'active' : ''}" data-id="${p.id}">
            <span>${p.id === s.active_pack ? "✓" : ""}</span>
            <span>${p.name}</span>
          </li>`).join("")}
      </ul>
      <div class="tray-vol">
        <span>VOL</span>
        <input type="range" min="0" max="100" value="${Math.round(s.volume*100)}" id="t-vol">
        <span id="t-vol-val">${Math.round(s.volume*100)}</span>
      </div>
      <footer>
        <a id="t-open">⚙ Open BubbleKeys</a>
        <a id="t-quit">⏻ Quit</a>
      </footer>
    </div>
  `;
  bind();
}

function bind() {
  document.getElementById("t-toggle")!.addEventListener("click", async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    const muted = btn.classList.toggle("off");
    btn.textContent = muted ? "OFF" : "ON";
    await setMuted(muted);
  });
  document.querySelectorAll<HTMLLIElement>(".tray-pack-list li").forEach(li => {
    li.addEventListener("click", async () => {
      await setActivePack(li.dataset.id!);
      await paint();
    });
  });
  const vol = document.getElementById("t-vol") as HTMLInputElement;
  vol.addEventListener("input", () => {
    document.getElementById("t-vol-val")!.textContent = vol.value;
    setVolume(Number(vol.value)/100);
  });
  document.getElementById("t-open")!.addEventListener("click", () => {
    showMain();
  });
  document.getElementById("t-quit")!.addEventListener("click", () => {
    quitApp();
  });
}
paint();
