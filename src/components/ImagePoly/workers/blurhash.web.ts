import blurhashDecode from "../decodePipeline/blurhash";

self.onmessage = async (event: MessageEvent<[string, number, number]>): Promise<void> => {
	self.postMessage(await blurhashDecode(event.data));
};
