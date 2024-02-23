import { decodeBlurHash } from "fast-blurhash";

/**
 * Decode blurhash and return as blob url
 *
 * @param data [blurhash, width, height]
 */
export default async function blurhashDecode(data: [string, number, number]): Promise<string> {
	// let [blurhash, width, height] = data;
	const blurhash = data[0];
	const width = data[1];
	const height = data[2];

	// decode blurHash image
	const pixels = decodeBlurHash(blurhash, width, height);

	// draw it on canvas
	const canvas = new OffscreenCanvas(width, height);
	const ctx = canvas.getContext("2d");

	if (!ctx) {
		throw new Error("Could not get context");
	}

	const imageData = ctx.createImageData(width, height);

	imageData.data.set(pixels);
	ctx.putImageData(imageData, 0, 0);

	// convert to blob and return as blob url
	const blob = await canvas.convertToBlob();

	return URL.createObjectURL(blob);
}
