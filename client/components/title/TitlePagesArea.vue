<script setup lang="ts">
import { toast } from 'vue-sonner'
import Image from '~/components/image/Image.vue'
import { GET_PAGE_FILE_PATH } from '~/composables/api/common'
import { useContentGetPages } from '~/composables/api/content'
import { useUserSetProgress } from '~/composables/api/user'

const props = defineProps<{
	titleId: string
	chapterId: string
	chapterIds: Array<{ id: string }>
}>()

const pages = useContentGetPages(props.chapterId)
const setProgress = useUserSetProgress()
function handleImageEmitInview(pageId: string): void {
	if (!pages.data.value) {
		return
	}

	const chapterIdx = props.chapterIds.findIndex(
		(chapter) => chapter.id === props.chapterId
	)
	if (chapterIdx === -1) {
		return
	}
	const chapterCount = props.chapterIds.length
	let chapterPercent = 0
	if (chapterCount > 1) {
		chapterPercent = chapterIdx / chapterCount
	}

	const pageIdx = pages.data.value.findIndex((page) => page.id === pageId)
	const pagePercent = (pageIdx + 1) / pages.data.value.length

	setProgress.mutate(
		{
			title_id: props.titleId,
			chapter_id: props.chapterId,
			page_id: pageId,
			percent: Math.round(
				(chapterPercent + pagePercent / chapterCount) * 100
			)
		},
		{
			onError: (error) => {
				toast.error('Failed to update progress', {
					description: `${error}`
				})
			}
		}
	)
}
</script>

<template>
	<div v-if="pages.status.value === 'error'">
		{{ pages.error.value }}
	</div>
	<div v-if="pages.data.value" class="mx-auto flex max-w-[700px] flex-col">
		<div v-for="page in pages.data.value" :id="page.id" :key="page.id">
			<Image
				:id="page.id"
				:src="GET_PAGE_FILE_PATH(page.id)"
				:is-jxl="page.jxl"
				:width="page.width"
				:height="page.height"
				:emit-when-in-view="true"
				@in-view="
					() => {
						handleImageEmitInview(page.id)
					}
				"
			/>
		</div>
	</div>
</template>
