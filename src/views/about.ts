import { resetOnboarding, openUrl, getAppVersion, factoryReset } from "../lib/ipc";
import { pixelConfirm } from "../lib/modal";
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
      <div class="about-actions">
        <button class="pixel-btn off" id="reset">${t("about.reset_onboarding")}</button>
        <button class="pixel-btn off" id="factory">${t("about.factory_reset")}</button>
      </div>
    </div>`;

  host.querySelector<HTMLButtonElement>("#github")!.addEventListener("click", () =>
    openUrl("https://github.com/sereeein/bubblekeys"));
  host.querySelector<HTMLButtonElement>("#check")!.addEventListener("click", () =>
    openUrl("https://github.com/sereeein/bubblekeys/releases"));
  host.querySelector<HTMLButtonElement>("#reset")!.addEventListener("click", async () => {
    await resetOnboarding();
    location.reload();
  });
  host.querySelector<HTMLButtonElement>("#factory")!.addEventListener("click", async () => {
    const ok = await pixelConfirm({
      title: t("about.factory_reset_confirm"),
      okLabel: t("common.ok"),
      cancelLabel: t("common.cancel"),
    });
    if (!ok) return;
    await factoryReset();
  });
}
