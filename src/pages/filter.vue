<script setup lang="ts">
import debounce from "debounce";
import { onMounted, ref } from "vue";

import { NuxtLink } from "#components";
import { definePageMeta } from "#imports";
import ChipSection from "~/components/filter/ChipSection.vue";
import {
	FilterReadingStatus,
	FilterSortBy,
	FilterSortOrder,
	FilterType,
} from "~/components/filter/FilterType";
import ItemCard from "~/components/ItemCard.vue";
import Toggle from "~/components/Toggle.vue";
import Input from "~/components/ui/input/Input.vue";
import { type FilterTitleResponseBody, useGetCategories } from "~/composables/api/content";
import { toCoverApiEndpoint } from "~/composables/api/file";
import { getSwiperBreakpoint } from "~/lib/swiper-break-points";

definePageMeta({
	layout: "nav-drawer",
});

// Key: category id, Value: category name
// const categories = ref<Record<string, string>>({});
// const snackbarMessage = ref("");

// void (async () => {
// 	const { data, message } = await indexApi.categories();

// 	if (data === undefined) {
// 		snackbarMessage.value = message ?? "";
// 		return;
// 	}

// 	for (const category of data) {
// 		categories.value[category.id] = category.name;
// 	}
// })();

const categories = useGetCategories();

// For the result grid styling =================================================

const imageContainerRef = ref<HTMLElement | null>(null);
const imagePerRow = ref(5);
const spaceBetween = ref(16);

// Results =====================================================================

const filteredTitles = ref<FilterTitleResponseBody>([]); /** found titles */
const filteredTitlesToDisplay = ref<FilterTitleResponseBody>([]);

function renderMoreResult(): void {
	const howFarFromBottom = document.body.getBoundingClientRect().bottom - window.innerHeight;

	if (howFarFromBottom < 200) {
		const newTitles = filteredTitles.value.slice(
			filteredTitlesToDisplay.value.length,
			filteredTitlesToDisplay.value.length + imagePerRow.value * 3,
		);

		filteredTitlesToDisplay.value = filteredTitlesToDisplay.value.concat(newTitles);
	}
}

const observer = new ResizeObserver(() => {
	const breakPoint = getSwiperBreakpoint();

	imagePerRow.value = breakPoint.slidesPerView;
	spaceBetween.value = breakPoint.spaceBetween;
});

onMounted(() => {
	window.addEventListener("scroll", debounce(renderMoreResult, 50));
	if (imageContainerRef.value === null) {
		return;
	}

	observer.observe(imageContainerRef.value);
});

// Chips variables =============================================================

const keywords = ref<string>("");
const inCategories = ref(new Set<string>());
const readingStatus = ref<string[]>([]);
const sortBy = ref("");
const sortOrder = ref("");

function chipCategoryHandler(eventTarget: HTMLElement): void {
	const uuid = eventTarget.getAttribute("uuid") ?? "";
	const selected = eventTarget.getAttribute("selected") === null;

	if (selected) {
		inCategories.value.add(uuid);
	} else {
		inCategories.value.delete(uuid);
	}
}

// watchEffect(async () => {
// 	const { data, message } = await indexApi.filter({
// 		keywords: keywords.value
// 			.split(" ")
// 			.map((keyword) => keyword.trim())
// 			.filter((keyword) => keyword !== ""),
// 		category_ids: Array.from(inCategories.value),
// 		is_reading: readingStatus.value.includes(FilterReadingStatus.Reading.name),
// 		is_finished: readingStatus.value.includes(FilterReadingStatus.Finished.name),
// 		is_bookmarked: readingStatus.value.includes(FilterReadingStatus.Bookmarked.name),
// 		is_favorite: readingStatus.value.includes(FilterReadingStatus.Liked.name),
// 		sort_by: sortBy.value,
// 		sort_order: sortOrder.value,
// 	});

// 	if (data === undefined) {
// 		snackbarMessage.value = message ?? "";
// 		return;
// 	}

// 	filteredTitles.value = data;
// 	filteredTitlesToDisplay.value = filteredTitles.value.slice(0, imagePerRow.value * 3);
// });
</script>

<template>
	<div class="mb-10 mt-3 flex w-full flex-col px-6 lg:mt-0 lg:pl-0 lg:pr-3">
		<!-- Filter region -->
		<div class="flex w-full flex-col gap-2">
			<div class="text-xl font-semibold" />
			<Input
				v-model="keywords"
				label="filter by keywords"
				value=""
				class="my-4 max-w-sm" />

			<ChipSection
				title="Status"
				:filter-type="FilterType.ReadingStatus"
				:filter-type-posible-val="FilterReadingStatus"
				@add="readingStatus.push($event)"
				@delete="readingStatus.splice(readingStatus.indexOf($event), 1)" />

			<ChipSection
				title="Sort by"
				:filter-type="FilterType.SortResult"
				:filter-type-posible-val="FilterSortBy"
				is-overwrite
				@overwrite="sortBy = $event" />

			<ChipSection
				title="Sort order"
				:filter-type="FilterType.SortOrder"
				:filter-type-posible-val="FilterSortOrder"
				is-overwrite
				@overwrite="sortOrder = $event" />

			<div class="flex flex-row flex-wrap items-center gap-4">
				<div class="text-xl font-semibold">
					in category
				</div>
				<!-- <md-chip-set class="flex-rows flex">
						<md-filter-chip
							v-for="{ id, name } in categories.data.value"
							:key="id"
							:uuid="id"
							:label="name"
							@click="chipCategoryHandler($event.target)" />
					</md-chip-set> -->
			</div>
		</div>

		<!-- Result region -->
		<Toggle :show="filteredTitles.length > 0">
			<div class="mb-8 mt-10 text-4xl font-bold">
				Here's what I found
			</div>
		</Toggle>
		<Toggle :show="filteredTitles.length === 0">
			<div class="mb-8 mt-10 text-4xl font-bold">
				Can't find anything
			</div>
		</Toggle>
		<div
			ref="imageContainerRef"
			class="grid"
			:style="{
				gridTemplateColumns: `repeat(${imagePerRow}, 1fr)`,
				gap: `${spaceBetween}px`,
			}">
			<NuxtLink
				v-for="title in filteredTitlesToDisplay"
				:key="title.id"
				:to="`/title/${title.id}`">
				<ItemCard
					:author="title.author ?? 'Unknown'"
					:cover="{
						src: toCoverApiEndpoint(title.id),
						width: title.cover_width,
						height: title.cover_height,
						blurhash: title.cover_blurhash,
						format: title.cover_format,
					}"
					:progress="title.page_read ? title.page_read / title.page_count : 0"
					:title="title.title"
					:title-id="title.id" />
			</NuxtLink>
		</div>
	</div>
</template>
