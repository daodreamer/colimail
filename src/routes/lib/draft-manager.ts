import { invoke } from "@tauri-apps/api/core";
import type { DraftAttachment, DraftType, DraftListItem } from "./types";

export class DraftManager {
  private autoSaveTimer: number | null = null;
  private readonly AUTO_SAVE_DELAY = 3000; // 3 seconds

  /**
   * Save draft to local database
   */
  async saveDraft(
    accountId: number,
    toAddr: string,
    ccAddr: string,
    subject: string,
    body: string,
    attachments: DraftAttachment[],
    draftType: DraftType,
    draftId?: number
  ): Promise<number> {
    try {
      const attachmentsJson = JSON.stringify(attachments);
      const id = await invoke<number>("save_draft", {
        accountId,
        toAddr,
        ccAddr,
        subject,
        body,
        attachments: attachmentsJson,
        draftType,
        draftId: draftId ?? null,
      });
      return id;
    } catch (error) {
      console.error("Failed to save draft:", error);
      throw error;
    }
  }

  /**
   * Load draft from local database
   */
  async loadDraft(draftId: number): Promise<{
    toAddr: string;
    ccAddr: string;
    subject: string;
    body: string;
    attachments: DraftAttachment[];
    draftType: DraftType;
  }> {
    try {
      const [toAddr, ccAddr, subject, body, attachmentsJson, draftType] = await invoke<
        [string, string, string, string, string, string]
      >("load_draft", { draftId });

      const attachments: DraftAttachment[] = attachmentsJson ? JSON.parse(attachmentsJson) : [];

      return {
        toAddr,
        ccAddr,
        subject,
        body,
        attachments,
        draftType: draftType as DraftType,
      };
    } catch (error) {
      console.error("Failed to load draft:", error);
      throw error;
    }
  }

  /**
   * List all drafts for an account
   */
  async listDrafts(accountId: number): Promise<DraftListItem[]> {
    try {
      const drafts = await invoke<DraftListItem[]>("list_drafts", { accountId });
      return drafts;
    } catch (error) {
      console.error("Failed to list drafts:", error);
      throw error;
    }
  }

  /**
   * Delete a draft
   */
  async deleteDraft(draftId: number): Promise<void> {
    try {
      await invoke("delete_draft", { draftId });
    } catch (error) {
      console.error("Failed to delete draft:", error);
      throw error;
    }
  }

  /**
   * Schedule auto-save with debounce
   */
  scheduleAutoSave(callback: () => Promise<void>) {
    if (this.autoSaveTimer !== null) {
      clearTimeout(this.autoSaveTimer);
    }

    this.autoSaveTimer = window.setTimeout(async () => {
      try {
        await callback();
      } catch (error) {
        console.error("Auto-save failed:", error);
      }
    }, this.AUTO_SAVE_DELAY);
  }

  /**
   * Cancel scheduled auto-save
   */
  cancelAutoSave() {
    if (this.autoSaveTimer !== null) {
      clearTimeout(this.autoSaveTimer);
      this.autoSaveTimer = null;
    }
  }

  /**
   * Convert Files to DraftAttachments
   */
  async filesToDraftAttachments(files: File[]): Promise<DraftAttachment[]> {
    const attachments: DraftAttachment[] = [];

    for (const file of files) {
      const arrayBuffer = await file.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);
      const dataArray = Array.from(uint8Array);

      attachments.push({
        filename: file.name,
        content_type: file.type || "application/octet-stream",
        data: dataArray,
      });
    }

    return attachments;
  }

  /**
   * Convert DraftAttachments back to File objects (for editing)
   */
  draftAttachmentsToFiles(attachments: DraftAttachment[]): File[] {
    return attachments.map((att) => {
      const uint8Array = new Uint8Array(att.data);
      const blob = new Blob([uint8Array], { type: att.content_type });
      return new File([blob], att.filename, { type: att.content_type });
    });
  }
}

// Export singleton instance
export const draftManager = new DraftManager();
