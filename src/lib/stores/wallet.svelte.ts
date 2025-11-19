// Wallet Store - Manages wallet connection state using WalletConnect only
import type { Address } from "viem";
import { walletConnectStore } from "./walletconnect.svelte";

/**
 * Unified wallet interface that wraps WalletConnect
 * This store provides a simplified API for the rest of the application
 */
class WalletStore {
    error = $state<string | null>(null);

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
