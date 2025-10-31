<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import { goto } from "$app/navigation";
  import GalleryVerticalEndIcon from "@lucide/svelte/icons/gallery-vertical-end";
  import MailIcon from "@lucide/svelte/icons/mail";
  import ShieldCheckIcon from "@lucide/svelte/icons/shield-check";
  import ZapIcon from "@lucide/svelte/icons/zap";
</script>

<div class="flex min-h-svh flex-col items-center justify-center bg-muted p-6">
  <div class="w-full max-w-4xl space-y-8">
    <!-- Logo and Title -->
    <div class="flex flex-col items-center gap-4 text-center">
      <div
        class="bg-primary text-primary-foreground flex size-16 items-center justify-center rounded-2xl"
      >
        <GalleryVerticalEndIcon class="size-8" />
      </div>
      <h1 class="text-4xl font-bold tracking-tight">Welcome to Colimail</h1>
      <p class="text-muted-foreground text-lg max-w-2xl">
        A fast, lightweight email client designed for productivity. Manage all your email
        accounts in one beautiful interface.
      </p>
    </div>

    <!-- Features Grid -->
    <div class="grid gap-4 md:grid-cols-3">
      <Card.Root>
        <Card.Header>
          <MailIcon class="mb-2 size-8 text-primary" />
          <Card.Title>Multi-Account</Card.Title>
          <Card.Description>
            Connect multiple email accounts and manage them all in one place
          </Card.Description>
        </Card.Header>
      </Card.Root>

      <Card.Root>
        <Card.Header>
          <ZapIcon class="mb-2 size-8 text-primary" />
          <Card.Title>Lightning Fast</Card.Title>
          <Card.Description>
            Built with performance in mind. No lag, even with thousands of emails
          </Card.Description>
        </Card.Header>
      </Card.Root>

      <Card.Root>
        <Card.Header>
          <ShieldCheckIcon class="mb-2 size-8 text-primary" />
          <Card.Title>Secure & Private</Card.Title>
          <Card.Description>
            Your data stays on your device. Passwords stored securely in OS keyring
          </Card.Description>
        </Card.Header>
      </Card.Root>
    </div>

    <!-- Action Buttons -->
    <div class="flex flex-col items-center gap-4">
      <div class="flex gap-4">
        <Button size="lg" onclick={() => goto("/auth/signup")}>
          Create Account
        </Button>
        <Button size="lg" variant="outline" onclick={() => goto("/auth/login")}>
          Sign In
        </Button>
      </div>
      <Button variant="ghost" onclick={() => goto("/")}>
        Continue without account
      </Button>
      <p class="text-muted-foreground text-sm text-center max-w-md">
        You can use Colimail without an account. Sign up to unlock Pro features,
        cloud sync, and more.
      </p>
    </div>
  </div>
</div>
