// WalletConnect Store - Manages WalletConnect connection for Tauri desktop app
import UniversalProvider from "@walletconnect/universal-provider";
import type { SessionTypes } from "@walletconnect/types";
import { createWalletClient, custom, type Address } from "viem";
import { arbitrumSepolia } from "viem/chains";
import { invoke } from "@tauri-apps/api/core";

// WalletConnect Project ID - Get from https://cloud.walletconnect.com/
const WALLETCONNECT_PROJECT_ID = "66514bc26e650775fc6944f25392cf4c";

class WalletConnectStore {
  address = $state<Address | null>(null);
  chainId = $state<number | null>(null);
  isConnected = $state(false);
  isConnecting = $state(false);
  error = $state<string | null>(null);
  uri = $state<string | null>(null); // QR code URI for mobile wallet connection

  private provider: UniversalProvider | null = null;
  private session: SessionTypes.Struct | null = null;

  constructor() {
    // Auto-restore session on mount
    this.restoreSession();
  }

  /**
   * Initialize WalletConnect provider
   */
  private async initProvider() {
    if (this.provider) return this.provider;

    try {
      this.provider = await UniversalProvider.init({
        projectId: WALLETCONNECT_PROJECT_ID,
        metadata: {
          name: "Colimail",
          description: "Secure email client with CMVH verification",
          url: "https://colimail.app",
          icons: ["https://colimail.app/icon.png"],
        },
      });

      // Set default chain to prevent "Cannot read properties of undefined (reading 'setDefaultChain')" error
      if (this.provider.setDefaultChain) {
        this.provider.setDefaultChain(`eip155:${arbitrumSepolia.id}`);
      }

      // Setup event listeners
      this.provider.on("display_uri", (uri: string) => {
        console.log("WalletConnect URI:", uri);
        this.uri = uri; // This will be used to generate QR code
      });

      this.provider.on("session_delete", () => {
        console.log("Session deleted");
        this.disconnect();
      });

      return this.provider;
    } catch (error) {
      console.error("Failed to initialize WalletConnect:", error);
      throw error;
    }
  }

  /**
   * Connect to wallet via WalletConnect
   */
  async connect() {
    // Prevent starting a new connection if already connecting
    if (this.isConnecting) {
      console.log("Connection already in progress, skipping");
      return;
    }

    console.log("Starting WalletConnect connection...");
    this.isConnecting = true;
    this.error = null;
    this.uri = null;

    try {
      const provider = await this.initProvider();

      // Check if connection was cancelled during provider initialization
      if (!this.isConnecting) {
        console.log("Connection cancelled during initialization");
        return;
      }

      // Connect and get session
      const session = await provider.connect({
        namespaces: {
          eip155: {
            methods: [
              "eth_sendTransaction",
              "eth_signTransaction",
              "eth_sign",
              "personal_sign",
              "eth_signTypedData",
            ],
            chains: [`eip155:${arbitrumSepolia.id}`],
            events: ["chainChanged", "accountsChanged"],
            rpcMap: {
              [arbitrumSepolia.id]: arbitrumSepolia.rpcUrls.default.http[0],
            },
          },
        },
      });

      // Check if connection was cancelled during the connect call
      if (!this.isConnecting) {
        console.log("Connection cancelled during connect");
        return;
      }

      if (session) {
        this.session = session as any; // Type compatibility workaround for WalletConnect types

        // Extract account address
        const accounts = session.namespaces.eip155?.accounts || [];
        if (accounts.length > 0) {
          // Format: "eip155:42161:0x..."
          const account = accounts[0].split(":")[2] as Address;
          this.address = account;
          this.chainId = arbitrumSepolia.id;
          this.isConnected = true;

          console.log("WalletConnect connected:", {
            address: this.address,
            chainId: this.chainId,
          });

          // Save session to secure storage
          await this.saveSessionToStorage();
        }
      }
    } catch (error) {
      console.error("WalletConnect connection failed:", error);
      // Only set error if we're still in connecting state (not cancelled)
      if (this.isConnecting) {
        this.error = error instanceof Error ? error.message : String(error);
      }
    } finally {
      // Only clear isConnecting if we're still in connecting state
      // This prevents race condition where cancel sets it to false, then finally sets it to false again
      if (this.isConnecting) {
        this.isConnecting = false;
        this.uri = null; // Clear URI after connection
        console.log("Connection attempt finished");
      }
    }
  }

  /**
   * Disconnect from wallet
   */
  async disconnect() {
    if (this.provider && this.session) {
      try {
        // Disconnect from WalletConnect (this also clears the session from WalletConnect's storage)
        await this.provider.disconnect();
        console.log("✅ WalletConnect provider disconnected");
      } catch (error) {
        console.error("Failed to disconnect WalletConnect:", error);
      }
    }

    // Clear all state
    this.address = null;
    this.chainId = null;
    this.isConnected = false;
    this.session = null;
    this.provider = null;

    // Clear session from secure storage (OS keyring)
    await this.clearSessionFromStorage();
  }

  /**
   * Cancel ongoing connection attempt
   * Used when user closes dialog or clicks cancel button before completing connection
   */
  async cancelConnection() {
    console.log("Cancelling WalletConnect connection...", {
      isConnecting: this.isConnecting,
      hasProvider: !!this.provider,
      uri: this.uri
    });

    try {
      // If provider exists and is connecting, disconnect it
      if (this.provider) {
        try {
          await this.provider.disconnect();
        } catch (error) {
          // Ignore disconnect errors during cancellation
          console.warn("Error during connection cancellation:", error);
        }
      }

      // Force clear all connection-related state immediately
      this.uri = null;
      this.isConnecting = false;
      this.error = null;
      this.provider = null; // Clear provider to force re-initialization

      console.log("✅ Connection cancelled successfully - state cleared");
    } catch (error) {
      console.error("Failed to cancel connection:", error);
      // Still force clear state even if cancellation failed
      this.uri = null;
      this.isConnecting = false;
      this.error = null;
      this.provider = null;
    }
  }

  /**
   * Restore previous session if available
   */
  async restoreSession() {
    try {
      const provider = await this.initProvider();

      if (provider.session) {
        this.session = provider.session as any; // Type compatibility issue with WalletConnect types
        const accounts = provider.session.namespaces.eip155?.accounts || [];

        if (accounts.length > 0) {
          const account = accounts[0].split(":")[2] as Address;
          this.address = account;
          this.chainId = arbitrumSepolia.id;
          this.isConnected = true;

          console.log("WalletConnect session restored:", {
            address: this.address,
          });

          // IMPORTANT: Save session to OS keyring for startup confirmation dialog
          // Preserve existing timestamp if session already exists in keyring
          await this.saveSessionToStoragePreserveTimestamp();
        }
      }
    } catch (error) {
      console.warn("Failed to restore WalletConnect session:", error);
    }
  }

  /**
   * Get viem wallet client for signing transactions
   */
  getWalletClient() {
    if (!this.provider || !this.isConnected) {
      throw new Error("WalletConnect not connected");
    }

    return createWalletClient({
      chain: arbitrumSepolia,
      transport: custom(this.provider),
    });
  }

  /**
   * Save session to secure storage
   */
  private async saveSessionToStorage() {
    if (!this.isConnected || !this.address || !this.session) return;

    try {
      await invoke("save_wallet_session", {
        session: {
          address: this.address,
          chain_id: this.chainId || 421614,
          connection_method: "walletconnect",
          last_active_timestamp: Math.floor(Date.now() / 1000),
          wc_session_topic: this.session.topic,
        }
      });
      console.log("✅ Wallet session saved to secure storage");
    } catch (error) {
      console.error("Failed to save wallet session:", error);
    }
  }

  /**
   * Save session to secure storage, preserving existing timestamp if available
   * This is used during session restoration to avoid updating the last_active time
   */
  private async saveSessionToStoragePreserveTimestamp() {
    if (!this.isConnected || !this.address || !this.session) return;

    try {
      // Try to get existing session to preserve timestamp
      const existingSession = await this.loadSessionFromStorage();
      const timestamp = existingSession?.last_active_timestamp || Math.floor(Date.now() / 1000);

      await invoke("save_wallet_session", {
        session: {
          address: this.address,
          chain_id: this.chainId || 421614,
          connection_method: "walletconnect",
          last_active_timestamp: timestamp,
          wc_session_topic: this.session.topic,
        }
      });
      console.log("✅ Wallet session saved to secure storage (timestamp preserved)");
    } catch (error) {
      console.error("Failed to save wallet session:", error);
    }
  }

  /**
   * Load session from secure storage
   */
  async loadSessionFromStorage(): Promise<WalletSession | null> {
    try {
      const session = await invoke<WalletSession | null>("get_wallet_session");
      return session;
    } catch (error) {
      console.error("Failed to load wallet session:", error);
      return null;
    }
  }

  /**
   * Clear session from secure storage
   */
  async clearSessionFromStorage() {
    try {
      await invoke("delete_wallet_session");
      console.log("✅ Wallet session cleared from secure storage");
    } catch (error) {
      console.error("Failed to clear wallet session:", error);
    }
  }
}

export const walletConnectStore = new WalletConnectStore();
