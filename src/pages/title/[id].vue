<script setup lang="ts">
import { Bookmark, Heart } from "lucide-vue-next";
import { ref, watchEffect } from "vue";

import { useRoute } from "#app";
import { definePageMeta } from "#imports";
import Button from "~/components/ui/Button.vue";
import { useGetTitle } from "~/composables/api/content";
import { useAddBookmark, useAddFavorite, useDeleteBookmark, useDeleteFavorite } from "~/composables/api/user";

definePageMeta({ layout: "nav-drawer" });

const title = useGetTitle((useRoute().params.id as string));

const [addFavorite, deleteFavorite] = [useAddFavorite(), useDeleteFavorite()];
const [addBookmark, deleteBookmark] = [useAddBookmark(), useDeleteBookmark()];

const [favorites, bookmarks] = [ref<number>(0), ref<number>(0)];
const [isFavorite, isBookmark] = [ref(false), ref(false)];

watchEffect(() => {
	if (title.isPending.value || !title.data.value) { return; }

	isFavorite.value = title.data.value.is_favorite;
	isBookmark.value = title.data.value.is_bookmark;
	favorites.value = title.data.value.favorites ?? 0;
	bookmarks.value = title.data.value.bookmarks ?? 0;
});

function toggleBookmark(): void {
	if (isBookmark.value) {
		deleteBookmark.mutate((useRoute().params.id as string));
		bookmarks.value--;
		isBookmark.value = false;
	} else {
		addBookmark.mutate((useRoute().params.id as string));
		bookmarks.value++;
		isBookmark.value = true;
	}
}

function toggleFavorite(): void {
	if (isFavorite.value) {
		deleteFavorite.mutate((useRoute().params.id as string));
		favorites.value--;
		isFavorite.value = false;
	} else {
		addFavorite.mutate((useRoute().params.id as string));
		favorites.value++;
		isFavorite.value = true;
	}
}
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
		<div class="my-7 flex w-full flex-row items-center justify-start gap-2">
			<Button
				:variant="isBookmark ? 'secondary' : 'outline'"
				size="lg"
				@click="toggleBookmark">
				<Bookmark :fill="isBookmark ? 'white' : 'transparent'" />
				{{ bookmarks === 0 ? "" : bookmarks }}
			</Button>
			<Button
				:variant="isFavorite ? 'secondary' : 'outline'"
				size="lg"
				@click="toggleFavorite">
				<Heart :fill="isFavorite ? 'white' : 'transparent'" />
				{{ favorites === 0 ? "" : favorites }}
			</Button>
		</div>

		<!-- Pages -->
		<!-- <div
					v-for="page in title.data.value?.pages"
					:id="page.id"
					:key="page.id"
					class="mx-auto max-w-[700px]">
					<ImagePoly
						:image="{
							src: fileApiUrl.page(page.id),
							format: page.format,
						}"
						@loaded="handleImageLoad(page.id)" />
				</div> -->
	</div>
</template>
