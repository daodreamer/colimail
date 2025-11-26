import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import * as EmailOperations from './email-operations';
import type { AccountConfig, EmailHeader } from '../lib/types';

// Mock dependencies
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  ask: vi.fn(),
}));

vi.mock('../lib/state.svelte', () => ({
  state: {
    selectedEmailUid: null,
    isLoadingBody: false,
    emailBody: null,
    attachments: [],
    isLoadingAttachments: false,
    error: null,
    emails: [],
    cmvhVerification: null,
    currentPage: 1,
    resetEmailState: vi.fn(),
  },
}));

vi.mock('../lib/utils', () => ({
  isTrashFolder: vi.fn((folder: string) => folder.toLowerCase().includes('trash')),
}));

vi.mock('$lib/cmvh', () => ({
  loadConfig: vi.fn(() => ({ enabled: false })),
  loadConfigAsync: vi.fn(() => Promise.resolve({ enabled: false, verifyOnChain: false })),
}));

vi.mock('$lib/cmvh/blockchain', () => ({
  verifyOnChain: vi.fn(),
}));

vi.mock('$lib/cmvh/cache', () => ({
  getCachedVerification: vi.fn(() => Promise.resolve(null)),
  cacheVerification: vi.fn(() => Promise.resolve()),
}));

import { invoke } from '@tauri-apps/api/core';
import { ask } from '@tauri-apps/plugin-dialog';
import { state as appState } from '../lib/state.svelte';

describe('Email Operations - Optimistic Updates & Rollback', () => {
  let mockAccounts: AccountConfig[];
  let mockEmails: EmailHeader[];

  beforeEach(() => {
    vi.clearAllMocks();

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

    mockEmails = [
      {
        uid: 100,
        subject: 'Test Email',
        from: 'sender@example.com',
        to: 'receiver@example.com',
        date: 'Mon, 15 Jan 2024 14:30:00 +0000',
        timestamp: 1705331400,
        seen: false,
        flagged: false,
        has_attachments: false,
      },
      {
        uid: 101,
        subject: 'Another Email',
        from: 'sender2@example.com',
        to: 'receiver@example.com',
        date: 'Mon, 16 Jan 2024 10:00:00 +0000',
        timestamp: 1705399200,
        seen: true,
        flagged: true,
        has_attachments: false,
      },
    ];

    // Reset app state
    appState.emails = [...mockEmails];
    appState.error = null;
    appState.selectedEmailUid = null;
  });

  afterEach(() => {
    appState.emails = [];
  });

  describe('handleToggleReadStatus', () => {
    it('should mark email as read with optimistic update', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const unseenEmail = mockEmails[0]; // uid: 100, seen: false
      await EmailOperations.handleToggleReadStatus(
        mockAccounts,
        1,
        unseenEmail.uid,
        'INBOX',
        appState.emails
      );

      // Should be marked as read immediately (optimistic)
      expect(appState.emails[0].seen).toBe(true);

      // Should call backend API
      expect(invoke).toHaveBeenCalledWith('mark_email_as_read', {
        config: mockAccounts[0],
        uid: 100,
        folder: 'INBOX',
      });
    });

    it('should mark email as unread with optimistic update', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const seenEmail = mockEmails[1]; // uid: 101, seen: true
      await EmailOperations.handleToggleReadStatus(
        mockAccounts,
        1,
        seenEmail.uid,
        'INBOX',
        appState.emails
      );

      // Should be marked as unread immediately (optimistic)
      expect(appState.emails[1].seen).toBe(false);

      // Should call backend API
      expect(invoke).toHaveBeenCalledWith('mark_email_as_unread', {
        config: mockAccounts[0],
        uid: 101,
        folder: 'INBOX',
      });
    });

    it('should rollback optimistic update on failure (mark as read)', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Network error'));

      const unseenEmail = mockEmails[0]; // uid: 100, seen: false
      await EmailOperations.handleToggleReadStatus(
        mockAccounts,
        1,
        unseenEmail.uid,
        'INBOX',
        appState.emails
      );

      // Should be rolled back to original state (unseen)
      expect(appState.emails[0].seen).toBe(false);

      // Should set error message
      expect(appState.error).toContain('Failed to mark as read');
      expect(appState.error).toContain('Network error');
    });

    it('should rollback optimistic update on failure (mark as unread)', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('IMAP timeout'));

      const seenEmail = mockEmails[1]; // uid: 101, seen: true
      await EmailOperations.handleToggleReadStatus(
        mockAccounts,
        1,
        seenEmail.uid,
        'INBOX',
        appState.emails
      );

      // Should be rolled back to original state (seen)
      expect(appState.emails[1].seen).toBe(true);

      // Should set error message
      expect(appState.error).toContain('Failed to mark as unread');
      expect(appState.error).toContain('IMAP timeout');
    });

    it('should handle missing account configuration', async () => {
      await EmailOperations.handleToggleReadStatus(
        mockAccounts,
        999, // Non-existent account ID
        100,
        'INBOX',
        appState.emails
      );

      expect(appState.error).toBe('Could not find selected account configuration.');
      expect(invoke).not.toHaveBeenCalled();
    });

    it('should handle missing email', async () => {
      await EmailOperations.handleToggleReadStatus(
        mockAccounts,
        1,
        999, // Non-existent UID
        'INBOX',
        appState.emails
      );

      expect(appState.error).toBe('Could not find selected email.');
      expect(invoke).not.toHaveBeenCalled();
    });
  });

  describe('handleStarToggle', () => {
    it('should flag email with optimistic update', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const unflaggedEmail = mockEmails[0]; // uid: 100, flagged: false
      await EmailOperations.handleStarToggle(
        unflaggedEmail.uid,
        true,
        mockAccounts,
        1,
        'INBOX',
        appState.emails
      );

      // Should be flagged immediately (optimistic)
      expect(appState.emails[0].flagged).toBe(true);

      // Should call backend API
      expect(invoke).toHaveBeenCalledWith('mark_email_as_flagged', {
        config: mockAccounts[0],
        uid: 100,
        folder: 'INBOX',
      });
    });

    it('should unflag email with optimistic update', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const flaggedEmail = mockEmails[1]; // uid: 101, flagged: true
      await EmailOperations.handleStarToggle(
        flaggedEmail.uid,
        false,
        mockAccounts,
        1,
        'INBOX',
        appState.emails
      );

      // Should be unflagged immediately (optimistic)
      expect(appState.emails[1].flagged).toBe(false);

      // Should call backend API
      expect(invoke).toHaveBeenCalledWith('mark_email_as_unflagged', {
        config: mockAccounts[0],
        uid: 101,
        folder: 'INBOX',
      });
    });

    it('should rollback optimistic update on failure (flag)', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Server error'));

      const unflaggedEmail = mockEmails[0]; // uid: 100, flagged: false
      await EmailOperations.handleStarToggle(
        unflaggedEmail.uid,
        true,
        mockAccounts,
        1,
        'INBOX',
        appState.emails
      );

      // Should be rolled back to original state (unflagged)
      expect(appState.emails[0].flagged).toBe(false);

      // Should set error message
      expect(appState.error).toContain('Failed to star email');
      expect(appState.error).toContain('Server error');
    });

    it('should rollback optimistic update on failure (unflag)', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Connection lost'));

      const flaggedEmail = mockEmails[1]; // uid: 101, flagged: true
      await EmailOperations.handleStarToggle(
        flaggedEmail.uid,
        false,
        mockAccounts,
        1,
        'INBOX',
        appState.emails
      );

      // Should be rolled back to original state (flagged)
      expect(appState.emails[1].flagged).toBe(true);

      // Should set error message
      expect(appState.error).toContain('Failed to unstar email');
      expect(appState.error).toContain('Connection lost');
    });
  });

  describe('handleDeleteEmail', () => {
    it('should move email to trash (not from trash folder)', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const loadEmailsForFolder = vi.fn();
      const emailToDelete = mockEmails[0]; // uid: 100

      appState.selectedEmailUid = emailToDelete.uid;

      await EmailOperations.handleDeleteEmail(
        mockAccounts,
        1,
        emailToDelete.uid,
        'INBOX', // Not a trash folder
        appState.emails,
        loadEmailsForFolder
      );

      // Should remove email from UI immediately (optimistic)
      expect(appState.emails.find((e) => e.uid === 100)).toBeUndefined();

      // Should call move_email_to_trash (not permanent delete)
      expect(invoke).toHaveBeenCalledWith('move_email_to_trash', {
        config: mockAccounts[0],
        uid: 100,
        folder: 'INBOX',
      });

      // Should NOT call permanent delete
      expect(invoke).not.toHaveBeenCalledWith('delete_email', expect.any(Object));
    });

    it('should permanently delete from trash folder with confirmation', async () => {
      vi.mocked(ask).mockResolvedValue(true); // User confirms
      vi.mocked(invoke).mockResolvedValue(undefined);

      const loadEmailsForFolder = vi.fn();
      const emailToDelete = mockEmails[0]; // uid: 100

      appState.selectedEmailUid = emailToDelete.uid;

      await EmailOperations.handleDeleteEmail(
        mockAccounts,
        1,
        emailToDelete.uid,
        'Trash', // IS a trash folder
        appState.emails,
        loadEmailsForFolder
      );

      // Should ask for confirmation
      expect(ask).toHaveBeenCalledWith(
        expect.stringContaining('PERMANENTLY delete'),
        expect.objectContaining({
          title: 'Permanently Delete Email?',
          kind: 'warning',
        })
      );

      // Should remove email from UI immediately (optimistic)
      expect(appState.emails.find((e) => e.uid === 100)).toBeUndefined();

      // Should call permanent delete
      expect(invoke).toHaveBeenCalledWith('delete_email', {
        config: mockAccounts[0],
        uid: 100,
        folder: 'Trash',
      });
    });

    it('should cancel permanent delete if user declines confirmation', async () => {
      vi.mocked(ask).mockResolvedValue(false); // User cancels

      const loadEmailsForFolder = vi.fn();
      const emailToDelete = mockEmails[0]; // uid: 100

      appState.selectedEmailUid = emailToDelete.uid;

      await EmailOperations.handleDeleteEmail(
        mockAccounts,
        1,
        emailToDelete.uid,
        'Trash',
        appState.emails,
        loadEmailsForFolder
      );

      // Should NOT delete email
      expect(invoke).not.toHaveBeenCalled();

      // Email should still exist in state (optimistic update not applied)
      expect(appState.emails.find((e) => e.uid === 100)).toBeDefined();
    });

    it('should reload emails on delete failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Delete failed'));

      const loadEmailsForFolder = vi.fn();
      const emailToDelete = mockEmails[0]; // uid: 100

      appState.selectedEmailUid = emailToDelete.uid;

      await EmailOperations.handleDeleteEmail(
        mockAccounts,
        1,
        emailToDelete.uid,
        'INBOX',
        appState.emails,
        loadEmailsForFolder
      );

      // Should set error
      expect(appState.error).toContain('Failed to delete email');

      // Should reload to sync with server
      expect(loadEmailsForFolder).toHaveBeenCalledWith('INBOX');
    });
  });

  describe('handlePageChange', () => {
    it('should update current page and reset email state', () => {
      appState.currentPage = 1;
      appState.selectedEmailUid = 100;
      appState.emailBody = '<p>Test body</p>';
      appState.attachments = [{ id: 1, filename: 'test.pdf' }] as any;

      EmailOperations.handlePageChange(2);

      expect(appState.currentPage).toBe(2);
      expect(appState.selectedEmailUid).toBeNull();
      expect(appState.emailBody).toBeNull();
      expect(appState.attachments).toEqual([]);
    });
  });

  describe('handleEmailClick', () => {
    it('should load email body and mark as read', async () => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'fetch_email_body_cached') {
          return Promise.resolve('<p>Email body content</p>');
        }
        if (cmd === 'load_attachments_info') {
          return Promise.resolve([]);
        }
        if (cmd === 'mark_email_as_read') {
          return Promise.resolve();
        }
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const unseenEmail = mockEmails[0]; // uid: 100, seen: false

      await EmailOperations.handleEmailClick(
        unseenEmail.uid,
        mockAccounts,
        1,
        'INBOX'
      );

      // Should set loading state initially
      expect(appState.selectedEmailUid).toBe(100);

      // Should load email body
      expect(invoke).toHaveBeenCalledWith('fetch_email_body_cached', {
        config: mockAccounts[0],
        uid: 100,
        folder: 'INBOX',
      });

      // Should mark as read (because email was unseen)
      expect(invoke).toHaveBeenCalledWith('mark_email_as_read', {
        config: mockAccounts[0],
        uid: 100,
        folder: 'INBOX',
      });

      // Should update local state
      expect(appState.emails[0].seen).toBe(true);
      expect(appState.isLoadingBody).toBe(false);
    });

    it('should NOT mark as read if email is already seen', async () => {
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === 'fetch_email_body_cached') {
          return Promise.resolve('<p>Email body content</p>');
        }
        if (cmd === 'load_attachments_info') {
          return Promise.resolve([]);
        }
        return Promise.reject(new Error(`Unknown command: ${cmd}`));
      });

      const seenEmail = mockEmails[1]; // uid: 101, seen: true

      await EmailOperations.handleEmailClick(
        seenEmail.uid,
        mockAccounts,
        1,
        'INBOX'
      );

      // Should load email body
      expect(invoke).toHaveBeenCalledWith('fetch_email_body_cached', expect.any(Object));

      // Should NOT call mark_email_as_read (email already seen)
      expect(invoke).not.toHaveBeenCalledWith('mark_email_as_read', expect.any(Object));
    });

    it('should handle missing account configuration', async () => {
      await EmailOperations.handleEmailClick(
        100,
        mockAccounts,
        999, // Non-existent account
        'INBOX'
      );

      expect(appState.error).toBe('Could not find selected account configuration.');
      expect(appState.isLoadingBody).toBe(false);
      expect(invoke).not.toHaveBeenCalled();
    });

    it('should handle email body fetch failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('IMAP connection failed'));

      await EmailOperations.handleEmailClick(
        100,
        mockAccounts,
        1,
        'INBOX'
      );

      expect(appState.error).toContain('Failed to fetch email body');
      expect(appState.error).toContain('IMAP connection failed');
      expect(appState.isLoadingBody).toBe(false);
    });
  });

  describe('handleMarkEmailAsRead', () => {
    it('should mark email as read with optimistic update', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const unseenEmail = mockEmails[0]; // uid: 100, seen: false

      await EmailOperations.handleMarkEmailAsRead(
        unseenEmail.uid,
        mockAccounts,
        1,
        'INBOX',
        appState.emails
      );

      // Should be marked as read immediately (optimistic)
      expect(appState.emails[0].seen).toBe(true);

      // Should call backend API
      expect(invoke).toHaveBeenCalledWith('mark_email_as_read', {
        config: mockAccounts[0],
        uid: 100,
        folder: 'INBOX',
      });
    });

    it('should rollback on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Network timeout'));

      const unseenEmail = mockEmails[0]; // uid: 100, seen: false

      await EmailOperations.handleMarkEmailAsRead(
        unseenEmail.uid,
        mockAccounts,
        1,
        'INBOX',
        appState.emails
      );

      // Should be rolled back to original state (unseen)
      expect(appState.emails[0].seen).toBe(false);

      // Should set error
      expect(appState.error).toContain('Failed to mark as read');
      expect(appState.error).toContain('Network timeout');
    });
  });

  describe('handleMarkEmailAsUnread', () => {
    it('should mark email as unread with optimistic update', async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      const seenEmail = mockEmails[1]; // uid: 101, seen: true

      await EmailOperations.handleMarkEmailAsUnread(
        seenEmail.uid,
        mockAccounts,
        1,
        'INBOX',
        appState.emails
      );

      // Should be marked as unread immediately (optimistic)
      expect(appState.emails[1].seen).toBe(false);

      // Should call backend API
      expect(invoke).toHaveBeenCalledWith('mark_email_as_unread', {
        config: mockAccounts[0],
        uid: 101,
        folder: 'INBOX',
      });
    });

    it('should rollback on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Server unavailable'));

      const seenEmail = mockEmails[1]; // uid: 101, seen: true

      await EmailOperations.handleMarkEmailAsUnread(
        seenEmail.uid,
        mockAccounts,
        1,
        'INBOX',
        appState.emails
      );

      // Should be rolled back to original state (seen)
      expect(appState.emails[1].seen).toBe(true);

      // Should set error
      expect(appState.error).toContain('Failed to mark as unread');
      expect(appState.error).toContain('Server unavailable');
    });
  });
});
