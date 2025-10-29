// Global state management using Svelte 5 runes
// This file contains all application state

import type { AccountConfig, EmailHeader, Folder, AttachmentInfo, DraftListItem } from "./types";

// Create a reactive state object
class AppState {
  // Account state
  accounts = $state<AccountConfig[]>([]);
  selectedAccountId = $state<number | null>(null);

  // Folder state
  folders = $state<Folder[]>([]);
  selectedFolderName = $state<string>("INBOX");
  isLoadingFolders = $state<boolean>(false);

  // Email list state
  emails = $state<EmailHeader[]>([]);
  selectedEmailUid = $state<number | null>(null);
  isLoadingEmails = $state<boolean>(false);

  // Pagination state
  currentPage = $state<number>(1);
  pageSize = $state<number>(50); // Default 50 emails per page

  // Email body state
  emailBody = $state<string | null>(null);
  isLoadingBody = $state<boolean>(false);

  // Attachments state
  attachments = $state<AttachmentInfo[]>([]);
  isLoadingAttachments = $state<boolean>(false);

  // Sync state
  isSyncing = $state<boolean>(false);
  lastSyncTime = $state<number>(0);
  syncInterval = $state<number>(300); // Default 5 minutes
  currentTime = $state<number>(Math.floor(Date.now() / 1000));

  // Compose dialog state
  showComposeDialog = $state<boolean>(false);
  composeTo = $state<string>("");
  composeCc = $state<string>("");
  composeSubject = $state<string>("");
  composeBody = $state<string>("");
  isReplyMode = $state<boolean>(false);
  isForwardMode = $state<boolean>(false);
  isSending = $state<boolean>(false);
  composeAttachments = $state<File[]>([]);
  attachmentSizeLimit = $state<number>(10 * 1024 * 1024); // Default 10MB
  currentDraftId = $state<number | null>(null); // Track the current draft being edited
  showSaveDraftDialog = $state<boolean>(false); // Show save draft confirmation dialog
  autoSaveTimerId = $state<number | null>(null); // Auto-save timer ID

  // Draft state
  drafts = $state<DraftListItem[]>([]);
  isLoadingDrafts = $state<boolean>(false);
  showDraftsFolder = $state<boolean>(false);

  // Derived state
  totalAttachmentSize = $derived<number>(
    this.composeAttachments.reduce((sum, file) => sum + file.size, 0)
  );

  selectedEmail = $derived<EmailHeader | undefined>(
    this.emails.find((email) => email.uid === this.selectedEmailUid)
  );

  // Error state
  error = $state<string | null>(null);

  // Helper methods
  resetComposeState() {
    this.composeTo = "";
    this.composeCc = "";
    this.composeSubject = "";
    this.composeBody = "";
    this.composeAttachments = [];
    this.isReplyMode = false;
    this.isForwardMode = false;
    this.isSending = false;
    this.currentDraftId = null;
    this.error = null;
    // Clear auto-save timer
    if (this.autoSaveTimerId !== null) {
      clearTimeout(this.autoSaveTimerId);
      this.autoSaveTimerId = null;
    }
  }

  resetEmailState() {
    this.selectedEmailUid = null;
    this.emailBody = null;
    this.attachments = [];
  }

  resetFolderState() {
    this.emails = [];
    this.resetEmailState();
    this.currentPage = 1; // Reset to first page when switching folders
  }
}

// Export a singleton instance
export const state = new AppState();
