import { defineStore } from "pinia";
import { onMounted, onUnmounted, ref } from "vue";

export const useScreenSize = defineStore("screen-size-store", () => {
	const width = ref(window.innerWidth);
	const height = ref(window.innerHeight);

	const observer = new ResizeObserver(() => {
		width.value = window.innerWidth;
		height.value = window.innerHeight;
	});

	onMounted(() => {
		observer.observe(window.document.body);
	});

	onUnmounted(() => {
		observer.disconnect();
	});

	return {
		width,
		height,
	};
});
