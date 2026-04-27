//! Mechvibes-compatible sound pack manifest parsing.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyDefineType { Single, Multi }

#[derive(Clone, Debug, Deserialize)]
pub struct PackManifest {
    pub id: String,
    pub name: String,
    pub key_define_type: KeyDefineType,
    pub sound: String,
    /// Only required for `Multi`. Map of keycode → [offset_ms, duration_ms].
    #[serde(default)]
    pub defines: HashMap<String, [u32; 2]>,
    #[serde(default = "default_true")]
    pub includes_numpad: bool,
    pub license: Option<String>,
    pub author: Option<String>,
    pub icon: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_true() -> bool { true }

#[derive(Debug, thiserror::Error)]
pub enum PackError {
    #[error("io: {0}")] Io(#[from] std::io::Error),
    #[error("json: {0}")] Json(#[from] serde_json::Error),
    #[error("pack id mismatches directory: '{0}' vs '{1}'")] IdMismatch(String, String),
    #[error("multi pack missing 'defines'")] MultiMissingDefines,
    #[error("decode: {0}")] Decode(String),
}

pub fn load_manifest(dir: &Path) -> Result<PackManifest, PackError> {
    let cfg_path = dir.join("config.json");
    let bytes = std::fs::read(&cfg_path)?;
    let m: PackManifest = serde_json::from_slice(&bytes)?;
    if matches!(m.key_define_type, KeyDefineType::Multi) && m.defines.is_empty() {
        return Err(PackError::MultiMissingDefines);
    }
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
    }

    #[test]
    fn parses_single_pack() {
        let m = load_manifest(&fixture("pack_single")).unwrap();
        assert_eq!(m.id, "test-single");
        assert!(matches!(m.key_define_type, KeyDefineType::Single));
        assert!(m.defines.is_empty());
    }

    #[test]
    fn rejects_multi_without_defines() {
        // Create the bad case inline: write a tmp dir with an invalid manifest.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("config.json"),
            r#"{ "id":"bad","name":"Bad","key_define_type":"multi","sound":"s.ogg" }"#,
        ).unwrap();
        let err = load_manifest(dir.path()).unwrap_err();
        assert!(matches!(err, PackError::MultiMissingDefines));
    }
}
