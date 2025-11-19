/**
 * Compose and Send Email Handlers
 * Handles email composition, reply, forward, and sending operations
 */

import { invoke } from "@tauri-apps/api/core";
import { toast } from "svelte-sonner";
import type { AccountConfig, EmailHeader, CMVHHeaders } from "../lib/types";
import { state as appState } from "../lib/state.svelte";
import { draftManager } from "../lib/draft-manager";
import { loadConfig } from "$lib/cmvh";
import { RewardService } from "$lib/cmvh/reward-service";
import { walletStore } from "$lib/stores/wallet.svelte";

/**
 * Handle compose new email button click
 */
export async function handleComposeClick(
  selectedAccountId: number | null,
  updateAttachmentSizeLimit: () => Promise<void>
) {
  if (!selectedAccountId) {
    appState.error = "Please select an account first.";
    return;
  }
  appState.showComposeDialog = true;
  appState.resetComposeState();
  await updateAttachmentSizeLimit();
}

/**
 * Handle reply to email button click
 */
export async function handleReplyClick(
  selectedAccountId: number | null,
  selectedEmailUid: number | null,
  emails: EmailHeader[],
  updateAttachmentSizeLimit: () => Promise<void>
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

  appState.showComposeDialog = true;
  appState.isReplyMode = true;
  appState.isForwardMode = false;
  appState.composeTo = selectedEmail.from;
  appState.composeSubject = selectedEmail.subject.toLowerCase().startsWith("re:")
    ? selectedEmail.subject
    : `Re: ${selectedEmail.subject}`;
  appState.composeBody = "";
  appState.composeAttachments = [];
  appState.error = null;
  await updateAttachmentSizeLimit();
}

/**
 * Handle forward email button click
 */
export async function handleForwardClick(
  selectedAccountId: number | null,
  selectedEmailUid: number | null,
  emails: EmailHeader[],
  updateAttachmentSizeLimit: () => Promise<void>
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

  appState.showComposeDialog = true;
  appState.isReplyMode = false;
  appState.isForwardMode = true;
  appState.composeTo = "";
  appState.composeSubject = selectedEmail.subject.toLowerCase().startsWith("fwd:")
    ? selectedEmail.subject
    : `Fwd: ${selectedEmail.subject}`;
  appState.composeBody = "";
  appState.composeAttachments = [];
  appState.error = null;
  await updateAttachmentSizeLimit();
}

/**
 * Handle attachment selection
 */
export function handleAttachmentSelect(event: Event) {
  const input = event.target as HTMLInputElement;
  if (!input.files) return;

  const newFiles = Array.from(input.files);
  const allFiles = [...appState.composeAttachments, ...newFiles];

  const totalSize = allFiles.reduce((sum, file) => sum + file.size, 0);
  if (totalSize > appState.attachmentSizeLimit) {
    const limitMB = (appState.attachmentSizeLimit / (1024 * 1024)).toFixed(2);
    const totalMB = (totalSize / (1024 * 1024)).toFixed(2);
    appState.error = `Total attachment size (${totalMB} MB) exceeds the limit for your email provider (${limitMB} MB)`;
    input.value = "";
    return;
  }

  appState.composeAttachments = allFiles;
  input.value = "";
  appState.error = null;
}

/**
 * Remove attachment from compose dialog
 */
export function removeAttachment(index: number) {
  appState.composeAttachments = appState.composeAttachments.filter((_, i) => i !== index);
}

/**
 * Update attachment size limit for selected account
 */
export async function updateAttachmentSizeLimit(
  selectedAccountId: number | null,
  accounts: AccountConfig[]
) {
  if (!selectedAccountId) return;

  const selectedConfig = accounts.find((acc) => acc.id === selectedAccountId);
  if (!selectedConfig) return;

  try {
    const limit = await invoke<number>("get_attachment_size_limit", {
      email: selectedConfig.email,
    });
    appState.attachmentSizeLimit = limit;
  } catch (e) {
    console.error("❌ Failed to get attachment size limit:", e);
  }
}

/**
 * Handle send email button click
 */
export async function handleSendEmail(
  selectedAccountId: number | null,
  selectedEmailUid: number | null,
  accounts: AccountConfig[],
  emails: EmailHeader[],
  emailBody: string | null,
  loadDrafts: () => Promise<void>
) {
  if (!selectedAccountId) {
    appState.error = "Please select an account first.";
    return;
  }

  if (!appState.composeTo || !appState.composeSubject) {
    appState.error = "Please fill in recipient and subject fields.";
    return;
  }

  const selectedConfig = accounts.find((acc) => acc.id === selectedAccountId);
  if (!selectedConfig) {
    appState.error = "Could not find selected account configuration.";
    return;
  }

  appState.isSending = true;
  appState.error = null;

  try {
    let attachmentsData: Array<{ filename: string; content_type: string; data: number[] }> | null = null;
    if (appState.composeAttachments.length > 0) {
      attachmentsData = [];
      for (const file of appState.composeAttachments) {
        const arrayBuffer = await file.arrayBuffer();
        const uint8Array = new Uint8Array(arrayBuffer);
        const dataArray = Array.from(uint8Array);

        attachmentsData.push({
          filename: file.name,
          content_type: file.type || "application/octet-stream",
          data: dataArray,
        });
      }
    }

    let result: string;

    // Check if CMVH signing is enabled for this email
    const shouldSignWithCMVH = appState.enableCMVHSigning;

    if (appState.isReplyMode) {
      result = await invoke<string>("reply_email", {
        config: selectedConfig,
        to: appState.composeTo,
        originalSubject: appState.composeSubject,
        body: appState.composeBody,
        cc: appState.composeCc || null,
        attachments: attachmentsData,
      });
    } else if (appState.isForwardMode) {
      const selectedEmail = emails.find((email) => email.uid === selectedEmailUid);
      if (!selectedEmail) {
        appState.error = "Could not find selected email.";
        appState.isSending = false;
        return;
      }
      result = await invoke<string>("forward_email", {
        config: selectedConfig,
        params: {
          to: appState.composeTo,
          originalSubject: selectedEmail.subject,
          originalFrom: selectedEmail.from,
          originalTo: selectedEmail.to,
          originalDate: selectedEmail.date,
          originalBody: emailBody || "",
          additionalMessage: appState.composeBody,
          cc: appState.composeCc || null,
          attachments: attachmentsData,
        },
      });
    } else {
      if (!appState.composeBody) {
        appState.error = "Please fill in the message body.";
        appState.isSending = false;
        return;
      }

      // First send the email, then attach reward if needed
      // (we do this because we need the email to be sent before creating the reward)

      // Send with or without CMVH signing based on user choice
      if (shouldSignWithCMVH) {
        // Load CMVH config to get private key
        const cmvhConfig = loadConfig();

        if (!cmvhConfig.privateKey || !cmvhConfig.derivedAddress) {
          appState.error = "CMVH signing enabled but private key not configured. Please configure in Settings.";
          appState.isSending = false;
          return;
        }

        try {
          // Step 1: Sign the email metadata
          const cmvhHeaders = await invoke<CMVHHeaders>("sign_email_with_cmvh", {
            privateKey: cmvhConfig.privateKey,
            content: {
              from: selectedConfig.email,
              to: appState.composeTo,
              subject: appState.composeSubject,
              body: "", // Body is not used in signature
            },
          });

          console.log("✅ Email signed with CMVH:", cmvhHeaders);

          // Step 2: Send email with CMVH headers
          result = await invoke<string>("send_email_with_cmvh", {
            config: selectedConfig,
            to: appState.composeTo,
            subject: appState.composeSubject,
            body: appState.composeBody,
            cc: appState.composeCc || null,
            attachments: attachmentsData,
            cmvhHeaders: cmvhHeaders,
          });

          toast.success("Email signed with CMVH and sent successfully!");
        } catch (signError) {
          console.error("❌ Failed to sign email with CMVH:", signError);
          appState.error = `Failed to sign email: ${signError}`;
          appState.isSending = false;
          return;
        }
      } else {
        // Send without CMVH signing (regular email)
        result = await invoke<string>("send_email", {
          config: selectedConfig,
          to: appState.composeTo,
          subject: appState.composeSubject,
          body: appState.composeBody,
          cc: appState.composeCc || null,
          attachments: attachmentsData,
        });
      }
    }

    // Handle Reward Attachment - AFTER email is successfully sent
    if (appState.attachReward && !appState.isReplyMode && !appState.isForwardMode) {
      if (!walletStore.isConnected) {
        toast.warning("Email sent, but reward not attached: wallet not connected.");
      } else if (!appState.rewardRecipient || !appState.rewardRecipient.startsWith("0x") || appState.rewardRecipient.length !== 42) {
        toast.warning("Email sent, but reward not attached: invalid recipient address.");
      } else {
        try {
          toast.loading("Attaching reward to blockchain...");

          const cmvhConfig = loadConfig();
          const rewardService = new RewardService(cmvhConfig);

          // Extract the recipient address
          const recipientAddress = appState.rewardRecipient;

          // Create email content object for hashing
          const emailContent = {
            subject: appState.composeSubject,
            from: selectedConfig.email,
            to: appState.composeTo,
            body: appState.composeBody,
          };

          // Create reward with 30 days expiry (default)
          const txHash = await rewardService.createReward(
            recipientAddress,
            appState.rewardAmount,
            emailContent,
            30 * 24 * 60 * 60 // 30 days in seconds
          );

          toast.dismiss();
          toast.success(`Reward attached! Transaction: ${txHash.slice(0, 10)}...${txHash.slice(-8)}`, {
            duration: 6000,
          });

          console.log("✅ Reward created:", {
            txHash,
            recipient: recipientAddress,
            amount: appState.rewardAmount,
            emailSubject: appState.composeSubject,
          });
        } catch (rewardError) {
          toast.dismiss();
          console.error("❌ Failed to attach reward:", rewardError);
          toast.error(`Email sent, but failed to attach reward: ${rewardError}`);
        }
      }
    }

    // Delete draft after successful send
    if (appState.currentDraftId) {
      try {
        await draftManager.deleteDraft(appState.currentDraftId);
      } catch (error) {
        console.error("Failed to delete draft after sending:", error);
      }
    }

    // Close compose dialog without showing save draft dialog
    appState.showComposeDialog = false;
    appState.resetComposeState();

    // Only show generic success toast if CMVH-specific toast wasn't already shown
    // and reward creation wasn't triggered
    if (!shouldSignWithCMVH && !appState.attachReward) {
      toast.success("Email sent successfully!");
    } else if (shouldSignWithCMVH && !appState.attachReward) {
      // CMVH toast was already shown
    } else if (!shouldSignWithCMVH && appState.attachReward) {
      // Reward toast will be shown
      toast.success("Email sent successfully!");
    }
  } catch (e) {
    appState.error = `Failed to send email: ${e}`;
  } finally {
    appState.isSending = false;
  }
}
