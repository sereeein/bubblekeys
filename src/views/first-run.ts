import { completeOnboarding, openAccessibilitySettings } from "../lib/ipc";

type Step = "welcome" | "why" | "grant" | "try";

export function renderFirstRun(host: HTMLElement, onDone: () => void) {
  let step: Step = "welcome";
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

    if (step === "try") {
      // Live keystroke detector via tauri event from backend (added in 8.3).
    }
  }
}

const welcome = () => `
  <div class="onboard">
    <h1>🫧 BubbleKeys</h1>
    <p class="subtitle">Welcome / 欢迎</p>
    <button class="pixel-btn" data-go="why">▶ START</button>
  </div>`;

const why = () => `
  <div class="onboard">
    <h2>NEEDS ACCESSIBILITY</h2>
    <p>BubbleKeys listens to your keyboard so it can play a sound on every keypress. Nothing is recorded, logged, or sent.</p>
    <button class="pixel-btn" data-go="open-system">⚙ OPEN SYSTEM SETTINGS</button>
  </div>`;

const grant = () => `
  <div class="onboard">
    <h2>↑ ENABLE BUBBLEKEYS</h2>
    <p>System Settings → Privacy & Security → Accessibility → toggle BubbleKeys on. We'll auto-advance once granted.</p>
    <button class="pixel-btn" data-go="try">SKIP DETECT (NEXT)</button>
  </div>`;

const tryIt = () => `
  <div class="onboard">
    <h2>✓ READY</h2>
    <p>Press any key to test. Then click DONE.</p>
    <button class="pixel-btn" data-go="done">DONE</button>
  </div>`;
