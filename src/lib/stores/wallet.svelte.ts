// Wallet Store - Manages wallet connection state using Svelte 5 runes
import { createPublicClient, custom, type Address, type WalletClient, createWalletClient } from "viem";
import { arbitrum, arbitrumSepolia } from "viem/chains";
import { NETWORK_CONFIG, type CMVHConfig } from "$lib/cmvh/types";

class WalletStore {
    address = $state<Address | null>(null);
    chainId = $state<number | null>(null);
    isConnected = $state(false);
    isConnecting = $state(false);
    error = $state<string | null>(null);

    // We don't store the client in state as it's not serializable/reactive in the same way
    // access it via getClient()

    constructor() {
        // Auto-connect on mount if previously connected
        this.checkConnection();
    }

    async checkConnection() {
        if (typeof window === "undefined" || !window.ethereum) return;

        try {
            const client = createWalletClient({
                chain: arbitrumSepolia, // Default to testnet for now
                transport: custom(window.ethereum)
            });

            const addresses = await client.getAddresses();
            if (addresses.length > 0) {
                this.address = addresses[0];
                this.isConnected = true;

                const chainId = await client.getChainId();
                this.chainId = chainId;
            }
        } catch (e) {
            console.error("Failed to check wallet connection:", e);
        }
    }

    async connect() {
        if (typeof window === "undefined" || !window.ethereum) {
            this.error = "No wallet found. Please install MetaMask or Rabby.";
            return;
        }

        this.isConnecting = true;
        this.error = null;

        try {
            const client = createWalletClient({
                chain: arbitrumSepolia,
                transport: custom(window.ethereum)
            });

            const addresses = await client.requestAddresses();

            if (addresses.length > 0) {
                this.address = addresses[0];
                this.isConnected = true;

                const chainId = await client.getChainId();
                this.chainId = chainId;
            }
        } catch (e) {
            console.error("Failed to connect wallet:", e);
            this.error = e instanceof Error ? e.message : String(e);
        } finally {
            this.isConnecting = false;
        }
    }

    async disconnect() {
        this.address = null;
        this.chainId = null;
        this.isConnected = false;
        // Note: We can't strictly "disconnect" from the wallet side via API, 
        // but we clear our local state.
    }

    async switchNetwork(network: "arbitrum" | "arbitrum-sepolia") {
        if (!window.ethereum) return;

        const config = NETWORK_CONFIG[network];
        const chainIdHex = `0x${config.chainId.toString(16)}`;

        try {
            await window.ethereum.request({
                method: 'wallet_switchEthereumChain',
                params: [{ chainId: chainIdHex }],
            });
            this.chainId = config.chainId;
        } catch (switchError: any) {
            // This error code indicates that the chain has not been added to MetaMask.
            if (switchError.code === 4902) {
                try {
                    await window.ethereum.request({
                        method: 'wallet_addEthereumChain',
                        params: [
                            {
                                chainId: chainIdHex,
                                chainName: config.name,
                                rpcUrls: [config.rpcUrl],
                                blockExplorerUrls: [config.explorerUrl],
                            },
                        ],
                    });
                    this.chainId = config.chainId;
                } catch (addError) {
                    this.error = "Failed to add network to wallet";
                }
            } else {
                this.error = "Failed to switch network";
            }
        }
    }
}

export const walletStore = new WalletStore();
