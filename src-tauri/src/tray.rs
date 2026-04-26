//! Menu-bar icon and dropdown window lifecycle.

use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};

const TRAY_ICON: &[u8] = include_bytes!("../icons/tray-icon.png");

pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let img = Image::from_bytes(TRAY_ICON)?;
    let menu = MenuBuilder::new(app)
        .item(&MenuItemBuilder::new("Open BubbleKeys").id("open").build(app)?)
        .separator()
        .item(&MenuItemBuilder::new("Quit").id("quit").build(app)?)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .icon(img)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { .. } = event {
                show_dropdown(tray.app_handle()).ok();
            }
        })
        .on_menu_event(|app, ev| match ev.id().as_ref() {
            "open" => {
                let _ = app.get_webview_window("main").map(|w| {
                    let _ = w.show();
                    let _ = w.set_focus();
                });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

fn show_dropdown(app: &AppHandle) -> tauri::Result<()> {
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
    win.on_window_event(move |ev| {
        if matches!(ev, tauri::WindowEvent::Focused(false)) {
            let _ = win_for_blur.hide();
        }
    });
    Ok(())
}
