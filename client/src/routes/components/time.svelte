<script lang="ts">
	import { onMount } from 'svelte';

	let time = new Date().toLocaleTimeString();

	const updateTime = () => {
		time = new Date().toLocaleTimeString(undefined, {
			hour: '2-digit',
			minute: '2-digit',
			month: 'short',
			day: 'numeric',
			hourCycle: 'h23'
		});
	};

	onMount(() => {
		setInterval(updateTime, 1000);
	})
</script>

<div class="card m-8 w-full max-w-md flex-wrap gap-4 p-2 text-white">
	<div class="m-12 flex flex-row overflow-hidden">
		<div class="text-3xl text-white">{time}</div>
	</div>
</div>

<style lang="postcss">
	.card {
		@apply flex flex-row items-center justify-center rounded-lg bg-slate-900 shadow-xl;
		height: calc(50vh - 62px);
		width: 100%;
		max-width: 312px;
		overflow: hidden;
		margin-top: auto;
		padding: 8px;
	}
</style>
