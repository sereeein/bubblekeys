import { getState, listPacks, setMuted, setVolume } from "../lib/ipc";

export async function renderHome(host: HTMLElement) {
  const state = await getState();
  const packs = await listPacks();
  const active = packs.find(p => p.id === state.active_pack);

  host.innerHTML = `
    <div class="home">
      <h1 class="pack-name">${active?.name ?? "—"}</h1>
      <div class="art" aria-hidden="true">⌨</div>
      <div class="vol-row">
        <span>VOL</span>
        <div class="pixel-bar"><div class="fill" id="vol-fill"></div></div>
        <span id="vol-val">${Math.round(state.volume * 100)}</span>
      </div>
      <button class="pixel-btn" id="toggle" aria-pressed="${!state.muted}">
        ${state.muted ? "▶ OFF" : "▶ ON"}
      </button>
    </div>`;

  const fill = host.querySelector<HTMLDivElement>("#vol-fill")!;
  const val  = host.querySelector<HTMLSpanElement>("#vol-val")!;
  const btn  = host.querySelector<HTMLButtonElement>("#toggle")!;
  const led  = document.getElementById("led")!;

  const renderVol = (v: number) => {
    fill.style.width = `${Math.round(v * 100)}%`;
    val.textContent = String(Math.round(v * 100));
  };
  renderVol(state.volume);

  btn.addEventListener("click", async () => {
    const nowMuted = btn.getAttribute("aria-pressed") === "true";
    // aria-pressed=true means "ON button is active" = not muted; clicking → mute
    await setMuted(nowMuted);
    btn.setAttribute("aria-pressed", String(!nowMuted));
    btn.textContent = nowMuted ? "▶ OFF" : "▶ ON";
    led.classList.toggle("off", nowMuted);
  });

  // ←/→ adjust volume by 5%
  document.addEventListener("keydown", async (e) => {
    if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
    const cur = (await getState()).volume;
    const next = Math.max(0, Math.min(1, cur + (e.key === "ArrowRight" ? 0.05 : -0.05)));
    await setVolume(next);
    renderVol(next);
  });
}
