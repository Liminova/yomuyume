<script setup lang="ts">
import { getSwiperBreakpoint } from "~/composables/swiperBreakPoint";
import NavDrawerWrapper from "~/layouts/NavDrawerWrapper.vue";

const imagePerRow = ref(5);
const spaceBetween = ref(16);

const categoryIdRaw = useRoute().params.id;
const categoryId = Array.isArray(categoryIdRaw) ? categoryIdRaw[0] : categoryIdRaw;
const snackbarMessage = ref("");

const titles = ref<Array<FilterItemServerResponse>>([]);

void (async () => {
	const { data, message } = await indexApi.filter({
		category_ids: [categoryId],
	});

	if (data === undefined) {
		snackbarMessage.value = message ?? "";
		return;
	}

	titles.value = data;
})();

const observer = new ResizeObserver(() => {
	const breakPoint = getSwiperBreakpoint();

	imagePerRow.value = breakPoint.slidesPerView;
	spaceBetween.value = breakPoint.spaceBetween;
});

const imageContainerRef = ref<HTMLElement | null>(null);

onMounted(() => {
	if (imageContainerRef.value === null) {
		return;
	}

	observer.observe(imageContainerRef.value);
});
</script>

<template>
	<Snackbar :message="snackbarMessage" @close="snackbarMessage = ''" />
	<NavDrawerWrapper>
		<div
			ref="imageContainerRef"
			class="my-3 grid px-6 lg:mt-0 lg:pl-0 lg:pr-3"
			:style="{
				gridTemplateColumns: `repeat(${imagePerRow}, 1fr)`,
				gap: `${spaceBetween}px`,
			}"
		>
			<nuxt-link v-for="title in titles" :key="title.id" :to="`/title/${title.id}`">
				<ItemCard
					:title="title.title"
					:author="title.author ?? 'Unknown'"
					:title-id="title.id"
					:cover="{
						width: title.width,
						height: title.height,
						blurhash: title.blurhash,
						format: title.format,
						src: fileApiUrl.cover(title.id),
					}"
				>
				</ItemCard>
			</nuxt-link>
		</div>
	</NavDrawerWrapper>
</template>
