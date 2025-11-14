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

// CMVH Error types for fine-grained error handling
// These match the Rust CMVHError enum variants
export type CMVHError =
  | {
      type: "InvalidPrivateKey";
      message: string;
    }
  | {
      type: "SigningFailed";
      message: string;
    }
  | {
      type: "SMTPConnectionFailed";
      server: string;
      port: number;
      message: string;
    }
  | {
      type: "SMTPAuthFailed";
      message: string;
    }
  | {
      type: "NetworkTimeout";
      duration_secs: number;
    }
  | {
      type: "RateLimited";
      retry_after_secs: number;
    }
  | {
      type: "InvalidEmailAddress";
      address: string;
      message: string;
    }
  | {
      type: "EmailBuildFailed";
      message: string;
    }
  | {
      type: "InvalidAttachment";
      filename: string;
      message: string;
    }
  | {
      type: "TokenError";
      message: string;
    }
  | {
      type: "Unknown";
      message: string;
    };

/**
 * Type guard to check if an error is a CMVHError
 */
export function isCMVHError(error: unknown): error is CMVHError {
  return (
    typeof error === "object" &&
    error !== null &&
    "type" in error &&
    typeof (error as any).type === "string"
  );
}

/**
 * Get user-friendly error message from CMVHError
 */
export function getCMVHErrorMessage(error: CMVHError): string {
  switch (error.type) {
    case "InvalidPrivateKey":
      return `Invalid private key: ${error.message}`;
    case "SigningFailed":
      return `Failed to sign email: ${error.message}`;
    case "SMTPConnectionFailed":
      return `Failed to connect to SMTP server ${error.server}:${error.port} - ${error.message}`;
    case "SMTPAuthFailed":
      return `SMTP authentication failed: ${error.message}`;
    case "NetworkTimeout":
      return `Network timeout after ${error.duration_secs} seconds`;
    case "RateLimited":
      return `Rate limited. Please retry after ${error.retry_after_secs} seconds`;
    case "InvalidEmailAddress":
      return `Invalid email address "${error.address}": ${error.message}`;
    case "EmailBuildFailed":
      return `Failed to build email: ${error.message}`;
    case "InvalidAttachment":
      return `Invalid attachment "${error.filename}": ${error.message}`;
    case "TokenError":
      return `Authentication token error: ${error.message}`;
    case "Unknown":
      return `Error: ${error.message}`;
  }
}

/**
 * Determine if error is retriable
 */
export function isRetriableError(error: CMVHError): boolean {
  return error.type === "NetworkTimeout" || error.type === "RateLimited";
}

/**
 * Get retry delay in seconds for retriable errors
 */
export function getRetryDelay(error: CMVHError): number | null {
  switch (error.type) {
    case "NetworkTimeout":
      return 5; // Retry after 5 seconds
    case "RateLimited":
      return error.retry_after_secs;
    default:
      return null;
  }
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
  // Onboarding
  hasSeenOnboarding?: boolean; // Whether user has seen the onboarding guide
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
  hasSeenOnboarding: false,
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
