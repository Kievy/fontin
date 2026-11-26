//! SQLite persistence. The whole library is loaded into memory on start;
//! the database only records mutations.

use crate::error::Result;
use crate::model::*;
use rusqlite::{Connection, params};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct Db {
    conn: Connection,
}

pub struct Loaded {
    pub sources: Vec<Source>,
    pub faces: Vec<Face>,
    pub active: HashSet<FaceId>,
    pub favorites: HashSet<String>,
    pub collections: Vec<Collection>,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             CREATE TABLE IF NOT EXISTS sources (
                id INTEGER PRIMARY KEY, kind TEXT NOT NULL, path TEXT NOT NULL,
                name TEXT NOT NULL, added_at INTEGER NOT NULL);
             CREATE TABLE IF NOT EXISTS faces (
                id INTEGER PRIMARY KEY, source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
                family_id TEXT NOT NULL, family TEXT NOT NULL, style TEXT NOT NULL,
                full_name TEXT NOT NULL, postscript TEXT NOT NULL, path TEXT NOT NULL,
                idx INTEGER NOT NULL, weight INTEGER NOT NULL, italic INTEGER NOT NULL,
                width INTEGER NOT NULL, format TEXT NOT NULL, variable INTEGER NOT NULL,
                size INTEGER NOT NULL, mtime INTEGER NOT NULL, added_at INTEGER NOT NULL);
             CREATE INDEX IF NOT EXISTS faces_source ON faces(source_id);
             CREATE TABLE IF NOT EXISTS active (face_id INTEGER PRIMARY KEY);
             CREATE TABLE IF NOT EXISTS favorites (family_id TEXT PRIMARY KEY);
             CREATE TABLE IF NOT EXISTS collections (
                id INTEGER PRIMARY KEY, name TEXT NOT NULL, color TEXT NOT NULL, created_at INTEGER NOT NULL);
             CREATE TABLE IF NOT EXISTS collection_items (
                collection_id INTEGER NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
                family_id TEXT NOT NULL, PRIMARY KEY (collection_id, family_id));
             CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO sources (id, kind, path, name, added_at) VALUES (?1, 'google', '', 'Google Fonts', ?2)",
            params![GOOGLE_SOURCE_ID, now()],
        )?;
        Ok(Self { conn })
    }

    pub fn load(&self) -> Result<Loaded> {
        let sources = self
            .conn
            .prepare("SELECT id, kind, path, name FROM sources ORDER BY id")?
            .query_map([], |r| {
                Ok(Source {
                    id: r.get(0)?,
                    kind: SourceKind::parse(&r.get::<_, String>(1)?),
                    path: PathBuf::from(r.get::<_, String>(2)?),
                    name: r.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let faces = self
            .conn
            .prepare(
                "SELECT id, source_id, family_id, family, style, full_name, postscript, path, idx,
                        weight, italic, width, format, variable, size, mtime, added_at FROM faces",
            )?
            .query_map([], |r| {
                Ok(Face {
                    id: r.get::<_, i64>(0)? as u64,
                    source_id: r.get(1)?,
                    family_id: r.get(2)?,
                    family: r.get(3)?,
                    style: r.get(4)?,
                    full_name: r.get(5)?,
                    postscript: r.get(6)?,
                    path: PathBuf::from(r.get::<_, String>(7)?),
                    index: r.get(8)?,
                    weight: r.get(9)?,
                    italic: r.get(10)?,
                    width: r.get(11)?,
                    format: FontFormat::parse(&r.get::<_, String>(12)?),
                    variable: r.get(13)?,
                    size: r.get::<_, i64>(14)? as u64,
                    mtime: r.get(15)?,
                    added_at: r.get(16)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let active = self
            .conn
            .prepare("SELECT face_id FROM active")?
            .query_map([], |r| Ok(r.get::<_, i64>(0)? as u64))?
            .collect::<rusqlite::Result<HashSet<_>>>()?;

        let favorites = self
            .conn
            .prepare("SELECT family_id FROM favorites")?
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<HashSet<_>>>()?;

        let mut members: HashMap<i64, HashSet<String>> = HashMap::new();
        for row in self
            .conn
            .prepare("SELECT collection_id, family_id FROM collection_items")?
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        {
            let (c, f) = row?;
            members.entry(c).or_default().insert(f);
        }
        let collections = self
            .conn
            .prepare("SELECT id, name, color FROM collections ORDER BY created_at")?
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?
            .map(|row| {
                row.map(|(id, name, color)| Collection {
                    id,
                    name,
                    color,
                    members: members.remove(&id).unwrap_or_default(),
                })
            })
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(Loaded {
            sources,
            faces,
            active,
            favorites,
            collections,
        })
    }

    pub fn add_source(&self, kind: SourceKind, path: &Path, name: &str) -> Result<Source> {
        self.conn.execute(
            "INSERT INTO sources (kind, path, name, added_at) VALUES (?1, ?2, ?3, ?4)",
            params![kind.as_str(), path.to_string_lossy(), name, now()],
        )?;
        Ok(Source {
            id: self.conn.last_insert_rowid(),
            kind,
            path: path.to_path_buf(),
            name: name.to_owned(),
        })
    }

    pub fn remove_source(&self, id: SourceId) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM active WHERE face_id IN (SELECT id FROM faces WHERE source_id = ?1)",
            [id],
        )?;
        tx.execute("DELETE FROM faces WHERE source_id = ?1", [id])?;
        tx.execute("DELETE FROM sources WHERE id = ?1", [id])?;
        tx.commit()?;
        Ok(())
    }

    pub fn apply_faces(&self, added: &[Face], removed: &[FaceId]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut del = tx.prepare_cached("DELETE FROM faces WHERE id = ?1")?;
            let mut del_active = tx.prepare_cached("DELETE FROM active WHERE face_id = ?1")?;
            for id in removed {
                del.execute([*id as i64])?;
                del_active.execute([*id as i64])?;
            }
            let mut ins = tx.prepare_cached(
                "INSERT OR REPLACE INTO faces (id, source_id, family_id, family, style, full_name, postscript,
                   path, idx, weight, italic, width, format, variable, size, mtime, added_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            )?;
            for f in added {
                ins.execute(params![
                    f.id as i64,
                    f.source_id,
                    f.family_id,
                    f.family,
                    f.style,
                    f.full_name,
                    f.postscript,
                    f.path.to_string_lossy(),
                    f.index,
                    f.weight,
                    f.italic,
                    f.width,
                    f.format.as_str(),
                    f.variable,
                    f.size as i64,
                    f.mtime,
                    f.added_at
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn set_active(&self, ids: &[FaceId], on: bool) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let sql = if on {
                "INSERT OR IGNORE INTO active (face_id) VALUES (?1)"
            } else {
                "DELETE FROM active WHERE face_id = ?1"
            };
            let mut st = tx.prepare_cached(sql)?;
            for id in ids {
                st.execute([*id as i64])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn set_favorite(&self, family_id: &str, on: bool) -> Result<()> {
        if on {
            self.conn.execute(
                "INSERT OR IGNORE INTO favorites (family_id) VALUES (?1)",
                [family_id],
            )?;
        } else {
            self.conn
                .execute("DELETE FROM favorites WHERE family_id = ?1", [family_id])?;
        }
        Ok(())
    }

    pub fn create_collection(&self, name: &str, color: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO collections (name, color, created_at) VALUES (?1, ?2, ?3)",
            params![name, color, now()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_collection(&self, id: i64, name: &str, color: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE collections SET name = ?2, color = ?3 WHERE id = ?1",
            params![id, name, color],
        )?;
        Ok(())
    }

    pub fn delete_collection(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM collections WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn set_collection_items(&self, id: i64, family_ids: &[String], on: bool) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let sql = if on {
                "INSERT OR IGNORE INTO collection_items (collection_id, family_id) VALUES (?1, ?2)"
            } else {
                "DELETE FROM collection_items WHERE collection_id = ?1 AND family_id = ?2"
            };
            let mut st = tx.prepare_cached(sql)?;
            for f in family_ids {
                st.execute(params![id, f])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn kv_get(&self, key: &str) -> Option<String> {
        self.conn
            .query_row("SELECT value FROM kv WHERE key = ?1", [key], |r| r.get(0))
            .ok()
    }

    pub fn kv_set(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO kv (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}
