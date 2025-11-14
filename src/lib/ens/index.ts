// ENS module - Ethereum Name Service integration
//
// Provides ENS name resolution with dual-layer caching for optimal performance.

export {
	cleanupENSCache,
	clearENSCache,
	getCachedENS,
	getENSCacheStats,
	saveENSCache,
	type ENSCacheStats,
} from "./cache";

export {
	formatAddressWithENS,
	preloadENS,
	resolveENSBatch,
	resolveENSWithCache,
} from "./resolver";
