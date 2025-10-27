<script lang="ts">
  import { Check, ChevronsUpDown } from "lucide-svelte";
  import * as Command from "$lib/components/ui/command";
  import * as Popover from "$lib/components/ui/popover";
  import { Button } from "$lib/components/ui/button";
  import { cn } from "$lib/utils";
  import { tick } from "svelte";

  interface ComboboxOption {
    value: string;
    label: string;
  }

  interface ComboboxProps {
    options: ComboboxOption[];
    value?: string;
    placeholder?: string;
    searchPlaceholder?: string;
    emptyText?: string;
    class?: string;
    onValueChange?: (value: string) => void;
  }

  let {
    options,
    value = $bindable(""),
    placeholder = "Select option...",
    searchPlaceholder = "Search...",
    emptyText = "No option found.",
    class: className,
    onValueChange
  }: ComboboxProps = $props();

  let open = $state(false);
  let triggerRef = $state<HTMLButtonElement>(null!);

  const selectedValue = $derived(
    options.find((o) => o.value === value)?.label || placeholder
  );

  function closeAndFocusTrigger() {
    open = false;
    tick().then(() => {
      triggerRef?.focus();
    });
  }

  function handleSelect(selectedValue: string) {
    value = selectedValue;
    if (onValueChange) {
      onValueChange(value);
    }
    closeAndFocusTrigger();
  }
</script>

<Popover.Root bind:open>
  <Popover.Trigger bind:ref={triggerRef}>
    {#snippet child({ props })}
      <Button
        {...props}
        variant="outline"
        role="combobox"
        aria-expanded={open}
        class={cn("w-full justify-between", className)}
      >
        {selectedValue}
        <ChevronsUpDown class="ml-2 h-4 w-4 shrink-0 opacity-50" />
      </Button>
    {/snippet}
  </Popover.Trigger>
  <Popover.Content class="w-full p-0">
    <Command.Root>
      <Command.Input placeholder={searchPlaceholder} />
      <Command.List>
        <Command.Empty>{emptyText}</Command.Empty>
        <Command.Group>
          {#each options as option}
            <Command.Item
              value={option.value}
              onSelect={() => {
                handleSelect(option.value);
              }}
            >
              <Check
                class={cn(
                  "mr-2 h-4 w-4",
                  value !== option.value && "text-transparent"
                )}
              />
              {option.label}
            </Command.Item>
          {/each}
        </Command.Group>
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
