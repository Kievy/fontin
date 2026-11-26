use crate::activation::{self, Activator};
use crate::db::Db;
use crate::error::Result;
use crate::google::{self, Catalog};
use crate::library::Library;
use crate::model::*;
use crate::settings::{self, Settings};
use parking_lot::{Mutex, RwLock};
use std::collections::HashSet;
use std::path::PathBuf;

pub struct AppState {
    pub lib: RwLock<Library>,
    pub db: Mutex<Db>,
    pub catalog: RwLock<Catalog>,
    pub activator: Box<dyn Activator>,
    pub http: reqwest::Client,
    pub google_dir: PathBuf,
    pub catalog_file: PathBuf,
    /// Google families currently being downloaded (prevents double clicks).
    pub downloading: Mutex<HashSet<String>>,
    pub settings: RwLock<Settings>,
}

impl AppState {
    pub fn init(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let google_dir = data_dir.join("google");
        std::fs::create_dir_all(&google_dir)?;
        let catalog_file = data_dir.join("google-catalog.json");

        let db = Db::open(&data_dir.join("library.db"))?;
        let lib = Library::from_loaded(db.load()?);
        let settings = Settings::parse(db.kv_get(settings::KEY));
        let activator = activation::platform(&data_dir)?;
        let catalog = google::load_cached(&catalog_file);
        let http = reqwest::Client::builder()
            .user_agent(concat!("Fontin/", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self {
            lib: RwLock::new(lib),
            db: Mutex::new(db),
            catalog: RwLock::new(catalog),
            activator,
            http,
            google_dir,
            catalog_file,
            downloading: Mutex::new(HashSet::new()),
            settings: RwLock::new(settings),
        })
    }

    /// Re-apply persisted activation to the OS.
    pub fn sync_activation(&self) -> Result<()> {
        let lib = self.lib.read();
        let faces: Vec<&Face> = lib
            .active
            .iter()
            .filter_map(|id| lib.faces.get(id))
            .collect();
        self.activator.sync(&faces)
    }

    /// Pull every font out of the OS without touching the saved state.
    pub fn release_all(&self) {
        let lib = self.lib.read();
        let faces: Vec<&Face> = lib.active.iter().filter_map(|id| lib.faces.get(id)).collect();
        if let Err(e) = self.activator.deactivate(&faces) {
            log::error!("deactivate on exit failed: {e}");
        }
        self.activator.flush();
    }

    /// Activate/deactivate faces: OS first, then persistence + index.
    pub fn set_faces_active(&self, ids: &[FaceId], on: bool) -> Result<()> {
        let faces: Vec<Face> = {
            let lib = self.lib.read();
            ids.iter()
                .filter_map(|id| lib.faces.get(id).cloned())
                .collect()
        };
        let refs: Vec<&Face> = faces.iter().collect();
        if on {
            self.activator.activate(&refs)?;
        } else {
            self.activator.deactivate(&refs)?;
        }
        let ids: Vec<FaceId> = faces.iter().map(|f| f.id).collect();
        self.db.lock().set_active(&ids, on)?;
        let mut lib = self.lib.write();
        for id in ids {
            if on {
                lib.active.insert(id);
            } else {
                lib.active.remove(&id);
            }
        }
        Ok(())
    }

    pub fn family_face_ids(&self, family_id: &str) -> Option<Vec<FaceId>> {
        self.lib
            .read()
            .families
            .get(family_id)
            .map(|f| f.faces.clone())
    }

    pub fn overview(&self) -> Overview {
        let lib = self.lib.read();
        let catalog = self.catalog.read();
        Overview {
            counts: lib.counts(&catalog),
            sources: lib.source_infos(),
            collections: lib.collection_infos(),
            google_ready: !catalog.families.is_empty(),
        }
    }
}
