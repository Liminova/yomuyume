<script setup lang="ts">
/* eslint-disable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-redundant-type-constituents, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unsafe-call */

import Autoplay from "embla-carousel-autoplay";
import { ArrowLeft, ArrowRight } from "lucide-vue-next";
import { onMounted, onUnmounted, ref } from "vue";

import Button from "~/components/ui/Button.vue";
import { Carousel, type CarouselApi, CarouselContent, CarouselItem } from "~/components/ui/carousel";
import { useContentActiveSearchTitle } from "~/composables/api/content";
import { cn } from "~/lib/utils";

import Toggle from "../Toggle.vue";
import HomeRecCard from "./HomeRecCarouselCard.vue";

const carouselRef = ref<InstanceType<typeof Carousel> | null>(null);
const titles = useContentActiveSearchTitle({});

const isNextSlideBarActive = ref(true);
const nextSlideBar = ref<HTMLElement | null>(null);
let [rafID, timeoutID] = [0, 0];

function startNextSlideBar(api: CarouselApi): void {
	if (nextSlideBar.value === null || !api) { return; }
	isNextSlideBarActive.value = true;

	// reset states
	nextSlideBar.value.style.animationName = "none";
	nextSlideBar.value.style.transform = "translate3d(-100%, 0, 0)";

	// set new states
	rafID = window.requestAnimationFrame(() => {
		timeoutID = window.setTimeout(() => {
			const timeUntilNext = api.plugins().autoplay.timeUntilNext();
			if (nextSlideBar.value === null || timeUntilNext === null) { return; }

			nextSlideBar.value.style.animationName = "autoplay-progress";
			nextSlideBar.value.style.animationDuration = `${timeUntilNext}ms`;
		});
	});
}
function stopNextSlideBar(): void {
	if (nextSlideBar.value === null) { return; }
	isNextSlideBarActive.value = false;

	nextSlideBar.value.style.animationName = "none";
	nextSlideBar.value.style.transform = "translate3d(-100%, 0, 0)";
}

onMounted(() => {
	if (carouselRef.value?.carouselApi === undefined) { return; }

	carouselRef.value.carouselApi.on("autoplay:timerset", startNextSlideBar);
	carouselRef.value.carouselApi.on("autoplay:timerstopped", stopNextSlideBar);
});

onUnmounted(() => {
	window.cancelAnimationFrame(rafID);
	window.clearTimeout(timeoutID);
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
			delay: 5000,
			stopOnInteraction: true,
			stopOnMouseEnter: true,
		})]">
		<CarouselContent>
			<CarouselItem
				v-for="(title) in titles.data.value?.data"
				:key="`rec${title.id}`">
				<HomeRecCard
					:title="title" />
			</CarouselItem>
		</CarouselContent>

		<div class="absolute bottom-0 right-0 flex flex-col gap-2 p-10">
			<div class="grid grid-cols-2 gap-2">
				<Button
					:disabled="!carouselRef?.canScrollPrev"
					:class="cn(
						'touch-manipulation h-8 w-8 rounded-full p-0',
						carouselRef?.orientation === 'vertical' && 'rotate-90'
					)"
					variant="outline"
					@click="carouselRef?.scrollPrev()">
					<slot>
						<ArrowLeft class="size-4 text-current" />
						<span class="sr-only">Previous recommended title</span>
					</slot>
				</Button>

				<Button
					:disabled="!carouselRef?.canScrollNext"
					:class="cn(
						'touch-manipulation h-8 w-8 rounded-full p-0',
						carouselRef?.orientation === 'vertical' && 'rotate-90'
					)"
					variant="outline"
					@click="carouselRef?.scrollNext()">
					<slot>
						<ArrowRight class="size-4 text-current" />
						<span class="sr-only">Next recommended title</span>
					</slot>
				</Button>
			</div>

			<!-- next slide indicator -->
			<Toggle :show="isNextSlideBarActive">
				<div class="relative overflow-hidden rounded-full">
					<div class="col-span-2 h-1 bg-secondary-foreground" />
					<div
						ref="nextSlideBar"
						class="filler absolute left-0 top-0 h-1 w-full border bg-secondary/80"
						:style="{
							animationTimingFunction: 'linear',
							animationIterationCount: '1',
						}" />
				</div>
			</Toggle>
		</div>
	</Carousel>
</template>

<style>
@keyframes autoplay-progress {
	from {
		transform: translate3d(-100%, 0, 0);
	}

	to {
		transform: translate3d(0, 0, 0);
	}
}
</style>
