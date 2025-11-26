<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import * as Sidebar from "$lib/components/ui/sidebar";

  // Components
  import AccountFolderSidebar from "./components/AccountFolderSidebar.svelte";
  import EmailListSidebar from "./components/EmailListSidebar.svelte";
  import DraftsList from "./components/DraftsList.svelte";
  import EmailBody from "./components/EmailBody.svelte";
  import ComposeDialog from "./components/ComposeDialog.svelte";
  import SaveDraftDialog from "./components/SaveDraftDialog.svelte";
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import SettingsDialog from "./components/SettingsDialog.svelte";
  import AddAccountDialog from "./components/AddAccountDialog.svelte";
  import ManageAccountDialog from "./components/ManageAccountDialog.svelte";
  import SetMasterPasswordDialog from "./components/SetMasterPasswordDialog.svelte";
  import UnlockEncryptionDialog from "./components/UnlockEncryptionDialog.svelte";
  import WalletSessionConfirmDialog from "./components/WalletSessionConfirmDialog.svelte";
  import RewardTransactionConfirmDialog from "./components/RewardTransactionConfirmDialog.svelte";

  // Types and utilities
  import type { AccountConfig } from "./lib/types";
  import { state as appState } from "./lib/state.svelte";
  import { draftManager } from "./lib/draft-manager";
  import { walletStore } from "$lib/stores/wallet.svelte";
  import { walletConnectStore } from "$lib/stores/walletconnect.svelte";

  // Handler modules
  import * as EmailOps from "./handlers/email-operations";
  import * as AccountFolder from "./handlers/account-folder";
  import * as DraftMgmt from "./handlers/draft-management";
  import * as ComposeSend from "./handlers/compose-send";
  import * as SyncIdle from "./handlers/sync-idle";

  // Page controller for initialization and lifecycle management
  import * as PageController from "./lib/page-controller.svelte";

  // Settings dialog state
  let showSettingsDialog = $state(false);
  let showAddAccountDialog = $state(false);
  let showManageAccountDialog = $state(false);

  // Encryption dialog state
  let showSetMasterPasswordDialog = $state(false);
  let showUnlockEncryptionDialog = $state(false);

  // Wallet session confirm dialog state
  let showWalletSessionConfirm = $state(false);
  let pendingWalletSession = $state<WalletSession | null>(null);

  // Reward transaction confirm dialog state
  let showRewardTransactionConfirm = $state(false);
  let pendingRewardTransaction = $state<ComposeSend.PendingRewardTransaction | null>(null);

  // Confirm delete draft dialog state
  let showConfirmDeleteDraft = $state(false);
  let draftToDelete: number | null = null;

  // Global context menu state - ensures only one context menu is open at a time
  let openContextMenuType = $state<"folder" | "email" | null>(null);
  let openContextMenuId = $state<string | number | null>(null);

  // Handle wallet session confirmation
  async function handleWalletSessionConfirm() {
    if (!pendingWalletSession) return;

    try {
      // Restore WalletConnect session
      await walletConnectStore.restoreSession();

      showWalletSessionConfirm = false;
      pendingWalletSession = null;

      // Load app after confirming wallet session
      await PageController.loadApp(handleAccountClick);
    } catch (error) {
      console.error("Failed to restore wallet session:", error);
      // Clear invalid session
      await invoke("delete_wallet_session");
      showWalletSessionConfirm = false;
      await PageController.loadApp(handleAccountClick);
    }
  }

  // Handle wallet session rejection
  async function handleWalletSessionReject() {
    try {
      // Disconnect WalletConnect
      await walletConnectStore.disconnect();

      // Also delete from keyring
      await invoke("delete_wallet_session");

      console.log("✅ Wallet disconnected successfully");
    } catch (error) {
      console.error("Failed to disconnect wallet:", error);
    }

    showWalletSessionConfirm = false;
    pendingWalletSession = null;

    // Load app after rejecting wallet session
    await PageController.loadApp(handleAccountClick);
  }

  // Lifecycle: Initialize app
  onMount(() => {
    let unlisten: (() => void) | undefined;
    let unlistenSound: (() => void) | undefined;
    let unlistenSettings: (() => void) | undefined;
    let timeUpdateTimer: ReturnType<typeof setInterval> | undefined;

    (async () => {
      try {
        // Set up event listeners FIRST (before encryption check)
        // This ensures listeners are active even if encryption is locked
        console.log("📡 Setting up event listeners...");

        // Listen for IDLE push notifications
        unlisten = await listen("idle-event", handleIdleEvent);
        console.log("✅ IDLE event listener registered");

        // Listen for notification sound event
        unlistenSound = await listen("play-notification-sound", () => {
          SyncIdle.playNotificationSound();
        });

        // Listen for open settings event from system tray
        unlistenSettings = await listen("open-settings", () => {
          showSettingsDialog = true;
        });

        // Listen for update available event
        await listen("update-available", async (event: any) => {
          const updateInfo = event.payload;
          const shouldUpdate = confirm(
            `A new version (${updateInfo.version}) is available!\n\nCurrent version: ${updateInfo.current_version}\n\nRelease notes:\n${updateInfo.body || "No release notes provided."}\n\nWould you like to download and install it now?`
          );
          if (shouldUpdate) {
            try {
              const update = await check();
              if (update) {
                await update.downloadAndInstall();
                await relaunch();
              }
            } catch (error) {
              console.error("Failed to download and install update:", error);
              alert(`Failed to install update: ${error}`);
            }
          }
        });

        // Update current time every minute
        timeUpdateTimer = setInterval(() => {
          appState.currentTime = Math.floor(Date.now() / 1000);
        }, 60000);

        // Now check encryption status
        const encryptionStatus = await invoke<{enabled: boolean, unlocked: boolean}>("get_encryption_status");

        if (!encryptionStatus.enabled) {
          // First launch - force user to set master password
          showSetMasterPasswordDialog = true;
          return; // Don't load anything until password is set
        } else if (!encryptionStatus.unlocked) {
          // Encryption enabled but locked - need to unlock
          showUnlockEncryptionDialog = true;
          return; // Don't load anything until unlocked
        }

        // Check for saved wallet session (after encryption is unlocked)
        const savedSession = await invoke<WalletSession | null>("get_wallet_session");
        if (savedSession) {
          console.log("🔐 Found saved wallet session:", savedSession);
          pendingWalletSession = savedSession;
          showWalletSessionConfirm = true;
          // Wait for user confirmation before loading app
          return;
        }

        // Encryption is enabled and unlocked, proceed normally
        await PageController.loadApp(handleAccountClick);
      } catch (e) {
        appState.error = `Failed to load accounts: ${e}`;
      }
    })();

    // Cleanup function
    return () => {
      if (unlisten) unlisten();
      if (unlistenSound) unlistenSound();
      if (unlistenSettings) unlistenSettings();
      if (timeUpdateTimer) clearInterval(timeUpdateTimer);
      // Use page controller cleanup
      PageController.cleanup();
    };
  });

  // Reload sync interval when returning from settings
  onMount(() => {
    const handleVisibilityChange = async () => {
      if (document.visibilityState === "visible") {
        try {
          const newInterval = await invoke<number>("get_sync_interval");
          if (newInterval !== appState.syncInterval) {
            await PageController.updateSyncInterval(newInterval);
          }
        } catch (e) {
          console.error("❌ Failed to reload sync interval:", e);
        }
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    window.addEventListener("focus", handleVisibilityChange);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      window.removeEventListener("focus", handleVisibilityChange);
    };
  });

  // Auto-save draft when compose state changes
  $effect(() => {
    // Watch for changes in compose fields
    const hasContent = appState.composeTo || appState.composeSubject || appState.composeBody;

    if (appState.showComposeDialog && hasContent && appState.selectedAccountId) {
      draftManager.scheduleAutoSave(async () => {
        await DraftMgmt.autoSaveDraft(appState.selectedAccountId);
      });
    }

    // Cleanup on unmount
    return () => {
      draftManager.cancelAutoSave();
    };
  });

  // ============================================
  // Wrapper functions for handlers
  // ============================================

  // Initialize app after encryption is set up
  async function initializeApp() {
    try {
      // Use page controller to initialize app with wallet session check
      const savedSession = await PageController.initializeApp(handleAccountClick);

      if (savedSession) {
        // Wallet session found, show confirmation dialog
        pendingWalletSession = savedSession;
        showWalletSessionConfirm = true;
        // Wait for user confirmation before loading app
        return;
      }

      // No wallet session, app is already loaded by controller
    } catch (e) {
      appState.error = `Failed to initialize app: ${e}`;
    }
  }

  // Account and folder handlers
  async function handleAccountClick(accountId: number) {
    await AccountFolder.handleAccountClick(
      accountId,
      appState.accounts,
      appState.syncInterval,
      loadEmailsForFolder
    );
  }

  async function loadEmailsForFolder(folderName: string) {
    await AccountFolder.loadEmailsForFolder(
      folderName,
      appState.accounts,
      appState.selectedAccountId,
      appState.syncInterval
    );
  }

  async function handleFolderClick(folderName: string) {
    await AccountFolder.handleFolderClick(
      folderName,
      appState.accounts,
      appState.selectedAccountId,
      appState.selectedFolderName,
      appState.syncInterval
    );
  }

  async function handleFolderCreated() {
    await AccountFolder.handleFolderCreated(appState.selectedAccountId);
  }

  async function handleFolderDeleted() {
    await AccountFolder.handleFolderDeleted(
      appState.selectedAccountId,
      appState.selectedFolderName,
      appState.accounts,
      appState.syncInterval
    );
  }

  async function handleAccountAdded() {
    await AccountFolder.handleAccountAdded(
      appState.accounts,
      appState.selectedAccountId,
      appState.syncInterval,
      loadEmailsForFolder,
      handleAccountClick
    );
  }

  async function handleAccountDeleted(email: string) {
    await AccountFolder.handleAccountDeleted(
      email,
      appState.selectedAccountId,
      appState.syncInterval,
      loadEmailsForFolder,
      handleAccountClick
    );
  }

  async function handleAccountUpdated() {
    await AccountFolder.handleAccountUpdated();
  }

  // Email operation handlers
  async function handleEmailClick(uid: number) {
    await EmailOps.handleEmailClick(
      uid,
      appState.accounts,
      appState.selectedAccountId,
      appState.selectedFolderName
    );
  }

  async function handleToggleReadStatus() {
    await EmailOps.handleToggleReadStatus(
      appState.accounts,
      appState.selectedAccountId,
      appState.selectedEmailUid,
      appState.selectedFolderName,
      appState.emails
    );
  }

  async function handleMarkEmailAsRead(uid: number) {
    await EmailOps.handleMarkEmailAsRead(
      uid,
      appState.accounts,
      appState.selectedAccountId,
      appState.selectedFolderName,
      appState.emails
    );
  }

  async function handleMarkEmailAsUnread(uid: number) {
    await EmailOps.handleMarkEmailAsUnread(
      uid,
      appState.accounts,
      appState.selectedAccountId,
      appState.selectedFolderName,
      appState.emails
    );
  }

  async function handleStarToggle(uid: number, flagged: boolean) {
    await EmailOps.handleStarToggle(
      uid,
      flagged,
      appState.accounts,
      appState.selectedAccountId,
      appState.selectedFolderName,
      appState.emails
    );
  }

  async function handleDeleteEmail() {
    await EmailOps.handleDeleteEmail(
      appState.accounts,
      appState.selectedAccountId,
      appState.selectedEmailUid,
      appState.selectedFolderName,
      appState.emails,
      loadEmailsForFolder
    );
  }

  async function handleDeleteEmailFromContextMenu(uid: number) {
    await EmailOps.handleDeleteEmailFromContextMenu(
      uid,
      appState.accounts,
      appState.selectedAccountId,
      appState.selectedFolderName,
      appState.emails,
      loadEmailsForFolder
    );
  }

  async function downloadAttachment(attachmentId: number, filename: string) {
    await EmailOps.downloadAttachment(attachmentId, filename);
  }

  function handlePageChange(page: number) {
    EmailOps.handlePageChange(page);
  }

  async function handleVerifyOnChain() {
    await EmailOps.handleVerifyOnChain(appState.selectedEmailUid, appState.emails);
  }

  // Draft management handlers
  async function loadDrafts() {
    await DraftMgmt.loadDrafts(appState.selectedAccountId);
  }

  async function handleSaveDraft() {
    await DraftMgmt.handleSaveDraft(appState.selectedAccountId, loadDrafts);
  }

  function handleCloseCompose() {
    DraftMgmt.handleCloseCompose(appState.selectedAccountId);
  }

  function handleDiscardDraft() {
    DraftMgmt.handleDiscardDraft();
  }

  function handleCancelSaveDraft() {
    DraftMgmt.handleCancelSaveDraft();
  }

  async function handleSaveDraftAndClose() {
    await DraftMgmt.handleSaveDraftAndClose(appState.selectedAccountId, loadDrafts);
  }

  async function handleDraftClick(draftId: number) {
    await DraftMgmt.handleDraftClick(draftId, updateAttachmentSizeLimit);
  }

  function handleDraftDelete(draftId: number) {
    draftToDelete = DraftMgmt.handleDraftDelete(draftId);
    showConfirmDeleteDraft = true;
  }

  async function confirmDeleteDraft() {
    await DraftMgmt.confirmDeleteDraft(draftToDelete, loadDrafts);
    showConfirmDeleteDraft = false;
    draftToDelete = null;
  }

  function cancelDeleteDraft() {
    showConfirmDeleteDraft = false;
    draftToDelete = null;
  }

  async function handleShowDrafts() {
    await DraftMgmt.handleShowDrafts(loadDrafts);
  }

  function handleHideDrafts() {
    DraftMgmt.handleHideDrafts();
  }

  // Compose and send handlers
  async function handleComposeClick() {
    await ComposeSend.handleComposeClick(appState.selectedAccountId, updateAttachmentSizeLimit);
  }

  async function handleReplyClick() {
    await ComposeSend.handleReplyClick(
      appState.selectedAccountId,
      appState.selectedEmailUid,
      appState.emails,
      updateAttachmentSizeLimit
    );
  }

  async function handleForwardClick() {
    await ComposeSend.handleForwardClick(
      appState.selectedAccountId,
      appState.selectedEmailUid,
      appState.emails,
      updateAttachmentSizeLimit
    );
  }

  function handleAttachmentSelect(event: Event) {
    ComposeSend.handleAttachmentSelect(event);
  }

  function removeAttachment(index: number) {
    ComposeSend.removeAttachment(index);
  }

  async function updateAttachmentSizeLimit() {
    await ComposeSend.updateAttachmentSizeLimit(appState.selectedAccountId, appState.accounts);
  }

  async function handleSendEmail() {
    const pendingReward = await ComposeSend.handleSendEmail(
      appState.selectedAccountId,
      appState.selectedEmailUid,
      appState.accounts,
      appState.emails,
      appState.emailBody,
      loadDrafts
    );

    // If there's a pending reward transaction, show confirmation dialog
    if (pendingReward) {
      pendingRewardTransaction = pendingReward;
      showRewardTransactionConfirm = true;
    }
  }

  // Handle reward transaction confirmation
  async function handleRewardTransactionConfirm() {
    if (!pendingRewardTransaction) return;

    try {
      await ComposeSend.executeRewardTransaction(pendingRewardTransaction);
      showRewardTransactionConfirm = false;
      pendingRewardTransaction = null;
    } catch (error) {
      console.error("Failed to execute reward transaction:", error);
      // Dialog will remain open, user can try again or cancel
    }
  }

  // Handle reward transaction cancellation
  function handleRewardTransactionCancel() {
    showRewardTransactionConfirm = false;
    pendingRewardTransaction = null;
  }

  // Sync and IDLE handlers
  async function handleManualRefresh() {
    await SyncIdle.handleManualRefresh();
  }

  async function handleIdleEvent(event: { payload: any }) {
    await SyncIdle.handleIdleEvent(event);
  }
</script>

<Sidebar.Provider style="--sidebar-width: 350px;">
  <Sidebar.Root collapsible="icon" class="overflow-hidden [&>[data-sidebar=sidebar]]:flex-row">
    <AccountFolderSidebar
      accounts={appState.accounts}
      selectedAccountId={appState.selectedAccountId}
      folders={appState.folders}
      selectedFolderName={appState.selectedFolderName}
      isLoadingFolders={appState.isLoadingFolders}
      isSyncing={appState.isSyncing}
      showDraftsFolder={appState.showDraftsFolder}
      onAccountSelect={handleAccountClick}
      onFolderClick={handleFolderClick}
      onAddAccount={() => (showAddAccountDialog = true)}
      onManageAccounts={() => (showManageAccountDialog = true)}
      onSettings={() => (showSettingsDialog = true)}
      onSyncMail={handleManualRefresh}
      onComposeClick={handleComposeClick}
      onShowDrafts={handleShowDrafts}
      onFolderCreated={handleFolderCreated}
      onFolderDeleted={handleFolderDeleted}
      {openContextMenuType}
      {openContextMenuId}
      onContextMenuChange={(type, id) => {
        openContextMenuType = type;
        openContextMenuId = id;
      }}
    />

    {#if appState.showDraftsFolder}
      <DraftsList
        drafts={appState.drafts}
        selectedDraftId={appState.currentDraftId}
        isLoading={appState.isLoadingDrafts}
        onDraftClick={handleDraftClick}
        onDraftDelete={handleDraftDelete}
      />
    {:else}
      <EmailListSidebar
        emails={appState.emails}
        selectedEmailUid={appState.selectedEmailUid}
        isLoading={appState.isLoadingEmails}
        error={appState.error}
        selectedAccountId={appState.selectedAccountId}
        selectedFolderName={appState.selectedFolderName}
        folders={appState.folders}
        currentUserEmail={appState.accounts.find((acc) => acc.id === appState.selectedAccountId)?.email || ""}
        currentPage={appState.currentPage}
        pageSize={appState.pageSize}
        onEmailClick={handleEmailClick}
        onPageChange={handlePageChange}
        onStarToggle={handleStarToggle}
        onMarkAsRead={handleMarkEmailAsRead}
        onMarkAsUnread={handleMarkEmailAsUnread}
        onDeleteEmail={handleDeleteEmailFromContextMenu}
        {openContextMenuType}
        {openContextMenuId}
        onContextMenuChange={(type, id) => {
          openContextMenuType = type;
          openContextMenuId = id;
        }}
      />
    {/if}
  </Sidebar.Root>

  <Sidebar.Inset class="flex flex-col">
    <header class="sticky top-0 flex shrink-0 items-center gap-2 border-b bg-background p-4 z-10">
      <Sidebar.Trigger class="-ml-1" />
    </header>
    <div class="flex-1 overflow-hidden">
      <EmailBody
        email={appState.selectedEmail}
        body={appState.emailBody}
        attachments={appState.attachments}
        isLoadingBody={appState.isLoadingBody}
        isLoadingAttachments={appState.isLoadingAttachments}
        error={appState.error}
        cmvhVerification={appState.cmvhVerification}
        onReply={handleReplyClick}
        onForward={handleForwardClick}
        onDelete={handleDeleteEmail}
        onDownloadAttachment={downloadAttachment}
        onToggleRead={handleToggleReadStatus}
        onVerifyOnChain={handleVerifyOnChain}
      />
    </div>
  </Sidebar.Inset>

  <ComposeDialog
    show={appState.showComposeDialog}
    mode={appState.isReplyMode ? "reply" : appState.isForwardMode ? "forward" : "compose"}
    bind:to={appState.composeTo}
    bind:cc={appState.composeCc}
    bind:subject={appState.composeSubject}
    bind:body={appState.composeBody}
    bind:attachments={appState.composeAttachments}
    bind:enableCMVHSigning={appState.enableCMVHSigning}
    attachmentSizeLimit={appState.attachmentSizeLimit}
    totalAttachmentSize={appState.totalAttachmentSize}
    isSending={appState.isSending}
    isDraft={appState.currentDraftId !== null}
    error={appState.error}
    onSend={handleSendEmail}
    onCancel={handleCloseCompose}
    onAttachmentAdd={handleAttachmentSelect}
    onAttachmentRemove={removeAttachment}
  />

  <SaveDraftDialog
    show={appState.showSaveDraftDialog}
    onSave={handleSaveDraftAndClose}
    onDiscard={handleDiscardDraft}
    onCancel={handleCancelSaveDraft}
  />

  <SettingsDialog bind:open={showSettingsDialog} onOpenChange={(open) => (showSettingsDialog = open)} />

  <AddAccountDialog
    bind:open={showAddAccountDialog}
    onOpenChange={(open) => (showAddAccountDialog = open)}
    onAccountAdded={handleAccountAdded}
  />

  <ManageAccountDialog
    bind:open={showManageAccountDialog}
    accounts={appState.accounts}
    onAccountDeleted={handleAccountDeleted}
    onAccountUpdated={handleAccountUpdated}
    onAddAccount={() => {
      showManageAccountDialog = false;
      showAddAccountDialog = true;
    }}
  />

  <ConfirmDialog
    bind:open={showConfirmDeleteDraft}
    title="Delete Draft"
    description="Are you sure you want to delete this draft? This action cannot be undone."
    confirmText="Delete"
    cancelText="Cancel"
    variant="destructive"
    onConfirm={confirmDeleteDraft}
    onCancel={cancelDeleteDraft}
  />

  <SetMasterPasswordDialog
    bind:open={showSetMasterPasswordDialog}
    onpasswordset={initializeApp}
  />

  <UnlockEncryptionDialog
    bind:open={showUnlockEncryptionDialog}
    onunlock={initializeApp}
  />

  <WalletSessionConfirmDialog
    bind:open={showWalletSessionConfirm}
    session={pendingWalletSession}
    onConfirm={handleWalletSessionConfirm}
    onReject={handleWalletSessionReject}
  />

  <RewardTransactionConfirmDialog
    bind:open={showRewardTransactionConfirm}
    amount={pendingRewardTransaction?.amount || "0"}
    recipientAddress={pendingRewardTransaction?.recipientAddress || ""}
    recipientEmail={pendingRewardTransaction?.recipientEmail || ""}
    emailSubject={pendingRewardTransaction?.emailSubject || ""}
    onConfirm={handleRewardTransactionConfirm}
    onCancel={handleRewardTransactionCancel}
  />
</Sidebar.Provider>
