import { renderHome } from "./views/home";
import { renderPacks } from "./views/packs";
import { renderSettings } from "./views/settings";
import { createRouter, setRouterRefresh } from "./lib/router";
import { renderFirstRun } from "./views/first-run";
import { getSettings } from "./lib/ipc";
import { detectLocale, setLocale, type Locale } from "./i18n";

const tabs   = document.getElementById("tabs")!;
const screen = document.getElementById("screen")!;

const stub = (label: string) => async (h: HTMLElement) => { h.innerHTML = `<p style="text-align:center; color:var(--c-subink)">${label} — coming soon</p>`; };

function bootMain() {
  const router = createRouter(screen, tabs, {
    home: renderHome,
    packs: renderPacks,
    settings: renderSettings,
    about: stub("ABOUT"),
  });
  setRouterRefresh(() => router.refresh());
}

(async () => {
  const s = await getSettings();
  setLocale((s.language === "auto" ? detectLocale() : s.language) as Locale);
  if (!s.onboarding_completed) {
    tabs.style.display = "none";
    renderFirstRun(screen, () => {
      tabs.style.display = "";
      bootMain();
    });
  } else {
    bootMain();
  }
})();
