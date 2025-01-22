import { defineStore } from "pinia";
import { ref } from "vue";

export const homeStore = defineStore("home", () => {
	const coverHeight = ref(300);

	function setCoverHeight(newVal: number): void {
		coverHeight.value = newVal;
	}

	const recommendsContainerHeight = 500;
	const gapPixel = 18;
	const snackbarMessage = ref("");

	return {
		coverHeight,
		setCoverHeight,
		recommendsContainerHeight,
		gapPixel,
		snackbarMessage,
	};
});
