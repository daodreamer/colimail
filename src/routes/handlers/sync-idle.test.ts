import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import * as SyncIdle from './sync-idle';
import type { AccountConfig, EmailHeader, Folder, IdleEvent } from '../lib/types';

// Mock dependencies
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('../lib/state.svelte', () => ({
  state: {
    accounts: [],
    selectedAccountId: null,
    selectedFolderName: 'INBOX',
    folders: [],
    emails: [],
    selectedEmailUid: null,
    isSyncing: false,
    lastSyncTime: 0,
    error: null,
  },
}));

import { invoke } from '@tauri-apps/api/core';
import { state as appState } from '../lib/state.svelte';

describe('Sync and IDLE Event Handlers', () => {
  let mockAccounts: AccountConfig[];
  let mockFolders: Folder[];
  let mockEmails: EmailHeader[];

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();

    mockAccounts = [
      {
        id: 1,
        email: 'test@example.com',
        imap_server: 'imap.example.com',
        imap_port: 993,
        smtp_server: 'smtp.example.com',
        smtp_port: 587,
      },
    ];

    mockFolders = [
      { id: 1, account_id: 1, name: 'INBOX', display_name: 'Inbox' } as Folder,
      { id: 2, account_id: 1, name: 'Sent', display_name: 'Sent' } as Folder,
    ];

    mockEmails = [
      {
        uid: 100,
        subject: 'Test Email 1',
        from: 'sender@example.com',
        to: 'receiver@example.com',
        date: 'Mon, 15 Jan 2024 14:30:00 +0000',
        timestamp: 1705331400,
        seen: false,
        flagged: false,
        has_attachments: false,
      },
    ];

    // Reset app state
    appState.accounts = [...mockAccounts];
    appState.selectedAccountId = 1;
    appState.selectedFolderName = 'INBOX';
    appState.folders = [...mockFolders];
    appState.emails = [...mockEmails];
    appState.isSyncing = false;
    appState.error = null;
    appState.selectedEmailUid = null;
    appState.lastSyncTime = 0;
  });

  afterEach(() => {
    vi.useRealTimers();
    SyncIdle.clearAutoSyncTimer();
  });

  describe('startAutoSyncTimer', () => {
    it('should return null for zero or negative syncInterval', () => {
      const timer1 = SyncIdle.startAutoSyncTimer(0, mockAccounts, 1, 'INBOX');
      const timer2 = SyncIdle.startAutoSyncTimer(-1, mockAccounts, 1, 'INBOX');

      expect(timer1).toBeNull();
      expect(timer2).toBeNull();
    });

    it('should create timer for positive syncInterval', () => {
      const timer = SyncIdle.startAutoSyncTimer(300, mockAccounts, 1, 'INBOX');

      expect(timer).not.toBeNull();
      expect(typeof timer).toBe('object'); // setInterval returns NodeJS.Timeout
    });

    it('should clear existing timer when called again', () => {
      const timer1 = SyncIdle.startAutoSyncTimer(300, mockAccounts, 1, 'INBOX');
      const timer2 = SyncIdle.startAutoSyncTimer(600, mockAccounts, 1, 'INBOX');

      expect(timer1).not.toBe(timer2);
      expect(timer2).not.toBeNull();
    });

    it('should check if sync is needed before syncing', async () => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'should_sync') return Promise.resolve(false); // No sync needed
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      SyncIdle.startAutoSyncTimer(300, mockAccounts, 1, 'INBOX');

      // Advance timer by 1 minute
      await vi.advanceTimersByTimeAsync(60000);

      // Should check if sync is needed
      expect(invoke).toHaveBeenCalledWith('should_sync', {
        accountId: 1,
        folder: 'INBOX',
        syncInterval: 300,
      });

      // Should NOT perform sync (should_sync returned false)
      expect(invoke).not.toHaveBeenCalledWith('sync_folders', expect.any(Object));
      expect(invoke).not.toHaveBeenCalledWith('sync_emails', expect.any(Object));
    });

    it('should perform sync when needed', async () => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'should_sync') return Promise.resolve(true); // Sync needed
        if (cmd === 'sync_folders') return Promise.resolve(mockFolders);
        if (cmd === 'sync_emails') return Promise.resolve([...mockEmails, {
          uid: 101,
          subject: 'New Email',
          from: 'new@example.com',
          to: 'receiver@example.com',
          date: 'Mon, 16 Jan 2024 10:00:00 +0000',
          timestamp: 1705399200,
          seen: false,
          flagged: false,
          has_attachments: false,
        }]);
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      SyncIdle.startAutoSyncTimer(300, mockAccounts, 1, 'INBOX');

      // Advance timer by 1 minute
      await vi.advanceTimersByTimeAsync(60000);

      // Should perform full sync
      expect(invoke).toHaveBeenCalledWith('sync_folders', { config: mockAccounts[0] });
      expect(invoke).toHaveBeenCalledWith('sync_emails', {
        config: mockAccounts[0],
        folder: 'INBOX',
      });

      // Should update state
      expect(appState.emails.length).toBe(2); // Original + new email
      expect(appState.lastSyncTime).toBeGreaterThan(0);
      expect(appState.isSyncing).toBe(false);
    });

    it('should not sync if already syncing', async () => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'should_sync') return Promise.resolve(true);
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      appState.isSyncing = true; // Already syncing

      SyncIdle.startAutoSyncTimer(300, mockAccounts, 1, 'INBOX');

      // Advance timer by 1 minute
      await vi.advanceTimersByTimeAsync(60000);

      // Should check if sync is needed
      expect(invoke).toHaveBeenCalledWith('should_sync', expect.any(Object));

      // Should NOT perform sync (already syncing)
      expect(invoke).not.toHaveBeenCalledWith('sync_folders', expect.any(Object));
      expect(invoke).not.toHaveBeenCalledWith('sync_emails', expect.any(Object));
    });

    it('should handle sync errors gracefully', async () => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'should_sync') return Promise.resolve(true);
        if (cmd === 'sync_folders') return Promise.reject(new Error('IMAP timeout'));
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      SyncIdle.startAutoSyncTimer(300, mockAccounts, 1, 'INBOX');

      // Advance timer by 1 minute
      await vi.advanceTimersByTimeAsync(60000);

      // Should log error
      expect(consoleErrorSpy).toHaveBeenCalledWith('❌ Auto-sync failed:', expect.any(Error));

      // Should reset syncing state
      expect(appState.isSyncing).toBe(false);

      consoleErrorSpy.mockRestore();
    });
  });

  describe('handleManualRefresh', () => {
    it('should sync all accounts and folders', async () => {
      vi.mocked(invoke).mockImplementation((cmd: string, args?: any) => {
        if (cmd === 'sync_folders') return Promise.resolve(mockFolders);
        if (cmd === 'sync_emails') return Promise.resolve(mockEmails);
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      await SyncIdle.handleManualRefresh();

      // Should sync folders for each account
      expect(invoke).toHaveBeenCalledWith('sync_folders', { config: mockAccounts[0] });

      // Should sync emails for each folder (INBOX + Sent)
      expect(invoke).toHaveBeenCalledWith('sync_emails', {
        config: mockAccounts[0],
        folder: 'INBOX',
      });
      expect(invoke).toHaveBeenCalledWith('sync_emails', {
        config: mockAccounts[0],
        folder: 'Sent',
      });

      // Should update state
      expect(appState.folders).toEqual(mockFolders);
      expect(appState.emails).toEqual(mockEmails);
      expect(appState.lastSyncTime).toBeGreaterThan(0);
      expect(appState.isSyncing).toBe(false);
    });

    it('should handle missing accounts', async () => {
      appState.accounts = [];

      await SyncIdle.handleManualRefresh();

      expect(appState.error).toBe('No accounts configured.');
      expect(invoke).not.toHaveBeenCalled();
    });

    it('should continue syncing other accounts if one fails', async () => {
      const account2 = {
        id: 2,
        email: 'test2@example.com',
        imap_server: 'imap2.example.com',
        imap_port: 993,
        smtp_server: 'smtp2.example.com',
        smtp_port: 587,
      };
      appState.accounts = [mockAccounts[0], account2];

      vi.mocked(invoke).mockImplementation((cmd: string, args?: any) => {
        if (cmd === 'sync_folders' && args?.config?.id === 1) {
          return Promise.reject(new Error('Account 1 sync failed'));
        }
        if (cmd === 'sync_folders' && args?.config?.id === 2) {
          return Promise.resolve(mockFolders);
        }
        if (cmd === 'sync_emails') {
          return Promise.resolve(mockEmails);
        }
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      await SyncIdle.handleManualRefresh();

      // Should log error for account 1
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        expect.stringContaining('Failed to sync account test@example.com'),
        expect.any(Error)
      );

      // Should still complete (not throw)
      expect(appState.isSyncing).toBe(false);
      expect(appState.lastSyncTime).toBeGreaterThan(0);

      consoleErrorSpy.mockRestore();
    });

    it('should handle account-level errors gracefully', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Network error'));

      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      await SyncIdle.handleManualRefresh();

      // Should log error for failed account sync
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        expect.stringContaining('Failed to sync account'),
        expect.any(Error)
      );

      // Should complete without throwing (graceful degradation)
      expect(appState.isSyncing).toBe(false);
      expect(appState.lastSyncTime).toBeGreaterThan(0);

      consoleErrorSpy.mockRestore();
    });
  });

  describe('handleIdleEvent - NewMessages', () => {
    it('should sync current folder when NewMessages event received', async () => {
      const newEmails = [
        ...mockEmails,
        {
          uid: 102,
          subject: 'New Incoming Email',
          from: 'new@example.com',
          to: 'receiver@example.com',
          date: 'Mon, 16 Jan 2024 11:00:00 +0000',
          timestamp: 1705403400,
          seen: false,
          flagged: false,
          has_attachments: false,
        },
      ];

      vi.mocked(invoke).mockResolvedValue(newEmails);

      const idleEvent: IdleEvent = {
        account_id: 1,
        folder_name: 'INBOX',
        event_type: { type: 'NewMessages' },
      };

      const consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

      await SyncIdle.handleIdleEvent({ payload: idleEvent });

      // Should sync emails for current folder
      expect(invoke).toHaveBeenCalledWith('sync_emails', {
        config: mockAccounts[0],
        folder: 'INBOX',
      });

      // Should update emails
      expect(appState.emails.length).toBe(2);
      expect(appState.lastSyncTime).toBeGreaterThan(0);

      // Should log sync activity
      expect(consoleLogSpy).toHaveBeenCalledWith(
        expect.stringContaining('Starting sync for current view')
      );

      consoleLogSpy.mockRestore();
    });

    it('should background sync for non-current folder', async () => {
      vi.mocked(invoke).mockResolvedValue(mockEmails);

      const idleEvent: IdleEvent = {
        account_id: 1,
        folder_name: 'Sent', // Not current folder (current is INBOX)
        event_type: { type: 'NewMessages' },
      };

      const consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {});
      const initialEmails = [...appState.emails];

      await SyncIdle.handleIdleEvent({ payload: idleEvent });

      // Should still sync in background
      expect(invoke).toHaveBeenCalledWith('sync_emails', {
        config: mockAccounts[0],
        folder: 'Sent',
      });

      // Should NOT update UI (emails should remain unchanged)
      expect(appState.emails).toEqual(initialEmails);

      // Should log background sync
      expect(consoleLogSpy).toHaveBeenCalledWith(
        expect.stringContaining('Background sync for non-current folder: Sent')
      );

      consoleLogSpy.mockRestore();
    });

    it('should preserve selected email UID if it still exists', async () => {
      appState.selectedEmailUid = 100; // Select first email

      const newEmails = [
        ...mockEmails, // UID 100 still exists
        {
          uid: 102,
          subject: 'New Email',
          from: 'new@example.com',
          to: 'receiver@example.com',
          date: 'Mon, 16 Jan 2024 11:00:00 +0000',
          timestamp: 1705403400,
          seen: false,
          flagged: false,
          has_attachments: false,
        },
      ];

      vi.mocked(invoke).mockResolvedValue(newEmails);

      const idleEvent: IdleEvent = {
        account_id: 1,
        folder_name: 'INBOX',
        event_type: { type: 'NewMessages' },
      };

      await SyncIdle.handleIdleEvent({ payload: idleEvent });

      // Selected UID should be preserved
      expect(appState.selectedEmailUid).toBe(100);
      expect(appState.emails.find((e) => e.uid === 100)).toBeDefined();
    });

    it('should warn if selected email no longer exists after sync', async () => {
      appState.selectedEmailUid = 100;

      const newEmails = [
        {
          uid: 102,
          subject: 'New Email',
          from: 'new@example.com',
          to: 'receiver@example.com',
          date: 'Mon, 16 Jan 2024 11:00:00 +0000',
          timestamp: 1705403400,
          seen: false,
          flagged: false,
          has_attachments: false,
        },
      ]; // UID 100 is gone!

      vi.mocked(invoke).mockResolvedValue(newEmails);

      const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      const idleEvent: IdleEvent = {
        account_id: 1,
        folder_name: 'INBOX',
        event_type: { type: 'NewMessages' },
      };

      await SyncIdle.handleIdleEvent({ payload: idleEvent });

      // Should warn about missing email
      expect(consoleWarnSpy).toHaveBeenCalledWith(
        expect.stringContaining('Selected email UID 100 no longer in list')
      );

      consoleWarnSpy.mockRestore();
    });

    it('should discard sync result if account/folder changed during sync', async () => {
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'sync_emails') {
          // Simulate user switching folder during sync
          appState.selectedFolderName = 'Sent';
          return Promise.resolve(mockEmails);
        }
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const initialEmails = [...appState.emails];
      const consoleLogSpy = vi.spyOn(console, 'log').mockImplementation(() => {});

      const idleEvent: IdleEvent = {
        account_id: 1,
        folder_name: 'INBOX',
        event_type: { type: 'NewMessages' },
      };

      await SyncIdle.handleIdleEvent({ payload: idleEvent });

      // Should discard result (folder changed to Sent)
      expect(appState.emails).toEqual(initialEmails); // Unchanged

      // Should log warning
      expect(consoleLogSpy).toHaveBeenCalledWith(
        expect.stringContaining('Account/folder changed during IDLE sync, discarding result')
      );

      consoleLogSpy.mockRestore();
    });
  });

  describe('handleIdleEvent - FlagsChanged', () => {
    it('should sync specific email flags', async () => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'sync_specific_email_flags') return Promise.resolve();
        if (cmd === 'load_emails_from_cache') {
          const updated = [...mockEmails];
          updated[0].seen = true; // Flag changed
          return Promise.resolve(updated);
        }
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const idleEvent: IdleEvent = {
        account_id: 1,
        folder_name: 'INBOX',
        event_type: { type: 'FlagsChanged', uid: 100 },
      };

      await SyncIdle.handleIdleEvent({ payload: idleEvent });

      // Should sync specific flags
      expect(invoke).toHaveBeenCalledWith('sync_specific_email_flags', {
        accountId: 1,
        folderName: 'INBOX',
        uid: 100,
        config: mockAccounts[0],
      });

      // Should reload from cache
      expect(invoke).toHaveBeenCalledWith('load_emails_from_cache', {
        accountId: 1,
        folder: 'INBOX',
      });

      // Should update state with new flags
      expect(appState.emails[0].seen).toBe(true);
    });
  });

  describe('handleIdleEvent - Expunge', () => {
    it('should perform full sync on Expunge event', async () => {
      const reducedEmails: EmailHeader[] = []; // All emails deleted

      vi.mocked(invoke).mockResolvedValue(reducedEmails);

      const idleEvent: IdleEvent = {
        account_id: 1,
        folder_name: 'INBOX',
        event_type: { type: 'Expunge' },
      };

      await SyncIdle.handleIdleEvent({ payload: idleEvent });

      // Should perform full sync
      expect(invoke).toHaveBeenCalledWith('sync_emails', {
        config: mockAccounts[0],
        folder: 'INBOX',
      });

      // Should update state
      expect(appState.emails).toEqual(reducedEmails);
    });
  });

  describe('handleIdleEvent - ConnectionLost', () => {
    it('should log warning on connection lost', async () => {
      const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      const idleEvent: IdleEvent = {
        account_id: 1,
        folder_name: 'INBOX',
        event_type: { type: 'ConnectionLost' },
      };

      await SyncIdle.handleIdleEvent({ payload: idleEvent });

      expect(consoleWarnSpy).toHaveBeenCalledWith(
        expect.stringContaining('IDLE connection lost for account 1')
      );

      consoleWarnSpy.mockRestore();
    });
  });

  describe('clearAutoSyncTimer', () => {
    it('should clear existing timer', () => {
      const timer = SyncIdle.startAutoSyncTimer(300, mockAccounts, 1, 'INBOX');
      expect(timer).not.toBeNull();

      SyncIdle.clearAutoSyncTimer();

      const clearedTimer = SyncIdle.getAutoSyncTimer();
      expect(clearedTimer).toBeNull();
    });

    it('should be safe to call multiple times', () => {
      SyncIdle.clearAutoSyncTimer();
      SyncIdle.clearAutoSyncTimer();
      SyncIdle.clearAutoSyncTimer();

      // Should not throw
      expect(SyncIdle.getAutoSyncTimer()).toBeNull();
    });
  });
});
