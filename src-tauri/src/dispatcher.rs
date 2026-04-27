//! Glues KeyEvent → active pack → PlayCommand. Honors mute state.

use std::sync::Arc;

use crate::audio_engine::{AudioEngine, PlayCommand, SampleData};
use crate::key_listener::{KeyEvent, KeyEventKind};
use crate::mute_controller::MuteController;
use crate::pack_store::{LoadedPack, PackSamples};

pub struct Dispatcher<E: AudioEngine + ?Sized> {
    engine: Arc<E>,
    mute: MuteController,
}

impl<E: AudioEngine + ?Sized> Dispatcher<E> {
    pub fn new(engine: Arc<E>, mute: MuteController) -> Self {
        Self { engine, mute }
    }

    pub fn handle(&self, ev: KeyEvent, pack: &LoadedPack, volume: f32, pitch_offset: f32) {
        if !matches!(ev.kind, KeyEventKind::Down) { return; }
        if self.mute.is_muted() { return; }

        let key = ev.keycode.to_string();
        let sample = match &pack.samples {
            PackSamples::Single(bytes) => Some(SampleData::Encoded(bytes.clone())),
            PackSamples::MultiPcm { rate, channels, slices } => {
                slices.get(&key)
                    .or_else(|| slices.values().next()) // fallback: any slice if key not mapped
                    .map(|s| SampleData::Pcm {
                        rate: *rate,
                        channels: *channels,
                        samples: s.clone(),
                    })
            }
        };

        if let Some(sample) = sample {
            self.engine.play(PlayCommand { sample, volume, pitch_offset });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use crate::pack_format::{KeyDefineType, PackManifest};

    struct CountingEngine { calls: Mutex<u32> }
    impl AudioEngine for CountingEngine {
        fn play(&self, _cmd: PlayCommand) { *self.calls.lock().unwrap() += 1; }
    }

    fn dummy_pack() -> LoadedPack {
        let manifest = PackManifest {
            id: "x".into(), name: "X".into(),
            key_define_type: KeyDefineType::Single,
            sound: "s.ogg".into(),
            defines: Default::default(),
            includes_numpad: true,
            license: None, author: None, icon: None, tags: vec![],
        };
        LoadedPack {
            manifest,
            samples: PackSamples::Single(Arc::new(vec![0u8; 4])),
            dir_name: "test".into(),
        }
    }

    #[test]
    fn keydown_plays() {
        let engine = Arc::new(CountingEngine { calls: Mutex::new(0) });
        let mute = MuteController::new();
        let d = Dispatcher::new(engine.clone(), mute);
        d.handle(KeyEvent { keycode: 0, kind: KeyEventKind::Down }, &dummy_pack(), 0.5, 0.0);
        assert_eq!(*engine.calls.lock().unwrap(), 1);
    }

    #[test]
    fn keyup_does_not_play() {
        let engine = Arc::new(CountingEngine { calls: Mutex::new(0) });
        let d = Dispatcher::new(engine.clone(), MuteController::new());
        d.handle(KeyEvent { keycode: 0, kind: KeyEventKind::Up }, &dummy_pack(), 0.5, 0.0);
        assert_eq!(*engine.calls.lock().unwrap(), 0);
    }

    #[test]
    fn mute_blocks_play() {
        let engine = Arc::new(CountingEngine { calls: Mutex::new(0) });
        let mute = MuteController::new();
        mute.set_user_muted(true);
        let d = Dispatcher::new(engine.clone(), mute);
        d.handle(KeyEvent { keycode: 0, kind: KeyEventKind::Down }, &dummy_pack(), 0.5, 0.0);
        assert_eq!(*engine.calls.lock().unwrap(), 0);
    }
}
