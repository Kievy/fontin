mod activation;
mod commands;
mod db;
mod error;
mod google;
mod i18n;
mod library;
mod model;
mod scanner;
mod settings;
mod state;
mod tray;

use model::SourceKind;
use state::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager, RunEvent, WindowEvent};
use tray::{build_tray, show_main};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        // A second launch (e.g. from the app menu while hidden) just shows the window.
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            show_main(app)
        }))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let state = Arc::new(AppState::init(data_dir)?);
            app.manage(state.clone());

            i18n::Lang::resolve(&state.settings.read().language).set_current();
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
                    if let Err(e) = st.backfill_hashes() {
                        log::error!("hash backfill failed: {e}");
                    }
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
                    .filter(|s| s.kind != SourceKind::Google)
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
            commands::add_files,
            commands::allow_duplicates,
            commands::set_language,
            commands::self_update_supported,
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
