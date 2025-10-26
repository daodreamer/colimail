<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { AccountConfig } from "../lib/types";
  import { formatTimeSince } from "../lib/utils";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Badge } from "$lib/components/ui/badge";
  import { Separator } from "$lib/components/ui/separator";
  import * as ButtonGroup from "$lib/components/ui/button-group";

  // Props
  let {
    accounts = [] as AccountConfig[],
    selectedAccountId = null as number | null,
    isSyncing = false,
    lastSyncTime = 0,
    currentTime = 0,
    onAccountClick,
    onCompose,
    onRefresh,
    onDeleteAccount,
    onManageAccounts,
  }: {
    accounts?: AccountConfig[];
    selectedAccountId?: number | null;
    isSyncing?: boolean;
    lastSyncTime?: number;
    currentTime?: number;
    onAccountClick: (accountId: number) => void;
    onCompose: () => void;
    onRefresh: () => void;
    onDeleteAccount: (email: string, event: MouseEvent) => void;
    onManageAccounts: () => void;
  } = $props();
</script>

<aside class="flex h-screen flex-col border-r bg-muted/40">
  <div class="border-b p-4">
    <h2 class="text-lg font-semibold">Accounts</h2>
  </div>

  <ScrollArea class="flex-1 px-3 py-2">
    <div class="space-y-1">
      {#each accounts as account (account.id)}
        <div class="flex items-center gap-1">
          <Button
            variant={account.id === selectedAccountId ? "default" : "ghost"}
            class="flex-1 justify-start gap-2 overflow-hidden"
            onclick={() => onAccountClick(account.id)}
          >
            <span class="truncate text-sm">{account.email}</span>
            {#await invoke("is_idle_active", { accountId: account.id, folderName: "INBOX" })}
              <Badge variant="outline" class="ml-auto shrink-0">⚪</Badge>
            {:then isActive}
              {#if isActive}
                <Badge variant="default" class="ml-auto shrink-0 animate-pulse bg-green-500 hover:bg-green-600">
                  🟢
                </Badge>
              {:else}
                <Badge variant="destructive" class="ml-auto shrink-0 opacity-50">🔴</Badge>
              {/if}
            {:catch}
              <Badge variant="outline" class="ml-auto shrink-0">⚪</Badge>
            {/await}
          </Button>
          <Button
            variant="ghost"
            size="icon"
            class="h-9 w-9 shrink-0 text-muted-foreground hover:bg-destructive hover:text-destructive-foreground"
            onclick={(e) => onDeleteAccount(account.email, e)}
            title="Delete account {account.email}"
          >
            ×
          </Button>
        </div>
      {/each}
      {#if accounts.length === 0}
        <p class="py-8 text-center text-sm text-muted-foreground">No accounts configured.</p>
      {/if}
    </div>
  </ScrollArea>

  <div class="space-y-3 p-3">
    <ButtonGroup.Root orientation="vertical" class="w-full">
      <Button
        variant="default"
        class="w-full justify-start"
        onclick={onCompose}
        disabled={!selectedAccountId}
      >
        <span class="text-base">✉️</span>
        <span class="ml-2">Compose</span>
      </Button>

      <Button
        variant="default"
        class="w-full justify-start"
        onclick={onRefresh}
        disabled={!selectedAccountId || isSyncing}
      >
        <span class="text-base">🔄</span>
        <span class="ml-2">{isSyncing ? "Syncing..." : "Refresh"}</span>
      </Button>
    </ButtonGroup.Root>

    {#if selectedAccountId && lastSyncTime > 0}
      <p class="text-center text-xs text-muted-foreground">
        Last sync: {formatTimeSince(lastSyncTime, currentTime)}
      </p>
    {/if}

    <Separator />

    <ButtonGroup.Root orientation="vertical" class="w-full">
      <Button variant="outline" class="w-full justify-start" href="/account">
        <span class="text-base">+</span>
        <span class="ml-2">Add Account</span>
      </Button>
      <Button variant="outline" class="w-full justify-start" onclick={onManageAccounts}>
        <span class="text-base">⚙️</span>
        <span class="ml-2">Manage Account</span>
      </Button>
      <Button variant="outline" class="w-full justify-start" href="/settings">
        <span class="text-base">⚙️</span>
        <span class="ml-2">Settings</span>
      </Button>
    </ButtonGroup.Root>
  </div>
</aside>

