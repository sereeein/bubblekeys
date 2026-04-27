//! Loads sound packs from a directory into RAM and exposes them to the dispatcher.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;

use crate::pack_format::{load_manifest, KeyDefineType, PackError, PackManifest};

/// Decoded sample payload for a pack.
///
/// `Single` keeps the raw OGG/WAV bytes (decoded on each play).
/// `MultiPcm` decodes the sprite once and stores per-key f32 PCM slices,
/// which are fed to the engine as `rodio::buffer::SamplesBuffer`.
#[derive(Clone, Debug)]
pub enum PackSamples {
    Single(Arc<Vec<u8>>),
    MultiPcm {
        rate: u32,
        channels: u16,
        slices: HashMap<String, Arc<Vec<f32>>>,
    },
}

#[derive(Clone, Debug)]
pub struct LoadedPack {
    pub manifest: PackManifest,
    pub samples: PackSamples,
    pub dir_name: String,
}

#[derive(Default)]
pub struct PackStore {
    packs: HashMap<String, LoadedPack>,
    bundled_ids: HashSet<String>,
}

impl PackStore {
    pub fn new() -> Self { Self::default() }

    pub fn load_dir(&mut self, dir: &Path) -> Result<(), PackError> {
        // Clear existing packs so on-disk deletions propagate when load_dir is
        // called again (e.g. after delete_pack or import_pack). bundled_ids is
        // intentionally NOT cleared — bundled status persists across reloads.
        self.packs.clear();
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if !entry.path().is_dir() { continue; }
            let manifest = load_manifest(&entry.path())?;
            let samples = decode_pack_samples(&entry.path(), &manifest)?;
            let dir_name = entry.file_name().to_string_lossy().to_string();
            self.packs.insert(manifest.id.clone(), LoadedPack { manifest, samples, dir_name });
        }
        Ok(())
    }

    pub fn ids(&self) -> Vec<String> {
        let mut v: Vec<String> = self.packs.keys().cloned().collect();
        v.sort();
        v
    }

    pub fn get(&self, id: &str) -> Option<&LoadedPack> { self.packs.get(id) }

    pub fn mark_bundled(&mut self, ids: &[String]) {
        self.bundled_ids = ids.iter().cloned().collect();
    }

    pub fn is_bundled(&self, id: &str) -> bool { self.bundled_ids.contains(id) }
}

fn decode_pack_samples(dir: &Path, m: &PackManifest) -> Result<PackSamples, PackError> {
    match m.key_define_type {
        KeyDefineType::Single => {
            let bytes = std::fs::read(dir.join(&m.sound))?;
            Ok(PackSamples::Single(Arc::new(bytes)))
        }
        KeyDefineType::Multi => decode_multi(dir, m),
    }
}

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
        let len = (*dur_ms as f32 * frames_per_ms) as usize;
        let end = (start + len).min(pcm.len());
        if start >= pcm.len() {
            continue;
        }
        slices.insert(key.clone(), Arc::new(pcm[start..end].to_vec()));
    }
    Ok(PackSamples::MultiPcm { rate, channels, slices })
}

/// Always scans the bundled source dir to determine which pack ids are bundled
/// (independent of whether copy actually happens), so the caller can mark them
/// protected from deletion on every launch — not just the first.
///
/// On first launch only, copies bundled packs from the app resource dir to the
/// user pack dir. Subsequent launches skip the copy but still return the same
/// bundled-id list.
pub fn install_default_packs(
    bundled_resource_dir: &Path,
    user_pack_dir: &Path,
) -> std::io::Result<Vec<String>> {
    let src = bundled_resource_dir.join("packs");

    let bundled_ids = if src.exists() {
        let mut ids = Vec::new();
        for entry in std::fs::read_dir(&src)? {
            let entry = entry?;
            if !entry.path().is_dir() { continue; }
            if let Ok(manifest) = load_manifest(&entry.path()) {
                ids.push(manifest.id);
            }
        }
        ids
    } else {
        log::warn!("bundled packs dir missing: {}", src.display());
        Vec::new()
    };

    let already_populated = user_pack_dir.exists()
        && std::fs::read_dir(user_pack_dir)?.next().is_some();
    if !already_populated && src.exists() {
        std::fs::create_dir_all(user_pack_dir)?;
        copy_dir_recursive(&src, user_pack_dir)?;
    }

    Ok(bundled_ids)
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn loads_single_fixture() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let mut store = PackStore::new();
        store.load_dir(&dir).unwrap();
        assert!(store.ids().contains(&"test-single".to_string()));
    }

    #[test]
    fn loads_all_default_packs() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs");
        let mut store = PackStore::new();
        store.load_dir(&dir).expect("load default packs");
        let ids = store.ids();
        for expected in [
            "bubbles",
            "cherry-red",
            "cherry-blue",
            "cherry-brown",
            "cherry-black",
            "cherry-silver",
            "cherry-red-silent",
            "cherry-purple",
            "cherry-white",
        ] {
            assert!(ids.contains(&expected.to_string()), "missing pack: {expected}");
        }
    }

    #[test]
    fn multi_pack_slices_two_keys() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pack_multi");
        let m = load_manifest(&dir).unwrap();
        let samples = decode_pack_samples(&dir, &m).unwrap();
        if let PackSamples::MultiPcm { slices, .. } = samples {
            assert!(slices.contains_key("1"));
            assert!(slices.contains_key("57"));
        } else {
            panic!("expected multi");
        }
    }

    #[test]
    fn dir_name_persisted_on_load() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let mut store = PackStore::new();
        store.load_dir(&dir).unwrap();
        let p = store.get("test-single").unwrap();
        assert_eq!(p.dir_name, "pack_single");
    }

    #[test]
    fn mark_bundled_works() {
        let mut store = PackStore::new();
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        store.load_dir(&dir).unwrap();
        assert!(!store.is_bundled("test-single"));
        store.mark_bundled(&["test-single".to_string()]);
        assert!(store.is_bundled("test-single"));
        assert!(!store.is_bundled("test-multi"));
    }

    #[test]
    fn load_dir_clears_previous_packs() {
        use tempfile::tempdir;
        let mut store = PackStore::new();
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        store.load_dir(&fixtures).unwrap();
        let initial_count = store.ids().len();
        assert!(initial_count > 0);

        // Reload from an empty temp dir → packs should be cleared.
        let empty = tempdir().unwrap();
        store.load_dir(empty.path()).unwrap();
        assert_eq!(store.ids().len(), 0);
    }
}
