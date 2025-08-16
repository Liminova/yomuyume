import { useState } from 'nuxt/app'
import { getCurrentScope, onMounted, onUnmounted, type Ref } from 'vue'

export interface ScreenSizeState {
	width: number
	height: number
}

export function useScreenSize(): Ref<ScreenSizeState, ScreenSizeState> {
	if (!getCurrentScope()) {
		throw new Error('useScreenSize must be used within a component')
	}

	const states = useState<ScreenSizeState>('screen-size', () => ({
		width: window.innerWidth,
		height: window.innerHeight
	}))

	const observer = new ResizeObserver(() => {
		states.value.width = window.innerWidth
		states.value.height = window.innerHeight
	})

	onMounted(() => {
		observer.observe(window.document.body)
	})

	onUnmounted(() => {
		observer.disconnect()
	})

	return states
}
