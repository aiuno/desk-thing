<script lang="ts">
	import { onMount } from 'svelte';
	import type { WeatherCode } from './wmo';
	import { wmo_descriptions } from './wmo';

	let weather: any;
	let current_hour: number = new Date().getHours();
	let location: any;
	let next_six_hours: Array<any> = [];
	let is_day: boolean = true;

	const getLocation = async () => {
		const response = await fetch('http://ip-api.com/json', {
			method: 'GET',
			headers: {}
		});
		const data = await response.json();

		location = data;
	};

	const getWeather = async () => {
		const lat = location.lat;
		const lon = location.lon;

		const response = await fetch(
			'https://api.open-meteo.com/v1/forecast?latitude=' +
				lat +
				'&longitude=' +
				lon +
				'&current=temperature_2m,apparent_temperature,is_day,precipitation,weather_code&hourly=temperature_2m,apparent_temperature,precipitation,weather_code,is_day&timezone=auto&forecast_days=2&models=metno_seamless',
			{
				method: 'GET',
				headers: {}
			}
		);
		const data = await response.json();

		is_day = data.hourly.is_day[current_hour];
		weather = data;
	};

	let unique = {};
	const getNextSixHours = async () => {
		next_six_hours = [];
		current_hour = new Date().getHours();

		for (let i = current_hour + 1; i < current_hour + 5; i++) {
			next_six_hours.push({
				hour: i,
				temperature: weather.hourly.temperature_2m[i],
				weather_code: weather.hourly.weather_code[i],
				is_day: weather.hourly.is_day[i],
				description: '',
				image: ''
			});

			if (next_six_hours[i - current_hour - 1].is_day) {
				next_six_hours[i - current_hour - 1].description =
					wmo_descriptions[weather.hourly.weather_code[i] as WeatherCode].day.description;
				next_six_hours[i - current_hour - 1].image =
					wmo_descriptions[weather.hourly.weather_code[i] as WeatherCode].day.image;
			} else {
				next_six_hours[i - current_hour - 1].description =
					wmo_descriptions[weather.hourly.weather_code[i] as WeatherCode].night.description;
				next_six_hours[i - current_hour - 1].image =
					wmo_descriptions[weather.hourly.weather_code[i] as WeatherCode].night.image;
			}
		}

		unique = {};
	};

	onMount(async () => {
		await getLocation();
		await getWeather();
		await getNextSixHours();

		setInterval(getWeather, 1000 * 60 * 10);
		setInterval(() => {
			getNextSixHours();
		}, 1000 * 60);
	});
</script>

<div class="card m-8 w-full max-w-md gap-4 p-4 text-white">
	{#if !weather}
		<div class="text-center text-gray-400">Loading...</div>
	{:else}
		<div class="flex h-full w-full flex-col p-2">
			<div class="flex flex-row">
				<div class="flex flex-col">
					<div class="text-xl">{location.city}</div>
					<div class="flex flex-row">
						<div class="text-2xl">{Math.round(weather.current.temperature_2m)}°</div>
						<div class="flex flex-col whitespace-nowrap">
							<div class="ml-2 text-xs text-gray-400">
								{is_day
									? wmo_descriptions[weather.current.weather_code as WeatherCode].day.description
									: wmo_descriptions[weather.current.weather_code as WeatherCode].night.description}
							</div>
							<div class="ml-2 text-xs text-gray-400">
								Feels like {Math.round(weather.current.apparent_temperature)}°
							</div>
						</div>
					</div>
				</div>
				<div class="relative flex w-full flex-col">
					<img
						src={is_day
							? wmo_descriptions[weather.current.weather_code as WeatherCode].day.image
							: wmo_descriptions[weather.current.weather_code as WeatherCode].night.image}
						alt={is_day
							? wmo_descriptions[weather.current.weather_code as WeatherCode].day.description
							: wmo_descriptions[weather.current.weather_code as WeatherCode].night.description}
						class="now-icon absolute right-0 h-20 w-20"
					/>
				</div>
			</div>
			<div class="flex h-full w-full flex-row">
				<div class="m-0 flex w-full flex-row">
					{#key unique}
						{#each next_six_hours as { hour, temperature, weather_code, is_day, description, image }}
							<div class="forecast flex flex-col">
								<div class="text-center text-gray-400">{hour > 23 ? hour - 24 : hour}:00</div>
								<div class="flex flex-col items-center justify-center">
									<img src={image} alt={description} class="h-12 w-12" />
									<!-- <div class="text-center text-gray-400 text-xs">{description}</div> -->
									<div class="text-center text-gray-400">{Math.round(temperature)}°C</div>
								</div>
							</div>
						{/each}
					{/key}
				</div>
			</div>
		</div>
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

	.forecast {
		@apply flex flex-col items-center justify-center;
		margin-bottom: -10px;
		margin-left: 26px;
	}

	.forecast:first-child {
		margin-left: 0px;
	}

	.forecast :nth-child(2) {
		margin-top: -10px;
	}

	.now-icon {
		top: -10px;
	}
</style>
