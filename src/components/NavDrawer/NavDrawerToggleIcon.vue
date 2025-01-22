<script setup lang="ts">
import { useNavDrawerStore } from "~/composables/nav-drawer-store";

const navDrawerStore = useNavDrawerStore();
</script>

<template>
	<div
		class="relative mx-4 flex size-12 cursor-pointer place-content-center place-items-center overflow-hidden rounded-full has-[input:focus-visible]:ring-2 has-[input:focus-visible]:ring-primary">
		<input
			v-model="navDrawerStore.isDrawerExpanded"
			type="checkbox"
			class="absolute z-[1] size-12 opacity-0">
		<div class="absolute flex flex-col gap-1">
			<div
				class="h-[2px] w-5 rounded-full bg-primary transition-transform"
				:class="navDrawerStore.isDrawerExpanded ? 'translate-x-[-2px]' : 'translate-x-0'" />
			<div
				class="h-[2px] w-5 rounded-full bg-primary transition-transform"
				:class="navDrawerStore.isDrawerExpanded ? 'translate-x-[4px]' : 'translate-x-0'" />
			<div
				class="h-[2px] w-5 rounded-full bg-primary transition-transform"
				:class="navDrawerStore.isDrawerExpanded ? 'translate-x-[-2px]' : 'translate-x-0'" />
		</div>
	</div>
</template>
