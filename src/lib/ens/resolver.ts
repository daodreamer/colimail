// ENS Resolver - Reverse resolution using viem
//
// Resolves Ethereum addresses to ENS names using mainnet ENS contracts.
// Supports caching and batch resolution for optimal performance.

import { createPublicClient, http, type Address } from "viem";
import { mainnet } from "viem/chains";
import { getCachedENS, saveENSCache } from "./cache";

// Create viem client for ENS resolution (must use mainnet)
const ensClient = createPublicClient({
	chain: mainnet,
	transport: http("https://eth-mainnet.g.alchemy.com/v2/demo"), // Using demo API key, replace with your own for production
});

/**
 * Resolve ENS name for an address with caching
 *
 * Flow:
 * 1. Check cache (L1 + L2)
 * 2. If cache miss, query ENS via viem
 * 3. Save result to cache
 * 4. Return ENS name or null
 *
 * @param address - Ethereum address
 * @returns ENS name if found, null if no ENS name exists
 */
export async function resolveENSWithCache(
	address: string,
): Promise<string | null> {
	try {
		// Step 1: Check cache
		const cached = await getCachedENS(address);
		if (cached !== undefined) {
			// Cache hit (can be null for "no ENS")
			return cached;
		}

		// Step 2: Cache miss - query ENS
		console.log(`🔍 Resolving ENS for ${address}...`);
		const ensName = await resolveENS(address);

		// Step 3: Save to cache
		await saveENSCache(address, ensName);

		// Step 4: Return result
		return ensName;
	} catch (error) {
		console.error(`Failed to resolve ENS for ${address}:`, error);
		return null;
	}
}

/**
 * Resolve ENS name for an address (direct RPC call, no caching)
 *
 * @param address - Ethereum address
 * @returns ENS name if found, null if no ENS name exists or error
 */
async function resolveENS(address: string): Promise<string | null> {
	try {
		const ensName = await ensClient.getEnsName({
			address: address as Address,
		});

		console.log(`✅ Resolved ${address} -> ${ensName || "no ENS"}`);
		return ensName;
	} catch (error) {
		console.error(`❌ ENS resolution failed for ${address}:`, error);
		return null;
	}
}

/**
 * Batch resolve ENS names for multiple addresses
 *
 * Uses concurrent requests with rate limiting to avoid overwhelming the RPC endpoint.
 *
 * @param addresses - Array of Ethereum addresses
 * @param concurrency - Maximum concurrent requests (default: 5)
 * @returns Map of address -> ENS name
 */
export async function resolveENSBatch(
	addresses: string[],
	concurrency = 5,
): Promise<Map<string, string | null>> {
	const results = new Map<string, string | null>();

	// Filter out addresses already in cache
	const uncachedAddresses: string[] = [];
	for (const address of addresses) {
		const cached = await getCachedENS(address);
		if (cached !== undefined) {
			results.set(address, cached);
		} else {
			uncachedAddresses.push(address);
		}
	}

	if (uncachedAddresses.length === 0) {
		console.log("✅ All addresses found in cache");
		return results;
	}

	console.log(
		`🔍 Resolving ${uncachedAddresses.length} addresses (${results.size} cached)`,
	);

	// Process in batches to limit concurrency
	for (let i = 0; i < uncachedAddresses.length; i += concurrency) {
		const batch = uncachedAddresses.slice(i, i + concurrency);

		const promises = batch.map(async (address) => {
			const ensName = await resolveENS(address);
			await saveENSCache(address, ensName);
			return { address, ensName };
		});

		const batchResults = await Promise.all(promises);
		for (const { address, ensName } of batchResults) {
			results.set(address, ensName);
		}
	}

	console.log(`✅ Resolved ${results.size} addresses total`);
	return results;
}

/**
 * Preload ENS names for a list of addresses in the background
 *
 * Useful for preloading ENS names when loading a list of emails.
 * Does not block, runs asynchronously.
 *
 * @param addresses - Array of Ethereum addresses
 */
export function preloadENS(addresses: string[]): void {
	if (addresses.length === 0) return;

	console.log(`🔄 Preloading ENS for ${addresses.length} addresses...`);

	// Run in background (don't await)
	setTimeout(async () => {
		try {
			await resolveENSBatch(addresses);
			console.log(`✅ Preload complete`);
		} catch (error) {
			console.error("Preload failed:", error);
		}
	}, 1000); // Delay 1 second to avoid blocking UI
}

/**
 * Format address with ENS name if available
 *
 * @param address - Ethereum address
 * @param ensName - ENS name (optional)
 * @returns Formatted string: "vitalik.eth (0x1234...5678)" or "0x1234...5678"
 */
export function formatAddressWithENS(
	address: string,
	ensName: string | null | undefined,
): string {
	const shortAddress = `${address.slice(0, 6)}...${address.slice(-4)}`;

	if (ensName) {
		return `${ensName} (${shortAddress})`;
	}

	return shortAddress;
}
