// CMVH Configuration Management

import type { CMVHConfig } from "./types";
import { DEFAULT_CMVH_CONFIG } from "./types";

const STORAGE_KEY = "cmvh_config";

/**
 * Load CMVH configuration from localStorage
 */
export function loadConfig(): CMVHConfig {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      return { ...DEFAULT_CMVH_CONFIG, ...JSON.parse(stored) };
    }
  } catch (error) {
    console.error("Failed to load CMVH config:", error);
  }
  return DEFAULT_CMVH_CONFIG;
}

/**
 * Save CMVH configuration to localStorage
 */
export function saveConfig(config: CMVHConfig): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
  } catch (error) {
    console.error("Failed to save CMVH config:", error);
  }
}

/**
 * Reset CMVH configuration to defaults
 */
export function resetConfig(): CMVHConfig {
  saveConfig(DEFAULT_CMVH_CONFIG);
  return DEFAULT_CMVH_CONFIG;
}
