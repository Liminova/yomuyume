<script setup lang="ts">
import { register } from "swiper/element/bundle";
import { computed } from "vue";

import { definePageMeta } from "#imports";
import CarouselWrapper from "~/components/home/HomeOldCarouselWrapper.vue";
import HomeRec from "~/components/home/HomeRecCarousel.vue";
import TitleCard from "~/components/title/TitleCard.vue";
import { useContentSearchTitle } from "~/composables/api/content";

definePageMeta({ layout: "nav-drawer", middleware: ["auth"] });
register();

const recentlyUpdatedInf = useContentSearchTitle({
	order_by: "date_updated",
	is_ascending: true,
});
const recentlyUpdated = computed(() => recentlyUpdatedInf.data.value?.pages.flatMap(page => page.data ?? []) ?? []);

const continueReadingInf = useContentSearchTitle({
	order_by: "progress_last_read_at",
	is_ascending: false,
});
const continueReading = computed(() => continueReadingInf.data.value?.pages.flatMap(page => page.data ?? []) ?? []);

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
				v-for="title in recentlyUpdated"
				:key="`a${title.id}`">
				<TitleCard
					:key="`a${title.id}`"
					:title="title" />
			</swiper-slide>
		</CarouselWrapper>

		<div
			class="w-fit origin-left text-3xl font-bold transition-transform">
			Continue reading
		</div>
		<CarouselWrapper>
			<swiper-slide
				v-for="title in continueReading"
				:key="`b${title.id}`">
				<TitleCard
					:key="`b${title.id}`"
					:title="title" />
			</swiper-slide>
		</CarouselWrapper>
	</div>
</template>
