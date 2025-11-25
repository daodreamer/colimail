// CMVH Signer - Frontend EIP-712 signing with WalletConnect

import type { EmailContent, CMVHHeaders } from "./types";
import { walletStore } from "$lib/stores/wallet.svelte";
import { arbitrumSepolia } from "viem/chains";

/**
 * EIP-712 Domain for CMVH v2
 * MUST match backend signer.rs exactly!
 */
const EIP712_DOMAIN = {
  name: "CMVHVerifier",
  version: "2.0.0",
  chainId: arbitrumSepolia.id, // 421614 (Arbitrum Sepolia)
  verifyingContract: "0x8f7B72f66C3bC42A8ca6207fDAc7ec1a07641F03", // CMVH contract on Arbitrum Sepolia
};

/**
 * EIP-712 Email type definition
 */
const EIP712_EMAIL_TYPE = {
  Email: [
    { name: "subject", type: "string" },
    { name: "from", type: "string" },
    { name: "to", type: "string" },
    { name: "timestamp", type: "uint256" },
  ],
};

/**
 * Sign email content with WalletConnect using EIP-712
 * @param content Email content to sign
 * @returns CMVH headers with signature
 * @throws Error if wallet not connected or signing fails
 */
export async function signEmailWithWallet(
  content: EmailContent
): Promise<CMVHHeaders> {
  // Check wallet connection
  if (!walletStore.isConnected || !walletStore.address) {
    throw new Error("Wallet not connected. Please connect wallet first.");
  }

  const provider = walletStore.getWalletConnectProvider();
  const session = walletStore.getWalletConnectSession();

  if (!provider || !session) {
    throw new Error("WalletConnect not initialized");
  }

  // Generate timestamp
  const timestamp = Math.floor(Date.now() / 1000);

  // Prepare EIP-712 message
  const message = {
    subject: content.subject,
    from: content.from,
    to: content.to,
    timestamp: timestamp,
  };

  const typedData = {
    types: {
      EIP712Domain: [
        { name: "name", type: "string" },
        { name: "version", type: "string" },
        { name: "chainId", type: "uint256" },
        { name: "verifyingContract", type: "address" },
      ],
      ...EIP712_EMAIL_TYPE,
    },
    primaryType: "Email" as const,
    domain: EIP712_DOMAIN,
    message: message,
  };

  console.log("📝 Preparing EIP-712 signature request:", {
    domain: EIP712_DOMAIN,
    message: message,
    typedData: typedData,
  });

  try {
    // Request signature from wallet via WalletConnect
    // Note: We don't specify the chain parameter because:
    // 1. EIP-712 signing is chain-agnostic (signature is the same regardless of current chain)
    // 2. The chain info is already in the domain (chainId: 421614)
    // 3. Specifying chain can cause "Missing or invalid chainId" errors if wallet isn't on that chain
    const result = await provider.request({
      method: "eth_signTypedData_v4",
      params: [
        walletStore.address, // Address (must match connected wallet)
        JSON.stringify(typedData), // EIP-712 typed data as JSON string
      ],
    });

    console.log("✅ Signature received from wallet:", result);

    // Construct CMVH headers (MUST match backend format)
    const cmvhHeaders: CMVHHeaders = {
      version: "2",
      address: walletStore.address.toLowerCase(),
      chain: "Arbitrum", // Match backend signer.rs:148
      timestamp: timestamp.toString(),
      hash_algo: "eip712",
      signature: result as string,
      ens: walletStore.ensName || undefined,
    };

    return cmvhHeaders;
  } catch (error) {
    console.error("❌ Failed to sign email with wallet:", error);

    // Log detailed error information for debugging
    if (typeof error === 'object' && error !== null) {
      console.error("Error details:", {
        code: (error as any).code,
        message: (error as any).message,
        data: (error as any).data,
      });
    }

    // User rejected signature
    if (error instanceof Error && error.message.includes("User rejected")) {
      throw new Error("Signature rejected by user");
    }

    // Format error message
    const errorMessage = typeof error === 'object' && error !== null
      ? (error as any).message || JSON.stringify(error)
      : String(error);

    throw new Error(`Failed to sign email: ${errorMessage}`);
  }
}

/**
 * Verify that wallet is ready for signing
 * @returns True if wallet is connected and ready
 */
export function isWalletReadyForSigning(): boolean {
  return (
    walletStore.isConnected &&
    !!walletStore.address &&
    !!walletStore.getWalletConnectProvider() &&
    !!walletStore.getWalletConnectSession()
  );
}
