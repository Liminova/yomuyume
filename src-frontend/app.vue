<script setup lang="ts">
import { NuxtPwaManifest } from "#components";
import Sonner from "~/components/ui/Sonner.vue";
import { useThemeWatcher } from "~/composables/use-theme";

import { useDrawerStoreWatcher } from "./composables/use-drawer-store";

useDrawerStoreWatcher();
useThemeWatcher();
</script>

<template>
	<NuxtLayout>
		<NuxtPwaManifest />
		<Sonner />
		<NuxtPage />
	</NuxtLayout>
</template>
