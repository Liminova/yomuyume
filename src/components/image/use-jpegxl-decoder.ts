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

const processing = new Set<PageID>();
const waitlists = new Map<PageID, Array<Ref<DecodedImgURL | null>>>();

const pool = new WorkerPool<JpegXLWorkerInput>(() => {
	return new Worker(
		new URL("./worker/jxl.ts", import.meta.url),
		{ type: "module", name: "jpegxl-worker" },
	);
});

export async function useJpegXLDecoder(
	payload: JpegXLInput,
	outputWebpURL: Ref<DecodedImgURL | null>,
	error: Ref<ErrorEvent | string | null>,
): Promise<void> {
	// we won't tell you when it's done because you won't need it
	onUnmounted(() => {
		outputWebpURL.value = null;
		const waitlist = waitlists.get(payload.id);
		if (waitlist) {
			const index = waitlist.indexOf(outputWebpURL);
			if (index !== -1) {
				waitlist.splice(index, 1);
			}
		}
	});

	// check in DB first
	const record = await getPageInDB(payload.id, StoreName.JPEGXL);
	if (record !== undefined) {
		const blob = new Blob([record.data], { type: "image/webp" });
		outputWebpURL.value = URL.createObjectURL(blob);
		return;
	}

	// it's decoding and we'll tell you when it's done
	const waitlist = waitlists.get(payload.id);
	if (waitlist) { waitlist.push(outputWebpURL); }
	waitlists.set(payload.id, [outputWebpURL]);

	if (processing.has(payload.id)) { return; }
	processing.add(payload.id);

	let doneFn: ()=> void;
	const doneSignal = new Promise<void>((resolve) => { doneFn = resolve; });

	const worker = await pool.getWorker();
	worker.onmessage = async (event: MessageEvent<WorkerOutput>): Promise<void> => {
		doneFn();
		if (event.data.type === "pong") { return; }
		if (event.data.error !== undefined) {
			error.value = event.data.error;
			return;
		}

		processing.delete(payload.id);

		const waitlist = waitlists.get(payload.id);
		if (waitlist === undefined) { return; } // no one wants me

		const record = await getPageInDB(payload.id, StoreName.JPEGXL);
		if (!record) {
			error.value = "Worker did not save the result to DB";
			return;
		}
		const blob = new Blob([record.data], { type: "image/webp" });
		for (const ref of waitlist.values()) {
			ref.value = URL.createObjectURL(blob);
		}
	};
	worker.inner.postMessage({
		type: "work",
		payload,
	} satisfies JpegXLWorkerInput);

	await doneSignal;
	pool.returnWorker(worker);
}
