<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { supabase } from "$lib/supabase";
  import { authStore } from "$lib/stores/auth.svelte";

  let status = $state<"processing" | "success" | "error">("processing");
  let message = $state("Processing authentication...");

  onMount(async () => {
    try {
      console.log('[Callback] OAuth callback page mounted');

      // Supabase automatically handles the callback and sets the session
      // We just need to wait a bit for it to process
      await new Promise(resolve => setTimeout(resolve, 500));

      // Check if we now have a session
      const { data: { session }, error: sessionError } = await supabase.auth.getSession();

      console.log('[Callback] Session check:', session ? `User: ${session.user.email}` : 'No session', sessionError);

      if (sessionError) {
        throw sessionError;
      }

      if (session) {
        console.log('[Callback] Session found, authentication successful');
        status = "success";
        message = "Authentication successful! Redirecting...";

        // Wait for authStore to sync
        await new Promise(resolve => setTimeout(resolve, 500));

        // Force refresh the auth state
        console.log('[Callback] Refreshing auth store...');
        await authStore.refreshUser();
        console.log('[Callback] Auth store refreshed, isAuthenticated:', authStore.isAuthenticated);

        // Redirect to main app
        setTimeout(() => {
          console.log('[Callback] Redirecting to main app');
          goto("/");
        }, 1000);
      } else {
        console.log('[Callback] No session found, checking URL hash for tokens...');
        // Check URL hash for tokens (fallback)
        const hashParams = new URLSearchParams(window.location.hash.substring(1));
        const accessToken = hashParams.get("access_token");
        const refreshToken = hashParams.get("refresh_token");
        const error = hashParams.get("error");
        const errorDescription = hashParams.get("error_description");

        if (error) {
          throw new Error(errorDescription || error);
        }

        if (accessToken) {
          console.log('[Callback] Found access token in URL hash, setting session manually');
          // Manually set the session
          const { error: setSessionError } = await supabase.auth.setSession({
            access_token: accessToken,
            refresh_token: refreshToken || "",
          });

          if (setSessionError) throw setSessionError;

          console.log('[Callback] Session set manually, authentication successful');
          status = "success";
          message = "Authentication successful! Redirecting...";

          // Wait for authStore to sync
          await new Promise(resolve => setTimeout(resolve, 500));
          console.log('[Callback] Refreshing auth store after manual session set...');
          await authStore.refreshUser();
          console.log('[Callback] Auth store refreshed, isAuthenticated:', authStore.isAuthenticated);

          setTimeout(() => {
            console.log('[Callback] Redirecting to main app');
            goto("/");
          }, 1000);
        } else {
          console.log('[Callback] No access token found in URL hash');
          throw new Error("No authentication session found");
        }
      }
    } catch (err: any) {
      status = "error";
      message = err.message || "Authentication failed";
      console.error("OAuth callback error:", err);

      // Redirect to login after showing error
      setTimeout(() => {
        goto("/auth/login");
      }, 3000);
    }
  });
</script>

<div class="flex min-h-svh flex-col items-center justify-center p-6">
  <div class="w-full max-w-md text-center">
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
