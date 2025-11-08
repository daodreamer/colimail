<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import LockIcon from "@lucide/svelte/icons/lock";

  interface Props {
    open: boolean;
    onpasswordset?: () => void;
  }

  let { open = $bindable(), onpasswordset }: Props = $props();

  let password = $state("");
  let confirmPassword = $state("");
  let error = $state("");
  let isLoading = $state(false);

  async function handleSetPassword() {
    // Validation
    if (password.length < 8) {
      error = "Password must be at least 8 characters long";
      return;
    }

    if (password !== confirmPassword) {
      error = "Passwords do not match";
      return;
    }

    isLoading = true;
    error = "";

    try {
      await invoke("enable_encryption", { password });

      // Clear password fields
      password = "";
      confirmPassword = "";

      // Close dialog and notify parent
      open = false;
      if (onpasswordset) {
        onpasswordset();
      }
    } catch (e) {
      error = String(e);
    } finally {
      isLoading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !isLoading) {
      handleSetPassword();
    }
  }
</script>

<Dialog.Root bind:open onOpenChange={(isOpen) => {
  // Prevent closing when loading or by user action
  if (!isOpen) {
    open = true;
  }
}}>
  <Dialog.Content class="sm:max-w-[425px]">
    <Dialog.Header>
      <div class="flex items-center gap-2">
        <LockIcon class="h-5 w-5" />
        <Dialog.Title>Set Master Password</Dialog.Title>
      </div>
      <Dialog.Description>
        Create a master password to encrypt your email data. This password will be required each time you start the application.
      </Dialog.Description>
    </Dialog.Header>

    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="password">Master Password</Label>
        <Input
          id="password"
          type="password"
          placeholder="Enter master password (min 8 characters)"
          bind:value={password}
          disabled={isLoading}
          onkeydown={handleKeydown}
          autofocus
        />
      </div>

      <div class="grid gap-2">
        <Label for="confirm-password">Confirm Password</Label>
        <Input
          id="confirm-password"
          type="password"
          placeholder="Confirm master password"
          bind:value={confirmPassword}
          disabled={isLoading}
          onkeydown={handleKeydown}
        />
      </div>

      {#if error}
        <div class="text-sm text-destructive">
          {error}
        </div>
      {/if}

      <div class="text-sm text-muted-foreground">
        <strong>Important:</strong> There is no password recovery option. Make sure to remember your password.
      </div>
    </div>

    <Dialog.Footer>
      <Button
        onclick={handleSetPassword}
        disabled={isLoading || !password || !confirmPassword}
        class="w-full"
      >
        {isLoading ? "Setting up..." : "Set Master Password"}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
