<script setup lang="ts">
import { Toaster } from "~/components/ui/sonner";
import { useTheme } from "~/composables/use-theme";
useTheme();

</script>

<template>
	<NuxtLayout>
		<Toaster />
		<NuxtPage />
	</NuxtLayout>
</template>
