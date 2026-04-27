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
    pub bundled: bool,
}

/// Picks a destination directory under `base` that doesn't yet exist, suffixing
/// with -2, -3, ... when `stem` collides. Used by import_pack to allow importing
/// the same archive twice without overwriting the previous import.
fn unique_dst(base: &std::path::Path, stem: &str) -> std::path::PathBuf {
    let mut candidate = base.join(stem);
    let mut suffix = 2;
    while candidate.exists() {
        candidate = base.join(format!("{stem}-{suffix}"));
        suffix += 1;
    }
    candidate
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
            bundled: store.is_bundled(&p.manifest.id),
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
pub fn open_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("only https URLs allowed".into());
    }
    std::process::Command::new("open")
        .arg(&url)
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[tauri::command]
pub fn close_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn start_drag(window: tauri::Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_pack(
    archive_path: String,
    custom_name: Option<String>,
    store: State<'_, Arc<RwLock<PackStore>>>,
) -> Result<String, String> {
    let path = std::path::Path::new(&archive_path);
    let user_pack_dir = crate::user_data_dir().join("packs");
    std::fs::create_dir_all(&user_pack_dir).map_err(|e| e.to_string())?;

    let dst = if path.is_dir() {
        let name = path.file_name().ok_or("invalid path")?;
        let stem = name.to_string_lossy();
        let dst = unique_dst(&user_pack_dir, &stem);
        copy_dir_recursive(path, &dst).map_err(|e| e.to_string())?;
        dst
    } else if path.extension().map(|e| e == "zip").unwrap_or(false) {
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        let stem = path
            .file_stem()
            .ok_or("invalid file")?
            .to_string_lossy()
            .to_string();
        let dst = unique_dst(&user_pack_dir, &stem);
        std::fs::create_dir_all(&dst).map_err(|e| e.to_string())?;

        // Detect whether all entries share a single top-level directory prefix.
        // Mechvibes packs from the community ship as `pack-name/config.json` etc.;
        // we strip that prefix so files land directly in `packs/<stem>/`.
        let strip_prefix = {
            let mut prefix: Option<String> = None;
            let mut consistent = true;
            for i in 0..archive.len() {
                let entry = archive.by_index(i).map_err(|e| e.to_string())?;
                let name = entry.name();
                let first = name.split('/').next().unwrap_or("");
                if first.is_empty() || !name.contains('/') {
                    consistent = false;
                    break;
                }
                match &prefix {
                    None => prefix = Some(first.to_string()),
                    Some(p) if p == first => {}
                    Some(_) => { consistent = false; break; }
                }
            }
            if consistent { prefix } else { None }
        };

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let mangled = entry.mangled_name();
            let relative = match &strip_prefix {
                Some(prefix) => mangled.strip_prefix(prefix).unwrap_or(&mangled).to_path_buf(),
                None => mangled,
            };
            if relative.as_os_str().is_empty() { continue; }
            let outpath = dst.join(&relative);
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
        dst
    } else {
        return Err("unsupported file (need .zip or directory)".into());
    };

    // Patch the freshly-extracted config.json: rewrite manifest.id when it
    // collides with an already-loaded pack, and apply custom_name if provided.
    let cfg_path = dst.join("config.json");
    let bytes = std::fs::read(&cfg_path).map_err(|e| e.to_string())?;
    let mut json: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let original_id = json["id"].as_str().unwrap_or("").to_string();

    let existing_ids: std::collections::HashSet<String> = {
        let s = store.read().unwrap();
        s.ids().into_iter().collect()
    };
    let mut new_id = original_id.clone();
    if existing_ids.contains(&original_id) {
        let mut suffix = 2;
        loop {
            let candidate = format!("{original_id}-{suffix}");
            if !existing_ids.contains(&candidate) { new_id = candidate; break; }
            suffix += 1;
        }
    }

    let mut dirty = false;
    if new_id != original_id {
        json["id"] = serde_json::Value::String(new_id.clone());
        dirty = true;
    }
    if let Some(name) = &custom_name {
        json["name"] = serde_json::Value::String(name.clone());
        dirty = true;
    }
    if dirty {
        let new_bytes = serde_json::to_vec_pretty(&json).map_err(|e| e.to_string())?;
        std::fs::write(&cfg_path, new_bytes).map_err(|e| e.to_string())?;
    }

    store
        .write()
        .unwrap()
        .load_dir(&user_pack_dir)
        .map_err(|e| e.to_string())?;
    Ok("imported".into())
}

#[tauri::command]
pub fn delete_pack(
    id: String,
    store: State<'_, Arc<RwLock<PackStore>>>,
    active: State<'_, Arc<RwLock<String>>>,
    settings: State<'_, Arc<RwLock<Settings>>>,
) -> Result<(), String> {
    let dir_name = {
        let s = store.read().unwrap();
        let pack = s.get(&id).ok_or_else(|| format!("unknown pack: {id}"))?;
        if s.is_bundled(&id) { return Err("cannot delete bundled pack".into()); }
        pack.dir_name.clone()
    };
    let user_pack_dir = crate::user_data_dir().join("packs");
    let target = user_pack_dir.join(&dir_name);
    std::fs::remove_dir_all(&target).map_err(|e| e.to_string())?;
    store.write().unwrap().load_dir(&user_pack_dir).map_err(|e| e.to_string())?;

    // If we just deleted the active pack, fall back to the first remaining (bundled) pack.
    let was_active = { active.read().unwrap().clone() == id };
    if was_active {
        let new_active = {
            let s = store.read().unwrap();
            s.ids().first().cloned().unwrap_or_default()
        };
        *active.write().unwrap() = new_active.clone();
        let snapshot = {
            let mut g = settings.write().unwrap();
            g.active_pack = new_active;
            g.clone()
        };
        save_settings(&snapshot).map_err(|e| e.to_string())?;
    }
    Ok(())
}
