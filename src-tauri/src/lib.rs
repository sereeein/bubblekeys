pub mod audio_engine;
pub mod dispatcher;
pub mod ipc;
pub mod key_listener;
pub mod mute_controller;
pub mod night_silent;
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

pub fn user_data_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME");
    PathBuf::from(home).join("Library/Application Support").join(APP_SUBDIR)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let trusted = key_listener::ensure_accessibility(true);
    log::info!("accessibility trusted at startup: {trusted}");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            ipc::list_packs,
            ipc::set_active_pack,
            ipc::get_state,
            ipc::set_muted,
            ipc::set_volume,
            ipc::preview_pack,
            ipc::get_settings,
            ipc::update_settings,
            ipc::complete_onboarding,
            ipc::reset_onboarding,
            ipc::open_accessibility_settings,
            ipc::check_accessibility,
            ipc::open_url,
            ipc::get_app_version,
            ipc::close_app,
            ipc::start_drag,
            ipc::import_pack,
            ipc::delete_pack,
        ])
        .setup(|app| {
            let resource_dir = app.path().resource_dir().expect("resource_dir");
            let user_dir = user_data_dir();
            let pack_dir = user_dir.join("packs");
            let bundled_ids = install_default_packs(&resource_dir, &pack_dir).unwrap_or_default();

            let settings = settings_store::load();

            let mut store = PackStore::new();
            store.load_dir(&pack_dir).expect("load packs");
            store.mark_bundled(&bundled_ids);
            log::info!("loaded {} packs from {}", store.ids().len(), pack_dir.display());

            log::info!("active pack: {}", settings.active_pack);
            let active_pack = Arc::new(RwLock::new(settings.active_pack.clone()));

            let engine = Arc::new(RodioEngine::new().expect("audio engine"));
            log::info!("audio engine ready");
            let mute = MuteController::new();
            mute.set_user_muted(settings.muted);

            let dispatcher = Dispatcher::new(engine.clone(), mute.clone());
            let store = Arc::new(RwLock::new(store));
            let store_for_thread = store.clone();
            let active_for_thread = active_pack.clone();
            let volume: Arc<RwLock<f32>> = Arc::new(RwLock::new(settings.volume));
            let volume_for_thread = volume.clone();

            // Retry-init pattern: CGEventTap creation fails silently when
            // Accessibility is not granted. Rather than blocking app startup,
            // we spawn a single thread that retries every 2s until trust is
            // granted, then runs the dispatcher loop. This means typing
            // produces sound within 2s of the user granting permission, with
            // no app restart required.
            thread::Builder::new()
                .name("bubblekeys-dispatch".into())
                .spawn(move || {
                    log::info!("dispatcher thread started, waiting for key listener init");
                    let listener = loop {
                        match MacKeyListener::start() {
                            Ok(l) => {
                                log::info!("key listener init succeeded");
                                break l;
                            }
                            Err(e) => {
                                log::warn!("key listener init failed ({e}), retry in 2s");
                                std::thread::sleep(std::time::Duration::from_secs(2));
                            }
                        }
                    };
                    let rx = listener.events();
                    log::info!("dispatcher receiving events");
                    while let Ok(ev) = rx.recv() {
                        let id = active_for_thread.read().unwrap().clone();
                        let guard = store_for_thread.read().unwrap();
                        if let Some(pack) = guard.get(&id) {
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
                            let ids = store_handle.read().unwrap().ids();
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

            let mute_for_night = mute.clone();
            app.manage(mute);
            app.manage(store);
            app.manage(active_pack);
            app.manage(volume);
            let settings_arc = Arc::new(RwLock::new(settings));
            night_silent::spawn(settings_arc.clone(), mute_for_night);
            app.manage(settings_arc);
            let engine_for_state: Arc<dyn AudioEngine> = engine.clone();
            app.manage(engine_for_state);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running BubbleKeys");
}
