<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";

import { useRoute } from "#app";
import { NuxtLink } from "#components";
import { definePageMeta } from "#imports";
import ItemCard from "~/components/ItemCard.vue";
import { useSearchTitle } from "~/composables/api/content";
import { getSwiperBreakpoint } from "~/lib/swiper-break-points";

definePageMeta({ layout: "nav-drawer" });

const imagePerRow = ref(5);
const spaceBetween = ref(16);

const searchTitle = useSearchTitle();
searchTitle.mutate({ category_ids: [useRoute().params.id as string] });

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

const showSomething = computed(() => searchTitle.data.value?.data && searchTitle.data.value.data.length > 0);
const showNothing = computed(() => searchTitle.data.value?.data === undefined || searchTitle.data.value.data.length === 0);
</script>

<template>
	<div>
		<div
			v-if="showSomething"
			ref="imageContainerRef"
			class="my-3 grid px-6 lg:mt-0 lg:pl-0 lg:pr-3"
			:style="{
				gridTemplateColumns: `repeat(${imagePerRow}, 1fr)`,
				gap: `${spaceBetween}px`,
			}">
			<NuxtLink
				v-for="title in searchTitle.data?.value?.data"
				:key="title.id"
				:to="`/title/${title.id}`">
				<ItemCard :title="title" />
			</NuxtLink>
		</div>
		<div
			v-if="showNothing"
			class="w-full py-10 text-center">
			This category is empty.
		</div>
	</div>
</template>
