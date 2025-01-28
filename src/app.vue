<script setup lang="ts">
import Sonner from "~/components/ui/Sonner.vue";
import { useTheme } from "~/composables/use-theme";
useTheme();

</script>

<template>
	<NuxtLayout>
		<Sonner />
		<NuxtPage />
	</NuxtLayout>
</template>
