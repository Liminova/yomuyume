import { type Ref, ref } from "vue";

import { type WorkerInputWrapper, type WorkerOutputWrapper, WorkerPool } from "~/lib/worker-pool";

interface BlurhashInput {
	blurhash: string;
	width: number;
	height: number;
}
type BlurhashOutput = Uint8ClampedArray;

export type BlurhashWorkerInput = WorkerInputWrapper<BlurhashInput>;
export type BlurhashWorkerOutput = WorkerOutputWrapper<BlurhashOutput, string>;

const pool = new WorkerPool<BlurhashInput, BlurhashOutput, string>(() => {
	return new Worker(
		new URL("../lib/workers/blurhash-decode.ts", import.meta.url),
		{ type: "module", name: "blurhash-decoder-worker" },
	);
});

export function useBlurhashDecoder(): {
	dataUrl: Ref<string | undefined>;
	error: Ref<ErrorEvent | string | null>;
	decode: ()=> Promise<void>;
	// eslint-disable-next-line no-unused-vars
	setInput: (props: BlurhashInput)=> void;
} {
	const dataUrl = ref<string | undefined>(undefined);
	const error = ref<ErrorEvent | string | null>(null);
	let blurhashString: BlurhashInput | null = null;

	return {
		dataUrl,
		error,
		decode: async (): Promise<void> => {
			if (!blurhashString) { return; }

			let doneFn: ()=> void;
			const doneSignal = new Promise<void>((resolve) => { doneFn = resolve; });

			const worker = await pool.getWorker((event: MessageEvent<BlurhashWorkerOutput>) => {
				doneFn();

				if (event.data.type === "pong") { return; }
				if (event.data.error !== undefined) {
					error.value = event.data.error;
					return;
				}
				if (!event.data.payload) {
					error.value = "No payload received from worker";
					return;
				}

				const blob = new Blob([event.data.payload], { type: "image/png" });
				dataUrl.value = URL.createObjectURL(blob);
			});

			worker.postMessage({
				type: "work",
				payload: blurhashString,
			} satisfies BlurhashWorkerInput);

			await doneSignal;
			pool.returnWorker(worker);
		},
		setInput: (input: BlurhashInput): void => {
			blurhashString = input;
		},
	};
}
