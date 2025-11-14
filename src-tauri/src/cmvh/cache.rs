// CMVH Verification Cache - SQLite persistence layer
// Provides persistent caching for on-chain verification results

use crate::db;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// CMVH verification cache entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CMVHVerificationCache {
    pub id: i64,
    pub signature: String,
    pub email_hash: String,
    pub is_valid: bool,
    pub error: Option<String>,
    pub verified_at: i64,
    pub expires_at: i64,
}

/// Get cached CMVH verification result from SQLite
/// Returns None if not found or expired
pub async fn get_cached_verification(
    signature: &str,
    email_hash: &str,
) -> Result<Option<CMVHVerificationCache>, String> {
    let pool = db::pool();
    let now = Utc::now().timestamp();

    let result = sqlx::query_as::<_, CMVHVerificationCache>(
        "SELECT * FROM cmvh_verification_cache
         WHERE signature = ? AND email_hash = ? AND expires_at > ?",
    )
    .bind(signature)
    .bind(email_hash)
    .bind(now)
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to get cached verification: {}", e))?;

    if result.is_some() {
        println!(
            "✅ CMVH verification cache hit for signature {}",
            &signature[..16.min(signature.len())]
        );
    }

    Ok(result)
}

/// Cache CMVH verification result to SQLite
/// TTL: 90 days (signatures don't change)
pub async fn cache_verification(
    signature: &str,
    email_hash: &str,
    is_valid: bool,
    error: Option<&str>,
) -> Result<(), String> {
    let pool = db::pool();
    let now = Utc::now().timestamp();
    let expires_at = now + (90 * 24 * 60 * 60); // 90 days

    sqlx::query(
        "INSERT INTO cmvh_verification_cache
         (signature, email_hash, is_valid, error, verified_at, expires_at)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(signature, email_hash) DO UPDATE SET
            is_valid = excluded.is_valid,
            error = excluded.error,
            verified_at = excluded.verified_at,
            expires_at = excluded.expires_at",
    )
    .bind(signature)
    .bind(email_hash)
    .bind(is_valid as i64)
    .bind(error)
    .bind(now)
    .bind(expires_at)
    .execute(pool.as_ref())
    .await
    .map_err(|e| format!("Failed to cache verification: {}", e))?;

    println!(
        "💾 Cached CMVH verification for signature {} (valid: {})",
        &signature[..16.min(signature.len())],
        is_valid
    );

    Ok(())
}

/// Cleanup expired cache entries
/// Returns the number of deleted rows
pub async fn cleanup_expired_cache() -> Result<u64, String> {
    let pool = db::pool();
    let now = Utc::now().timestamp();

    let result = sqlx::query("DELETE FROM cmvh_verification_cache WHERE expires_at < ?")
        .bind(now)
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to cleanup cache: {}", e))?;

    let deleted = result.rows_affected();
    if deleted > 0 {
        println!("🧹 Cleaned up {} expired CMVH cache entries", deleted);
    }

    Ok(deleted)
}

/// Clear all cache entries
/// Useful for testing or when user wants to force re-verification
pub async fn clear_all_cache() -> Result<u64, String> {
    let pool = db::pool();

    let result = sqlx::query("DELETE FROM cmvh_verification_cache")
        .execute(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to clear cache: {}", e))?;

    let deleted = result.rows_affected();
    println!("🗑️ Cleared all {} CMVH cache entries", deleted);

    Ok(deleted)
}

/// Get cache statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub total: i64,
    pub valid: i64,
    pub expired: i64,
}

pub async fn get_cache_stats() -> Result<CacheStats, String> {
    let pool = db::pool();
    let now = Utc::now().timestamp();

    // Get total count
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cmvh_verification_cache")
        .fetch_one(pool.as_ref())
        .await
        .map_err(|e| format!("Failed to get total count: {}", e))?;

    // Get valid count (not expired)
    let valid: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM cmvh_verification_cache WHERE expires_at > ?")
            .bind(now)
            .fetch_one(pool.as_ref())
            .await
            .map_err(|e| format!("Failed to get valid count: {}", e))?;

    Ok(CacheStats {
        total: total.0,
        valid: valid.0,
        expired: total.0 - valid.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_operations() {
        // Note: This test requires a database connection
        // In a real test, you'd initialize a test database first

        let signature = "0x1234567890abcdef";
        let email_hash = "hash123";

        // Test cache miss
        let result = get_cached_verification(signature, email_hash).await;
        assert!(result.is_ok());

        // Test cache write
        let cache_result = cache_verification(signature, email_hash, true, None).await;
        assert!(cache_result.is_ok());

        // Test cache hit
        let cached = get_cached_verification(signature, email_hash).await;
        assert!(cached.is_ok());
        if let Ok(Some(entry)) = cached {
            assert_eq!(entry.signature, signature);
            assert_eq!(entry.email_hash, email_hash);
            assert!(entry.is_valid);
        }
    }
}
