/**
 * CMVH Verification Cache
 * Dual-layer caching: Memory (L1) + SQLite (L2) for optimal performance
 *
 * Architecture:
 * - L1 (Memory): Instant access, session-scoped
 * - L2 (SQLite): Persistent across restarts, 90-day TTL
 *
 * Flow:
 * 1. Check L1 memory cache (fastest)
 * 2. If miss, check L2 SQLite cache (fast, persistent)
 * 3. If miss, perform RPC call and cache in both layers
 */

import { invoke } from "@tauri-apps/api/core";
import type { CMVHHeaders } from "./types";

export interface EmailContent {
  subject: string;
  from: string;
  to: string;
  body: string;
}

export interface OnChainVerificationResult {
  isValid: boolean;
  error?: string;
  timestamp?: number;
}

// Rust CMVHVerificationCache type (from backend)
interface RustCMVHCache {
  id: number;
  signature: string;
  email_hash: string;
  is_valid: boolean;
  error: string | null;
  verified_at: number;
  expires_at: number;
}

interface CMVHVerificationCache {
  emailHash: string; // Email content hash (unique identifier)
  result: OnChainVerificationResult;
  timestamp: number;
  expiresAt: number;
}

// L1 Memory cache TTL: 1 hour (lightweight, session-scoped)
const MEMORY_CACHE_TTL = 60 * 60 * 1000;

// L1 (Memory) cache store - fastest access
const memoryCache = new Map<string, CMVHVerificationCache>();

/**
 * Simple hash function for string data
 */
function hashString(str: string): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash; // Convert to 32-bit integer
  }
  return Math.abs(hash).toString(36);
}

/**
 * Generate cache key from CMVH headers and email content
 * Uses signature as primary key since it uniquely identifies the signed email
 */
function getCacheKey(headers: CMVHHeaders, content: EmailContent): string {
  // Signature is unique per email, so it's the perfect cache key
  // Add content hash to ensure we're verifying the same email
  const contentData = `${content.subject}:${content.from}:${content.to}`;
  const contentHash = hashString(contentData);
  return `${headers.signature}:${contentHash}`;
}

/**
 * Get cached verification result (dual-layer)
 * Returns null if not found in either cache
 */
export async function getCachedVerification(
  headers: CMVHHeaders,
  content: EmailContent
): Promise<OnChainVerificationResult | null> {
  const key = getCacheKey(headers, content);
  const emailHash = hashEmailContent(content);

  // 1. Check L1 (Memory) cache first - fastest
  const memCached = memoryCache.get(key);
  if (memCached && Date.now() <= memCached.expiresAt) {
    console.log(
      `⚡ L1 cache hit (${Math.round((Date.now() - memCached.timestamp) / 1000)}s ago)`
    );
    return memCached.result;
  }

  // 2. L1 miss - check L2 (SQLite) cache
  try {
    const sqliteCached = await invoke<RustCMVHCache | null>("get_cmvh_cache", {
      signature: headers.signature,
      emailHash,
    });

    if (sqliteCached) {
      console.log(
        `💾 L2 cache hit (${Math.round((Date.now() - sqliteCached.verified_at * 1000) / 1000)}s ago)`
      );

      // Promote to L1 cache for faster subsequent access
      const result: OnChainVerificationResult = {
        isValid: sqliteCached.is_valid,
        error: sqliteCached.error || undefined,
        timestamp: sqliteCached.verified_at * 1000,
      };

      memoryCache.set(key, {
        emailHash,
        result,
        timestamp: Date.now(),
        expiresAt: Date.now() + MEMORY_CACHE_TTL,
      });

      return result;
    }
  } catch (error) {
    console.error("❌ Failed to check L2 cache:", error);
    // Continue - will return null and trigger RPC call
  }

  // 3. Both caches missed
  return null;
}

/**
 * Hash email content for cache key
 */
function hashEmailContent(content: EmailContent): string {
  const data = `${content.subject}:${content.from}:${content.to}`;
  return hashString(data);
}

/**
 * Cache verification result (dual-layer)
 * Saves to both memory and SQLite for optimal performance
 */
export async function cacheVerification(
  headers: CMVHHeaders,
  content: EmailContent,
  result: OnChainVerificationResult
): Promise<void> {
  const key = getCacheKey(headers, content);
  const emailHash = hashEmailContent(content);

  // 1. Save to L1 (Memory) cache - immediate access
  memoryCache.set(key, {
    emailHash,
    result,
    timestamp: Date.now(),
    expiresAt: Date.now() + MEMORY_CACHE_TTL,
  });

  // 2. Save to L2 (SQLite) cache - persistent across restarts
  try {
    await invoke("save_cmvh_cache", {
      signature: headers.signature,
      emailHash,
      isValid: result.isValid,
      error: result.error || null,
    });

    console.log(
      `💾 Cached CMVH verification (L1 + L2) for signature ${headers.signature.substring(0, 16)}...`
    );
    console.log(`   L1 size: ${memoryCache.size} entries`);
  } catch (error) {
    console.error("❌ Failed to save to L2 cache:", error);
    // L1 cache still works, so this is not fatal
  }
}

/**
 * Clear expired cache entries (both layers)
 * Called periodically to prevent memory bloat
 */
export async function cleanupCache(): Promise<number> {
  const now = Date.now();
  let removed = 0;

  // Cleanup L1 (Memory) cache
  for (const [key, entry] of memoryCache.entries()) {
    if (now > entry.expiresAt) {
      memoryCache.delete(key);
      removed++;
    }
  }

  // Cleanup L2 (SQLite) cache
  try {
    const sqliteRemoved = await invoke<number>("cleanup_cmvh_cache");
    removed += sqliteRemoved;
  } catch (error) {
    console.error("❌ Failed to cleanup L2 cache:", error);
  }

  if (removed > 0) {
    console.log(`🧹 Cleaned up ${removed} expired CMVH cache entries`);
  }

  return removed;
}

/**
 * Clear all cache entries (both layers)
 * Useful for testing or when user changes settings
 */
export async function clearAllCache(): Promise<void> {
  const memSize = memoryCache.size;
  memoryCache.clear();

  try {
    const sqliteRemoved = await invoke<number>("clear_cmvh_cache");
    console.log(
      `🗑️ Cleared all CMVH cache entries (L1: ${memSize}, L2: ${sqliteRemoved})`
    );
  } catch (error) {
    console.error("❌ Failed to clear L2 cache:", error);
    console.log(`🗑️ Cleared L1 cache (${memSize} entries)`);
  }
}

/**
 * Get cache statistics (both layers)
 */
export async function getCacheStats() {
  const now = Date.now();
  let memValid = 0;
  let memExpired = 0;

  // L1 (Memory) stats
  for (const entry of memoryCache.values()) {
    if (now > entry.expiresAt) {
      memExpired++;
    } else {
      memValid++;
    }
  }

  // L2 (SQLite) stats
  let sqliteStats = { total: 0, valid: 0, expired: 0 };
  try {
    sqliteStats = await invoke<{ total: number; valid: number; expired: number }>(
      "get_cmvh_cache_stats"
    );
  } catch (error) {
    console.error("❌ Failed to get L2 cache stats:", error);
  }

  return {
    l1: {
      total: memoryCache.size,
      valid: memValid,
      expired: memExpired,
    },
    l2: sqliteStats,
    combined: {
      total: memoryCache.size + sqliteStats.total,
      valid: memValid + sqliteStats.valid,
      expired: memExpired + sqliteStats.expired,
    },
  };
}

// Cleanup expired cache every hour
const CLEANUP_INTERVAL = 60 * 60 * 1000;
if (typeof window !== "undefined") {
  setInterval(cleanupCache, CLEANUP_INTERVAL);
  console.log("🔄 CMVH cache cleanup scheduled (every hour)");
}
