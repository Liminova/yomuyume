<script setup lang="ts">
import { NuxtLink } from '#components'
import { useVibrate } from '@vueuse/core'

const vibrate = useVibrate()
</script>

<template>
	<NuxtLink to="/" @click="vibrate.vibrate">
		<div class="flex flex-row items-center justify-center gap-2">
			<div class="size-11">
				<img src="/favicon/apple-touch-icon.png" class="size-full" />
			</div>
			<div
				class="hidden text-3xl text-neutral-500 md:block"
				style="font-family: Permanent Marker"
			>
				Yomuyume
			</div>
		</div>
	</NuxtLink>
</template>
