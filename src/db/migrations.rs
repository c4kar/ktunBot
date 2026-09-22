//! Tiny migration runner for the `libsql` crate (no sqlx-style migrator).
//!
//! Reads `migrations/*.sql` (sorted by filename) on boot and applies each via
//! `execute_batch`. Tracks applied files in `_migrations` so reruns are no-ops.

use crate::error::Result;
use std::path::Path;

const MIGRATIONS_TABLE: &str =
    "CREATE TABLE IF NOT EXISTS _migrations (name TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)";

const EMBEDDED_MIGRATIONS: &[(&str, &str)] = &[
    ("0001_init.sql", include_str!("../../migrations/0001_init.sql")),
    (
        "0002_ecosystem_expansion.sql",
        include_str!("../../migrations/0002_ecosystem_expansion.sql"),
    ),
];

pub async fn run(conn: &libsql::Connection) -> Result<()> {
    conn.execute_batch(MIGRATIONS_TABLE).await?;

    for &(name, sql) in EMBEDDED_MIGRATIONS {
        let already: Vec<String> = match check_applied(conn, name).await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, name = %name, "migration check failed; applying");
                Vec::new()
            }
        };
        if already.is_empty() {
            conn.execute_batch(sql).await?;
            let now_str = chrono::Utc::now().timestamp().to_string();
            conn.execute(
                "INSERT INTO _migrations (name, applied_at) VALUES (?1, ?2)",
                [name.to_string(), now_str],
            )
            .await?;
            tracing::info!(name = %name, "migration applied successfully");
        }
    }
    Ok(())
}

async fn check_applied(conn: &libsql::Connection, name: &str) -> Result<Vec<String>> {
    let mut rows = conn
        .query("SELECT name FROM _migrations WHERE name = ?1", [name])
        .await?;
    let mut out = Vec::new();
    while let Ok(Some(row)) = rows.next().await {
        if let Ok(v) = row.get_str(0) {
            out.push(v.to_string());
        }
    }
    Ok(out)
}

/// Whether the configured database URL is a local file path.
pub fn is_local_file(url: &str) -> bool {
    url.starts_with("file:")
}

pub fn local_path(url: &str) -> &str {
    url.strip_prefix("file:").unwrap_or(url)
}

pub async fn ensure_parent_dir(url: &str) -> Result<()> {
    if is_local_file(url) {
        let path = Path::new(local_path(url));
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
    }
    Ok(())
}
