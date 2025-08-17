import init, { decode } from '../blurhash-webp-wasm/blurhash_webp_wasm'
import { StoreName, setPageInDB } from '../db'
import type { BlurhashWorkerInput } from '../decode-blurhash'
import type { WorkerOutput } from './pool'

declare const self: Worker

let inited = false
let initing = false
const waitInitQueue: ((value: void | PromiseLike<void>) => void)[] = []

self.onmessage = async (
	event: MessageEvent<BlurhashWorkerInput>
): Promise<void> => {
	if (!inited) {
		if (initing) {
			await new Promise<void>((resolve) => waitInitQueue.push(resolve))
		} else {
			initing = true
			await init()
			// eslint-disable-next-line require-atomic-updates
			inited = true
			while (waitInitQueue.length > 0) {
				waitInitQueue.shift()?.()
			}
		}
	}

	if (event.data.type === 'ping') {
		self.postMessage({ type: 'pong' } satisfies WorkerOutput)
		return
	}
	if (!event.data.payload) {
		self.postMessage({
			type: 'done',
			error: 'No payload received'
		} satisfies WorkerOutput)
		return
	}
	const { id, blurhash, height, width } = event.data.payload

	const smallWidth = 12
	const smallHeight = Math.floor((smallWidth * height) / width)

	try {
		await setPageInDB({
			page: {
				id,
				data: decode(blurhash, smallWidth, smallHeight)
			},
			type: StoreName.BLURHASH
		})

		self.postMessage({ type: 'done' } satisfies WorkerOutput)
	} catch (error) {
		self.postMessage({
			type: 'done',
			error: `${error}`
		} satisfies WorkerOutput)
	}
}

self.onerror = (event: ErrorEvent): boolean => {
	self.postMessage({
		type: 'done',
		error: event.message
	} satisfies WorkerOutput)

	return true
}
