<script setup lang="ts">
import { NuxtPwaManifest } from "#components";
import Sonner from "~/components/ui/Sonner.vue";
import { useTheme } from "~/composables/use-theme";

useTheme();
</script>

<template>
	<NuxtLayout>
		<NuxtPwaManifest />
		<Sonner />
		<NuxtPage />
	</NuxtLayout>
</template>
