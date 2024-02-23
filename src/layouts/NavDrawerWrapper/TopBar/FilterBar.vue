<script setup lang="ts">
import "@material/web/ripple/ripple.js";
</script>

<template>
	<nuxt-link
		to="/filter"
		class="elevation-1 relative overflow-hidden rounded-full"
		@click="vibrate"
	>
		<md-ripple />
		<div
			class="w-xs hidden h-12 flex-row items-center justify-between rounded-full bg-[--md-sys-color-surface-container-high] pl-4 pr-3 md:flex"
		>
			<div class="flex w-full items-center justify-center gap-4">
				<i class="fa-light fa-filters text-xl text-[--md-sys-color-on-surface-variant]"></i>
				<div class="bg-transparent pr-24 dark:text-[--md-sys-color-primary-fixed-dim]">
					Ctrl + K
				</div>
			</div>
		</div>
		<div class="flex size-12 items-center justify-center md:hidden">
			<i class="fa-light fa-filters text-[--md-sys-color-on-surface-variant)] text-2xl"></i>
		</div>
	</nuxt-link>
</template>
