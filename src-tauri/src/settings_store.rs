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
