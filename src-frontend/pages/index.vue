<script setup lang="ts">
import { register } from "swiper/element/bundle";

import { definePageMeta } from "#imports";
import CarouselWrapper from "~/components/home/HomeOldCarouselWrapper.vue";
import HomeRec from "~/components/home/HomeRecCarousel.vue";
import ItemCard from "~/components/ItemCard.vue";
import { useContentActiveSearchTitle } from "~/composables/api/content";

definePageMeta({ layout: "nav-drawer", middleware: ["auth"] });

register();

const titles = useContentActiveSearchTitle({});
</script>

<template>
	<div class="mb-7 mt-3 flex w-full flex-col gap-7 px-6 lg:mt-0 lg:pl-0 lg:pr-3">
		<HomeRec />

		<div
			class="w-fit origin-left text-3xl font-bold transition-transform">
			Recently updated
		</div>
		<CarouselWrapper>
			<swiper-slide
				v-for="title in titles.data.value?.data"
				:key="`up${title.id}`">
				<ItemCard
					:key="`up${title.id}`"
					:title="title" />
			</swiper-slide>
		</CarouselWrapper>

		<div
			class="w-fit origin-left text-3xl font-bold transition-transform">
			Newly added
		</div>
		<CarouselWrapper>
			<swiper-slide
				v-for="title in titles.data.value?.data"
				:key="`new${title.id}`">
				<ItemCard
					:key="`new${title.id}`"
					:title="title" />
			</swiper-slide>
		</CarouselWrapper>

		<div
			class="w-fit origin-left text-3xl font-bold transition-transform">
			Completed stories
		</div>
		<CarouselWrapper>
			<swiper-slide
				v-for="title in titles.data.value?.data"
				:key="`done${title.id}`">
				<ItemCard
					:key="`done${title.id}`"
					:title="title" />
			</swiper-slide>
		</CarouselWrapper>
	</div>
</template>
