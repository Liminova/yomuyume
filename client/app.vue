<script setup lang="ts">
import { NuxtPwaManifest } from '#components'
import { initIsJxlNative } from './components/image/is-jxl-native'
import { initDrawerStore } from './composables/use-drawer-store'
import Sonner from '~/components/ui/Sonner.vue'
import { initTheme } from '~/composables/use-theme'

initDrawerStore()
initTheme()
await initIsJxlNative()
</script>

<template>
	<NuxtLayout>
		<NuxtPwaManifest />
		<Sonner />
		<NuxtPage />
	</NuxtLayout>
</template>
