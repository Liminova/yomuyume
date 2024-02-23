import newRoute from "./newRoute";

type StatusSrvResponse = {
	server_time: string;
	version: string;
	echo?: string;

	message?: string;
};
type StatusFnResponse = {
	data?: StatusSrvResponse;
	message?: string;
};
async function status(endpoint: string): Promise<StatusFnResponse> {
	let res: Response;

	try {
		res = await fetch(new URL("/api/utils/status", endpoint).toString(), {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message };
	}

	try {
		const data = (await res.json()) as StatusSrvResponse;

		return { data: res.ok ? data : undefined, message: data.message ?? "" };
	} catch {
		return { message: "Can't parse server response" };
	}
}

type TagsServerResponse = {
	data?: Array<[number, string]>;
	message?: string;
};
type TagsFnResponse = TagsServerResponse;

async function tags(): Promise<TagsFnResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute("/api/utils/tags"), {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
				Authorization: `Bearer ${globalStore.token}`,
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message };
	}

	try {
		const data = (await res.json()) as TagsServerResponse;

		return { data: res.ok ? data.data : undefined, message: data.message ?? "" };
	} catch {
		return { message: "Can't parse server response" };
	}
}

type SsimEvalTitleServerResponse = {
	id: string;
	title: string;
	desc: string;
	tags: Array<number>;
	blurhash: string;
	width: number;
	height: number;
	format: string;
};
type SsimEvalServerResponse = {
	title_a: SsimEvalTitleServerResponse;
	title_b: SsimEvalTitleServerResponse;
	ssim: number;
	message?: string;
};
type SsimEvalFnResponse = {
	data?: SsimEvalServerResponse;
	message?: string;
};

// Return 2 random titles and their ssim
async function ssimEval(): Promise<SsimEvalFnResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute("/api/utils/ssim_eval"), {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
				Authorization: `Bearer ${globalStore.token}`,
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message };
	}

	try {
		const data = (await res.json()) as SsimEvalServerResponse;

		return { data: res.ok ? data : undefined, message: data.message ?? "" };
	} catch {
		return { message: "Can't parse server response" };
	}
}

export default { status, tags, ssimEval };
export type { SsimEvalServerResponse, SsimEvalTitleServerResponse };
