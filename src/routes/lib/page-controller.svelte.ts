import { invoke } from "@tauri-apps/api/core";
import type { AccountConfig } from "./types";
import { state as appState } from "./state.svelte";
import * as SyncIdle from "../handlers/sync-idle";

/**
 * Page Controller for main application initialization and lifecycle management
 * Separates page-level orchestration logic from UI components
 */

/**
 * Re-export WalletSession interface from global.d.ts
 * (Commented out to use global definition instead)
 */
// The WalletSession interface is defined globally in src/global.d.ts

/**
 * IDLE connection failure information
 */
export interface IdleConnectionFailure {
  email: string;
  error: string;
}

/**
 * Auto-sync timer reference
 */
let autoSyncTimer: ReturnType<typeof setInterval> | null = null;

/**
 * Load application data and start services
 * @param handleAccountClick - Callback to handle account selection
 * @returns Array of IDLE connection failures (empty if all succeeded)
 */
export async function loadApp(
  handleAccountClick: (accountId: number) => Promise<void>
): Promise<IdleConnectionFailure[]> {
  // Load accounts and sync interval from backend
  appState.accounts = await invoke<AccountConfig[]>("load_account_configs");
  appState.syncInterval = await invoke<number>("get_sync_interval");

  // Auto-select first account if available and none is selected
  if (appState.accounts.length > 0 && !appState.selectedAccountId) {
    await handleAccountClick(appState.accounts[0].id);
  }

  // Start auto-sync timer
  startAutoSyncTimer();

  // Start IDLE connections for all accounts and collect failures
  const failures = await startIdleConnections();
  return failures;
}

/**
 * Start IDLE connections for all accounts
 * @returns Array of connection failures
 */
async function startIdleConnections(): Promise<IdleConnectionFailure[]> {
  const failures: IdleConnectionFailure[] = [];

  for (const account of appState.accounts) {
    try {
      await invoke("start_idle", {
        accountId: account.id,
        folderName: "INBOX",
        config: account,
      });
      console.log(`✅ IDLE started for ${account.email}`);
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      console.error(`❌ Failed to start IDLE for account ${account.email}:`, errorMessage);

      // Collect failure information for user notification
      failures.push({
        email: account.email,
        error: errorMessage,
      });
    }
  }

  return failures;
}

/**
 * Start auto-sync timer for periodic email synchronization
 */
function startAutoSyncTimer(): void {
  // Clear existing timer if any
  if (autoSyncTimer) {
    clearInterval(autoSyncTimer);
  }

  autoSyncTimer = SyncIdle.startAutoSyncTimer(
    appState.syncInterval,
    appState.accounts,
    appState.selectedAccountId,
    appState.selectedFolderName
  );

  console.log(`⏰ Auto-sync timer started with interval: ${appState.syncInterval}s`);
}

/**
 * Stop auto-sync timer
 */
export function stopAutoSyncTimer(): void {
  if (autoSyncTimer) {
    clearInterval(autoSyncTimer);
    autoSyncTimer = null;
    console.log("⏰ Auto-sync timer stopped");
  }
}

/**
 * Initialize application with wallet session check
 * @param handleAccountClick - Callback to handle account selection
 * @returns Object with wallet session (if found) and IDLE connection failures
 */
export async function initializeApp(handleAccountClick: (accountId: number) => Promise<void>): Promise<{
  walletSession: WalletSession | null;
  idleFailures: IdleConnectionFailure[];
}> {
  try {
    // Check for saved wallet session (after encryption is unlocked)
    const savedSession = await invoke<WalletSession | null>("get_wallet_session");

    if (savedSession) {
      console.log("🔐 Found saved wallet session:", savedSession);
      // Return session for confirmation dialog, no IDLE failures yet
      return {
        walletSession: savedSession,
        idleFailures: [],
      };
    }

    // No wallet session, proceed with app loading
    const failures = await loadApp(handleAccountClick);
    return {
      walletSession: null,
      idleFailures: failures,
    };
  } catch (e) {
    appState.error = `Failed to initialize app: ${e}`;
    throw e;
  }
}

/**
 * Reload application data (useful after account/folder changes)
 * @param handleAccountClick - Callback to handle account selection
 */
export async function reloadApp(handleAccountClick: (accountId: number) => Promise<void>): Promise<void> {
  console.log("🔄 Reloading application...");
  await loadApp(handleAccountClick);
}

/**
 * Update sync interval and restart timer
 * @param newInterval - New sync interval in seconds
 */
export async function updateSyncInterval(newInterval: number): Promise<void> {
  appState.syncInterval = newInterval;
  await invoke("set_sync_interval", { interval: newInterval });
  startAutoSyncTimer();
  console.log(`✅ Sync interval updated to ${newInterval}s`);
}

/**
 * Cleanup function to be called on component unmount
 */
export function cleanup(): void {
  stopAutoSyncTimer();
  console.log("🧹 Page controller cleanup completed");
}
