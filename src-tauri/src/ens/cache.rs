//! ENS name cache module
//!
//! This module provides SQLite-backed caching for ENS reverse resolution results.
//! Caching reduces RPC calls and improves performance when displaying ENS names.

use crate::db;
use serde::{Deserialize, Serialize};

/// ENS cache entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ENSCache {
    pub address: String,
    pub ens_name: Option<String>,
    pub resolved_at: i64,
    pub expires_at: i64,
}

/// Get cached ENS name for an address
///
/// Returns None if:
/// - No cache entry exists
/// - Cache entry has expired
pub async fn get_cached_ens(address: &str) -> Result<Option<ENSCache>, String> {
    let pool = db::pool();
    let now = chrono::Utc::now().timestamp();

    // Normalize address to lowercase for consistent lookups
    let address_lower = address.to_lowercase();

    let result = sqlx::query_as::<_, ENSCache>(
        "SELECT * FROM ens_cache
         WHERE address = ? AND expires_at > ?",
    )
    .bind(&address_lower)
    .bind(now)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to query ENS cache: {}", e))?;

    Ok(result)
}

/// Save ENS name to cache
///
/// # Arguments
/// * `address` - Ethereum address (will be normalized to lowercase)
/// * `ens_name` - ENS name (None if address has no ENS)
/// * `ttl_days` - Time to live in days (default: 7)
pub async fn save_ens_cache(
    address: &str,
    ens_name: Option<&str>,
    ttl_days: i64,
) -> Result<(), String> {
    let pool = db::pool();
    let now = chrono::Utc::now().timestamp();
    let expires_at = now + (ttl_days * 24 * 60 * 60);

    // Normalize address to lowercase
    let address_lower = address.to_lowercase();

    sqlx::query(
        "INSERT INTO ens_cache (address, ens_name, resolved_at, expires_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(address) DO UPDATE SET
           ens_name = excluded.ens_name,
           resolved_at = excluded.resolved_at,
           expires_at = excluded.expires_at",
    )
    .bind(&address_lower)
    .bind(ens_name)
    .bind(now)
    .bind(expires_at)
    .execute(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to save ENS cache: {}", e))?;

    Ok(())
}

/// Clean up expired ENS cache entries
///
/// Returns the number of deleted entries
pub async fn cleanup_expired_ens_cache() -> Result<u64, String> {
    let pool = db::pool();
    let now = chrono::Utc::now().timestamp();

    let result = sqlx::query("DELETE FROM ens_cache WHERE expires_at <= ?")
        .bind(now)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to cleanup ENS cache: {}", e))?;

    Ok(result.rows_affected())
}

/// Clear all ENS cache entries
///
/// Returns the number of deleted entries
pub async fn clear_all_ens_cache() -> Result<u64, String> {
    let pool = db::pool();

    let result = sqlx::query("DELETE FROM ens_cache")
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to clear ENS cache: {}", e))?;

    Ok(result.rows_affected())
}

/// Get ENS cache statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct ENSCacheStats {
    pub total_entries: i64,
    pub entries_with_name: i64,
    pub entries_without_name: i64,
    pub expired_entries: i64,
}

pub async fn get_ens_cache_stats() -> Result<ENSCacheStats, String> {
    let pool = db::pool();
    let now = chrono::Utc::now().timestamp();

    let total_entries = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM ens_cache")
        .fetch_one(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to count total entries: {}", e))?
        .0;

    let entries_with_name =
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM ens_cache WHERE ens_name IS NOT NULL")
            .fetch_one(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to count entries with name: {}", e))?
            .0;

    let entries_without_name =
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM ens_cache WHERE ens_name IS NULL")
            .fetch_one(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to count entries without name: {}", e))?
            .0;

    let expired_entries =
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM ens_cache WHERE expires_at <= ?")
            .bind(now)
            .fetch_one(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to count expired entries: {}", e))?
            .0;

    Ok(ENSCacheStats {
        total_entries,
        entries_with_name,
        entries_without_name,
        expired_entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ens_cache() {
        // This test requires database initialization
        // Run with: cargo test --package colimail --lib ens::cache::tests::test_ens_cache
    }
}
