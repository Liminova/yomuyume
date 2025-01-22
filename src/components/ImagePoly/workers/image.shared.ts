import imageDecode from "../decodePipeline/image";
import { IMAGE_WORKER_COUNT } from "../worker-count";

type MyMessageData = [string, string, string]; /** src, format, jwt token */

const queue: Array<{ data: MyMessageData; port: MessagePort }> = [];
let activeWorkers = 0;

async function processQueue(): Promise<void> {
	if (activeWorkers < IMAGE_WORKER_COUNT && queue.length > 0) {
		activeWorkers++;
		const job = queue.shift();

		if (!job) {
			return;
		}

		job.port.postMessage(await imageDecode(job.data));
		activeWorkers--;
		await processQueue();
	}
}

// @ts-expect-error - self is a SharedWorkerGlobalScope
self.onconnect = (event: MessageEvent<MyMessageData>): void => {
	const port = event.ports[0];

	port.onmessage = async (event: MessageEvent<MyMessageData>): Promise<void> => {
		queue.push({ data: event.data, port });
		await processQueue();
	};
};
