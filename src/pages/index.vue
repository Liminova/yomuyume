<script setup lang="ts">
import { register } from "swiper/element/bundle";
import { ref } from "vue";

import { definePageMeta } from "#imports";
import CardRecommend from "~/components/home/CardRecommend.vue";
import CarouselWrapper from "~/components/home/CarouselWrapper.vue";
import ItemCard from "~/components/ItemCard.vue";
import { type GetTitleResponseBody, useSearchTitle } from "~/composables/api/content";

definePageMeta({ layout: "nav-drawer" });

register();

const recommendsItems = ref<GetTitleResponseBody[]>([]);
const recentlyUpdatedItems = ref<GetTitleResponseBody[]>([]);
const newlyAddedItems = ref<GetTitleResponseBody[]>([]);
const completedStoriesItems = ref<GetTitleResponseBody[]>([]);

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
		<Carousel
			:opts="{
				align: 'start',
				loop: true,
			}">
			<CarouselContent>
				<CarouselItem
					v-for="(title) in recommendsItems"
					:key="title.id">
					<CardRecommend :title="title" />
				</CarouselItem>
			</CarouselContent>
		</Carousel>

		<div
			class="w-fit origin-left text-3xl font-bold transition-transform hover:scale-[1.02]">
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
			class="w-fit origin-left text-3xl font-bold transition-transform hover:scale-[1.02]">
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
			class="w-fit origin-left text-3xl font-bold transition-transform hover:scale-[1.02]">
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
