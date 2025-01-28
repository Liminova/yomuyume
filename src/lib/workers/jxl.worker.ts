import type { JxlPolyfillWorkerInput, JxlPolyfillWorkerOutput } from "~/composables/use-jxl-decoder";
import init, { decode } from "~/lib/jxl-webp-wasm";

declare const self: Worker;

await init();

self.onmessage = async (event: MessageEvent<JxlPolyfillWorkerInput>): Promise<void> => {
	if (event.data.type === "ping") {
		self.postMessage({ type: "pong" } satisfies JxlPolyfillWorkerOutput);
		return;
	}
	if (!event.data.payload) {
		self.postMessage({
			type: "done",
			error: "No payload received",
		} satisfies JxlPolyfillWorkerOutput);
		return;
	}

	try {
		const jxlRaw = await fetch(event.data.payload);
		const jxlBuf = await jxlRaw.arrayBuffer();
		const jxlArr = new Uint8Array(jxlBuf);

		const data = decode(jxlArr);
		const blob = new Blob([data], { type: "image/webp" });

		self.postMessage({
			type: "done",
			payload: URL.createObjectURL(blob),
		} satisfies JxlPolyfillWorkerOutput);
	} catch (error) {
		self.postMessage({
			type: "done",
			error: `${error}`,
		} satisfies JxlPolyfillWorkerOutput);
	}
};

self.onerror = (event: ErrorEvent): boolean => {
	self.postMessage({
		type: "done",
		error: event.message,
	} satisfies JxlPolyfillWorkerOutput);

	return true;
};
