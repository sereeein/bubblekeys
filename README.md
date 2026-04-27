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

Prerequisites: macOS 14+, Node.js 20+, Rust 1.85+ (`rustup install 1.85`), Xcode Command Line Tools.

```bash
npm install
npm run tauri dev
```

## License

MIT — see [LICENSE](./LICENSE).
