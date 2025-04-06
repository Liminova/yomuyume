export interface WorkerInputWrapper<I> {
	type: "ping" | "work";
	payload?: I;
}

export interface WorkerOutput {
	type: "pong" | "done";
	error?: ErrorEvent | string;
}

type OnmessageSignature = (_: MessageEvent<WorkerOutput>)=> Promise<void>;

interface MyWorker {
	inner: Worker;
	onmessage?: OnmessageSignature;
}

export class WorkerPool<I> {
	private readonly newWorkerFn: ()=> Worker;
	private semaphore: number;
	private readonly pool: MyWorker[] = [];

	private readonly queue: Array<(worker: MyWorker)=> void> = [];

	public constructor(newWorkerFn: ()=> Worker, maxConcurrentWorker = 4) {
		this.newWorkerFn = newWorkerFn;
		this.semaphore = maxConcurrentWorker;
	}

	/**
	 * We have a race condition between .onmessage and .postMessage, so this
	 * would be used ONLY when the worker is first created to ensure that it's
	 * ready to receive messages.
	 */
	private async ensureWorkerReady(myWorker: MyWorker): Promise<MyWorker> {
		const pollingIntervalID = window.setInterval(() => {
			myWorker.inner.postMessage({ type: "ping" } satisfies WorkerInputWrapper<I>);
		}, 0);

		return new Promise((resolve) => {
			myWorker.inner.onmessage = (event: MessageEvent<WorkerOutput>): void => {
				void myWorker.onmessage?.(event);
				if (event.data.type === "pong") {
					clearInterval(pollingIntervalID);
					resolve(myWorker);
				}
			};
		});
	}

	/**
	 * Get a worker from the pool. If there's no worker available, create a new
	 * one and return it.
	 *
	 * Also assign the onmessage handler "to" the worker.
	 */
	public async getWorker(): Promise<MyWorker> {
		const worker = this.pool.shift();
		if (worker) { return worker; }

		if (this.semaphore > 0) {
			this.semaphore -= 1;
			return this.ensureWorkerReady({
				inner: this.newWorkerFn(),
			} satisfies MyWorker);
		}

		return new Promise((resolve) => {
			this.queue.push((worker: MyWorker) => { resolve(worker); });
		});
	}

	public returnWorker(worker: MyWorker): void {
		const next = this.queue.shift();
		if (next) { next(worker); return; }
		this.pool.push(worker);
		this.semaphore++;
	}
}
