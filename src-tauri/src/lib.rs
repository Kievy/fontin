mod activation;
mod commands;
mod db;
mod error;
mod google;
mod library;
mod model;
mod scanner;
mod state;

use model::SourceKind;
use state::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let state = Arc::new(AppState::init(data_dir)?);
            app.manage(state.clone());

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Fontin");
}
