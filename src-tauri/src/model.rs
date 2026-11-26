//! Core data types shared by the library, the scanner and the commands.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub type FaceId = u64;
pub type SourceId = i64;

/// Source row reserved for fonts downloaded from Google Fonts.
pub const GOOGLE_SOURCE_ID: SourceId = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Folder,
    Google,
    /// Individual font files added one by one.
    Files,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceKind::Folder => "folder",
            SourceKind::Google => "google",
            SourceKind::Files => "files",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "google" => SourceKind::Google,
            "files" => SourceKind::Files,
            _ => SourceKind::Folder,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Source {
    pub id: SourceId,
    pub kind: SourceKind,
    pub path: PathBuf,
    pub name: String,
    /// Only for `SourceKind::Files`.
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FontFormat {
    Ttf,
    Otf,
    Ttc,
}

impl FontFormat {
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "ttf" => Some(Self::Ttf),
            "otf" => Some(Self::Otf),
            "ttc" | "otc" => Some(Self::Ttc),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ttf => "TTF",
            Self::Otf => "OTF",
            Self::Ttc => "TTC",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "OTF" => Self::Otf,
            "TTC" => Self::Ttc,
            _ => Self::Ttf,
        }
    }
}

/// A single font face (one style), possibly one entry of a collection file.
#[derive(Debug, Clone)]
pub struct Face {
    pub id: FaceId,
    pub source_id: SourceId,
    pub family_id: String,
    pub family: String,
    pub style: String,
    pub full_name: String,
    pub postscript: String,
    pub path: PathBuf,
    pub index: u32,
    pub weight: u16,
    pub italic: bool,
    pub width: u16,
    pub format: FontFormat,
    pub variable: bool,
    pub size: u64,
    pub mtime: i64,
    pub added_at: i64,
    /// xxh3 of the whole file: identical copies in different places share it.
    pub hash: u64,
}

pub fn face_id(path: &std::path::Path, index: u32) -> FaceId {
    let mut bytes = path.as_os_str().as_encoded_bytes().to_vec();
    bytes.extend_from_slice(&index.to_le_bytes());
    xxhash_rust::xxh3::xxh3_64(&bytes)
}

pub fn local_family_id(source_id: SourceId, family: &str) -> String {
    let key = format!("{source_id}\u{0}{}", family.to_lowercase());
    format!("l:{:016x}", xxhash_rust::xxh3::xxh3_64(key.as_bytes()))
}

pub fn google_family_id(family: &str) -> String {
    format!("g:{family}")
}

/// Fingerprint of a file's bytes, used to detect duplicate copies.
pub fn content_hash(data: &[u8]) -> u64 {
    // 0 is reserved for "not computed yet" (rows from older versions).
    xxhash_rust::xxh3::xxh3_64(data).max(1)
}

pub fn fmt_id(id: FaceId) -> String {
    format!("{id:016x}")
}

pub fn parse_id(s: &str) -> Option<FaceId> {
    u64::from_str_radix(s, 16).ok()
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Grouping of faces shown as one row in the UI.
#[derive(Debug, Clone)]
pub struct Family {
    pub id: String,
    pub name: String,
    pub name_lower: String,
    pub source_id: SourceId,
    pub faces: Vec<FaceId>,
    pub added_at: i64,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub members: std::collections::HashSet<String>,
}

// ---------- Wire types (frontend) ----------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyRow {
    pub id: String,
    pub name: String,
    pub provider: &'static str,
    pub styles: u32,
    pub format: &'static str,
    /// 0 = off, 1 = partially active, 2 = fully active
    pub active: u8,
    pub favorite: bool,
    pub downloaded: bool,
    pub preview_face: Option<String>,
    pub preview_weight: u16,
    pub preview_italic: bool,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FaceInfo {
    pub id: String,
    pub style: String,
    pub full_name: String,
    pub postscript: String,
    pub weight: u16,
    pub italic: bool,
    pub width: u16,
    pub variable: bool,
    pub format: &'static str,
    pub path: String,
    pub size: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyDetail {
    pub row: FamilyRow,
    pub faces: Vec<FaceInfo>,
    pub collections: Vec<i64>,
    pub designers: Vec<String>,
}

/// A file that was not indexed because an identical copy is already there.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateFile {
    pub path: String,
    /// Family of the copy already in the library.
    pub family: String,
    /// Path of the copy already in the library.
    pub existing: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderNode {
    pub path: String,
    pub name: String,
    pub count: u32,
    pub children: Vec<FolderNode>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub id: SourceId,
    pub kind: SourceKind,
    pub name: String,
    pub path: String,
    pub count: u32,
    pub active: u32,
    pub tree: Vec<FolderNode>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionInfo {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub count: u32,
    pub active: u32,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    pub all: u32,
    pub recent: u32,
    pub favorites: u32,
    pub active: u32,
    pub inactive: u32,
    pub google: u32,
    pub google_active: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub counts: Counts,
    pub sources: Vec<SourceInfo>,
    pub collections: Vec<CollectionInfo>,
    pub google_ready: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum Scope {
    All,
    Recent,
    Favorites,
    Active,
    Inactive,
    Google,
    Collection(i64),
    Source(SourceId),
    Folder(String),
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Sort {
    #[default]
    Name,
    Recent,
    Popular,
    Styles,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    pub scope: Scope,
    #[serde(default)]
    pub search: String,
    #[serde(default)]
    pub sort: Sort,
    #[serde(default)]
    pub category: Option<String>,
}
