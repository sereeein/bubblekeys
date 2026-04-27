//! Tauri IPC commands. Frontend ↔ backend boundary.

use std::sync::{Arc, RwLock};
use serde::Serialize;
use tauri::State;

use crate::audio_engine::{AudioEngine, PlayCommand};
use crate::mute_controller::MuteController;
use crate::pack_store::PackStore;
use crate::settings_store::{save as save_settings, Settings};

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
        .filter_map(|id| store.get(&id).map(|p| PackSummary {
            id: p.manifest.id.clone(),
            name: p.manifest.name.clone(),
        }))
        .collect()
}

#[tauri::command]
pub fn set_active_pack(
    id: String,
    active: State<'_, Arc<RwLock<String>>>,
    store: State<'_, Arc<PackStore>>,
    settings: State<'_, Arc<RwLock<Settings>>>,
) -> Result<(), String> {
    if store.get(&id).is_none() {
        return Err(format!("unknown pack: {id}"));
    }
    *active.write().unwrap() = id.clone();
    let snapshot = {
        let mut s = settings.write().unwrap();
        s.active_pack = id;
        s.clone()
    };
    save_settings(&snapshot).map_err(|e| e.to_string())
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
pub fn set_muted(
    muted: bool,
    mute: State<'_, MuteController>,
    settings: State<'_, Arc<RwLock<Settings>>>,
) -> Result<(), String> {
    mute.set_user_muted(muted);
    let snapshot = {
        let mut s = settings.write().unwrap();
        s.muted = muted;
        s.clone()
    };
    save_settings(&snapshot).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_volume(
    volume: f32,
    store: State<'_, Arc<RwLock<f32>>>,
    settings: State<'_, Arc<RwLock<Settings>>>,
) -> Result<(), String> {
    let v = volume.clamp(0.0, 1.0);
    *store.write().unwrap() = v;
    let snapshot = {
        let mut s = settings.write().unwrap();
        s.volume = v;
        s.clone()
    };
    save_settings(&snapshot).map_err(|e| e.to_string())
}

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

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}
