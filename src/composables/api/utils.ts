import newRoute from "./newRoute";
import { StatusResponseBody, TagsMapResponseBody, GenericResponseBody } from "~/composables/bridge";

async function status(endpoint: string): Promise<{ data?: StatusResponseBody; message?: string }> {
	let res: Response;

	try {
		res = await fetch(new URL("/api/utils/status", endpoint).toString(), {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
				Accept: "bitcode",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	const data = StatusResponseBody.from_bitcode(buffer);

	if (res.ok) {
		return { data };
	}

	return { message: GenericResponseBody.from_bitcode(buffer).message };
}

async function tags(): Promise<{ data?: TagsMapResponseBody; message?: string }> {
	let res: Response;

	try {
		res = await fetch(newRoute("/api/utils/tags"), {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
				Authorization: `Bearer ${globalStore.token}`,
				Accept: "bitcode",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	const data = TagsMapResponseBody.from_bitcode(buffer);

	if (res.ok) {
		return { data };
	}

	return { message: GenericResponseBody.from_bitcode(buffer).message };
}

export default { status, tags };
