<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";

import { definePageMeta } from "#imports";
import { useGetCategories } from "~/composables/api/content";
import { getSwiperBreakpoint } from "~/lib/swiper-break-points";

const imageContainerRef = ref<HTMLElement | null>(null);
const imagePerRow = ref(5);
const spaceBetween = ref(16);

const categories = useGetCategories();

definePageMeta({ layout: "nav-drawer", middleware: ["auth"] });

const observer = new ResizeObserver(() => {
	const breakPoint = getSwiperBreakpoint();
	imagePerRow.value = breakPoint.slidesPerView;
	spaceBetween.value = breakPoint.spaceBetween;
});

onMounted(() => {
	if (imageContainerRef.value === null) { return; }
	observer.observe(imageContainerRef.value);
});

onUnmounted(() => {
	observer.disconnect();
});
</script>

<template>
	<div
		v-if="categories.isSuccess"
		ref="imageContainerRef"
		class="mt-3 grid px-6 lg:mt-0 lg:pl-0 lg:pr-3"
		:style="{
			gridTemplateColumns: `repeat(${imagePerRow}, 1fr)`,
			gap: `${spaceBetween}px`,
		}">
		<NuxtLink
			v-for="{ id, name } in categories.data.value"
			:key="id"
			:to="`/category/${id}`"
			class="elevation-2 rounded-xl">
			<div class="my-3 text-center text-xl font-bold">
				{{ name ?? "Untitled" }}
			</div>
		</NuxtLink>
	</div>
</template>
