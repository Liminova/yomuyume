<script setup lang="ts">

import { NuxtLink } from "#components";
import Image from "~/components/Image.vue";
import type { GetTitleResponseBody } from "~/composables/api/content";
import { toCoverApiEndpoint } from "~/composables/api/file";
import type { MyImage } from "~/lib/types";

import { homeStore } from "./utils";

const props = withDefaults(defineProps<{
	title: GetTitleResponseBody;
	isFirstTitle?: boolean;
	isLastTitle?: boolean;
}>(), {
	isFirstTitle: true,
	isLastTitle: false,
});

const store = homeStore();
</script>

<template>
	<NuxtLink
		:to="`/title/${props.title.id}`"
		class="relative flex h-full flex-row justify-center overflow-hidden bg-black/50 sm:static"
		:class="{
			'rounded-l-3xl': props.isFirstTitle,
			'rounded-r-3xl': props.isLastTitle,
		}">
		<!-- Background -->
		<div
			class="absolute left-0 top-0 z-[-1] hidden w-full overflow-hidden sm:block"
			:class="{
				'rounded-l-3xl': props.isFirstTitle,
				'rounded-r-3xl': props.isLastTitle,
			}"
			:style="{ height: `${store.recommendsContainerHeight}px` }">
			<Image
				:id="props.title.id"
				class="w-full scale-110 overflow-hidden object-cover blur-sm"
				:draggable="false"
				image-class="overflow-hidden"
				:src="toCoverApiEndpoint(props.title.id)"
				:blurhash="props.title.cover_blurhash"
				:width="props.title.cover_width"
				:height="props.title.cover_height"
				:is-jxl="props.title.cover_jxl" />
		</div>

		<!-- Cover -->
		<div class="size-full sm:min-w-[350px] sm:max-w-xs lg:py-10 lg:pl-10">
			<Image
				:id="props.title.id"
				:draggable="false"
				class="h-full overflow-hidden lg:rounded-2xl"
				image-class="h-full object-cover"
				:src="toCoverApiEndpoint(props.title.id)"
				:blurhash="props.title.cover_blurhash"
				:width="props.title.cover_width"
				:height="props.title.cover_height"
				:is-jxl="props.title.cover_jxl" />
		</div>

		<div
			class="pointer-events-none absolute left-0 top-0 flex size-full bg-black/50 sm:hidden" />

		<!-- Informations -->
		<div
			class="absolute left-0 top-0 z-[1] flex size-full flex-col justify-end p-7 sm:static sm:z-auto sm:max-w-3xl sm:justify-start sm:bg-transparent sm:p-10">
			<div
				class="text-lg font-light"
				data-theme="dark">
				{{ props.title.author ?? "Unknown" }}
			</div>
			<div
				class="truncate-2 mb-1 text-balance text-3xl font-bold"
				data-theme="dark">
				{{ props.title.title ?? "Untitled" }}
			</div>

			<div
				v-if="props.title.release"
				class="truncate">
				{{ props.title.release }}
			</div>

			<!-- <div
				v-if="titleTagNames.includes(`completed`)"
				class="mb-2"
				data-theme="dark">
				<i class="fa-solid fa-circle-check mr-2" />
				<span>Completed</span>
			</div> -->

			<div
				v-if="title.tags?.length !== 0"
				class="mb-2 hidden flex-row flex-wrap gap-2 sm:flex">
				<span
					v-for="[id, name] in title.tags"
					:key="id">
					<div>
						{{ name }}
					</div>
				</span>
			</div>

			<!-- <div
				v-if="props.title.description"
				class="truncate-5 sm:truncate-8 z-[1] overflow-hidden"
				data-theme="dark">
				{{ props.title.description }}
			</div> -->
		</div>
	</NuxtLink>
</template>
