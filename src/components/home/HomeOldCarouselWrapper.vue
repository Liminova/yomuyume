<script setup lang="ts">
import { swiperBreakpoints } from "~/lib/swiper-break-points";
</script>

<template>
	<swiper-container
		class="w-full overflow-hidden rounded-t-xl"
		:space-between="18"
		:slides-per-view="2"
		:breakpoints="swiperBreakpoints">
		<slot />
	</swiper-container>
</template>
