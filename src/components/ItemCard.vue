<script setup lang="ts">
import { computed } from "vue";

import { NuxtLink } from "#components";
import type { GetTitleResponseBody } from "~/composables/api/content";
import { toCoverApiEndpoint } from "~/composables/api/file";

import Image from "./Image.vue";

const props = defineProps<{ title: GetTitleResponseBody }>();

const progress = computed(() => {
	return props.title.page_read;
});
</script>

<template>
	<NuxtLink
		class="flex flex-col items-start justify-center"
		:to="`/title/${title.id}`">
		<div class="img-cover group relative w-full overflow-hidden rounded-xl">
			<Image
				:id="title.id"
				emit-when-in-view
				:src="toCoverApiEndpoint(title.id)"
				:blurhash="title.cover_blurhash"
				:width="title.cover_width"
				:height="title.cover_height"
				:is-jxl="title.cover_jxl"
				class="aspect-[3/4]"
				image-class="rounded-xl h-full object-cover" />
			<div
				class="absolute left-0 top-0 size-full bg-[rgba(255_255_255/0.08)] opacity-0 transition-opacity group-[.img-cover]:hover:opacity-100" />
			<md-linear-progress
				v-show="progress !== 0"
				:value="progress !== 0"
				class="absolute bottom-0 w-full" />

			<i
				v-show="progress === 1"
				class="fa-solid fa-circle-check absolute right-2 top-1 text-xl text-[--md-sys-color-on-secondary-fixed-variant]" />
		</div>
		<div>
			<div class="mt-2 truncate text-sm font-light text-[--md-sys-color-on-surface]">
				{{ title.author ?? "Unknown" }}
			</div>
			<div
				class="truncate-2 text-balance text-lg font-bold text-[--md-sys-color-inverse-surface]">
				{{ title.title ?? "Untitled" }}
			</div>
		</div>
	</NuxtLink>
</template>
