<script setup lang="ts">

import Image from "~/components/image/Image.vue";
import { GET_PAGE_FILE_PATH } from "~/composables/api/common";
import { useContentGetPages } from "~/composables/api/content";

const props = defineProps<{ chapterId: string }>();

const pages = useContentGetPages(props.chapterId);
</script>

<template>
	<div
		v-if="pages.isError">
		{{ pages.error.value }}
	</div>
	<div
		v-if="pages.data.value"
		class="mx-auto flex max-w-[700px] flex-col">
		<div
			v-for="page in pages.data.value"
			:id="page.id"
			:key="page.id">
			<Image
				:id="page.id"
				:src="GET_PAGE_FILE_PATH(page.id)"
				:is-jxl="page.jxl"
				:width="page.width"
				:height="page.height" />
		</div>
	</div>
</template>
