<script setup lang="ts">
import type { SsimEvalTitleServerResponse } from "~/composables/api";
import NavDrawerWrapper from "~/layouts/NavDrawerWrapper.vue";

const titles = ref([]) as Ref<Array<SsimEvalTitleServerResponse>>;
const snackbarMessage: Ref<string> = ref("");

void (async () => {
	const { data, message } = await utilsApi.ssimEval();

	if (data === undefined) {
		snackbarMessage.value = message ?? "";
		return;
	}

	titles.value = [data.title_a, data.title_b];
})();
</script>

<template>
	<div>
		<Snackbar :message="snackbarMessage" @close="snackbarMessage = ''" />
		<NavDrawerWrapper class="flex size-full flex-row">
			<div class="grid w-full grid-cols-2 gap-4">
				<div v-for="title in titles" :key="title.id" class="flex flex-col items-center">
					<ImagePoly
						:image="{
							src: fileApiUrl.thumbnail(title.id),
							format: title.format,
							width: title.width,
							height: title.height,
							blurhash: title.blurhash,
						}"
						class="rounded-2xl"
						:style="{
							width: '100%',
							objectFit: 'cover',
							overflow: 'hidden',
							aspectRatio: 3 / 4,
						}"
					/>
					<div class="text-2xl font-bold">{{ title.title }}</div>
					<div class="text">{{ title.desc }}</div>
				</div>
			</div>
		</NavDrawerWrapper>
	</div>
</template>
