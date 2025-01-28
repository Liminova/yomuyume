<script setup lang="ts">
import { useIntersectionObserver } from "@vueuse/core";
import { ref, watchEffect } from "vue";
import { toast } from "vue-sonner";

import { useBlurhashDecoder } from "~/composables/use-blurhash-decoder";
import { useJpegXLDecoder } from "~/composables/use-jpegxl-decoder";
import { isJxlNative } from "~/lib/image/is-jxl-native";
import { cn } from "~/lib/utils";

const emit = defineEmits(["in-view"]);
const props = withDefaults(defineProps<{
	id: string;
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
		:class="cn('relative', props.class)">
		<!-- Blurhash placeholder -->
		<img
			v-if="blurhashImgURL && showBlurhash"
			loading="lazy"
			class="left-0 top-0 -z-10"
			:style="{
				position: imageFullyLoaded ? 'absolute' : 'static',
			}"
			:class="props.imageClass"
			:src="blurhashImgURL"
			:draggable="props.draggable">

		<!-- Actual image -->
		<img
			loading="lazy"
			:class="cn('left-0 top-0', props.imageClass)"
			:style="{
				opacity: imageFullyLoaded ? 1 : 0,
				transition: 'opacity 150ms ease',
				position: imageFullyLoaded ? 'static' : 'absolute',
			}"
			:src="realImgURL === null ? undefined : realImgURL"
			:draggable="props.draggable"
			@load="imageFullyLoaded = true"
			@transitionend="showBlurhash = false">
	</div>
</template>
