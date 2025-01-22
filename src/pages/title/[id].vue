<script setup lang="ts">
import { ref, watchEffect } from "vue";

import { useRoute } from "#app";
import { useGetTitle } from "~/composables/api/content";
import { useAddBookmark, useAddFavorite, useDeleteBookmark, useDeleteFavorite } from "~/composables/api/user";
import NavDrawerWrapper from "~/layouts/nav-drawer.vue";

const route = useRoute();
const idRaw = route.params.id;
const titleId = Array.isArray(idRaw) ? idRaw[0] : idRaw;
const snackbarMessage = ref("");
const snackbarTimeout = ref(5000);

const title = useGetTitle(titleId);
const addFavorite = useAddFavorite();
const deleteFavorite = useDeleteFavorite();
const addBookmark = useAddBookmark();
const deleteBookmark = useDeleteBookmark();

const isFavorite = ref(false);
const isBookmark = ref(false);

const favorites = ref(0);
const bookmarks = ref(0);

watchEffect(() => {
	if (!title.isSuccess.value || !title.data.value) { return; }

	isFavorite.value = title.data.value.is_favorite;
	isBookmark.value = title.data.value.is_bookmark;
	favorites.value = title.data.value.favorites;
	bookmarks.value = title.data.value.bookmarks;
});

function toggleBookmark(): void {
	if (isBookmark.value) {
		deleteBookmark.mutate(titleId);
	} else {
		addBookmark.mutate(titleId);
	}
}

function toggleFavorite(): void {
	if (isFavorite.value) {
		deleteFavorite.mutate(titleId);
	} else {
		addFavorite.mutate(titleId);
	}
}

// Fetching all infos
// void (async () => {
// 	const { data } = await indexApi.title(id);

// 	if (data === undefined) {
// 		await navigateTo("/404");
// 		return;
// 	}

// 	title.value = data;
// 	document.title = data.title;
// 	isFavorite.value = data.is_favorite ?? false;
// 	isBookmark.value = data.is_bookmark ?? false;
// 	favorites.value = data.favorites ?? BigInt(0);
// 	bookmarks.value = data.bookmarks ?? BigInt(0);
// })();

// async function toggleFavorite() {
// 	const { message, ok } = await userApi.favorite(titleId, isFavorite.value ? "DELETE" : "PUT");

// 	snackbarMessage.value = message ?? "";
// 	if (ok !== true) {
// 		return;
// 	}

// 	isFavorite.value = !isFavorite.value;
// 	favorites.value = favorites.value + (isFavorite.value ? BigInt(-1) : BigInt(1));
// }

// async function toggleBookmark() {
// 	const { message, ok } = await userApi.bookmark(titleId, isBookmark.value ? "DELETE" : "PUT");

// 	snackbarMessage.value = message ?? "";
// 	if (ok !== true) {
// 		return;
// 	}

// 	bookmarks.value = bookmarks.value + (isBookmark.value ? BigInt(-1) : BigInt(1));
// 	isBookmark.value = !isBookmark.value;
// }

const currPageIdx = ref(0);
// const pageObserver = new IntersectionObserver(
// 	(entries) => {
// 		for (const entry of entries) {
// 			if (!entry.isIntersecting) {
// 				continue;
// 			}

// 			const element = entry.target;

// 			currPageIdx.value = titleInfo.data.value.pages.findIndex((page) => page.id === element.id);
// 		}
// 	},
// 	{
// 		root: null,
// 		rootMargin: "0px",
// 		threshold: 0.5,
// 	}
// );

// async function saveProgress(currentPageIndex: number) {
// 	const { ok, message } = await userApi.progress(titleId, currentPageIndex);

// 	if (ok !== true) {
// 		snackbarMessage.value = message ?? "";
// 	}
// }

// watchEffect(() => {
// 	void debounce(saveProgress, 30000)(currPageIdx.value);
// });

// function handleImageLoad(pageId: string) {
// 	const element = document.getElementById(pageId);

// 	if (element === null) {
// 		return;
// 	}

// 	pageObserver.observe(element);
// }
</script>

<template>
	<div>
		<Snackbar
			:message="snackbarMessage"
			:timeout="snackbarTimeout"
			@close="snackbarMessage = ''" />
		<NavDrawerWrapper>
			<div class="mt-3 px-0 lg:mt-0 lg:pl-0 lg:pr-3">
				<!-- Basic infos -->
				<div class="mb-7 px-7 lg:px-0">
					<div class="text-6xl font-semibold">
						{{ title.data.value?.title }}
					</div>
					<div
						v-if="title.data.value?.description"
						class="mt-7 text-justify">
						{{ title.data.value?.description }}
					</div>
				</div>

				<!-- Like and fav buttons -->
				<div class="my-7 flex w-full grow-[100] flex-row items-center justify-center gap-7">
					<Button
						class="scale-125"
						@click="toggleBookmark">
						<i
							class="fa-bookmark"
							:class="{
								'fa-solid': isBookmark,
								'fa-light': !isBookmark,
							}" />
						{{ bookmarks > 0 ? bookmarks : "" }}
					</Button>
					<Button
						class="scale-125"
						@click="toggleFavorite">
						<i
							class="fa-heart"
							:class="{
								'fa-solid': isFavorite,
								'fa-light': !isFavorite,
							}" />
						{{ favorites > 0 ? favorites : "" }}
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
		</NavDrawerWrapper>
	</div>
</template>
