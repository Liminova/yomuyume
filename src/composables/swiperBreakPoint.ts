import type { BreakpointRecord } from "./types";

const swiperBreakpoints: BreakpointRecord = {
	0: {
		slidesPerView: 2,
		spaceBetween: 16,
	},
	768: {
		slidesPerView: 3,
		spaceBetween: 16,
	},
	1024: {
		slidesPerView: 4,
		spaceBetween: 16,
	},
	1280: {
		slidesPerView: 5,
		spaceBetween: 16,
	},
	1536: {
		slidesPerView: 6,
		spaceBetween: 16,
	},
	1792: {
		slidesPerView: 7,
		spaceBetween: 16,
	},
};

/**
 * Gets the number of slides per view and the space between slides from the
 * swiper breakpoints configuration based on the current window width.
 *
 * @param breakpoints Swiper breakpoints configuration, default is
 * swiperBreakpoints from src/variables/store.ts
 */
function getSwiperBreakpoint(): { slidesPerView: number; spaceBetween: number } {
	const sortedBreakpoints = Object.keys(swiperBreakpoints).sort((a, b) => Number(b) - Number(a));

	// this matches the behavior of swiperjs: from a certain width, look down to
	// the nearest smaller breakpoint and use that as the slidesPerView
	const breakpoint = sortedBreakpoints.find(
		(breakpoint) => Number(breakpoint) <= window.innerWidth
	);

	if (breakpoint) {
		const data = swiperBreakpoints[Number(breakpoint)];
		let spaceBetween = 0;

		if (typeof data.spaceBetween === "number" && data.spaceBetween > 0) {
			spaceBetween = data.spaceBetween;
		}

		return { slidesPerView: data.slidesPerView, spaceBetween };
	}

	// if the window width is smaller than the smallest breakpoint, return the
	// smallest breakpoint
	const data = swiperBreakpoints[Number(sortedBreakpoints[sortedBreakpoints.length - 1])];
	let spaceBetween = 0;

	if (typeof data.spaceBetween === "number" && data.spaceBetween > 0) {
		spaceBetween = data.spaceBetween;
	}

	return { slidesPerView: data.slidesPerView, spaceBetween };
}

export { getSwiperBreakpoint, swiperBreakpoints };
