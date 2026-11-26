//! Font activation without installation.
//!
//! * Linux   – symlinks in a Fontin-owned directory registered with fontconfig.
//! * Windows – `AddFontResourceW` (session scoped, re-applied on start).
//! * macOS   – `CTFontManagerRegisterFontsForURL` with session scope.
//!
//! The original files are never copied or moved.

use crate::error::Result;
use crate::model::Face;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

pub trait Activator: Send + Sync {
    fn activate(&self, faces: &[&Face]) -> Result<()>;
    fn deactivate(&self, faces: &[&Face]) -> Result<()>;
    /// Make the OS state match exactly `faces` (startup reconciliation).
    fn sync(&self, faces: &[&Face]) -> Result<()>;
    /// Block until the OS has seen pending changes (used right before exit).
    fn flush(&self) {}
}

pub fn platform(data_dir: &std::path::Path) -> Result<Box<dyn Activator>> {
    #[cfg(target_os = "linux")]
    return Ok(Box::new(linux::Fontconfig::new(data_dir)?));
    #[cfg(target_os = "macos")]
    {
        let _ = data_dir;
        return Ok(Box::new(macos::CoreText::default()));
    }
    #[cfg(windows)]
    {
        let _ = data_dir;
        return Ok(Box::new(windows::Gdi::default()));
    }
    #[allow(unreachable_code)]
    Err(crate::error::Error::msg("plataforma não suportada"))
}

/// Collections (.ttc) contain several faces in one file: activate files once.
pub(crate) fn unique_paths<'a>(faces: &'a [&'a Face]) -> Vec<&'a std::path::Path> {
    let mut seen = std::collections::HashSet::new();
    faces
        .iter()
        .map(|f| f.path.as_path())
        .filter(|p| seen.insert(*p))
        .collect()
}
