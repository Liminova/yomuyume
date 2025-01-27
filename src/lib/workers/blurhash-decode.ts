import { decodeBlurHash } from "fast-blurhash";

import type { BlurhashWorkerInput, BlurhashWorkerOutput } from "~/composables/use-blurhash-decoder";

declare const self: Worker;

function handleMessage(event: MessageEvent<BlurhashWorkerInput>): BlurhashWorkerOutput {
	if (event.data.type === "ping") {
		return { type: "pong" };
	}
	if (!event.data.payload) {
		return {
			type: "done",
			error: "No payload received",
		};
	}
	const { blurhash, height, width } = event.data.payload;

	const smallWidth = 16;
	const smallHeight = Math.floor(smallWidth * height / width);

	return {
		type: "done",
		payload: decodeBlurHash(blurhash, smallWidth, smallHeight),
	};
}

self.onmessage = (event: MessageEvent<BlurhashWorkerInput>): void => {
	self.postMessage(handleMessage(event));
};

self.onerror = (event: ErrorEvent): boolean => {
	self.postMessage({
		type: "done",
		error: event.message,
	} satisfies BlurhashWorkerOutput);

	return true;
};
