import { useScreenSize } from './use-screen-size'
import { useDebounceFn } from '@vueuse/core'
import { useRoute, useState } from 'nuxt/app'
import { type Ref, onMounted, onUnmounted, watchEffect } from 'vue'

export enum DrawerKind {
	AlwaysVisible = 'always-visible', // desktop
	OffScreen = 'off-screen' // mobile
}

export interface DrawerStore {
	kind: DrawerKind
	expanded: boolean
	topBarVisible: boolean
}

export function useDrawerStore(): Ref<DrawerStore> {
	return useState<DrawerStore>('drawer-store', () => ({
		expanded: true,
		kind: DrawerKind.AlwaysVisible,
		topBarVisible: true
	}))
}

/** Run this function once in `app.vue` */
export function initDrawerStore(): void {
	const drawerStore = useDrawerStore()
	const screenSize = useScreenSize()
	const route = useRoute()

	watchEffect(() => {
		if (route.path.startsWith('/title') || screenSize.value.width < 1024) {
			drawerStore.value.kind = DrawerKind.OffScreen
			return
		}
		drawerStore.value.kind = DrawerKind.AlwaysVisible
	})

	const controller = new AbortController()
	onMounted(() => {
		if (window.innerWidth < 1280) drawerStore.value.expanded = false

		// auto show/hide top bar on scroll
		let prevScrollPos = -document.body.getBoundingClientRect().top
		function toggleTopBar(): void {
			if (window.innerWidth >= 1024) {
				drawerStore.value.topBarVisible = true
				return
			}
			const currentScrollPos = -document.body.getBoundingClientRect().top

			if (prevScrollPos > currentScrollPos || currentScrollPos < 100) {
				drawerStore.value.topBarVisible = true
			} else {
				drawerStore.value.topBarVisible = false
				drawerStore.value.expanded = false
			}
			prevScrollPos = currentScrollPos
		}

		const debouncedToggleTopBar = useDebounceFn(toggleTopBar, 50)

		window.addEventListener('scroll', debouncedToggleTopBar, {
			signal: controller.signal
		})
		window.addEventListener('resize', debouncedToggleTopBar, {
			signal: controller.signal
		})
	})
	onUnmounted(() => {
		controller.abort()
	})
}
