//! Tauri IPC commands. Frontend ↔ backend boundary.

use std::sync::{Arc, RwLock};
use serde::Serialize;
use tauri::State;

use crate::audio_engine::{AudioEngine, PlayCommand};
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
