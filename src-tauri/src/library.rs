//! In-memory index: every query is served from here, never from disk.

use crate::google::Catalog;
use crate::model::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

const RECENT_LIMIT: usize = 100;

#[derive(Default)]
pub struct Library {
    pub sources: Vec<Source>,
    pub faces: HashMap<FaceId, Face>,
    pub families: HashMap<String, Family>,
    pub active: HashSet<FaceId>,
    pub favorites: HashSet<String>,
    pub collections: Vec<Collection>,
    /// Files the user chose to keep even though an identical copy exists.
    pub allowed_duplicates: HashSet<PathBuf>,
    recent: HashSet<String>,
}

impl Library {
    pub fn from_loaded(l: crate::db::Loaded) -> Self {
        let mut lib = Library {
            sources: l.sources,
            faces: l.faces.into_iter().map(|f| (f.id, f)).collect(),
            active: l.active,
            favorites: l.favorites,
            collections: l.collections,
            allowed_duplicates: l.allowed_duplicates,
            ..Default::default()
        };
        lib.active.retain(|id| lib.faces.contains_key(id));
        lib.rebuild();
        lib
    }

    /// Regroup faces into families. O(n), called after bulk changes.
    pub fn rebuild(&mut self) {
        let mut families: HashMap<String, Family> =
            HashMap::with_capacity(self.families.len().max(64));
        for f in self.faces.values() {
            let fam = families
                .entry(f.family_id.clone())
                .or_insert_with(|| Family {
                    id: f.family_id.clone(),
                    name: f.family.clone(),
                    name_lower: f.family.to_lowercase(),
                    source_id: f.source_id,
                    faces: Vec::new(),
                    added_at: f.added_at,
                });
            fam.faces.push(f.id);
            fam.added_at = fam.added_at.max(f.added_at);
        }
        for fam in families.values_mut() {
            let faces = &self.faces;
            fam.faces.sort_by_key(|id| {
                let f = &faces[id];
                (f.width.abs_diff(5), f.weight, f.italic, f.style.clone())
            });
        }
        let mut by_time: Vec<&Family> = families.values().collect();
        by_time.sort_by_key(|f| std::cmp::Reverse(f.added_at));
        self.recent = by_time
            .iter()
            .take(RECENT_LIMIT)
            .map(|f| f.id.clone())
            .collect();
        self.families = families;
    }

    pub fn source(&self, id: SourceId) -> Option<&Source> {
        self.sources.iter().find(|s| s.id == id)
    }

    pub fn active_state(&self, fam: &Family) -> u8 {
        let n = fam
            .faces
            .iter()
            .filter(|id| self.active.contains(id))
            .count();
        match n {
            0 => 0,
            n if n == fam.faces.len() => 2,
            _ => 1,
        }
    }

    /// The face used for previews: regular upright, closest to 400.
    pub fn preview_face(&self, fam: &Family) -> Option<&Face> {
        fam.faces
            .iter()
            .map(|id| &self.faces[id])
            .min_by_key(|f| (f.italic, f.width.abs_diff(5), f.weight.abs_diff(400)))
    }

    pub fn row(&self, fam: &Family, catalog: &Catalog) -> FamilyRow {
        let preview = self.preview_face(fam);
        let google = fam.source_id == GOOGLE_SOURCE_ID;
        let format = preview.map(|f| f.format.as_str()).unwrap_or("TTF");
        FamilyRow {
            id: fam.id.clone(),
            name: fam.name.clone(),
            provider: if google { "google" } else { "local" },
            styles: fam.faces.len() as u32,
            format,
            active: self.active_state(fam),
            favorite: self.favorites.contains(&fam.id),
            downloaded: true,
            preview_face: preview.map(|f| fmt_id(f.id)),
            preview_weight: preview.map(|f| f.weight).unwrap_or(400),
            preview_italic: preview.map(|f| f.italic).unwrap_or(false),
            category: if google {
                catalog.get(&fam.name).map(|g| g.category.clone())
            } else {
                None
            },
        }
    }

    pub fn google_row(&self, g: &crate::google::GoogleFamily) -> FamilyRow {
        let id = google_family_id(&g.family);
        let local = self.families.get(&id);
        let preview = local.and_then(|f| self.preview_face(f));
        // Variant keys look like "400" / "700i": pick the upright closest to 400.
        let (weight, italic) = g
            .variants
            .iter()
            .map(|v| {
                (
                    v.trim_end_matches('i').parse::<u16>().unwrap_or(400),
                    v.ends_with('i'),
                )
            })
            .min_by_key(|(w, i)| (*i, w.abs_diff(400)))
            .unwrap_or((400, false));
        FamilyRow {
            name: g.family.clone(),
            provider: "google",
            styles: local
                .map(|f| f.faces.len() as u32)
                .unwrap_or(g.variants.len() as u32),
            format: "TTF",
            active: local.map(|f| self.active_state(f)).unwrap_or(0),
            favorite: self.favorites.contains(&id),
            downloaded: local.is_some(),
            preview_face: preview.map(|f| fmt_id(f.id)),
            preview_weight: weight,
            preview_italic: italic,
            category: Some(g.category.clone()),
            id,
        }
    }

    pub fn query(&self, q: &Query, catalog: &Catalog) -> Vec<FamilyRow> {
        let needle = q.search.trim().to_lowercase();
        let category = q.category.as_deref().filter(|c| !c.is_empty());

        if let Scope::Google = q.scope {
            let mut rows: Vec<(u32, FamilyRow)> = catalog
                .families
                .iter()
                .filter(|g| needle.is_empty() || g.family_lower.contains(&needle))
                .filter(|g| category.is_none_or(|c| g.category == c))
                .map(|g| (g.popularity, self.google_row(g)))
                .collect();
            match q.sort {
                Sort::Popular | Sort::Recent => rows.sort_by_key(|r| r.0),
                Sort::Styles => rows.sort_by_key(|r| std::cmp::Reverse(r.1.styles)),
                Sort::Name => rows.sort_by_key(|a| a.1.name.to_lowercase()),
            }
            return rows.into_iter().map(|r| r.1).collect();
        }

        let folder = match &q.scope {
            Scope::Folder(p) => Some(PathBuf::from(p)),
            _ => None,
        };
        let collection = match q.scope {
            Scope::Collection(id) => self.collections.iter().find(|c| c.id == id),
            _ => None,
        };

        let mut fams: Vec<&Family> = self
            .families
            .values()
            .filter(|f| needle.is_empty() || f.name_lower.contains(&needle))
            .filter(|f| match &q.scope {
                Scope::All | Scope::Google => true,
                Scope::Recent => self.recent.contains(&f.id),
                Scope::Favorites => self.favorites.contains(&f.id),
                Scope::Active => self.active_state(f) > 0,
                Scope::Inactive => self.active_state(f) == 0,
                Scope::Collection(_) => collection.is_some_and(|c| c.members.contains(&f.id)),
                Scope::Source(id) => f.source_id == *id,
                Scope::Folder(_) => {
                    let dir = folder.as_deref().unwrap();
                    f.faces
                        .iter()
                        .any(|id| self.faces[id].path.starts_with(dir))
                }
            })
            .filter(|f| {
                category.is_none_or(|c| {
                    f.source_id == GOOGLE_SOURCE_ID
                        && catalog.get(&f.name).is_some_and(|g| g.category == c)
                })
            })
            .collect();

        match q.sort {
            Sort::Name | Sort::Popular => fams.sort_by(|a, b| a.name_lower.cmp(&b.name_lower)),
            Sort::Recent => fams.sort_by_key(|f| std::cmp::Reverse(f.added_at)),
            Sort::Styles => fams.sort_by(|a, b| {
                b.faces
                    .len()
                    .cmp(&a.faces.len())
                    .then(a.name_lower.cmp(&b.name_lower))
            }),
        }

        // Favorites and collections may reference Google families that were never downloaded.
        let mut rows: Vec<FamilyRow> = fams.into_iter().map(|f| self.row(f, catalog)).collect();
        let remote = match q.scope {
            Scope::Favorites => Some(&self.favorites),
            Scope::Collection(_) => collection.map(|c| &c.members),
            _ => None,
        };
        for id in remote
            .into_iter()
            .flatten()
            .filter(|id| !self.families.contains_key(*id))
        {
            if let Some(g) = id.strip_prefix("g:").and_then(|n| catalog.get(n))
                && (needle.is_empty() || g.family_lower.contains(&needle))
            {
                rows.push(self.google_row(g));
            }
        }
        rows
    }

    pub fn detail(&self, id: &str, catalog: &Catalog) -> Option<FamilyDetail> {
        let collections = self
            .collections
            .iter()
            .filter(|c| c.members.contains(id))
            .map(|c| c.id)
            .collect();
        let gname = id.strip_prefix("g:");
        let designers = gname
            .and_then(|n| catalog.get(n))
            .map(|g| g.designers.clone())
            .unwrap_or_default();

        if let Some(fam) = self.families.get(id) {
            let faces = fam
                .faces
                .iter()
                .map(|fid| {
                    let f = &self.faces[fid];
                    FaceInfo {
                        id: fmt_id(f.id),
                        style: f.style.clone(),
                        full_name: f.full_name.clone(),
                        postscript: f.postscript.clone(),
                        weight: f.weight,
                        italic: f.italic,
                        width: f.width,
                        variable: f.variable,
                        format: f.format.as_str(),
                        path: f.path.to_string_lossy().into_owned(),
                        size: f.size,
                        active: self.active.contains(fid),
                    }
                })
                .collect();
            return Some(FamilyDetail {
                row: self.row(fam, catalog),
                faces,
                collections,
                designers,
            });
        }
        let g = catalog.get(gname?)?;
        Some(FamilyDetail {
            row: self.google_row(g),
            faces: vec![],
            collections,
            designers,
        })
    }

    pub fn counts(&self, catalog: &Catalog) -> Counts {
        let mut c = Counts {
            all: self.families.len() as u32,
            recent: self.recent.len() as u32,
            ..Default::default()
        };
        for f in self.families.values() {
            if self.active_state(f) > 0 {
                c.active += 1;
                if f.source_id == GOOGLE_SOURCE_ID {
                    c.google_active += 1;
                }
            } else {
                c.inactive += 1;
            }
        }
        c.favorites = self.favorites.len() as u32;
        c.google = catalog.families.len() as u32;
        c
    }

    pub fn source_infos(&self) -> Vec<SourceInfo> {
        let mut by_source: HashMap<SourceId, Vec<&Family>> = HashMap::new();
        for f in self.families.values() {
            by_source.entry(f.source_id).or_default().push(f);
        }
        self.sources
            .iter()
            .filter(|s| s.kind != SourceKind::Google)
            .map(|s| {
                let fams = by_source.get(&s.id).map(Vec::as_slice).unwrap_or(&[]);
                SourceInfo {
                    id: s.id,
                    kind: s.kind,
                    name: s.name.clone(),
                    path: s.path.to_string_lossy().into_owned(),
                    count: fams.len() as u32,
                    active: fams.iter().filter(|f| self.active_state(f) > 0).count() as u32,
                    tree: if s.kind == SourceKind::Folder {
                        self.folder_tree(&s.path, fams)
                    } else {
                        vec![]
                    },
                }
            })
            .collect()
    }

    /// Directory tree under a source root, counting families per folder.
    fn folder_tree(&self, root: &Path, fams: &[&Family]) -> Vec<FolderNode> {
        // (families, active families) per directory.
        let mut counts: BTreeMap<PathBuf, (u32, u32)> = BTreeMap::new();
        let mut dirs: HashSet<&Path> = HashSet::new();
        for fam in fams {
            let on = (self.active_state(fam) > 0) as u32;
            dirs.clear();
            for id in &fam.faces {
                let mut cur = self.faces[id].path.parent();
                while let Some(d) = cur {
                    if d == root || !d.starts_with(root) {
                        break;
                    }
                    dirs.insert(d);
                    cur = d.parent();
                }
            }
            for d in &dirs {
                let c = counts.entry(d.to_path_buf()).or_default();
                c.0 += 1;
                c.1 += on;
            }
        }
        build_children(root, &counts)
    }

    pub fn collection_infos(&self) -> Vec<CollectionInfo> {
        self.collections
            .iter()
            .map(|c| CollectionInfo {
                id: c.id,
                name: c.name.clone(),
                color: c.color.clone(),
                count: c.members.len() as u32,
                active: c
                    .members
                    .iter()
                    .filter(|id| {
                        self.families
                            .get(*id)
                            .is_some_and(|f| self.active_state(f) > 0)
                    })
                    .count() as u32,
            })
            .collect()
    }

    /// Index of what is on disk for a source, for incremental rescans.
    pub fn known(&self, source_id: SourceId) -> crate::scanner::Known {
        let mut k: crate::scanner::Known = HashMap::new();
        for f in self.faces.values().filter(|f| f.source_id == source_id) {
            k.entry(f.path.clone())
                .or_insert((f.size, f.mtime, Vec::new()))
                .2
                .push(f.id);
        }
        k
    }

    /// An indexed face with the same bytes as (`hash`, `index`), if any.
    pub fn same_content(&self, hash: u64, index: u32) -> Option<&Face> {
        self.faces
            .values()
            .find(|f| f.hash == hash && f.index == index)
    }

    /// Content fingerprints already in the library, excluding `ignore` faces.
    pub fn content_keys(&self, ignore: &[FaceId]) -> HashSet<(u64, u32)> {
        let ignore: HashSet<&FaceId> = ignore.iter().collect();
        self.faces
            .values()
            .filter(|f| f.hash != 0 && !ignore.contains(&f.id))
            .map(|f| (f.hash, f.index))
            .collect()
    }

    pub fn apply_faces(&mut self, added: Vec<Face>, removed: &[FaceId]) {
        for id in removed {
            self.faces.remove(id);
            self.active.remove(id);
        }
        for f in added {
            self.faces.insert(f.id, f);
        }
        self.rebuild();
    }
}

fn build_children(parent: &Path, counts: &BTreeMap<PathBuf, (u32, u32)>) -> Vec<FolderNode> {
    // Paths sharing a prefix are contiguous in a BTreeMap, so only scan that range.
    counts
        .range(parent.to_path_buf()..)
        .take_while(|(p, _)| p.starts_with(parent))
        .filter(|(p, _)| p.parent() == Some(parent))
        .map(|(p, n)| FolderNode {
            path: p.to_string_lossy().into_owned(),
            name: p
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default(),
            count: n.0,
            active: n.1,
            children: build_children(p, counts),
        })
        .collect()
}
