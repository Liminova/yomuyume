import { type Ref, ref } from "vue";

import { type WorkerInputWrapper, type WorkerOutputWrapper, WorkerPool } from "~/lib/worker-pool";

/** basically just the image url */
type JxlPolyfillInput = string;
type JxlPolyfillOutput = Uint8Array;

export type JxlPolyfillWorkerInput = WorkerInputWrapper<JxlPolyfillInput>;
export type JxlPolyfillWorkerOutput = WorkerOutputWrapper<JxlPolyfillOutput, string>;

const pool = new WorkerPool<JxlPolyfillInput, JxlPolyfillOutput, string>(() => {
	return new Worker(
		new URL("../lib/workers/jxl-decode.ts", import.meta.url),
		{ type: "module", name: "jxl-decoder-worker" },
	);
});

/**
 * Assign `imageUrl` to `dataUrl` and return immediately if it's not in JXL format.
 *
 * Else use polyfill to decode to WebP format.
 */
export function useJxlDecoder(): {
	dataUrl: Ref<string | undefined>;
	error: Ref<ErrorEvent | string | null>;
	decode: ()=> Promise<void>;
	setInput: (_: JxlPolyfillInput)=> void;
} {
	const dataUrl = ref<string | undefined>(undefined);
	const error = ref<ErrorEvent | string | null>(null);
	let imageUrl: JxlPolyfillInput = "";

	return {
		dataUrl,
		error,
		decode: async (): Promise<void> => {
			let doneFn: ()=> void;
			const doneSignal = new Promise<void>((resolve) => { doneFn = resolve; });

			const worker = await pool.getWorker((event: MessageEvent<JxlPolyfillWorkerOutput>) => {
				doneFn();

				if (event.data.type === "pong") { return; }
				if (event.data.error !== undefined) {
					error.value = event.data.error;
					return;
				}
				if (!event.data.payload) {
					error.value = "No payload received from worker"; return;
				}

				const blob = new Blob([event.data.payload], { type: "image/webp" });
				dataUrl.value = URL.createObjectURL(blob);
			});

			worker.postMessage({
				type: "work",
				payload: imageUrl,
			} satisfies JxlPolyfillWorkerInput);

			await doneSignal;
			pool.returnWorker(worker);
		},
		setInput: (input: JxlPolyfillInput): void => {
			imageUrl = input;
		},
	};
}
