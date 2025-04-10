import { onUnmounted, type Ref } from "vue";

import { getPageInDB, StoreName } from "./db";
import { type WorkerInputWrapper, type WorkerOutput, WorkerPool } from "./worker/pool";

type PageID = string;
type DecodedImgURL = string;
interface JpegXLInput {
	id: PageID;
	url: string;
}
export type JpegXLWorkerInput = WorkerInputWrapper<JpegXLInput>;

const queue = new Map<PageID, Array<Ref<DecodedImgURL | null>>>();
const pool = new WorkerPool<JpegXLWorkerInput>(() => new Worker(
	new URL("./worker/jxl.ts", import.meta.url),
	{ type: "module", name: "jxl-worker" },
));

export function cleanupJpegXlTask(id: PageID, outputWebpURL: Ref<DecodedImgURL | null>): void {
	const refs = queue.get(id);
	if (!refs) { return; }
	queue.set(id, refs.filter(ref => ref !== outputWebpURL));
}

export function decodeJpegXL(
	payload: JpegXLInput,
	outputWebpURL: Ref<DecodedImgURL | null>,
	error: Ref<ErrorEvent | string | null>,
): void {
	onUnmounted(() => {
		const refs = queue.get(payload.id);
		if (!refs) { return; }
		queue.set(payload.id, refs.filter(ref => ref !== outputWebpURL));
	});

	void (async (): Promise<void> => {
		// check in DB first
		const record = await getPageInDB(payload.id, StoreName.JPEGXL);
		if (record !== undefined) {
			outputWebpURL.value = URL.createObjectURL(new Blob([record.data], { type: "image/webp" }));
			return;
		}

		// it's decoding and we'll tell you when it's done
		const refs = queue.get(payload.id);
		if (refs !== undefined) {
			if (refs.includes(outputWebpURL)) { return; }
			refs.push(outputWebpURL);
			return;
		}
		queue.set(payload.id, [outputWebpURL]);

		const worker = await pool.getWorker();
		// eslint-disable-next-line require-atomic-updates
		worker.onmessage = async (event: MessageEvent<WorkerOutput>): Promise<void> => {
			pool.returnWorker(worker);
			if (event.data.type === "pong") { return; }
			if (event.data.error !== undefined) {
				error.value = event.data.error;
				return;
			}

			const refs = queue.get(payload.id);
			if (refs === undefined) { return; } // no one wants me

			const record = await getPageInDB(payload.id, StoreName.JPEGXL);
			if (!record) {
				error.value = "Worker did not save the result to DB";
				return;
			}
			const url = URL.createObjectURL(new Blob([record.data], { type: "image/webp" }));
			for (const ref of refs) {
				ref.value = url;
			}
		};
		worker.inner.postMessage({ type: "work", payload } satisfies JpegXLWorkerInput);
	})();
}
