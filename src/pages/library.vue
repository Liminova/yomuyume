<script setup lang="ts">
import type { CategoryResponse } from "~/composables/bridge";
import { getSwiperBreakpoint } from "~/composables/swiperBreakPoint";
import NavDrawerWrapper from "~/layouts/NavDrawerWrapper.vue";

const imageContainerRef = ref<HTMLElement | null>(null);
const imagePerRow = ref(5);
const spaceBetween = ref(16);

const snackbarMessage = ref("");

const categories: Ref<Array<CategoryResponse>> = ref([]);

void (async () => {
	const { data, message } = await indexApi.categories();

	if (data === undefined) {
		snackbarMessage.value = message ?? "";
		return;
	}

	categories.value = data;
})();

const observer = new ResizeObserver(() => {
	const breakPoint = getSwiperBreakpoint();

	imagePerRow.value = breakPoint.slidesPerView;
	spaceBetween.value = breakPoint.spaceBetween;
});

onMounted(() => {
	if (imageContainerRef.value === null) {
		return;
	}

	observer.observe(imageContainerRef.value);
});

document.title = "Yomuyume - Library";
</script>

<template>
	<div>
		<Snackbar :message="snackbarMessage" @close="snackbarMessage = ''" />

		<NavDrawerWrapper>
			<div
				ref="imageContainerRef"
				class="mt-3 grid px-6 lg:mt-0 lg:pl-0 lg:pr-3"
				:style="{
					gridTemplateColumns: `repeat(${imagePerRow}, 1fr)`,
					gap: `${spaceBetween}px`,
				}"
			>
				<nuxt-link
					v-for="category in categories"
					:key="category.id"
					:to="`/category/${category.id}`"
					class="elevation-2 rounded-xl"
				>
					<div class="my-3 text-center text-xl font-bold">{{ category.name }}</div>
				</nuxt-link>
			</div>
		</NavDrawerWrapper>
	</div>
</template>
