import { completeOnboarding, openAccessibilitySettings, checkAccessibility } from "../lib/ipc";
import { t } from "../i18n";

type Step = "welcome" | "why" | "grant" | "try";

export function renderFirstRun(host: HTMLElement, onDone: () => void) {
  let step: Step = "welcome";
  let pollHandle: number | undefined;
  paint();

  function paint() {
    if (step === "welcome") host.innerHTML = welcome();
    else if (step === "why") host.innerHTML = why();
    else if (step === "grant") host.innerHTML = grant();
    else host.innerHTML = tryIt();
    bind();
  }

  function bind() {
    host.querySelectorAll<HTMLButtonElement>("[data-go]").forEach(b => {
      b.addEventListener("click", async () => {
        const next = b.dataset.go as Step | "done" | "open-system";
        if (next === "open-system") {
          await openAccessibilitySettings();
          step = "grant"; paint();
        } else if (next === "done") {
          await completeOnboarding();
          onDone();
        } else { step = next; paint(); }
      });
    });

    if (pollHandle) {
      window.clearInterval(pollHandle);
      pollHandle = undefined;
    }

    if (step === "grant") {
      pollHandle = window.setInterval(async () => {
        const ok = await checkAccessibility();
        if (ok) {
          window.clearInterval(pollHandle);
          pollHandle = undefined;
          step = "try";
          paint();
        }
      }, 1000);
    }
  }
}

const welcome = () => `
  <div class="onboard">
    <h1>🫧 BubbleKeys</h1>
    <p class="subtitle">${t("onboarding.welcome.title")}</p>
    <button class="pixel-btn" data-go="why">${t("onboarding.welcome.cta")}</button>
  </div>`;

const why = () => `
  <div class="onboard">
    <h2>${t("onboarding.why.title")}</h2>
    <p>${t("onboarding.why.body")}</p>
    <button class="pixel-btn" data-go="open-system">${t("onboarding.why.cta")}</button>
  </div>`;

const grant = () => `
  <div class="onboard">
    <h2>${t("onboarding.grant.title")}</h2>
    <p>${t("onboarding.grant.body")}</p>
    <p class="subtitle"><span id="accessibility-status">${t("onboarding.grant.waiting")}</span></p>
  </div>`;

const tryIt = () => `
  <div class="onboard">
    <h2>${t("onboarding.try.title")}</h2>
    <p>${t("onboarding.try.body")}</p>
    <button class="pixel-btn" data-go="done">${t("onboarding.try.cta")}</button>
  </div>`;
