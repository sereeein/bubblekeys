//! Loads sound packs from a directory into RAM and exposes them to the dispatcher.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::pack_format::{load_manifest, KeyDefineType, PackError, PackManifest};

#[derive(Clone, Debug)]
pub struct LoadedPack {
    pub manifest: PackManifest,
    /// Single-sound packs: one entry under "*". Multi: one entry per defined keycode (string).
    pub samples_by_key: HashMap<String, Arc<Vec<u8>>>,
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
            let samples = decode_samples(&entry.path(), &manifest)?;
            self.packs.insert(manifest.id.clone(), LoadedPack { manifest, samples_by_key: samples });
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

fn decode_samples(dir: &Path, m: &PackManifest) -> Result<HashMap<String, Arc<Vec<u8>>>, PackError> {
    let mut out = HashMap::new();
    let sound_path = dir.join(&m.sound);
    let bytes = Arc::new(std::fs::read(&sound_path)?);
    match m.key_define_type {
        KeyDefineType::Single => {
            out.insert("*".into(), bytes);
        }
        KeyDefineType::Multi => {
            // v1: store one shared sprite blob; dispatcher slices by offset.
            // For Phase 3 we treat multi-packs as if every key plays the whole sprite (good enough).
            // Phase 10 (Mechvibes import) refines this with sprite slicing.
            for keycode in m.defines.keys() {
                out.insert(keycode.clone(), bytes.clone());
            }
        }
    }
    Ok(out)
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
}
