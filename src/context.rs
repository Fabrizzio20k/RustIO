use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};
use rusqlite::{ffi::sqlite3_auto_extension, params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};

const DIMS: usize = 384;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Memory {
    pub task_id: String,
    pub role: String,
    pub content: String,
    pub distance: f32,
}

pub struct ContextStore {
    db: Connection,
    embedder: TextEmbedding,
}

impl ContextStore {
    pub fn new(db_path: &str) -> Result<Self> {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        }

        let db = Connection::open(db_path)
            .with_context(|| format!("no se pudo abrir la base de datos en {db_path}"))?;

        db.execute_batch(&format!(
            "CREATE TABLE IF NOT EXISTS memories (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id    TEXT    NOT NULL,
                role       TEXT    NOT NULL,
                content    TEXT    NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS memory_vec USING vec0(
                embedding float[{DIMS}]
            );"
        ))
        .context("no se pudieron crear las tablas de contexto")?;

        let embedder = TextEmbedding::try_new(
            TextInitOptions::new(EmbeddingModel::MultilingualE5Small)
                .with_show_download_progress(true),
        )
        .context("no se pudo inicializar el modelo de embeddings (fastembed)")?;

        Ok(Self { db, embedder })
    }

    fn embed(&mut self, prefix: &str, text: &str) -> Result<Vec<f32>> {
        let input = format!("{prefix}{text}");
        let mut out = self
            .embedder
            .embed(vec![input], None)
            .context("fallo al generar el embedding")?;
        out.pop().context("fastembed devolvió un resultado vacío")
    }

    pub fn add(&mut self, task_id: &str, role: &str, content: &str) -> Result<i64> {
        let vec = self.embed("passage: ", content)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as i64;

        self.db.execute(
            "INSERT INTO memories (task_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![task_id, role, content, now],
        )?;
        let id = self.db.last_insert_rowid();

        self.db.execute(
            "INSERT INTO memory_vec (rowid, embedding) VALUES (?1, ?2)",
            params![id, f32_to_blob(&vec)],
        )?;

        Ok(id)
    }

    pub fn search(&mut self, query: &str, k: usize) -> Result<Vec<Memory>> {
        let vec = self.embed("query: ", query)?;
        let blob = f32_to_blob(&vec);

        let hits: Vec<(i64, f32)> = {
            let mut stmt = self.db.prepare(
                "SELECT rowid, distance FROM memory_vec
                 WHERE embedding MATCH ?1 AND k = ?2
                 ORDER BY distance",
            )?;
            let rows = stmt.query_map(params![blob, k as i64], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)? as f32))
            })?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };

        let mut out = Vec::with_capacity(hits.len());
        for (id, distance) in hits {
            let mem = self.db.query_row(
                "SELECT task_id, role, content FROM memories WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Memory {
                        task_id: row.get(0)?,
                        role: row.get(1)?,
                        content: row.get(2)?,
                        distance,
                    })
                },
            )?;
            out.push(mem);
        }
        Ok(out)
    }
}

fn f32_to_blob(v: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(v.len() * 4);
    for x in v {
        bytes.extend_from_slice(&x.to_le_bytes());
    }
    bytes
}
