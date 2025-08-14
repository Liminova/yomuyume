<script setup lang="ts">
import { useRoute, useRouter } from "nuxt/app";
import { ref, watchEffect } from "vue";

import { definePageMeta } from "#imports";
import TitleInteractionArea from "~/components/title/TitleInteractionArea.vue";
import TitlePagesArea from "~/components/title/TitlePagesArea.vue";
import Button from "~/components/ui/Button.vue";
import { useContentGetTitle } from "~/composables/api/content";

definePageMeta({ layout: "nav-drawer", middleware: ["auth"] });

const titleId = useRoute().params.id as string;
const title = useContentGetTitle(titleId);

const [route, router] = [useRoute(), useRouter()];
const activeChapterId = ref(route.params.chapterId as string | undefined);
function setActiveChapter(chapterId: string): void {
	activeChapterId.value = chapterId;
	void router.push({ query: { chapterId } });
}

watchEffect(() => {
	if (title.data.value?.chapters?.length === 1) {
		activeChapterId.value = title.data.value.chapters[0].id;
	}
});
</script>

<template>
	<div class="mt-3 px-0 lg:mt-0 lg:pl-0 lg:pr-3">
		<!-- basic infos -->
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

		<!-- like & fav buttons -->
		<TitleInteractionArea
			:title-id="titleId"
			:bookmarks="title.data.value?.bookmarks"
			:favorites="title.data.value?.favorites"
			:is-bookmark="title.data.value?.is_bookmark"
			:is-favorite="title.data.value?.is_favorite" />

		<!-- chapters -->
		<div
			v-if="title.data.value?.chapters !== undefined && title.data.value.chapters.length > 1"
			class="grid grid-cols-[repeat(auto-fill,minmax(6rem,1fr))] gap-4">
			<Button
				v-for="chapter in title.data.value?.chapters"
				:key="chapter.id"
				variant="outline"
				@click="setActiveChapter(chapter.id)">
				<div class="flex items-center gap-2">
					Chapter&nbsp;
					{{ chapter.number }}
					{{ chapter.description ? ` - ${chapter.description}` : "" }}
				</div>
			</Button>
		</div>

		<!-- Pages -->
		<TitlePagesArea
			v-if="activeChapterId"
			:title-id="titleId"
			:chapter-ids="title.data.value?.chapters ?? []"
			:chapter-id="activeChapterId" />
	</div>
</template>
