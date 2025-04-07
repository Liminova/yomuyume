<script setup lang="ts">
import { computed } from "vue";

import { definePageMeta } from "#imports";
import TitleCard from "~/components/title/TitleCard.vue";
import { FavoriteTitlesQuery, useContentSearchTitle } from "~/composables/api/content";

definePageMeta({ layout: "nav-drawer", middleware: ["auth"] });

const titlesInf = useContentSearchTitle(FavoriteTitlesQuery);
const titles = computed(() => titlesInf.data.value?.pages.flatMap(page => page.data ?? []) ?? []);
</script>

<template>
	<div class="grid grid-cols-[repeat(auto-fill,minmax(12rem,1fr))] gap-4">
		<div
			v-for="title in titles"
			:key="title.id">
			<TitleCard :title="title" />
		</div>
	</div>
</template>
