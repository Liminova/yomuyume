import { createInjectionState } from "@vueuse/core";
import emblaCarouselVue from "embla-carousel-vue";
import { onMounted, ref } from "vue";

import type { CarouselProps } from "./interface";

const [createInject, useInject] = createInjectionState(({ opts, orientation, plugins }: CarouselProps) => {
	const [carouselRef, carouselApi] = emblaCarouselVue({
		...opts,
		axis: orientation === "horizontal" ? "x" : "y",
	}, plugins);

	const canScrollNext = ref(false);
	const canScrollPrev = ref(false);

	function prelude(): void {
		canScrollNext.value = carouselApi.value?.canScrollNext() ?? false;
		canScrollPrev.value = carouselApi.value?.canScrollPrev() ?? false;
	}

	onMounted(() => {
		if (!carouselApi.value) { return; }
		carouselApi.value.on("init", prelude);
		carouselApi.value.on("reInit", prelude);
		carouselApi.value.on("select", prelude);
	});

	return {
		carouselRef,
		carouselApi,
		orientation,
		canScrollNext,
		canScrollPrev,
		scrollNext: (): void => {
			carouselApi.value?.scrollNext();
		},
		scrollPrev: (): void => {
			carouselApi.value?.scrollPrev();
		},
	};
});

// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type
function useCarousel() {
	const carouselState = useInject();

	if (!carouselState) { throw new Error("useCarousel must be used within a <Carousel />"); }

	return carouselState;
}

export { createInject as createCarouselInject, useCarousel };

