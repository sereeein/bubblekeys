export type TabId = "home" | "packs" | "settings" | "about";

const tabs: TabId[] = ["home", "packs", "settings", "about"];

export interface RouterApi {
  activate(tab: TabId): Promise<void>;
  next(): Promise<void>;
  prev(): Promise<void>;
}

export function createRouter(
  hostScreen: HTMLElement,
  hostTabs: HTMLElement,
  views: Record<TabId, (host: HTMLElement) => Promise<void>>,
): RouterApi {
  let active: TabId = "home";

  function paintTabs() {
    hostTabs.innerHTML = tabs.map(t => `
      <button class="tab ${t === active ? 'active' : ''}" data-tab="${t}">${t.toUpperCase()}</button>
    `).join("");
    hostTabs.querySelectorAll<HTMLButtonElement>(".tab").forEach(b => {
      b.addEventListener("click", () => activate(b.dataset.tab as TabId));
    });
  }

  async function activate(tab: TabId) {
    active = tab;
    paintTabs();
    hostScreen.innerHTML = "";
    await views[tab](hostScreen);
  }

  paintTabs();
  views[active](hostScreen);

  document.addEventListener("keydown", (e) => {
    if (e.key === "ArrowRight" && (e.metaKey || e.ctrlKey)) next();
    if (e.key === "ArrowLeft"  && (e.metaKey || e.ctrlKey)) prev();
  });

  async function next() {
    const i = tabs.indexOf(active);
    await activate(tabs[(i + 1) % tabs.length]);
  }
  async function prev() {
    const i = tabs.indexOf(active);
    await activate(tabs[(i - 1 + tabs.length) % tabs.length]);
  }

  return { activate, next, prev };
}
