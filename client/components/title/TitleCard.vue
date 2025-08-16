<script setup lang="ts">
import { NuxtLink } from '#components'
import Image from '~/components/image'
import { GET_COVER_FILE_PATH } from '~/composables/api/common'
import type { BaseTitleResponse } from '~/composables/api/content'

const props = defineProps<{ title: BaseTitleResponse }>()
</script>

<template>
	<NuxtLink
		class="flex flex-col items-start justify-center"
		:to="`/title/${props.title.id}`"
	>
		<div class="img-cover group relative w-full overflow-hidden rounded-xl">
			<Image
				:id="title.id"
				emit-when-in-view
				:src="GET_COVER_FILE_PATH(title.id)"
				:blurhash="title.cover_blurhash"
				:width="title.cover_width"
				:height="title.cover_height"
				:is-jxl="title.cover_jxl"
				class="aspect-[3/4] h-full rounded-xl object-cover"
			/>
			<div
				class="absolute left-0 top-0 size-full bg-[rgba(255_255_255/0.08)] opacity-0 transition-opacity group-[.img-cover]:hover:opacity-100"
			/>
			<!-- <md-linear-progress
				v-show="progress !== 0"
				:value="progress !== 0"
				class="absolute bottom-0 w-full" /> -->

			<i
				v-show="title.progress_percent === 100"
				class="fa-solid fa-circle-check absolute right-2 top-1 text-xl text-[--md-sys-color-on-secondary-fixed-variant]"
			/>
		</div>
		<div>
			<div
				class="mt-2 truncate text-sm font-light text-[--md-sys-color-on-surface]"
			>
				{{ title.author ?? 'Unknown' }}
			</div>
			<div
				class="truncate-2 text-balance text-lg font-bold text-[--md-sys-color-inverse-surface]"
			>
				{{ title.title ?? 'Untitled' }}
			</div>
		</div>
	</NuxtLink>
</template>
