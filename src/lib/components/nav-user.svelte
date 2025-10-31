<script lang="ts">
	import BadgeCheckIcon from "@lucide/svelte/icons/badge-check";
	import BellIcon from "@lucide/svelte/icons/bell";
	import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down";
	import CreditCardIcon from "@lucide/svelte/icons/credit-card";
	import LogOutIcon from "@lucide/svelte/icons/log-out";
	import LogInIcon from "@lucide/svelte/icons/log-in";
	import UserPlusIcon from "@lucide/svelte/icons/user-plus";
	import SparklesIcon from "@lucide/svelte/icons/sparkles";
	import SettingsIcon from "@lucide/svelte/icons/settings";

	import * as Avatar from "$lib/components/ui/avatar/index.js";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import * as Sidebar from "$lib/components/ui/sidebar/index.js";
	import { useSidebar } from "$lib/components/ui/sidebar/index.js";
	import { goto } from "$app/navigation";

	let {
		user,
		isAuthenticated = false,
		onSettings,
		onUpgrade,
		onAccount,
		onBilling,
		onNotifications,
		onLogout
	}: {
		user: { name: string; email: string; avatar: string };
		isAuthenticated?: boolean;
		onSettings?: () => void;
		onUpgrade?: () => void;
		onAccount?: () => void;
		onBilling?: () => void;
		onNotifications?: () => void;
		onLogout?: () => void;
	} = $props();

	const sidebar = useSidebar();
</script>

<Sidebar.Menu>
	<Sidebar.MenuItem>
		<DropdownMenu.Root>
			<DropdownMenu.Trigger>
				{#snippet child({ props })}
					<Sidebar.MenuButton
						{...props}
						size="lg"
						class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground md:h-8 md:p-0"
					>
						<Avatar.Root class="size-8 rounded-lg">
							<Avatar.Image src={user.avatar} alt={user.name} />
							<Avatar.Fallback class="rounded-lg">CN</Avatar.Fallback>
						</Avatar.Root>
						<div class="grid flex-1 text-left text-sm leading-tight">
							<span class="truncate font-medium">{user.name}</span>
							<span class="truncate text-xs">{user.email}</span>
						</div>
						<ChevronsUpDownIcon class="ml-auto size-4" />
					</Sidebar.MenuButton>
				{/snippet}
			</DropdownMenu.Trigger>
			<DropdownMenu.Content
				class="w-(--bits-dropdown-menu-anchor-width) min-w-56 rounded-lg"
				side={sidebar.isMobile ? "bottom" : "right"}
				align="end"
				sideOffset={4}
			>
				{#if isAuthenticated}
					<!-- Authenticated user menu -->
					<DropdownMenu.Label class="p-0 font-normal">
						<div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
							<Avatar.Root class="size-8 rounded-lg">
								<Avatar.Image src={user.avatar} alt={user.name} />
								<Avatar.Fallback class="rounded-lg">
									{user.name.substring(0, 2).toUpperCase()}
								</Avatar.Fallback>
							</Avatar.Root>
							<div class="grid flex-1 text-left text-sm leading-tight">
								<span class="truncate font-medium">{user.name}</span>
								<span class="truncate text-xs">{user.email}</span>
							</div>
						</div>
					</DropdownMenu.Label>
					<DropdownMenu.Separator />
					<DropdownMenu.Group>
						{#if onUpgrade}
							<DropdownMenu.Item onclick={onUpgrade}>
								<SparklesIcon />
								Upgrade to Pro
							</DropdownMenu.Item>
						{/if}
					</DropdownMenu.Group>
					<DropdownMenu.Separator />
					<DropdownMenu.Group>
						{#if onAccount}
							<DropdownMenu.Item onclick={onAccount}>
								<BadgeCheckIcon />
								Account
							</DropdownMenu.Item>
						{/if}
						{#if onBilling}
							<DropdownMenu.Item onclick={onBilling}>
								<CreditCardIcon />
								Billing
							</DropdownMenu.Item>
						{/if}
						{#if onNotifications}
							<DropdownMenu.Item onclick={onNotifications}>
								<BellIcon />
								Notifications
							</DropdownMenu.Item>
						{/if}
						{#if onSettings}
							<DropdownMenu.Item onclick={onSettings}>
								<SettingsIcon />
								Settings
							</DropdownMenu.Item>
						{/if}
					</DropdownMenu.Group>
					<DropdownMenu.Separator />
					{#if onLogout}
						<DropdownMenu.Item onclick={onLogout}>
							<LogOutIcon />
							Log out
						</DropdownMenu.Item>
					{/if}
				{:else}
					<!-- Guest user menu - show login/signup options -->
					<DropdownMenu.Label class="p-0 font-normal">
						<div class="flex flex-col gap-1 px-3 py-2">
							<span class="text-sm font-medium">Welcome to Colimail</span>
							<span class="text-xs text-muted-foreground">
								Sign in to unlock Pro features
							</span>
						</div>
					</DropdownMenu.Label>
					<DropdownMenu.Separator />
					<DropdownMenu.Group>
						<DropdownMenu.Item onclick={() => goto("/auth/login")}>
							<LogInIcon />
							Sign In
						</DropdownMenu.Item>
						<DropdownMenu.Item onclick={() => goto("/auth/signup")}>
							<UserPlusIcon />
							Create Account
						</DropdownMenu.Item>
					</DropdownMenu.Group>
					{#if onSettings}
						<DropdownMenu.Separator />
						<DropdownMenu.Group>
							<DropdownMenu.Item onclick={onSettings}>
								<SettingsIcon />
								Settings
							</DropdownMenu.Item>
						</DropdownMenu.Group>
					{/if}
				{/if}
			</DropdownMenu.Content>
		</DropdownMenu.Root>
	</Sidebar.MenuItem>
</Sidebar.Menu>
