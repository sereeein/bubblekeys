//! Loads sound packs from a directory into RAM and exposes them to the dispatcher.

use std::collections::HashMap;
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
            let samples = decode_pack_samples(&entry.path(), &manifest)?;
            self.packs.insert(manifest.id.clone(), LoadedPack { manifest, samples });
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
    fn loads_all_four_default_packs() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs");
        let mut store = PackStore::new();
        store.load_dir(&dir).expect("load default packs");
        let ids = store.ids();
        for expected in ["cherry-blue", "cherry-red", "cherry-brown", "bubbles"] {
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
}
