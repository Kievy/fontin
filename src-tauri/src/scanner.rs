//! Fast, parallel font discovery. Files are memory-mapped and only the
//! tables needed for naming/classification are touched by `ttf-parser`.

use crate::model::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use ttf_parser::name_id;

pub struct ScanResult {
    pub added: Vec<Face>,
    /// Faces whose file vanished or changed and must be dropped.
    pub removed: Vec<FaceId>,
    pub unchanged: usize,
}

/// Snapshot of what is already indexed for a source: path -> (size, mtime, face ids).
pub type Known = HashMap<PathBuf, (u64, i64, Vec<FaceId>)>;

pub fn scan_source(source_id: SourceId, root: &Path, known: &Known) -> ScanResult {
    let files: Vec<(PathBuf, u64, i64, FontFormat)> = walkdir::WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            let fmt = FontFormat::from_ext(e.path().extension()?.to_str()?)?;
            // Skip macOS resource-fork junk.
            if e.file_name().to_string_lossy().starts_with("._") {
                return None;
            }
            let md = e.metadata().ok()?;
            Some((e.into_path(), md.len(), mtime_of(&md), fmt))
        })
        .collect();

    let mut seen: HashMap<&Path, ()> = HashMap::with_capacity(files.len());
    let mut to_parse = Vec::new();
    let mut removed = Vec::new();
    let mut unchanged = 0;

    for f in &files {
        seen.insert(&f.0, ());
        match known.get(&f.0) {
            Some((size, mtime, _)) if *size == f.1 && *mtime == f.2 => unchanged += 1,
            Some((_, _, ids)) => {
                removed.extend(ids.iter().copied());
                to_parse.push(f);
            }
            None => to_parse.push(f),
        }
    }
    for (path, (_, _, ids)) in known {
        if !seen.contains_key(path.as_path()) {
            removed.extend(ids.iter().copied());
        }
    }

    let now = now();
    let added: Vec<Face> = to_parse
        .par_iter()
        .flat_map_iter(|(path, size, mtime, fmt)| {
            parse_file(path, *fmt, source_id, *size, *mtime, now, None).unwrap_or_default()
        })
        .collect();

    ScanResult {
        added,
        removed,
        unchanged,
    }
}

pub fn mtime_of(md: &std::fs::Metadata) -> i64 {
    md.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Parse every face in a font file. `family_override` forces the grouping
/// (used for Google Fonts, whose files may carry sub-family names).
pub fn parse_file(
    path: &Path,
    fmt: FontFormat,
    source_id: SourceId,
    size: u64,
    mtime: i64,
    added_at: i64,
    family_override: Option<&str>,
) -> Option<Vec<Face>> {
    let file = std::fs::File::open(path).ok()?;
    // SAFETY: read-only mapping; a concurrent truncation would at worst make
    // ttf-parser see garbage, which it handles by returning an error.
    let map = unsafe { memmap2::Mmap::map(&file) }.ok()?;
    let data: &[u8] = &map;
    let count = ttf_parser::fonts_in_collection(data).unwrap_or(1);

    let mut out = Vec::with_capacity(count as usize);
    for index in 0..count {
        let Ok(face) = ttf_parser::Face::parse(data, index) else {
            continue;
        };
        let family = name(&face, name_id::TYPOGRAPHIC_FAMILY)
            .or_else(|| name(&face, name_id::FAMILY))
            .unwrap_or_else(|| file_stem(path));
        let style = name(&face, name_id::TYPOGRAPHIC_SUBFAMILY)
            .or_else(|| name(&face, name_id::SUBFAMILY))
            .unwrap_or_else(|| "Regular".into());
        let full_name =
            name(&face, name_id::FULL_NAME).unwrap_or_else(|| format!("{family} {style}"));
        let postscript = name(&face, name_id::POST_SCRIPT_NAME).unwrap_or_default();

        let family = family_override.map(str::to_owned).unwrap_or(family);
        let family_id = match family_override {
            Some(f) => google_family_id(f),
            None => local_family_id(source_id, &family),
        };

        out.push(Face {
            id: face_id(path, index),
            source_id,
            family_id,
            family,
            style,
            full_name,
            postscript,
            path: path.to_path_buf(),
            index,
            weight: face.weight().to_number(),
            italic: face.is_italic() || face.is_oblique(),
            width: face.width().to_number(),
            format: fmt,
            variable: face.is_variable(),
            size,
            mtime,
            added_at,
        });
    }
    Some(out)
}

fn file_stem(p: &Path) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Pick the best-language record for a name id (English first).
fn name(face: &ttf_parser::Face, id: u16) -> Option<String> {
    let mut fallback = None;
    for rec in face.names() {
        if rec.name_id != id {
            continue;
        }
        let Some(s) = rec.to_string() else { continue };
        let s = s.trim().to_owned();
        if s.is_empty() {
            continue;
        }
        let english = matches!(rec.platform_id, ttf_parser::PlatformId::Unicode)
            || (rec.platform_id == ttf_parser::PlatformId::Windows && rec.language_id == 0x0409)
            || (rec.platform_id == ttf_parser::PlatformId::Macintosh && rec.language_id == 0);
        if english {
            return Some(s);
        }
        fallback.get_or_insert(s);
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_system_fonts_incrementally() {
        let root = Path::new("/usr/share/fonts");
        if !root.exists() {
            return;
        }
        let t = std::time::Instant::now();
        let r = scan_source(7, root, &Known::new());
        eprintln!("full scan: {} faces in {:?}", r.added.len(), t.elapsed());
        assert!(!r.added.is_empty());
        assert!(r.added.iter().all(|f| !f.family.is_empty() && f.weight > 0));

        let mut known = Known::new();
        for f in &r.added {
            known
                .entry(f.path.clone())
                .or_insert((f.size, f.mtime, vec![]))
                .2
                .push(f.id);
        }
        let t = std::time::Instant::now();
        let again = scan_source(7, root, &known);
        eprintln!(
            "incremental: {} unchanged in {:?}",
            again.unchanged,
            t.elapsed()
        );
        assert!(again.added.is_empty() && again.removed.is_empty());
    }
}
