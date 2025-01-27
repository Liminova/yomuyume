<script setup lang="ts">
import { useIntersectionObserver } from "@vueuse/core";
import { onBeforeMount, onMounted, ref } from "vue";

import { useBlurhashDecoder } from "~/composables/use-blurhash-decoder";
import { useJxlDecoder } from "~/composables/use-jxl-decoder";
import { isJxlNative } from "~/lib/is-jxl-native";

const emit = defineEmits(["loaded", "in-view"]);
const props = withDefaults(defineProps<{
	class?: string;
	draggable?: boolean;
	imageClass?: string;
	emitWhenInView?: boolean;

	src?: string;
	blurhash?: string;
	width?: number;
	height?: number;
	isJxl?: boolean;
}>(), {
	class: undefined,
	draggable: false,
	imageClass: undefined,
	emitWhenInView: false,

	src: undefined,
	blurhash: undefined,
	width: undefined,
	height: undefined,
	isJxl: false,
});

const blurhashDecoder = useBlurhashDecoder();
const jxlDecoder = useJxlDecoder();
onBeforeMount(() => {
	if (props.blurhash
		&& props.width !== undefined
		&& props.height !== undefined
		&& !Number.isNaN(props.width)
		&& !Number.isNaN(props.height)) {
		blurhashDecoder.setInput({
			blurhash: props.blurhash,
			width: props.width,
			height: props.height,
		});
		void blurhashDecoder.decode();
	}

	if (props.src && props.isJxl && !isJxlNative) {
		jxlDecoder.setInput(props.src);
		void jxlDecoder.decode();
	} else {
		jxlDecoder.dataUrl.value = props.src;
	}
});

const container = ref<HTMLElement | null>(null);
const imageFullyLoaded = ref(false);
const { stop } = useIntersectionObserver(
	container,
	([entry]) => {
		if (props.emitWhenInView && entry.isIntersecting && imageFullyLoaded.value) {
			emit("in-view");
		}
	},
);
onMounted(() => {
	if (!props.emitWhenInView) { stop(); }
});
</script>

<template>
	<div
		ref="container"
		class="relative"
		:class="props.class">
		<!-- Blurhash placeholder -->
		<img
			v-if="props.blurhash && blurhashDecoder.dataUrl && !imageFullyLoaded"
			loading="lazy"
			class="left-0 top-0 -z-10"
			:style="{
				position: imageFullyLoaded ? 'absolute' : 'static',
			}"
			:class="props.imageClass"
			:src="blurhashDecoder.dataUrl.value"
			:draggable="props.draggable">

		<!-- Actual image -->
		<img
			v-if="jxlDecoder.dataUrl.value"
			loading="lazy"
			class="left-0 top-0"
			:style="{
				transition: 'opacity 0.5s ease',
				position: imageFullyLoaded ? 'static' : 'absolute',
			}"
			:src="jxlDecoder.dataUrl.value"
			:class="props.imageClass"
			:draggable="props.draggable"
			@load="() => {
				imageFullyLoaded = true;
				emit('loaded');
			}">
	</div>
</template>
