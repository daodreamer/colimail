/**
 * ENS Resolver Tests
 * Focus: Discovering bugs in request deduplication, batch resolution, and caching
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ENSResolver } from './ens-resolver';
import type { Address } from 'viem';

// Mock viem
vi.mock('viem', async () => {
  const actual = await vi.importActual('viem');
  return {
    ...actual,
    createPublicClient: vi.fn(() => ({
      getEnsName: vi.fn(),
      getEnsAvatar: vi.fn(),
    })),
    http: vi.fn(),
  };
});

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('ENSResolver - Request Deduplication', () => {
  let resolver: ENSResolver;
  let mockGetEnsName: any;

  beforeEach(() => {
    resolver = new ENSResolver();
    // @ts-ignore - accessing private property for testing
    mockGetEnsName = resolver.publicClient.getEnsName;
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('should deduplicate concurrent requests for same address', async () => {
    const testAddress = '0x1234567890123456789012345678901234567890' as Address;

    // Mock RPC to return ENS name after delay
    mockGetEnsName.mockImplementation(() =>
      new Promise((resolve) => setTimeout(() => resolve('test.eth'), 100))
    );

    // Fire 5 concurrent requests for same address
    const promises = Array(5)
      .fill(null)
      .map(() => resolver.resolve(testAddress));

    const results = await Promise.all(promises);

    // All should get same result
    results.forEach((result) => {
      expect(result?.name).toBe('test.eth');
    });

    // Should only call RPC once (deduplication working)
    expect(mockGetEnsName).toHaveBeenCalledTimes(1);
  });

  it('should not deduplicate sequential requests (after first completes)', async () => {
    const testAddress = '0x1234567890123456789012345678901234567890' as Address;

    mockGetEnsName.mockResolvedValue('test.eth');

    // First request
    await resolver.resolve(testAddress);

    // Second request after first completes
    await resolver.resolve(testAddress);

    // First call = RPC, second call = memory cache (not another RPC)
    expect(mockGetEnsName).toHaveBeenCalledTimes(1);
  });

  it('should handle concurrent requests for different addresses', async () => {
    const addr1 = '0x1111111111111111111111111111111111111111' as Address;
    const addr2 = '0x2222222222222222222222222222222222222222' as Address;

    mockGetEnsName
      .mockImplementation((params: any) => {
        if (params.address === addr1) return Promise.resolve('alice.eth');
        if (params.address === addr2) return Promise.resolve('bob.eth');
        return Promise.resolve(null);
      });

    const [result1, result2] = await Promise.all([
      resolver.resolve(addr1),
      resolver.resolve(addr2),
    ]);

    expect(result1?.name).toBe('alice.eth');
    expect(result2?.name).toBe('bob.eth');
    expect(mockGetEnsName).toHaveBeenCalledTimes(2);
  });
});

describe('ENSResolver - Batch Resolution', () => {
  let resolver: ENSResolver;
  let mockGetEnsName: any;

  beforeEach(() => {
    resolver = new ENSResolver();
    // @ts-ignore
    mockGetEnsName = resolver.publicClient.getEnsName;
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('should resolve multiple addresses in parallel', async () => {
    const addresses = [
      '0x1111111111111111111111111111111111111111',
      '0x2222222222222222222222222222222222222222',
      '0x3333333333333333333333333333333333333333',
    ] as Address[];

    mockGetEnsName.mockImplementation((params: any) => {
      const addr = params.address.toLowerCase();
      if (addr.includes('1111')) return Promise.resolve('alice.eth');
      if (addr.includes('2222')) return Promise.resolve('bob.eth');
      if (addr.includes('3333')) return Promise.resolve(null);
      return Promise.resolve(null);
    });

    const results = await resolver.resolveBatch(addresses);

    expect(results.size).toBe(3);
    expect(results.get(addresses[0])?.name).toBe('alice.eth');
    expect(results.get(addresses[1])?.name).toBe('bob.eth');
    expect(results.get(addresses[2])).toBeNull();
  });

  it('should batch resolve duplicate addresses only once', async () => {
    const duplicateAddresses = [
      '0x1111111111111111111111111111111111111111',
      '0x1111111111111111111111111111111111111111', // duplicate
      '0x2222222222222222222222222222222222222222',
    ] as Address[];

    mockGetEnsName.mockImplementation((params: any) => {
      const addr = params.address.toLowerCase();
      if (addr.includes('1111')) return Promise.resolve('alice.eth');
      if (addr.includes('2222')) return Promise.resolve('bob.eth');
      return Promise.resolve(null);
    });

    await resolver.resolveBatch(duplicateAddresses);

    // Should only call RPC twice (deduplication for duplicate address)
    expect(mockGetEnsName).toHaveBeenCalledTimes(2);
  });
});

describe('ENSResolver - Caching', () => {
  let resolver: ENSResolver;
  let mockGetEnsName: any;

  beforeEach(() => {
    resolver = new ENSResolver();
    // @ts-ignore
    mockGetEnsName = resolver.publicClient.getEnsName;
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('should cache results in memory', async () => {
    const testAddress = '0x1234567890123456789012345678901234567890' as Address;

    mockGetEnsName.mockResolvedValue('test.eth');

    // First call - should hit RPC
    await resolver.resolve(testAddress);

    // Second call - should use cache
    await resolver.resolve(testAddress);

    // Only called once (cache hit)
    expect(mockGetEnsName).toHaveBeenCalledTimes(1);
  });

  it('should return cached result synchronously via getCached', async () => {
    const testAddress = '0x1234567890123456789012345678901234567890' as Address;

    mockGetEnsName.mockResolvedValue('test.eth');

    // Resolve first to populate cache
    await resolver.resolve(testAddress);

    // Get cached result synchronously
    const cached = resolver.getCached(testAddress);

    expect(cached?.name).toBe('test.eth');
  });

  it('should return undefined from getCached if not in cache', () => {
    const testAddress = '0x9999999999999999999999999999999999999999' as Address;

    const cached = resolver.getCached(testAddress);

    expect(cached).toBeUndefined();
  });

  it('should cache null results (negative caching)', async () => {
    const testAddress = '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' as Address;

    // Mock no ENS name found
    mockGetEnsName.mockResolvedValue(null);

    // First call
    const result1 = await resolver.resolve(testAddress);
    expect(result1).toBeNull();

    // Second call - should use cached null
    const result2 = await resolver.resolve(testAddress);
    expect(result2).toBeNull();

    // Only called once (null was cached)
    expect(mockGetEnsName).toHaveBeenCalledTimes(1);
  });

  it('should clear cache for specific address', async () => {
    const testAddress = '0x1234567890123456789012345678901234567890' as Address;

    mockGetEnsName.mockResolvedValue('test.eth');

    // Populate cache
    await resolver.resolve(testAddress);

    // Clear cache for this address
    resolver.clearCache(testAddress);

    // Next call should hit RPC again
    await resolver.resolve(testAddress);

    expect(mockGetEnsName).toHaveBeenCalledTimes(2);
  });

  it('should clear all cache', async () => {
    const addr1 = '0x1111111111111111111111111111111111111111' as Address;
    const addr2 = '0x2222222222222222222222222222222222222222' as Address;

    mockGetEnsName
      .mockResolvedValueOnce('alice.eth')
      .mockResolvedValueOnce('bob.eth')
      .mockResolvedValueOnce('alice.eth')
      .mockResolvedValueOnce('bob.eth');

    // Populate cache
    await resolver.resolve(addr1);
    await resolver.resolve(addr2);

    // Clear all cache
    resolver.clearCache();

    // Next calls should hit RPC again
    await resolver.resolve(addr1);
    await resolver.resolve(addr2);

    expect(mockGetEnsName).toHaveBeenCalledTimes(4);
  });
});

describe('ENSResolver - Edge Cases', () => {
  let resolver: ENSResolver;
  let mockGetEnsName: any;

  beforeEach(() => {
    resolver = new ENSResolver();
    // @ts-ignore
    mockGetEnsName = resolver.publicClient.getEnsName;
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it('should handle RPC errors gracefully', async () => {
    const testAddress = '0x1234567890123456789012345678901234567890' as Address;

    // Mock RPC error
    mockGetEnsName.mockRejectedValue(new Error('Network error'));

    const result = await resolver.resolve(testAddress);

    // Should return null on error (not throw)
    expect(result).toBeNull();
  });

  it('should handle case-insensitive addresses', async () => {
    const lowerCaseAddr = '0xabcdef1234567890abcdef1234567890abcdef12' as Address;
    const upperCaseAddr = '0xABCDEF1234567890ABCDEF1234567890ABCDEF12' as Address;
    const mixedCaseAddr = '0xAbCdEf1234567890AbCdEf1234567890AbCdEf12' as Address;

    mockGetEnsName.mockResolvedValue('test.eth');

    // Resolve with lowercase
    await resolver.resolve(lowerCaseAddr);

    // Should use cache for uppercase and mixed case
    await resolver.resolve(upperCaseAddr);
    await resolver.resolve(mixedCaseAddr);

    // Should only call RPC once (case normalization working)
    expect(mockGetEnsName).toHaveBeenCalledTimes(1);
  });

  it('should handle empty batch resolution', async () => {
    const results = await resolver.resolveBatch([]);

    expect(results.size).toBe(0);
    expect(mockGetEnsName).not.toHaveBeenCalled();
  });
});
