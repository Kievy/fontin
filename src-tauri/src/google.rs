//! Google Fonts provider: the catalog comes from Google's public metadata
//! endpoint (no API key) and is cached on disk. Files are only fetched when
//! a family is activated, then kept in the local cache.

use crate::error::{Error, Result};
use crate::model::*;
use futures_util::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const METADATA_URL: &str = "https://fonts.google.com/metadata/fonts";
const DOWNLOAD_LIST_URL: &str = "https://fonts.google.com/download/list?family=";
const CATALOG_MAX_AGE: i64 = 60 * 60 * 24;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleFamily {
    pub family: String,
    #[serde(skip)]
    pub family_lower: String,
    pub category: String,
    pub variants: Vec<String>,
    pub popularity: u32,
    pub designers: Vec<String>,
    pub variable: bool,
}

#[derive(Default)]
pub struct Catalog {
    pub families: Vec<GoogleFamily>,
    index: HashMap<String, usize>,
    pub fetched_at: i64,
}

impl Catalog {
    fn new(mut families: Vec<GoogleFamily>, fetched_at: i64) -> Self {
        families.sort_by_key(|f| f.popularity);
        for f in &mut families {
            f.family_lower = f.family.to_lowercase();
        }
        let index = families
            .iter()
            .enumerate()
            .map(|(i, f)| (f.family.clone(), i))
            .collect();
        Self {
            families,
            index,
            fetched_at,
        }
    }

    pub fn get(&self, family: &str) -> Option<&GoogleFamily> {
        self.index.get(family).map(|&i| &self.families[i])
    }

    pub fn is_stale(&self) -> bool {
        self.families.is_empty() || now() - self.fetched_at > CATALOG_MAX_AGE
    }
}

#[derive(Serialize, Deserialize)]
struct CacheFile {
    fetched_at: i64,
    families: Vec<GoogleFamily>,
}

pub fn load_cached(path: &Path) -> Catalog {
    std::fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice::<CacheFile>(&b).ok())
        .map(|c| Catalog::new(c.families, c.fetched_at))
        .unwrap_or_default()
}

/// Google prefixes some JSON responses with an XSSI guard.
fn strip_guard(body: &str) -> &str {
    body.trim_start().strip_prefix(")]}'").unwrap_or(body)
}

pub async fn fetch_catalog(http: &reqwest::Client, cache: &Path) -> Result<Catalog> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Meta {
        family_metadata_list: Vec<Entry>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Entry {
        family: String,
        category: String,
        #[serde(default)]
        fonts: HashMap<String, serde_json::Value>,
        #[serde(default)]
        axes: Vec<serde_json::Value>,
        #[serde(default)]
        popularity: u32,
        #[serde(default)]
        designers: Vec<String>,
    }

    let body = http
        .get(METADATA_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let meta: Meta = serde_json::from_str(strip_guard(&body))?;
    let families: Vec<GoogleFamily> = meta
        .family_metadata_list
        .into_iter()
        .map(|e| {
            let mut variants: Vec<String> = e.fonts.into_keys().collect();
            variants.sort_by_key(|v| {
                (
                    v.ends_with('i'),
                    v.trim_end_matches('i').parse::<u32>().unwrap_or(400),
                )
            });
            GoogleFamily {
                family: e.family,
                family_lower: String::new(),
                category: e.category,
                variants,
                popularity: e.popularity,
                designers: e.designers,
                variable: !e.axes.is_empty(),
            }
        })
        .collect();

    let fetched_at = now();
    let tmp = cache.with_extension("tmp");
    std::fs::write(
        &tmp,
        serde_json::to_vec(&CacheFile {
            fetched_at,
            families: families.clone(),
        })?,
    )?;
    std::fs::rename(&tmp, cache)?;
    Ok(Catalog::new(families, fetched_at))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadList {
    manifest: Manifest,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    #[serde(default)]
    file_refs: Vec<FileRef>,
}
#[derive(Deserialize, Clone)]
struct FileRef {
    filename: String,
    url: String,
}

/// Choose which files to activate. Static instances are preferred for app
/// compatibility; Google bundles sibling families (e.g. "Roboto Condensed")
/// in the same archive, so statics are filtered by the family prefix.
fn pick_files(family: &str, refs: Vec<FileRef>) -> Vec<FileRef> {
    let is_font = |f: &FileRef| {
        let l = f.filename.to_ascii_lowercase();
        l.ends_with(".ttf") || l.ends_with(".otf")
    };
    let refs: Vec<FileRef> = refs.into_iter().filter(is_font).collect();
    let prefix = format!("{}-", family.replace(' ', ""));
    let statics: Vec<FileRef> = refs
        .iter()
        .filter(|f| f.filename.contains('/'))
        .cloned()
        .collect();
    let own_statics: Vec<FileRef> = statics
        .iter()
        .filter(|f| {
            f.filename
                .rsplit('/')
                .next()
                .is_some_and(|n| n.starts_with(&prefix))
        })
        .cloned()
        .collect();
    if !own_statics.is_empty() {
        return own_statics;
    }
    let top: Vec<FileRef> = refs
        .iter()
        .filter(|f| !f.filename.contains('/'))
        .cloned()
        .collect();
    if !top.is_empty() {
        return top;
    }
    statics
}

pub fn family_dir(root: &Path, family: &str) -> PathBuf {
    let slug: String = family
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    root.join(slug)
}

/// Download a family into the cache and return the parsed faces.
pub async fn download_family(
    http: &reqwest::Client,
    root: &Path,
    family: &str,
    mut progress: impl FnMut(usize, usize),
) -> Result<Vec<Face>> {
    let url = format!("{DOWNLOAD_LIST_URL}{}", urlencoding::encode(family));
    let body = http
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let list: DownloadList = serde_json::from_str(strip_guard(&body))?;
    let files = pick_files(family, list.manifest.file_refs);
    if files.is_empty() {
        return Err(crate::i18n::Msg::GoogleNoFiles(family.to_owned()).into());
    }

    let dir = family_dir(root, family);
    tokio::fs::create_dir_all(&dir).await?;
    let total = files.len();
    progress(0, total);

    let mut done = 0;
    let mut paths = Vec::with_capacity(total);
    let mut results = stream::iter(files.into_iter().map(|f| {
        let http = http.clone();
        let dest = dir.join(f.filename.rsplit('/').next().unwrap_or(&f.filename));
        async move {
            if tokio::fs::metadata(&dest).await.is_err() {
                let bytes = http
                    .get(&f.url)
                    .send()
                    .await?
                    .error_for_status()?
                    .bytes()
                    .await?;
                let tmp = dest.with_extension("part");
                tokio::fs::write(&tmp, &bytes).await?;
                tokio::fs::rename(&tmp, &dest).await?;
            }
            Ok::<PathBuf, Error>(dest)
        }
    }))
    .buffer_unordered(6);

    while let Some(r) = results.next().await {
        paths.push(r?);
        done += 1;
        progress(done, total);
    }

    let family = family.to_owned();
    let faces = tokio::task::spawn_blocking(move || {
        let added_at = now();
        paths
            .iter()
            .filter_map(|p| {
                let md = std::fs::metadata(p).ok()?;
                let fmt = FontFormat::from_ext(p.extension()?.to_str()?)?;
                crate::scanner::parse_file(
                    p,
                    fmt,
                    GOOGLE_SOURCE_ID,
                    md.len(),
                    crate::scanner::mtime_of(&md),
                    added_at,
                    Some(&family),
                )
            })
            .flatten()
            .collect::<Vec<_>>()
    })
    .await
    .map_err(|e| Error::msg(e.to_string()))?;
    Ok(faces)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Network test: `cargo test -- --ignored google`
    #[tokio::test]
    #[ignore]
    async fn google_catalog_and_download() {
        let http = reqwest::Client::builder()
            .user_agent("Fontin-test")
            .build()
            .unwrap();
        let dir = std::env::temp_dir().join(format!("fontin-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cat = fetch_catalog(&http, &dir.join("c.json")).await.unwrap();
        assert!(cat.families.len() > 1000, "catalog too small");
        assert!(cat.get("Roboto").is_some());

        for fam in ["Roboto", "Open Sans", "Buda"] {
            let faces = download_family(&http, &dir, fam, |_, _| {}).await.unwrap();
            eprintln!(
                "{fam}: {} faces: {:?}",
                faces.len(),
                faces.iter().map(|f| &f.style).collect::<Vec<_>>()
            );
            assert!(!faces.is_empty());
            assert!(
                faces
                    .iter()
                    .all(|f| f.family == fam && f.family_id == google_family_id(fam))
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
