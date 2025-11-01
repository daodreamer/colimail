<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { supabase, getAuthErrorMessage, validatePassword } from "$lib/supabase";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";

  let password = $state("");
  let confirmPassword = $state("");
  let loading = $state(false);
  let error = $state("");
  let success = $state(false);

  onMount(() => {
    // Check if we have a valid recovery token
    const hashParams = new URLSearchParams(window.location.hash.substring(1));
    const accessToken = hashParams.get("access_token");
    const type = hashParams.get("type");

    if (!accessToken || type !== "recovery") {
      error = "Invalid or expired password reset link.";
    }
  });

  async function handleResetPassword(e: Event) {
    e.preventDefault();
    error = "";

    // Validation
    if (!password || !confirmPassword) {
      error = "Please fill in all fields";
      return;
    }

    // Validate password strength
    const passwordValidation = validatePassword(password);
    if (!passwordValidation.valid) {
      error = passwordValidation.message;
      return;
    }

    if (password !== confirmPassword) {
      error = "Passwords do not match";
      return;
    }

    try {
      loading = true;

      // Update the user's password
      const { error: updateError } = await supabase.auth.updateUser({
        password: password,
      });

      if (updateError) throw updateError;

      success = true;

      // Redirect to login after a short delay
      setTimeout(() => {
        goto("/auth/login");
      }, 2000);
    } catch (err: any) {
      error = getAuthErrorMessage(err);
      console.error("Password reset error:", err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex min-h-svh flex-col items-center justify-center p-6">
  <div class="w-full max-w-md">
    <Card.Root>
      <Card.Header class="text-center">
        <Card.Title class="text-xl">Reset Your Password</Card.Title>
        <Card.Description>Enter your new password below</Card.Description>
      </Card.Header>
      <Card.Content>
        <form onsubmit={handleResetPassword}>
          <div class="space-y-4">
            {#if error}
              <div class="bg-destructive/15 text-destructive rounded-md p-3 text-sm">
                {error}
              </div>
            {/if}

            {#if success}
              <div class="bg-green-500/15 text-green-700 dark:text-green-400 rounded-md p-3 text-sm">
                Password updated successfully! Redirecting to login...
              </div>
            {/if}

            <div class="space-y-2">
              <Label for="password">New Password</Label>
              <Input
                id="password"
                type="password"
                required
                bind:value={password}
                disabled={loading || success}
                placeholder="Enter new password"
              />
              <p class="text-xs text-muted-foreground">
                Must be at least 8 characters and include lowercase, uppercase, digits, and symbols.
              </p>
            </div>

            <div class="space-y-2">
              <Label for="confirm-password">Confirm Password</Label>
              <Input
                id="confirm-password"
                type="password"
                required
                bind:value={confirmPassword}
                disabled={loading || success}
                placeholder="Confirm new password"
              />
            </div>

            <div class="space-y-2">
              <Button type="submit" disabled={loading || success} class="w-full">
                {loading ? "Updating Password..." : success ? "Password Updated!" : "Update Password"}
              </Button>
            </div>
          </div>
        </form>
      </Card.Content>
    </Card.Root>
  </div>
</div>
