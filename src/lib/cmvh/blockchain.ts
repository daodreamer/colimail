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
    stateMutability: "pure",
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
    stateMutability: "pure",
    inputs: [
      { name: "signer", type: "address" },
      { name: "subject", type: "string" },
      { name: "from", type: "string" },
      { name: "to", type: "string" },
      { name: "signature", type: "bytes" },
    ],
    outputs: [{ name: "isValid", type: "bool" }],
  },
  {
    name: "hashEmail",
    type: "function",
    stateMutability: "pure",
    inputs: [
      { name: "subject", type: "string" },
      { name: "from", type: "string" },
      { name: "to", type: "string" },
    ],
    outputs: [{ name: "hash", type: "bytes32" }],
  },
  {
    name: "recoverSigner",
    type: "function",
    stateMutability: "pure",
    inputs: [
      { name: "emailHash", type: "bytes32" },
      { name: "signature", type: "bytes" },
    ],
    outputs: [{ name: "signer", type: "address" }],
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
    const contractAddress = (config.contractAddress || networkConfig.contractAddress) as Address;

    console.log("📋 On-chain verification parameters:");
    console.log(`   Contract: ${contractAddress}`);
    console.log(`   Signer: ${headers.address}`);
    console.log(`   Subject: "${content.subject}"`);
    console.log(`   From: "${content.from}"`);
    console.log(`   To: "${content.to}"`);
    console.log(`   Signature length: ${headers.signature.length} chars (expected: 132 for 0x + 65 bytes)`);
    console.log(`   Signature: ${headers.signature}`);

    // Test: Call contract's hashEmail function to see what hash it computes
    const contractEmailHash = await client.readContract({
      address: contractAddress,
      abi: CMVH_VERIFIER_ABI,
      functionName: "hashEmail",
      args: [content.subject, content.from, content.to],
    });
    console.log(`📊 Contract computed hash: ${contractEmailHash}`);

    // Test: Try to recover signer from signature
    const recoveredSigner = await client.readContract({
      address: contractAddress,
      abi: CMVH_VERIFIER_ABI,
      functionName: "recoverSigner",
      args: [contractEmailHash as `0x${string}`, headers.signature as Hex],
    });
    console.log(`🔍 Contract recovered signer: ${recoveredSigner}`);
    console.log(`🔍 Expected signer: ${headers.address}`);

    // Call contract verifyEmail function (body excluded from signature)
    const isValid = await client.readContract({
      address: contractAddress,
      abi: CMVH_VERIFIER_ABI,
      functionName: "verifyEmail",
      args: [
        headers.address as Address,
        content.subject,
        content.from,
        content.to,
        headers.signature as Hex,
      ],
    });

    console.log(`📊 Contract returned: ${isValid}`);

    if (!isValid) {
      return {
        isValid: false,
        error: "Signature verification failed on-chain (contract returned false)"
      };
    }

    return { isValid: true };
  } catch (error) {
    console.error("❌ On-chain verification exception:", error);
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
