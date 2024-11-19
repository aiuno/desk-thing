<script lang="ts">
	import IcRoundSkipPrevious from '~icons/ic/round-skip-previous';
	import IcRoundSkipNext from '~icons/ic/round-skip-next';
	import IcRoundPlayArrow from '~icons/ic/round-play-arrow';
	import IcRoundPause from '~icons/ic/round-pause';
	import LineMdSpotifyFilled from '~icons/line-md/spotify-filled';
	import { onMount } from 'svelte';

	import { extractColors } from './helpers';
	import { server_ip } from '../../settings';

	let song_info: any;

	let album_cover = '';
	let song = '';
	let artist = '';

	let playing = false;
	let average_color = '#000000';

	let connected = false;

	let devices: Array<any> = [];
	let list_devices = false;

	const togglePlaying = async () => {
		await fetch(server_ip, {
			method: 'POST',
			body: playing ? 'SP_PAUSE' : 'SP_PLAY'
		});

		playing = !playing;
	};

	const currentTrack = async () => {
		const response = await fetch(server_ip, {
			method: 'POST',
			body: 'SP_CURRENT_TRACK'
		});

		let data: any;
		try {
			data = await response.json();
		} catch (e) {
			data = null;
		}

		connected = response.status === 200;

		return data;
	};

	const nextTrack = async () => {
		await fetch(server_ip, {
			method: 'POST',
			body: 'SP_NEXT'
		});
	};

	const previousTrack = async () => {
		if (song_info.progress_ms > 5000) {
			await fetch(server_ip, {
				method: 'POST',
				body: 'SP_SEEK 0'
			});
		} else {
			await fetch(server_ip, {
				method: 'POST',
				body: 'SP_PREV'
			});
		}
	};

	const updatePlayer = async () => {
		if (!connected) return;

		const current_time = new Date().getMilliseconds();
		song_info = await currentTrack();

		const player = document.getElementById('player');

		if (!song_info) {
			if (player) player.style.backgroundColor = '#111827';
			return;
		}

		const item = song_info.item;
		const cover = item.album.images[1].url;

		album_cover = cover;
		song = item.name;
		artist = item.artists[0].name;

		playing = song_info.is_playing;

		const response_time = new Date().getMilliseconds();
		const progress = document.querySelector('.progress') as HTMLElement;
		if (progress) {
			// Calculate progress and add the request time to it
			const progressWidth =
				((song_info.progress_ms + (response_time - current_time)) / item.duration_ms) * 100;
			progress.style.width = `${progressWidth}%`;
		}

		extractColors(album_cover, { crossOrigin: 'anonymous' }).then((colors) => {
			const bestColor = colors.reduce(
				(closest, color) => {
					const lightnessDifference = Math.abs(color.lightness - 0.3);
					return lightnessDifference < closest.lightnessDifference
						? { color, lightnessDifference }
						: closest;
				},
				{ color: colors[0], lightnessDifference: 10 }
			).color;

			average_color = bestColor.hex;
			if (player) player.style.backgroundColor = average_color;
		});
	};

	const getDevices = async () => {
		const response = await fetch(server_ip, {
			method: 'POST',
			body: 'SP_DEVICES'
		});

		let devices: any;
		try {
			devices = await response.json();
		} catch (e) {
			devices = null;
		}

		connected = response.status === 200;

		return devices;
	};

	const transferPlayback = async (device_id: string) => {
		await fetch(server_ip, {
			method: 'POST',
			body: `SP_TRANSFER ${device_id}`
		});
	};

	// check if mouse is over progress bar and seek to that position
	const seek_x = (e: MouseEvent) => {
		const progress = document.querySelector('.progress-bar') as HTMLElement;
		const progressWidth = progress.offsetWidth;
		const x = e.clientX - progress.getBoundingClientRect().left;
		const seek_ms = (x / progressWidth) * song_info.item.duration_ms;

		fetch(server_ip, {
			method: 'POST',
			body: `SP_SEEK ${Math.round(seek_ms)}`
		});
	};

	onMount(async () => {
		await updatePlayer();
		devices = (await getDevices()).devices;

		setInterval(async () => {
			await updatePlayer();
		}, 1000);

		setInterval(async () => {
			if (!connected || song_info) {
				return;
			}

			devices = (await getDevices()).devices;
		}, 1000 * 5);
	});
</script>

<div id="player" class="card relative w-full max-w-md bg-slate-900 p-4 px-0 pb-0 text-white">
	{#if connected && song_info}
		<img
			src={album_cover}
			alt="Album Cover"
			class="mx-auto h-32 w-32 rounded-lg md:h-48 md:w-48 lg:h-56 lg:w-56"
		/>
		<p class="md:text-md mx-4 mt-4 text-center text-sm lg:text-xl">{song}</p>
		<p class="mt-2 text-center text-sm text-gray-300 md:text-base lg:text-lg">{artist}</p>

		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="controls-container bottom-0 mt-auto w-full rounded-lg bg-black bg-opacity-20 pb-4">
			<div class="progress-bar h-1 w-full rounded-lg bg-gray-700" on:mouseup={seek_x}>
				<div class="progress h-full w-0 rounded-lg bg-white"></div>
			</div>

			<div class="controls mt-4 flex justify-center space-x-4">
				<button
					class="h-10 w-10 rounded-lg md:h-12 md:w-12 lg:h-14 lg:w-14"
					on:mouseup={previousTrack}><IcRoundSkipPrevious /></button
				>
				{#if !playing}
					<button
						class="h-10 w-10 rounded-lg md:h-12 md:w-12 lg:h-14 lg:w-14"
						on:mouseup={togglePlaying}><IcRoundPlayArrow /></button
					>
				{:else}
					<button
						class="h-10 w-10 rounded-lg md:h-12 md:w-12 lg:h-14 lg:w-14"
						on:mouseup={togglePlaying}><IcRoundPause /></button
					>
				{/if}
				<button class="h-10 w-10 rounded-lg md:h-12 md:w-12 lg:h-14 lg:w-14" on:mouseup={nextTrack}
					><IcRoundSkipNext /></button
				>
			</div>
		</div>
	{:else if list_devices}
		<button
			class="overlay active"
			on:click={() => {
				list_devices = false;
			}}
			aria-label="x"
		></button>
		<div class="text-center text-gray-400">Select a device:</div>
		<div class="grid w-2/3 grid-cols-1 divide-y divide-gray-600 self-center">
			{#each devices as device, i}
				<button
					class="z-10 bg-gray-800 p-2 text-white {i === 0 ? 'mt-4' : ''} {i === devices.length - 1
						? 'mt-0'
						: ''} 
						{devices.length === 1 ? 'rounded-lg' : i < devices.length - 1 ? 'rounded-t-lg' : 'rounded-b-lg'}"
					on:click={async () => {
						list_devices = false;
						await transferPlayback(device.id);
						await updatePlayer();
					}}>{device.name}</button
				>
			{/each}
		</div>
	{:else}
		<LineMdSpotifyFilled class="mx-auto mb-2 h-20 w-20 text-gray-400" />
		{#if !connected}
			<div class="text-center text-gray-400">Not connected...</div>
		{:else}
			<div class="text-center text-gray-400">Waiting for device...</div>
			{#if devices.length > 0}
				<button
					class="mt-4 rounded-lg bg-gray-800 p-2 text-white"
					on:click={() => (list_devices = true)}
					>{devices.length}
					{devices.length === 1 ? 'device' : 'devices'}
					available</button
				>
			{/if}
		{/if}
	{/if}
</div>

<style lang="postcss">
	.card {
		@apply flex flex-col items-center justify-center rounded-lg shadow-xl;
		width: 100%;
		height: 92.5%;
		overflow: hidden;
	}

	.progress-bar {
		@apply flex items-center;
	}

	.controls {
		@apply flex flex-row justify-center;
	}

	button:hover {
		cursor: none;
	}

	button:active {
		cursor: none;
	}

	.controls button {
		font-size: xx-large;
	}

	.controls-container {
		width: 100%;
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
