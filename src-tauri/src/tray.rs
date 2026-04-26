//! Menu-bar icon and dropdown window lifecycle.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};

const TRAY_ICON: &[u8] = include_bytes!("../icons/tray-icon.png");

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let img = Image::from_bytes(TRAY_ICON)?;
    let menu = MenuBuilder::new(app)
        .item(&MenuItemBuilder::new("🫧 Quick Panel").id("panel").build(app)?)
        .separator()
        .item(&MenuItemBuilder::new("Open BubbleKeys").id("open").build(app)?)
        .separator()
        .item(&MenuItemBuilder::new("Quit").id("quit").build(app)?)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .icon(img)
        .icon_as_template(true)
        .menu(&menu)
        .on_menu_event(|app, ev| match ev.id().as_ref() {
            "panel" => {
                log::info!("menu event: panel");
                if let Err(e) = show_dropdown(app) {
                    log::error!("show_dropdown failed: {e}");
                }
            }
            "open" => {
                log::info!("menu event: open");
                let _ = app.get_webview_window("main").map(|w| {
                    let _ = w.show();
                    let _ = w.set_focus();
                });
            }
            "quit" => {
                log::info!("menu event: quit");
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;
    log::info!("tray icon installed");
    Ok(())
}

fn show_dropdown(app: &AppHandle) -> tauri::Result<()> {
    log::info!(
        "show_dropdown called, cached: {}",
        app.get_webview_window("tray").is_some()
    );
    if let Some(win) = app.get_webview_window("tray") {
        win.show()?;
        win.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "tray", WebviewUrl::App("tray.html".into()))
        .title("BubbleKeys")
        .inner_size(280.0, 360.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(true)
        .build()?;

    let win = app.get_webview_window("tray").unwrap();
    let win_for_blur = win.clone();
    let focus_seen = Arc::new(AtomicBool::new(false));
    let focus_seen_for_event = focus_seen.clone();

    win.on_window_event(move |ev| match ev {
        tauri::WindowEvent::Focused(true) => {
            focus_seen_for_event.store(true, Ordering::Relaxed);
        }
        tauri::WindowEvent::Focused(false) => {
            if focus_seen_for_event.load(Ordering::Relaxed) {
                let _ = win_for_blur.hide();
            }
        }
        _ => {}
    });
    Ok(())
}
