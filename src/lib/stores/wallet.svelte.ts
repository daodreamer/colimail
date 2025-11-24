// Wallet Store - Manages wallet connection state using WalletConnect only
import type { Address } from "viem";
import { walletConnectStore } from "./walletconnect.svelte";
import { ensResolver } from "$lib/services/ens-resolver";
import { toast } from "svelte-sonner";

/**
 * Unified wallet interface that wraps WalletConnect
 * This store provides a simplified API for the rest of the application
 */
class WalletStore {
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

    // Error state from WalletConnect store
    get error(): string | null {
        return walletConnectStore.error;
    }

    // WalletConnect QR code URI
    get walletConnectUri(): string | null {
        return walletConnectStore.uri;
    }

    /**
     * Connect via WalletConnect (shows QR code for mobile wallet)
     */
    async connect() {
        try {
            await walletConnectStore.connect();

            // Show success toast with wallet info
            if (this.address) {
                // Resolve ENS name in background
                this.resolveENS();

                // Show success notification
                const displayName = this.getDisplayName();
                toast.success('Wallet connected successfully', {
                    description: displayName,
                    duration: 3000,
                });
            }
        } catch (e) {
            console.error("Failed to connect WalletConnect:", e);

            // Show friendly error toast (only for non-user-cancellation errors)
            const errorMessage = e instanceof Error ? e.message : String(e);
            const isUserCancellation = errorMessage.includes("cancelled by user");

            if (!isUserCancellation) {
                const friendlyMessage = this.getFriendlyErrorMessage(e);
                toast.error('Connection failed', {
                    description: friendlyMessage,
                    duration: 5000,
                });
            }
        }
    }

    /**
     * Disconnect wallet
     */
    async disconnect() {
        const wasConnected = this.isConnected;
        const displayName = wasConnected ? this.getDisplayName() : '';

        await walletConnectStore.disconnect();

        // Clear ENS state
        this.ensName = null;
        this.ensAvatar = null;
        this.isResolvingENS = false;

        // Show disconnect toast
        if (wasConnected) {
            toast.info('Wallet disconnected', {
                description: displayName,
                duration: 2000,
            });
        }
    }

    /**
     * Cancel ongoing connection attempt
     * Used when user closes dialog or clicks cancel button before completing connection
     */
    async cancelConnection() {
        await walletConnectStore.cancelConnection();
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

    /**
     * Get user-friendly error message for wallet connection errors
     */
    private getFriendlyErrorMessage(error: unknown): string {
        const errorMsg = error instanceof Error ? error.message : String(error);

        // User cancelled
        if (errorMsg.toLowerCase().includes('user rejected') ||
            errorMsg.toLowerCase().includes('user canceled') ||
            errorMsg.toLowerCase().includes('user denied')) {
            return 'Connection was cancelled. Please try again.';
        }

        // Timeout
        if (errorMsg.toLowerCase().includes('timeout')) {
            return 'Connection timed out. Please scan the QR code again.';
        }

        // Network errors
        if (errorMsg.toLowerCase().includes('network') ||
            errorMsg.toLowerCase().includes('fetch')) {
            return 'Network error. Please check your internet connection.';
        }

        // QR code related
        if (errorMsg.toLowerCase().includes('qr')) {
            return 'Failed to generate QR code. Please try again.';
        }

        // Generic fallback
        return 'Please try scanning the QR code again.';
    }
}

export const walletStore = new WalletStore();
