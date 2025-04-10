import { useAsyncData, useState } from "nuxt/app";

// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type
export function useIsJxlNative() {
	return useState("is-jxl-native", () => true);
}
/** Run this function once in `app.vue` */
export async function initIsJxlNative(): Promise<void> {
	const { data } = await useAsyncData<boolean>("is-jxl-native", async (): Promise<boolean> => {
		const image = new Image();

		image.src = `data:image/jxl;base64,/woIAAAMABKIAgC4AF3lEgAAFSqjjBu8nOv58kOHxbSN6wxttW1hSaLIODZJJ3BIEkkaoCUzGM6qJAE=`;

		return new Promise((resolve, _): void => {
			image.onload = (): void => {
				resolve(true);
			};

			image.onerror = (): void => {
				resolve(false);
			};
		});
	}, {
		server: false,
		immediate: true,
	});

	const isJxlNative = useIsJxlNative();
	isJxlNative.value = data.value ?? true;
}
