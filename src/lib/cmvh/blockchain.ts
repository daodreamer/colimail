// CMVH Blockchain Verifier - On-chain signature verification using viem

import { createPublicClient, http, type Address, type Hex } from "viem";
import { arbitrum, arbitrumSepolia } from "viem/chains";
import type { CMVHHeaders, EmailContent, CMVHConfig } from "./types";
import { NETWORK_CONFIG } from "./types";

// CMVHVerifier contract ABI (only the functions we need)
const CMVH_VERIFIER_ABI = [
  {
    name: "verifySignature",
    type: "function",
    stateMutability: "view",
    inputs: [
      { name: "signer", type: "address" },
      { name: "emailHash", type: "bytes32" },
      { name: "signature", type: "bytes" },
    ],
    outputs: [{ name: "isValid", type: "bool" }],
  },
  {
    name: "verifyEmail",
    type: "function",
    stateMutability: "view",
    inputs: [
      { name: "signer", type: "address" },
      { name: "subject", type: "string" },
      { name: "from", type: "string" },
      { name: "to", type: "string" },
      { name: "body", type: "string" },
      { name: "signature", type: "bytes" },
    ],
    outputs: [{ name: "isValid", type: "bool" }],
  },
] as const;

/**
 * Create viem public client for blockchain calls
 */
export function createCMVHClient(config: CMVHConfig) {
  const networkConfig = NETWORK_CONFIG[config.network];
  const chain = config.network === "arbitrum" ? arbitrum : arbitrumSepolia;

  return createPublicClient({
    chain,
    transport: http(config.rpcUrl || networkConfig.rpcUrl),
  });
}

/**
 * Verify signature on-chain
 */
export async function verifyOnChain(
  headers: CMVHHeaders,
  content: EmailContent,
  config: CMVHConfig
): Promise<{ isValid: boolean; error?: string }> {
  try {
    const client = createCMVHClient(config);
    const networkConfig = NETWORK_CONFIG[config.network];

    // Call contract verifyEmail function
    const isValid = await client.readContract({
      address: (config.contractAddress ||
        networkConfig.contractAddress) as Address,
      abi: CMVH_VERIFIER_ABI,
      functionName: "verifyEmail",
      args: [
        headers.address as Address,
        content.subject,
        content.from,
        content.to,
        content.body,
        headers.signature as Hex,
      ],
    });

    return { isValid };
  } catch (error) {
    console.error("On-chain verification failed:", error);
    return {
      isValid: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Get explorer URL for address
 */
export function getExplorerUrl(
  address: string,
  network: "arbitrum" | "arbitrum-sepolia"
): string {
  const explorerUrl = NETWORK_CONFIG[network].explorerUrl;
  return `${explorerUrl}/address/${address}`;
}

/**
 * Get explorer URL for transaction
 */
export function getTxExplorerUrl(
  txHash: string,
  network: "arbitrum" | "arbitrum-sepolia"
): string {
  const explorerUrl = NETWORK_CONFIG[network].explorerUrl;
  return `${explorerUrl}/tx/${txHash}`;
}

/**
 * Format address (0x1234...5678)
 */
export function formatAddress(address: string): string {
  if (!address || address.length < 10) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

/**
 * Format timestamp to readable date
 */
export function formatTimestamp(timestamp: string): string {
  try {
    const date = new Date(parseInt(timestamp) * 1000);
    return date.toLocaleString();
  } catch {
    return timestamp;
  }
}
