import init, { decode } from "jxl-webp-wasm";

import type { JxlPolyfillWorkerInput, JxlPolyfillWorkerOutput } from "~/composables/use-jxl-decoder";

declare const self: Worker;

await init();

async function handleMessage(event: MessageEvent<JxlPolyfillWorkerInput>): Promise<JxlPolyfillWorkerOutput> {
	if (event.data.type === "ping") {
		return { type: "pong" };
	}
	if (!event.data.payload) {
		return {
			type: "done",
			error: "No payload received",
		};
	}

	try {
		const imageJxl = await fetch(event.data.payload);
		const imageJxlArrayBuffer = await imageJxl.arrayBuffer();
		const imageJxlU8Array = new Uint8Array(imageJxlArrayBuffer);

		return {
			type: "done",
			payload: decode(imageJxlU8Array),
		};
	} catch (error) {
		return {
			type: "done",
			error: `${error}`,
		};
	}
}

self.onmessage = async (event: MessageEvent<JxlPolyfillWorkerInput>): Promise<void> => {
	self.postMessage(await handleMessage(event));
};

self.onerror = (event: ErrorEvent): boolean => {
	self.postMessage({
		type: "done",
		error: event.message,
	} satisfies JxlPolyfillWorkerOutput);

	return true;
};
