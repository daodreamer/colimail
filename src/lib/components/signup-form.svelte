<script lang="ts">
  import { cn } from "$lib/utils.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import type { HTMLAttributes } from "svelte/elements";
  import { signUpWithEmail, signInWithGoogle, getAuthErrorMessage, validatePassword } from "$lib/supabase";
  import { goto } from "$app/navigation";
  import { authStore } from "$lib/stores/auth.svelte";

  let { class: className, ...restProps }: HTMLAttributes<HTMLDivElement> = $props();

  let email = $state("");
  let password = $state("");
  let confirmPassword = $state("");
  let name = $state("");
  let loading = $state(false);
  let error = $state("");
  let successMessage = $state("");

  async function handleEmailSignup(e: Event) {
    e.preventDefault();
    error = "";
    successMessage = "";

    // Validation
    if (!email || !password || !confirmPassword || !name) {
      error = "Please fill in all required fields";
      return;
    }

    // Validate name
    if (name.length < 2) {
      error = "Name must be at least 2 characters long";
      return;
    }

    if (name.length > 30) {
      error = "Name must be less than 30 characters";
      return;
    }

    // Name should only contain alphanumeric characters, underscores, and hyphens
    if (!/^[a-zA-Z0-9_-]+$/.test(name)) {
      error = "Name can only contain letters, numbers, underscores, and hyphens";
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
      await signUpWithEmail(email, password, name);
      successMessage = "Account created successfully! Please check your email for verification.";

      // Wait a bit to show success message, then redirect
      setTimeout(() => {
        goto("/");
      }, 2000);
    } catch (err: any) {
      error = getAuthErrorMessage(err);
      console.error("Signup error:", err);
    } finally {
      loading = false;
    }
  }

  async function handleGoogleSignup() {
    error = "";
    try {
      loading = true;
      const { url } = await signInWithGoogle();
      if (url) {
        // Check if running in Tauri environment
        const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
        console.log('[Signup] Is Tauri environment:', isTauri);

        if (isTauri) {
          // Open OAuth in a new window (desktop app)
          console.log('[Signup] Importing WebviewWindow...');
          const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
          console.log('[Signup] WebviewWindow imported');

          console.log('[Signup] Creating OAuth window with URL:', url);
          const oauthWindow = new WebviewWindow('oauth-google-signup', {
            url,
            title: 'Sign up with Google',
            width: 500,
            height: 700,
            resizable: false,
            center: true,
            alwaysOnTop: true,
          });

          console.log('[Signup] OAuth window created:', oauthWindow);

          // Monitor when user closes the window (cancel OAuth)
          oauthWindow.once('tauri://destroyed', () => {
            console.log('[Signup] OAuth window closed');

            // Check if authentication was successful
            setTimeout(async () => {
              if (authStore.isAuthenticated) {
                console.log('[Signup] OAuth successful, redirecting to main app');
                goto("/");
              } else {
                console.log('[Signup] OAuth cancelled or failed');
                loading = false;
              }
            }, 500);
          });

          oauthWindow.once('tauri://error', (e) => {
            console.log('[Signup] OAuth window error:', e);
            loading = false;
          });

          // Important: Do not navigate the main window
          // The OAuth window will handle the authentication
          console.log('[Signup] OAuth window created, main window stays on signup page');
        } else {
          // Fallback for web/browser environment
          console.log('[Signup] Not in Tauri, redirecting main window to OAuth URL');
          window.location.href = url;
        }
      }
    } catch (err: any) {
      error = getAuthErrorMessage(err);
      console.error("Google signup error:", err);
      loading = false;
    }
  }
</script>

<div class={cn("flex flex-col gap-6", className)} {...restProps}>
  <Card.Root class="overflow-hidden p-0">
    <Card.Content class="grid p-0 md:grid-cols-2">
      <form class="p-6 md:p-8" onsubmit={handleEmailSignup}>
        <div class="space-y-4">
          <div class="flex flex-col items-center gap-2 text-center">
            <h1 class="text-2xl font-bold">Create your account</h1>
            <p class="text-muted-foreground text-balance text-sm">
              Enter your details below to create your account
            </p>
          </div>

          {#if error}
            <div class="bg-destructive/15 text-destructive rounded-md p-3 text-sm">
              {error}
            </div>
          {/if}

          {#if successMessage}
            <div class="bg-green-500/15 text-green-700 dark:text-green-400 rounded-md p-3 text-sm">
              {successMessage}
            </div>
          {/if}

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
            <p class="text-xs text-muted-foreground">
              We'll use this to contact you. We will not share your email with anyone else.
            </p>
          </div>

          <div class="space-y-2">
            <Label for="name">Name</Label>
            <Input
              id="name"
              type="text"
              placeholder="Your name"
              required
              bind:value={name}
              disabled={loading}
            />
            <p class="text-xs text-muted-foreground">
              2-30 characters. Letters, numbers, underscores, and hyphens only.
            </p>
          </div>

          <div class="space-y-2">
            <div class="grid grid-cols-2 gap-4">
              <div class="space-y-2">
                <Label for="password">Password</Label>
                <Input
                  id="password"
                  type="password"
                  required
                  bind:value={password}
                  disabled={loading}
                />
              </div>
              <div class="space-y-2">
                <Label for="confirm-password">Confirm Password</Label>
                <Input
                  id="confirm-password"
                  type="password"
                  required
                  bind:value={confirmPassword}
                  disabled={loading}
                />
              </div>
            </div>
            <p class="text-xs text-muted-foreground">
              Must be at least 8 characters and include lowercase, uppercase, digits, and symbols.
            </p>
          </div>

          <div class="space-y-2">
            <Button type="submit" disabled={loading} class="w-full">
              {loading ? "Creating Account..." : "Create Account"}
            </Button>
          </div>

          <div class="relative">
            <Separator />
            <div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 bg-card px-2 text-xs text-muted-foreground">
              Or continue with
            </div>
          </div>

          <div class="space-y-2">
            <Button variant="outline" type="button" onclick={handleGoogleSignup} disabled={loading} class="w-full">
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                <path
                  d="M12.48 10.92v3.28h7.84c-.24 1.84-.853 3.187-1.787 4.133-1.147 1.147-2.933 2.4-6.053 2.4-4.827 0-8.6-3.893-8.6-8.72s3.773-8.72 8.6-8.72c2.6 0 4.507 1.027 5.907 2.347l2.307-2.307C18.747 1.44 16.133 0 12.48 0 5.867 0 .307 5.387.307 12s5.56 12 12.173 12c3.573 0 6.267-1.173 8.373-3.36 2.16-2.16 2.84-5.213 2.84-7.667 0-.76-.053-1.467-.173-2.053H12.48z"
                  fill="currentColor"
                />
              </svg>
              {loading ? "Connecting..." : "Sign up with Google"}
            </Button>
          </div>

          <p class="text-center text-sm text-muted-foreground">
            Already have an account? <a href="/auth/login" class="underline">Sign in</a>
          </p>
        </div>
      </form>
      <div class="bg-muted relative hidden md:block">
        <div class="absolute inset-0 flex items-center justify-center p-8">
          <div class="text-center">
            <h2 class="text-2xl font-bold mb-4">Welcome to Colimail</h2>
            <p class="text-muted-foreground">
              A fast, lightweight email client built for productivity. Manage all your email
              accounts in one place.
            </p>
          </div>
        </div>
      </div>
    </Card.Content>
  </Card.Root>
  <p class="px-6 text-center text-xs text-muted-foreground">
    By clicking continue, you agree to our <a href="/terms" class="underline">Terms of Service</a>
    and <a href="/privacy" class="underline">Privacy Policy</a>.
  </p>
</div>
