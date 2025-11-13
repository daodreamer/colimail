/**
 * Draft Management Handlers
 * Handles draft creation, loading, saving, and deletion
 */

import { toast } from "svelte-sonner";
import type { DraftType } from "../lib/types";
import { state as appState } from "../lib/state.svelte";
import { draftManager } from "../lib/draft-manager";

/**
 * Auto-save draft when compose state changes
 */
export async function autoSaveDraft(selectedAccountId: number | null) {
  if (!selectedAccountId) return;
  if (!appState.composeTo && !appState.composeSubject && !appState.composeBody) return;

  try {
    const attachments = await draftManager.filesToDraftAttachments(appState.composeAttachments);
    const draftType: DraftType = appState.isReplyMode ? "reply" : appState.isForwardMode ? "forward" : "compose";

    const draftId = await draftManager.saveDraft(
      selectedAccountId,
      appState.composeTo,
      appState.composeCc,
      appState.composeSubject,
      appState.composeBody,
      attachments,
      draftType,
      appState.currentDraftId ?? undefined
    );

    // Update current draft ID if it's a new draft
    if (!appState.currentDraftId) {
      appState.currentDraftId = draftId;
    }
  } catch (error) {
    console.error("Auto-save draft failed:", error);
  }
}

/**
 * Handle save draft button click
 */
export async function handleSaveDraft(selectedAccountId: number | null, loadDrafts: () => Promise<void>) {
  if (!selectedAccountId) {
    appState.error = "Please select an account first.";
    return;
  }

  try {
    const attachments = await draftManager.filesToDraftAttachments(appState.composeAttachments);
    const draftType: DraftType = appState.isReplyMode ? "reply" : appState.isForwardMode ? "forward" : "compose";

    const draftId = await draftManager.saveDraft(
      selectedAccountId,
      appState.composeTo,
      appState.composeCc,
      appState.composeSubject,
      appState.composeBody,
      attachments,
      draftType,
      appState.currentDraftId ?? undefined
    );
    appState.currentDraftId = draftId;

    // Close compose dialog
    appState.showComposeDialog = false;
    appState.showSaveDraftDialog = false;
    appState.resetComposeState();

    // Reload drafts list if we're in drafts view
    if (appState.showDraftsFolder) {
      await loadDrafts();
    }

    toast.success("Draft saved successfully");
  } catch (error) {
    appState.error = `Failed to save draft: ${error}`;
  }
}

/**
 * Handle close compose dialog
 */
export function handleCloseCompose(selectedAccountId: number | null) {
  // Check if there's any content worth saving
  const hasContent = appState.composeTo || appState.composeSubject || appState.composeBody;

  if (hasContent && selectedAccountId) {
    // Show save draft confirmation dialog (keep compose dialog open)
    appState.showSaveDraftDialog = true;
  } else {
    // No content, just close
    appState.showComposeDialog = false;
    appState.resetComposeState();
  }
}

/**
 * Discard draft and close compose dialog
 */
export function handleDiscardDraft() {
  // Delete draft if it exists
  if (appState.currentDraftId) {
    draftManager.deleteDraft(appState.currentDraftId).catch((error) => {
      console.error("Failed to delete draft:", error);
    });
  }

  appState.showSaveDraftDialog = false;
  appState.showComposeDialog = false;
  appState.resetComposeState();
}

/**
 * Cancel save draft dialog
 */
export function handleCancelSaveDraft() {
  // Just close the save draft dialog, keep compose dialog open
  appState.showSaveDraftDialog = false;
  // Don't close the compose dialog - user wants to continue editing
}

/**
 * Save draft and close compose dialog
 */
export async function handleSaveDraftAndClose(
  selectedAccountId: number | null,
  loadDrafts: () => Promise<void>
) {
  await handleSaveDraft(selectedAccountId, loadDrafts);
}

/**
 * Load drafts for current account
 */
export async function loadDrafts(selectedAccountId: number | null) {
  if (!selectedAccountId) return;

  appState.isLoadingDrafts = true;
  try {
    const drafts = await draftManager.listDrafts(selectedAccountId);
    appState.drafts = drafts;
  } catch (error) {
    console.error("Failed to load drafts:", error);
    appState.error = `Failed to load drafts: ${error}`;
  } finally {
    appState.isLoadingDrafts = false;
  }
}

/**
 * Open draft for editing
 */
export async function handleDraftClick(
  draftId: number,
  updateAttachmentSizeLimit: () => Promise<void>
) {
  try {
    const draft = await draftManager.loadDraft(draftId);

    // Set compose state from draft
    appState.composeTo = draft.toAddr;
    appState.composeCc = draft.ccAddr;
    appState.composeSubject = draft.subject;
    appState.composeBody = draft.body;
    appState.currentDraftId = draftId;

    // Convert draft attachments back to File objects
    appState.composeAttachments = draftManager.draftAttachmentsToFiles(draft.attachments);

    // Set mode based on draft type
    appState.isReplyMode = draft.draftType === "reply";
    appState.isForwardMode = draft.draftType === "forward";

    // Open compose dialog
    appState.showComposeDialog = true;

    // Switch back to email view
    appState.showDraftsFolder = false;

    await updateAttachmentSizeLimit();
  } catch (error) {
    appState.error = `Failed to load draft: ${error}`;
  }
}

/**
 * Show confirmation dialog for draft deletion
 */
export function handleDraftDelete(draftId: number): number {
  return draftId;
}

/**
 * Confirm and delete draft
 */
export async function confirmDeleteDraft(
  draftToDelete: number | null,
  loadDrafts: () => Promise<void>
): Promise<void> {
  if (draftToDelete === null) return;

  try {
    await draftManager.deleteDraft(draftToDelete);
    toast.success("Draft deleted");

    // Reload drafts
    await loadDrafts();
  } catch (error) {
    appState.error = `Failed to delete draft: ${error}`;
  }
}

/**
 * Toggle drafts view
 */
export async function handleShowDrafts(loadDrafts: () => Promise<void>) {
  appState.showDraftsFolder = true;
  // Clear selected folder to prevent other folders from showing as selected
  appState.selectedFolderName = "";
  await loadDrafts();
}

/**
 * Hide drafts view
 */
export function handleHideDrafts() {
  appState.showDraftsFolder = false;
}
