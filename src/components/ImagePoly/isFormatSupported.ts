async function isFormatSupported(format: string, testData: string): Promise<boolean> {
	const image = new Image();

	image.src = `data:image/${format};base64,${testData}`;

	return new Promise((resolve, _) => {
		image.onload = () => {
			resolve(true);
		};

		image.onerror = () => {
			resolve(false);
		};
	});
}

let isAvifSupported = false;
let isJxlSupported = false;

void Promise.all([
	isFormatSupported(
		"avif",
		"AAAAIGZ0eXBhdmlmAAAAAGF2aWZtaWYxbWlhZk1BMUIAAADybWV0YQAAAAAAAAAoaGRscgAAAAAAAAAAcGljdAAAAAAAAAAAAAAAAGxpYmF2aWYAAAAADnBpdG0AAAAAAAEAAAAeaWxvYwAAAABEAAABAAEAAAABAAABGgAAAB0AAAAoaWluZgAAAAAAAQAAABppbmZlAgAAAAABAABhdjAxQ29sb3IAAAAAamlwcnAAAABLaXBjbwAAABRpc3BlAAAAAAAAAAIAAAACAAAAEHBpeGkAAAAAAwgICAAAAAxhdjFDgQ0MAAAAABNjb2xybmNseAACAAIAAYAAAAAXaXBtYQAAAAAAAAABAAEEAQKDBAAAACVtZGF0EgAKCBgANogQEAwgMg8f8D///8WfhwB8+ErK42A="
	),
	isFormatSupported(
		"jxl",
		"/woIAAAMABKIAgC4AF3lEgAAFSqjjBu8nOv58kOHxbSN6wxttW1hSaLIODZJJ3BIEkkaoCUzGM6qJAE="
	),
]).then((results) => {
	isAvifSupported = results[0];
	isJxlSupported = results[1];
});

export { isAvifSupported, isJxlSupported };
