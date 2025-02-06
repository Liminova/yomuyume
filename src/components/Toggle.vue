<script setup lang="ts">
/**
 * Example
 */
import type { HTMLAttributes } from "vue";

const props = withDefaults(defineProps<{
	show: boolean;
	dimension?: "horizontal" | "vertical";
	class?: HTMLAttributes["class"];
}>(), {
	dimension: "vertical",
	class: "",
});
</script>

<template>
	<div
		:style="{
			transitionProperty: 'grid-template-columns, grid-template-rows, opacity',
			transitionDuration: '300ms',
			transitionTimingFunction: 'cubic-bezier(0.4, 0, 0.2, 1)',
		}"
		:class="{
			'grid': true,
			'pointer-events-none opacity-0': !props.show,
			'opacity-100': props.show,
			'grid-cols-[1fr]': props.dimension === 'horizontal' && props.show,
			'grid-rows-[1fr]': props.dimension === 'vertical' && props.show,
			'grid-cols-[0fr]': props.dimension === 'horizontal' && !props.show,
			'grid-rows-[0fr]': props.dimension === 'vertical' && !props.show,
		}">
		<div :class="props.dimension === 'horizontal' ? 'min-w-0' : 'min-h-0'">
			<div :class="props.class">
				<slot />
			</div>
		</div>
	</div>
</template>
