import { invoke } from "@tauri-apps/api/core";

export interface PackSummary { id: string; name: string }
export interface AppState { active_pack: string; muted: boolean; volume: number }

export const listPacks      = ()                  => invoke<PackSummary[]>("list_packs");
export const setActivePack  = (id: string)        => invoke<void>("set_active_pack", { id });
export const getState       = ()                  => invoke<AppState>("get_state");
export const setMuted       = (muted: boolean)    => invoke<void>("set_muted", { muted });
export const setVolume      = (volume: number)    => invoke<void>("set_volume", { volume });
export const previewPack    = (id: string)        => invoke<void>("preview_pack", { id });
