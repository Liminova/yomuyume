<script setup lang="ts">
import { useRoute } from "nuxt/app";
import { computed, onMounted, onUnmounted, ref } from "vue";

import { definePageMeta } from "#imports";
import TitleCard from "~/components/title/TitleCard.vue";
import { useContentSearchTitle } from "~/composables/api/content";
import { getSwiperBreakpoint } from "~/lib/swiper-break-points";

definePageMeta({ layout: "nav-drawer" });

const imagePerRow = ref(5);
const spaceBetween = ref(16);

const searchResultsInf = useContentSearchTitle({
	category_ids: [useRoute().params.id as string],
});
const searchResults = computed(() => searchResultsInf.data.value?.pages.flatMap(page => page.data ?? []) ?? []);

const observer = new ResizeObserver(() => {
	const breakPoint = getSwiperBreakpoint();

	imagePerRow.value = breakPoint.slidesPerView;
	spaceBetween.value = breakPoint.spaceBetween;
});

const imageContainerRef = ref<HTMLElement | null>(null);
onMounted(() => {
	if (imageContainerRef.value === null) { return; }
	observer.observe(imageContainerRef.value);
});

onUnmounted(() => {
	observer.disconnect();
});
</script>

<template>
	<div>
		<div
			v-if="searchResults.length !== 0"
			ref="imageContainerRef"
			class="my-3 grid px-6 lg:mt-0 lg:pl-0 lg:pr-3"
			:style="{
				gridTemplateColumns: `repeat(${imagePerRow}, 1fr)`,
				gap: `${spaceBetween}px`,
			}">
			<NuxtLink
				v-for="title in searchResults"
				:key="title.id"
				:to="`/title/${title.id}`">
				<TitleCard :title="title" />
			</NuxtLink>
		</div>

		<div
			v-if="searchResults.length === 0"
			class="w-full py-10 text-center">
			This category is empty.
		</div>
	</div>
</template>
