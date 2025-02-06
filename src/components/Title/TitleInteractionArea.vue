<script setup lang="ts">
import { clamp } from "@vueuse/core";
import { Bookmark, Heart } from "lucide-vue-next";

import { ref, watchEffect } from "#imports";
import Button from "~/components/ui/Button.vue";
import { useBookmark, useFavorite } from "~/composables/api/user";
import { cn } from "~/lib/utils";

const props = defineProps<{
	titleId: string;
	class?: string;
	isFavorite?: boolean;
	isBookmark?: boolean;
	favorites?: number;
	bookmarks?: number;
}>();

const [addFavorite, deleteFavorite] = [useFavorite("PUT"), useFavorite("DELETE")];
const [addBookmark, deleteBookmark] = [useBookmark("PUT"), useBookmark("DELETE")];

const [innerFavorites, innerBookmarks] = [ref<number>(0), ref<number>(0)];
const [innerIsFavorite, innerIsBookmark] = [ref(false), ref(false)];

watchEffect(() => {
	innerIsFavorite.value = props.isFavorite;
	innerIsBookmark.value = props.isBookmark;
	innerFavorites.value = props.favorites ?? 0;
	innerBookmarks.value = props.bookmarks ?? 0;
});

function toggleBookmark(): void {
	if (innerIsBookmark.value) {
		deleteBookmark.mutate(props.titleId);
		innerBookmarks.value = clamp(innerBookmarks.value - 1, 0, Infinity);
		innerIsBookmark.value = false;
	} else {
		addBookmark.mutate(props.titleId);
		innerBookmarks.value = clamp(innerBookmarks.value + 1, 0, Infinity);
		innerIsBookmark.value = true;
	}
}

function toggleFavorite(): void {
	if (innerIsFavorite.value) {
		deleteFavorite.mutate(props.titleId);
		innerFavorites.value--;
		innerIsFavorite.value = false;
	} else {
		addFavorite.mutate(props.titleId);
		innerFavorites.value++;
		innerIsFavorite.value = true;
	}
}

</script>

<template>
	<div :class="cn('my-7 flex w-full flex-row items-center justify-start gap-2', props.class)">
		<Button
			:variant="innerIsBookmark ? 'secondary' : 'outline'"
			size="lg"
			@click="toggleBookmark">
			<Bookmark :fill="innerIsBookmark ? 'white' : 'transparent'" />
			{{ innerBookmarks === 0 ? "" : innerBookmarks }}
		</Button>
		<Button
			:variant="innerIsFavorite ? 'secondary' : 'outline'"
			size="lg"
			@click="toggleFavorite">
			<Heart :fill="innerIsFavorite ? 'white' : 'transparent'" />
			{{ innerFavorites === 0 ? "" : innerFavorites }}
		</Button>
	</div>
</template>
