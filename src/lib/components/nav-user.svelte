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
	import BirdIcon from "@lucide/svelte/icons/bird";
	import WalletIcon from "@lucide/svelte/icons/wallet";
	import LinkIcon from "@lucide/svelte/icons/link";
	import UnlinkIcon from "@lucide/svelte/icons/unlink";
	import LoaderCircleIcon from "@lucide/svelte/icons/loader-circle";

	import * as Avatar from "$lib/components/ui/avatar/index.js";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import * as Sidebar from "$lib/components/ui/sidebar/index.js";
	import { useSidebar } from "$lib/components/ui/sidebar/index.js";
	import { goto } from "$app/navigation";
	import { walletStore } from "$lib/stores/wallet.svelte";
	import WalletConnectDialog from "$lib/components/WalletConnectDialog.svelte";

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

	// WalletConnect dialog state
	let showWalletConnectDialog = $state(false);

	// Handler for wallet connection - opens dialog
	function handleWalletConnect() {
		showWalletConnectDialog = true;
	}

	// Handler for wallet disconnection
	async function handleWalletDisconnect() {
		try {
			await walletStore.disconnect();
		} catch (error) {
			console.error("Failed to disconnect wallet:", error);
		}
	}

	// Handler for managing wallet (open settings)
	function handleManageWallet() {
		if (onSettings) {
			onSettings();
			// TODO: Optionally scroll to CMVH section in settings
		}
	}
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
							{#if isAuthenticated && user.avatar}
								<Avatar.Image src={user.avatar} alt={user.name} />
								<Avatar.Fallback class="rounded-lg">
									{user.name.substring(0, 2).toUpperCase()}
								</Avatar.Fallback>
							{:else}
								<Avatar.Fallback class="rounded-lg">
									<BirdIcon class="size-4" />
								</Avatar.Fallback>
							{/if}
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
				<!-- Wallet Status Section - Always visible for all users -->
				<DropdownMenu.Group>
					<DropdownMenu.Label class="flex items-center gap-2">
						<WalletIcon class="size-4" />
						Wallet Status
					</DropdownMenu.Label>
					{#if walletStore.isConnecting}
						<DropdownMenu.Item disabled>
							<LoaderCircleIcon class="size-4 animate-spin" />
							<span class="text-sm">Connecting...</span>
						</DropdownMenu.Item>
					{:else if walletStore.isConnected}
						<DropdownMenu.Item disabled class="flex-col items-start gap-1">
							<div class="flex items-center gap-2 w-full">
								<div class="size-2 rounded-full bg-green-500 shrink-0"></div>
								<span class="text-sm font-medium">Connected</span>
							</div>
							<div class="pl-4 space-y-0.5">
								{#if walletStore.isResolvingENS}
									<div class="flex items-center gap-1.5">
										<LoaderCircleIcon class="size-3 animate-spin text-muted-foreground" />
										<span class="text-xs text-muted-foreground">Resolving ENS...</span>
									</div>
								{:else}
									<p class="text-xs font-mono">
										{walletStore.getDisplayName()}
									</p>
									{#if walletStore.ensName && walletStore.address}
										<p class="text-xs text-muted-foreground font-mono">
											{walletStore.formatAddress(walletStore.address, 'short')}
										</p>
									{/if}
								{/if}
								{#if walletStore.chainId}
									<p class="text-xs text-muted-foreground">
										Chain ID: {walletStore.chainId}
									</p>
								{/if}
							</div>
						</DropdownMenu.Item>
						{#if onSettings}
							<DropdownMenu.Item onclick={handleManageWallet}>
								<SettingsIcon class="size-4" />
								Manage Wallet
							</DropdownMenu.Item>
						{/if}
						<DropdownMenu.Item onclick={handleWalletDisconnect}>
							<UnlinkIcon class="size-4" />
							Disconnect
						</DropdownMenu.Item>
					{:else}
						<DropdownMenu.Item disabled class="flex-col items-start gap-1">
							<div class="flex items-center gap-2 w-full">
								<div class="size-2 rounded-full bg-red-500 shrink-0"></div>
								<span class="text-sm">Not connected</span>
							</div>
						</DropdownMenu.Item>
						<DropdownMenu.Item onclick={handleWalletConnect}>
							<LinkIcon class="size-4" />
							Connect Wallet
						</DropdownMenu.Item>
					{/if}
				</DropdownMenu.Group>
				<DropdownMenu.Separator />

				{#if isAuthenticated}
					<!-- Authenticated user menu -->
					<DropdownMenu.Label class="p-0 font-normal">
						<div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
							<Avatar.Root class="size-8 rounded-lg">
								{#if user.avatar}
									<Avatar.Image src={user.avatar} alt={user.name} />
									<Avatar.Fallback class="rounded-lg">
										{user.name.substring(0, 2).toUpperCase()}
									</Avatar.Fallback>
								{:else}
									<Avatar.Fallback class="rounded-lg">
										<BirdIcon class="size-4" />
									</Avatar.Fallback>
								{/if}
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

<!-- WalletConnect Dialog -->
<WalletConnectDialog bind:open={showWalletConnectDialog} />
