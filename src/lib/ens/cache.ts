// ENS Cache - Dual-layer caching for ENS name resolution
//
// Architecture:
// L1: Memory cache (session-scoped, 1-hour TTL, <1ms latency)
// L2: SQLite cache (persistent, 7-day TTL, ~2-5ms latency)
//
// This reduces RPC calls by >90% and improves user experience

import { invoke } from "@tauri-apps/api/core";

// L1 Memory cache entry
interface MemoryCacheEntry {
	ensName: string | null;
	expiresAt: number;
}

// L2 SQLite cache entry (from Rust)
interface SQLiteCacheEntry {
	address: string;
	ens_name: string | null;
	resolved_at: number;
	expires_at: number;
}

// L1: In-memory cache (session-scoped)
const memoryCache = new Map<string, MemoryCacheEntry>();

// Cache configuration
const MEMORY_TTL_MS = 60 * 60 * 1000; // 1 hour
const SQLITE_TTL_DAYS = 7; // 7 days

/**
 * Get cached ENS name for an address (checks both L1 and L2)
 *
 * @param address - Ethereum address (case-insensitive)
 * @returns ENS name if cached and valid, null if no ENS name, undefined if not cached
 */
export async function getCachedENS(
	address: string,
): Promise<string | null | undefined> {
	const addressLower = address.toLowerCase();

	// 1. Check L1 (Memory) cache - <1ms
	const memoryCached = memoryCache.get(addressLower);
	if (memoryCached && Date.now() < memoryCached.expiresAt) {
		console.log(`🎯 L1 hit for ${address}: ${memoryCached.ensName || "no ENS"}`);
		return memoryCached.ensName;
	}

	// 2. Check L2 (SQLite) cache - ~2-5ms
	try {
		const sqliteCached = await invoke<SQLiteCacheEntry | null>("get_ens_cache", {
			address: addressLower,
		});

		if (sqliteCached) {
			console.log(
				`💾 L2 hit for ${address}: ${sqliteCached.ens_name || "no ENS"}`,
			);

			// Promote to L1 for faster subsequent access
			memoryCache.set(addressLower, {
				ensName: sqliteCached.ens_name,
				expiresAt: Date.now() + MEMORY_TTL_MS,
			});

			return sqliteCached.ens_name;
		}
	} catch (error) {
		console.error("Failed to query L2 cache:", error);
		// Continue to undefined (cache miss)
	}

	// Cache miss
	console.log(`❌ Cache miss for ${address}`);
	return undefined;
}

/**
 * Save ENS name to cache (both L1 and L2)
 *
 * @param address - Ethereum address
 * @param ensName - ENS name (null if address has no ENS)
 */
export async function saveENSCache(
	address: string,
	ensName: string | null,
): Promise<void> {
	const addressLower = address.toLowerCase();

	// Save to L1 (immediate)
	memoryCache.set(addressLower, {
		ensName,
		expiresAt: Date.now() + MEMORY_TTL_MS,
	});

	// Save to L2 (persistent)
	try {
		await invoke("save_ens_cache", {
			address: addressLower,
			ensName,
			ttlDays: SQLITE_TTL_DAYS,
		});
		console.log(`💾 Saved to cache: ${address} -> ${ensName || "no ENS"}`);
	} catch (error) {
		console.error("Failed to save to L2 cache:", error);
	}
}

/**
 * Clear all ENS cache entries (both L1 and L2)
 *
 * @returns Number of L2 entries deleted
 */
export async function clearENSCache(): Promise<number> {
	// Clear L1
	memoryCache.clear();
	console.log("🗑️ Cleared L1 cache");

	// Clear L2
	try {
		const deleted = await invoke<number>("clear_ens_cache");
		console.log(`🗑️ Cleared L2 cache: ${deleted} entries deleted`);
		return deleted;
	} catch (error) {
		console.error("Failed to clear L2 cache:", error);
		return 0;
	}
}

/**
 * Clean up expired ENS cache entries (L2 only, L1 auto-expires)
 *
 * @returns Number of L2 entries deleted
 */
export async function cleanupENSCache(): Promise<number> {
	try {
		const deleted = await invoke<number>("cleanup_ens_cache");
		console.log(`🧹 Cleaned up ${deleted} expired L2 entries`);
		return deleted;
	} catch (error) {
		console.error("Failed to cleanup L2 cache:", error);
		return 0;
	}
}

/**
 * Get ENS cache statistics
 */
export interface ENSCacheStats {
	total_entries: number;
	entries_with_name: number;
	entries_without_name: number;
	expired_entries: number;
	memory_entries: number; // L1 only
}

export async function getENSCacheStats(): Promise<ENSCacheStats> {
	try {
		const sqliteStats = await invoke<Omit<ENSCacheStats, "memory_entries">>(
			"get_ens_cache_stats",
		);

		return {
			...sqliteStats,
			memory_entries: memoryCache.size,
		};
	} catch (error) {
		console.error("Failed to get cache stats:", error);
		return {
			total_entries: 0,
			entries_with_name: 0,
			entries_without_name: 0,
			expired_entries: 0,
			memory_entries: memoryCache.size,
		};
	}
}
