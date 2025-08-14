<script setup lang="ts">

import { definePageMeta } from "#imports";
import TitleCard from "~/components/title/TitleCard.vue";
import { BookmarkedTitlesQuery, useContentSearchTitle } from "~/composables/api/content";

definePageMeta({ layout: "nav-drawer", middleware: ["auth"] });

const titles = useContentSearchTitle(BookmarkedTitlesQuery);
</script>

<template>
	<div class="grid grid-cols-[repeat(auto-fill,minmax(12rem,1fr))] gap-4">
		<div
			v-for="title in titles.flattened.value"
			:key="title.id">
			<TitleCard :title="title" />
		</div>
	</div>
</template>
