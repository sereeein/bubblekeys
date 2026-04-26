pub mod audio_engine;
pub mod key_listener;

use std::sync::Arc;
use std::thread;

use audio_engine::{AudioEngine, PlayCommand, RodioEngine};
use key_listener::{KeyListener, MacKeyListener};

const EMBEDDED_CLICK: &[u8] = include_bytes!("../assets/click.ogg");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let engine: Arc<dyn AudioEngine> = Arc::new(
        RodioEngine::new().expect("audio engine init"),
    );
    let listener = MacKeyListener::start().expect("key listener init");
    let rx = listener.events();
    let click_bytes = Arc::new(EMBEDDED_CLICK.to_vec());

    // Dispatcher thread: every keydown → click sound at fixed volume.
    let engine_for_thread = engine.clone();
    thread::Builder::new()
        .name("bubblekeys-dispatch".into())
        .spawn(move || {
            while let Ok(ev) = rx.recv() {
                if matches!(ev.kind, key_listener::KeyEventKind::Down) {
                    engine_for_thread.play(PlayCommand {
                        sample: click_bytes.clone(),
                        volume: 0.65,
                        pitch_offset: 0.0,
                    });
                }
            }
        })
        .expect("dispatcher thread");

    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running BubbleKeys");
}
