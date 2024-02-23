import { isAvifSupported, isJxlSupported } from "./isFormatSupported";
import { BLURHASH_WORKER_COUNT, IMAGE_WORKER_COUNT } from "./workerCount";
import type { MyImage } from "~/composables/types";

type blurhashQueueType = {
	data: [string, number, number] /** blurhash, width, height */;
	renderedDataRef: Ref<string>;
};

type imageQueueType = {
	data: [string, string, string] /** src, format, jwt token */;
	renderedDataRef: Ref<string>;
};

async function isNative(format: string): Promise<boolean> {
	switch (format) {
		case "jxl":
			return Promise.resolve(isJxlSupported);
		case "avif":
			return Promise.resolve(isAvifSupported);
		default:
			return Promise.resolve(true);
	}
}

class WebWorkerRenderer {
	private readonly blurhashQueue: Array<blurhashQueueType> = [];
	private readonly imageQueue: Array<imageQueueType> = [];

	private readonly blurhashWorkers: Array<{ instance: Worker; isReady: boolean }> = [];
	private imageWorkers: Array<{ instance: Worker; isReady: boolean }> = [];

	private polyfillWorkersSpunUp = false;

	constructor() {
		this.blurhashWorkers = Array.from({ length: BLURHASH_WORKER_COUNT }, () => ({
			instance: new Worker(new URL("./workers/blurhash.web.ts", import.meta.url), {
				type: "module",
				name: "blurhashRenderer",
			}),
			isReady: true,
		}));
	}

	private spinUpPolyfillWorkers() {
		this.polyfillWorkersSpunUp = true;
		this.imageWorkers = Array.from({ length: IMAGE_WORKER_COUNT }, () => ({
			instance: new Worker(new URL("./workers/image.web.ts", import.meta.url), {
				type: "module",
				name: "imageRenderer",
			}),
			isReady: true,
		}));
	}

	private processQueue(
		queue: Array<blurhashQueueType | imageQueueType>,
		workers: Array<{ instance: Worker; isReady: boolean }>
	) {
		const freeWorker = workers.find((worker) => worker.isReady);

		if (freeWorker && queue.length > 0) {
			const job = queue.shift();

			if (!job) {
				return;
			}

			freeWorker.isReady = false;
			freeWorker.instance.onmessage = (event: MessageEvent<string>) => {
				job.renderedDataRef.value = event.data;
				freeWorker.isReady = true;
				this.processQueue(queue, workers);
			};

			freeWorker.instance.postMessage(job.data);
		}
	}

	async new(image: MyImage, blurhashRef: Ref<string>, imageRef: Ref<string>) {
		// Decode blurhash
		if (image.width !== undefined && image.height !== undefined && image.blurhash) {
			this.blurhashQueue.push({
				data: [image.blurhash, image.width, image.height],
				renderedDataRef: blurhashRef,
			});
			this.processQueue(this.blurhashQueue, this.blurhashWorkers);
		}

		// Is native?
		if (await isNative(image.format)) {
			imageRef.value = image.src;
			return;
		}

		// Web Worker init
		if (!this.polyfillWorkersSpunUp) {
			this.spinUpPolyfillWorkers();
		}

		// Decode image
		this.imageQueue.push({
			data: [image.src, image.format, globalStore.token],
			renderedDataRef: imageRef,
		});
		this.processQueue(this.imageQueue, this.imageWorkers);
	}
}

class SharedWorkerRenderer {
	async new(image: MyImage, blurhashRef: Ref<string>, imageRef: Ref<string>) {
		// Decode blurhash
		if (image.width !== undefined && image.height !== undefined && image.blurhash) {
			const blurhashWorker = new SharedWorker(
				new URL("./workers/blurhash.shared.ts", import.meta.url),
				{
					type: "module",
					name: "blurhashRenderer",
				}
			);

			blurhashWorker.port.onmessage = (event: MessageEvent<string>) =>
				(blurhashRef.value = event.data);
			blurhashWorker.port.postMessage([image.blurhash, image.width, image.height]);
		}

		// Is native?
		if (await isNative(image.format)) {
			imageRef.value = image.src;
			return;
		}

		// Decode image
		const imageWorker = new SharedWorker(
			new URL("./workers/image.shared.ts", import.meta.url),
			{
				type: "module",
				name: "imageRenderer",
			}
		);

		imageWorker.port.onmessage = (event: MessageEvent<string>) => {
			imageRef.value = event.data;
		};

		imageWorker.port.postMessage([image.src, image.format, globalStore.token]);
	}
}

const renderer = "SharedWorker" in window ? new SharedWorkerRenderer() : new WebWorkerRenderer();

/**
 * Renders the blurhash and the actual image using Shared/Web Workers.
 *
 * @param image a MyImage object
 * @param blurhashRef the ref to store the rendered blurhash
 */
export default function renderImage(
	image: MyImage,
	blurhashRef: Ref<string>,
	imageRef: Ref<string>
) {
	void renderer.new(image, blurhashRef, imageRef);
}
