<script setup lang="ts">
import "@material/web/chips/assist-chip.js";
import { homeStore } from "./utils";
import type { FilterItemServerResponse, TitleServerResponse } from "~/composables/api";
import ImagePoly from "~/components/ImagePoly.vue";
import { fileApiUrl, indexApi, utilsApi } from "~/composables/api";

const props = defineProps({
	previewTitle: {
		type: Object as () => FilterItemServerResponse,
		required: true,
	},
	isFirstTitle: { type: Boolean, default: true },
	isLastTitle: { type: Boolean, default: false },
});

const cover = {
	src: fileApiUrl.thumbnail(props.previewTitle.id),
	width: props.previewTitle.width,
	height: props.previewTitle.height,
	format: props.previewTitle.format,
	blurhash: props.previewTitle.blurhash,
};

const store = homeStore();
const fullTitle: Ref<TitleServerResponse> = ref({}) as Ref<TitleServerResponse>;
const titleTagNames = ref<Array<string>>([]);
const tagList = ref<Array<[number, string]>>([]);

void (async () => {
	const tagResp = await utilsApi.tags();

	if (tagResp.data === undefined) {
		store.snackbarMessage = tagResp.message ?? "";
		return;
	}

	tagList.value = tagResp.data;

	const resp = await indexApi.title(props.previewTitle.id);

	if (resp.data === undefined) {
		store.snackbarMessage = resp.message ?? "";
		return;
	}

	fullTitle.value = resp.data;

	titleTagNames.value = resp.data.tag_ids.map((tagId) => {
		const tagName = tagList.value.find((tag) => tag[0] === tagId);

		return tagName ? tagName[1] : "";
	});
})();

/** */
</script>

<template>
	<nuxt-link
		:to="`/title/${props.previewTitle.id}`"
		class="relative flex h-full flex-row justify-center overflow-hidden bg-black/50 sm:static"
		:class="{ 'rounded-l-3xl': props.isFirstTitle, 'rounded-r-3xl': props.isLastTitle }"
	>
		<!-- Background -->
		<div
			class="absolute left-0 top-0 z-[-1] hidden w-full overflow-hidden sm:block"
			:class="{ 'rounded-l-3xl': props.isFirstTitle, 'rounded-r-3xl': props.isLastTitle }"
			:style="{ height: `${store.recommendsContainerHeight}px` }"
		>
			<ImagePoly
				class="w-full scale-110 overflow-hidden object-cover blur-sm"
				:draggable="false"
				:image="cover"
				image-class="overflow-hidden"
				:lazy="false"
			/>
		</div>

		<!-- Cover -->
		<div class="size-full sm:min-w-[350px] sm:max-w-xs lg:py-10 lg:pl-10">
			<ImagePoly
				:draggable="false"
				:image="cover"
				class="h-full overflow-hidden lg:rounded-2xl"
				image-class="h-full object-cover"
			/>
		</div>

		<div
			class="pointer-events-none absolute left-0 top-0 flex size-full bg-black/50 sm:hidden"
		/>

		<!-- Informations -->
		<div
			class="absolute left-0 top-0 z-[1] flex size-full flex-col justify-end p-7 sm:static sm:z-auto sm:max-w-3xl sm:justify-start sm:bg-transparent sm:p-10"
		>
			<div class="text-lg font-light" data-theme="dark">
				{{ props.previewTitle.author ?? "Unknown" }}
			</div>
			<div class="truncate-2 mb-1 text-balance text-3xl font-bold" data-theme="dark">
				{{ props.previewTitle.title }}
			</div>

			<div class="truncate">
				{{ props.previewTitle.release_date }}
			</div>

			<div v-if="titleTagNames.includes(`completed`)" class="mb-2" data-theme="dark">
				<i class="fa-solid fa-circle-check mr-2" />
				<span>Completed</span>
			</div>

			<div
				v-if="titleTagNames.length !== 0"
				class="mb-2 hidden flex-row flex-wrap gap-2 sm:flex"
			>
				<span v-for="tag in titleTagNames" :key="tag">
					<md-assist-chip :key="tag" :label="tag" class="elevation-3" />
				</span>
			</div>

			<div
				v-if="fullTitle.description"
				class="truncate-5 sm:truncate-8 z-[1] overflow-hidden"
				data-theme="dark"
			>
				{{ fullTitle.description }}
			</div>
		</div>
	</nuxt-link>
</template>
