<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { supabase, getAuthErrorMessage } from "$lib/supabase";
  import { authStore } from "$lib/stores/auth.svelte";

  let status = $state<"processing" | "success" | "error" | "browser_info">("processing");
  let message = $state("Verifying your email...");
  let isTauriEnv = $state(false);

  onMount(async () => {
    // Check if running in Tauri environment
    isTauriEnv = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

    try {
      console.log('[Verify] Email verification page mounted');
      console.log('[Verify] Tauri environment:', isTauriEnv);

      // Check for errors in URL
      const hashParams = new URLSearchParams(window.location.hash.substring(1));
      const queryParams = new URLSearchParams(window.location.search);

      const error = hashParams.get("error") || queryParams.get("error");
      const errorDescription = hashParams.get("error_description") || queryParams.get("error_description");

      if (error) {
        throw new Error(errorDescription || error);
      }

      // Supabase v2 automatically handles email verification through detectSessionInUrl
      // Wait for the session to be established
      await new Promise(resolve => setTimeout(resolve, 1500));

      // Verify session was established
      const { data: { session }, error: sessionError } = await supabase.auth.getSession();

      console.log('[Verify] Session check:', session ? `User: ${session.user.email}` : 'No session', sessionError);

      if (sessionError) {
        throw sessionError;
      }

      if (!session) {
        // If no session, this might be opened in browser
        // Show info message instead of error
        if (!isTauriEnv) {
          console.log('[Verify] No session in browser, showing info message');
          status = "browser_info";
          message = "Email verified! Please open the desktop app to sign in.";
          return;
        }
        throw new Error("Failed to verify email. Please try again.");
      }

      console.log('[Verify] Email verified successfully');
      status = "success";
      message = isTauriEnv
        ? "Email verified successfully! Redirecting to app..."
        : "Email verified successfully! Please open the desktop app to continue.";

      // Only proceed with app logic if in Tauri environment
      if (isTauriEnv) {
        // Wait for authStore to sync
        await new Promise(resolve => setTimeout(resolve, 500));

        // Refresh auth state
        console.log('[Verify] Refreshing auth store...');
        await authStore.refreshUser();
        console.log('[Verify] Auth store refreshed, isAuthenticated:', authStore.isAuthenticated);

        // Redirect to main app
        setTimeout(() => {
          console.log('[Verify] Redirecting to main app');
          goto("/", { replaceState: true });
        }, 1500);
      } else {
        // In browser, try to open desktop app via deep link
        const fullUrl = window.location.href;
        const deepLink = fullUrl.replace(/^https?:\/\/[^/]+/, 'colimail://');
        console.log('[Verify] Attempting to open desktop app:', deepLink);

        // Try to open deep link
        setTimeout(() => {
          window.location.href = deepLink;
        }, 2000);
      }
    } catch (err: any) {
      status = "error";
      message = getAuthErrorMessage(err);
      console.error("Email verification error:", err);

      // Redirect to login after showing error
      if (isTauriEnv) {
        setTimeout(() => {
          goto("/auth/login", { replaceState: true });
        }, 3000);
      }
    }
  });
</script>

<div class="flex min-h-svh flex-col items-center justify-center p-6 bg-background">
  <div class="w-full max-w-md text-center space-y-4">
    {#if status === "processing"}
      <div class="mb-4">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
      </div>
      <h2 class="text-xl font-semibold mb-2">Verifying Email...</h2>
      <p class="text-muted-foreground">{message}</p>
    {:else if status === "success"}
      <div class="mb-4 text-green-600 dark:text-green-400">
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
            d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      </div>
      <h2 class="text-2xl font-semibold mb-4 text-green-600 dark:text-green-400">
        Email Verified!
      </h2>

      {#if isTauriEnv}
        <p class="text-muted-foreground mb-4">
          Your email has been verified successfully. Redirecting to the application...
        </p>
        <div class="mt-4">
          <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-green-600"></div>
        </div>
      {:else}
        <div class="bg-muted/50 rounded-lg p-6 space-y-4 text-left">
          <p class="text-foreground">
            <strong>Your email has been verified successfully!</strong>
          </p>
          <p class="text-muted-foreground">
            To continue using Colimail, please:
          </p>
          <ol class="list-decimal list-inside space-y-2 text-muted-foreground ml-2">
            <li>Open the <strong class="text-foreground">Colimail desktop application</strong></li>
            <li>Sign in with your email and password</li>
          </ol>
          <div class="pt-4 border-t border-border">
            <p class="text-sm text-muted-foreground">
              If the app didn't open automatically, please launch it manually from your applications folder.
            </p>
          </div>
        </div>
        <p class="text-sm text-muted-foreground mt-4">
          You can safely close this browser window.
        </p>
      {/if}
    {:else if status === "browser_info"}
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
            d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      </div>
      <h2 class="text-2xl font-semibold mb-4 text-blue-600 dark:text-blue-400">
        Email Verified!
      </h2>
      <div class="bg-muted/50 rounded-lg p-6 space-y-4 text-left">
        <p class="text-foreground">
          <strong>Your email has been verified successfully!</strong>
        </p>
        <p class="text-muted-foreground">
          To start using Colimail, please:
        </p>
        <ol class="list-decimal list-inside space-y-2 text-muted-foreground ml-2">
          <li>Open the <strong class="text-foreground">Colimail desktop application</strong></li>
          <li>Sign in with your email and password</li>
        </ol>
        <div class="pt-4 border-t border-border">
          <p class="text-sm text-muted-foreground">
            <strong>Note:</strong> Colimail is a desktop application. Please install and launch the app to access your email.
          </p>
        </div>
      </div>
      <p class="text-sm text-muted-foreground mt-4">
        You can safely close this browser window.
      </p>
    {:else}
      <div class="mb-4 text-destructive">
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
            d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      </div>
      <h2 class="text-2xl font-semibold mb-2 text-destructive">Verification Failed</h2>
      <div class="bg-destructive/10 rounded-lg p-4">
        <p class="text-muted-foreground">{message}</p>
      </div>
      {#if isTauriEnv}
        <p class="text-sm text-muted-foreground mt-4">
          Redirecting to login page...
        </p>
      {:else}
        <p class="text-sm text-muted-foreground mt-4">
          Please try signing up again or contact support if the problem persists.
        </p>
      {/if}
    {/if}
  </div>
</div>
