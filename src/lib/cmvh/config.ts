// CMVH Configuration Management

import { invoke } from "@tauri-apps/api/core";
import type { CMVHConfig } from "./types";
import { DEFAULT_CMVH_CONFIG, CMVH_CONFIG_VERSION } from "./types";

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
