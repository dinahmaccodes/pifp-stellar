//! Database layer — migrations, queries, and cursor management.

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tracing::info;

use crate::errors::Result;
use crate::events::{EventRecord, PifpEvent};

/// Establish a SQLite connection pool and run pending migrations.
pub async fn init_pool(database_url: &str) -> Result<SqlitePool> {
    // Make sure the file is created if it doesn't exist yet.
    let url = if database_url.starts_with("sqlite:") {
        database_url.to_string()
    } else {
        format!("sqlite:{database_url}")
    };

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Database migrations applied successfully");
    Ok(pool)
}

// ─────────────────────────────────────────────────────────
// Cursor helpers
// ─────────────────────────────────────────────────────────

/// Read the last-seen ledger from the cursor row.
/// Returns `0` when no cursor has been persisted yet.
pub async fn get_last_ledger(pool: &SqlitePool) -> Result<i64> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT last_ledger FROM indexer_cursor WHERE id = 1")
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(v,)| v).unwrap_or(0))
}

/// Persist the last-seen ledger (and optionally a pagination cursor string).
pub async fn save_cursor(
    pool: &SqlitePool,
    last_ledger: i64,
    last_cursor: Option<&str>,
) -> Result<()> {
    sqlx::query("UPDATE indexer_cursor SET last_ledger = ?1, last_cursor = ?2 WHERE id = 1")
        .bind(last_ledger)
        .bind(last_cursor)
        .execute(pool)
        .await?;
    Ok(())
}

/// Read back the raw cursor string (used to resume pagination mid-ledger).
pub async fn get_cursor_string(pool: &SqlitePool) -> Result<Option<String>> {
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT last_cursor FROM indexer_cursor WHERE id = 1")
            .fetch_optional(pool)
            .await?;
    Ok(row.and_then(|(v,)| v))
}

// ─────────────────────────────────────────────────────────
// Event writes
// ─────────────────────────────────────────────────────────

/// Persist a batch of decoded events.  Events that share the same
/// `(ledger, tx_hash, event_type, project_id)` tuple are silently ignored
/// to make the indexer idempotent.
pub async fn insert_events(pool: &SqlitePool, events: &[PifpEvent]) -> Result<usize> {
    let mut count = 0usize;
    for ev in events {
        let rows_affected = sqlx::query(
            r#"
            INSERT OR IGNORE INTO events
                (event_type, project_id, actor, amount, ledger, timestamp, contract_id, tx_hash)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(&ev.event_type)
        .bind(&ev.project_id)
        .bind(&ev.actor)
        .bind(&ev.amount)
        .bind(ev.ledger)
        .bind(ev.timestamp)
        .bind(&ev.contract_id)
        .bind(&ev.tx_hash)
        .execute(pool)
        .await?
        .rows_affected();

        count += rows_affected as usize;
    }
    Ok(count)
}

// ─────────────────────────────────────────────────────────
// Event reads
// ─────────────────────────────────────────────────────────

/// Fetch all events for a given project, ordered by ledger ascending.
pub async fn get_events_for_project(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Vec<EventRecord>> {
    let rows = sqlx::query_as::<_, EventRecord>(
        r#"
        SELECT id, event_type, project_id, actor, amount, ledger, timestamp,
               contract_id, tx_hash, created_at
        FROM   events
        WHERE  project_id = ?1
        ORDER  BY ledger ASC, id ASC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Fetch all events, ordered by ledger ascending.
pub async fn get_all_events(pool: &SqlitePool) -> Result<Vec<EventRecord>> {
    let rows = sqlx::query_as::<_, EventRecord>(
        r#"
        SELECT id, event_type, project_id, actor, amount, ledger, timestamp,
               contract_id, tx_hash, created_at
        FROM   events
        ORDER  BY ledger ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
