import { onUnmounted, type Ref } from "vue";

import { getPageInDB, StoreName } from "./db";
import { type WorkerInputWrapper, type WorkerOutput, WorkerPool } from "./worker/pool";

type PageID = string;
type DecodedImgURL = string;
interface BlurhashInput {
	id: PageID;
	blurhash: string;
	width: number;
	height: number;
}
export type BlurhashWorkerInput = WorkerInputWrapper<BlurhashInput>;

const pool = new WorkerPool<BlurhashInput>(() => {
	return new Worker(
		new URL("./worker/blurhash.ts", import.meta.url),
		{ type: "module", name: "blurhash-worker" },
	);
});

const processing = new Set<PageID>();
const waitlists = new Map<PageID, Array<Ref<DecodedImgURL | null>>>();

export async function useBlurhashDecoder(
	payload: BlurhashInput,
	outputImgURL: Ref<DecodedImgURL | null>,
	error: Ref<ErrorEvent | string | null>,
): Promise<void> {
	// we won't tell you when it's done because you won't need it
	onUnmounted(() => {
		outputImgURL.value = null;
		const waitlist = waitlists.get(payload.id);
		if (waitlist) {
			const index = waitlist.indexOf(outputImgURL);
			if (index !== -1) {
				waitlist.splice(index, 1);
			}
		}
	});

	// check in DB first
	const record = await getPageInDB(payload.id, StoreName.BLURHASH);
	if (record) {
		const blob = new Blob([record.data], { type: "image/webp" });
		outputImgURL.value = URL.createObjectURL(blob);
		return;
	}

	// it's decoding and we'll tell you when it's done
	const waitlist = waitlists.get(payload.id);
	if (waitlist) { waitlist.push(outputImgURL); }
	waitlists.set(payload.id, [outputImgURL]);

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

		const record = await getPageInDB(payload.id, StoreName.BLURHASH);
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
	} satisfies BlurhashWorkerInput);

	await doneSignal;
	pool.returnWorker(worker);
}
