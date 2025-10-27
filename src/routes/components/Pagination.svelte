<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { ChevronLeft, ChevronRight } from "lucide-svelte";

  // Props
  let {
    currentPage = 1,
    totalPages = 1,
    pageSize = 50,
    totalItems = 0,
    onPageChange,
  }: {
    currentPage: number;
    totalPages: number;
    pageSize?: number;
    totalItems?: number;
    onPageChange: (page: number) => void;
  } = $props();

  let pageInput = $state(currentPage.toString());

  // Calculate the range of items being displayed
  const startItem = $derived((currentPage - 1) * pageSize + 1);
  const endItem = $derived(Math.min(currentPage * pageSize, totalItems));

  // Update pageInput when currentPage changes
  $effect(() => {
    pageInput = currentPage.toString();
  });

  function handlePrevious() {
    if (currentPage > 1) {
      onPageChange(currentPage - 1);
    }
  }

  function handleNext() {
    if (currentPage < totalPages) {
      onPageChange(currentPage + 1);
    }
  }

  function handlePageInputChange(event: Event) {
    const target = event.target as HTMLInputElement;
    pageInput = target.value;
  }

  function handlePageInputKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      const page = parseInt(pageInput);
      if (!isNaN(page) && page >= 1 && page <= totalPages) {
        onPageChange(page);
      } else {
        // Reset to current page if invalid
        pageInput = currentPage.toString();
      }
    }
  }

  function handlePageInputBlur() {
    // Reset to current page on blur if invalid
    const page = parseInt(pageInput);
    if (isNaN(page) || page < 1 || page > totalPages) {
      pageInput = currentPage.toString();
    }
  }
</script>

{#if totalPages > 1}
  <div class="flex items-center justify-center gap-0.1 px-1 py-1 text-[11px]">
    <!-- Email range display -->
    <span class="text-muted-foreground whitespace-nowrap">
      {startItem}-{endItem}/{totalItems}
    </span>

    <Button
      variant="ghost"
      size="icon"
      class="h-6 w-6 shrink-0"
      disabled={currentPage === 1}
      onclick={handlePrevious}
      aria-label="Previous page"
    >
      <ChevronLeft class="h-3.5 w-3.5" />
    </Button>

    <div class="flex items-center gap-1 shrink-0">
      <span class="text-muted-foreground whitespace-nowrap">Page</span>
      <Input
        type="text"
        value={pageInput}
        oninput={handlePageInputChange}
        onkeydown={handlePageInputKeydown}
        onblur={handlePageInputBlur}
        class="h-5 w-9 px-0.5 text-center text-[7px]"
        aria-label="Current page"
      />
      <span class="text-muted-foreground whitespace-nowrap">of {totalPages}</span>
    </div>

    <Button
      variant="ghost"
      size="icon"
      class="h-6 w-6 shrink-0"
      disabled={currentPage === totalPages}
      onclick={handleNext}
      aria-label="Next page"
    >
      <ChevronRight class="h-3.5 w-3.5" />
    </Button>
  </div>
{/if}
