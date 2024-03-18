<script setup lang="ts">
import "@material/web/textfield/filled-text-field.js";
import "@material/web/chips/chip-set.js";
import "@material/web/chips/filter-chip.js";
import "@material/web/radio/radio.js";
import debounce from "debounce";
import ChipSection from "~/components/filter/ChipSection.vue";
import {
	FilterReadingStatus,
	FilterSortBy,
	FilterSortOrder,
	FilterType,
} from "~/components/filter/FilterType";
import { getSwiperBreakpoint } from "~/composables/swiperBreakPoint";
import NavDrawerWrapper from "~/layouts/NavDrawerWrapper.vue";

// Key: category id, Value: category name
const categories = ref<Record<string, string>>({});
const snackbarMessage = ref("");

void (async () => {
	const { data, message } = await indexApi.categories();

	if (data === undefined) {
		snackbarMessage.value = message ?? "";
		return;
	}

	for (const category of data) {
		categories.value[category.id] = category.name;
	}
})();

// For the result grid styling =================================================

const imageContainerRef = ref<HTMLElement | null>(null);
const imagePerRow = ref(5);
const spaceBetween = ref(16);

// Results =====================================================================

const filteredTitles: Ref<Array<FilterItemServerResponse>> = ref([]); /** found titles */
const filteredTitlesToDisplay: Ref<Array<FilterItemServerResponse>> = ref([]);

function renderMoreResult() {
	const howFarFromBottom = document.body.getBoundingClientRect().bottom - window.innerHeight;

	if (howFarFromBottom < 200) {
		const newTitles = filteredTitles.value.slice(
			filteredTitlesToDisplay.value.length,
			filteredTitlesToDisplay.value.length + imagePerRow.value * 3
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
const readingStatus = ref<Array<string>>([]);
const sortBy = ref("");
const sortOrder = ref("");

function chipCategoryHandler(eventTarget: HTMLElement) {
	const uuid = eventTarget.getAttribute("uuid") ?? "";
	const selected = eventTarget.getAttribute("selected") === null;

	if (selected) {
		inCategories.value.add(uuid);
	} else {
		inCategories.value.delete(uuid);
	}
}

watchEffect(async () => {
	const { data, message } = await indexApi.filter({
		keywords: keywords.value
			.split(" ")
			.map((keyword) => keyword.trim())
			.filter((keyword) => keyword !== ""),
		category_ids: Array.from(inCategories.value),
		is_reading: readingStatus.value.includes(FilterReadingStatus.Reading.name),
		is_finished: readingStatus.value.includes(FilterReadingStatus.Finished.name),
		is_bookmarked: readingStatus.value.includes(FilterReadingStatus.Bookmarked.name),
		is_favorite: readingStatus.value.includes(FilterReadingStatus.Liked.name),
		sort_by: sortBy.value,
		sort_order: sortOrder.value,
	});

	if (data === undefined) {
		snackbarMessage.value = message ?? "";
		return;
	}

	filteredTitles.value = data;
	filteredTitlesToDisplay.value = filteredTitles.value.slice(0, imagePerRow.value * 3);
});

document.title = "Yomuyume - Filter";
</script>

<template>
	<div>
		<Snackbar :message="snackbarMessage" @close="snackbarMessage = ''" />
		<NavDrawerWrapper class="mb-10 mt-3 flex w-full flex-col px-6 lg:mt-0 lg:pl-0 lg:pr-3">
			<!-- Filter region -->
			<div class="flex w-full flex-col gap-2">
				<div class="text-xl font-semibold"></div>
				<md-filled-text-field
					v-model="keywords"
					label="filter by keywords"
					value=""
					class="my-4 max-w-sm"
				/>

				<ChipSection
					title="Status"
					:filter-type="FilterType.ReadingStatus"
					:filter-type-posible-val="FilterReadingStatus"
					@add="readingStatus.push($event)"
					@delete="readingStatus.splice(readingStatus.indexOf($event), 1)"
				/>

				<ChipSection
					title="Sort by"
					:filter-type="FilterType.SortResult"
					:filter-type-posible-val="FilterSortBy"
					is-overwrite
					@overwrite="sortBy = $event"
				/>

				<ChipSection
					title="Sort order"
					:filter-type="FilterType.SortOrder"
					:filter-type-posible-val="FilterSortOrder"
					is-overwrite
					@overwrite="sortOrder = $event"
				/>

				<div class="flex flex-row flex-wrap items-center gap-4">
					<div class="text-xl font-semibold">in category</div>
					<md-chip-set class="flex-rows flex">
						<md-filter-chip
							v-for="category_id in Object.keys(categories)"
							:key="category_id"
							:uuid="category_id"
							:label="categories[category_id]"
							@click="chipCategoryHandler($event.target)"
						/>
					</md-chip-set>
				</div>
			</div>

			<!-- Result region -->
			<Toggle :show="filteredTitles.length > 0">
				<div class="mb-8 mt-10 text-4xl font-bold">Here's what I found</div>
			</Toggle>
			<Toggle :show="filteredTitles.length === 0">
				<div class="mb-8 mt-10 text-4xl font-bold">Can't find anything</div>
			</Toggle>
			<div
				ref="imageContainerRef"
				class="grid"
				:style="{
					gridTemplateColumns: `repeat(${imagePerRow}, 1fr)`,
					gap: `${spaceBetween}px`,
				}"
			>
				<nuxt-link
					v-for="title in filteredTitlesToDisplay"
					:key="title.id"
					:to="`/title/${title.id}`"
				>
					<ItemCard
						:author="title.author ?? 'Unknown'"
						:cover="{
							src: fileApiUrl.cover(title.id),
							width: title.width,
							height: title.height,
							blurhash: title.blurhash,
							format: title.format,
						}"
						:progress="title.page_read ? title.page_read / title.page_count : 0"
						:title="title.title"
						:title-id="title.id"
					/>
				</nuxt-link>
			</div>
		</NavDrawerWrapper>
	</div>
</template>
