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
  enabled: boolean;
  autoVerify: boolean;
  verifyOnChain: boolean;
  rpcUrl: string;
  network: "arbitrum" | "arbitrum-sepolia";
  contractAddress: string;
}

// Default configuration
export const DEFAULT_CMVH_CONFIG: CMVHConfig = {
  enabled: true,
  autoVerify: true,
  verifyOnChain: false,
  rpcUrl: "https://sepolia-rollup.arbitrum.io/rpc",
  network: "arbitrum-sepolia",
  contractAddress: "0xf251c131d6b9f71992e2ba43023d3b52588dbd02", // Deployed contract
};

// Network configurations
export const NETWORK_CONFIG = {
  "arbitrum-sepolia": {
    chainId: 421614,
    name: "Arbitrum Sepolia",
    rpcUrl: "https://sepolia-rollup.arbitrum.io/rpc",
    contractAddress: "0xf251c131d6b9f71992e2ba43023d3b52588dbd02",
    explorerUrl: "https://sepolia.arbiscan.io",
  },
  arbitrum: {
    chainId: 42161,
    name: "Arbitrum One",
    rpcUrl: "https://arb1.arbitrum.io/rpc",
    contractAddress: "", // TODO: Deploy to mainnet
    explorerUrl: "https://arbiscan.io",
  },
} as const;
