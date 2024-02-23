import avifDec from "~/assets/polyfill/avif_dec";
import jxlDec from "~/assets/polyfill/jxl_dec";

function getDecoder(format: string) {
	switch (format) {
		case "jxl":
			// eslint-disable-next-line @typescript-eslint/no-unsafe-call
			return Promise.resolve(jxlDec());
		case "avif":
			// eslint-disable-next-line @typescript-eslint/no-unsafe-call
			return Promise.resolve(avifDec());
		default:
			throw new Error(`Unknown format to polyfill: ${format}`);
	}
}

/**
 * Decode image and return as blob url
 *
 * @param data [src, format, jwt token]
 */
export default async function imageDecode(data: [string, string, string]): Promise<string> {
	const [src, format, token] = data;

	/** Fetch image from network */
	// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
	const decoder = await getDecoder(format);
	const response = await fetch(src, {
		method: "GET",
		headers: {
			Authorization: `Bearer ${token}`,
		},
	});

	if (!response.ok) {
		return Promise.resolve(src); // don't even care, just return the src
	}

	// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
	const decoded: ImageData = decoder.decode(await response.arrayBuffer());

	// const canvas = new MyOffscreenCanvas(decoded.width, decoded.height).fromImageData(decoded);
	const canvas = new OffscreenCanvas(decoded.width, decoded.height);
	const ctx = canvas.getContext("2d");

	if (!ctx) {
		throw new Error("Could not get context");
	}

	ctx.putImageData(decoded, 0, 0);
	const blob = await canvas.convertToBlob();

	return Promise.resolve(URL.createObjectURL(blob));
}
