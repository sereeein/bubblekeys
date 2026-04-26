import { renderHome } from "./views/home";
import { renderPacks } from "./views/packs";
import { createRouter } from "./lib/router";

const tabs = document.getElementById("tabs")!;
const screen = document.getElementById("screen")!;

const stub = (label: string) => async (h: HTMLElement) => { h.innerHTML = `<p style="text-align:center; color:var(--c-subink)">${label} — coming soon</p>`; };

createRouter(screen, tabs, {
  home: renderHome,
  packs: renderPacks,
  settings: stub("SETTINGS"),
  about: stub("ABOUT"),
});
