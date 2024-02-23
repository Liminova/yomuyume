<script setup lang="ts">
defineProps({
	title: {
		type: String,
		required: true,
	},
});
</script>

<template>
	<div class="flex flex-row flex-wrap items-center gap-4">
		<div class="text-xl font-semibold">{{ title }}</div>
		<md-chip-set class="flex flex-row items-stretch">
			<slot />
		</md-chip-set>
	</div>
</template>
