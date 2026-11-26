use super::{Activator, unique_paths};
use crate::error::Result;
use crate::model::Face;
use std::collections::HashSet;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

const CONF_NAME: &str = "90-fontin.conf";

pub struct Fontconfig {
    dir: PathBuf,
    refresh: mpsc::Sender<()>,
}

impl Fontconfig {
    pub fn new(data_dir: &Path) -> Result<Self> {
        let dir = data_dir.join("active");
        std::fs::create_dir_all(&dir)?;
        write_conf(&dir)?;

        // Debounced `fc-cache` so bulk toggles cost one refresh.
        let (tx, rx) = mpsc::channel::<()>();
        let cache_dir = dir.clone();
        std::thread::Builder::new()
            .name("fontin-fc-cache".into())
            .spawn(move || {
                while rx.recv().is_ok() {
                    while rx.recv_timeout(Duration::from_millis(400)).is_ok() {}
                    // `-f`: fontconfig compares dir mtimes at 1s granularity, so two
                    // toggles within the same second would otherwise leave a stale cache.
                    let status = std::process::Command::new("fc-cache")
                        .arg("-f")
                        .arg(&cache_dir)
                        .status();
                    if let Err(e) = status {
                        log::warn!("fc-cache failed: {e}");
                    }
                }
            })?;
        Ok(Self { dir, refresh: tx })
    }

    fn link_name(&self, path: &Path) -> PathBuf {
        // Hash keeps names unique even for identical file names in different folders.
        let h = xxhash_rust::xxh3::xxh3_64(path.as_os_str().as_encoded_bytes());
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.dir.join(format!("{h:016x}-{name}"))
    }

    fn touch(&self) {
        let _ = self.refresh.send(());
    }
}

fn write_conf(dir: &Path) -> Result<()> {
    let Some(cfg) = dirs::config_dir() else {
        return Ok(());
    };
    let conf_dir = cfg.join("fontconfig").join("conf.d");
    std::fs::create_dir_all(&conf_dir)?;
    let body = format!(
        "<?xml version=\"1.0\"?>\n<!DOCTYPE fontconfig SYSTEM \"urn:fontconfig:fonts.dtd\">\n\
         <!-- Managed by Fontin: fonts activated without installation. -->\n\
         <fontconfig>\n  <dir>{}</dir>\n</fontconfig>\n",
        xml_escape(&dir.to_string_lossy())
    );
    let target = conf_dir.join(CONF_NAME);
    if std::fs::read_to_string(&target).ok().as_deref() != Some(body.as_str()) {
        std::fs::write(target, body)?;
    }
    Ok(())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

impl Activator for Fontconfig {
    fn activate(&self, faces: &[&Face]) -> Result<()> {
        for p in unique_paths(faces) {
            let link = self.link_name(p);
            if std::fs::symlink_metadata(&link).is_err() {
                symlink(p, &link)?;
            }
        }
        self.touch();
        Ok(())
    }

    fn deactivate(&self, faces: &[&Face]) -> Result<()> {
        for p in unique_paths(faces) {
            match std::fs::remove_file(self.link_name(p)) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        self.touch();
        Ok(())
    }

    fn flush(&self) {
        // The debounced worker dies with the process, so refresh synchronously.
        let _ = std::process::Command::new("fc-cache")
            .arg("-f")
            .arg(&self.dir)
            .status();
    }

    fn sync(&self, faces: &[&Face]) -> Result<()> {
        let wanted: HashSet<PathBuf> = unique_paths(faces)
            .into_iter()
            .map(|p| self.link_name(p))
            .collect();
        for entry in std::fs::read_dir(&self.dir)?.flatten() {
            let p = entry.path();
            // Drop links that are no longer wanted or whose target disappeared.
            if !wanted.contains(&p) || std::fs::metadata(&p).is_err() {
                let _ = std::fs::remove_file(&p);
            }
        }
        self.activate(faces)
    }
}
