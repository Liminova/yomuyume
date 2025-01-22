<script setup lang="ts">
import { swiperBreakpoints } from "~/lib/swiper-break-points";

import { homeStore } from "./utils";

const store = homeStore();
</script>

<template>
	<swiper-container
		class="w-full overflow-hidden rounded-t-xl"
		:space-between="store.gapPixel"
		:slides-per-view="2"
		:breakpoints="swiperBreakpoints">
		<slot />
	</swiper-container>
</template>
