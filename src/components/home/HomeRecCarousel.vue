<script setup lang="ts">
/* eslint-disable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-redundant-type-constituents, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call */

import Autoplay from "embla-carousel-autoplay";
import { ArrowLeft, ArrowRight } from "lucide-vue-next";
import { onMounted, ref } from "vue";

import Button from "~/components/ui/Button.vue";
import { Carousel, CarouselContent, CarouselItem } from "~/components/ui/carousel";
import { useSearchTitle } from "~/composables/api/content";
import { cn } from "~/lib/utils";

import HomeRecCard from "./HomeRecCarouselCard.vue";

const carouselRef = ref<InstanceType<typeof Carousel> | null>(null);
const titles = useSearchTitle();
titles.mutate({});

onMounted(() => {
	if (carouselRef.value?.carouselApi === undefined) { return; }
	carouselRef.value?.carouselApi.on("autoplay:timerset", (api) => {
		console.log(api.plugins().autoplay.timeUntilNext());
	});
});
</script>

<template>
	<Carousel
		ref="carouselRef"
		:orientation="carouselRef?.orientation"
		:opts="{
			align: 'start',
			loop: true,
		}"
		:plugins="[Autoplay({
			delay: 1000,
			stopOnInteraction: true,
			stopOnMouseEnter: true,
		})]"
		class="">
		<CarouselContent>
			<CarouselItem
				v-for="(title) in titles.data.value?.data"
				:key="title.id">
				<HomeRecCard
					:title="title" />
			</CarouselItem>
		</CarouselContent>
		<div class="absolute bottom-0 right-0 flex flex-row gap-2 p-10">
			<Button
				:disabled="!carouselRef?.canScrollPrev"
				:class="cn(
					'touch-manipulation h-8 w-8 rounded-full p-0',
					carouselRef?.orientation === 'vertical' && 'rotate-90'
				)"
				variant="outline"
				@click="carouselRef?.scrollPrev">
				<slot>
					<ArrowLeft class="size-4 text-current" />
					<span class="sr-only">Previous Slide</span>
				</slot>
			</Button>
			<Button
				:disabled="!carouselRef?.canScrollNext"
				:class="cn(
					'touch-manipulation h-8 w-8 rounded-full p-0',
					carouselRef?.orientation === 'vertical' && 'rotate-90'
				)"
				variant="outline"
				@click="carouselRef?.scrollNext">
				<slot>
					<ArrowRight class="size-4 text-current" />
					<span class="sr-only">Next Slide</span>
				</slot>
			</Button>
		</div>
	</Carousel>
</template>
