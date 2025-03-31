<script setup lang="ts">
import { useIntersectionObserver } from "@vueuse/core";
import { ref, useTemplateRef, watchEffect } from "vue";
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

/* eslint-disable no-unused-vars */
enum LoadState {
	NotLoaded = "not-loaded",
	Loading = "loading",
	Loaded = "loaded",
}
/* eslint-enable no-unused-vars */

const imageLoadState = ref(LoadState.NotLoaded);
const blurhashLoadState = ref(LoadState.NotLoaded);
const showBlurhash = ref(true);

const blurhashImgURL = ref<string | null>(null);
const realImgURL = ref<string | null>(null);
const error = ref<string | null>(null);

const container = useTemplateRef<HTMLElement>("container");
useIntersectionObserver(container, ([entry]) => {
	if (!entry.isIntersecting) { return; }

	if (props.emitWhenInView) { emit("in-view"); }

	if (props.blurhash
		&& blurhashLoadState.value === LoadState.NotLoaded
		&& props.width !== undefined
		&& props.height !== undefined
		&& Number(props.width) > 0
		&& Number(props.height) > 0) {
		blurhashLoadState.value = LoadState.Loading;
		void useBlurhashDecoder({
			id: props.id,
			blurhash: props.blurhash,
			width: props.width,
			height: props.height,
		}, blurhashImgURL, error);
	}

	if (props.src && imageLoadState.value === LoadState.NotLoaded) {
		imageLoadState.value = LoadState.Loading;
		if (props.isJxl && !isJxlNative) {
			void useJpegXLDecoder({
				id: props.id,
				url: props.src,
			}, realImgURL, error);
		} else {
			realImgURL.value = props.src;
		}
	}
});

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
				imageLoadState === LoadState.Loaded ? 'absolute left-0 top-0' : 'static',
				props.class
			)"
			:src="blurhashImgURL"
			:draggable="props.draggable"
			@load="blurhashLoadState = LoadState.Loaded">

		<!-- Actual image -->
		<img
			loading="lazy"
			:class="cn('transition-opacity',
				imageLoadState !== LoadState.Loaded ? 'absolute left-0 top-0 ' : 'static',
				imageLoadState === LoadState.Loaded ? 'opacity-100' : 'opacity-0',
				props.class
			)"
			:src="realImgURL === null ? undefined : realImgURL"
			:draggable="props.draggable"
			@load="imageLoadState = LoadState.Loaded"
			@transitionend="showBlurhash = false">
	</div>
</template>
