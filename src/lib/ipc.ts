import { invoke } from "@tauri-apps/api/core";

export interface PackSummary { id: string; name: string }
export interface AppState { active_pack: string; muted: boolean; volume: number }

export const listPacks      = ()                  => invoke<PackSummary[]>("list_packs");
export const setActivePack  = (id: string)        => invoke<void>("set_active_pack", { id });
export const getState       = ()                  => invoke<AppState>("get_state");
export const setMuted       = (muted: boolean)    => invoke<void>("set_muted", { muted });
export const setVolume      = (volume: number)    => invoke<void>("set_volume", { volume });
export const previewPack    = (id: string)        => invoke<void>("preview_pack", { id });

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
export const getSettings    = ()                  => invoke<Settings>("get_settings");
export const updateSettings = (s: Settings)       => invoke<void>("update_settings", { newSettings: s });
export const showMain       = ()                  => invoke<void>("show_main");
export const quitApp        = ()                  => invoke<void>("quit_app");
