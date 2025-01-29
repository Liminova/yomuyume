<script setup lang="ts">
import { useVibrate } from "@vueuse/core";

import { NuxtLink } from "#components";
const vibrate = useVibrate();
</script>

<template>
	<NuxtLink
		to="/"
		@click="vibrate.vibrate">
		<div class="flex flex-row items-center justify-center gap-2">
			<div class="size-11">
				<img
					src="/favicon/apple-touch-icon.png"
					class="size-full">
			</div>
			<div
				class="hidden text-3xl text-neutral-500 md:block"
				style="font-family: Permanent Marker">
				Yomuyume
			</div>
		</div>
	</NuxtLink>
</template>
