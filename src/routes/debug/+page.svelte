<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import type { AccountConfig } from "../lib/types";

  let accounts: AccountConfig[] = $state([]);
  let selectedAccountId: number | null = $state(null);
  let uid: string = $state("32042");
  let folder: string = $state("INBOX");
  let debugData: any = $state(null);
  let error: string | null = $state(null);
  let isLoading: boolean = $state(false);

  async function loadAccounts() {
    try {
      accounts = await invoke<AccountConfig[]>("load_account_configs");
      if (accounts.length > 0 && !selectedAccountId) {
        selectedAccountId = accounts[0].id;
      }
    } catch (e) {
      error = `Failed to load accounts: ${e}`;
    }
  }

  async function fetchDebugData() {
    if (!selectedAccountId) {
      error = "Please select an account";
      return;
    }

    const selectedConfig = accounts.find((acc) => acc.id === selectedAccountId);
    if (!selectedConfig) {
      error = "Could not find selected account configuration";
      return;
    }

    isLoading = true;
    error = null;
    debugData = null;

    try {
      debugData = await invoke("debug_fetch_email_raw", {
        config: selectedConfig,
        uid: parseInt(uid),
        folder: folder || null,
      });
    } catch (e) {
      error = `Failed to fetch debug data: ${e}`;
    } finally {
      isLoading = false;
    }
  }

  loadAccounts();
</script>

<div class="container mx-auto p-8">
  <Card>
    <CardHeader>
      <CardTitle>Email Debug Tool</CardTitle>
      <p class="text-sm text-muted-foreground">
        Fetch raw email data for troubleshooting display issues
      </p>
    </CardHeader>
    <CardContent class="space-y-4">
      <div>
        <label for="account" class="block text-sm font-medium mb-2">Account</label>
        <select
          id="account"
          bind:value={selectedAccountId}
          class="w-full rounded-md border border-input bg-background px-3 py-2"
        >
          {#each accounts as account}
            <option value={account.id}>{account.email}</option>
          {/each}
        </select>
      </div>

      <div>
        <label for="uid" class="block text-sm font-medium mb-2">Email UID</label>
        <Input id="uid" bind:value={uid} placeholder="Enter email UID" />
      </div>

      <div>
        <label for="folder" class="block text-sm font-medium mb-2">Folder</label>
        <Input id="folder" bind:value={folder} placeholder="INBOX" />
      </div>

      <Button onclick={fetchDebugData} disabled={isLoading}>
        {isLoading ? "Loading..." : "Fetch Debug Data"}
      </Button>

      {#if error}
        <div class="rounded-md bg-destructive/10 p-4 text-sm text-destructive">
          {error}
        </div>
      {/if}

      {#if debugData}
        <Card>
          <CardHeader>
            <CardTitle class="text-lg">Debug Data</CardTitle>
          </CardHeader>
          <CardContent>
            <ScrollArea class="h-[600px]">
              <pre class="text-xs whitespace-pre-wrap break-words">{JSON.stringify(
                  debugData,
                  null,
                  2
                )}</pre>
            </ScrollArea>
          </CardContent>
        </Card>
      {/if}

      <div class="pt-4">
        <Button variant="outline" onclick={() => (window.location.href = "/")}>
          ← Back to Inbox
        </Button>
      </div>
    </CardContent>
  </Card>
</div>
