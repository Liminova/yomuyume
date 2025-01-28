import type { BlurhashWorkerInput, BlurhashWorkerOutput } from "~/composables/use-blurhash-decoder";
import init, { decode } from "~/lib/blurhash-webp-wasm";

declare const self: Worker;

await init();

self.onmessage = (event: MessageEvent<BlurhashWorkerInput>): void => {
	if (event.data.type === "ping") {
		self.postMessage({ type: "pong" } satisfies BlurhashWorkerOutput);
		return;
	}
	if (!event.data.payload) {
		self.postMessage({
			type: "done",
			error: "No payload received",
		} satisfies BlurhashWorkerOutput);
		return;
	}
	const { blurhash, height, width } = event.data.payload;

	const smallWidth = 12;
	const smallHeight = Math.floor(smallWidth * height / width);

	try {
		const data = decode(blurhash, smallWidth, smallHeight);
		const blob = new Blob([data], { type: "image/webp" });

		self.postMessage({
			type: "done",
			payload: URL.createObjectURL(blob),
		} satisfies BlurhashWorkerOutput);
	} catch (error) {
		self.postMessage({
			type: "done",
			error: `${error}`,
		} satisfies BlurhashWorkerOutput);
	}
};

self.onerror = (event: ErrorEvent): boolean => {
	self.postMessage({
		type: "done",
		error: event.message,
	} satisfies BlurhashWorkerOutput);

	return true;
};
