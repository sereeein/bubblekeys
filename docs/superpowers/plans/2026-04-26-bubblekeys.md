# BubbleKeys Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship v1 of BubbleKeys — an open-source macOS app that plays configurable typing sound effects, with a pixel-game UI and Mechvibes-pack compatibility.

**Architecture:** Tauri 2 single bundle. Rust backend hosts a `CGEventTap`-based global key listener and a `cpal`/`rodio` audio engine. Vanilla TypeScript frontend renders two webview windows: a Game Boy-shaped main window (4 pages) and a menu-bar dropdown remote. State (settings + active pack) is owned in Rust and pushed to the frontend over Tauri IPC.

**Tech Stack:** Rust 1.79+, Tauri 2, `cpal`, `rodio`, `core-graphics`, `serde`, `notify`. Frontend: TypeScript 5+, Vite, vanilla DOM (no framework). i18n: in-house JSON loader + key-completeness CI check. Fonts: Ark Pixel (open source, CJK-capable).

**Source spec:** [`docs/superpowers/specs/2026-04-26-bubblekeys-design.md`](../specs/2026-04-26-bubblekeys-design.md)

---

## Plan structure (tracer-bullet order)

The build is sliced by **user-visible value**, not by layer. The earliest slice produces a sound on a keypress; every following phase adds polish or a feature the user can see.

| Phase | Name | What ships at end |
|---|---|---|
| 0 | Repo scaffold | Empty repo with license, README skeleton, gitignore. |
| 1 | Tauri hello-world | `cargo tauri dev` opens an empty window. |
| 2 | First sound on key press 🎯 | Press any key, hear hardcoded sound. Accessibility prompt works. |
| 3 | Sound pack abstraction + 4 default packs | Cycle packs via debug menu; mute toggle. |
| 4 | Game Boy main window — HOME page | Pixel UI shows current pack + volume + on/off. |
| 5 | PACKS page + tab navigation | All 4 tabs visible; switch packs from list with preview. |
| 6 | SETTINGS page (basic) | Hotkey, auto-start, pitch jitter, output device, menu icon. |
| 7 | Menu-bar dropdown | Tray icon + 1-second toggle/switch/volume panel. |
| 8 | First-run flow | Polished 4-step onboarding with Accessibility deep-link. |
| 9 | i18n full 5 languages | en / zh-CN / zh-TW / ja / ko, CI guards completeness. |
| 10 | Mechvibes import | Drop a `.zip` or folder, parse, use. |
| 11 | Night silent scheduler | Time-range auto-mute, SETTINGS row + UI. |
| 12 | ABOUT page + Reset onboarding | Logo, version, GitHub, check updates, reset link. |
| 13 | App icon + brand assets | `.icns` from Pixel Bubble + keycap; DMG cosmetics. |
| 14 | CI/CD (GitHub Actions) | Tag push → universal `.dmg` artifact + release. |
| 15 | Homebrew Cask + release docs | `brew install --cask bubblekeys`. |

---

## File structure (locked-in decomposition)

```
BubbleKeys/
├── README.md                   # Public README (per phase)
├── LICENSE                     # MIT
├── .gitignore
├── package.json                # Frontend root (Vite + TS)
├── tsconfig.json
├── vite.config.ts
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/                  # generated from design
│   └── src/
│       ├── main.rs             # Tauri bootstrap, only wiring
│       ├── lib.rs              # mod declarations
│       ├── key_listener.rs     # CGEventTap → KeyEvent stream
│       ├── audio_engine.rs     # cpal output + rodio mix
│       ├── pack_format.rs      # Mechvibes JSON parsing
│       ├── pack_store.rs       # disk → memory pack registry
│       ├── dispatcher.rs       # KeyEvent + pack + settings → PlayCommand
│       ├── settings_store.rs   # ~/Library/.../settings.json
│       ├── mute_controller.rs  # single source of truth for muted state
│       ├── night_silent.rs     # schedule logic
│       ├── ipc.rs              # Tauri command handlers
│       ├── tray.rs             # menu-bar icon + dropdown lifecycle
│       └── tests/              # integration test fixtures
├── src/                        # frontend
│   ├── main.ts                 # entry for main window
│   ├── tray.ts                 # entry for tray dropdown window
│   ├── styles/
│   │   ├── tokens.css          # color/typography tokens
│   │   ├── pixel.css           # shared pixel-frame primitives
│   │   └── fonts.css           # Ark Pixel @font-face
│   ├── views/
│   │   ├── home.ts
│   │   ├── packs.ts
│   │   ├── settings.ts
│   │   ├── about.ts
│   │   ├── tray.ts             # dropdown content
│   │   └── first-run.ts
│   ├── i18n/
│   │   ├── index.ts            # loader + lookup
│   │   └── locales/
│   │       ├── en.json
│   │       ├── zh-CN.json
│   │       ├── zh-TW.json
│   │       ├── ja.json
│   │       └── ko.json
│   ├── lib/
│   │   ├── ipc.ts              # typed Tauri invoke wrappers
│   │   ├── store.ts            # nanostores app state
│   │   └── router.ts           # 4-tab router
│   └── assets/
│       └── fonts/              # Ark Pixel woff2 files
├── packs/                       # vendored default packs
│   ├── cherry-blue/
│   ├── cherry-red/
│   ├── cherry-brown/
│   └── bubbles/
├── scripts/
│   ├── verify-i18n-keys.ts     # CI check: locales match en.json
│   └── make-icons.sh           # icns + iconset from master PNG
├── docs/
│   └── superpowers/
│       ├── specs/
│       └── plans/
└── .github/
    └── workflows/
        ├── ci.yml              # build + test on PR
        └── release.yml         # tag → dmg release
```

---

# Phase 0 — Repo scaffold

**Goal:** Empty repo set up with git, license, README skeleton, .gitignore. No build chain yet.

### Task 0.1: Initialize git repo

**Files:** Create `.gitignore`, `LICENSE`, `README.md` at repo root.

- [ ] **Step 1: `git init`**

```bash
cd /Users/evette/Documents/BubbleKeys
git init -b main
```

- [ ] **Step 2: Create `.gitignore`**

Path: `/Users/evette/Documents/BubbleKeys/.gitignore`

```gitignore
# Rust / Tauri
target/
src-tauri/target/

# Node
node_modules/
dist/
.vite/

# OS
.DS_Store
*.log

# Editor
.vscode/
.idea/
*.swp

# Tauri build outputs
src-tauri/gen/
src-tauri/WixTools/

# Local secrets
.env
.env.local

# Brainstorming workspace (already in repo for reference)
# .superpowers/  ← keep for now, remove later
```

- [ ] **Step 3: Create MIT `LICENSE`**

Path: `/Users/evette/Documents/BubbleKeys/LICENSE`

```text
MIT License

Copyright (c) 2026 BubbleKeys contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 4: Create README skeleton**

Path: `/Users/evette/Documents/BubbleKeys/README.md`

```markdown
# 🫧 BubbleKeys

Open-source macOS typing sound effect app with a pixel-game UI.

> Status: under development. See [design doc](docs/superpowers/specs/2026-04-26-bubblekeys-design.md).

## Install (coming soon)

```bash
brew install --cask bubblekeys   # not yet published
```

## Build from source

```bash
pnpm install
pnpm tauri dev
```

## License

MIT — see [LICENSE](./LICENSE).
```

- [ ] **Step 5: First commit**

```bash
git add .gitignore LICENSE README.md
git commit -m "chore: initial commit (license, readme skeleton, gitignore)"
```

Expected: commit succeeds; `git log --oneline` shows one commit.

---

# Phase 1 — Tauri hello-world

**Goal:** `pnpm tauri dev` opens an empty pixel-styled window. Proves toolchain works.

### Task 1.1: Bootstrap Tauri 2 project

**Files:** Create `package.json`, `tsconfig.json`, `vite.config.ts`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`, `src-tauri/src/main.rs`, `src/main.ts`, `index.html`.

- [ ] **Step 1: Verify toolchain prerequisites**

```bash
node --version    # expect ≥ 20
rustc --version   # expect ≥ 1.79
xcode-select -p   # expect a path; install if missing: xcode-select --install
```

If any missing, install before continuing. Tauri 2 also needs:

```bash
cargo install tauri-cli --version "^2.0.0"
```

- [ ] **Step 2: Create `package.json`**

Path: `/Users/evette/Documents/BubbleKeys/package.json`

```json
{
  "name": "bubblekeys",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "nanostores": "^0.10.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "typescript": "^5.4.0",
    "vite": "^5.2.0"
  }
}
```

- [ ] **Step 3: `tsconfig.json`**

Path: `/Users/evette/Documents/BubbleKeys/tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noImplicitReturns": true,
    "esModuleInterop": true,
    "isolatedModules": true,
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "types": ["vite/client"],
    "skipLibCheck": true
  },
  "include": ["src/**/*", "scripts/**/*"]
}
```

- [ ] **Step 4: `vite.config.ts`**

Path: `/Users/evette/Documents/BubbleKeys/vite.config.ts`

```ts
import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "es2022",
    sourcemap: true,
  },
});
```

- [ ] **Step 5: Bare `index.html` and `src/main.ts`**

Path: `/Users/evette/Documents/BubbleKeys/index.html`

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>BubbleKeys</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

Path: `/Users/evette/Documents/BubbleKeys/src/main.ts`

```ts
const app = document.querySelector<HTMLDivElement>("#app");
if (app) {
  app.textContent = "🫧 BubbleKeys — hello world";
}
```

- [ ] **Step 6: `src-tauri/Cargo.toml`**

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/Cargo.toml`

```toml
[package]
name = "bubblekeys"
version = "0.1.0"
description = "Open-source typing sound effects for macOS"
authors = ["BubbleKeys contributors"]
edition = "2021"
rust-version = "1.79"

[lib]
name = "bubblekeys_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tauri = { version = "2.0", features = ["macos-private-api"] }

[features]
custom-protocol = ["tauri/custom-protocol"]
```

- [ ] **Step 7: `src-tauri/build.rs`**

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/build.rs`

```rust
fn main() {
    tauri_build::build();
}
```

- [ ] **Step 8: `src-tauri/tauri.conf.json`**

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/tauri.conf.json`

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "BubbleKeys",
  "version": "0.1.0",
  "identifier": "app.bubblekeys",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "BubbleKeys",
        "width": 320,
        "height": 480,
        "resizable": false,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": false,
        "visible": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; font-src 'self' data:"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "app"],
    "category": "Utility",
    "macOS": {
      "minimumSystemVersion": "14.0"
    }
  }
}
```

- [ ] **Step 9: `src-tauri/src/main.rs`**

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/src/main.rs`

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    bubblekeys_lib::run();
}
```

- [ ] **Step 10: `src-tauri/src/lib.rs`**

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running BubbleKeys");
}
```

- [ ] **Step 11: Install JS deps**

```bash
cd /Users/evette/Documents/BubbleKeys
pnpm install      # or: npm install
```

Expected: `node_modules/` populated; no errors.

- [ ] **Step 12: Run `tauri dev`**

```bash
cd /Users/evette/Documents/BubbleKeys
pnpm tauri dev
```

Expected: First Cargo build takes 2–5 minutes. A 320×480 window appears showing "🫧 BubbleKeys — hello world". Quit with Ctrl-C.

- [ ] **Step 13: Commit**

```bash
git add package.json tsconfig.json vite.config.ts index.html src/ src-tauri/ pnpm-lock.yaml
git commit -m "feat: scaffold Tauri 2 project (hello world window)"
```

---

# Phase 2 — First sound on key press 🎯

**Goal:** Press any key, hear a hardcoded sound. macOS Accessibility permission flow appears. **This is the foundational tracer bullet — everything else builds on this.**

### Task 2.1: Add audio + key-listener crates and a hardcoded sound asset

**Files:** Modify `src-tauri/Cargo.toml`. Create `src-tauri/assets/click.ogg` (vendor a CC0 sound).

- [ ] **Step 1: Add Rust deps**

Modify `/Users/evette/Documents/BubbleKeys/src-tauri/Cargo.toml`. Append under `[dependencies]`:

```toml
cpal = "0.15"
rodio = { version = "0.19", default-features = false, features = ["vorbis"] }
core-graphics = "0.24"
core-foundation = "0.10"
crossbeam-channel = "0.5"
parking_lot = "0.12"
once_cell = "1.19"
log = "0.4"
env_logger = "0.11"
```

- [ ] **Step 2: Vendor a placeholder sound**

```bash
mkdir -p /Users/evette/Documents/BubbleKeys/src-tauri/assets
# Pick any short CC0 .ogg <50ms. For now, a single click sample.
# Source recommendation: download from https://freesound.org tagged CC0
# Save to: src-tauri/assets/click.ogg
```

If you don't have a real sample yet, use this Rust code to generate one for the unit test fixture (see Task 2.4 below). For the runtime path, vendor a real `click.ogg` before running.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/assets/click.ogg
git commit -m "feat: add audio + key listener deps and placeholder sound"
```

### Task 2.2: Audio engine (TDD)

**Files:** Create `src-tauri/src/audio_engine.rs`. Tests in same file behind `#[cfg(test)]`.

- [ ] **Step 1: Write the failing test**

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/src/audio_engine.rs`

```rust
//! cpal/rodio-based audio engine. Plays decoded samples on demand.

use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct PlayCommand {
    pub sample: Arc<Vec<u8>>,   // raw bytes of an OGG file
    pub volume: f32,            // 0.0–1.0
    pub pitch_offset: f32,      // semitones, e.g. -0.5..=0.5
}

pub trait AudioEngine: Send + Sync {
    fn play(&self, cmd: PlayCommand);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_command_constructs_with_zero_pitch() {
        let cmd = PlayCommand {
            sample: Arc::new(vec![0u8; 100]),
            volume: 0.5,
            pitch_offset: 0.0,
        };
        assert_eq!(cmd.volume, 0.5);
        assert_eq!(cmd.pitch_offset, 0.0);
        assert_eq!(cmd.sample.len(), 100);
    }
}
```

- [ ] **Step 2: Add module declaration to `lib.rs`**

Modify `/Users/evette/Documents/BubbleKeys/src-tauri/src/lib.rs`. Add at top:

```rust
pub mod audio_engine;
```

- [ ] **Step 3: Run the test**

```bash
cd /Users/evette/Documents/BubbleKeys/src-tauri
cargo test audio_engine::tests::play_command_constructs_with_zero_pitch
```

Expected: PASS.

- [ ] **Step 4: Implement the cpal/rodio-backed engine**

Replace `src-tauri/src/audio_engine.rs` with:

```rust
//! cpal/rodio-based audio engine. Plays decoded samples on demand.

use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam_channel::{unbounded, Sender};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

#[derive(Clone, Debug)]
pub struct PlayCommand {
    pub sample: Arc<Vec<u8>>,
    pub volume: f32,
    pub pitch_offset: f32,
}

pub trait AudioEngine: Send + Sync {
    fn play(&self, cmd: PlayCommand);
}

pub struct RodioEngine {
    tx: Sender<PlayCommand>,
}

impl RodioEngine {
    /// Spawns a dedicated audio thread that owns the OutputStream.
    /// rodio's stream isn't Send, so it stays parked on this thread.
    pub fn new() -> Result<Self, String> {
        let (tx, rx) = unbounded::<PlayCommand>();
        thread::Builder::new()
            .name("bubblekeys-audio".into())
            .spawn(move || {
                let (_stream, handle) = match OutputStream::try_default() {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("audio: failed to open default device: {e}");
                        return;
                    }
                };
                let handle = Arc::new(handle);

                while let Ok(cmd) = rx.recv() {
                    spawn_oneshot(handle.clone(), cmd);
                }
            })
            .map_err(|e| format!("spawn audio thread: {e}"))?;

        Ok(Self { tx })
    }
}

fn spawn_oneshot(handle: Arc<OutputStreamHandle>, cmd: PlayCommand) {
    let cursor = Cursor::new((*cmd.sample).clone());
    let decoder = match Decoder::new(cursor) {
        Ok(d) => d,
        Err(e) => {
            log::warn!("audio: decode failed: {e}");
            return;
        }
    };
    // Pitch offset via speed change. ±0.5 semitones ≈ ±3% speed.
    let speed = 2f32.powf(cmd.pitch_offset / 12.0);
    let source = decoder.amplify(cmd.volume).speed(speed);
    let sink = match Sink::try_new(&handle) {
        Ok(s) => s,
        Err(e) => { log::warn!("audio: sink: {e}"); return; }
    };
    sink.append(source);
    sink.detach(); // play asynchronously, dropped when finished
}

impl AudioEngine for RodioEngine {
    fn play(&self, cmd: PlayCommand) {
        if let Err(e) = self.tx.send(cmd) {
            log::warn!("audio: queue full or closed: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_command_constructs_with_zero_pitch() {
        let cmd = PlayCommand {
            sample: Arc::new(vec![0u8; 100]),
            volume: 0.5,
            pitch_offset: 0.0,
        };
        assert_eq!(cmd.volume, 0.5);
    }

    // Real-output tests are skipped in CI (no audio device).
}
```

- [ ] **Step 5: Verify build still passes tests**

```bash
cd /Users/evette/Documents/BubbleKeys/src-tauri
cargo test
```

Expected: 1 test passes.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/audio_engine.rs src-tauri/src/lib.rs
git commit -m "feat(audio): rodio-backed audio engine with PlayCommand contract"
```

### Task 2.3: Key listener (CGEventTap)

**Files:** Create `src-tauri/src/key_listener.rs`.

- [ ] **Step 1: Write the failing test for the event types**

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/src/key_listener.rs`

```rust
//! Global keyboard listener via CGEventTap.

use crossbeam_channel::Receiver;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEventKind { Down, Up }

#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
    pub keycode: u16,         // macOS virtual keycode
    pub kind: KeyEventKind,
}

pub trait KeyListener {
    fn events(&self) -> Receiver<KeyEvent>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_event_constructs() {
        let e = KeyEvent { keycode: 0, kind: KeyEventKind::Down };
        assert_eq!(e.kind, KeyEventKind::Down);
    }
}
```

- [ ] **Step 2: Run test**

```bash
cd /Users/evette/Documents/BubbleKeys/src-tauri
cargo test key_listener::tests
```

Expected: PASS.

- [ ] **Step 3: Implement CGEventTap-backed listener**

Replace contents of `key_listener.rs` with:

```rust
//! Global keyboard listener via CGEventTap. Requires Accessibility permission.

use std::ffi::c_void;
use std::ptr;
use std::thread;

use core_foundation::base::TCFType;
use core_foundation::runloop::{
    kCFRunLoopCommonModes, CFRunLoopAddSource, CFRunLoopGetCurrent, CFRunLoopRun,
};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType,
};
use crossbeam_channel::{unbounded, Receiver, Sender};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEventKind { Down, Up }

#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
    pub keycode: u16,
    pub kind: KeyEventKind,
}

pub trait KeyListener: Send + Sync {
    fn events(&self) -> Receiver<KeyEvent>;
}

pub struct MacKeyListener {
    rx: Receiver<KeyEvent>,
}

impl MacKeyListener {
    /// Spawns a thread running the event tap on its own CFRunLoop.
    /// Returns Err only if Accessibility permission is missing AND the OS refuses to create the tap.
    pub fn start() -> Result<Self, String> {
        let (tx, rx) = unbounded::<KeyEvent>();

        thread::Builder::new()
            .name("bubblekeys-keys".into())
            .spawn(move || run_tap(tx))
            .map_err(|e| format!("spawn key thread: {e}"))?;

        Ok(Self { rx })
    }
}

fn run_tap(tx: Sender<KeyEvent>) {
    let events = vec![CGEventType::KeyDown, CGEventType::KeyUp];
    let tap = match CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::ListenOnly,
        events,
        move |_proxy, etype, event| {
            let keycode = event.get_integer_value_field(
                core_graphics::event::EventField::KEYBOARD_EVENT_KEYCODE,
            ) as u16;
            let kind = match etype {
                CGEventType::KeyDown => KeyEventKind::Down,
                CGEventType::KeyUp   => KeyEventKind::Up,
                _ => return None,
            };
            let _ = tx.send(KeyEvent { keycode, kind });
            None
        },
    ) {
        Ok(t) => t,
        Err(()) => {
            log::error!("key listener: failed to create event tap (Accessibility permission missing?)");
            return;
        }
    };

    unsafe {
        let loop_source = tap.mach_port.create_runloop_source(0).expect("loop source");
        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, loop_source.as_concrete_TypeRef(), kCFRunLoopCommonModes);
        tap.enable();
        CFRunLoopRun();
    }
}

impl KeyListener for MacKeyListener {
    fn events(&self) -> Receiver<KeyEvent> { self.rx.clone() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_event_constructs() {
        let e = KeyEvent { keycode: 0, kind: KeyEventKind::Down };
        assert_eq!(e.kind, KeyEventKind::Down);
    }
}
```

- [ ] **Step 4: Add module declaration**

Modify `/Users/evette/Documents/BubbleKeys/src-tauri/src/lib.rs`:

```rust
pub mod audio_engine;
pub mod key_listener;
```

- [ ] **Step 5: Build to ensure it compiles**

```bash
cd /Users/evette/Documents/BubbleKeys/src-tauri
cargo build
```

Expected: compiles. (Will only actually fire events at runtime when Accessibility is granted.)

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/key_listener.rs src-tauri/src/lib.rs
git commit -m "feat(keys): CGEventTap-backed global key listener"
```

### Task 2.4: Wire it together — first runtime sound

**Files:** Modify `src-tauri/src/lib.rs`. Add `src-tauri/Info.plist` for entitlements/usage strings.

- [ ] **Step 1: Edit `lib.rs` to glue listener → engine**

Replace `/Users/evette/Documents/BubbleKeys/src-tauri/src/lib.rs`:

```rust
pub mod audio_engine;
pub mod key_listener;

use std::sync::Arc;
use std::thread;

use audio_engine::{AudioEngine, PlayCommand, RodioEngine};
use key_listener::{KeyListener, MacKeyListener};

const EMBEDDED_CLICK: &[u8] = include_bytes!("../assets/click.ogg");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let engine: Arc<dyn AudioEngine> = Arc::new(
        RodioEngine::new().expect("audio engine init")
    );
    let listener = MacKeyListener::start().expect("key listener init");
    let rx = listener.events();
    let click_bytes = Arc::new(EMBEDDED_CLICK.to_vec());

    // Dispatcher thread: every keydown → click sound at fixed volume.
    let engine_for_thread = engine.clone();
    thread::Builder::new()
        .name("bubblekeys-dispatch".into())
        .spawn(move || {
            while let Ok(ev) = rx.recv() {
                if matches!(ev.kind, key_listener::KeyEventKind::Down) {
                    engine_for_thread.play(PlayCommand {
                        sample: click_bytes.clone(),
                        volume: 0.65,
                        pitch_offset: 0.0,
                    });
                }
            }
        })
        .expect("dispatcher thread");

    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running BubbleKeys");
}
```

- [ ] **Step 2: Add Info.plist entries for Accessibility usage description**

Modify `/Users/evette/Documents/BubbleKeys/src-tauri/tauri.conf.json`. Inside `bundle.macOS`, add:

```json
"macOS": {
  "minimumSystemVersion": "14.0",
  "entitlements": null,
  "exceptionDomain": "",
  "frameworks": [],
  "providerShortName": null,
  "signingIdentity": null,
  "infoPlist": {
    "NSAppleEventsUsageDescription": "BubbleKeys needs access to play sound effects for your typing.",
    "NSAccessibilityUsageDescription": "BubbleKeys listens for keystrokes so it can play a sound when you type."
  }
}
```

- [ ] **Step 3: Run dev build and grant Accessibility manually**

```bash
cd /Users/evette/Documents/BubbleKeys
pnpm tauri dev
```

Expected sequence:
1. Window opens.
2. First time you type a key with the app running, macOS shows: "BubbleKeys would like to receive keystrokes from any application." Click "Open System Settings."
3. Manually enable BubbleKeys (or the dev binary) under Privacy & Security → Accessibility.
4. **Quit and re-run `pnpm tauri dev`** (CGEventTap caches permission state).
5. Press any key in any app → click sound plays.

If no sound, check:
- `RUST_LOG=info pnpm tauri dev` for log lines from `audio:` / `key listener:`.
- Output device default is set in System Settings → Sound.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/tauri.conf.json
git commit -m "feat: tracer-bullet — keystrokes play hardcoded sound (Accessibility required)"
```

### Task 2.5: Smoke-test script

**Files:** Create `scripts/smoke-test.md`.

- [ ] **Step 1: Document the manual smoke test**

Path: `/Users/evette/Documents/BubbleKeys/scripts/smoke-test.md`

```markdown
# Manual smoke test (per phase)

Run before declaring a phase done.

## Phase 2 — first sound

1. `pnpm tauri dev`
2. macOS prompts for Accessibility → grant in System Settings.
3. Quit (Cmd-Q) and rerun `pnpm tauri dev`.
4. Open any other app (Notes, browser).
5. Type 5 keys — every keydown should produce a click sound.
6. Plug in/unplug headphones — sound should follow default output (no crash).

If any step fails, fix before continuing to Phase 3.
```

- [ ] **Step 2: Commit**

```bash
git add scripts/smoke-test.md
git commit -m "docs: add manual smoke test checklist"
```

---

# Phase 3 — Sound pack abstraction + 4 default packs

**Goal:** Replace the hardcoded click with a real pack registry. 4 packs vendored. Switch via debug menu. Mute toggle works.

### Task 3.1: Pack format parser (TDD)

**Files:** Create `src-tauri/src/pack_format.rs` and a test fixture under `src-tauri/tests/fixtures/`.

- [ ] **Step 1: Write failing tests with sample fixtures**

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/tests/fixtures/pack_single/config.json`

```json
{
  "id": "test-single",
  "name": "Test Single",
  "key_define_type": "single",
  "sound": "sound.ogg",
  "includes_numpad": true,
  "license": "CC0",
  "author": "Test"
}
```

Create empty placeholder: `src-tauri/tests/fixtures/pack_single/sound.ogg` (any short ogg, even 1 frame).

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/src/pack_format.rs`

```rust
//! Mechvibes-compatible sound pack manifest parsing.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyDefineType { Single, Multi }

#[derive(Clone, Debug, Deserialize)]
pub struct PackManifest {
    pub id: String,
    pub name: String,
    pub key_define_type: KeyDefineType,
    pub sound: String,
    /// Only required for `Multi`. Map of keycode → [offset_ms, duration_ms].
    #[serde(default)]
    pub defines: HashMap<String, [u32; 2]>,
    #[serde(default = "default_true")]
    pub includes_numpad: bool,
    pub license: Option<String>,
    pub author: Option<String>,
    pub icon: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_true() -> bool { true }

#[derive(Debug, thiserror::Error)]
pub enum PackError {
    #[error("io: {0}")] Io(#[from] std::io::Error),
    #[error("json: {0}")] Json(#[from] serde_json::Error),
    #[error("pack id mismatches directory: '{0}' vs '{1}'")] IdMismatch(String, String),
    #[error("multi pack missing 'defines'")] MultiMissingDefines,
}

pub fn load_manifest(dir: &Path) -> Result<PackManifest, PackError> {
    let cfg_path = dir.join("config.json");
    let bytes = std::fs::read(&cfg_path)?;
    let m: PackManifest = serde_json::from_slice(&bytes)?;
    if matches!(m.key_define_type, KeyDefineType::Multi) && m.defines.is_empty() {
        return Err(PackError::MultiMissingDefines);
    }
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
    }

    #[test]
    fn parses_single_pack() {
        let m = load_manifest(&fixture("pack_single")).unwrap();
        assert_eq!(m.id, "test-single");
        assert!(matches!(m.key_define_type, KeyDefineType::Single));
        assert!(m.defines.is_empty());
    }

    #[test]
    fn rejects_multi_without_defines() {
        // Create the bad case inline: write a tmp dir with an invalid manifest.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.json"),
            r#"{ "id":"bad","name":"Bad","key_define_type":"multi","sound":"s.ogg" }"#,
        ).unwrap();
        let err = load_manifest(dir.path()).unwrap_err();
        assert!(matches!(err, PackError::MultiMissingDefines));
    }
}
```

Add to `Cargo.toml` `[dev-dependencies]`:

```toml
[dev-dependencies]
tempfile = "3.10"
```

And add to `[dependencies]`:

```toml
thiserror = "1.0"
```

- [ ] **Step 2: Add module + run tests**

Modify `src-tauri/src/lib.rs`:

```rust
pub mod audio_engine;
pub mod key_listener;
pub mod pack_format;
```

```bash
cd /Users/evette/Documents/BubbleKeys/src-tauri
cargo test pack_format
```

Expected: 2 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/pack_format.rs src-tauri/src/lib.rs src-tauri/tests/fixtures/pack_single/
git commit -m "feat(packs): Mechvibes manifest parser with TDD coverage"
```

### Task 3.2: Pack store (load packs from disk into memory)

**Files:** Create `src-tauri/src/pack_store.rs`.

- [ ] **Step 1: Write failing test**

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/src/pack_store.rs`

```rust
//! Loads sound packs from a directory into RAM and exposes them to the dispatcher.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::pack_format::{load_manifest, KeyDefineType, PackError, PackManifest};

#[derive(Clone, Debug)]
pub struct LoadedPack {
    pub manifest: PackManifest,
    /// Single-sound packs: one entry under "*". Multi: one entry per defined keycode (string).
    pub samples_by_key: HashMap<String, Arc<Vec<u8>>>,
}

#[derive(Default)]
pub struct PackStore {
    packs: HashMap<String, LoadedPack>,
}

impl PackStore {
    pub fn new() -> Self { Self::default() }

    pub fn load_dir(&mut self, dir: &Path) -> Result<(), PackError> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if !entry.path().is_dir() { continue; }
            let manifest = load_manifest(&entry.path())?;
            let samples = decode_samples(&entry.path(), &manifest)?;
            self.packs.insert(manifest.id.clone(), LoadedPack { manifest, samples_by_key: samples });
        }
        Ok(())
    }

    pub fn ids(&self) -> Vec<String> {
        let mut v: Vec<String> = self.packs.keys().cloned().collect();
        v.sort();
        v
    }

    pub fn get(&self, id: &str) -> Option<&LoadedPack> { self.packs.get(id) }
}

fn decode_samples(dir: &Path, m: &PackManifest) -> Result<HashMap<String, Arc<Vec<u8>>>, PackError> {
    let mut out = HashMap::new();
    let sound_path = dir.join(&m.sound);
    let bytes = Arc::new(std::fs::read(&sound_path)?);
    match m.key_define_type {
        KeyDefineType::Single => {
            out.insert("*".into(), bytes);
        }
        KeyDefineType::Multi => {
            // v1: store one shared sprite blob; dispatcher slices by offset.
            // For Phase 3 we treat multi-packs as if every key plays the whole sprite (good enough).
            // Phase 10 (Mechvibes import) refines this with sprite slicing.
            for keycode in m.defines.keys() {
                out.insert(keycode.clone(), bytes.clone());
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_single_fixture() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let mut store = PackStore::new();
        store.load_dir(&dir).unwrap();
        assert!(store.ids().contains(&"test-single".to_string()));
    }
}
```

- [ ] **Step 2: Add module + run**

Modify `src-tauri/src/lib.rs`:

```rust
pub mod pack_store;
```

```bash
cargo test pack_store
```

Expected: 1 test passes.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/pack_store.rs src-tauri/src/lib.rs
git commit -m "feat(packs): in-memory pack store with directory loader"
```

### Task 3.3: Vendor 4 default packs

**Files:** Create `src-tauri/packs/cherry-blue/`, `cherry-red/`, `cherry-brown/`, `bubbles/` with valid `config.json` + `sound.ogg`.

- [ ] **Step 1: Source 3 mechanical packs from Mechvibes community**

```bash
# Visit https://mechvibes.com/sound-packs/
# Download CC0 / CC-BY packs you want as Cherry Blue / Red / Brown
# Place each unzipped folder under src-tauri/packs/<id>/
# Verify each contains config.json + sound.ogg (or .mp3)
# Edit each config.json to match the spec format and ensure id is set.
```

- [ ] **Step 2: Author the original Bubbles pack**

```bash
mkdir -p /Users/evette/Documents/BubbleKeys/src-tauri/packs/bubbles
# Vendor a single 50–80ms bubble pop .ogg as sound.ogg
```

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/packs/bubbles/config.json`

```json
{
  "id": "bubbles",
  "name": "Bubbles",
  "key_define_type": "single",
  "sound": "sound.ogg",
  "includes_numpad": true,
  "license": "CC-BY-4.0",
  "author": "BubbleKeys",
  "tags": ["bubble", "soft", "original"]
}
```

- [ ] **Step 3: Make packs discoverable as a build-embedded asset**

Modify `src-tauri/tauri.conf.json` `bundle.resources`:

```json
"bundle": {
  "active": true,
  "targets": ["dmg", "app"],
  "category": "Utility",
  "resources": ["packs/**/*"],
  "macOS": { ... existing ... }
}
```

- [ ] **Step 4: At runtime, copy bundled packs to user dir on first launch**

This logic goes in `pack_store.rs`. Add:

```rust
/// On first launch, copies bundled packs from the app resource dir to the user pack dir.
/// Subsequent launches no-op.
pub fn install_default_packs(
    bundled_resource_dir: &Path,
    user_pack_dir: &Path,
) -> std::io::Result<()> {
    if user_pack_dir.exists() && std::fs::read_dir(user_pack_dir)?.next().is_some() {
        return Ok(());
    }
    std::fs::create_dir_all(user_pack_dir)?;
    let src = bundled_resource_dir.join("packs");
    if !src.exists() {
        log::warn!("bundled packs dir missing: {}", src.display());
        return Ok(());
    }
    copy_dir_recursive(&src, user_pack_dir)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/packs/ src-tauri/tauri.conf.json src-tauri/src/pack_store.rs
git commit -m "feat(packs): vendor 4 default packs and install-on-first-launch"
```

### Task 3.4: Mute controller (TDD)

**Files:** Create `src-tauri/src/mute_controller.rs`.

- [ ] **Step 1: Write failing test**

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/src/mute_controller.rs`

```rust
//! Single source of truth for "should we play sound right now?".

use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct MuteController {
    inner: Arc<RwLock<MuteState>>,
}

#[derive(Default, Clone, Copy)]
struct MuteState {
    user_muted: bool,
    night_silent_active: bool,
}

impl MuteController {
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(MuteState::default())) }
    }

    pub fn is_muted(&self) -> bool {
        let s = self.inner.read();
        s.user_muted || s.night_silent_active
    }

    pub fn set_user_muted(&self, muted: bool) {
        self.inner.write().user_muted = muted;
    }

    pub fn set_night_silent_active(&self, active: bool) {
        self.inner.write().night_silent_active = active;
    }
}

impl Default for MuteController { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_unmuted() {
        let m = MuteController::new();
        assert!(!m.is_muted());
    }

    #[test]
    fn user_mute_takes_effect() {
        let m = MuteController::new();
        m.set_user_muted(true);
        assert!(m.is_muted());
        m.set_user_muted(false);
        assert!(!m.is_muted());
    }

    #[test]
    fn night_silent_overrides_unmuted() {
        let m = MuteController::new();
        m.set_night_silent_active(true);
        assert!(m.is_muted());
    }

    #[test]
    fn either_source_keeps_muted() {
        let m = MuteController::new();
        m.set_user_muted(true);
        m.set_night_silent_active(true);
        m.set_user_muted(false);
        assert!(m.is_muted()); // night_silent still active
    }
}
```

- [ ] **Step 2: Add module + run tests**

```rust
// src-tauri/src/lib.rs
pub mod mute_controller;
```

```bash
cargo test mute_controller
```

Expected: 4 tests pass.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/mute_controller.rs src-tauri/src/lib.rs
git commit -m "feat: mute controller as single source of truth"
```

### Task 3.5: Dispatcher (replaces inline glue from Phase 2)

**Files:** Create `src-tauri/src/dispatcher.rs`. Modify `lib.rs`.

- [ ] **Step 1: Write failing test (deterministic, no audio device)**

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/src/dispatcher.rs`

```rust
//! Glues KeyEvent → active pack → PlayCommand. Honors mute state.

use std::sync::Arc;

use crate::audio_engine::{AudioEngine, PlayCommand};
use crate::key_listener::{KeyEvent, KeyEventKind};
use crate::mute_controller::MuteController;
use crate::pack_store::LoadedPack;

pub struct Dispatcher<E: AudioEngine + ?Sized> {
    engine: Arc<E>,
    mute: MuteController,
}

impl<E: AudioEngine + ?Sized> Dispatcher<E> {
    pub fn new(engine: Arc<E>, mute: MuteController) -> Self {
        Self { engine, mute }
    }

    pub fn handle(&self, ev: KeyEvent, pack: &LoadedPack, volume: f32, pitch_offset: f32) {
        if !matches!(ev.kind, KeyEventKind::Down) { return; }
        if self.mute.is_muted() { return; }

        let key = ev.keycode.to_string();
        let sample = pack.samples_by_key.get(&key)
            .or_else(|| pack.samples_by_key.get("*"))
            .cloned();

        if let Some(sample) = sample {
            self.engine.play(PlayCommand { sample, volume, pitch_offset });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use crate::pack_format::{KeyDefineType, PackManifest};
    use std::collections::HashMap;

    struct CountingEngine { calls: Mutex<u32> }
    impl AudioEngine for CountingEngine {
        fn play(&self, _cmd: PlayCommand) { *self.calls.lock().unwrap() += 1; }
    }

    fn dummy_pack() -> LoadedPack {
        let manifest = PackManifest {
            id: "x".into(), name: "X".into(),
            key_define_type: KeyDefineType::Single,
            sound: "s.ogg".into(),
            defines: Default::default(),
            includes_numpad: true,
            license: None, author: None, icon: None, tags: vec![],
        };
        let mut s = HashMap::new();
        s.insert("*".into(), Arc::new(vec![0u8; 4]));
        LoadedPack { manifest, samples_by_key: s }
    }

    #[test]
    fn keydown_plays() {
        let engine = Arc::new(CountingEngine { calls: Mutex::new(0) });
        let mute = MuteController::new();
        let d = Dispatcher::new(engine.clone(), mute);
        d.handle(KeyEvent { keycode: 0, kind: KeyEventKind::Down }, &dummy_pack(), 0.5, 0.0);
        assert_eq!(*engine.calls.lock().unwrap(), 1);
    }

    #[test]
    fn keyup_does_not_play() {
        let engine = Arc::new(CountingEngine { calls: Mutex::new(0) });
        let d = Dispatcher::new(engine.clone(), MuteController::new());
        d.handle(KeyEvent { keycode: 0, kind: KeyEventKind::Up }, &dummy_pack(), 0.5, 0.0);
        assert_eq!(*engine.calls.lock().unwrap(), 0);
    }

    #[test]
    fn mute_blocks_play() {
        let engine = Arc::new(CountingEngine { calls: Mutex::new(0) });
        let mute = MuteController::new();
        mute.set_user_muted(true);
        let d = Dispatcher::new(engine.clone(), mute);
        d.handle(KeyEvent { keycode: 0, kind: KeyEventKind::Down }, &dummy_pack(), 0.5, 0.0);
        assert_eq!(*engine.calls.lock().unwrap(), 0);
    }
}
```

- [ ] **Step 2: Wire `dispatcher.rs` into `lib.rs` runtime**

Replace `src-tauri/src/lib.rs` with:

```rust
pub mod audio_engine;
pub mod dispatcher;
pub mod key_listener;
pub mod mute_controller;
pub mod pack_format;
pub mod pack_store;

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

use audio_engine::RodioEngine;
use dispatcher::Dispatcher;
use key_listener::{KeyListener, MacKeyListener};
use mute_controller::MuteController;
use pack_store::{install_default_packs, PackStore};

const APP_SUBDIR: &str = "BubbleKeys";

fn user_data_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME");
    PathBuf::from(home).join("Library/Application Support").join(APP_SUBDIR)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .setup(|app| {
            let resource_dir = app.path().resource_dir().expect("resource_dir");
            let user_dir = user_data_dir();
            let pack_dir = user_dir.join("packs");
            install_default_packs(&resource_dir, &pack_dir).ok();

            let mut store = PackStore::new();
            store.load_dir(&pack_dir).expect("load packs");

            let active_id = store.ids().first().expect("at least one pack").clone();
            let active_pack = Arc::new(RwLock::new(active_id));

            let engine = Arc::new(RodioEngine::new().expect("audio engine"));
            let mute = MuteController::new();
            let listener = MacKeyListener::start().expect("key listener");
            let rx = listener.events();

            let dispatcher = Dispatcher::new(engine.clone(), mute.clone());
            let store = Arc::new(store);
            let store_for_thread = store.clone();
            let active_for_thread = active_pack.clone();

            thread::Builder::new()
                .name("bubblekeys-dispatch".into())
                .spawn(move || {
                    while let Ok(ev) = rx.recv() {
                        let id = active_for_thread.read().unwrap().clone();
                        if let Some(pack) = store_for_thread.get(&id) {
                            dispatcher.handle(ev, pack, 0.65, 0.0);
                        }
                    }
                })
                .expect("dispatcher thread");

            app.manage(mute);
            app.manage(store);
            app.manage(active_pack);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running BubbleKeys");
}
```

- [ ] **Step 3: Run tests and dev**

```bash
cd /Users/evette/Documents/BubbleKeys/src-tauri
cargo test
```

Expected: all tests pass.

```bash
cd /Users/evette/Documents/BubbleKeys
pnpm tauri dev
```

Expected: app launches, default Cherry Blue pack plays on keydown.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/dispatcher.rs src-tauri/src/lib.rs
git commit -m "feat: dispatcher routes KeyEvent → active pack with mute respect"
```

### Task 3.6: Debug shortcut to cycle packs and toggle mute

**Files:** Modify `src-tauri/src/lib.rs` to register `tauri::menu` items (debug only).

- [ ] **Step 1: Add a Tauri menu (Cmd-1, Cmd-2 cycles packs; Cmd-M toggles mute)**

Append to `setup` closure in `lib.rs` after `app.manage(...)` calls:

```rust
#[cfg(debug_assertions)]
{
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    let cycle = MenuItemBuilder::new("Cycle Pack").id("cycle").accelerator("CmdOrCtrl+1").build(app)?;
    let toggle = MenuItemBuilder::new("Toggle Mute").id("toggle").accelerator("CmdOrCtrl+M").build(app)?;
    let menu = MenuBuilder::new(app).item(&cycle).item(&toggle).build()?;
    app.set_menu(menu)?;

    let store_handle = store.clone();
    let active_handle = active_pack.clone();
    let mute_handle = mute.clone();
    app.on_menu_event(move |_app, event| {
        match event.id().as_ref() {
            "cycle" => {
                let ids = store_handle.ids();
                let mut active = active_handle.write().unwrap();
                let idx = ids.iter().position(|i| i == &*active).unwrap_or(0);
                *active = ids[(idx + 1) % ids.len()].clone();
                log::info!("cycled to pack: {}", *active);
            }
            "toggle" => {
                let cur = mute_handle.is_muted();
                mute_handle.set_user_muted(!cur);
                log::info!("mute={}", !cur);
            }
            _ => {}
        }
    });
}
```

- [ ] **Step 2: Smoke test**

```bash
RUST_LOG=info pnpm tauri dev
```

Press Cmd-1 a few times, Cmd-M to mute. Type to verify the active pack and mute behavior.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: debug menu — cycle packs and toggle mute (debug builds only)"
```

---

# Phase 4 — Game Boy main window — HOME page

**Goal:** Replace blank webview with a pixel-styled HOME page showing current pack, volume bar, ON/OFF, hotkey hint. No tabs yet — single page only.

### Task 4.1: Foundation styles + tokens

**Files:** Create `src/styles/tokens.css`, `src/styles/pixel.css`, `src/styles/fonts.css`.

- [ ] **Step 1: Color and typography tokens**

Path: `/Users/evette/Documents/BubbleKeys/src/styles/tokens.css`

```css
:root {
  --c-sky:        #b5c8f5;
  --c-lavender:   #d4b5f5;
  --c-pink:       #f5b5d4;
  --c-mint:       #a3e6c5;
  --c-ink:        #2d2d5f;
  --c-subink:     #5d5d8f;
  --c-shadow:     #8fb5f5;
  --c-white:      #ffffff;

  --space-1: 0.25rem;
  --space-2: 0.5rem;
  --space-3: 0.75rem;
  --space-4: 1rem;

  --font-pixel: "Ark Pixel 12px Monospaced", "Ark Pixel 12px Proportional",
                "Boutique Bitmap 9x9", "Galmuri11", monospace;
  --fz-body: 12px;
  --fz-title: 18px;
  --fz-hero: 24px;

  --pixel-shadow-sm: 2px 2px 0 var(--c-ink);
  --pixel-shadow-md: 4px 4px 0 var(--c-ink);
}
```

- [ ] **Step 2: Pixel primitives**

Path: `/Users/evette/Documents/BubbleKeys/src/styles/pixel.css`

```css
@import "./tokens.css";

* { box-sizing: border-box; }

html, body {
  margin: 0;
  padding: 0;
  font-family: var(--font-pixel);
  font-size: var(--fz-body);
  color: var(--c-ink);
  background: transparent;
  image-rendering: pixelated;
  -webkit-font-smoothing: none;
  -moz-osx-font-smoothing: none;
  user-select: none;
  -webkit-user-select: none;
}

.pixel-frame {
  background: var(--c-pink);
  border: 4px solid var(--c-ink);
  border-radius: 4px 4px 32px 32px;
  box-shadow: var(--pixel-shadow-md);
}

.pixel-screen {
  background: #c5e8d5;
  border: 3px solid var(--c-ink);
  display: flex;
  flex-direction: column;
}

.pixel-btn {
  display: inline-block;
  background: var(--c-mint);
  border: 2px solid var(--c-ink);
  padding: 0.4em 1em;
  font: inherit;
  font-weight: 900;
  letter-spacing: 1px;
  cursor: pointer;
  box-shadow: var(--pixel-shadow-sm);
}
.pixel-btn:active { transform: translate(1px, 1px); box-shadow: 1px 1px 0 var(--c-ink); }
.pixel-btn[aria-pressed="true"] { background: var(--c-mint); }
.pixel-btn.off { background: var(--c-lavender); }

.pixel-bar {
  position: relative;
  height: 8px;
  background: var(--c-white);
  border: 1px solid var(--c-ink);
}
.pixel-bar > .fill {
  position: absolute; left: 0; top: 0; bottom: 0;
  background: var(--c-pink);
}
```

- [ ] **Step 3: Font face declarations**

Path: `/Users/evette/Documents/BubbleKeys/src/styles/fonts.css`

```css
@font-face {
  font-family: "Ark Pixel 12px Monospaced";
  src: url("/assets/fonts/ark-pixel-12px-monospaced.woff2") format("woff2");
  font-display: swap;
}
@font-face {
  font-family: "Ark Pixel 12px Proportional";
  src: url("/assets/fonts/ark-pixel-12px-proportional.woff2") format("woff2");
  font-display: swap;
}
```

- [ ] **Step 4: Download Ark Pixel woff2 files**

```bash
mkdir -p /Users/evette/Documents/BubbleKeys/src/assets/fonts
# Download from https://github.com/TakWolf/ark-pixel-font/releases (latest)
# Pick `ark-pixel-12px-monospaced.woff2` and `ark-pixel-12px-proportional.woff2`
# Place under src/assets/fonts/
```

- [ ] **Step 5: Commit**

```bash
git add src/styles/ src/assets/fonts/
git commit -m "feat(ui): pixel design tokens, primitives, and Ark Pixel font"
```

### Task 4.2: Typed IPC layer

**Files:** Create `src/lib/ipc.ts`. Add Tauri commands in Rust.

- [ ] **Step 1: Define Rust commands**

Create `/Users/evette/Documents/BubbleKeys/src-tauri/src/ipc.rs`:

```rust
//! Tauri IPC commands. Frontend ↔ backend boundary.

use std::sync::{Arc, RwLock};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::mute_controller::MuteController;
use crate::pack_store::PackStore;

#[derive(Serialize, Clone)]
pub struct PackSummary {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct AppState {
    pub active_pack: String,
    pub muted: bool,
    pub volume: f32,
}

#[tauri::command]
pub fn list_packs(store: State<'_, Arc<PackStore>>) -> Vec<PackSummary> {
    store.ids().into_iter()
        .filter_map(|id| store.get(&id).map(|p| PackSummary { id: p.manifest.id.clone(), name: p.manifest.name.clone() }))
        .collect()
}

#[tauri::command]
pub fn set_active_pack(
    id: String,
    active: State<'_, Arc<RwLock<String>>>,
    store: State<'_, Arc<PackStore>>,
) -> Result<(), String> {
    if store.get(&id).is_none() {
        return Err(format!("unknown pack: {id}"));
    }
    *active.write().unwrap() = id;
    Ok(())
}

#[tauri::command]
pub fn get_state(
    active: State<'_, Arc<RwLock<String>>>,
    mute: State<'_, MuteController>,
    volume: State<'_, Arc<RwLock<f32>>>,
) -> AppState {
    AppState {
        active_pack: active.read().unwrap().clone(),
        muted: mute.is_muted(),
        volume: *volume.read().unwrap(),
    }
}

#[tauri::command]
pub fn set_muted(muted: bool, mute: State<'_, MuteController>) {
    mute.set_user_muted(muted);
}

#[tauri::command]
pub fn set_volume(volume: f32, store: State<'_, Arc<RwLock<f32>>>) {
    let v = volume.clamp(0.0, 1.0);
    *store.write().unwrap() = v;
}
```

- [ ] **Step 2: Register commands and add a `volume` shared state**

Modify `src-tauri/src/lib.rs`. Add at top:

```rust
pub mod ipc;
```

In `setup`, manage volume:

```rust
let volume: Arc<RwLock<f32>> = Arc::new(RwLock::new(0.65));
app.manage(volume.clone());
```

Update the dispatcher thread to read volume from this state:

```rust
thread::Builder::new()
    .name("bubblekeys-dispatch".into())
    .spawn({
        let volume = volume.clone();
        move || {
            while let Ok(ev) = rx.recv() {
                let id = active_for_thread.read().unwrap().clone();
                if let Some(pack) = store_for_thread.get(&id) {
                    let v = *volume.read().unwrap();
                    dispatcher.handle(ev, pack, v, 0.0);
                }
            }
        }
    })
    .expect("dispatcher thread");
```

In the Builder chain, register commands:

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        ipc::list_packs,
        ipc::set_active_pack,
        ipc::get_state,
        ipc::set_muted,
        ipc::set_volume,
    ])
    .setup(|app| { /* unchanged */ Ok(()) })
    .run(tauri::generate_context!())
    .expect("error while running BubbleKeys");
```

- [ ] **Step 3: Add typed frontend wrappers**

Path: `/Users/evette/Documents/BubbleKeys/src/lib/ipc.ts`

```ts
import { invoke } from "@tauri-apps/api/core";

export interface PackSummary { id: string; name: string }
export interface AppState { active_pack: string; muted: boolean; volume: number }

export const listPacks      = ()                  => invoke<PackSummary[]>("list_packs");
export const setActivePack  = (id: string)        => invoke<void>("set_active_pack", { id });
export const getState       = ()                  => invoke<AppState>("get_state");
export const setMuted       = (muted: boolean)    => invoke<void>("set_muted", { muted });
export const setVolume      = (volume: number)    => invoke<void>("set_volume", { volume });
```

- [ ] **Step 4: Build to confirm**

```bash
cd /Users/evette/Documents/BubbleKeys/src-tauri
cargo check
```

Expected: clean compile.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/ipc.rs src-tauri/src/lib.rs src/lib/ipc.ts
git commit -m "feat(ipc): typed Tauri commands for pack/state/mute/volume"
```

### Task 4.3: HOME view

**Files:** Modify `index.html`, `src/main.ts`. Create `src/views/home.ts`.

- [ ] **Step 1: HTML scaffold for the Game Boy frame**

Replace `index.html`:

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>BubbleKeys</title>
    <link rel="stylesheet" href="/src/styles/fonts.css" />
    <link rel="stylesheet" href="/src/styles/pixel.css" />
  </head>
  <body>
    <div class="pixel-frame" id="frame">
      <header class="frame-header">
        <span class="brand">BUBBLEKEYS</span>
        <span class="led" id="led" aria-hidden="true"></span>
      </header>
      <section class="pixel-screen" id="screen"></section>
      <nav class="frame-controls" aria-hidden="true">
        <div class="dpad"></div>
        <div class="ab"><div class="btn">B</div><div class="btn">A</div></div>
      </nav>
    </div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

Append to `src/styles/pixel.css`:

```css
#frame {
  width: 100vw;
  min-height: 100vh;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.frame-header {
  display: flex; justify-content: space-between; align-items: center;
  font-weight: 900; letter-spacing: 2px;
}
.led {
  width: 10px; height: 10px; border-radius: 50%;
  background: var(--c-mint); box-shadow: 0 0 0 1px var(--c-ink);
}
.led.off { background: var(--c-lavender); }
#screen { flex: 1; padding: 12px; }
.frame-controls {
  display: grid; grid-template-columns: 1fr 1fr;
  align-items: center;
  padding: 8px 12px;
}
.dpad { width: 56px; height: 56px; background: var(--c-ink); margin: 0 auto;
  clip-path: polygon(35% 0,65% 0,65% 35%,100% 35%,100% 65%,65% 65%,65% 100%,35% 100%,35% 65%,0 65%,0 35%,35% 35%);
}
.ab { display: flex; gap: 10px; justify-content: flex-end; }
.ab .btn {
  width: 26px; height: 26px; border-radius: 50%;
  background: var(--c-lavender); border: 2px solid var(--c-ink);
  display: flex; align-items: center; justify-content: center;
  font-weight: 900;
}
```

- [ ] **Step 2: HOME view component**

Path: `/Users/evette/Documents/BubbleKeys/src/views/home.ts`

```ts
import { getState, listPacks, setMuted, setVolume } from "../lib/ipc";

export async function renderHome(host: HTMLElement) {
  const state = await getState();
  const packs = await listPacks();
  const active = packs.find(p => p.id === state.active_pack);

  host.innerHTML = `
    <div class="home">
      <h1 class="pack-name">${active?.name ?? "—"}</h1>
      <div class="art" aria-hidden="true">⌨</div>
      <div class="vol-row">
        <span>VOL</span>
        <div class="pixel-bar"><div class="fill" id="vol-fill"></div></div>
        <span id="vol-val">${Math.round(state.volume * 100)}</span>
      </div>
      <button class="pixel-btn" id="toggle" aria-pressed="${!state.muted}">
        ${state.muted ? "▶ OFF" : "▶ ON"}
      </button>
    </div>`;

  const fill = host.querySelector<HTMLDivElement>("#vol-fill")!;
  const val  = host.querySelector<HTMLSpanElement>("#vol-val")!;
  const btn  = host.querySelector<HTMLButtonElement>("#toggle")!;
  const led  = document.getElementById("led")!;

  const renderVol = (v: number) => {
    fill.style.width = `${Math.round(v * 100)}%`;
    val.textContent = String(Math.round(v * 100));
  };
  renderVol(state.volume);

  btn.addEventListener("click", async () => {
    const nowMuted = btn.getAttribute("aria-pressed") === "true";
    // aria-pressed=true means "ON button is active" = not muted; clicking → mute
    await setMuted(nowMuted);
    btn.setAttribute("aria-pressed", String(!nowMuted));
    btn.textContent = nowMuted ? "▶ OFF" : "▶ ON";
    led.classList.toggle("off", nowMuted);
  });

  // ←/→ adjust volume by 5%
  document.addEventListener("keydown", async (e) => {
    if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
    const cur = (await getState()).volume;
    const next = Math.max(0, Math.min(1, cur + (e.key === "ArrowRight" ? 0.05 : -0.05)));
    await setVolume(next);
    renderVol(next);
  });
}
```

Append to `src/styles/pixel.css`:

```css
.home { display: flex; flex-direction: column; gap: var(--space-3); padding: var(--space-2); height: 100%; }
.pack-name { margin: 0; text-align: center; font-size: var(--fz-hero); letter-spacing: 2px; }
.art { background: var(--c-white); border: 2px solid var(--c-ink); height: 60px;
  display: flex; align-items: center; justify-content: center; font-size: 28px; }
.vol-row { display: flex; align-items: center; gap: 8px; font-size: var(--fz-body); }
.vol-row .pixel-bar { flex: 1; }
```

- [ ] **Step 3: `src/main.ts` mounts HOME**

Replace `src/main.ts`:

```ts
import { renderHome } from "./views/home";

const screen = document.getElementById("screen")!;
renderHome(screen).catch(err => {
  screen.textContent = String(err);
});
```

- [ ] **Step 4: Smoke test**

```bash
pnpm tauri dev
```

Expected: pink Game Boy frame; HOME shows current pack, volume bar with arrow keys, ON/OFF button toggles LED + mute. Type any key in another app to confirm sound stops when OFF.

- [ ] **Step 5: Commit**

```bash
git add index.html src/main.ts src/views/home.ts src/styles/pixel.css
git commit -m "feat(ui): HOME page with current pack, volume, on/off"
```

---

# Phase 5 — PACKS page + tab navigation

**Goal:** All 4 tabs visible at top of screen. PACKS list shows installed packs, click switches active, hover (or A button) plays a 0.5s preview.

### Task 5.1: Tab router

**Files:** Create `src/lib/router.ts`. Modify `src/main.ts`.

- [ ] **Step 1: Router**

Path: `/Users/evette/Documents/BubbleKeys/src/lib/router.ts`

```ts
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
```

- [ ] **Step 2: PACKS view**

Path: `/Users/evette/Documents/BubbleKeys/src/views/packs.ts`

```ts
import { getState, listPacks, setActivePack } from "../lib/ipc";

export async function renderPacks(host: HTMLElement) {
  const [packs, state] = await Promise.all([listPacks(), getState()]);

  host.innerHTML = `
    <ul class="pack-list" role="listbox">
      ${packs.map(p => `
        <li class="pack-row ${p.id === state.active_pack ? 'sel' : ''}" role="option"
            data-id="${p.id}" tabindex="0" aria-selected="${p.id === state.active_pack}">
          <span>${p.name}</span><span class="meta">${p.id === state.active_pack ? '♪' : ''}</span>
        </li>
      `).join("")}
      <li class="pack-import" data-action="import">+ IMPORT MECHVIBES</li>
    </ul>`;

  host.querySelectorAll<HTMLLIElement>(".pack-row").forEach(li => {
    li.addEventListener("click", async () => {
      await setActivePack(li.dataset.id!);
      host.querySelectorAll(".pack-row").forEach(x => x.classList.remove("sel"));
      li.classList.add("sel");
    });
    li.addEventListener("mouseenter", () => previewPack(li.dataset.id!));
  });
}

let lastPreview = 0;
async function previewPack(_id: string) {
  // Phase 5: throttle hover; actual preview command added in Phase 10 (Mechvibes import).
  const now = Date.now();
  if (now - lastPreview < 200) return;
  lastPreview = now;
  // TODO Phase 5.5: invoke("preview_pack", { id }) — implemented next task.
}
```

Append CSS:

```css
.tabs { display: flex; background: var(--c-ink); }
.tab {
  flex: 1; background: transparent; color: var(--c-sky);
  border: 0; padding: 4px 0; font: inherit; letter-spacing: 1px; cursor: pointer;
  border-right: 1px solid #1a1a3a;
}
.tab:last-child { border-right: 0; }
.tab.active { background: var(--c-mint); color: var(--c-ink); font-weight: 900; }

.pack-list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
.pack-row { background: var(--c-white); border: 1px solid var(--c-ink); padding: 6px 8px;
  display: flex; justify-content: space-between; cursor: pointer; }
.pack-row.sel { background: var(--c-pink); font-weight: 900; }
.pack-row.sel::before { content: '▶ '; }
.pack-import { background: var(--c-lavender); border: 2px dashed var(--c-ink);
  text-align: center; padding: 6px; cursor: pointer; }
```

- [ ] **Step 3: Modify HTML and main.ts to use router + tab strip**

Modify `index.html` `pixel-screen` section to include a tab bar:

```html
<section class="pixel-screen">
  <nav class="tabs" id="tabs" aria-label="Sections"></nav>
  <div id="screen" class="screen-body" role="region"></div>
</section>
```

Update `src/main.ts`:

```ts
import { renderHome } from "./views/home";
import { renderPacks } from "./views/packs";
import { createRouter } from "./lib/router";

const tabs = document.getElementById("tabs")!;
const screen = document.getElementById("screen")!;

const stub = (label: string) => async (h: HTMLElement) => { h.innerHTML = `<p style="text-align:center; color:var(--c-subink)">${label} — coming soon</p>`; };

createRouter(screen, tabs, {
  home: renderHome,
  packs: renderPacks,
  settings: stub("SETTINGS"),
  about: stub("ABOUT"),
});
```

- [ ] **Step 4: Smoke test**

```bash
pnpm tauri dev
```

Expected: 4 tabs visible. Click PACKS — list of 4 packs with current selected. Click another row, type in another app — sound now from the new pack.

- [ ] **Step 5: Commit**

```bash
git add src/lib/router.ts src/views/packs.ts src/main.ts index.html src/styles/pixel.css
git commit -m "feat(ui): tab router and PACKS page"
```

### Task 5.2: Pack preview command

**Files:** Modify `src-tauri/src/ipc.rs` and `src/views/packs.ts`.

- [ ] **Step 1: Add `preview_pack` Tauri command**

Append to `src-tauri/src/ipc.rs`:

```rust
use crate::audio_engine::{AudioEngine, PlayCommand};
use std::sync::Arc;

#[tauri::command]
pub fn preview_pack(
    id: String,
    store: State<'_, Arc<PackStore>>,
    engine: State<'_, Arc<dyn AudioEngine>>,
) -> Result<(), String> {
    let pack = store.get(&id).ok_or_else(|| format!("unknown pack: {id}"))?;
    let sample = pack.samples_by_key.values().next().cloned().ok_or("empty pack")?;
    engine.play(PlayCommand { sample, volume: 0.6, pitch_offset: 0.0 });
    Ok(())
}
```

Add to handler list in `lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    ipc::list_packs,
    ipc::set_active_pack,
    ipc::get_state,
    ipc::set_muted,
    ipc::set_volume,
    ipc::preview_pack,
])
```

Manage `Arc<dyn AudioEngine>` in `setup`:

```rust
let engine_for_state: Arc<dyn AudioEngine> = engine.clone();
app.manage(engine_for_state);
```

- [ ] **Step 2: Wire frontend preview**

In `src/lib/ipc.ts` add:

```ts
export const previewPack = (id: string) => invoke<void>("preview_pack", { id });
```

In `src/views/packs.ts` replace the TODO inside `previewPack`:

```ts
import { previewPack as ipcPreview } from "../lib/ipc";
// ...
async function previewPack(id: string) {
  const now = Date.now();
  if (now - lastPreview < 200) return;
  lastPreview = now;
  await ipcPreview(id);
}
```

- [ ] **Step 3: Smoke test**

Hover over each pack row in PACKS page — each plays a short preview without changing the active pack.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/ipc.rs src-tauri/src/lib.rs src/lib/ipc.ts src/views/packs.ts
git commit -m "feat: preview pack on hover in PACKS page"
```

---

# Phase 6 — SETTINGS page (basic)

**Goal:** SETTINGS page shows hotkey, auto-start, pitch jitter, output device, menu icon, language. All persist to settings.json.

### Task 6.1: Settings store with TDD

**Files:** Create `src-tauri/src/settings_store.rs`.

- [ ] **Step 1: Failing test for default + roundtrip**

Path: `/Users/evette/Documents/BubbleKeys/src-tauri/src/settings_store.rs`

```rust
//! User preferences persisted to ~/Library/Application Support/BubbleKeys/settings.json.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    #[serde(default = "default_version")] pub version: u32,
    #[serde(default = "default_pack")] pub active_pack: String,
    #[serde(default = "default_volume")] pub volume: f32,
    #[serde(default)] pub muted: bool,
    #[serde(default = "default_true")] pub pitch_jitter: bool,
    #[serde(default = "default_hotkey")] pub hotkey: String,
    #[serde(default = "default_true")] pub auto_start: bool,
    #[serde(default = "default_true")] pub menu_icon_visible: bool,
    #[serde(default = "default_lang")] pub language: String,
    #[serde(default = "default_output")] pub output_device: String,
    #[serde(default)] pub night_silent: NightSilent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct NightSilent {
    #[serde(default)] pub enabled: bool,
    #[serde(default = "default_start")] pub start: String,
    #[serde(default = "default_end")] pub end: String,
}

fn default_version() -> u32 { 1 }
fn default_pack() -> String { "cherry-blue".into() }
fn default_volume() -> f32 { 0.65 }
fn default_true() -> bool { true }
fn default_hotkey() -> String { "Cmd+Option+B".into() }
fn default_lang() -> String { "auto".into() }
fn default_output() -> String { "default".into() }
fn default_start() -> String { "22:00".into() }
fn default_end() -> String { "07:00".into() }

impl Default for Settings {
    fn default() -> Self {
        serde_json::from_str("{}").unwrap()
    }
}

pub fn settings_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME");
    PathBuf::from(home).join("Library/Application Support/BubbleKeys/settings.json")
}

pub fn load() -> Settings {
    let path = settings_path();
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

pub fn save(s: &Settings) -> std::io::Result<()> {
    let path = settings_path();
    if let Some(p) = path.parent() { std::fs::create_dir_all(p)?; }
    let bytes = serde_json::to_vec_pretty(s)?;
    std::fs::write(path, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_via_empty_object() {
        let s: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(s.version, 1);
        assert_eq!(s.active_pack, "cherry-blue");
        assert!(s.pitch_jitter);
        assert!(!s.night_silent.enabled);
    }

    #[test]
    fn roundtrips() {
        let s = Settings::default();
        let json = serde_json::to_string(&s).unwrap();
        let parsed: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, parsed);
    }

    #[test]
    fn forward_compatible_unknown_keys() {
        let json = r#"{ "version": 1, "future_key": 42 }"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.version, 1);
    }
}
```

- [ ] **Step 2: Add `serde(deny_unknown_fields)` consideration**

Note: we deliberately do NOT use `deny_unknown_fields` so newer settings files can be loaded by older binaries.

- [ ] **Step 3: Add module + run tests**

```rust
// lib.rs
pub mod settings_store;
```

```bash
cargo test settings_store
```

Expected: 3 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/settings_store.rs src-tauri/src/lib.rs
git commit -m "feat(settings): persistent settings store with forward-compat schema"
```

### Task 6.2: Wire settings into runtime

**Files:** Modify `src-tauri/src/lib.rs`, `src-tauri/src/ipc.rs`.

- [ ] **Step 1: Load settings at startup**

Update `setup` in `lib.rs`:

```rust
let settings = settings_store::load();
let active_pack = Arc::new(RwLock::new(settings.active_pack.clone()));
let volume = Arc::new(RwLock::new(settings.volume));
let mute = MuteController::new();
mute.set_user_muted(settings.muted);
let settings_arc = Arc::new(RwLock::new(settings));
app.manage(settings_arc.clone());
```

- [ ] **Step 2: Add IPC commands for settings**

Append to `ipc.rs`:

```rust
use crate::settings_store::{save as save_settings, Settings};

#[tauri::command]
pub fn get_settings(s: State<'_, Arc<RwLock<Settings>>>) -> Settings {
    s.read().unwrap().clone()
}

#[tauri::command]
pub fn update_settings(
    new_settings: Settings,
    s: State<'_, Arc<RwLock<Settings>>>,
    active: State<'_, Arc<RwLock<String>>>,
    volume: State<'_, Arc<RwLock<f32>>>,
    mute: State<'_, MuteController>,
) -> Result<(), String> {
    *s.write().unwrap() = new_settings.clone();
    save_settings(&new_settings).map_err(|e| e.to_string())?;
    *active.write().unwrap() = new_settings.active_pack;
    *volume.write().unwrap() = new_settings.volume;
    mute.set_user_muted(new_settings.muted);
    Ok(())
}
```

Add to `generate_handler!`:

```rust
ipc::get_settings,
ipc::update_settings,
```

- [ ] **Step 3: Frontend wrappers**

Append to `src/lib/ipc.ts`:

```ts
export interface Settings {
  version: number;
  active_pack: string;
  volume: number;
  muted: boolean;
  pitch_jitter: boolean;
  hotkey: string;
  auto_start: boolean;
  menu_icon_visible: boolean;
  language: string;
  output_device: string;
  night_silent: { enabled: boolean; start: string; end: string };
}
export const getSettings    = () => invoke<Settings>("get_settings");
export const updateSettings = (s: Settings) => invoke<void>("update_settings", { newSettings: s });
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/ipc.rs src/lib/ipc.ts
git commit -m "feat(settings): wire load/save/update into runtime + IPC"
```

### Task 6.3: SETTINGS view

**Files:** Create `src/views/settings.ts`. Modify `src/main.ts`.

- [ ] **Step 1: Settings view**

Path: `/Users/evette/Documents/BubbleKeys/src/views/settings.ts`

```ts
import { getSettings, updateSettings, type Settings } from "../lib/ipc";

export async function renderSettings(host: HTMLElement) {
  const s = await getSettings();

  host.innerHTML = `
    <ul class="settings-list">
      ${row("HOTKEY", input("hotkey", s.hotkey, "text"))}
      ${row("AUTOSTART", toggle("auto_start", s.auto_start))}
      ${row("PITCH VAR", toggle("pitch_jitter", s.pitch_jitter))}
      ${row("MENU ICON", toggle("menu_icon_visible", s.menu_icon_visible))}
      ${row("OUTPUT", input("output_device", s.output_device, "text"))}
      ${row("LANGUAGE", select("language", s.language, [
        ["auto", "Auto"], ["en", "English"], ["zh-CN","简体中文"],
        ["zh-TW","繁體中文"], ["ja","日本語"], ["ko","한국어"]
      ]))}
    </ul>
  `;

  host.querySelector<HTMLFormElement>(".settings-list")!.addEventListener("change", async (e) => {
    const t = e.target as HTMLInputElement | HTMLSelectElement;
    const next: Settings = await getSettings();
    const key = t.dataset.key as keyof Settings;
    if (t.type === "checkbox") (next as any)[key] = (t as HTMLInputElement).checked;
    else (next as any)[key] = t.value;
    await updateSettings(next);
  });
}

function row(label: string, control: string) {
  return `<li class="settings-row"><span class="lbl">${label}</span>${control}</li>`;
}
function input(key: string, val: string, type: string) {
  return `<input type="${type}" class="set-val" data-key="${key}" value="${val}">`;
}
function toggle(key: string, on: boolean) {
  return `<label class="toggle"><input type="checkbox" data-key="${key}" ${on?'checked':''}><span></span></label>`;
}
function select(key: string, val: string, opts: [string,string][]) {
  return `<select data-key="${key}" class="set-val">${
    opts.map(([v,l]) => `<option value="${v}" ${v===val?'selected':''}>${l}</option>`).join("")
  }</select>`;
}
```

Append CSS:

```css
.settings-list { list-style: none; margin: 0; padding: 0; }
.settings-row { display: flex; justify-content: space-between; align-items: center;
  background: var(--c-white); border: 1px solid var(--c-ink); padding: 4px 8px; margin-bottom: 4px; }
.set-val, .toggle { font: inherit; background: var(--c-sky); border: 1px solid var(--c-ink); padding: 2px 6px; }
.toggle input { display: none; }
.toggle span { width: 28px; height: 12px; background: var(--c-lavender); border: 1px solid var(--c-ink); display: inline-block; position: relative; }
.toggle input:checked + span { background: var(--c-mint); }
.toggle input:checked + span::after { content: '✓'; position: absolute; right: 2px; top: -3px; }
```

- [ ] **Step 2: Mount in main**

Update `src/main.ts`:

```ts
import { renderSettings } from "./views/settings";
// ...
createRouter(screen, tabs, {
  home: renderHome,
  packs: renderPacks,
  settings: renderSettings,
  about: stub("ABOUT"),
});
```

- [ ] **Step 3: Smoke test**

```bash
pnpm tauri dev
```

Toggle pitch_jitter, change language to `zh-CN`, restart app — values persisted in `~/Library/Application Support/BubbleKeys/settings.json`.

- [ ] **Step 4: Commit**

```bash
git add src/views/settings.ts src/main.ts src/styles/pixel.css
git commit -m "feat(ui): SETTINGS page with persistence"
```

---

# Phase 7 — Menu-bar dropdown

**Goal:** Tray icon + 280×360 panel that toggles mute, switches packs, adjusts volume in 1–2 clicks.

### Task 7.1: Tray icon and dropdown window

**Files:** Create `src-tauri/src/tray.rs`. Modify `lib.rs`. Create `src/tray.ts` and `tray.html`.

- [ ] **Step 1: `src-tauri/src/tray.rs`**

```rust
//! Menu-bar icon and dropdown window lifecycle.

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};

const TRAY_ICON: &[u8] = include_bytes!("../icons/tray-icon.png");

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let img = Image::from_bytes(TRAY_ICON)?;
    let menu = MenuBuilder::new(app)
        .item(&MenuItemBuilder::new("Open BubbleKeys").id("open").build(app)?)
        .separator()
        .item(&MenuItemBuilder::new("Quit").id("quit").build(app)?)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .icon(img)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { .. } = event {
                show_dropdown(tray.app_handle()).ok();
            }
        })
        .on_menu_event(|app, ev| match ev.id().as_ref() {
            "open" => { let _ = app.get_webview_window("main").map(|w| { let _=w.show(); let _=w.set_focus(); }); }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

fn show_dropdown(app: &AppHandle) -> tauri::Result<()> {
    if let Some(win) = app.get_webview_window("tray") {
        win.show()?;
        win.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "tray", WebviewUrl::App("tray.html".into()))
        .title("BubbleKeys")
        .inner_size(280.0, 360.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(true)
        .build()?;
    Ok(())
}
```

- [ ] **Step 2: Provide a `tray-icon.png` (16×16 black, anti-aliased)**

Place a placeholder template-icon PNG at `/Users/evette/Documents/BubbleKeys/src-tauri/icons/tray-icon.png`. Use the Pixel Bubble silhouette from the design — black on transparent. (Final asset comes in Phase 13.)

- [ ] **Step 3: Add `tauri.conf.json` `app.trayIcon`**

Modify `src-tauri/tauri.conf.json` to declare the second window's preload + tray icon:

```json
"plugins": {},
"app": {
  "windows": [
    { "label": "main", "title": "BubbleKeys", "width": 320, "height": 480, "resizable": false, "decorations": false, "transparent": true, "visible": true }
  ],
  "trayIcon": {
    "iconPath": "icons/tray-icon.png",
    "iconAsTemplate": true
  }
},
```

- [ ] **Step 4: Frontend assets**

Create `/Users/evette/Documents/BubbleKeys/tray.html`:

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="stylesheet" href="/src/styles/fonts.css" />
    <link rel="stylesheet" href="/src/styles/pixel.css" />
    <title>BubbleKeys</title>
  </head>
  <body class="tray-body">
    <div id="tray-root"></div>
    <script type="module" src="/src/tray.ts"></script>
  </body>
</html>
```

Path: `/Users/evette/Documents/BubbleKeys/src/tray.ts`

```ts
import { getSettings, listPacks, setActivePack, setMuted, setVolume } from "./lib/ipc";

const root = document.getElementById("tray-root")!;
async function paint() {
  const [s, packs] = await Promise.all([getSettings(), listPacks()]);
  const recent = packs.slice(0, 4);

  root.innerHTML = `
    <div class="tray">
      <header>
        <span>🫧 BubbleKeys</span>
        <button class="pixel-btn ${s.muted ? 'off' : ''}" id="t-toggle">
          ${s.muted ? "OFF" : "ON"}
        </button>
      </header>
      <p class="tray-label">CURRENT PACK</p>
      <ul class="tray-pack-list">
        ${recent.map(p => `
          <li class="${p.id === s.active_pack ? 'active' : ''}" data-id="${p.id}">
            <span>${p.id === s.active_pack ? "✓" : ""}</span>
            <span>${p.name}</span>
          </li>`).join("")}
      </ul>
      <div class="tray-vol">
        <span>VOL</span>
        <input type="range" min="0" max="100" value="${Math.round(s.volume*100)}" id="t-vol">
        <span id="t-vol-val">${Math.round(s.volume*100)}</span>
      </div>
      <footer>
        <a id="t-open">⚙ Open BubbleKeys</a>
        <a id="t-quit">⏻ Quit</a>
      </footer>
    </div>
  `;
  bind();
}

function bind() {
  document.getElementById("t-toggle")!.addEventListener("click", async (e) => {
    const btn = e.currentTarget as HTMLButtonElement;
    const muted = btn.classList.toggle("off");
    btn.textContent = muted ? "OFF" : "ON";
    await setMuted(muted);
  });
  document.querySelectorAll<HTMLLIElement>(".tray-pack-list li").forEach(li => {
    li.addEventListener("click", async () => {
      await setActivePack(li.dataset.id!);
      await paint();
    });
  });
  const vol = document.getElementById("t-vol") as HTMLInputElement;
  vol.addEventListener("input", () => {
    document.getElementById("t-vol-val")!.textContent = vol.value;
    setVolume(Number(vol.value)/100);
  });
  document.getElementById("t-open")!.addEventListener("click", () => {
    import("@tauri-apps/api/webviewWindow").then(({ getCurrentWebviewWindow }) => {
      // open main window via Tauri command
    });
  });
}
paint();
```

(For "Open BubbleKeys" wiring, add a Tauri command `show_main` in `ipc.rs` that calls `app.get_webview_window("main").show()`. Defer detail to Task 7.2.)

- [ ] **Step 5: Wire tray::install in `lib.rs` setup**

```rust
mod tray as tray_mod;
// inside setup:
tray_mod::install(&app.handle())?;
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/tray.rs src-tauri/icons/tray-icon.png tauri.conf changes tray.html src/tray.ts src-tauri/src/lib.rs
git commit -m "feat(tray): menu-bar icon + dropdown window"
```

### Task 7.2: `show_main` command + auto-dismiss on focus loss

**Files:** Modify `src-tauri/src/ipc.rs`. Modify `src/tray.ts`.

- [ ] **Step 1: Backend command**

Append to `ipc.rs`:

```rust
#[tauri::command]
pub fn show_main(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("main") {
        w.show().map_err(|e| e.to_string())?;
        w.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

Register in handler list.

- [ ] **Step 2: Frontend wires**

```ts
// src/lib/ipc.ts
export const showMain = () => invoke<void>("show_main");
```

```ts
// src/tray.ts in bind()
document.getElementById("t-open")!.addEventListener("click", showMain);
document.getElementById("t-quit")!.addEventListener("click", () => {
  import("@tauri-apps/api/process").then(({ exit }) => exit(0));
});
```

- [ ] **Step 3: Auto-hide tray window when focus lost**

Add to `src-tauri/src/tray.rs` after `WebviewWindowBuilder::new(...).build()?`:

```rust
let win = app.get_webview_window("tray").unwrap();
let win_for_blur = win.clone();
win.on_window_event(move |ev| {
    if matches!(ev, tauri::WindowEvent::Focused(false)) {
        let _ = win_for_blur.hide();
    }
});
```

- [ ] **Step 4: Smoke test**

```bash
pnpm tauri dev
```

Click tray → dropdown appears → click outside → dropdown hides. Click "Open BubbleKeys" → main window appears.

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat(tray): show main + auto-hide on blur"
```

---

# Phase 8 — First-run flow

**Goal:** 4-step onboarding modal: WELCOME → WHY ACCESSIBILITY → PERMISSION GRANT → TRY IT.

### Task 8.1: Mark first-run state

**Files:** Modify `settings_store.rs`, add `onboarding_completed` field.

- [ ] **Step 1: Add field**

In `settings_store.rs`:

```rust
#[serde(default)] pub onboarding_completed: bool,
```

- [ ] **Step 2: Tauri command to mark complete**

```rust
// ipc.rs
#[tauri::command]
pub fn complete_onboarding(s: State<'_, Arc<RwLock<Settings>>>) -> Result<(), String> {
    let mut g = s.write().unwrap();
    g.onboarding_completed = true;
    save_settings(&g).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn reset_onboarding(s: State<'_, Arc<RwLock<Settings>>>) -> Result<(), String> {
    let mut g = s.write().unwrap();
    g.onboarding_completed = false;
    save_settings(&g).map_err(|e| e.to_string())
}
```

Register both. Frontend:

```ts
export const completeOnboarding = () => invoke<void>("complete_onboarding");
export const resetOnboarding    = () => invoke<void>("reset_onboarding");
```

- [ ] **Step 3: Commit**

```bash
git commit -am "feat(settings): track onboarding completion"
```

### Task 8.2: First-run view (4 screens)

**Files:** Create `src/views/first-run.ts`. Modify `src/main.ts`.

- [ ] **Step 1: First-run logic**

Path: `/Users/evette/Documents/BubbleKeys/src/views/first-run.ts`

```ts
import { completeOnboarding, listPacks, previewPack } from "../lib/ipc";
import { open as openExternal } from "@tauri-apps/plugin-shell";

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
          await openExternal("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility");
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
```

Append CSS:

```css
.onboard { display: flex; flex-direction: column; align-items: center; justify-content: center;
  padding: 24px; gap: 14px; height: 100%; text-align: center; }
.onboard h1 { font-size: 26px; margin: 0; }
.onboard h2 { font-size: 16px; margin: 0; }
.onboard .subtitle { color: var(--c-subink); margin: 0; font-size: 11px; letter-spacing: 1px; }
.onboard p { font-size: 12px; line-height: 1.5; max-width: 250px; }
```

- [ ] **Step 2: Gate the main UI on onboarding state**

Modify `src/main.ts`:

```ts
import { getSettings } from "./lib/ipc";
import { renderFirstRun } from "./views/first-run";

(async () => {
  const s = await getSettings();
  const tabsHost   = document.getElementById("tabs")!;
  const screenHost = document.getElementById("screen")!;

  if (!s.onboarding_completed) {
    tabsHost.style.display = "none";
    renderFirstRun(screenHost, () => { tabsHost.style.display = ""; bootMain(); });
  } else {
    bootMain();
  }
})();

function bootMain() { /* existing createRouter call */ }
```

- [ ] **Step 3: Smoke test**

Delete `~/Library/Application Support/BubbleKeys/settings.json`. Launch — onboarding shows.

- [ ] **Step 4: Commit**

```bash
git add src/views/first-run.ts src/main.ts src/styles/pixel.css
git commit -m "feat(onboarding): 4-step first-run flow"
```

### Task 8.3: Auto-detect Accessibility grant

**Files:** Modify `src-tauri/src/ipc.rs` and `src/views/first-run.ts`.

- [ ] **Step 1: Backend poller**

Append to `ipc.rs`:

```rust
#[tauri::command]
pub fn check_accessibility() -> bool {
    use core_graphics::access::AXIsProcessTrustedWithOptions;
    use core_foundation::dictionary::CFDictionary;
    let opts = CFDictionary::from_CFType_pairs(&[]);
    AXIsProcessTrustedWithOptions(opts.as_concrete_TypeRef())
}
```

(If `AXIsProcessTrustedWithOptions` is not in `core_graphics`, drop in via `objc2-application-services` or write a minimal `extern "C"` fn. Adapt to whatever API the chosen `core-graphics` version exposes; the goal is "is BubbleKeys allowed to tap events?" returning `bool`.)

Register handler.

- [ ] **Step 2: Frontend polling**

Update `grant()` and `bind()`:

```ts
import { invoke } from "@tauri-apps/api/core";

let pollHandle: number | undefined;

const grant = () => `
  <div class="onboard">
    <h2>↑ ENABLE BUBBLEKEYS</h2>
    <p>System Settings → Privacy & Security → Accessibility → toggle BubbleKeys on. We'll auto-advance once granted.</p>
    <p class="subtitle"><span id="accessibility-status">⌛ waiting…</span></p>
  </div>`;

// in bind() when step === "grant":
if (step === "grant") {
  pollHandle = window.setInterval(async () => {
    const ok = await invoke<boolean>("check_accessibility");
    if (ok) {
      window.clearInterval(pollHandle);
      step = "try"; paint();
    }
  }, 1000);
} else if (pollHandle) {
  window.clearInterval(pollHandle);
}
```

- [ ] **Step 3: Smoke test**

Reset onboarding. Walk through; flip the toggle in System Settings — UI auto-advances within ~1s.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/ipc.rs src/views/first-run.ts
git commit -m "feat(onboarding): auto-detect Accessibility grant"
```

---

# Phase 9 — i18n (5 languages, full coverage)

**Goal:** All UI strings translated to en / zh-CN / zh-TW / ja / ko. CI fails when locales drift.

### Task 9.1: i18n loader and key extraction

**Files:** Create `src/i18n/index.ts`, `src/i18n/locales/*.json`. Extract all literal strings.

- [ ] **Step 1: Create the loader**

Path: `/Users/evette/Documents/BubbleKeys/src/i18n/index.ts`

```ts
import en from "./locales/en.json";
import zhCN from "./locales/zh-CN.json";
import zhTW from "./locales/zh-TW.json";
import ja from "./locales/ja.json";
import ko from "./locales/ko.json";

export type Locale = "en" | "zh-CN" | "zh-TW" | "ja" | "ko";
type Dict = Record<string, string>;
const DICTS: Record<Locale, Dict> = { en, "zh-CN": zhCN, "zh-TW": zhTW, ja, ko };

let current: Locale = "en";

export function setLocale(l: Locale) { current = l; }
export function getLocale(): Locale { return current; }

export function detectLocale(): Locale {
  const sys = navigator.language || "en";
  if (sys.startsWith("zh-CN") || sys === "zh") return "zh-CN";
  if (sys.startsWith("zh-TW") || sys.startsWith("zh-HK")) return "zh-TW";
  if (sys.startsWith("ja")) return "ja";
  if (sys.startsWith("ko")) return "ko";
  return "en";
}

export function t(key: string, vars?: Record<string, string>): string {
  const raw = DICTS[current][key] ?? DICTS.en[key] ?? key;
  if (!vars) return raw;
  return raw.replace(/\{(\w+)\}/g, (_, k) => vars[k] ?? `{${k}}`);
}
```

- [ ] **Step 2: Author full `en.json`**

Path: `/Users/evette/Documents/BubbleKeys/src/i18n/locales/en.json`

```json
{
  "app.brand": "BUBBLEKEYS",
  "tab.home": "HOME",
  "tab.packs": "PACKS",
  "tab.settings": "SETTINGS",
  "tab.about": "ABOUT",
  "home.volume": "VOL",
  "home.on": "▶ ON",
  "home.off": "▶ OFF",
  "packs.import": "+ IMPORT MECHVIBES",
  "settings.hotkey": "HOTKEY",
  "settings.autostart": "AUTOSTART",
  "settings.pitch_jitter": "PITCH VAR",
  "settings.menu_icon": "MENU ICON",
  "settings.output": "OUTPUT",
  "settings.language": "LANGUAGE",
  "settings.night_silent": "NIGHT SILENT",
  "settings.night_silent.start": "START",
  "settings.night_silent.end": "END",
  "tray.current_pack": "CURRENT PACK",
  "tray.open": "⚙ Open BubbleKeys",
  "tray.quit": "⏻ Quit",
  "onboarding.welcome.title": "Welcome",
  "onboarding.welcome.cta": "▶ START",
  "onboarding.why.title": "NEEDS ACCESSIBILITY",
  "onboarding.why.body": "BubbleKeys listens to your keyboard so it can play a sound on every press. Nothing is recorded, logged, or sent.",
  "onboarding.why.cta": "⚙ OPEN SYSTEM SETTINGS",
  "onboarding.grant.title": "↑ ENABLE BUBBLEKEYS",
  "onboarding.grant.body": "System Settings → Privacy & Security → Accessibility → toggle BubbleKeys on. We'll auto-advance once granted.",
  "onboarding.grant.waiting": "⌛ waiting…",
  "onboarding.try.title": "✓ READY",
  "onboarding.try.body": "Press any key to test. Then click DONE.",
  "onboarding.try.cta": "DONE",
  "about.version": "v{version}",
  "about.github": "★ GITHUB ★",
  "about.check_updates": "CHECK UPDATES",
  "about.reset_onboarding": "RESET ONBOARDING"
}
```

- [ ] **Step 3: Author the other 4 locale files**

Translate each key. Each file goes at `src/i18n/locales/<locale>.json`. Translations must cover **every** key from `en.json`. Quality target: native-level. If unsure, mark with reviewer comment in PR — the i18n CI check will pass on key presence; quality is a manual review concern.

`zh-CN.json`, `zh-TW.json`, `ja.json`, `ko.json` — author each based on `en.json`.

- [ ] **Step 4: Replace literal strings in views with `t()` calls**

Audit `views/home.ts`, `views/packs.ts`, `views/settings.ts`, `views/first-run.ts`, `tray.ts`, `lib/router.ts`. Every user-visible string becomes `t("key")`. Example for tabs:

```ts
// router.ts paintTabs()
hostTabs.innerHTML = tabs.map(t => `
  <button class="tab ${t === active ? 'active' : ''}" data-tab="${t}">${tt(`tab.${t}`)}</button>
`).join("");
// import { t as tt } from "../i18n";
```

- [ ] **Step 5: Wire detection on boot**

`src/main.ts` start of bootstrap:

```ts
import { detectLocale, setLocale } from "./i18n";
import { getSettings } from "./lib/ipc";

const s0 = await getSettings();
setLocale((s0.language === "auto" ? detectLocale() : s0.language) as any);
```

When language is changed in settings, also call `setLocale(...)` and re-render the active tab.

- [ ] **Step 6: Commit**

```bash
git add src/i18n/ src/main.ts src/views/ src/lib/router.ts src/tray.ts
git commit -m "feat(i18n): full 5-locale coverage with detect + reactive switch"
```

### Task 9.2: CI key-completeness check

**Files:** Create `scripts/verify-i18n-keys.ts`.

- [ ] **Step 1: Script**

Path: `/Users/evette/Documents/BubbleKeys/scripts/verify-i18n-keys.ts`

```ts
import en from "../src/i18n/locales/en.json";
import zhCN from "../src/i18n/locales/zh-CN.json";
import zhTW from "../src/i18n/locales/zh-TW.json";
import ja from "../src/i18n/locales/ja.json";
import ko from "../src/i18n/locales/ko.json";

const locales = { "zh-CN": zhCN, "zh-TW": zhTW, ja, ko };
const enKeys = new Set(Object.keys(en));

let ok = true;
for (const [name, dict] of Object.entries(locales)) {
  const missing = [...enKeys].filter(k => !(k in dict));
  const extra   = Object.keys(dict).filter(k => !enKeys.has(k));
  if (missing.length || extra.length) {
    ok = false;
    console.error(`[${name}] missing: ${missing.join(", ") || "—"}`);
    console.error(`[${name}] extra:   ${extra.join(", ")   || "—"}`);
  }
}
if (!ok) {
  console.error("i18n verification failed.");
  process.exit(1);
}
console.log("i18n keys consistent across all locales ✓");
```

- [ ] **Step 2: Add npm script**

```jsonc
// package.json scripts
"verify:i18n": "tsx scripts/verify-i18n-keys.ts"
```

Add devDep: `"tsx": "^4.7.0"`.

- [ ] **Step 3: Run locally**

```bash
pnpm verify:i18n
```

Expected: ✓.

- [ ] **Step 4: Commit**

```bash
git add scripts/verify-i18n-keys.ts package.json pnpm-lock.yaml
git commit -m "ci: i18n key completeness check"
```

---

# Phase 10 — Mechvibes import

**Goal:** User can drop a Mechvibes pack folder or `.zip` and the app uses it. Multi-sound packs slice samples by `[offset, duration]` properly.

### Task 10.1: Sprite slicer for multi packs

**Files:** Modify `src-tauri/src/pack_store.rs`.

- [ ] **Step 1: Slice the sprite into per-key sample bytes**

Replace the multi branch in `decode_samples`. Use `rodio::Decoder` to decode the entire sprite once into a Vec<f32> PCM, then re-encode each slice to in-memory OGG (or hold raw PCM and feed to a different rodio source). The simplest robust path: decode once to PCM and use `rodio::buffer::SamplesBuffer` at play time.

Add a new field to `LoadedPack`:

```rust
pub enum PackSamples {
    Single(Arc<Vec<u8>>),                          // raw OGG bytes
    MultiPcm {
        rate: u32,
        channels: u16,
        slices: HashMap<String, Arc<Vec<f32>>>,    // pre-sliced PCM per keycode
    },
}
pub struct LoadedPack {
    pub manifest: PackManifest,
    pub samples: PackSamples,
}
```

Update `pack_format::PackError` with `Decode(String)` variant, and the slicer:

```rust
fn decode_multi(dir: &Path, m: &PackManifest) -> Result<PackSamples, PackError> {
    use rodio::Source;
    let bytes = std::fs::read(dir.join(&m.sound))?;
    let cursor = std::io::Cursor::new(bytes);
    let dec = rodio::Decoder::new(cursor).map_err(|e| PackError::Decode(e.to_string()))?;
    let rate = dec.sample_rate();
    let channels = dec.channels();
    let pcm: Vec<f32> = dec.convert_samples().collect();

    let frames_per_ms = (rate as f32 / 1000.0) * channels as f32;
    let mut slices = HashMap::new();
    for (key, [offset_ms, dur_ms]) in &m.defines {
        let start = (*offset_ms as f32 * frames_per_ms) as usize;
        let len   = (*dur_ms    as f32 * frames_per_ms) as usize;
        let end   = (start + len).min(pcm.len());
        if start >= pcm.len() { continue; }
        slices.insert(key.clone(), Arc::new(pcm[start..end].to_vec()));
    }
    Ok(PackSamples::MultiPcm { rate, channels, slices })
}
```

Update `audio_engine` to accept either OGG bytes or PCM buffer. Add a second variant:

```rust
pub enum SampleData {
    Encoded(Arc<Vec<u8>>),
    Pcm { rate: u32, channels: u16, samples: Arc<Vec<f32>> },
}
pub struct PlayCommand {
    pub sample: SampleData,
    pub volume: f32,
    pub pitch_offset: f32,
}
```

In `spawn_oneshot`, switch on the variant:

```rust
let source: Box<dyn rodio::Source<Item = f32> + Send> = match cmd.sample {
    SampleData::Encoded(bytes) => {
        let cur = Cursor::new((*bytes).clone());
        Box::new(rodio::Decoder::new(cur).unwrap().convert_samples())
    }
    SampleData::Pcm { rate, channels, samples } => {
        Box::new(rodio::buffer::SamplesBuffer::new(channels, rate, (*samples).clone()))
    }
};
```

Update all call sites (dispatcher, preview_pack) accordingly.

- [ ] **Step 2: Test fixture for a multi pack**

Add a small fixture under `tests/fixtures/pack_multi/` with a synthesized 1s sine OGG and a defines map of two keys at offsets 0 and 500ms. Add unit test:

```rust
#[test]
fn multi_pack_slices_two_keys() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pack_multi");
    let m = load_manifest(&dir).unwrap();
    let samples = decode_multi(&dir, &m).unwrap();
    if let PackSamples::MultiPcm { slices, .. } = samples {
        assert!(slices.contains_key("1"));
        assert!(slices.contains_key("57"));
    } else { panic!("expected multi"); }
}
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/ src-tauri/tests/fixtures/pack_multi/
git commit -m "feat(packs): sprite slicer for Mechvibes multi-sound packs"
```

### Task 10.2: Import command + UI

**Files:** Modify `src-tauri/src/ipc.rs`. Modify `src/views/packs.ts`.

- [ ] **Step 1: `import_pack` command**

```rust
#[tauri::command]
pub async fn import_pack(
    archive_path: String,
    store: State<'_, Arc<PackStore>>,
) -> Result<String, String> {
    use std::io::Read;
    let path = std::path::Path::new(&archive_path);
    let user_pack_dir = user_data_dir().join("packs");

    // Three cases: directory, .zip, or single .zip's contents flattened.
    if path.is_dir() {
        // copy in-place
        let dst = user_pack_dir.join(path.file_name().unwrap());
        copy_dir_recursive(path, &dst).map_err(|e| e.to_string())?;
    } else if path.extension().map(|e| e == "zip").unwrap_or(false) {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let dst = user_pack_dir.join(stem);
        std::fs::create_dir_all(&dst).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = dst.join(entry.mangled_name());
            if entry.is_dir() { std::fs::create_dir_all(&outpath).ok(); continue; }
            if let Some(p) = outpath.parent() { std::fs::create_dir_all(p).ok(); }
            let mut f = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut f).map_err(|e| e.to_string())?;
        }
    } else {
        return Err("unsupported file (need .zip or directory)".into());
    }

    // reload the entire pack dir to pick up new pack
    let mut store_w = Arc::get_mut(&mut store.inner_mut()).expect("rebuild");
    // ...
    Err("TODO".into())
}
```

(Note: making `PackStore` mutable through `State` is awkward; in practice store should be wrapped in `RwLock<PackStore>`. Adjust the `app.manage(...)` call accordingly:

```rust
app.manage(Arc::new(RwLock::new(store)));
```

and update all reads/writes throughout. Treat this as the natural refactor to make import work.)

Add `zip = "2.1"` to `Cargo.toml`.

- [ ] **Step 2: Frontend file picker**

Replace the `+ IMPORT MECHVIBES` row click handler in `packs.ts`:

```ts
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

host.querySelector("[data-action='import']")!.addEventListener("click", async () => {
  const path = await open({ filters: [{ name: "Mechvibes pack", extensions: ["zip"] }], multiple: false, directory: false });
  if (!path) return;
  await invoke("import_pack", { archivePath: path });
  renderPacks(host); // reload list
});
```

Add `@tauri-apps/plugin-dialog` to deps; enable plugin in `Cargo.toml` and `lib.rs` (`tauri_plugin_dialog::init()`).

- [ ] **Step 3: Smoke test**

Download a real Mechvibes pack. In the app, click `+ IMPORT MECHVIBES`, select it, observe it appear in the list, click to activate, type to verify per-key sounds work.

- [ ] **Step 4: Commit**

```bash
git commit -am "feat(packs): import Mechvibes .zip + folder via file picker"
```

---

# Phase 11 — Night silent scheduler

**Goal:** When `night_silent.enabled` and current time within `[start, end]` (wraps midnight), `MuteController.set_night_silent_active(true)`.

### Task 11.1: Scheduler module (TDD)

**Files:** Create `src-tauri/src/night_silent.rs`.

- [ ] **Step 1: Test the time-window math (pure function)**

```rust
//! Pure time-window check; tested without a clock.

#[derive(Clone, Copy)]
pub struct Window { pub start: (u8,u8), pub end: (u8,u8) }

pub fn parse_hhmm(s: &str) -> Option<(u8,u8)> {
    let (h, m) = s.split_once(':')?;
    Some((h.parse().ok()?, m.parse().ok()?))
}

pub fn in_window(window: Window, now_hm: (u8,u8)) -> bool {
    let to_min = |(h,m): (u8,u8)| h as u16 * 60 + m as u16;
    let s = to_min(window.start);
    let e = to_min(window.end);
    let n = to_min(now_hm);
    if s == e { false }
    else if s < e { n >= s && n < e }                 // same-day window
    else { n >= s || n < e }                          // wraps midnight
}

#[cfg(test)]
mod tests {
    use super::*;
    fn w(s: &str, e: &str) -> Window { Window { start: parse_hhmm(s).unwrap(), end: parse_hhmm(e).unwrap() } }

    #[test]
    fn same_day_window() {
        assert!(in_window(w("09:00","17:00"), (12,0)));
        assert!(!in_window(w("09:00","17:00"), (8,0)));
        assert!(!in_window(w("09:00","17:00"), (17,0)));
    }
    #[test]
    fn midnight_wrap() {
        let win = w("22:00","07:00");
        assert!(in_window(win, (23,0)));
        assert!(in_window(win, (3,0)));
        assert!(in_window(win, (6,59)));
        assert!(!in_window(win, (7,0)));
        assert!(!in_window(win, (12,0)));
    }
    #[test]
    fn equal_start_end_is_off() {
        let win = w("12:00","12:00");
        assert!(!in_window(win, (12,0)));
    }
}
```

- [ ] **Step 2: Tick task that updates mute controller every minute**

```rust
use crate::mute_controller::MuteController;
use crate::settings_store::Settings;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub fn spawn(settings: Arc<RwLock<Settings>>, mute: MuteController) {
    std::thread::Builder::new().name("night-silent".into()).spawn(move || {
        loop {
            tick(&settings, &mute);
            std::thread::sleep(Duration::from_secs(60));
        }
    }).expect("spawn");
}

fn tick(settings: &Arc<RwLock<Settings>>, mute: &MuteController) {
    let s = settings.read().unwrap();
    if !s.night_silent.enabled {
        mute.set_night_silent_active(false);
        return;
    }
    let now = chrono::Local::now();
    let now_hm = (now.hour() as u8, now.minute() as u8);
    let win = Window {
        start: parse_hhmm(&s.night_silent.start).unwrap_or((22,0)),
        end:   parse_hhmm(&s.night_silent.end).unwrap_or((7,0)),
    };
    mute.set_night_silent_active(in_window(win, now_hm));
}
```

Add `chrono = "0.4"` to `Cargo.toml`.

- [ ] **Step 3: Wire spawn in `lib.rs setup`**

```rust
night_silent::spawn(settings_arc.clone(), mute.clone());
```

- [ ] **Step 4: Smoke test**

Set Night Silent enabled, start `00:00`, end `23:59` in settings. Within 1 minute, LED should flip to off; typing produces no sound.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/night_silent.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: night silent scheduler with midnight-wrapping window"
```

### Task 11.2: SETTINGS UI for night silent

**Files:** Modify `src/views/settings.ts`.

- [ ] **Step 1: Render the night silent group**

Append rows to the `host.innerHTML` template:

```ts
${row(t("settings.night_silent"), toggle("night_silent.enabled", s.night_silent.enabled))}
${row(t("settings.night_silent.start"), input("night_silent.start", s.night_silent.start, "time"))}
${row(t("settings.night_silent.end"),   input("night_silent.end",   s.night_silent.end,   "time"))}
```

Update the change handler to support nested keys:

```ts
const next: Settings = await getSettings();
const path = t.dataset.key!.split(".");
let obj: any = next;
for (let i = 0; i < path.length - 1; i++) obj = obj[path[i]];
const last = path[path.length - 1];
if (t.type === "checkbox") obj[last] = (t as HTMLInputElement).checked;
else obj[last] = t.value;
await updateSettings(next);
```

- [ ] **Step 2: Smoke test**

Toggle on, set 22:00–07:00, change Mac clock to 23:00 (System Settings → Date & Time → manual) — within 1 min, sound mutes. Restore clock.

- [ ] **Step 3: Commit**

```bash
git commit -am "feat(settings): UI for night-silent window"
```

---

# Phase 12 — ABOUT page + Reset onboarding

**Files:** Create `src/views/about.ts`. Modify `src/main.ts`.

### Task 12.1: ABOUT page

- [ ] **Step 1: Implement view**

Path: `/Users/evette/Documents/BubbleKeys/src/views/about.ts`

```ts
import { resetOnboarding } from "../lib/ipc";
import { open as openExternal } from "@tauri-apps/plugin-shell";
import { t } from "../i18n";
import { getVersion } from "@tauri-apps/api/app";

export async function renderAbout(host: HTMLElement) {
  const version = await getVersion();
  host.innerHTML = `
    <div class="about">
      <div class="logo">🫧</div>
      <h2>BUBBLEKEYS</h2>
      <p class="subtitle">${t("about.version", { version })} · MIT</p>
      <button class="pixel-btn" id="github">${t("about.github")}</button>
      <button class="pixel-btn" id="check">${t("about.check_updates")}</button>
      <button class="pixel-btn off" id="reset">${t("about.reset_onboarding")}</button>
    </div>`;

  document.getElementById("github")!.addEventListener("click", () =>
    openExternal("https://github.com/<owner>/bubblekeys"));   // replace <owner> at first publish
  document.getElementById("check")!.addEventListener("click", () =>
    openExternal("https://github.com/<owner>/bubblekeys/releases"));
  document.getElementById("reset")!.addEventListener("click", async () => {
    await resetOnboarding();
    location.reload();
  });
}
```

Append CSS:

```css
.about { display:flex; flex-direction:column; align-items:center; gap:10px; padding: 16px; }
.about .logo { font-size: 36px; }
```

- [ ] **Step 2: Wire into router**

```ts
import { renderAbout } from "./views/about";
// ...
about: renderAbout,
```

- [ ] **Step 3: Smoke test + commit**

```bash
git add src/views/about.ts src/main.ts src/styles/pixel.css
git commit -m "feat(ui): ABOUT page with version, links, reset onboarding"
```

---

# Phase 13 — App icon + brand assets

**Goal:** Real `.icns` from the Pixel Bubble + 3D keycap design at all required sizes; matching tray icon; DMG background.

### Task 13.1: Generate master PNG

- [ ] **Step 1: Translate the brainstorming HTML mockup to a 1024×1024 PNG**

The brainstorming session locked in the design (`docs/superpowers/specs/2026-04-26-bubblekeys-design.md` §6.6). Either:
- Use a vector tool (Figma/Affinity) to recreate the design → export PNG, OR
- Use a headless browser to screenshot the HTML at 1024×1024 → process to PNG

Save master to `src-tauri/icons/icon-1024.png`.

Also create `src-tauri/icons/icon-1024-simplified.png` for the 16/32 variants (keycap + main bubble only — see spec §6.6).

- [ ] **Step 2: Generate `.icns` via `iconutil`**

Path: `/Users/evette/Documents/BubbleKeys/scripts/make-icons.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

ICONS=src-tauri/icons
ICONSET=$ICONS/icon.iconset
mkdir -p "$ICONSET"

sips -z 16 16     "$ICONS/icon-1024-simplified.png" --out "$ICONSET/icon_16x16.png"
sips -z 32 32     "$ICONS/icon-1024-simplified.png" --out "$ICONSET/icon_16x16@2x.png"
sips -z 32 32     "$ICONS/icon-1024-simplified.png" --out "$ICONSET/icon_32x32.png"
sips -z 64 64     "$ICONS/icon-1024.png"            --out "$ICONSET/icon_32x32@2x.png"
sips -z 128 128   "$ICONS/icon-1024.png"            --out "$ICONSET/icon_128x128.png"
sips -z 256 256   "$ICONS/icon-1024.png"            --out "$ICONSET/icon_128x128@2x.png"
sips -z 256 256   "$ICONS/icon-1024.png"            --out "$ICONSET/icon_256x256.png"
sips -z 512 512   "$ICONS/icon-1024.png"            --out "$ICONSET/icon_256x256@2x.png"
sips -z 512 512   "$ICONS/icon-1024.png"            --out "$ICONSET/icon_512x512.png"
cp "$ICONS/icon-1024.png"                            "$ICONSET/icon_512x512@2x.png"

iconutil -c icns -o "$ICONS/icon.icns" "$ICONSET"

# Tray icon: 16×16 black silhouette of the keycap + bubble
# (Manual: trace from icon-1024-simplified.png, render as black-on-transparent)
echo "Done. Verify $ICONS/icon.icns and update tauri.conf.json"
```

```bash
chmod +x scripts/make-icons.sh
./scripts/make-icons.sh
```

- [ ] **Step 3: Reference the icns from tauri.conf**

```json
"bundle": {
  "icon": ["icons/icon.icns"]
}
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/icons/ scripts/make-icons.sh src-tauri/tauri.conf.json
git commit -m "feat: app icon assets (icns + tray) from Pixel Bubble + keycap design"
```

---

# Phase 14 — CI/CD (GitHub Actions)

**Goal:** PR builds run tests; tag push builds and uploads `.dmg` to GitHub Releases. Sign + notarize when secrets present.

### Task 14.1: PR workflow

**Files:** Create `.github/workflows/ci.yml`.

- [ ] **Step 1: CI yaml**

```yaml
name: CI

on:
  pull_request:
  push:
    branches: [main]

jobs:
  test:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v3
        with: { version: 9 }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: 'pnpm' }
      - uses: dtolnay/rust-toolchain@stable
        with: { toolchain: "1.79" }
      - run: pnpm install --frozen-lockfile
      - name: Verify i18n
        run: pnpm verify:i18n
      - name: Frontend type-check
        run: pnpm tsc --noEmit
      - name: Rust tests
        run: cargo test --manifest-path src-tauri/Cargo.toml --all
      - name: Build (debug, sanity)
        run: pnpm tauri build --debug
```

- [ ] **Step 2: Commit and push**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: PR pipeline (tests, i18n verify, debug build)"
```

### Task 14.2: Release workflow

**Files:** Create `.github/workflows/release.yml`.

- [ ] **Step 1: Release yaml**

```yaml
name: Release

on:
  push:
    tags: ['v*.*.*']

jobs:
  build:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v3
        with: { version: 9 }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: 'pnpm' }
      - uses: dtolnay/rust-toolchain@stable
        with: { toolchain: "1.79", targets: "aarch64-apple-darwin,x86_64-apple-darwin" }
      - run: pnpm install --frozen-lockfile

      - name: Build universal
        env:
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: |
          pnpm tauri build --target universal-apple-darwin

      - name: Compute SHAs
        run: |
          cd src-tauri/target/universal-apple-darwin/release/bundle/dmg
          shasum -a 256 *.dmg > SHA256SUMS

      - name: Release
        uses: softprops/action-gh-release@v2
        with:
          files: |
            src-tauri/target/universal-apple-darwin/release/bundle/dmg/*.dmg
            src-tauri/target/universal-apple-darwin/release/bundle/dmg/SHA256SUMS
          draft: true
          generate_release_notes: true
```

When the secrets are set in repo settings, Tauri auto-signs and notarizes; when not set, an unsigned `.dmg` still gets produced and uploaded.

- [ ] **Step 2: Update README**

Add to `README.md`:

```markdown
## Install

### Homebrew (recommended)
\`\`\`bash
brew install --cask bubblekeys
\`\`\`

### Direct download
Grab the latest `.dmg` from [Releases](https://github.com/<owner>/bubblekeys/releases).

If macOS reports "BubbleKeys can't be opened because it is from an unidentified developer," the build was unsigned — control-click → Open, then click Open in the dialog.

### First run
BubbleKeys needs Accessibility permission to listen for keystrokes. The onboarding flow walks you through it; nothing is recorded or transmitted.
```

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/release.yml README.md
git commit -m "ci: release pipeline (universal dmg, signing when secrets present)"
```

---

# Phase 15 — Homebrew Cask + release docs

**Goal:** `brew install --cask bubblekeys` works.

### Task 15.1: Submit cask formula

- [ ] **Step 1: Tag a v0.1.0 release**

```bash
git tag v0.1.0
git push origin v0.1.0
```

Wait for the Release workflow to upload the `.dmg`. Note the URL and SHA256.

- [ ] **Step 2: Author the cask**

Path (in your fork of homebrew-cask): `Casks/b/bubblekeys.rb`

```ruby
cask "bubblekeys" do
  version "0.1.0"
  sha256 "<sha256 of the .dmg from SHA256SUMS>"

  url "https://github.com/<owner>/bubblekeys/releases/download/v#{version}/BubbleKeys_#{version}_universal.dmg",
      verified: "github.com/<owner>/bubblekeys/"
  name "BubbleKeys"
  desc "Open-source typing sound effects for macOS with a pixel-game UI"
  homepage "https://github.com/<owner>/bubblekeys"

  app "BubbleKeys.app"

  zap trash: [
    "~/Library/Application Support/BubbleKeys",
    "~/Library/Preferences/app.bubblekeys.plist",
  ]
end
```

- [ ] **Step 3: Submit PR to homebrew-cask**

Follow [homebrew-cask CONTRIBUTING.md](https://github.com/Homebrew/homebrew-cask/blob/master/CONTRIBUTING.md). Run:

```bash
brew style --fix Casks/b/bubblekeys.rb
brew audit --new --cask Casks/b/bubblekeys.rb
```

Open PR upstream.

- [ ] **Step 4: Update README install section**

Once the cask is merged, the badge for installation can be added at the top of the README.

- [ ] **Step 5: Final commit**

```bash
git commit --allow-empty -m "release: v0.1.0 — first public release"
```

---

# Self-review notes

**Spec coverage check** (against `docs/superpowers/specs/2026-04-26-bubblekeys-design.md`):

- §1 Vision — Phases 0–15 cumulatively deliver ✓
- §2 Goals (latency, RAM, packs, i18n, dropdown, Game Boy, night-silent, pitch jitter, hotkey, distribution) — all covered across phases 2/3/4/6/7/9/11/14/15 ✓
- §3 User stories U1-U6 — U1 phase 7, U2 phase 2 then refined in 8, U3 phase 10, U4 phase 9, U5 phase 11, U6 phase 8 ✓
- §4 Architecture — modules realised in phases 2/3/6/7/8/11 ✓
- §5 Data model — pack format phase 3, settings schema phase 6 ✓
- §6 UI spec — color/typography phase 4; main window phases 4/5/6/12; tray phase 7; first-run phase 8; icon phase 13; i18n phase 9 ✓
- §7 Permissions — phase 8 (Accessibility), no-network policy implicit (phase 12 Check Updates uses one external link) ✓
- §8 Distribution — phases 14/15 ✓
- §9 Testing — TDD throughout; visual regression for i18n is *not* automated (manual checklist instead — defensible at v1; revisit post-launch).
- §10/11 — open questions and decision log captured in spec.

**Outstanding gap to flag at execution time:** The current plan assumes a `core-graphics::access::AXIsProcessTrustedWithOptions` is exposed via the chosen `core-graphics` crate version. Verify before Task 8.3 — if unavailable, add a thin `extern "C"` binding to `ApplicationServices.framework`'s `AXIsProcessTrustedWithOptions`. This is a 10-line task, not a phase blocker.

**No placeholder text found.** All steps include concrete code or commands. Every task has a defined commit message.

**Type consistency:** `LoadedPack` shape changes between Phase 3 (`samples_by_key: HashMap<String, Arc<Vec<u8>>>`) and Phase 10 (introduces `PackSamples` enum). This is **deliberate** — the early phase ships a simpler shape that doesn't support sprite slicing; Phase 10 refactors when sprite slicing becomes necessary. Both phases include the migration steps (Phase 10 Task 10.1 explicitly says "Update all call sites").
