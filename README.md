# 🫧 BubbleKeys

Open-source macOS typing sound effect app with a pixel-game UI.

> Status: under development. See [design doc](docs/superpowers/specs/2026-04-26-bubblekeys-design.md).

## Install

### Homebrew (recommended, once published)

```bash
brew install --cask bubblekeys
```

> The Homebrew cask isn't published yet. Use the direct download below in the meantime.

### Direct download

Grab the latest `.dmg` from [Releases](https://github.com/sereeein/bubblekeys/releases).

If macOS reports *"BubbleKeys can't be opened because it is from an unidentified developer"*, the build was unsigned — control-click the app → **Open**, then click **Open** in the dialog. Or, from Terminal:

```bash
xattr -dr com.apple.quarantine /Applications/BubbleKeys.app
```

### First run

BubbleKeys needs **Accessibility permission** to listen for keystrokes. The onboarding flow walks you through it. Nothing is recorded, logged, or transmitted — see the [design doc §7](docs/superpowers/specs/2026-04-26-bubblekeys-design.md) for the privacy model.

## Build from source

Prerequisites: macOS 14+, Node.js 20+, Rust stable (`rustup default stable`), Xcode Command Line Tools.

```bash
npm install
npm run tauri dev
```

## Sound packs

BubbleKeys ships with 4 default packs (cherry-blue, cherry-red, cherry-brown, bubbles). Import additional packs via **PACKS → + IMPORT MECHVIBES** — the importer accepts any [Mechvibes](https://mechvibes.com/sound-packs/)-format `.zip` (or unpacked directory). You can rename a pack on import and delete imported packs anytime; the bundled defaults are protected.

## Changelog

### v0.2.0

- **Pack management**: imported packs can be deleted from the list (× button). Bundled defaults are protected.
- **Multi-import**: importing the same pack twice no longer overwrites — duplicates are auto-suffixed.
- **Custom names on import**: name the pack however you like before it lands in the list.
- **Pack list pagination**: lists with more than 8 packs paginate with ◀/▶ buttons or plain ←/→ keys.
- **Settings layout**: night-silent start/end merged into a single row; layout tightened to fit 480 px window without scrolling.

### v0.1.x

Initial release line. Audio + drag + close-button + ad-hoc signing fixes; multi-language UI; Accessibility auto-detect.

## License

MIT — see [LICENSE](./LICENSE).
