<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { supabase, getAuthErrorMessage } from "$lib/supabase";
  import { authStore } from "$lib/stores/auth.svelte";

  let status = $state<"processing" | "success" | "error" | "browser_warning">("processing");
  let message = $state("Processing authentication...");
  let isTauriEnv = $state(false);

  onMount(async () => {
    // Check if running in Tauri environment
    isTauriEnv = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

    // If opened in browser (e.g., from email verification link), show a friendly message
    if (!isTauriEnv) {
      console.log('[Callback] Not in Tauri environment, showing browser warning');
      status = "browser_warning";
      message = "Please open the Colimail desktop application to complete the verification process.";

      // Try to open in desktop app using deep link
      const fullUrl = window.location.href;
      const deepLink = fullUrl.replace(/^https?:\/\/[^/]+/, 'colimail://');
      console.log('[Callback] Attempting to open deep link:', deepLink);

      // Attempt to trigger deep link
      window.location.href = deepLink;

      return;
    }

    try {
      console.log('[Callback] OAuth callback page mounted in Tauri environment');

      // Check for OAuth errors in URL first
      const hashParams = new URLSearchParams(window.location.hash.substring(1));
      const error = hashParams.get("error");
      const errorDescription = hashParams.get("error_description");

      if (error) {
        throw new Error(errorDescription || error);
      }

      // Supabase v2 automatically handles the OAuth callback through detectSessionInUrl
      // The auth state change listener will be triggered automatically
      // We just need to wait a bit for the session to be established
      await new Promise(resolve => setTimeout(resolve, 1000));

      // Verify session was established
      const { data: { session }, error: sessionError } = await supabase.auth.getSession();

      console.log('[Callback] Session check:', session ? `User: ${session.user.email}` : 'No session', sessionError);

      if (sessionError) {
        throw sessionError;
      }

      if (!session) {
        throw new Error("Failed to establish authentication session. Please try again.");
      }

      console.log('[Callback] Session found, authentication successful');
      status = "success";
      message = "Authentication successful! Redirecting...";

      // Wait for authStore to sync (the onAuthStateChange listener should have triggered)
      await new Promise(resolve => setTimeout(resolve, 500));

      // Force refresh the auth state to ensure everything is in sync
      console.log('[Callback] Refreshing auth store...');
      await authStore.refreshUser();
      console.log('[Callback] Auth store refreshed, isAuthenticated:', authStore.isAuthenticated);

      // Redirect to main app
      setTimeout(() => {
        console.log('[Callback] Redirecting to main app');
        goto("/", { replaceState: true }); // Use replaceState to prevent back button issues
      }, 1000);
    } catch (err: any) {
      status = "error";
      message = getAuthErrorMessage(err);
      console.error("OAuth callback error:", err);

      // Redirect to login after showing error
      setTimeout(() => {
        goto("/auth/login", { replaceState: true });
      }, 3000);
    }
  });
</script>

<div class="flex min-h-svh flex-col items-center justify-center p-6 bg-background">
  <div class="w-full max-w-md text-center space-y-4">
    {#if status === "processing"}
      <div class="mb-4">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
      </div>
      <h2 class="text-xl font-semibold mb-2">Authenticating...</h2>
      <p class="text-muted-foreground">{message}</p>
    {:else if status === "success"}
      <div class="mb-4 text-green-600 dark:text-green-400">
        <svg
          class="inline-block h-12 w-12"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M5 13l4 4L19 7"
          />
        </svg>
      </div>
      <h2 class="text-xl font-semibold mb-2 text-green-600 dark:text-green-400">Success!</h2>
      <p class="text-muted-foreground">{message}</p>
    {:else if status === "browser_warning"}
      <div class="mb-4 text-blue-600 dark:text-blue-400">
        <svg
          class="inline-block h-16 w-16"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
          />
        </svg>
      </div>
      <h2 class="text-2xl font-semibold mb-4">Email Verification</h2>
      <div class="bg-muted/50 rounded-lg p-6 space-y-4 text-left">
        <p class="text-foreground">
          <strong>Your email has been verified successfully!</strong>
        </p>
        <p class="text-muted-foreground">
          To complete the sign-up process, please:
        </p>
        <ol class="list-decimal list-inside space-y-2 text-muted-foreground ml-2">
          <li>Open the <strong class="text-foreground">Colimail desktop application</strong></li>
          <li>Sign in with your email and password</li>
        </ol>
        <div class="pt-4 border-t border-border">
          <p class="text-sm text-muted-foreground">
            <strong>Note:</strong> This verification link is meant to be opened in the desktop application.
            If the app didn't open automatically, please launch it manually.
          </p>
        </div>
      </div>
      <p class="text-sm text-muted-foreground mt-4">
        You can safely close this browser window.
      </p>
    {:else}
      <div class="mb-4 text-destructive">
        <svg
          class="inline-block h-12 w-12"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      </div>
      <h2 class="text-xl font-semibold mb-2 text-destructive">Error</h2>
      <p class="text-muted-foreground">{message}</p>
    {/if}
  </div>
</div>
