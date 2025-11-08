<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import KeyIcon from "@lucide/svelte/icons/key";

  interface Props {
    open: boolean;
    onunlock?: () => void;
  }

  let { open = $bindable(), onunlock }: Props = $props();

  let password = $state("");
  let error = $state("");
  let isLoading = $state(false);

  async function handleUnlock() {
    if (!password) {
      error = "Please enter your password";
      return;
    }

    isLoading = true;
    error = "";

    try {
      await invoke("unlock_encryption_with_password", { password });

      // Clear password field
      password = "";

      // Close dialog and notify parent
      open = false;
      if (onunlock) {
        onunlock();
      }
    } catch (e) {
      error = String(e);
      password = "";
    } finally {
      isLoading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !isLoading) {
      handleUnlock();
    }
  }
</script>

<Dialog.Root bind:open onOpenChange={(isOpen) => {
  // Prevent closing by user action - must unlock first
  if (!isOpen) {
    open = true;
  }
}}>
  <Dialog.Content class="sm:max-w-[425px]">
    <Dialog.Header>
      <div class="flex items-center gap-2">
        <KeyIcon class="h-5 w-5" />
        <Dialog.Title>Unlock Encryption</Dialog.Title>
      </div>
      <Dialog.Description>
        Enter your master password to unlock email encryption.
      </Dialog.Description>
    </Dialog.Header>

    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="unlock-password">Master Password</Label>
        <Input
          id="unlock-password"
          type="password"
          placeholder="Enter your master password"
          bind:value={password}
          disabled={isLoading}
          onkeydown={handleKeydown}
          autofocus
        />
      </div>

      {#if error}
        <div class="text-sm text-destructive">
          {error}
        </div>
      {/if}
    </div>

    <Dialog.Footer>
      <Button
        onclick={handleUnlock}
        disabled={isLoading || !password}
        class="w-full"
      >
        {isLoading ? "Unlocking..." : "Unlock"}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
