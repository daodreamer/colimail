/**
 * ENS Resolver Service - Three-tier caching for ENS name resolution
 *
 * Architecture:
 * - Layer 1: Memory cache (Map) - <1ms, 5 min TTL
 * - Layer 2: SQLite cache - ~5-10ms, 7 days TTL
 * - Layer 3: RPC fallback - ~200-500ms
 */

import { createPublicClient, http, type Address } from 'viem';
import { mainnet } from 'viem/chains';
import { invoke } from '@tauri-apps/api/core';

/**
 * ENS information structure
 */
export interface ENSInfo {
  name: string;           // "vitalik.eth"
  avatar?: string | null; // IPFS/HTTP URL
  resolvedAt: number;     // Unix timestamp (milliseconds)
  ttl: number;           // Cache TTL in seconds
}

/**
 * SQLite ENS cache entry
 */
interface ENSCacheEntry {
  address: string;
  ens_name: string | null;
  resolved_at: number;  // Unix timestamp (seconds)
  expires_at: number;   // Unix timestamp (seconds)
}

/**
 * ENS Resolver with three-tier caching
 */
export class ENSResolver {
  private memoryCache: Map<string, ENSInfo | null> = new Map();
  private publicClient;

  // Cache TTLs
  private readonly MEMORY_TTL = 5 * 60 * 1000; // 5 minutes in milliseconds
  private readonly SQLITE_TTL = 7 * 24 * 60 * 60; // 7 days in seconds

  constructor() {
    // Initialize viem public client for Ethereum mainnet
    this.publicClient = createPublicClient({
      chain: mainnet,
      transport: http('https://eth.llamarpc.com'), // Free public RPC
    });
  }

  /**
   * Resolve ENS name for a given address
   * Uses 3-tier caching: Memory → SQLite → RPC
   */
  async resolve(address: Address): Promise<ENSInfo | null> {
    const normalizedAddress = address.toLowerCase();

    // Layer 1: Check memory cache
    const memoryCached = this.getFromMemoryCache(normalizedAddress);
    if (memoryCached !== undefined) {
      return memoryCached;
    }

    // Layer 2: Check SQLite cache
    const sqliteCached = await this.getFromSQLiteCache(normalizedAddress);
    if (sqliteCached !== undefined) {
      // Store in memory cache for faster subsequent access
      this.storeInMemoryCache(normalizedAddress, sqliteCached);
      return sqliteCached;
    }

    // Layer 3: RPC resolution
    try {
      const ensInfo = await this.resolveViaRPC(address);

      // Cache the result (even if null)
      await this.cacheResult(normalizedAddress, ensInfo);

      return ensInfo;
    } catch (error) {
      console.error('ENS resolution failed:', error);
      return null;
    }
  }

  /**
   * Batch resolve multiple addresses
   */
  async resolveBatch(addresses: Address[]): Promise<Map<Address, ENSInfo | null>> {
    const results = new Map<Address, ENSInfo | null>();

    // Resolve all addresses in parallel
    const promises = addresses.map(async (address) => {
      const ensInfo = await this.resolve(address);
      results.set(address, ensInfo);
    });

    await Promise.all(promises);

    return results;
  }

  /**
   * Clear cache for specific address or all
   */
  clearCache(address?: Address): void {
    if (address) {
      this.memoryCache.delete(address.toLowerCase());
    } else {
      this.memoryCache.clear();
    }
  }

  /**
   * Pre-warm cache with frequently used addresses
   */
  async warmCache(addresses: Address[]): Promise<void> {
    if (addresses.length === 0) return;

    // Resolve all addresses in background
    await this.resolveBatch(addresses);
  }

  /**
   * Get from memory cache (Layer 1)
   */
  private getFromMemoryCache(address: string): ENSInfo | null | undefined {
    const cached = this.memoryCache.get(address);

    if (cached === undefined) {
      return undefined; // Not in cache
    }

    if (cached === null) {
      return null; // Cached negative result
    }

    // Check if expired
    const now = Date.now();
    if (now - cached.resolvedAt > this.MEMORY_TTL) {
      this.memoryCache.delete(address);
      return undefined;
    }

    return cached;
  }

  /**
   * Store in memory cache (Layer 1)
   */
  private storeInMemoryCache(address: string, ensInfo: ENSInfo | null): void {
    this.memoryCache.set(address, ensInfo);
  }

  /**
   * Get from SQLite cache (Layer 2)
   */
  private async getFromSQLiteCache(address: string): Promise<ENSInfo | null | undefined> {
    try {
      const cached = await invoke<ENSCacheEntry | null>('get_ens_cache', {
        address,
      });

      if (!cached) {
        return undefined; // Not in cache
      }

      // Check if expired
      const now = Math.floor(Date.now() / 1000);
      if (now > cached.expires_at) {
        return undefined; // Expired
      }

      // Return cached ENS info
      if (!cached.ens_name) {
        return null; // Cached negative result
      }

      return {
        name: cached.ens_name,
        avatar: null, // Avatar not cached in SQLite (would require additional field)
        resolvedAt: cached.resolved_at * 1000, // Convert to milliseconds
        ttl: cached.expires_at - cached.resolved_at,
      };
    } catch (error) {
      console.error('Failed to get from SQLite cache:', error);
      return undefined;
    }
  }

  /**
   * Save to SQLite cache (Layer 2)
   */
  private async saveToSQLiteCache(address: string, ensInfo: ENSInfo | null): Promise<void> {
    try {
      await invoke('save_ens_cache', {
        address,
        ensName: ensInfo?.name || null,
        ttlDays: 7, // 7 days TTL
      });
    } catch (error) {
      console.error('Failed to save to SQLite cache:', error);
    }
  }

  /**
   * Resolve ENS via RPC (Layer 3)
   */
  private async resolveViaRPC(address: Address): Promise<ENSInfo | null> {
    const ensName = await this.publicClient.getEnsName({ address });

    if (!ensName) {
      return null; // No ENS name for this address
    }

    // Try to get avatar (optional, may fail)
    let ensAvatar: string | null = null;
    try {
      ensAvatar = await this.publicClient.getEnsAvatar({
        name: ensName,
      });
    } catch (error) {
      // Avatar resolution failed, continue without it
      console.warn('Failed to resolve ENS avatar:', error);
    }

    return {
      name: ensName,
      avatar: ensAvatar,
      resolvedAt: Date.now(),
      ttl: this.MEMORY_TTL / 1000, // Convert to seconds
    };
  }

  /**
   * Cache result in all layers
   */
  private async cacheResult(address: string, ensInfo: ENSInfo | null): Promise<void> {
    // Store in memory cache
    this.storeInMemoryCache(address, ensInfo);

    // Store in SQLite cache
    await this.saveToSQLiteCache(address, ensInfo);
  }
}

// Singleton instance
export const ensResolver = new ENSResolver();
