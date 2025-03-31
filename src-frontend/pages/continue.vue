<script setup lang="ts">
import { definePageMeta } from "#imports";

definePageMeta({ layout: "nav-drawer", middleware: ["auth"] });
</script>

<template>
	<div>Continue reading</div>
</template>
