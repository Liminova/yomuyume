import debounce from "debounce";
import { defineStore } from "pinia";
import { onMounted, ref } from "vue";

export const useNavDrawerStore = defineStore("nav-drawer-store", () => {
	const isDrawerExpanded = ref(true);
	const isTopBarVisible = ref(true);

	onMounted(() => {
		if (window.innerWidth < 1280) {
			isDrawerExpanded.value = false;
		}

		// auto show/hide top bar on scroll
		let prevScrollPos = -document.body.getBoundingClientRect().top;
		function toggleTopBar(): void {
			if (window.innerWidth >= 1024) {
				isTopBarVisible.value = true;
				return;
			}
			const currentScrollPos = -document.body.getBoundingClientRect().top;
			if (prevScrollPos > currentScrollPos || currentScrollPos < 100) {
				isTopBarVisible.value = true;
			} else {
				isTopBarVisible.value = false;
				isDrawerExpanded.value = false;
			}
			prevScrollPos = currentScrollPos;
		}

		window.onscroll = debounce(toggleTopBar, 0);
		window.onresize = debounce(toggleTopBar, 100);
	});

	return {
		isDrawerExpanded,
		isTopBarVisible,
	};
});
