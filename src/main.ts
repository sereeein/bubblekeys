const app = document.querySelector<HTMLDivElement>("#app");
if (app) {
  app.innerHTML = `
    <h1>🫧 BUBBLEKEYS</h1>
    <p>Phase 2 tracer bullet</p>
    <p>Type in any other app — every keydown should play a click. Toggle with <kbd>Cmd-Q</kbd> to quit.</p>
  `;
}
