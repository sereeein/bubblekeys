//! Glues KeyEvent → active pack → PlayCommand. Honors mute state.

use std::sync::Arc;

use crate::audio_engine::{AudioEngine, PlayCommand};
use crate::key_listener::{KeyEvent, KeyEventKind};
use crate::mute_controller::MuteController;
use crate::pack_store::LoadedPack;

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
        let sample = pack.samples_by_key.get(&key)
            .or_else(|| pack.samples_by_key.get("*"))
            .cloned();

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
    use std::collections::HashMap;

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
        let mut s = HashMap::new();
        s.insert("*".into(), Arc::new(vec![0u8; 4]));
        LoadedPack { manifest, samples_by_key: s }
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
