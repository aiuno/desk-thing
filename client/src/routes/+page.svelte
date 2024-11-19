<script lang="ts">
	import SpotifyPlayer from './components/spotify/player.svelte';
	import Launcher from './components/launcher.svelte';
	import Time from './components/time.svelte';
	import Weather from './components/weather/weather.svelte';

	import { onMount } from 'svelte';

	onMount(() => {
		// if top right corner is pressed reload the page
		document.addEventListener('mouseup', (e) => {
			if (e.clientX > window.innerWidth - 40 && e.clientY < 40) {
				location.reload();
			}
		});
	});
</script>

<div class="m-12 mr-0 grid grid-cols-[45vw,45vw]">
	<SpotifyPlayer />
	<div class="grid grid-rows-[auto,auto] gap-0">
		<Weather />
		<Launcher />
	</div>
</div>

<style lang="postcss">
	:global(html, body) {
		@apply bg-black;
		overflow: hidden;
		cursor: none;
	}
</style>
