import { resetOnboarding, openUrl, getAppVersion } from "../lib/ipc";
import { t } from "../i18n";

export async function renderAbout(host: HTMLElement) {
  const version = await getAppVersion();
  host.innerHTML = `
    <div class="about">
      <div class="logo">🫧</div>
      <h2>BUBBLEKEYS</h2>
      <p class="subtitle">${t("about.version", { version })} · MIT</p>
      <button class="pixel-btn" id="github">${t("about.github")}</button>
      <button class="pixel-btn" id="check">${t("about.check_updates")}</button>
      <button class="pixel-btn off" id="reset">${t("about.reset_onboarding")}</button>
    </div>`;

  host.querySelector<HTMLButtonElement>("#github")!.addEventListener("click", () =>
    openUrl("https://github.com/bubblekeys/bubblekeys"));
  host.querySelector<HTMLButtonElement>("#check")!.addEventListener("click", () =>
    openUrl("https://github.com/bubblekeys/bubblekeys/releases"));
  host.querySelector<HTMLButtonElement>("#reset")!.addEventListener("click", async () => {
    await resetOnboarding();
    location.reload();
  });
}
