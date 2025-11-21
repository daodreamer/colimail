// Wallet Store - Manages wallet connection state using WalletConnect only
import type { Address } from "viem";
import { walletConnectStore } from "./walletconnect.svelte";
import { ensResolver } from "$lib/services/ens-resolver";

/**
 * Unified wallet interface that wraps WalletConnect
 * This store provides a simplified API for the rest of the application
 */
class WalletStore {
    error = $state<string | null>(null);

    // ENS state
    ensName = $state<string | null>(null);
    ensAvatar = $state<string | null>(null);
    isResolvingENS = $state(false);

    // All state is derived from WalletConnect store
    get address(): Address | null {
        return walletConnectStore.address;
    }

    get chainId(): number | null {
        return walletConnectStore.chainId;
    }

    get isConnected(): boolean {
        return walletConnectStore.isConnected;
    }

    get isConnecting(): boolean {
        return walletConnectStore.isConnecting;
    }

    // WalletConnect QR code URI
    get walletConnectUri(): string | null {
        return walletConnectStore.uri;
    }

    /**
     * Connect via WalletConnect (shows QR code for mobile wallet)
     */
    async connect() {
        this.error = null;

        try {
            await walletConnectStore.connect();

            // Resolve ENS name in background after connection
            if (this.address) {
                this.resolveENS();
            }
        } catch (e) {
            console.error("Failed to connect WalletConnect:", e);
            this.error = e instanceof Error ? e.message : String(e);
        }
    }

    /**
     * Disconnect wallet
     */
    async disconnect() {
        await walletConnectStore.disconnect();
        this.error = null;

        // Clear ENS state
        this.ensName = null;
        this.ensAvatar = null;
        this.isResolvingENS = false;
    }

    /**
     * Cancel ongoing connection attempt
     * Used when user closes dialog or clicks cancel button before completing connection
     */
    async cancelConnection() {
        await walletConnectStore.cancelConnection();
        this.error = null;
    }

    /**
     * Resolve ENS name and avatar for current address
     */
    async resolveENS(): Promise<void> {
        if (!this.address) {
            console.warn('Cannot resolve ENS: no address connected');
            return;
        }

        this.isResolvingENS = true;

        try {
            const ensInfo = await ensResolver.resolve(this.address);

            if (ensInfo) {
                this.ensName = ensInfo.name;
                this.ensAvatar = ensInfo.avatar || null;
            } else {
                // No ENS name for this address
                this.ensName = null;
                this.ensAvatar = null;
            }
        } catch (error) {
            console.error('Failed to resolve ENS:', error);
            // Don't throw, just log - ENS resolution is optional
        } finally {
            this.isResolvingENS = false;
        }
    }

    /**
     * Format address for display
     * @param address - Address to format (defaults to current address)
     * @param format - 'short' (0x1234...5678) or 'full'
     */
    formatAddress(address: Address | null = this.address, format: 'short' | 'full' = 'short'): string {
        if (!address) return '';

        if (format === 'full') {
            return address;
        }

        // Short format: 0x1234...5678
        return `${address.slice(0, 6)}...${address.slice(-4)}`;
    }

    /**
     * Get display name (ENS name if available, otherwise formatted address)
     */
    getDisplayName(): string {
        if (this.ensName) {
            return this.ensName;
        }

        return this.formatAddress();
    }

    /**
     * Save current session to secure storage
     * (WalletConnect handles this internally)
     */
    async saveCurrentSession() {
        // WalletConnect handles its own session storage
        // This method exists for compatibility with existing code
    }

    /**
     * Clear session from secure storage
     */
    async clearSession() {
        await walletConnectStore.clearSessionFromStorage();
    }
}

export const walletStore = new WalletStore();
