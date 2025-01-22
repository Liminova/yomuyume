<script setup lang="ts">
import { ref } from "vue";

import type { MyImage } from "~/lib/types";

import renderImage from "./ImagePoly/render-image";

const props = withDefaults(defineProps<{
	class?: string;
	draggable?: boolean;
	image: MyImage;
	imageClass?: string;
	lazy?: boolean;
}>(), {
	class: "",
	draggable: false,
	imageClass: "",
	lazy: true,
});

const blurhashUrl = ref("");
const imageUrl = ref("");
const imageFullyLoaded = ref(false);

const emit = defineEmits(["loaded"]);

function handleImageLoad(): void {
	imageFullyLoaded.value = true;
	emit("loaded");
}

renderImage(props.image, blurhashUrl, imageUrl);
</script>

<template>
	<div
		class="relative"
		:class="props.class">
		<!-- Blurhash placeholder -->
		<img
			v-if="props.image.blurhash && blurhashUrl && !imageFullyLoaded"
			:loading="props.lazy ? 'lazy' : 'eager'"
			class="left-0 top-0 -z-10"
			:style="{
				position: imageFullyLoaded ? 'absolute' : 'static',
			}"
			:class="props.imageClass"
			:src="blurhashUrl"
			:draggable="props.draggable">

		<!-- Actual image -->
		<img
			v-if="imageUrl"
			:loading="props.lazy ? 'lazy' : 'eager'"
			class="left-0 top-0"
			:style="{
				transition: 'opacity 0.5s ease',
				position: imageFullyLoaded ? 'static' : 'absolute',
			}"
			:src="imageUrl"
			:class="props.imageClass"
			:draggable="props.draggable"
			@load="handleImageLoad">
	</div>
</template>
