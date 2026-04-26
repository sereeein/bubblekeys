# Manual smoke test (per phase)

Run before declaring a phase done.

## Phase 2 — first sound

1. `npm run tauri dev`
2. macOS prompts for Accessibility → grant in System Settings.
3. Quit (Cmd-Q) and rerun `npm run tauri dev`.
4. Open any other app (Notes, browser).
5. Type 5 keys — every keydown should produce a click sound.
6. Plug in/unplug headphones — sound should follow default output (no crash).

If any step fails, fix before continuing to Phase 3.
