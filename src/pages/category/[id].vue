<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";

import { useRoute } from "#app";
import { NuxtLink } from "#components";
import ItemCard from "~/components/ItemCard.vue";
import type { TitleResponseBody } from "~/composables/api/content";
import { toCoverApiEndpoint } from "~/composables/api/file";
import NavDrawerWrapper from "~/layouts/nav-drawer.vue";
import { getSwiperBreakpoint } from "~/lib/swiper-break-points";

const imagePerRow = ref(5);
const spaceBetween = ref(16);

const categoryId = useRoute().params.id as string;

const titles = ref<TitleResponseBody[]>([]);

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
	<NavDrawerWrapper>
		<div
			ref="imageContainerRef"
			class="my-3 grid px-6 lg:mt-0 lg:pl-0 lg:pr-3"
			:style="{
				gridTemplateColumns: `repeat(${imagePerRow}, 1fr)`,
				gap: `${spaceBetween}px`,
			}">
			<NuxtLink
				v-for="title in titles"
				:key="title.id"
				:to="`/title/${title.id}`">
				<ItemCard
					:title="title.title"
					:author="title.author ?? 'Unknown'"
					:title-id="title.id"
					:cover="{
						width: title.cover_width,
						height: title.cover_height,
						blurhash: title.cover_blurhash,
						format: title.cover_format,
						src: toCoverApiEndpoint(title.id),
					}" />
			</NuxtLink>
		</div>
	</NavDrawerWrapper>
</template>
