async function status(): Promise<void> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL("/api/utils/status", import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return "/api/utils/status";
	})()
	const response = await fetch(endpoint, {
		method: "GET",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
	});
	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}
}

export interface TagsMapResponseBody {
	data: Array<{
		id: number;
		name: string;
	}>;
}

async function tags(): Promise<TagsMapResponseBody> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL("/api/utils/tags", import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return "/api/utils/tags";
	})();
	const response = await fetch(endpoint, {
		method: "GET",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
	});
	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}

	return (await response.json()) as TagsMapResponseBody;
}

export interface ScanningProgressResponseBody {
	scanning_completed: boolean;
	scanning_progress: number;
}

async function scanningProgress(): Promise<ScanningProgressResponseBody> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL("/api/utils/scanning_progress", import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return "/api/utils/scanning_progress";
	})();
	const response = await fetch(endpoint, {
		method: "GET",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
	});
	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}

	return (await response.json()) as ScanningProgressResponseBody;
}

export default { status, tags, scanningProgress };
