//! libSQL/Turso client wrapper.
//!
//! Interface mirrors `ktunPlatform/crates/indexer-service/src/storage/turso.rs`
//! so this module can be lifted into the platform workspace later with a
//! path-dep swap.

use crate::config::Config;
use crate::error::Result;
use libsql::{Builder, Connection};
use std::sync::Arc;
use std::time::Duration;

use crate::db::migrations;


#[derive(Clone)]
pub struct Db {
    conn: Arc<Connection>,
}

impl Db {
    pub async fn connect(config: &Config) -> Result<Self> {
        migrations::ensure_parent_dir(&config.turso_database_url).await?;

        let db = if migrations::is_local_file(&config.turso_database_url) {
            let path = migrations::local_path(&config.turso_database_url);
            tracing::info!(path = %path, "opening local libSQL database");
            Builder::new_local(path).build().await?
        } else {
            tracing::info!(url = %config.turso_database_url, "connecting to remote Turso");
            let b = Builder::new_remote(
                config.turso_database_url.clone(),
                config.turso_auth_token.clone().unwrap_or_default(),
            );
            b.build().await?
        };

        let conn = Arc::new(db.connect()?);
        if migrations::is_local_file(&config.turso_database_url) {
            let _ = conn.execute("PRAGMA journal_mode = WAL;", ()).await;
            let _ = conn.execute("PRAGMA busy_timeout = 5000;", ()).await;
            let _ = conn.execute("PRAGMA synchronous = NORMAL;", ()).await;
        }

        migrations::run(&conn).await?;
        Ok(Self { conn })
    }

    /// Get a cached blob if it hasn't expired. Returns the raw bytes.
    pub async fn cache_get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let now = chrono::Utc::now().timestamp();
        let now_str = now.to_string();
        let mut rows = self
            .conn
            .query(
                "SELECT value FROM cache_blob WHERE key = ?1 AND expires_at > ?2",
                [key.to_string(), now_str],
            )
            .await?;
        if let Ok(Some(row)) = rows.next().await {
            if let Ok(libsql::Value::Blob(blob)) = row.get_value(0) {
                return Ok(Some(blob));
            }
        }
        Ok(None)
    }

    /// Insert-or-replace a cached blob with a TTL in seconds.
    pub async fn cache_set(&self, key: &str, value: Vec<u8>, ttl: Duration) -> Result<()> {
        let expires_at = chrono::Utc::now().timestamp() + ttl.as_secs() as i64;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO cache_blob (key, value, expires_at) VALUES (?1, ?2, ?3)",
                libsql::params![key, value, expires_at],
            )
            .await?;
        Ok(())
    }

    /// Delete expired cache entries. Cheap; called opportunistically.
    pub async fn cache_evict(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let now_str = now.to_string();
        self.conn
            .execute(
                "DELETE FROM cache_blob WHERE expires_at <= ?1",
                [now_str],
            )
            .await?;
        Ok(())
    }

    /// Retrieve student profile (department, grade, term, credits).
    pub async fn get_student_profile(&self, chat_id: i64) -> Result<Option<StudentProfile>> {
        let mut rows = self
            .conn
            .query(
                "SELECT department, grade, term, magnum_credits FROM student_profiles WHERE chat_id = ?1",
                [chat_id],
            )
            .await?;
        if let Ok(Some(row)) = rows.next().await {
            let dept: String = row.get(0)?;
            let grade: i64 = row.get(1)?;
            let term: String = row.get(2)?;
            let credits: i64 = row.get(3)?;
            return Ok(Some(StudentProfile {
                chat_id,
                department: dept,
                grade: grade as u8,
                term,
                magnum_credits: credits,
            }));
        }
        Ok(None)
    }

    /// Save or update student profile.
    pub async fn upsert_student_profile(
        &self,
        chat_id: i64,
        department: &str,
        grade: u8,
        term: &str,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO student_profiles (chat_id, department, grade, term, updated_at) 
                 VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)
                 ON CONFLICT(chat_id) DO UPDATE SET 
                     department = excluded.department,
                     grade = excluded.grade,
                     term = excluded.term,
                     updated_at = CURRENT_TIMESTAMP",
                libsql::params![chat_id, department, grade as i64, term],
            )
            .await?;
        Ok(())
    }

    /// Retrieve cached telegram file_id for instant zero-bandwidth sends.
    pub async fn get_cached_telegram_file_id(&self, key: &str) -> Result<Option<String>> {
        let mut rows = self
            .conn
            .query(
                "SELECT file_id FROM telegram_file_cache WHERE cache_key = ?1",
                [key],
            )
            .await?;
        if let Ok(Some(row)) = rows.next().await {
            let file_id: String = row.get(0)?;
            return Ok(Some(file_id));
        }
        Ok(None)
    }

    /// Store a telegram file_id in cache.
    pub async fn set_cached_telegram_file_id(&self, key: &str, file_id: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO telegram_file_cache (cache_key, file_id) VALUES (?1, ?2)",
                libsql::params![key, file_id],
            )
            .await?;
        Ok(())
    }

    /// Enqueue a newly uploaded student study material for AI processing.
    pub async fn enqueue_intake(&self, item: &IntakeQueueItem) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO intake_queue (id, chat_id, file_name, file_size, storage_path, department, course_hint, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)",
                libsql::params![
                    item.id.clone(),
                    item.chat_id,
                    item.file_name.clone(),
                    item.file_size,
                    item.storage_path.clone(),
                    item.department.clone(),
                    item.course_hint.clone(),
                    item.status.clone()
                ],
            )
            .await?;
        Ok(())
    }

    /// Retrieve pending queue items for processing.
    pub async fn get_pending_intake_items(&self, limit: usize) -> Result<Vec<IntakeQueueItem>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, chat_id, file_name, file_size, storage_path, department, course_hint, status, quality_score, rejection_reason, created_at, processed_at
                 FROM intake_queue WHERE status = 'QUEUED' ORDER BY created_at ASC LIMIT ?1",
                libsql::params![limit as i64],
            )
            .await?;
        let mut list = Vec::new();
        while let Ok(Some(row)) = rows.next().await {
            list.push(IntakeQueueItem {
                id: row.get(0)?,
                chat_id: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                storage_path: row.get(4)?,
                department: row.get(5)?,
                course_hint: row.get(6)?,
                status: row.get(7)?,
                quality_score: row.get(8)?,
                rejection_reason: row.get(9)?,
                created_at: row.get(10)?,
                processed_at: row.get(11)?,
            });
        }
        Ok(list)
    }

    /// Update status of an intake queue item after evaluation.
    pub async fn update_intake_status(
        &self,
        id: &str,
        status: &str,
        score: Option<i64>,
        reason: Option<&str>,
    ) -> Result<()> {
        self.conn
            .execute(
                "UPDATE intake_queue 
                 SET status = ?1, quality_score = ?2, rejection_reason = ?3, processed_at = CURRENT_TIMESTAMP
                 WHERE id = ?4",
                libsql::params![status, score, reason, id],
            )
            .await?;
        Ok(())
    }

    /// Add magnum credits to a student upon accepted contribution.
    pub async fn add_student_credits(&self, chat_id: i64, credits: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE student_profiles SET magnum_credits = magnum_credits + ?1, updated_at = CURRENT_TIMESTAMP WHERE chat_id = ?2",
                libsql::params![credits, chat_id],
            )
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StudentProfile {
    pub chat_id: i64,
    pub department: String,
    pub grade: u8,
    pub term: String,
    pub magnum_credits: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntakeQueueItem {
    pub id: String,
    pub chat_id: i64,
    pub file_name: String,
    pub file_size: i64,
    pub storage_path: String,
    pub department: Option<String>,
    pub course_hint: Option<String>,
    pub status: String,
    pub quality_score: Option<i64>,
    pub rejection_reason: Option<String>,
    pub created_at: Option<String>,
    pub processed_at: Option<String>,
}
