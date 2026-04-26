# BubbleKeys — Design Document

**Date:** 2026-04-26
**Status:** Draft, pending user review
**Project:** BubbleKeys — open-source Mac typing sound effect app with pixel-game UI

---

## 1. Purpose & Vision

BubbleKeys plays a configurable sound on every keystroke so a quiet keyboard can sound like a mechanical one — or a stream of bubbles, or anything the user imports. It is distributed open-source on macOS, with a deliberately distinctive **pixel-game aesthetic** (handheld-console main window + menu-bar dropdown remote).

The vision is *not* "another Mechvibes clone." The bet is that **personality + polish + Mac-native feel** can carve out an audience even in a category with prior art.

## 2. Goals & Non-goals

### Goals (v1)
- Play accurate typing sounds with **< 10ms latency**, **< 50MB RAM** idle.
- Ship 4 default packs (Cherry Blue / Red / Brown + original Bubbles) on first install.
- **Mechvibes-format compatibility** so users can drop in any of the ~1000+ existing community packs.
- **5-language UI** (en / zh-CN / zh-TW / ja / ko) — including pixel-styled CJK rendering.
- Always-on **menu-bar dropdown** for 1-second toggle/switch/volume.
- **Game Boy-shaped main window** for setup, browsing packs, settings, about.
- **Night-silent** scheduling (user-defined time range auto-mutes).
- **Per-keystroke pitch jitter** to avoid mechanical repetition.
- **Global hotkey** to toggle on/off without focus.
- Distributed via **GitHub Releases** + **Homebrew Cask**.

### Non-goals (explicitly deferred to post-v1)
- Drag-and-drop arbitrary folder as custom pack (Mechvibes import covers this).
- In-app microphone recording mode.
- Per-app enable/disable rules.
- Daily/weekly typing statistics.
- Windows / Linux ports (architecture allows it later, but not v1).
- Mac App Store distribution (sandbox forbids global keyboard listening).

### Success criteria
- A new user can install → grant Accessibility → hear sounds within **60 seconds**.
- Running in background uses **< 0.5% CPU** while idle, **< 2% CPU** while typing actively.
- Menu-bar dropdown lets a user mute mid-call in **< 1 second** without opening the main window.
- All 4 default packs render and play correctly without internet.
- All UI strings render without overflow in the longest of the 5 languages.

## 3. User Stories

| # | Story | Where it's served |
|---|-------|------|
| U1 | As a remote worker on a Zoom call, I tap the menu-bar icon to mute typing sound in <1s. | Menu bar dropdown |
| U2 | As a new user, I install the app and start hearing a sound the first time I press a key, without reading docs. | First-run flow |
| U3 | As a Mechvibes user with a favorite pack, I can import it and use it identically. | PACKS page → Import button |
| U4 | As a non-English speaker, I get the UI in my system language automatically. | i18n auto-detect on first launch |
| U5 | As a night user, I want the app to auto-mute between 22:00 and 07:00 so I don't wake roommates. | SETTINGS page → Night silent |
| U6 | As a privacy-conscious user, I want to know exactly why the app needs Accessibility permission. | First-run flow step 2 |

## 4. Architecture

### 4.1 Tech stack

- **Tauri 2** — Rust backend + Web frontend, single `.app` bundle (~10–15 MB). Signing strategy is described in §8.
- **Rust backend** for: global keyboard listener, audio playback engine, sound-pack loading, settings persistence, IPC.
- **Web frontend** for: all UI (Game Boy main window + menu-bar dropdown). Framework: **Vanilla TypeScript + lightweight reactive state** (e.g., nanostores). React/Vue is unnecessary at this scale and adds bundle weight.
- **Audio engine:** Rust crate `cpal` for low-latency output + `rodio` for decoded sample playback. Sound files preloaded into memory at pack-switch time.
- **Keyboard listener:** macOS `CGEventTap` via Rust crate `core-graphics` (already used by similar projects). Requires Accessibility permission.

### 4.2 Process model

```
┌─────────────────────────────────────────────────┐
│  BubbleKeys.app (single Tauri process)          │
│                                                 │
│  ┌────────────┐    IPC     ┌─────────────────┐  │
│  │  Frontend  │  <───────> │  Rust Backend   │  │
│  │  (WebView) │            │                 │  │
│  └────────────┘            │  ┌────────────┐ │  │
│                            │  │ KeyListener│ │  │
│                            │  │ (CGEventTap)│ │  │
│                            │  └─────┬──────┘ │  │
│                            │        │        │  │
│                            │        ▼        │  │
│                            │  ┌────────────┐ │  │
│                            │  │ AudioEngine│ │  │
│                            │  │ (cpal)     │ │  │
│                            │  └────────────┘ │  │
│                            │                 │  │
│                            │  ┌────────────┐ │  │
│                            │  │ Settings + │ │  │
│                            │  │ PackStore  │ │  │
│                            │  └────────────┘ │  │
│                            └─────────────────┘  │
│                                                 │
│  Two webview windows:                           │
│   • Main (Game Boy, 320×480, hidden by default) │
│   • MenuBar dropdown (280×360)                  │
└─────────────────────────────────────────────────┘
```

The **keyboard-listener thread is independent of the UI lifecycle** — closing the main window does not affect playback. Only quit (from menu) tears down the listener.

### 4.3 Modules

Each module has a clear single purpose, well-defined inputs/outputs, and is testable in isolation.

| Module | Responsibility | Depends on |
|---|---|---|
| `key_listener` | Watches global keystrokes via `CGEventTap`. Emits `KeyEvent { keycode, kind: down/up }` over an internal channel. Handles permission errors. | macOS Accessibility |
| `audio_engine` | Owns `cpal` output stream. Receives `PlayCommand { sample, volume, pitch_offset }`. Mixes/decodes/plays. | `cpal`, `rodio` |
| `pack_store` | Loads sound packs from disk into memory. Exposes `get_active_pack() → Pack`. Watches a packs directory. Imports Mechvibes JSON+ogg. | `pack_format` |
| `pack_format` | Parses Mechvibes-format `config.json` + sprite sheets. Validates. | — |
| `dispatcher` | Glue: receives `KeyEvent` + active pack + settings, computes which sample to play (per-key vs single-sound packs), applies pitch jitter, sends to `audio_engine`. Honors mute state. | all of the above |
| `settings_store` | Persists user preferences to `~/Library/Application Support/BubbleKeys/settings.json`. | — |
| `mute_controller` | Owns the on/off state. Inputs: hotkey, menu-bar toggle, night-silent schedule. Single source of truth queried by `dispatcher`. | `settings_store` |
| `ipc_bridge` | Tauri command handlers. Exposes safe interface to frontend (get/set settings, list packs, switch pack, toggle mute, etc.). | all of the above |
| `tray` | Menu-bar icon + dropdown window lifecycle. | `tauri::tray` |
| `frontend/router` | 4-page tab navigation in main window. | — |
| `frontend/i18n` | Language detection, string lookup, layout flexing. | — |
| `frontend/views/*` | Home, Packs, Settings, About pages + Menu-bar dropdown + First-run wizard. | router, i18n |

### 4.4 Data flow (typing → sound)

1. macOS dispatches a key event.
2. `key_listener` (running on a CFRunLoop thread) receives it via `CGEventTap` callback. **Hot path: must return quickly.**
3. It posts `KeyEvent` to `dispatcher` over a non-blocking channel.
4. `dispatcher` checks `mute_controller.is_muted()`. If muted, drop.
5. Otherwise, looks up sample in active pack:
   - If pack is **multi-sound** (Mechvibes "defines" map): use `keycode → sample_id`.
   - If pack is **single-sound**: use the one sample.
6. Apply pitch jitter (random ±N cents within configured range).
7. Send `PlayCommand` to `audio_engine`.
8. `audio_engine` mixes into output buffer; sound emerges from speakers.

**Latency budget:** key event → audio out target **< 10ms**. Achievable because samples are pre-decoded and held in RAM; `cpal` callback runs at audio-thread priority.

## 5. Data Model

### 5.1 Sound pack (Mechvibes-compatible)

```jsonc
// pack/config.json
{
  "id": "cherry-blue",
  "name": "Cherry Blue",
  "key_define_type": "multi", // or "single"
  "sound": "sound.ogg",       // sprite sheet (or single file)
  "defines": {                // only for multi
    "1":  [0, 100],            // keycode → [offset_ms, duration_ms]
    "57": [120, 80],           // ...
    "..."
  },
  "includes_numpad": true,
  "icon": "icon.png",          // optional pack-card art
  "tags": ["mechanical", "cherry"],
  "license": "CC0",
  "author": "Mechvibes Community"
}
```

Loading: pack_store walks `~/Library/Application Support/BubbleKeys/packs/<id>/` (default packs unpack here on first run). Mechvibes import = drop a folder or `.zip` in here.

### 5.2 Settings

```jsonc
// ~/Library/Application Support/BubbleKeys/settings.json
{
  "version": 1,
  "active_pack": "cherry-blue",
  "volume": 0.65,            // 0.0–1.0
  "muted": false,
  "pitch_jitter": true,      // bool; range constant for v1
  "hotkey": "cmd+option+b",
  "auto_start": true,
  "menu_icon_visible": true,
  "language": "auto",        // auto | en | zh-CN | zh-TW | ja | ko
  "output_device": "default",
  "night_silent": {
    "enabled": false,
    "start": "22:00",
    "end":   "07:00"
  }
}
```

## 6. UI Specification

### 6.1 Color palette

| Role | Hex |
|---|---|
| Sky / primary panel | `#b5c8f5` |
| Lavender | `#d4b5f5` |
| Pink accent | `#f5b5d4` |
| Mint (success / active) | `#a3e6c5` |
| Ink (lines, text) | `#2d2d5f` |
| Sub-ink (secondary text) | `#5d5d8f` |
| Shadow blue | `#8fb5f5` |

All UI uses these only. Pixel-art shadows are 2–4px hard offsets in `#2d2d5f`, never blurred.

### 6.2 Typography

- Primary font family stack: `"Ark Pixel 12px Monospaced", "Ark Pixel 12px Proportional", "Boutique Bitmap 9x9", "Galmuri11", monospace`.
- Two sizes only: **12px body**, **18–24px headings**.
- All button/row paddings in `em` (not `px`) so text length variance across languages doesn't break layout.
- All text containers use `flex` / auto-width — no fixed-width word slots.

### 6.3 Main window — Game Boy

**Window:** 320×480 logical px, **non-resizable**, no native title bar (custom drag handle at top of pixel frame).

**Frame:** pink (`#f5b5d4`) handheld body with 4px ink border, rounded bottom; 6×6 ink shadow on white desktop. Top status row shows `BUBBLEKEYS` label + LED dot (mint when on, lavender when muted).

**Screen area:** 4 tabs persistent at top (clickable + D-pad ←/→). One main view at a time. **No nested menus, ever.**

#### 6.3.1 HOME page
Hero: current pack name (large, centered) + small art square. Below: volume bar (`←/→` adjusts), ON/OFF pill (mint when on, shows hotkey hint).

#### 6.3.2 PACKS page
Vertical list of installed packs. Selected row highlighted pink, ▶ marker. **Hovering or pressing A on a row plays a 0.5s preview** (using a representative sample for multi-sound packs). Bottom row: dashed `+ IMPORT MECHVIBES` button → opens file picker for `.zip`/folder.

#### 6.3.3 SETTINGS page
List of settings rows (label left, value right). All inline-edit:
- Hotkey (record-on-click)
- Auto-start at login (toggle)
- Pitch variation (toggle)
- Output device (popover list)
- Menu icon visible (toggle)
- **Language** (popover with 5 options: Auto / English / 简体中文 / 繁體中文 / 日本語 / 한국어)
- **Night silent** (toggle + two time pickers; time range wraps midnight)

#### 6.3.4 ABOUT page
Logo + version + license badge + GitHub link button + Check Updates button + Reset Onboarding link (re-runs the first-run flow).

#### 6.3.5 Controls
- D-pad ←/→ : switch tabs
- D-pad ↑/↓ : navigate within current page
- A button : confirm / preview / open
- B button : back / close popover
- **All controls also work via mouse on tabs / list rows / buttons.** D-pad/A/B are redundant, not required.

### 6.4 Menu-bar dropdown

**Window:** 280×360, panel-style, anchored under menu-bar icon. Hidden until icon click; auto-dismisses on focus loss.

Sections top-to-bottom:
1. Header: `🫧 BubbleKeys` + ON/OFF pill (clickable, mint=on).
2. **CURRENT PACK**: 4 most recent packs as clickable rows, ✓ on active.
3. Volume slider with numeric value.
4. Footer: `⚙ Open BubbleKeys` (opens main window) · `⏻ Quit`.

This dropdown is the **daily driver**. It must do mute/switch/volume in 1–2 clicks each.

### 6.5 First-run flow

4 modal-style screens inside the main window (no separate window). Cannot be skipped on the first launch but can be reopened from About > Reset Onboarding later.

1. **WELCOME** — logo, auto-detected language preview, "GET STARTED".
2. **WHY ACCESSIBILITY** — plain-language explanation: "BubbleKeys needs to listen to your keyboard so it can play a sound on every press. We don't record, log, or transmit anything." Button: "OPEN SYSTEM SETTINGS".
3. **PERMISSION GRANT** — App opens `x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility`. Pixel illustration shows where to flip the toggle. Polls for permission grant in background; auto-advances when granted.
4. **TRY IT** — "Tap any key to test." On first detected keystroke, show ✓ READY. Quick pack-switcher (4 default packs). Done → main window.

### 6.6 App icon

`Pixel Bubble + 3D keycap` design at 1024×1024 master, exported for `.icns` at 16/32/64/128/256/512/1024 (each @1x and @2x). 16px variant is auto-simplified (keycap silhouette + main bubble only — no small bubbles, no sparkles).

### 6.7 i18n

- Language detection: read `NSLocale.preferredLanguages` on first launch; map to one of `en / zh-CN / zh-TW / ja / ko`. Fallback `en`.
- Strings live in `frontend/locales/<lang>.json`. One key per UI string, no concatenation in code.
- All 5 locales must be 100% complete before release. CI fails if any locale has missing keys vs `en.json`.
- **Layout test:** automated screenshot diff in 5 languages on every PR for the 4 main pages + dropdown + first-run.

## 7. Permissions, Security, Privacy

- **Accessibility** (required): for `CGEventTap`. Requested via the first-run flow with explanation.
- **No network access** in v1 (Check Updates uses a single GitHub API call; show URL up front).
- Sound packs are loaded as data only; **never executed**. Mechvibes import reads JSON + audio bytes, nothing else.
- Settings stored locally; no telemetry.

## 8. Distribution & Build

- **GitHub Releases** (primary): each git tag triggers GitHub Actions to:
  - Build universal `.dmg` (Intel + Apple Silicon).
  - Signing: if `APPLE_DEVELOPER_ID` and `APPLE_NOTARIZATION_PASSWORD` secrets exist, sign + notarize. Otherwise produce an unsigned build with README instructions for `xattr -dr com.apple.quarantine` / `⌃ + 右键 Open` workaround.
  - In-app updates: ship Sparkle **only when notarized builds become available**. v1 unsigned builds rely on Homebrew bumps + manual download.
  - Upload `.dmg`, `.zip`, and SHA checksums.
- **Homebrew Cask** (secondary): a `bubblekeys.rb` cask formula in homebrew/cask, updated on each release.
- **Mac App Store**: not pursued — sandbox forbids `CGEventTap`.
- License: **MIT** for code; per-pack licenses preserved in pack metadata.

## 9. Testing Strategy

- **Unit tests** (Rust): pack-format parser, pitch-jitter math, mute-state transitions, settings serialization.
- **Integration tests** (Rust): pack loading from a fixture directory; dispatcher correctly routes events; night-silent schedule flips mute.
- **Frontend tests:** i18n key completeness check; visual regression for 4 main pages × 5 languages × light bg.
- **Manual QA checklist** (before each release): first-run on a fresh user account; permission revocation handling; pack import from real Mechvibes pack; hotkey conflict with Spotlight/Mission Control.

## 10. Open Questions / Future Work

- **Sparkle in-app updates**: only viable once builds are notarized. v1 ships without; users update via Homebrew or manual download. Revisit after first stable release.
- **Output device hot-swap** when the user changes their selected device in Settings — confirm `cpal` rebuild-stream cost is fine on user's hardware.
- **Packs marketplace UI** (browsing community packs in-app) — explicit non-goal for v1; might be v2.
- **Recording mode** (record your own physical keyboard into a pack) — v2 candidate.

## 11. Decisions log (from brainstorming session)

| # | Decision | Notes |
|---|---|---|
| D1 | **Tauri** over Electron / native Swift | Smaller bundle than Electron, easier pixel UI than SwiftUI, opens future cross-platform door. |
| D2 | **MVP scope = "Standard"** with Mechvibes import | Skip recording mode, per-app rules, statistics for v1. |
| D3 | **Game Boy main window + menu-bar dropdown** combo | Hero visual + always-on remote. |
| D4 | **4 fixed pages** in main window: HOME / PACKS / SETTINGS / ABOUT | Tabs always visible; D-pad redundant with mouse. |
| D5 | **Project name = BubbleKeys** | Final. |
| D6 | **5 UI languages**: en / zh-CN / zh-TW / ja / ko | Auto-detect default; full layout test in CI. |
| D7 | **Pack preview in PACKS list directly** | Hover/A button plays 0.5s sample. |
| D8 | **Night silent** with user time range | New SETTINGS row. |
| D9 | **Icon = Pixel Bubble + 3D keycap** | 16px auto-simplified. |
| D10 | **Distribution: GitHub Releases + Homebrew Cask**; skip MAS | Sandbox forbids global keyboard listening. |
| D11 | **License MIT for code**; packs keep their own licenses | 3 default mechanical packs sourced from permissively-licensed Mechvibes packs (verify each pack's actual license at vendoring time, must be ≥ CC-BY); Bubbles is original (CC-BY). |
| D12 | **Font: Ark Pixel** with Boutique Bitmap 9x9 / Galmuri 11 fallback | Open source, CJK coverage, consistent style. |
