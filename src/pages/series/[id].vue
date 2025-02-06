<script setup lang="ts">
import { useRoute } from "#app";
import { definePageMeta } from "#imports";
import TitleInteractionArea from "~/components/Title/TitleInteractionArea.vue";
import { useGetSeries } from "~/composables/api/content";

definePageMeta({ layout: "nav-drawer" });

const titleID = useRoute().params.id as string;
const title = useGetSeries(titleID);
</script>

<template>
	<div class="mt-3 px-0 lg:mt-0 lg:pl-0 lg:pr-3">
		<!-- Basic infos -->
		<div class="mb-7 px-7 lg:px-0">
			<div class="text-6xl font-semibold">
				{{ title.data.value?.title ?? "Untitled" }}
			</div>
			<div
				v-if="title.data.value?.description"
				class="mt-7 text-justify">
				{{ title.data.value?.description }}
			</div>
		</div>

		<!-- Like and fav buttons -->
		<TitleInteractionArea
			:title-id="titleID"
			:bookmarks="title.data.value?.bookmarks"
			:favorites="title.data.value?.favorites"
			:is-bookmark="title.data.value?.is_bookmark"
			:is-favorite="title.data.value?.is_favorite" />

		<!-- Chapters -->
		<div

			v-if="title.data.value">
			<div
				v-for="chapter in title.data.value?.chapters"
				:key="chapter.id">
				<div>
					Chapter {{ chapter.number }}
					{{ chapter.description ? ` - ${chapter.description}` : "" }}
				</div>
			</div>
		</div>
	</div>
</template>
