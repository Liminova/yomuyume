import { useDebounceFn } from "@vueuse/core";
import { useRoute } from "nuxt/app";
import { defineStore } from "pinia";
import { onMounted, onUnmounted, ref, watchEffect } from "vue";

import { useScreenSize } from "./use-screen-size";

/* eslint-disable no-unused-vars */

export enum DrawerState {
	Expanded = "expanded",
	Collapsed = "collapsed",
}

export enum DrawerKind {
	AlwaysVisible = "always-visible", // desktop
	OffScreen = "off-screen", // mobile
}

/* eslint-enable no-unused-vars */

export const useNavDrawerStore = defineStore("nav-drawer-store", () => {
	const isDrawerExpanded = ref(true);
	const isTopBarVisible = ref(true);

	const kind = ref<DrawerKind>(DrawerKind.AlwaysVisible);
	const state = ref<DrawerState>(DrawerState.Expanded);

	const route = useRoute();
	const screenSize = useScreenSize();
	watchEffect(() => {
		if (route.path.startsWith("/title") || screenSize.width < 1024) {
			kind.value = DrawerKind.OffScreen;
			return;
		}
		kind.value = DrawerKind.AlwaysVisible;
	});

	const controller = new AbortController();
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

		const debouncedToggleTopBar = useDebounceFn(toggleTopBar, 50);

		window.addEventListener("scroll", debouncedToggleTopBar, { signal: controller.signal });
		window.addEventListener("resize", debouncedToggleTopBar, { signal: controller.signal });
	});
	onUnmounted(() => {
		controller.abort();
	});

	return {
		navDrawerStyle: kind,
		navDrawerState: state,
		isDrawerExpanded,
		isTopBarVisible,
	};
});
