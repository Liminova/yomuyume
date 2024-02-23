import blurhashDecode from "../decodePipeline/blurhash";
import { BLURHASH_WORKER_COUNT } from "../workerCount";

type MyMessageData = [string, number, number]; /** blurhash, width, height */

const queue: Array<{ data: MyMessageData; port: MessagePort }> = [];
let activeWorkers = 0;

// @ts-expect-error - self is a SharedWorkerGlobalScope
self.onconnect = (event: MessageEvent<MyMessageData>) => {
	const port = event.ports[0];

	port.onmessage = async (event: MessageEvent<MyMessageData>) => {
		queue.push({ data: event.data, port });
		await processQueue();
	};
};

async function processQueue() {
	if (activeWorkers < BLURHASH_WORKER_COUNT && queue.length > 0) {
		activeWorkers++;
		const job = queue.shift();

		if (!job) {
			return;
		}

		job.port.postMessage(await blurhashDecode(job.data));
		activeWorkers--;
		await processQueue();
	}
}
