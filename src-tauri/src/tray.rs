//! System tray icon and its menu (labels follow the current language).

use crate::commands;
use crate::i18n::Msg;
use crate::state::AppState;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};

fn tray_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let open = MenuItem::with_id(app, "open", Msg::TrayOpen.text(), true, None::<&str>)?;
    let off = MenuItem::with_id(
        app,
        "deactivate-all",
        Msg::TrayDeactivateAll.text(),
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", Msg::TrayQuit.text(), true, None::<&str>)?;
    let sep = || PredefinedMenuItem::separator(app);
    Menu::with_items(app, &[&open, &sep()?, &off, &sep()?, &quit])
}

/// Rebuild the menu after a language change.
pub fn refresh_tray(app: &AppHandle) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id("main"), tray_menu(app)) {
        let _ = tray.set_menu(Some(menu));
    }
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// Turn every active font off (saved state included) without blocking the tray.
fn deactivate_all(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let Some(state) = app.try_state::<Arc<AppState>>() else {
            return;
        };
        let ids: Vec<_> = state.lib.read().active.iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        match state.set_faces_active(&ids, false) {
            Ok(()) => {
                let _ = app.emit(commands::EV_LIBRARY, ());
                let _ = app.emit(
                    commands::EV_NOTICE,
                    Msg::StylesDeactivated(ids.len()).text(),
                );
            }
            Err(e) => log::error!("deactivate all failed: {e}"),
        }
    });
}

pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = tray_menu(app)?;
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Fontin")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, e| match e.id.as_ref() {
            "open" => show_main(app),
            "deactivate-all" => deactivate_all(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, e| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = e
            {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}
