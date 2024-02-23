import blurhashDecode from "../decodePipeline/blurhash";

self.onmessage = async (event: MessageEvent<[string, number, number]>) => {
	self.postMessage(await blurhashDecode(event.data));
};
