<script setup lang="ts">
import { useIntersectionObserver } from "@vueuse/core";
import { ref, watchEffect } from "vue";
import { toast } from "vue-sonner";

import { cn } from "~/lib/utils";

import { isJxlNative } from "./is-jxl-native";
import { useBlurhashDecoder } from "./use-blurhash-decoder";
import { useJpegXLDecoder } from "./use-jpegxl-decoder";

const emit = defineEmits(["in-view"]);
const props = withDefaults(defineProps<{
	id: string;
	class?: string;
	draggable?: boolean;
	emitWhenInView?: boolean;

	src?: string;
	blurhash?: string;
	width?: number;
	height?: number;
	isJxl?: boolean;
}>(), {
	class: undefined,
	draggable: false,
	emitWhenInView: false,

	src: undefined,
	blurhash: undefined,
	width: undefined,
	height: undefined,
	isJxl: false,
});

const imageFullyLoaded = ref(false);
const showBlurhash = ref(true);

const blurhashImgURL = ref<string | null>(null);
const realImgURL = ref<string | null>(null);
const error = ref<string | null>(null);

if (props.blurhash
	&& props.width !== undefined
	&& props.height !== undefined
	&& !Number.isNaN(props.width)
	&& !Number.isNaN(props.height)) {
	void useBlurhashDecoder({
		id: props.id,
		blurhash: props.blurhash,
		width: props.width,
		height: props.height,
	}, blurhashImgURL, error);
}
if (props.src) {
	if (props.isJxl && !isJxlNative) {
		void useJpegXLDecoder({
			id: props.id,
			url: props.src,
		}, realImgURL, error);
	} else {
		realImgURL.value = props.src;
	}
}

const container = ref<HTMLElement | null>(null);
if (props.emitWhenInView) {
	useIntersectionObserver(
		container,
		([entry]) => {
			if (entry.isIntersecting && imageFullyLoaded.value) {
				emit("in-view");
			}
		},
	);
}

watchEffect(() => {
	if (!error.value) { return; }
	toast.error(error.value);
});
</script>

<template>
	<div
		ref="container"
		class="relative size-full">
		<!-- Blurhash placeholder -->
		<img
			v-if="blurhashImgURL && showBlurhash"
			loading="lazy"
			:class="cn(
				imageFullyLoaded ? 'absolute left-0 top-0' : 'static',
				props.class
			)"
			:src="blurhashImgURL"
			:draggable="props.draggable">

		<!-- Actual image -->
		<img
			loading="lazy"
			:class="cn('transition-opacity',
				!imageFullyLoaded ? 'absolute left-0 top-0 ' : 'static',
				imageFullyLoaded ? 'opacity-100' : 'opacity-0',
				props.class
			)"
			:src="realImgURL === null ? undefined : realImgURL"
			:draggable="props.draggable"
			@load="imageFullyLoaded = true"
			@transitionend="showBlurhash = false">
	</div>
</template>
