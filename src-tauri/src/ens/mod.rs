//! ENS (Ethereum Name Service) module
//!
//! This module provides ENS name caching for reverse resolution.
//! The actual ENS resolution is done on the frontend using viem.

pub mod cache;

// Re-export public functions
pub use cache::{
    cleanup_expired_ens_cache, clear_all_ens_cache, get_cached_ens, get_ens_cache_stats,
    save_ens_cache, ENSCache, ENSCacheStats,
};
