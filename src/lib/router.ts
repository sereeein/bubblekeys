import { t as tt } from "../i18n";

export type TabId = "home" | "packs" | "settings" | "about";
export type ViewCleanup = () => void;
export type ViewFn = (host: HTMLElement) => Promise<ViewCleanup | void>;

const tabs: TabId[] = ["home", "packs", "settings", "about"];

export interface RouterApi {
  activate(tab: TabId): Promise<void>;
  next(): Promise<void>;
  prev(): Promise<void>;
  refresh(): Promise<void>;
}

let _refresh: () => Promise<void> = async () => {};
export function setRouterRefresh(fn: () => Promise<void>) { _refresh = fn; }
export async function refreshActiveTab() { await _refresh(); }

export function createRouter(
  hostScreen: HTMLElement,
  hostTabs: HTMLElement,
  views: Record<TabId, ViewFn>,
): RouterApi {
  let active: TabId = "home";
  let currentCleanup: ViewCleanup | null = null;

  function paintTabs() {
    hostTabs.innerHTML = tabs.map(t => `
      <button class="tab ${t === active ? 'active' : ''}" data-tab="${t}">${tt(`tab.${t}`)}</button>
    `).join("");
    hostTabs.querySelectorAll<HTMLButtonElement>(".tab").forEach(b => {
      b.addEventListener("click", () => activate(b.dataset.tab as TabId));
    });
  }

  async function activate(tab: TabId) {
    if (currentCleanup) { currentCleanup(); currentCleanup = null; }
    active = tab;
    paintTabs();
    hostScreen.innerHTML = "";
    const result = await views[tab](hostScreen);
    if (typeof result === "function") currentCleanup = result;
  }

  paintTabs();
  void (async () => {
    const result = await views[active](hostScreen);
    if (typeof result === "function") currentCleanup = result;
  })();

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

  async function refresh() {
    await activate(active);
  }

  return { activate, next, prev, refresh };
}
