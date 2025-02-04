<script setup lang="ts">
import { NuxtLink } from "#components";
import Image from "~/components/image";
import { GET_COVER_PATH } from "~/composables/api/constants";
import type { GetTitleResponseBody } from "~/composables/api/content";

const props = defineProps<{ title: GetTitleResponseBody }>();
</script>

<template>
	<NuxtLink
		:to="`/${props.title.is_series ? 'series' : 'oneshot'}/${props.title.id}`"
		class="relative flex flex-row justify-center overflow-hidden rounded-3xl bg-black/50 max-sm:aspect-[10/12] sm:static">
		<!-- Background -->
		<div class="absolute z-[-1] hidden size-full overflow-hidden object-cover px-2 sm:block">
			<Image
				:id="props.title.id"
				class="size-full rounded-3xl object-cover"
				:draggable="false"
				:src="GET_COVER_PATH(props.title.id)"
				:blurhash="props.title.cover_blurhash"
				:width="props.title.cover_width"
				:height="props.title.cover_height"
				:is-jxl="props.title.cover_jxl" />
		</div>

		<!-- Cover -->
		<div class="size-full sm:max-lg:max-w-56 lg:max-w-sm lg:py-10 lg:pl-10">
			<Image
				:id="props.title.id"
				class="size-full overflow-hidden object-cover sm:aspect-[10/16] lg:rounded-2xl"
				:draggable="false"
				:src="GET_COVER_PATH(props.title.id)"
				:blurhash="props.title.cover_blurhash"
				:width="props.title.cover_width"
				:height="props.title.cover_height"
				:is-jxl="props.title.cover_jxl" />
		</div>

		<div
			class="pointer-events-none absolute left-0 top-0 flex size-full bg-gradient-to-t from-black/80 to-transparent sm:hidden" />

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
