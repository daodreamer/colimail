// CMVH Types - TypeScript interfaces matching Rust types

export interface CMVHHeaders {
  version: string;
  address: string;
  chain: string;
  timestamp: string;
  hash_algo: string;
  signature: string;
  ens?: string;
  reward?: string;
  proof_url?: string;
}

export interface EmailContent {
  subject: string;
  from: string;
  to: string;
  body: string;
}

export interface VerificationResult {
  is_valid: boolean;
  signer_address?: string;
  ens_name?: string;
  timestamp?: string;
  chain?: string;
  error?: string;
}

export type VerificationStatus =
  | "idle"
  | "verifying"
  | "verified-local"
  | "verified-onchain"
  | "invalid"
  | "error";

export interface VerificationState {
  status: VerificationStatus;
  result?: VerificationResult;
  error?: string;
}

// CMVH Configuration
export interface CMVHConfig {
  version?: number; // Config version for migration
  enabled: boolean;
  autoVerify: boolean;
  verifyOnChain: boolean;
  rpcUrl: string;
  network: "arbitrum" | "arbitrum-sepolia";
  contractAddress: string;
  // Signing configuration
  enableSigning: boolean;
  privateKey: string; // Hex-encoded private key (without 0x prefix)
  derivedAddress: string; // Ethereum address derived from private key
}

// Configuration version for migration
export const CMVH_CONFIG_VERSION = 2; // Updated for pure functions contract

// Default configuration
export const DEFAULT_CMVH_CONFIG: CMVHConfig = {
  version: CMVH_CONFIG_VERSION,
  enabled: true,
  autoVerify: true,
  verifyOnChain: false,
  rpcUrl: "https://sepolia-rollup.arbitrum.io/rpc",
  network: "arbitrum-sepolia",
  contractAddress: "0xc4BAD26e321A8D0FE3bA3337Fc3846c25506308a", // Deployed contract (pure functions)
  enableSigning: false,
  privateKey: "",
  derivedAddress: "",
};

// Network configurations
export const NETWORK_CONFIG = {
  "arbitrum-sepolia": {
    chainId: 421614,
    name: "Arbitrum Sepolia",
    rpcUrl: "https://sepolia-rollup.arbitrum.io/rpc",
    contractAddress: "0xc4BAD26e321A8D0FE3bA3337Fc3846c25506308a", // Verified: CMVHVerifier (pure functions)
    explorerUrl: "https://sepolia.arbiscan.io",
    // Contract verified at: https://sepolia.arbiscan.io/address/0xc4BAD26e321A8D0FE3bA3337Fc3846c25506308a
  },
  arbitrum: {
    chainId: 42161,
    name: "Arbitrum One",
    rpcUrl: "https://arb1.arbitrum.io/rpc",
    contractAddress: "", // Not deployed yet - use arbitrum-sepolia for testing
    explorerUrl: "https://arbiscan.io",
  },
} as const;
