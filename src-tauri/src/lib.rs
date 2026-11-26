mod activation;
mod commands;
mod db;
mod error;
mod google;
mod library;
mod model;
mod scanner;
mod settings;
mod state;

use model::SourceKind;
use state::AppState;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};

fn show_main(app: &AppHandle) {
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
        let Some(state) = app.try_state::<Arc<AppState>>() else { return };
        let ids: Vec<_> = state.lib.read().active.iter().copied().collect();
        if ids.is_empty() {
            return;
        }
        match state.set_faces_active(&ids, false) {
            Ok(()) => {
                let _ = app.emit(commands::EV_LIBRARY, ());
                let _ = app.emit(commands::EV_NOTICE, format!("{} estilos desativados", ids.len()));
            }
            Err(e) => log::error!("deactivate all failed: {e}"),
        }
    });
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Abrir Fontin", true, None::<&str>)?;
    let off = MenuItem::with_id(app, "deactivate-all", "Desativar todas as fontes", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let sep = || PredefinedMenuItem::separator(app);
    let menu = Menu::with_items(app, &[&open, &sep()?, &off, &sep()?, &quit])?;
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
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        // A second launch (e.g. from the app menu while hidden) just shows the window.
        .plugin(tauri_plugin_single_instance::init(|app, _, _| show_main(app)))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let state = Arc::new(AppState::init(data_dir)?);
            app.manage(state.clone());

            if let Err(e) = build_tray(app.handle()) {
                log::warn!("tray unavailable: {e}");
            }
            // The window is created hidden (tauri.conf.json) to avoid a flash.
            if !state.settings.read().start_hidden {
                show_main(app.handle());
            }

            // Everything below is off the startup path: the UI renders from the
            // persisted index immediately and refreshes when these finish.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let st = state.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    if let Err(e) = st.sync_activation() {
                        log::error!("activation sync failed: {e}");
                    }
                })
                .await;

                let folders: Vec<_> = state
                    .lib
                    .read()
                    .sources
                    .iter()
                    .filter(|s| s.kind == SourceKind::Folder)
                    .map(|s| s.id)
                    .collect();
                for id in folders {
                    if let Err(e) = commands::scan(handle.clone(), state.clone(), id).await {
                        log::warn!("rescan {id} failed: {e}");
                    }
                }
                if let Err(e) = commands::refresh_catalog(&handle, &state, false).await {
                    log::warn!("google catalog refresh failed: {e}");
                    let _ = handle.emit(commands::EV_CATALOG, ());
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::overview,
            commands::query_families,
            commands::family_detail,
            commands::face_data,
            commands::set_family_active,
            commands::set_face_active,
            commands::set_families_active,
            commands::toggle_favorite,
            commands::add_folder,
            commands::rescan_source,
            commands::remove_source,
            commands::create_collection,
            commands::update_collection,
            commands::delete_collection,
            commands::set_in_collection,
            commands::refresh_google,
            commands::get_setting,
            commands::set_setting,
            commands::get_settings,
            commands::set_settings,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<Arc<AppState>>();
                if state.settings.read().hide_on_close {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building Fontin")
        .run(|app, event| {
            if let RunEvent::Exit = event
                && let Some(state) = app.try_state::<Arc<AppState>>()
                && state.settings.read().deactivate_on_exit
            {
                state.release_all();
            }
        });
}
