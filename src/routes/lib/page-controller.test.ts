import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import * as PageController from './page-controller.svelte';

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock state module
vi.mock('./state.svelte', () => ({
  state: {
    accounts: [],
    syncInterval: 300,
    selectedAccountId: null,
    selectedFolderName: null,
    error: null,
  },
}));

// Mock sync-idle module
vi.mock('../handlers/sync-idle', () => ({
  startAutoSyncTimer: vi.fn(() => 123 as unknown as ReturnType<typeof setInterval>),
}));

import { invoke } from '@tauri-apps/api/core';
import { state as appState } from './state.svelte';
import * as SyncIdle from '../handlers/sync-idle';

describe('PageController', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset app state
    appState.accounts = [];
    appState.syncInterval = 300;
    appState.selectedAccountId = null;
    appState.selectedFolderName = 'INBOX';
    appState.error = null;
  });

  afterEach(() => {
    PageController.cleanup();
  });

  describe('loadApp', () => {
    it('should load accounts and sync interval from backend', async () => {
      const mockAccounts = [
        {
          id: 1,
          email: 'test@example.com',
          imap_server: 'imap.example.com',
          imap_port: 993,
          smtp_server: 'smtp.example.com',
          smtp_port: 587,
        },
      ];

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'load_account_configs') {
          return Promise.resolve(mockAccounts);
        }
        if (cmd === 'get_sync_interval') {
          return Promise.resolve(600);
        }
        if (cmd === 'start_idle') {
          return Promise.resolve();
        }
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const handleAccountClick = vi.fn();

      const failures = await PageController.loadApp(handleAccountClick);

      expect(invoke).toHaveBeenCalledWith('load_account_configs');
      expect(invoke).toHaveBeenCalledWith('get_sync_interval');
      expect(appState.accounts).toEqual(mockAccounts);
      expect(appState.syncInterval).toBe(600);
      expect(failures).toEqual([]); // No failures
    });

    it('should auto-select first account if none selected', async () => {
      const mockAccounts = [
        {
          id: 1,
          email: 'test@example.com',
          imap_server: 'imap.example.com',
          imap_port: 993,
          smtp_server: 'smtp.example.com',
          smtp_port: 587,
        },
      ];

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'load_account_configs') return Promise.resolve(mockAccounts);
        if (cmd === 'get_sync_interval') return Promise.resolve(300);
        if (cmd === 'start_idle') return Promise.resolve();
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const handleAccountClick = vi.fn().mockResolvedValue(undefined);

      await PageController.loadApp(handleAccountClick);

      expect(handleAccountClick).toHaveBeenCalledWith(1);
    });

    it('should not auto-select if account already selected', async () => {
      const mockAccounts = [
        {
          id: 1,
          email: 'test@example.com',
          imap_server: 'imap.example.com',
          imap_port: 993,
          smtp_server: 'smtp.example.com',
          smtp_port: 587,
        },
      ];

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'load_account_configs') return Promise.resolve(mockAccounts);
        if (cmd === 'get_sync_interval') return Promise.resolve(300);
        if (cmd === 'start_idle') return Promise.resolve();
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      appState.selectedAccountId = 1; // Already selected
      const handleAccountClick = vi.fn();

      await PageController.loadApp(handleAccountClick);

      expect(handleAccountClick).not.toHaveBeenCalled();
    });

    it('should start auto-sync timer', async () => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'load_account_configs') return Promise.resolve([]);
        if (cmd === 'get_sync_interval') return Promise.resolve(300);
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const handleAccountClick = vi.fn();

      await PageController.loadApp(handleAccountClick);

      expect(SyncIdle.startAutoSyncTimer).toHaveBeenCalled();
    });

    it('should start IDLE connections for all accounts', async () => {
      const mockAccounts = [
        {
          id: 1,
          email: 'test1@example.com',
          imap_server: 'imap.example.com',
          imap_port: 993,
          smtp_server: 'smtp.example.com',
          smtp_port: 587,
        },
        {
          id: 2,
          email: 'test2@example.com',
          imap_server: 'imap.example.com',
          imap_port: 993,
          smtp_server: 'smtp.example.com',
          smtp_port: 587,
        },
      ];

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'load_account_configs') return Promise.resolve(mockAccounts);
        if (cmd === 'get_sync_interval') return Promise.resolve(300);
        if (cmd === 'start_idle') return Promise.resolve();
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const handleAccountClick = vi.fn();

      await PageController.loadApp(handleAccountClick);

      // Should call start_idle for each account
      expect(invoke).toHaveBeenCalledWith('start_idle', {
        accountId: 1,
        folderName: 'INBOX',
        config: mockAccounts[0],
      });
      expect(invoke).toHaveBeenCalledWith('start_idle', {
        accountId: 2,
        folderName: 'INBOX',
        config: mockAccounts[1],
      });
    });

    it('should handle IDLE connection errors gracefully and return failures', async () => {
      const mockAccounts = [
        {
          id: 1,
          email: 'test@example.com',
          imap_server: 'imap.example.com',
          imap_port: 993,
          smtp_server: 'smtp.example.com',
          smtp_port: 587,
        },
      ];

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'load_account_configs') return Promise.resolve(mockAccounts);
        if (cmd === 'get_sync_interval') return Promise.resolve(300);
        if (cmd === 'start_idle') return Promise.reject(new Error('IDLE connection failed'));
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      const handleAccountClick = vi.fn();

      const failures = await PageController.loadApp(handleAccountClick);

      // Should not throw error but return failure information
      expect(consoleErrorSpy).toHaveBeenCalled();
      expect(failures).toHaveLength(1);
      expect(failures[0]).toEqual({
        email: 'test@example.com',
        error: 'IDLE connection failed',
      });
      consoleErrorSpy.mockRestore();
    });
  });

  describe('initializeApp', () => {
    it('should return wallet session if found', async () => {
      const mockSession: WalletSession = {
        address: '0x1234567890123456789012345678901234567890',
        chain_id: 1,
        connection_method: 'walletconnect',
        last_active_timestamp: Date.now(),
      };

      vi.mocked(invoke).mockResolvedValue(mockSession);

      const handleAccountClick = vi.fn();
      const result = await PageController.initializeApp(handleAccountClick);

      expect(result.walletSession).toEqual(mockSession);
      expect(result.idleFailures).toEqual([]);
      expect(invoke).toHaveBeenCalledWith('get_wallet_session');
    });

    it('should load app and return null wallet session if none found', async () => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'get_wallet_session') return Promise.resolve(null);
        if (cmd === 'load_account_configs') return Promise.resolve([]);
        if (cmd === 'get_sync_interval') return Promise.resolve(300);
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const handleAccountClick = vi.fn();
      const result = await PageController.initializeApp(handleAccountClick);

      expect(result.walletSession).toBeNull();
      expect(result.idleFailures).toEqual([]);
      expect(invoke).toHaveBeenCalledWith('load_account_configs');
    });

    it('should return IDLE failures when connections fail', async () => {
      const mockAccounts = [{ id: 1, email: 'test@example.com', imap_server: 'imap.example.com', imap_port: 993, smtp_server: 'smtp.example.com', smtp_port: 587 }];

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'get_wallet_session') return Promise.resolve(null);
        if (cmd === 'load_account_configs') return Promise.resolve(mockAccounts);
        if (cmd === 'get_sync_interval') return Promise.resolve(300);
        if (cmd === 'start_idle') return Promise.reject(new Error('IDLE failed'));
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      const handleAccountClick = vi.fn();
      const result = await PageController.initializeApp(handleAccountClick);

      expect(result.walletSession).toBeNull();
      expect(result.idleFailures).toHaveLength(1);
      expect(result.idleFailures[0].email).toBe('test@example.com');
      consoleErrorSpy.mockRestore();
    });

    it('should set error state on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Database error'));

      const handleAccountClick = vi.fn();

      await expect(PageController.initializeApp(handleAccountClick)).rejects.toThrow();
      expect(appState.error).toContain('Failed to initialize app');
    });
  });

  describe('updateSyncInterval', () => {
    it('should update sync interval and restart timer', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await PageController.updateSyncInterval(600);

      expect(appState.syncInterval).toBe(600);
      expect(invoke).toHaveBeenCalledWith('set_sync_interval', { interval: 600 });
      expect(SyncIdle.startAutoSyncTimer).toHaveBeenCalled();
    });
  });

  describe('cleanup', () => {
    it('should stop auto-sync timer', () => {
      const consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

      PageController.cleanup();

      expect(consoleLogSpy).toHaveBeenCalledWith(
        expect.stringContaining('Page controller cleanup completed')
      );
      consoleLogSpy.mockRestore();
    });
  });

  describe('stopAutoSyncTimer', () => {
    it('should be callable without error', () => {
      // stopAutoSyncTimer should be safe to call even if no timer exists
      expect(() => PageController.stopAutoSyncTimer()).not.toThrow();
    });
  });
});
