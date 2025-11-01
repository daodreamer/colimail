<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { cn } from "$lib/utils.js";
  import type { HTMLAttributes } from "svelte/elements";
  import { signInWithEmail, signInWithGoogle, resetPassword, getAuthErrorMessage, getCurrentSession, exchangeCodeForSession, supabase } from "$lib/supabase";
  import { goto } from "$app/navigation";
  import { authStore } from "$lib/stores/auth.svelte";
  import { onMount } from "svelte";

  let { class: className, ...restProps }: HTMLAttributes<HTMLDivElement> = $props();

  let email = $state("");
  let password = $state("");
  let loading = $state(false);
  let error = $state("");
  let resetSent = $state(false);

  // Setup OAuth callback listener on component mount (for desktop app)
  onMount(() => {
    console.log('[Login Component] onMount called');
    const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
    console.log('[Login Component] isTauri:', isTauri);
    console.log('[Login Component] window.__TAURI_INTERNALS__:', window && (window as any).__TAURI_INTERNALS__);
    
    if (isTauri) {
      console.log('[Login] Setting up global OAuth callback listener...');
      
      let unlistenFn: (() => void) | null = null;
      let isProcessing = false; // Prevent multiple simultaneous processing
      
      (async () => {
        console.log('[Login] Importing @tauri-apps/api/event...');
        const { listen } = await import('@tauri-apps/api/event');
        console.log('[Login] listen function imported:', typeof listen);
        
        console.log('[Login] Calling listen for oauth-code-received event...');
        unlistenFn = await listen<string>('oauth-code-received', async (event) => {
          console.log('[Login] *** GLOBAL EVENT RECEIVED *** OAuth code:', event.payload);
          console.log('[Login] Full event object:', event);
          
          // Prevent concurrent processing
          if (isProcessing) {
            console.warn('[Login] Already processing an OAuth callback, ignoring this event');
            return;
          }
          
          try {
            isProcessing = true;
            loading = true;
            // Use the code directly with Supabase's session handling
            // Supabase will handle PKCE verification internally
            console.log('[Login] Processing OAuth code with Supabase...');
            
            // Exchange code for session using Supabase
            const { data, error: exchangeError } = await supabase.auth.exchangeCodeForSession(event.payload);
            
            if (exchangeError) {
              console.error('[Login] Code exchange error:', exchangeError);
              error = getAuthErrorMessage(exchangeError);
              loading = false;
              isProcessing = false;
              return;
            }
            
            console.log('[Login] Code exchange successful, session data:', data);
            
            // Refresh auth state, passing the session we just got to avoid calling getSession() again
            console.log('[Login] Calling authStore.refreshUser() with session...');
            await authStore.refreshUser(data.session);
            console.log('[Login] authStore.refreshUser() completed');
            console.log('[Login] authStore.isAuthenticated:', authStore.isAuthenticated);
            
            if (authStore.isAuthenticated) {
              console.log('[Login] OAuth successful, redirecting to main app');
              loading = false;
              if (unlistenFn) unlistenFn();
              console.log('[Login] Calling goto("/")...');
              goto("/");
              console.log('[Login] goto("/") called');
            } else {
              console.error('[Login] Authentication failed after code exchange');
              error = "Authentication failed. Please try again.";
              loading = false;
            }
          } catch (exchangeError: any) {
            console.error('[Login] Error exchanging code:', exchangeError);
            error = getAuthErrorMessage(exchangeError);
            loading = false;
          }
        });
        
        console.log('[Login] Event listener registered successfully, unlisten function:', typeof unlistenFn);
      })();

      // Return cleanup function
      return () => {
        console.log('[Login] Cleanup: unlistening from oauth-code-received');
        if (unlistenFn) unlistenFn();
      };
    } else {
      console.log('[Login] Not in Tauri environment, skipping OAuth listener setup');
    }
  });

  async function handleEmailLogin(e: Event) {
    e.preventDefault();
    error = "";
    resetSent = false;

    if (!email || !password) {
      error = "Please enter your email and password";
      return;
    }

    try {
      loading = true;
      console.log('[Login] Attempting email login for:', email);
      await signInWithEmail(email, password);
      console.log('[Login] Email login successful');

      // Wait a bit for authStore to sync
      await new Promise(resolve => setTimeout(resolve, 500));

      // Force refresh auth state
      console.log('[Login] Refreshing auth store...');
      await authStore.refreshUser();
      console.log('[Login] Auth store refreshed, isAuthenticated:', authStore.isAuthenticated);

      // Redirect to main app
      console.log('[Login] Redirecting to main app');
      goto("/");
    } catch (err: any) {
      error = getAuthErrorMessage(err);
      console.error("[Login] Login error:", err);
    } finally {
      loading = false;
    }
  }

  async function handleGoogleLogin() {
    error = "";
    resetSent = false;

    try {
      loading = true;
      console.log('[Login] Starting Google login...');
      const response = await signInWithGoogle();
      console.log('[Login] Full OAuth response:', response);
      const { url } = response;
      console.log('[Login] Got OAuth URL:', url);

      if (url) {
        // Check if running in Tauri environment
        const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
        console.log('[Login] Is Tauri environment:', isTauri);

        if (isTauri) {
          // Desktop app: Open OAuth URL in system default browser
          console.log('[Login] Opening OAuth in system browser...');
          const { openUrl } = await import('@tauri-apps/plugin-opener');
          await openUrl(url);
          console.log('[Login] System browser opened, waiting for callback via global listener...');
          
          // IMPORTANT: Don't reset loading state here!
          // Keep button disabled while waiting for OAuth callback
          // The global listener (set in onMount) will handle the callback and reset loading
          console.log('[Login] Keeping loading state, waiting for deep link callback...');
        } else {
          // Web environment: redirect main window
          window.location.href = url;
        }
      }
    } catch (err: any) {
      console.error("[Login] Google login error details:", {
        message: err.message,
        code: err.code,
        status: err.status,
        details: err
      });
      error = getAuthErrorMessage(err);
      console.error("Google login error:", err);
      loading = false;
    }
  }

  async function handleForgotPassword() {
    if (!email) {
      error = "Please enter your email address first";
      return;
    }

    try {
      loading = true;
      error = "";
      await resetPassword(email);
      resetSent = true;
    } catch (err: any) {
      error = getAuthErrorMessage(err);
      console.error("Reset password error:", err);
    } finally {
      loading = false;
    }
  }
</script>

<div class={cn("flex flex-col gap-6", className)} {...restProps}>
  <Card.Root>
    <Card.Header class="text-center">
      <Card.Title class="text-xl">Welcome back</Card.Title>
      <Card.Description>Login with your Google account or email</Card.Description>
    </Card.Header>
    <Card.Content>
      <form onsubmit={handleEmailLogin}>
        <div class="space-y-4">
          {#if error}
            <div class="bg-destructive/15 text-destructive rounded-md p-3 text-sm">
              {error}
            </div>
          {/if}

          {#if resetSent}
            <div class="bg-green-500/15 text-green-700 dark:text-green-400 rounded-md p-3 text-sm">
              Password reset email sent! Please check your inbox.
            </div>
          {/if}

          <div class="space-y-2">
            <Button
              variant="outline"
              type="button"
              onclick={handleGoogleLogin}
              disabled={loading}
              class="w-full"
            >
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                <path
                  d="M12.48 10.92v3.28h7.84c-.24 1.84-.853 3.187-1.787 4.133-1.147 1.147-2.933 2.4-6.053 2.4-4.827 0-8.6-3.893-8.6-8.72s3.773-8.72 8.6-8.72c2.6 0 4.507 1.027 5.907 2.347l2.307-2.307C18.747 1.44 16.133 0 12.48 0 5.867 0 .307 5.387.307 12s5.56 12 12.173 12c3.573 0 6.267-1.173 8.373-3.36 2.16-2.16 2.84-5.213 2.84-7.667 0-.76-.053-1.467-.173-2.053H12.48z"
                  fill="currentColor"
                />
              </svg>
              {loading ? "Connecting..." : "Login with Google"}
            </Button>
          </div>

          <div class="relative">
            <Separator />
            <div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 bg-card px-2 text-xs text-muted-foreground">
              Or continue with email
            </div>
          </div>

          <div class="space-y-2">
            <Label for="email">Email</Label>
            <Input
              id="email"
              type="email"
              placeholder="m@example.com"
              required
              bind:value={email}
              disabled={loading}
            />
          </div>

          <div class="space-y-2">
            <div class="flex items-center">
              <Label for="password">Password</Label>
              <button
                type="button"
                class="ml-auto text-sm underline-offset-4 hover:underline"
                onclick={handleForgotPassword}
                disabled={loading}
              >
                Forgot your password?
              </button>
            </div>
            <Input
              id="password"
              type="password"
              required
              bind:value={password}
              disabled={loading}
            />
          </div>

          <div class="space-y-2">
            <Button type="submit" disabled={loading} class="w-full">
              {loading ? "Logging in..." : "Login"}
            </Button>
            <p class="text-center text-sm text-muted-foreground">
              Don't have an account? <a href="/auth/signup" class="underline">Sign up</a>
            </p>
          </div>
        </div>
      </form>
    </Card.Content>
  </Card.Root>
  <p class="px-6 text-center text-xs text-muted-foreground">
    By clicking continue, you agree to our <a href="/terms" class="underline">Terms of Service</a>
    and <a href="/privacy" class="underline">Privacy Policy</a>.
  </p>
</div>
