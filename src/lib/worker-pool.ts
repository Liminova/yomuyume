export interface WorkerInputWrapper<I> {
	type: "ping" | "work";
	payload?: I;
}

export interface WorkerOutputWrapper<O, E = unknown> {
	type: "pong" | "done";
	payload?: O;
	error?: E | ErrorEvent | string;
}

export class WorkerPool<I, O, E = unknown> {
	private readonly newWorkerFn: ()=> Worker;
	private semaphore: number;
	private readonly pool: Worker[] = [];
	private readonly queue: Array<(_: Worker)=> void> = [];

	public constructor(newWorkerFn: ()=> Worker, maxConcurrentWorker = 4) {
		this.newWorkerFn = newWorkerFn;
		this.semaphore = maxConcurrentWorker;
	}

	private async ensureWorkerReady(worker: Worker, onmessage: (_: MessageEvent<WorkerOutputWrapper<O, E>>)=> void): Promise<Worker> {
		const pollingIntervalID = window.setInterval(() => {
			worker.postMessage({ type: "ping" } satisfies WorkerInputWrapper<I>);
		}, 0);

		return new Promise((resolve) => {
			worker.onmessage = (event: MessageEvent<WorkerOutputWrapper<O, E>>): void => {
				onmessage(event);
				if (event.data.type === "pong") {
					clearInterval(pollingIntervalID);
					resolve(worker);
				}
			};
		});
	}

	public async getWorker(onmessage: (_: MessageEvent<WorkerOutputWrapper<O, E>>)=> void): Promise<Worker> {
		const worker = this.pool.shift();
		if (worker) {
			return this.ensureWorkerReady(worker, onmessage);
		}

		if (this.semaphore > 0) {
			this.semaphore -= 1;
			return this.ensureWorkerReady(this.newWorkerFn(), onmessage);
		}

		return new Promise((resolve) => {
			this.queue.push((worker: Worker) => { resolve(worker); });
		});
	}

	public returnWorker(worker: Worker): void {
		const next = this.queue.shift();
		if (next) { next(worker); return; }
		this.pool.push(worker);
		this.semaphore++;
	}
}
