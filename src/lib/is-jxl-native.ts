export const isJxlNative = await (async (): Promise<boolean> => {
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
})();
