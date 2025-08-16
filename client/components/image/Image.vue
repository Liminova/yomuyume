<script setup lang="ts">
import { decodeBlurhash } from './decode-blurhash'
import { decodeJpegXL } from './decode-jpegxl'
import { useIsJxlNative } from './is-jxl-native'
import { onMounted, onUnmounted, ref, watchEffect } from 'vue'
import { toast } from 'vue-sonner'
import { cn } from '~/lib/utils'

const emit = defineEmits(['in-view'])
const props = withDefaults(
	defineProps<{
		id: string
		class?: string
		draggable?: boolean
		emitWhenInView?: boolean

		src: string
		blurhash?: string
		width?: number
		height?: number
		isJxl?: boolean
	}>(),
	{
		class: undefined,
		draggable: false,
		emitWhenInView: false,

		blurhash: undefined,
		width: 0,
		height: 0,
		isJxl: false
	}
)

const showBlurhash = ref(props.blurhash !== undefined)
const imageLoaded = ref(false)

const blurhashImgURL = ref<string | null>(null)
const realImgURL = ref<string | null>(null)
const error = ref<string | null>(null)

if (props.blurhash && props.width > 0 && props.height > 0) {
	decodeBlurhash(
		{
			id: props.id,
			blurhash: props.blurhash,
			width: props.width,
			height: props.height
		},
		blurhashImgURL,
		error
	)
}

if (props.isJxl && !useIsJxlNative().value) {
	decodeJpegXL(
		{
			id: props.id,
			url: props.src
		},
		realImgURL,
		error
	)
} else {
	realImgURL.value = props.src
}

// emit signal when in view
const container = ref<HTMLElement>()
const observer = new IntersectionObserver(
	([entry]) => {
		if (entry.isIntersecting) {
			emit('in-view')
		}
	},
	{ rootMargin: '0px 0px -100px 0px' }
)
onMounted(() => {
	if (!container.value || !props.emitWhenInView) {
		return
	}
	observer.observe(container.value)
})
onUnmounted(() => {
	observer.disconnect()
})

watchEffect(() => {
	if (!error.value) {
		return
	}
	toast.error(error.value)
})
</script>

<template>
	<div ref="container" class="relative size-full">
		<!-- Blurhash placeholder -->
		<img
			v-if="blurhashImgURL && showBlurhash"
			loading="lazy"
			:class="cn(imageLoaded && 'absolute left-0 top-0', props.class)"
			:src="blurhashImgURL"
			:draggable="props.draggable"
		/>

		<!-- Actual image -->
		<img
			loading="lazy"
			:class="
				cn(
					'transition-opacity',
					!imageLoaded && 'opacity-0 absolute left-0 top-0',
					props.class
				)
			"
			:src="realImgURL === null ? undefined : realImgURL"
			:draggable="props.draggable"
			@load="imageLoaded = true"
			@transitionend="showBlurhash = false"
		/>
	</div>
</template>
