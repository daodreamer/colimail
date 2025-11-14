//! ENS-related Tauri commands
//!
//! These commands provide ENS name caching functionality to the frontend.
//! The actual ENS resolution is done on the frontend using viem.

use crate::ens::{self, ENSCache, ENSCacheStats};
use tauri::command;

/// Get cached ENS name for an address
#[command]
pub async fn get_ens_cache(address: String) -> Result<Option<ENSCache>, String> {
    ens::get_cached_ens(&address).await
}

/// Save ENS name to cache
///
/// # Arguments
/// * `address` - Ethereum address
/// * `ens_name` - ENS name (null if address has no ENS)
/// * `ttl_days` - Time to live in days (optional, defaults to 7)
#[command]
pub async fn save_ens_cache(
    address: String,
    ens_name: Option<String>,
    ttl_days: Option<i64>,
) -> Result<(), String> {
    let ttl = ttl_days.unwrap_or(7);
    ens::save_ens_cache(&address, ens_name.as_deref(), ttl).await
}

/// Clean up expired ENS cache entries
///
/// Returns the number of deleted entries
#[command]
pub async fn cleanup_ens_cache() -> Result<u64, String> {
    ens::cleanup_expired_ens_cache().await
}

/// Clear all ENS cache entries
///
/// Returns the number of deleted entries
#[command]
pub async fn clear_ens_cache() -> Result<u64, String> {
    ens::clear_all_ens_cache().await
}

/// Get ENS cache statistics
#[command]
pub async fn get_ens_cache_stats() -> Result<ENSCacheStats, String> {
    ens::get_ens_cache_stats().await
}
