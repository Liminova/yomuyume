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

const queue = new Map<PageID, Array<Ref<DecodedImgURL | null>>>();
const pool = new WorkerPool<BlurhashInput>(() => {
	return new Worker(
		new URL("./worker/blurhash.ts", import.meta.url),
		{ type: "module", name: "blurhash-worker" },
	);
});

export async function useBlurhashDecoder(
	payload: BlurhashInput,
	outputImgURL: Ref<DecodedImgURL | null>,
	error: Ref<ErrorEvent | string | null>,
): Promise<void> {
	// we won't tell you when it's done because you won't need it
	onUnmounted(() => {
		outputImgURL.value = null;
		const refs = queue.get(payload.id);
		if (!refs) { return; }
		queue.set(payload.id, refs.filter(ref => ref !== outputImgURL));
	});

	// check in DB first
	const record = await getPageInDB(payload.id, StoreName.BLURHASH);
	if (record) {
		outputImgURL.value = URL.createObjectURL(new Blob([record.data], { type: "image/webp" }));
		return;
	}

	// it's decoding and we'll tell you when it's done
	const refs = queue.get(payload.id);
	if (refs !== undefined) {
		if (refs.includes(outputImgURL)) { return; }
		refs.push(outputImgURL);
		return;
	}
	queue.set(payload.id, [outputImgURL]);

	const worker = await pool.getWorker();
	worker.onmessage = async (event: MessageEvent<WorkerOutput>): Promise<void> => {
		pool.returnWorker(worker);
		if (event.data.type === "pong") { return; }
		if (event.data.error !== undefined) {
			error.value = event.data.error;
			return;
		}

		const refs = queue.get(payload.id);
		if (refs === undefined) { return; } // no one wants me

		const record = await getPageInDB(payload.id, StoreName.BLURHASH);
		if (!record) {
			error.value = "Worker did not save the result to DB";
			return;
		}
		const url = URL.createObjectURL(new Blob([record.data], { type: "image/webp" }));
		for (const ref of refs) {
			ref.value = url;
		}
	};
	worker.inner.postMessage({ type: "work", payload } satisfies BlurhashWorkerInput);
}
