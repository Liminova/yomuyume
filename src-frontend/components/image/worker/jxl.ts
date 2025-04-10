import { setPageInDB, StoreName } from "../db";
import type { JpegXLWorkerInput } from "../decode-jpegxl";
import init, { decode } from "../jxl-webp-wasm";
import type { WorkerOutput } from "./pool";

declare const self: Worker;

let inited = false;
let initing = false;
const waitInitQueue: Array<(value: void | PromiseLike<void>)=> void> = [];

self.onmessage = async (event: MessageEvent<JpegXLWorkerInput>): Promise<void> => {
	if (!inited) {
		if (initing) {
			await new Promise<void>(resolve => waitInitQueue.push(resolve));
		} else {
			initing = true;
			await init();
			// eslint-disable-next-line require-atomic-updates
			inited = true;
			while (waitInitQueue.length > 0) {
				waitInitQueue.shift()?.();
			}
		}
	}

	if (event.data.type === "ping") {
		self.postMessage({ type: "pong" } satisfies WorkerOutput);
		return;
	}
	if (!event.data.payload) {
		self.postMessage({
			type: "done",
			error: "No payload received",
		} satisfies WorkerOutput);
		return;
	}

	try {
		const jxlRaw = await fetch(event.data.payload.url);
		const jxlBuf = await jxlRaw.arrayBuffer();

		await setPageInDB({
			page: {
				id: event.data.payload.id,
				data: decode(new Uint8Array(jxlBuf)),
			}, type: StoreName.JPEGXL,
		});

		self.postMessage({ type: "done" } satisfies WorkerOutput);
	} catch (error) {
		self.postMessage({
			type: "done",
			error: `${error}`,
		} satisfies WorkerOutput);
	}
};

self.onerror = (event: ErrorEvent): boolean => {
	self.postMessage({
		type: "done",
		error: event.message,
	} satisfies WorkerOutput);

	return true;
};
