<script setup lang="ts">
import { register } from "swiper/element/bundle";
import { ref } from "vue";

import { definePageMeta } from "#imports";
import CardRecommend from "~/components/home/CardRecommend.vue";
import CarouselWrapper from "~/components/home/CarouselWrapper.vue";
import { homeStore } from "~/components/home/utils";
import ItemCard from "~/components/ItemCard.vue";
import type { FilterTitleResponseBody } from "~/composables/api/content";
import { toCoverApiEndpoint } from "~/composables/api/file";

definePageMeta({
	layout: "nav-drawer",
});

register();

const store = homeStore();

const recommendsItems = ref<FilterTitleResponseBody>([]);
const recentlyUpdatedItems = ref<FilterTitleResponseBody>([]);
const newlyAddedItems = ref<FilterTitleResponseBody>([]);
const completedStoriesItems = ref<FilterTitleResponseBody>([]);

// void Promise.all([
// 	indexApi.filter({ keywords: [""], limit: 10 }),
// 	indexApi.filter({
// 		keywords: [""],
// 		limit: 10,
// 		sort_by: "update date",
// 		sort_order: "descending",
// 	}),
// 	indexApi.filter({
// 		keywords: [""],
// 		limit: 10,
// 		sort_by: "add date",
// 		sort_order: "descending",
// 	}),
// 	indexApi.filter({ keywords: [""], limit: 10, is_finished: true }),
// ]).then(([recommends, recentlyUpdated, newlyAdded, completedStories]) => {
// 	recommendsItems.value = recommends.data ?? [];
// 	recentlyUpdatedItems.value = recentlyUpdated.data ?? [];
// 	newlyAddedItems.value = newlyAdded.data ?? [];
// 	completedStoriesItems.value = completedStories.data ?? [];
// });
</script>

<template>
	<div class="mb-7 mt-3 flex w-full flex-col gap-7 px-6 lg:mt-0 lg:pl-0 lg:pr-3">
		<div class="text-5xl font-bold">
			You might want to read
		</div>
		<swiper-container
			class="w-full overflow-hidden rounded-3xl"
			:style="{ height: `${store.recommendsContainerHeight}px` }">
			<!-- :autoplay-delay="5000" -->
			<swiper-slide
				v-for="(title, index) in recommendsItems"
				:key="title.id">
				<CardRecommend
					:is-first-title="index === 0"
					:is-last-title="index === recommendsItems.length - 1"
					:title="title" />
			</swiper-slide>
		</swiper-container>

		<div
			class="w-fit origin-left text-4xl font-bold transition-transform hover:scale-[1.02]">
			Recently updated
		</div>
		<CarouselWrapper>
			<swiper-slide
				v-for="title in recentlyUpdatedItems"
				:key="title.id">
				<ItemCard
					:key="title.id"
					:title="title" />
			</swiper-slide>
		</CarouselWrapper>

		<div
			class="w-fit origin-left text-4xl font-bold transition-transform hover:scale-[1.02]">
			Newly added
		</div>
		<CarouselWrapper>
			<swiper-slide
				v-for="title in newlyAddedItems"
				:key="title.id">
				<ItemCard
					:key="title.id"
					:title="title" />
			</swiper-slide>
		</CarouselWrapper>

		<div
			class="w-fit origin-left text-4xl font-bold transition-transform hover:scale-[1.02]">
			Completed stories
		</div>
		<CarouselWrapper>
			<swiper-slide
				v-for="title in completedStoriesItems"
				:key="title.id">
				<ItemCard
					:key="title.id"
					:title="title" />
			</swiper-slide>
		</CarouselWrapper>
	</div>
</template>
