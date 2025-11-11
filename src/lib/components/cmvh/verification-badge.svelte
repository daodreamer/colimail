<script lang="ts">
	import { Badge } from "$lib/components/ui/badge";
	import type { VerificationState } from "$lib/cmvh/types";
	import { Shield, ShieldCheck, ShieldAlert, ShieldX, Loader2 } from "lucide-svelte";

	interface Props {
		verification: VerificationState;
		onclick?: () => void;
	}

	let { verification, onclick }: Props = $props();

	let config = $derived.by(() => {
		switch (verification.status) {
			case "verified-local":
				return {
					label: "Verified",
					variant: "default" as const,
					icon: ShieldCheck,
					class: "bg-green-500/10 text-green-600 border-green-500/30 hover:bg-green-500/20",
				};
			case "verified-onchain":
				return {
					label: "On-Chain Verified",
					variant: "default" as const,
					icon: ShieldCheck,
					class: "bg-blue-500/10 text-blue-600 border-blue-500/30 hover:bg-blue-500/20",
				};
			case "invalid":
				return {
					label: "Invalid Signature",
					variant: "destructive" as const,
					icon: ShieldX,
					class: "",
				};
			case "error":
				return {
					label: "Verification Error",
					variant: "destructive" as const,
					icon: ShieldAlert,
					class: "",
				};
			case "verifying":
				return {
					label: "Verifying...",
					variant: "secondary" as const,
					icon: Loader2,
					class: "",
				};
			default:
				return {
					label: "Not Signed",
					variant: "outline" as const,
					icon: Shield,
					class: "opacity-60",
				};
		}
	});
</script>

<Badge
	variant={config.variant}
	class="cursor-pointer transition-all {config.class}"
	{onclick}
>
	{@const Icon = config.icon}
	<Icon class="h-3 w-3 mr-1" />
	{config.label}
	{#if verification.result?.chain}
		<span class="ml-1 opacity-70 text-xs">({verification.result.chain})</span>
	{/if}
</Badge>
