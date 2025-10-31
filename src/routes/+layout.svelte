<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { authStore } from '$lib/stores/auth.svelte';

	let { children } = $props();

	// Initialize auth store
	// This will be available globally throughout the app
	// We don't block rendering while loading auth state

	// Disable default context menu globally
	onMount(() => {
		const handleContextMenu = (e: MouseEvent) => {
			e.preventDefault();
		};

		document.addEventListener('contextmenu', handleContextMenu);

		return () => {
			document.removeEventListener('contextmenu', handleContextMenu);
		};
	});
</script>

{@render children()}
