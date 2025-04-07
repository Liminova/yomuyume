<script setup lang="ts">
import { VueQueryDevtools } from "@tanstack/vue-query-devtools";

import { NuxtPwaManifest } from "#components";
import Sonner from "~/components/ui/Sonner.vue";
import { useTheme } from "~/composables/use-theme";
useTheme();
</script>

<template>
	<NuxtLayout>
		<NuxtPwaManifest />
		<VueQueryDevtools />
		<Sonner />
		<NuxtPage />
	</NuxtLayout>
</template>
