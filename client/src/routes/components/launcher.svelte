<script lang="ts">
	import BxsFolder from '~icons/bxs/folder';
	import { onMount } from 'svelte';

	import { server_ip } from '../settings';

	let selected_folder: string = '';
	let shortcuts: Array<any> = [];

	const getShortcuts = async () => {
		const response = await fetch(server_ip, {
			method: 'POST',
			body: 'SC_GET'
		});
		
		shortcuts = await response.json();
	};

	const runShortcut = async (name: string) => {
		await fetch(server_ip, {
			method: 'POST',
			body: `SC_RUN ${name}`
		});
	};

	onMount(async () => {
		await getShortcuts();

		setInterval(async () => {
			await getShortcuts();
		}, 5000);
	});
</script>

<div class="card m-8 w-full max-w-md flex-wrap gap-4 p-2 text-white relative">
	{#if shortcuts.length === 0}
		<div class="text-center text-gray-400">Nothing here...</div>
	{:else if selected_folder !== ''}
		<button
			class="overlay active"
			on:click={() => {
				selected_folder = '';
			}}
			aria-label="x"
		></button>

		{#each shortcuts as shortcut}
			{#if shortcut.name === selected_folder && shortcut.is === 'folder'}
				{#each shortcut.shortcuts as new_shortcut}
					<div class="flex flex-col items-center justify-center gap-1">
						<button
							class="shortcut flex h-12 w-12 items-center justify-center bg-gray-800 text-white"
							on:mouseup={() => {
								runShortcut(shortcut.name + '/' + new_shortcut.name);
							}}
						>
							<img src={new_shortcut.icon} alt={new_shortcut.name} class="h-10 w-10" />
						</button>
						<div class="text-center text-xs text-gray-400">{new_shortcut.name}</div>
					</div>
				{/each}
			{/if}
		{/each}
	{:else}
		{#each shortcuts as shortcut}
			{#if shortcut.is === 'folder'}
				<div class="flex flex-col items-center justify-center gap-1 relative">
					<button
						class="shortcut flex h-12 w-12 items-center justify-center bg-gray-800 text-white"
						on:mouseup={() => {
							selected_folder = shortcut.name;
						}}
					>
						<img src={shortcut.icon} alt={shortcut.name} class="h-10 w-10" />
					</button>
					<BxsFolder class="h-3 w-3 absolute z-10 bottom-5 right-1" />
					<div class="text-center text-xs text-gray-400">{shortcut.name} </div>
				</div>
			{:else}
				<div class="flex flex-col items-center justify-center gap-1">
					<button
						class="shortcut flex h-12 w-12 items-center justify-center bg-gray-800 text-white"
						on:mouseup={() => {
							runShortcut(shortcut.name);
						}}
					>
						<img src={shortcut.icon} alt={shortcut.name} class="h-10 w-10" />
					</button>
					<div class="text-center text-xs text-gray-400">{shortcut.name}</div>
				</div>
			{/if}
		{/each}
	{/if}
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

	.card .shortcut {
		border-radius: 50%;
		padding: 8px;
		align-content: center;
		overflow: hidden;
		cursor: none;
		z-index: 3;
	}

	.overlay {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		background-color: transparent;
		z-index: 1;
		display: none;
		cursor: none;
	}

	.overlay:hover {
		cursor: none;
	}

	.overlay.active {
		display: block;
	}
</style>
