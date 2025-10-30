<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';

	let { children } = $props();

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
