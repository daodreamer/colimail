/**
 * Email Operations Handlers
 * Handles email viewing, status changes, deletion, and related operations
 */

import { invoke } from "@tauri-apps/api/core";
import { ask } from "@tauri-apps/plugin-dialog";
import type { AccountConfig, EmailHeader, AttachmentInfo } from "../lib/types";
import { state as appState } from "../lib/state.svelte";
import { isTrashFolder } from "../lib/utils";

/**
 * Handle email click - load and display email body
 */
export async function handleEmailClick(
  uid: number,
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  selectedFolderName: string
) {
  appState.selectedEmailUid = uid;
  appState.isLoadingBody = true;
  appState.emailBody = null;
  appState.attachments = [];
  appState.error = null;

  const selectedConfig = accounts.find((acc) => acc.id === selectedAccountId);
  if (!selectedConfig) {
    appState.error = "Could not find selected account configuration.";
    appState.isLoadingBody = false;
    return;
  }

  try {
    appState.emailBody = await invoke<string>("fetch_email_body_cached", {
      config: selectedConfig,
      uid,
      folder: selectedFolderName,
    });

    console.log(`📧 Loaded email body for UID ${uid}, length: ${appState.emailBody?.length || 0} bytes`);
    console.log(`📧 Body preview (first 200 chars):`, appState.emailBody?.substring(0, 200));

    if (selectedAccountId) {
      await loadAttachmentsForEmail(selectedAccountId, uid, selectedFolderName);
    }

    // Auto-mark as read when opening email
    const selectedEmail = appState.emails.find((email) => email.uid === uid);
    if (selectedEmail && !selectedEmail.seen) {
      try {
        await invoke("mark_email_as_read", {
          config: selectedConfig,
          uid,
          folder: selectedFolderName,
        });

        // Update local state
        selectedEmail.seen = true;
        appState.emails = [...appState.emails];
      } catch (e) {
        console.error("Failed to mark email as read:", e);
      }
    }
  } catch (e) {
    console.error(`❌ Failed to fetch email body:`, e);
    appState.error = `Failed to fetch email body: ${e}`;
  } finally {
    appState.isLoadingBody = false;
  }
}

/**
 * Load attachments for a specific email
 */
async function loadAttachmentsForEmail(
  accountId: number,
  uid: number,
  folderName: string
) {
  appState.isLoadingAttachments = true;
  try {
    appState.attachments = await invoke<AttachmentInfo[]>("load_attachments_info", {
      accountId,
      folderName,
      uid,
    });
  } catch (e) {
    console.error("❌ Failed to load attachments:", e);
  } finally {
    appState.isLoadingAttachments = false;
  }
}

/**
 * Toggle read/unread status for currently selected email
 */
export async function handleToggleReadStatus(
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  selectedEmailUid: number | null,
  selectedFolderName: string,
  emails: EmailHeader[]
) {
  if (!selectedAccountId || !selectedEmailUid) {
    appState.error = "Please select an email first.";
    return;
  }

  const selectedEmail = emails.find((email) => email.uid === selectedEmailUid);
  if (!selectedEmail) {
    appState.error = "Could not find selected email.";
    return;
  }

  const selectedConfig = accounts.find((acc) => acc.id === selectedAccountId);
  if (!selectedConfig) {
    appState.error = "Could not find selected account configuration.";
    return;
  }

  // Store previous state for rollback
  const previousSeenState = selectedEmail.seen;
  const newSeenState = !selectedEmail.seen;

  // Optimistic update: immediately update UI
  selectedEmail.seen = newSeenState;
  appState.emails = [...appState.emails];

  try {
    // Send request to server
    if (newSeenState) {
      await invoke("mark_email_as_read", {
        config: selectedConfig,
        uid: selectedEmailUid,
        folder: selectedFolderName,
      });
    } else {
      await invoke("mark_email_as_unread", {
        config: selectedConfig,
        uid: selectedEmailUid,
        folder: selectedFolderName,
      });
    }
    // Success - UI already updated optimistically
  } catch (e) {
    // Rollback on error
    console.error("❌ Failed to toggle read status, rolling back:", e);
    selectedEmail.seen = previousSeenState;
    appState.emails = [...appState.emails];
    appState.error = `Failed to mark as ${newSeenState ? "read" : "unread"}: ${e}`;
  }
}

/**
 * Mark email as read from context menu
 */
export async function handleMarkEmailAsRead(
  uid: number,
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  selectedFolderName: string,
  emails: EmailHeader[]
) {
  if (!selectedAccountId) {
    appState.error = "Please select an account first.";
    return;
  }

  const selectedEmail = emails.find((email) => email.uid === uid);
  if (!selectedEmail) {
    appState.error = "Could not find email.";
    return;
  }

  const selectedConfig = accounts.find((acc) => acc.id === selectedAccountId);
  if (!selectedConfig) {
    appState.error = "Could not find account configuration.";
    return;
  }

  // Store previous state for rollback
  const previousSeenState = selectedEmail.seen;

  // Optimistic update: immediately update UI
  selectedEmail.seen = true;
  appState.emails = [...appState.emails];

  try {
    // Send request to server
    await invoke("mark_email_as_read", {
      config: selectedConfig,
      uid,
      folder: selectedFolderName,
    });
    // Success - UI already updated optimistically
  } catch (e) {
    // Rollback on error
    console.error("❌ Failed to mark as read, rolling back:", e);
    selectedEmail.seen = previousSeenState;
    appState.emails = [...appState.emails];
    appState.error = `Failed to mark as read: ${e}`;
  }
}

/**
 * Mark email as unread from context menu
 */
export async function handleMarkEmailAsUnread(
  uid: number,
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  selectedFolderName: string,
  emails: EmailHeader[]
) {
  if (!selectedAccountId) {
    appState.error = "Please select an account first.";
    return;
  }

  const selectedEmail = emails.find((email) => email.uid === uid);
  if (!selectedEmail) {
    appState.error = "Could not find email.";
    return;
  }

  const selectedConfig = accounts.find((acc) => acc.id === selectedAccountId);
  if (!selectedConfig) {
    appState.error = "Could not find account configuration.";
    return;
  }

  // Store previous state for rollback
  const previousSeenState = selectedEmail.seen;

  // Optimistic update: immediately update UI
  selectedEmail.seen = false;
  appState.emails = [...appState.emails];

  try {
    // Send request to server
    await invoke("mark_email_as_unread", {
      config: selectedConfig,
      uid,
      folder: selectedFolderName,
    });
    // Success - UI already updated optimistically
  } catch (e) {
    // Rollback on error
    console.error("❌ Failed to mark as unread, rolling back:", e);
    selectedEmail.seen = previousSeenState;
    appState.emails = [...appState.emails];
    appState.error = `Failed to mark as unread: ${e}`;
  }
}

/**
 * Toggle star/flag status for an email
 */
export async function handleStarToggle(
  uid: number,
  flagged: boolean,
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  selectedFolderName: string,
  emails: EmailHeader[]
) {
  if (!selectedAccountId) {
    appState.error = "Please select an account first.";
    return;
  }

  const selectedEmail = emails.find((email) => email.uid === uid);
  if (!selectedEmail) {
    appState.error = "Could not find email.";
    return;
  }

  const selectedConfig = accounts.find((acc) => acc.id === selectedAccountId);
  if (!selectedConfig) {
    appState.error = "Could not find account configuration.";
    return;
  }

  // Store previous state for rollback
  const previousFlaggedState = selectedEmail.flagged;

  // Optimistic update: immediately update UI
  selectedEmail.flagged = flagged;
  appState.emails = [...appState.emails];

  try {
    // Send request to server
    if (flagged) {
      await invoke("mark_email_as_flagged", {
        config: selectedConfig,
        uid,
        folder: selectedFolderName,
      });
    } else {
      await invoke("mark_email_as_unflagged", {
        config: selectedConfig,
        uid,
        folder: selectedFolderName,
      });
    }
    // Success - UI already updated optimistically
  } catch (e) {
    // Rollback on error
    console.error("❌ Failed to toggle star status, rolling back:", e);
    selectedEmail.flagged = previousFlaggedState;
    appState.emails = [...appState.emails];
    appState.error = `Failed to ${flagged ? "star" : "unstar"} email: ${e}`;
  }
}

/**
 * Delete currently selected email
 */
export async function handleDeleteEmail(
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  selectedEmailUid: number | null,
  selectedFolderName: string,
  emails: EmailHeader[],
  loadEmailsForFolder: (folderName: string) => Promise<void>
) {
  if (!selectedAccountId || !selectedEmailUid) {
    appState.error = "Please select an email first.";
    return;
  }

  const selectedEmail = emails.find((email) => email.uid === selectedEmailUid);
  if (!selectedEmail) {
    appState.error = "Could not find selected email.";
    return;
  }

  const selectedConfig = accounts.find((acc) => acc.id === selectedAccountId);
  if (!selectedConfig) {
    appState.error = "Could not find selected account configuration.";
    return;
  }

  const isInTrash = isTrashFolder(selectedFolderName);

  // Only ask for confirmation when permanently deleting from trash folder
  if (isInTrash) {
    const confirmMessage = `Are you sure you want to PERMANENTLY delete this email?\n\nThis action cannot be undone.\n\nSubject: ${selectedEmail.subject}`;

    // IMPORTANT: Use Tauri's ask() dialog instead of native confirm()
    // This properly blocks execution until user responds
    const userConfirmed = await ask(confirmMessage, {
      title: "Permanently Delete Email?",
      kind: "warning",
    });

    if (!userConfirmed) {
      return;
    }
  }

  // Proceed with backend operations
  appState.error = null;

  try {
    const deletedUid = selectedEmailUid;

    // Immediately remove from UI (optimistic update) for instant feedback
    appState.emails = emails.filter((email) => email.uid !== deletedUid);
    appState.resetEmailState();

    if (isInTrash) {
      // Permanently delete from server
      await invoke("delete_email", {
        config: selectedConfig,
        uid: deletedUid,
        folder: selectedFolderName,
      });

      // No success message needed - the UI update provides instant feedback
    } else {
      // Move to trash on server (no confirmation needed)
      await invoke("move_email_to_trash", {
        config: selectedConfig,
        uid: deletedUid,
        folder: selectedFolderName,
      });

      // No success message needed - instant UI feedback is better for smooth UX
    }

    // Note: The IDLE event handler will sync in the background if needed,
    // but we've already updated the UI for immediate feedback
  } catch (e) {
    appState.error = `Failed to delete email: ${e}`;
    // On error, reload to ensure UI is in sync with server
    await loadEmailsForFolder(selectedFolderName);
  }
}

/**
 * Delete email from context menu
 */
export async function handleDeleteEmailFromContextMenu(
  uid: number,
  accounts: AccountConfig[],
  selectedAccountId: number | null,
  selectedFolderName: string,
  emails: EmailHeader[],
  loadEmailsForFolder: (folderName: string) => Promise<void>
) {
  if (!selectedAccountId) {
    appState.error = "Please select an account first.";
    return;
  }

  const selectedEmail = emails.find((email) => email.uid === uid);
  if (!selectedEmail) {
    appState.error = "Could not find email.";
    return;
  }

  const selectedConfig = accounts.find((acc) => acc.id === selectedAccountId);
  if (!selectedConfig) {
    appState.error = "Could not find account configuration.";
    return;
  }

  const isInTrash = isTrashFolder(selectedFolderName);

  // Only ask for confirmation when permanently deleting from trash folder
  if (isInTrash) {
    const confirmMessage = `Are you sure you want to PERMANENTLY delete this email?\n\nThis action cannot be undone.\n\nSubject: ${selectedEmail.subject}`;

    const userConfirmed = await ask(confirmMessage, {
      title: "Permanently Delete Email?",
      kind: "warning",
    });

    if (!userConfirmed) {
      return;
    }
  }

  // Proceed with backend operations
  appState.error = null;

  try {
    // Immediately remove from UI (optimistic update) for instant feedback
    appState.emails = emails.filter((email) => email.uid !== uid);

    // If this was the selected email, reset the selection
    if (appState.selectedEmailUid === uid) {
      appState.resetEmailState();
    }

    if (isInTrash) {
      // Permanently delete from server
      await invoke("delete_email", {
        config: selectedConfig,
        uid,
        folder: selectedFolderName,
      });
    } else {
      // Move to trash on server (no confirmation needed)
      await invoke("move_email_to_trash", {
        config: selectedConfig,
        uid,
        folder: selectedFolderName,
      });
    }

    // Note: The IDLE event handler will sync in the background if needed,
    // but we've already updated the UI for immediate feedback
  } catch (e) {
    appState.error = `Failed to delete email: ${e}`;
    // On error, reload to ensure UI is in sync with server
    await loadEmailsForFolder(selectedFolderName);
  }
}

/**
 * Download attachment to file
 */
export async function downloadAttachment(attachmentId: number, filename: string) {
  try {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const filePath = await save({
      defaultPath: filename,
      title: "Save Attachment",
    });

    if (filePath) {
      await invoke("save_attachment_to_file", {
        attachmentId,
        filePath,
      });
    }
  } catch (e) {
    console.error("❌ Failed to save attachment:", e);
    appState.error = `Failed to download attachment: ${e}`;
  }
}

/**
 * Handle page change for email pagination
 */
export function handlePageChange(page: number) {
  appState.currentPage = page;
  // Reset selected email when changing pages
  appState.selectedEmailUid = null;
  appState.emailBody = null;
  appState.attachments = [];
}
