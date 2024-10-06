
export interface CategoriesResponseBody {
	data: Array<{
		id: string;
		name: string;
		description?: string;
	}>
}

async function categories(): Promise<CategoriesResponseBody> {
	const endpoint = (()=>{
		if (import.meta.dev) {
			// @ts-ignore env does exist
			return new URL("/api/index/categories", import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return "/api/index/categories";
	})()
	const response = await fetch(endpoint, {
		method: "GET",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
	});
	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}

	return (await response.json()) as CategoriesResponseBody;
}

export interface FilterTitleResponseBody {
	data: Array<{
		id: string;
		title: string;
		author?: string;
		category_id?: string;
		release?: string;
		favorite_count?: number;
		page_count: number;
		page_read?: number;

		blurhash?: string;
		blurhash_width?: number;
		blurhash_height?: number;
	}>
}

async function filter(body: {
	keywords?: Array<string>;
	category_ids?: Array<string>;
	tag_ids?: Array<number>;
	limit?: number;

	is_reading?: boolean;
	is_finished?: boolean;
	is_bookmarked?: boolean;
	is_favorite?: boolean;

	sort_by?: string;
	sort_order?: string;
}): Promise<FilterTitleResponseBody> {
	const endpoint = (()=>{
		if (import.meta.dev) {
			// @ts-ignore env does exist
			return new URL("/api/index/filter", import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return "/api/index/filter";
	})()
	const response = await fetch(endpoint, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
		body: JSON.stringify(body),
	});
	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}

	return (await response.json()) as FilterTitleResponseBody;
}

export interface TitleResponseBody {
	data?: {
		category_id?: string;
		title: string;
		author?: string;
		description?: string;
		release_date_unix_utc_seconds?: number;

		cover_blurhash?: string;
		blurhash_width?: number;
		blurhash_height?: number;

		tag_ids: Array<number>;
		pages: Array<{
			id: string;
			format: string;
			description?: string;
		}>
		favorites?: number;
		bookmarks?: number;
		is_favorite?: boolean;
		is_bookmark?: boolean;
		page_read?: number;
		date_added_unix_utc_seconds: number;
		date_updated_unix_utc_seconds?: number;
	}
}

async function title(id: string): Promise<TitleResponseBody> {
	const endpoint = (()=>{
		if (import.meta.dev) {
			// @ts-ignore env does exist
			return new URL(`/api/index/title/${id}`, import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return `/api/index/title/${id}`;
	})()
	const response = await fetch(endpoint, {
		method: "GET",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
	});
	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}

	return (await response.json()) as TitleResponseBody;
}

export default { categories, filter, title };
