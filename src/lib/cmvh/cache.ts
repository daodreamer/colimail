/**
 * CMVH Verification Cache
 * Caches on-chain verification results to avoid redundant RPC calls
 */

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

interface CMVHVerificationCache {
  emailHash: string; // Email content hash (unique identifier)
  result: OnChainVerificationResult;
  timestamp: number;
  expiresAt: number;
}

// Cache TTL: 24 hours (signatures don't change once created)
const CACHE_TTL = 24 * 60 * 60 * 1000;

// In-memory cache store
const cacheStore = new Map<string, CMVHVerificationCache>();

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
 * Get cached verification result
 * Returns null if not cached or expired
 */
export function getCachedVerification(
  headers: CMVHHeaders,
  content: EmailContent
): OnChainVerificationResult | null {
  const key = getCacheKey(headers, content);
  const cached = cacheStore.get(key);

  if (!cached) {
    return null;
  }

  // Check if expired
  if (Date.now() > cached.expiresAt) {
    cacheStore.delete(key);
    console.log(`🗑️ Expired CMVH verification cache for ${key.substring(0, 16)}...`);
    return null;
  }

  console.log(`✅ Using cached CMVH verification result (cached ${Math.round((Date.now() - cached.timestamp) / 1000)}s ago)`);
  return cached.result;
}

/**
 * Cache verification result
 */
export function cacheVerification(
  headers: CMVHHeaders,
  content: EmailContent,
  result: OnChainVerificationResult
): void {
  const key = getCacheKey(headers, content);
  const contentData = `${content.subject}:${content.from}:${content.to}`;
  const emailHash = hashString(contentData);

  cacheStore.set(key, {
    emailHash,
    result,
    timestamp: Date.now(),
    expiresAt: Date.now() + CACHE_TTL,
  });

  console.log(`💾 Cached CMVH verification for signature ${headers.signature.substring(0, 16)}...`);
  console.log(`   Cache size: ${cacheStore.size} entries`);
}

/**
 * Clear expired cache entries
 * Called periodically to prevent memory bloat
 */
export function cleanupCache(): number {
  const now = Date.now();
  let removed = 0;

  for (const [key, entry] of cacheStore.entries()) {
    if (now > entry.expiresAt) {
      cacheStore.delete(key);
      removed++;
    }
  }

  if (removed > 0) {
    console.log(`🧹 Cleaned up ${removed} expired CMVH cache entries`);
  }

  return removed;
}

/**
 * Clear all cache entries
 * Useful for testing or when user changes settings
 */
export function clearAllCache(): void {
  const size = cacheStore.size;
  cacheStore.clear();
  console.log(`🗑️ Cleared all ${size} CMVH cache entries`);
}

/**
 * Get cache statistics
 */
export function getCacheStats() {
  const now = Date.now();
  let validEntries = 0;
  let expiredEntries = 0;

  for (const entry of cacheStore.values()) {
    if (now > entry.expiresAt) {
      expiredEntries++;
    } else {
      validEntries++;
    }
  }

  return {
    total: cacheStore.size,
    valid: validEntries,
    expired: expiredEntries,
  };
}

// Cleanup expired cache every hour
const CLEANUP_INTERVAL = 60 * 60 * 1000;
if (typeof window !== "undefined") {
  setInterval(cleanupCache, CLEANUP_INTERVAL);
  console.log("🔄 CMVH cache cleanup scheduled (every hour)");
}
