// CMVH Blockchain Verifier - On-chain signature verification using viem

import { createPublicClient, http, type Address, type Hex } from "viem";
import { arbitrum, arbitrumSepolia } from "viem/chains";
import type { CMVHHeaders, EmailContent, CMVHConfig } from "./types";
import { NETWORK_CONFIG } from "./types";

// CMVHVerifier contract ABI (UUPS Proxy v2.0.0 with EIP-712 and timestamp)
// Split into individual ABIs for better type safety with viem
const VERIFY_EMAIL_ABI = [
  {
    name: "verifyEmail",
    type: "function",
    stateMutability: "nonpayable",
    inputs: [
      { name: "signer", type: "address" },
      { name: "subject", type: "string" },
      { name: "from", type: "string" },
      { name: "to", type: "string" },
      { name: "timestamp", type: "uint256" },
      { name: "signature", type: "bytes" },
    ],
    outputs: [{ name: "isValid", type: "bool" }],
  },
] as const;

const GET_EMAIL_STRUCT_HASH_ABI = [
  {
    name: "getEmailStructHash",
    type: "function",
    stateMutability: "pure",
    inputs: [
      { name: "subject", type: "string" },
      { name: "from", type: "string" },
      { name: "to", type: "string" },
      { name: "timestamp", type: "uint256" },
    ],
    outputs: [{ name: "structHash", type: "bytes32" }],
  },
] as const;

const GET_DOMAIN_SEPARATOR_ABI = [
  {
    name: "getDomainSeparator",
    type: "function",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "bytes32" }],
  },
] as const;

const RECOVER_SIGNER_ABI = [
  {
    name: "recoverSigner",
    type: "function",
    stateMutability: "pure",
    inputs: [
      { name: "digest", type: "bytes32" },
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
 * Verify signature on-chain (with EIP-712 and timestamp support)
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

    // Parse timestamp from headers (Unix timestamp in seconds)
    const timestamp = BigInt(headers.timestamp);

    console.log("📋 On-chain verification parameters:");
    console.log(`   Contract: ${contractAddress}`);
    console.log(`   Signer: ${headers.address}`);
    console.log(`   Subject: "${content.subject}"`);
    console.log(`   From: "${content.from}"`);
    console.log(`   To: "${content.to}"`);
    console.log(`   Timestamp: ${headers.timestamp} (${new Date(Number(timestamp) * 1000).toISOString()})`);
    console.log(`   Signature length: ${headers.signature.length} chars (expected: 132 for 0x + 65 bytes)`);
    console.log(`   Signature: ${headers.signature}`);

    // Test: Call contract's getEmailStructHash to compute EIP-712 struct hash
    const contractStructHash = await client.readContract({
      address: contractAddress,
      abi: GET_EMAIL_STRUCT_HASH_ABI,
      functionName: "getEmailStructHash",
      args: [content.subject, content.from, content.to, timestamp],
    });
    console.log(`📊 Contract computed EIP-712 struct hash: ${contractStructHash}`);

    // Test: Get domain separator
    const domainSeparator = await client.readContract({
      address: contractAddress,
      abi: GET_DOMAIN_SEPARATOR_ABI,
      functionName: "getDomainSeparator",
      args: [],
    });
    console.log(`📊 Contract domain separator: ${domainSeparator}`);

    // Test: Try to recover signer from signature using digest
    const recoveredSigner = await client.readContract({
      address: contractAddress,
      abi: RECOVER_SIGNER_ABI,
      functionName: "recoverSigner",
      args: [contractStructHash as `0x${string}`, headers.signature as Hex],
    });
    console.log(`🔍 Contract recovered signer: ${recoveredSigner}`);
    console.log(`🔍 Expected signer: ${headers.address}`);

    // Call contract verifyEmail function with timestamp (EIP-712 compliance)
    const isValid = await client.readContract({
      address: contractAddress,
      abi: VERIFY_EMAIL_ABI,
      functionName: "verifyEmail",
      args: [
        headers.address as Address,
        content.subject,
        content.from,
        content.to,
        timestamp,
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
