<script lang="ts">
	import * as Card from "$lib/components/ui/card";
	import { Badge } from "$lib/components/ui/badge";
	import { Button } from "$lib/components/ui/button";
	import type { VerificationResult, CMVHConfig } from "$lib/cmvh/types";
	import { getExplorerUrl, formatAddress, formatTimestamp } from "$lib/cmvh/blockchain";
	import { ShieldCheck, ShieldX, ExternalLink, Globe } from "lucide-svelte";

	interface Props {
		result: VerificationResult;
		config: CMVHConfig;
		onVerifyOnChain?: () => void;
	}

	let { result, config, onVerifyOnChain }: Props = $props();

	let explorerUrl = $derived.by(() => {
		if (!result.signer_address) return null;
		return getExplorerUrl(result.signer_address, config.network);
	});

	function openExplorer() {
		if (explorerUrl) {
			window.open(explorerUrl, "_blank");
		}
	}
</script>

<Card.Root class="w-full max-w-2xl">
	<Card.Header>
		<Card.Title class="flex items-center gap-2">
			{#if result.is_valid}
				<ShieldCheck class="h-5 w-5 text-green-600" />
				<span>Email Verification</span>
			{:else}
				<ShieldX class="h-5 w-5 text-red-600" />
				<span>Verification Failed</span>
			{/if}
		</Card.Title>
		<Card.Description>
			{#if result.is_valid}
				This email has been cryptographically signed and verified using CMVH (ColiMail
				Verification Header)
			{:else}
				This email's signature could not be verified
			{/if}
		</Card.Description>
	</Card.Header>

	<Card.Content class="space-y-4">
		{#if result.is_valid}
			<div class="flex items-start gap-2 text-sm">
				<ShieldCheck class="h-4 w-4 text-green-600 mt-0.5" />
				<div class="flex-1">
					<p class="font-semibold">Signature Verified</p>
					<p class="text-muted-foreground">
						The sender's identity and message integrity have been confirmed locally
					</p>
				</div>
			</div>

			<div class="grid grid-cols-1 gap-4 mt-4">
				{#if result.ens_name}
					<div>
						<p class="text-sm font-medium mb-1">Signer</p>
						<Badge variant="default" class="font-mono">
							{result.ens_name}
						</Badge>
					</div>
				{/if}

				{#if result.signer_address}
					<div>
						<p class="text-sm font-medium mb-1">Ethereum Address</p>
						<div class="flex items-center gap-2">
							<code class="text-sm font-mono bg-muted px-2 py-1 rounded">
								{formatAddress(result.signer_address)}
							</code>
							<Button variant="ghost" size="sm" onclick={openExplorer}>
								<ExternalLink class="h-3 w-3" />
							</Button>
						</div>
					</div>
				{/if}

				{#if result.chain}
					<div>
						<p class="text-sm font-medium mb-1">Blockchain</p>
						<Badge variant="outline">
							<Globe class="h-3 w-3 mr-1" />
							{result.chain}
						</Badge>
					</div>
				{/if}

				{#if result.timestamp}
					<div>
						<p class="text-sm font-medium mb-1">Signed At</p>
						<p class="text-sm text-muted-foreground">
							{formatTimestamp(result.timestamp)}
						</p>
					</div>
				{/if}
			</div>
		{:else}
			<div class="flex items-start gap-2 text-sm text-destructive">
				<ShieldX class="h-4 w-4 mt-0.5" />
				<div class="flex-1">
					<p class="font-semibold">Verification Failed</p>
					<p class="text-muted-foreground">
						{result.error || "The signature could not be verified. This may indicate the email was modified or the signature is invalid."}
					</p>
				</div>
			</div>
		{/if}
	</Card.Content>

	<Card.Footer class="flex flex-wrap gap-2">
		{#if onVerifyOnChain && result.is_valid}
			<Button variant="outline" onclick={onVerifyOnChain}>
				<Globe class="h-4 w-4 mr-2" />
				Verify On-Chain
			</Button>
		{/if}
		{#if result.signer_address}
			<Button variant="ghost" onclick={openExplorer}>
				<ExternalLink class="h-4 w-4 mr-2" />
				View on {config.network === "arbitrum" ? "Arbiscan" : "Sepolia Arbiscan"}
			</Button>
		{/if}
	</Card.Footer>
</Card.Root>
