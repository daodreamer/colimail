// CMVH Configuration Management

import { invoke } from "@tauri-apps/api/core";
import type { CMVHConfig } from "./types";
import { DEFAULT_CMVH_CONFIG, CMVH_CONFIG_VERSION, NETWORK_CONFIG } from "./types";
import { createCMVHClient } from "./blockchain";
import type { Address, Hex } from "viem";

const STORAGE_KEY = "cmvh_config";

// In-memory cache to avoid frequent file reads
let configCache: CMVHConfig | null = null;

/**
 * Migrate old configuration to new version
 */
async function migrateConfig(oldConfig: Partial<CMVHConfig>): Promise<CMVHConfig> {
  const configVersion = oldConfig.version || 1;

  // If config is outdated, update contract address
  if (configVersion < CMVH_CONFIG_VERSION) {
    console.log(`🔄 Migrating CMVH config from v${configVersion} to v${CMVH_CONFIG_VERSION}`);

    // Reset contract address to new default
    const migratedConfig = {
      ...DEFAULT_CMVH_CONFIG,
      ...oldConfig,
      version: CMVH_CONFIG_VERSION,
      contractAddress: DEFAULT_CMVH_CONFIG.contractAddress, // Force update
    };

    // Save migrated config
    await saveConfig(migratedConfig);
    return migratedConfig;
  }

  return { ...DEFAULT_CMVH_CONFIG, ...oldConfig };
}

/**
 * Load CMVH configuration from Tauri secure storage (synchronous wrapper)
 */
export function loadConfig(): CMVHConfig {
  // Return cached config if available
  if (configCache) {
    return configCache;
  }

  // Return default config (async load will update cache)
  loadConfigAsync().then((config) => {
    configCache = config;
  });

  return DEFAULT_CMVH_CONFIG;
}

/**
 * Load CMVH configuration asynchronously from Tauri secure storage
 */
export async function loadConfigAsync(): Promise<CMVHConfig> {
  try {
    const stored = await invoke<string>("get_secure_storage", { key: STORAGE_KEY });
    if (stored) {
      const oldConfig = JSON.parse(stored) as Partial<CMVHConfig>;
      const config = await migrateConfig(oldConfig);
      configCache = config;
      return config;
    }
  } catch (error) {
    console.log("CMVH config not found in secure storage, using defaults");
  }

  configCache = DEFAULT_CMVH_CONFIG;
  return DEFAULT_CMVH_CONFIG;
}

/**
 * Save CMVH configuration to Tauri secure storage
 */
export async function saveConfig(config: CMVHConfig): Promise<void> {
  try {
    await invoke("set_secure_storage", {
      key: STORAGE_KEY,
      value: JSON.stringify(config),
    });
    configCache = config;
    console.log("✅ CMVH config saved to secure storage");
  } catch (error) {
    console.error("Failed to save CMVH config:", error);
    throw error;
  }
}

/**
 * Reset CMVH configuration to defaults
 */
export async function resetConfig(): Promise<CMVHConfig> {
  await saveConfig(DEFAULT_CMVH_CONFIG);
  return DEFAULT_CMVH_CONFIG;
}

/**
 * Clear config cache (useful for testing)
 */
export function clearConfigCache(): void {
  configCache = null;
}

/**
 * Validate that the CMVH contract is properly deployed
 * This checks:
 * 1. Contract bytecode exists at the address
 * 2. Contract responds to function calls correctly
 */
export async function validateContractDeployment(config: CMVHConfig): Promise<{
  isValid: boolean;
  error?: string;
  explorerUrl?: string;
}> {
  const networkConfig = NETWORK_CONFIG[config.network];
  const contractAddressStr = config.contractAddress || networkConfig.contractAddress;

  // Skip validation if no contract address configured
  if (!contractAddressStr || contractAddressStr === "") {
    return {
      isValid: false,
      error: `No contract address configured for ${config.network}`,
    };
  }

  const contractAddress = contractAddressStr as Address;

  console.log(`🔍 Validating CMVHVerifier contract deployment on ${networkConfig.name}...`);
  console.log(`   Contract address: ${contractAddress}`);
  console.log(`   Network: ${networkConfig.name} (chainId: ${networkConfig.chainId})`);
  console.log(`   RPC URL: ${config.rpcUrl || networkConfig.rpcUrl}`);

  try {
    const client = createCMVHClient(config);

    // 1. Check if contract bytecode exists
    const bytecode = await client.getBytecode({
      address: contractAddress,
    });

    if (!bytecode || bytecode === "0x") {
      const error = `Contract not deployed at ${contractAddress} on ${networkConfig.name}`;
      console.error(`❌ ${error}`);
      return {
        isValid: false,
        error,
        explorerUrl: `${networkConfig.explorerUrl}/address/${contractAddress}`,
      };
    }

    console.log(`✅ Contract bytecode found (${bytecode.length} bytes)`);

    // 2. Test contract functionality with a simple call
    // Use hashEmail function as it's a pure function that should always work
    const testHash = await client.readContract({
      address: contractAddress,
      abi: [
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
      ],
      functionName: "hashEmail",
      args: ["test", "test@example.com", "test@example.com"],
    });

    console.log(`✅ Contract function call successful (hash: ${testHash})`);

    // 3. Test signature verification with dummy data
    const dummyHash = "0x0000000000000000000000000000000000000000000000000000000000000000" as Hex;
    const dummySig = "0x" + "00".repeat(65) as Hex;
    const dummyAddress = "0x0000000000000000000000000000000000000000" as Address;

    await client.readContract({
      address: contractAddress,
      abi: [
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
      ],
      functionName: "verifySignature",
      args: [dummyAddress, dummyHash, dummySig],
    });

    console.log("✅ Contract signature verification function accessible");
    console.log(`✅ CMVHVerifier contract validation passed on ${networkConfig.name}`);
    console.log(`   Explorer: ${networkConfig.explorerUrl}/address/${contractAddress}`);

    return {
      isValid: true,
      explorerUrl: `${networkConfig.explorerUrl}/address/${contractAddress}`,
    };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error(`❌ Contract validation failed: ${errorMessage}`);
    return {
      isValid: false,
      error: `Contract validation failed: ${errorMessage}`,
      explorerUrl: `${networkConfig.explorerUrl}/address/${contractAddress}`,
    };
  }
}
