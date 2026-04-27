//! Tauri IPC commands. Frontend ↔ backend boundary.

use std::sync::{Arc, RwLock};
use serde::Serialize;
use tauri::State;

use crate::audio_engine::{AudioEngine, PlayCommand, SampleData};
use crate::mute_controller::MuteController;
use crate::pack_store::{copy_dir_recursive, PackSamples, PackStore};
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
pub fn list_packs(store: State<'_, Arc<RwLock<PackStore>>>) -> Vec<PackSummary> {
    let store = store.read().unwrap();
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
    store: State<'_, Arc<RwLock<PackStore>>>,
    settings: State<'_, Arc<RwLock<Settings>>>,
) -> Result<(), String> {
    if store.read().unwrap().get(&id).is_none() {
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
    store: State<'_, Arc<RwLock<PackStore>>>,
    engine: State<'_, Arc<dyn AudioEngine>>,
) -> Result<(), String> {
    let store_guard = store.read().unwrap();
    let pack = store_guard.get(&id).ok_or_else(|| format!("unknown pack: {id}"))?;
    let sample = match &pack.samples {
        PackSamples::Single(bytes) => SampleData::Encoded(bytes.clone()),
        PackSamples::MultiPcm { rate, channels, slices } => {
            let s = slices.values().next().ok_or("empty pack")?;
            SampleData::Pcm {
                rate: *rate,
                channels: *channels,
                samples: s.clone(),
            }
        }
    };
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

#[tauri::command]
pub fn check_accessibility() -> bool {
    crate::key_listener::ensure_accessibility(false)
}

#[tauri::command]
pub async fn import_pack(
    archive_path: String,
    store: State<'_, Arc<RwLock<PackStore>>>,
) -> Result<String, String> {
    let path = std::path::Path::new(&archive_path);
    let user_pack_dir = crate::user_data_dir().join("packs");
    std::fs::create_dir_all(&user_pack_dir).map_err(|e| e.to_string())?;

    if path.is_dir() {
        let name = path.file_name().ok_or("invalid path")?;
        let dst = user_pack_dir.join(name);
        copy_dir_recursive(path, &dst).map_err(|e| e.to_string())?;
    } else if path.extension().map(|e| e == "zip").unwrap_or(false) {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        let stem = path
            .file_stem()
            .ok_or("invalid file")?
            .to_string_lossy()
            .to_string();
        let dst = user_pack_dir.join(stem);
        std::fs::create_dir_all(&dst).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = dst.join(entry.mangled_name());
            if entry.is_dir() {
                std::fs::create_dir_all(&outpath).ok();
                continue;
            }
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p).ok();
            }
            let mut f = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut f).map_err(|e| e.to_string())?;
        }
    } else {
        return Err("unsupported file (need .zip or directory)".into());
    }

    store
        .write()
        .unwrap()
        .load_dir(&user_pack_dir)
        .map_err(|e| e.to_string())?;
    Ok("imported".into())
}
