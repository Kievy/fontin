use crate::error::{Error, Result};
use crate::model::*;
use crate::state::AppState;
use crate::{google, scanner};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

type St<'a> = State<'a, Arc<AppState>>;

pub const EV_LIBRARY: &str = "library-changed";
pub const EV_CATALOG: &str = "catalog-changed";
pub const EV_DOWNLOAD: &str = "download-progress";
pub const EV_SCAN: &str = "scan-progress";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgress<'a> {
    family_id: &'a str,
    done: usize,
    total: usize,
    finished: bool,
    error: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanProgress {
    source_id: SourceId,
    scanning: bool,
    added: usize,
    removed: usize,
}

#[tauri::command]
pub async fn overview(state: St<'_>) -> Result<Overview> {
    Ok(state.overview())
}

#[tauri::command]
pub async fn query_families(state: St<'_>, query: Query) -> Result<Vec<FamilyRow>> {
    let lib = state.lib.read();
    let catalog = state.catalog.read();
    Ok(lib.query(&query, &catalog))
}

#[tauri::command]
pub async fn family_detail(state: St<'_>, id: String) -> Result<Option<FamilyDetail>> {
    let lib = state.lib.read();
    let catalog = state.catalog.read();
    Ok(lib.detail(&id, &catalog))
}

/// Raw font bytes for previews, sent as a binary IPC payload (ArrayBuffer in JS).
#[tauri::command]
pub async fn face_data(state: St<'_>, id: String) -> Result<tauri::ipc::Response> {
    let fid = parse_id(&id).ok_or_else(|| Error::msg("id inválido"))?;
    let path = state
        .lib
        .read()
        .faces
        .get(&fid)
        .map(|f| f.path.clone())
        .ok_or_else(|| Error::msg("face desconhecida"))?;
    Ok(tauri::ipc::Response::new(tokio::fs::read(path).await?))
}

/// Toggle a whole family. Google families are downloaded on first activation.
#[tauri::command]
pub async fn set_family_active(
    app: AppHandle,
    state: St<'_>,
    id: String,
    active: bool,
) -> Result<()> {
    let state = state.inner().clone();
    if active && state.family_face_ids(&id).is_none() {
        let family = id
            .strip_prefix("g:")
            .ok_or_else(|| Error::msg("família desconhecida"))?
            .to_owned();
        download_google(&app, &state, &id, &family).await?;
    }
    let ids = state
        .family_face_ids(&id)
        .ok_or_else(|| Error::msg("família sem arquivos"))?;
    tokio::task::spawn_blocking(move || state.set_faces_active(&ids, active))
        .await
        .map_err(|e| Error::msg(e.to_string()))??;
    let _ = app.emit(EV_LIBRARY, ());
    Ok(())
}

#[tauri::command]
pub async fn set_face_active(
    app: AppHandle,
    state: St<'_>,
    id: String,
    active: bool,
) -> Result<()> {
    let fid = parse_id(&id).ok_or_else(|| Error::msg("id inválido"))?;
    let state = state.inner().clone();
    tokio::task::spawn_blocking(move || state.set_faces_active(&[fid], active))
        .await
        .map_err(|e| Error::msg(e.to_string()))??;
    let _ = app.emit(EV_LIBRARY, ());
    Ok(())
}

/// Bulk toggle for the current view (e.g. "deactivate all").
#[tauri::command]
pub async fn set_families_active(
    app: AppHandle,
    state: St<'_>,
    ids: Vec<String>,
    active: bool,
) -> Result<()> {
    let state = state.inner().clone();
    let face_ids: Vec<FaceId> = {
        let lib = state.lib.read();
        ids.iter()
            .filter_map(|id| lib.families.get(id))
            .flat_map(|f| f.faces.iter().copied())
            .collect()
    };
    tokio::task::spawn_blocking(move || state.set_faces_active(&face_ids, active))
        .await
        .map_err(|e| Error::msg(e.to_string()))??;
    let _ = app.emit(EV_LIBRARY, ());
    Ok(())
}

async fn download_google(
    app: &AppHandle,
    state: &Arc<AppState>,
    id: &str,
    family: &str,
) -> Result<()> {
    if !state.downloading.lock().insert(id.to_owned()) {
        return Err(Error::msg("download já em andamento"));
    }
    let emit = |done, total, finished, error: Option<String>| {
        let _ = app.emit(
            EV_DOWNLOAD,
            DownloadProgress {
                family_id: id,
                done,
                total,
                finished,
                error,
            },
        );
    };
    let res = google::download_family(&state.http, &state.google_dir, family, |d, t| {
        emit(d, t, false, None)
    })
    .await;
    state.downloading.lock().remove(id);
    match res {
        Ok(faces) => {
            let stale: Vec<FaceId> = state.family_face_ids(id).unwrap_or_default();
            state.db.lock().apply_faces(&faces, &stale)?;
            state.lib.write().apply_faces(faces, &stale);
            emit(1, 1, true, None);
            Ok(())
        }
        Err(e) => {
            emit(0, 0, true, Some(e.to_string()));
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn toggle_favorite(app: AppHandle, state: St<'_>, id: String) -> Result<bool> {
    let on = !state.lib.read().favorites.contains(&id);
    state.db.lock().set_favorite(&id, on)?;
    let mut lib = state.lib.write();
    if on {
        lib.favorites.insert(id);
    } else {
        lib.favorites.remove(&id);
    }
    drop(lib);
    let _ = app.emit(EV_LIBRARY, ());
    Ok(on)
}

// ---------- Sources ----------

#[tauri::command]
pub async fn add_folder(app: AppHandle, state: St<'_>, path: String) -> Result<SourceId> {
    let path = PathBuf::from(path);
    if !path.is_dir() {
        return Err(Error::msg("pasta não encontrada"));
    }
    {
        let lib = state.lib.read();
        if let Some(s) = lib
            .sources
            .iter()
            .find(|s| s.kind == SourceKind::Folder && path.starts_with(&s.path))
        {
            return Err(Error::msg(format!("já incluída em “{}”", s.name)));
        }
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Pasta".into());
    let source = state
        .db
        .lock()
        .add_source(SourceKind::Folder, &path, &name)?;
    let id = source.id;
    state.lib.write().sources.push(source);
    scan(app, state.inner().clone(), id).await?;
    Ok(id)
}

#[tauri::command]
pub async fn rescan_source(app: AppHandle, state: St<'_>, id: SourceId) -> Result<()> {
    scan(app, state.inner().clone(), id).await
}

pub async fn scan(app: AppHandle, state: Arc<AppState>, id: SourceId) -> Result<()> {
    let _ = app.emit(
        EV_SCAN,
        ScanProgress {
            source_id: id,
            scanning: true,
            added: 0,
            removed: 0,
        },
    );
    let st = state.clone();
    let (added, removed) = tokio::task::spawn_blocking(move || -> Result<(usize, usize)> {
        let (root, known) = {
            let lib = st.lib.read();
            let src = lib
                .source(id)
                .ok_or_else(|| Error::msg("fonte desconhecida"))?;
            (src.path.clone(), lib.known(id))
        };
        let r = scanner::scan_source(id, &root, &known);
        let counts = (r.added.len(), r.removed.len());
        log::info!("scan {id}: +{} -{} ={}", counts.0, counts.1, r.unchanged);
        if counts == (0, 0) {
            return Ok(counts);
        }
        // Deactivate faces that vanished before dropping them from the index.
        let gone_active: Vec<Face> = {
            let lib = st.lib.read();
            r.removed
                .iter()
                .filter(|i| lib.active.contains(i))
                .filter_map(|i| lib.faces.get(i).cloned())
                .collect()
        };
        if !gone_active.is_empty() {
            let refs: Vec<&Face> = gone_active.iter().collect();
            let _ = st.activator.deactivate(&refs);
        }
        st.db.lock().apply_faces(&r.added, &r.removed)?;
        st.lib.write().apply_faces(r.added, &r.removed);
        Ok(counts)
    })
    .await
    .map_err(|e| Error::msg(e.to_string()))??;
    let _ = app.emit(
        EV_SCAN,
        ScanProgress {
            source_id: id,
            scanning: false,
            added,
            removed,
        },
    );
    let _ = app.emit(EV_LIBRARY, ());
    Ok(())
}

#[tauri::command]
pub async fn remove_source(app: AppHandle, state: St<'_>, id: SourceId) -> Result<()> {
    if id == GOOGLE_SOURCE_ID {
        return Err(Error::msg("o provedor Google não pode ser removido"));
    }
    let state = state.inner().clone();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let faces: Vec<Face> = {
            let lib = state.lib.read();
            lib.faces
                .values()
                .filter(|f| f.source_id == id && lib.active.contains(&f.id))
                .cloned()
                .collect()
        };
        let refs: Vec<&Face> = faces.iter().collect();
        state.activator.deactivate(&refs)?;
        state.db.lock().remove_source(id)?;
        let mut lib = state.lib.write();
        let ids: Vec<FaceId> = lib
            .faces
            .values()
            .filter(|f| f.source_id == id)
            .map(|f| f.id)
            .collect();
        lib.sources.retain(|s| s.id != id);
        lib.apply_faces(vec![], &ids);
        Ok(())
    })
    .await
    .map_err(|e| Error::msg(e.to_string()))??;
    let _ = app.emit(EV_LIBRARY, ());
    Ok(())
}

// ---------- Collections ----------

#[tauri::command]
pub async fn create_collection(
    app: AppHandle,
    state: St<'_>,
    name: String,
    color: String,
) -> Result<i64> {
    let name = name.trim().to_owned();
    if name.is_empty() {
        return Err(Error::msg("nome vazio"));
    }
    let id = state.db.lock().create_collection(&name, &color)?;
    state.lib.write().collections.push(Collection {
        id,
        name,
        color,
        members: Default::default(),
    });
    let _ = app.emit(EV_LIBRARY, ());
    Ok(id)
}

#[tauri::command]
pub async fn update_collection(
    app: AppHandle,
    state: St<'_>,
    id: i64,
    name: String,
    color: String,
) -> Result<()> {
    state.db.lock().update_collection(id, &name, &color)?;
    if let Some(c) = state
        .lib
        .write()
        .collections
        .iter_mut()
        .find(|c| c.id == id)
    {
        c.name = name;
        c.color = color;
    }
    let _ = app.emit(EV_LIBRARY, ());
    Ok(())
}

#[tauri::command]
pub async fn delete_collection(app: AppHandle, state: St<'_>, id: i64) -> Result<()> {
    state.db.lock().delete_collection(id)?;
    state.lib.write().collections.retain(|c| c.id != id);
    let _ = app.emit(EV_LIBRARY, ());
    Ok(())
}

#[tauri::command]
pub async fn set_in_collection(
    app: AppHandle,
    state: St<'_>,
    id: i64,
    family_ids: Vec<String>,
    member: bool,
) -> Result<()> {
    state
        .db
        .lock()
        .set_collection_items(id, &family_ids, member)?;
    if let Some(c) = state
        .lib
        .write()
        .collections
        .iter_mut()
        .find(|c| c.id == id)
    {
        for f in family_ids {
            if member {
                c.members.insert(f);
            } else {
                c.members.remove(&f);
            }
        }
    }
    let _ = app.emit(EV_LIBRARY, ());
    Ok(())
}

// ---------- Google ----------

#[tauri::command]
pub async fn refresh_google(app: AppHandle, state: St<'_>, force: bool) -> Result<u32> {
    refresh_catalog(&app, state.inner(), force).await
}

pub async fn refresh_catalog(app: &AppHandle, state: &Arc<AppState>, force: bool) -> Result<u32> {
    if !force && !state.catalog.read().is_stale() {
        return Ok(state.catalog.read().families.len() as u32);
    }
    let catalog = google::fetch_catalog(&state.http, &state.catalog_file).await?;
    let n = catalog.families.len() as u32;
    *state.catalog.write() = catalog;
    let _ = app.emit(EV_CATALOG, ());
    Ok(n)
}

// ---------- Settings ----------

#[tauri::command]
pub async fn get_setting(state: St<'_>, key: String) -> Result<Option<String>> {
    Ok(state.db.lock().kv_get(&key))
}

#[tauri::command]
pub async fn set_setting(state: St<'_>, key: String, value: String) -> Result<()> {
    state.db.lock().kv_set(&key, &value)
}
