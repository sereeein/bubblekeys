pub mod audio_engine;
pub mod dispatcher;
pub mod ipc;
pub mod key_listener;
pub mod mute_controller;
pub mod pack_format;
pub mod pack_store;
pub mod settings_store;

use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

use audio_engine::{AudioEngine, RodioEngine};
use dispatcher::Dispatcher;
use key_listener::{KeyListener, MacKeyListener};
use mute_controller::MuteController;
use pack_store::{install_default_packs, PackStore};
use tauri::Manager;

const APP_SUBDIR: &str = "BubbleKeys";

fn user_data_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME");
    PathBuf::from(home).join("Library/Application Support").join(APP_SUBDIR)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let trusted = key_listener::ensure_accessibility(true);
    log::info!("accessibility trusted at startup: {trusted}");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            ipc::list_packs,
            ipc::set_active_pack,
            ipc::get_state,
            ipc::set_muted,
            ipc::set_volume,
            ipc::preview_pack,
        ])
        .setup(|app| {
            let resource_dir = app.path().resource_dir().expect("resource_dir");
            let user_dir = user_data_dir();
            let pack_dir = user_dir.join("packs");
            install_default_packs(&resource_dir, &pack_dir).ok();

            let mut store = PackStore::new();
            store.load_dir(&pack_dir).expect("load packs");
            log::info!("loaded {} packs from {}", store.ids().len(), pack_dir.display());

            let active_id = store.ids().first().expect("at least one pack").clone();
            log::info!("active pack: {active_id}");
            let active_pack = Arc::new(RwLock::new(active_id));

            let engine = Arc::new(RodioEngine::new().expect("audio engine"));
            log::info!("audio engine ready");
            let mute = MuteController::new();
            let listener = MacKeyListener::start().expect("key listener init");
            log::info!("key listener thread spawned");
            let rx = listener.events();

            let dispatcher = Dispatcher::new(engine.clone(), mute.clone());
            let store = Arc::new(store);
            let store_for_thread = store.clone();
            let active_for_thread = active_pack.clone();
            let volume: Arc<RwLock<f32>> = Arc::new(RwLock::new(0.65));
            let volume_for_thread = volume.clone();

            thread::Builder::new()
                .name("bubblekeys-dispatch".into())
                .spawn(move || {
                    log::info!("dispatcher thread started");
                    while let Ok(ev) = rx.recv() {
                        let id = active_for_thread.read().unwrap().clone();
                        if let Some(pack) = store_for_thread.get(&id) {
                            let v = *volume_for_thread.read().unwrap();
                            dispatcher.handle(ev, pack, v, 0.0);
                        }
                    }
                    log::warn!("dispatcher thread exiting (channel closed)");
                })
                .expect("dispatcher thread");

            #[cfg(debug_assertions)]
            {
                use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
                let cycle = MenuItemBuilder::new("Cycle Pack").id("cycle").accelerator("CmdOrCtrl+1").build(app)?;
                // Use Cmd+Shift+M to avoid conflict with macOS's built-in "Minimize Window" (Cmd+M).
                let toggle = MenuItemBuilder::new("Toggle Mute").id("toggle").accelerator("CmdOrCtrl+Shift+M").build(app)?;
                // macOS only renders submenus in the menu bar — flat MenuItems at root don't show.
                // Build an "App" submenu (BubbleKeys name + Quit so the user can exit) and a "Debug" submenu with our items.
                let app_submenu = SubmenuBuilder::new(app, "BubbleKeys")
                    .item(&PredefinedMenuItem::quit(app, Some("Quit BubbleKeys"))?)
                    .build()?;
                let debug_submenu = SubmenuBuilder::new(app, "Debug")
                    .item(&cycle)
                    .item(&toggle)
                    .build()?;
                let menu = MenuBuilder::new(app)
                    .item(&app_submenu)
                    .item(&debug_submenu)
                    .build()?;
                app.set_menu(menu)?;

                let store_handle = store.clone();
                let active_handle = active_pack.clone();
                let mute_handle = mute.clone();
                app.on_menu_event(move |_app, event| {
                    match event.id().as_ref() {
                        "cycle" => {
                            let ids = store_handle.ids();
                            let mut active = active_handle.write().unwrap();
                            let idx = ids.iter().position(|i| i == &*active).unwrap_or(0);
                            *active = ids[(idx + 1) % ids.len()].clone();
                            log::info!("cycled to pack: {}", *active);
                        }
                        "toggle" => {
                            let cur = mute_handle.is_muted();
                            mute_handle.set_user_muted(!cur);
                            log::info!("mute={}", !cur);
                        }
                        _ => {}
                    }
                });
            }

            app.manage(mute);
            app.manage(store);
            app.manage(active_pack);
            app.manage(volume);
            let engine_for_state: Arc<dyn AudioEngine> = engine.clone();
            app.manage(engine_for_state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running BubbleKeys");
}
