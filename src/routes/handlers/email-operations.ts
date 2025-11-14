/**
 * Email Operations Handlers
 * Handles email viewing, status changes, deletion, and related operations
 */

import { invoke } from "@tauri-apps/api/core";
import { ask } from "@tauri-apps/plugin-dialog";
import type { AccountConfig, EmailHeader, AttachmentInfo, CMVHHeaders, CMVHVerificationResult } from "../lib/types";
import { state as appState } from "../lib/state.svelte";
import { isTrashFolder } from "../lib/utils";
import { loadConfig } from "$lib/cmvh";
import { verifyOnChain } from "$lib/cmvh/blockchain";
import { getCachedVerification, cacheVerification } from "$lib/cmvh/cache";
import type { EmailContent } from "$lib/cmvh/cache";

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

    // Check for CMVH verification if enabled
    await verifyCMVHIfEnabled(selectedConfig, uid, selectedFolderName);

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

/**
 * Verify CMVH signature if verification is enabled in settings
 */
async function verifyCMVHIfEnabled(
  config: AccountConfig,
  uid: number,
  folder: string
) {
  // Reset CMVH verification state
  appState.cmvhVerification = null;

  const cmvhConfig = loadConfig();
  if (!cmvhConfig.enabled) {
    console.log("📧 CMVH verification disabled in settings");
    return;
  }

  try {
    // Check if email has CMVH headers
    const rawHeaders = await invoke<string>("fetch_email_raw_headers", {
      config,
      uid,
      folder,
    });

    const hasCMVH = await invoke<boolean>("has_cmvh_headers", {
      rawHeaders,
    });

    if (!hasCMVH) {
      console.log("📧 Email does not contain CMVH headers");
      appState.cmvhVerification = {
        hasCMVH: false,
      };
      return;
    }

    // Parse CMVH headers
    const headers = await invoke<CMVHHeaders>("parse_email_cmvh_headers", {
      rawHeaders,
    });

    // Get email content for verification
    const selectedEmail = appState.emails.find((email) => email.uid === uid);
    if (!selectedEmail) {
      throw new Error("Could not find email");
    }

    // Extract clean email addresses (remove display names)
    const extractEmail = (addr: string): string => {
      const match = addr.match(/<(.+?)>/);
      return match ? match[1] : addr.trim();
    };

    const emailContent = {
      from: extractEmail(selectedEmail.from),
      to: extractEmail(selectedEmail.to),
      subject: selectedEmail.subject,
      body: "", // Body is not used in CMVH verification
    };

    // Verify signature
    const verificationResult = await invoke<{ is_valid: boolean }>("verify_cmvh_signature", {
      headers,
      content: emailContent,
    });

    appState.cmvhVerification = {
      hasCMVH: true,
      isValid: verificationResult.is_valid,
      headers,
      verifiedAt: Date.now(),
    };
  } catch (error) {
    console.error("❌ CMVH verification failed:", error);
    appState.cmvhVerification = {
      hasCMVH: false,
      error: String(error),
    };
  }
}

/**
 * Verify CMVH signature on-chain via smart contract
 */
export async function handleVerifyOnChain(
  selectedEmailUid: number | null,
  emails: EmailHeader[]
) {
  if (!appState.cmvhVerification?.hasCMVH || !appState.cmvhVerification.isValid) {
    console.error("Cannot verify on-chain: email not locally verified");
    return;
  }

  if (!appState.cmvhVerification.headers) {
    console.error("Cannot verify on-chain: missing CMVH headers");
    return;
  }

  // Load config asynchronously to ensure we have the latest from secure storage
  const { loadConfigAsync } = await import("$lib/cmvh");
  const cmvhConfig = await loadConfigAsync();

  console.log("🔧 CMVH Config loaded:");
  console.log(`   Enabled: ${cmvhConfig.enabled}`);
  console.log(`   Verify on-chain: ${cmvhConfig.verifyOnChain}`);
  console.log(`   Contract: ${cmvhConfig.contractAddress}`);
  console.log(`   Network: ${cmvhConfig.network}`);

  if (!cmvhConfig.enabled || !cmvhConfig.verifyOnChain) {
    console.error("On-chain verification is disabled in settings");
    return;
  }

  const selectedEmail = emails.find((email) => email.uid === selectedEmailUid);
  if (!selectedEmail) {
    console.error("Cannot find selected email");
    return;
  }

  // Extract clean email addresses
  const extractEmail = (addr: string): string => {
    const match = addr.match(/<(.+?)>/);
    return match ? match[1] : addr.trim();
  };

  const emailContent: EmailContent = {
    from: extractEmail(selectedEmail.from),
    to: extractEmail(selectedEmail.to),
    subject: selectedEmail.subject,
    body: "", // Body not used in verification
  };

  // 1. Check cache first
  const cached = await getCachedVerification(
    appState.cmvhVerification.headers!,
    emailContent
  );

  if (cached) {
    // Use cached result - instant response
    console.log("⚡ Using cached on-chain verification result");
    appState.cmvhVerification = {
      ...appState.cmvhVerification,
      isOnChainVerified: cached.isValid,
      onChainVerifiedAt: cached.timestamp,
      isVerifyingOnChain: false,
      fromCache: true,
      error: cached.error,
    };
    return;
  }

  // 2. Not cached - perform verification
  appState.cmvhVerification = {
    ...appState.cmvhVerification,
    isVerifyingOnChain: true,
    fromCache: false,
  };

  try {
    console.log("🔗 Verifying CMVH signature on-chain...");
    console.log(`   Network: ${cmvhConfig.network}`);
    console.log(`   RPC: ${cmvhConfig.rpcUrl || "default"}`);

    const result = await verifyOnChain(
      appState.cmvhVerification.headers!, // We already checked it exists above
      emailContent,
      cmvhConfig
    );

    // 3. Cache the result
    await cacheVerification(appState.cmvhVerification.headers!, emailContent, {
      isValid: result.isValid,
      error: result.error,
      timestamp: Date.now(),
    });

    if (result.isValid) {
      console.log("✅ On-chain verification PASSED");
      appState.cmvhVerification = {
        ...appState.cmvhVerification,
        isOnChainVerified: true,
        onChainVerifiedAt: Date.now(),
        isVerifyingOnChain: false,
        fromCache: false,
      };
    } else {
      console.error("❌ On-chain verification FAILED:", result.error);
      appState.cmvhVerification = {
        ...appState.cmvhVerification,
        isOnChainVerified: false,
        isVerifyingOnChain: false,
        fromCache: false,
        error: result.error || "On-chain verification failed",
      };
    }
  } catch (error) {
    console.error("❌ On-chain verification error:", error);

    // Cache the error result to avoid repeated failures
    await cacheVerification(appState.cmvhVerification.headers!, emailContent, {
      isValid: false,
      error: String(error),
      timestamp: Date.now(),
    });

    appState.cmvhVerification = {
      ...appState.cmvhVerification,
      isOnChainVerified: false,
      isVerifyingOnChain: false,
      fromCache: false,
      error: String(error),
    };
  }
}
