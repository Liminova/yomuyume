<script setup lang="ts">
import "@material/web/button/text-button.js";
import debounce from "debounce";
import type { TitleResponseBody } from "~/composables/bridge";
import { indexApi, userApi, fileApiUrl } from "~/composables/api";
import NavDrawerWrapper from "~/layouts/NavDrawerWrapper.vue";

const route = useRoute();
const idRaw = route.params.id;
const id = Array.isArray(idRaw) ? idRaw[0] : idRaw;
const snackbarMessage = ref("");
const snackbarTimeout = ref(5000);

const title = ref({}) as Ref<TitleResponseBody>;

const isFavorite = ref(false);
const isBookmark = ref(false);
const favorites: Ref<bigint> = ref(BigInt(0));
const bookmarks: Ref<bigint> = ref(BigInt(0));

// Fetching all infos
void (async () => {
	const { data } = await indexApi.title(id);

	if (data === undefined) {
		await navigateTo("/404");
		return;
	}

	title.value = data;
	document.title = data.title;
	isFavorite.value = data.is_favorite ?? false;
	isBookmark.value = data.is_bookmark ?? false;
	favorites.value = data.favorites ?? BigInt(0);
	bookmarks.value = data.bookmarks ?? BigInt(0);
})();

async function toggleFavorite() {
	const { message, ok } = await userApi.favorite(id, isFavorite.value ? "DELETE" : "PUT");

	snackbarMessage.value = message ?? "";
	if (ok !== true) {
		return;
	}

	isFavorite.value = !isFavorite.value;
	favorites.value = favorites.value + (isFavorite.value ? BigInt(-1) : BigInt(1));
}

async function toggleBookmark() {
	const { message, ok } = await userApi.bookmark(id, isBookmark.value ? "DELETE" : "PUT");

	snackbarMessage.value = message ?? "";
	if (ok !== true) {
		return;
	}

	bookmarks.value = bookmarks.value + (isBookmark.value ? BigInt(-1) : BigInt(1));
	isBookmark.value = !isBookmark.value;
}

const currentPageIndex = ref(0);
const pageObservers = new IntersectionObserver(
	(entries) => {
		for (const entry of entries) {
			if (!entry.isIntersecting) {
				continue;
			}

			const element = entry.target;

			currentPageIndex.value = title.value.pages.findIndex((page) => page.id === element.id);
		}
	},
	{
		root: null,
		rootMargin: "0px",
		threshold: 0.5,
	}
);

async function saveProgress(currentPageIndex: number) {
	const { ok, message } = await userApi.progress(id, currentPageIndex);

	if (ok !== true) {
		snackbarMessage.value = message ?? "";
	}
}

watchEffect(() => {
	void debounce(saveProgress, 30000)(currentPageIndex.value);
});

function handleImageLoad(pageId: string) {
	const element = document.getElementById(pageId);

	if (element === null) {
		return;
	}

	pageObservers.observe(element);
}
</script>

<template>
	<div>
		<Snackbar
			:message="snackbarMessage"
			:timeout="snackbarTimeout"
			@close="snackbarMessage = ''"
		/>
		<NavDrawerWrapper>
			<div class="mt-3 px-0 lg:mt-0 lg:pl-0 lg:pr-3">
				<!-- Basic infos -->
				<div class="mb-7 px-7 lg:px-0">
					<div class="text-6xl font-semibold">{{ title.title }}</div>
					<div v-if="title.description" class="mt-7 text-justify">
						{{ title.description }}
					</div>
				</div>

				<!-- Like and fav buttons -->
				<div class="my-7 flex w-full grow-[100] flex-row items-center justify-center gap-7">
					<md-text-button class="scale-125" @click="toggleBookmark">
						<i
							class="fa-bookmark"
							:class="{
								'fa-solid': isBookmark,
								'fa-light': !isBookmark,
							}"
						/>
						{{ bookmarks > 0 ? bookmarks : "" }}
					</md-text-button>
					<md-text-button class="scale-125" @click="toggleFavorite">
						<i
							class="fa-heart"
							:class="{
								'fa-solid': isFavorite,
								'fa-light': !isFavorite,
							}"
						/>
						{{ favorites > 0 ? favorites : "" }}
					</md-text-button>
				</div>

				<!-- Pages -->
				<div
					v-for="page in title.pages"
					:key="page.id"
					:id="page.id"
					class="mx-auto max-w-[700px]"
				>
					<ImagePoly
						@loaded="handleImageLoad(page.id)"
						:image="{
							src: fileApiUrl.page(page.id),
							format: page.format,
						}"
					/>
				</div>
			</div>
		</NavDrawerWrapper>
	</div>
</template>
