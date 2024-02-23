import imageDecode from "../decodePipeline/image";

type MyMessageData = [string, string, string]; /** src, format, jwt token */

self.onmessage = async (event: MessageEvent<MyMessageData>) => {
	self.postMessage(await imageDecode(event.data));
};
