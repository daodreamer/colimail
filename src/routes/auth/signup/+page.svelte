<script lang="ts">
  import SignupForm from "$lib/components/signup-form.svelte";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import { Button } from "$lib/components/ui/button";
</script>

<div class="bg-muted flex min-h-svh flex-col items-center justify-center p-6 md:p-10">
  <!-- Back button in top-left corner -->
  <div class="absolute top-4 left-4 z-50">
    <Button variant="outline" size="default" href="/" class="gap-2">
      <ArrowLeftIcon class="size-4" />
      Back to App
    </Button>
  </div>

  <div class="w-full max-w-sm md:max-w-4xl">
    <SignupForm />
  </div>
</div>
