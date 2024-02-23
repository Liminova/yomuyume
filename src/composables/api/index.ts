import newRoute from "./newRoute";

type CategoryItemSrvResponse = {
	id: string;
	name: string;
	description?: string;
};

type CategorySrvResponse = {
	data?: Array<CategoryItemSrvResponse>;
	message?: string;
};

type CategoriesFnResponse = CategorySrvResponse;

async function categories(): Promise<CategoriesFnResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute("/api/index/categories"), {
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
		const data = (await res.json()) as CategorySrvResponse;

		return { data: res.ok ? data.data : undefined, message: data.message ?? "" };
	} catch {
		return { message: "Can't parse server response" };
	}
}

type FilterItemSrvResponse = {
	id: string;
	title: string;
	author?: string;
	category_id: string;
	release_date?: string;
	favorite_count?: number;
	page_count: number;
	page_read?: number;

	blurhash: string;
	width: number;
	height: number;
	format: string;
};

type FilterSrvResponse = {
	data?: Array<FilterItemSrvResponse>;
	message?: string;
};

type FilterFnResponse = FilterSrvResponse;

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
}): Promise<FilterFnResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute("/api/index/filter"), {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				Authorization: `Bearer ${globalStore.token}`,
			},
			body: JSON.stringify(body),
		});
	} catch (e) {
		return { message: (e as { message: string }).message };
	}

	try {
		const data = (await res.json()) as FilterSrvResponse;

		return { data: res.ok ? data.data : undefined, message: data.message ?? "" };
	} catch {
		return { message: "Can't parse server response" };
	}
}

type TitleServerResponse = {
	category_id: string;
	title: string;
	author?: string;
	description?: string;
	release?: string;
	thumbnail: {
		blurhash: string;
		width: number;
		height: number;
		format: string;
	};
	tag_ids: Array<number>;
	pages: Array<{
		id: string;
		format: string;
		description?: string;
	}>;
	favorites?: number;
	bookmarks?: number;
	is_favorite?: boolean;
	is_bookmark?: boolean;
	page_read?: number;
	date_added: string;
	date_updated: string;

	message?: string;
};

/** What the below fn returns */
type TitleFnResponse = {
	data?: TitleServerResponse;
	message?: string;
};

async function title(id: string): Promise<TitleFnResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/index/title/${id}`), {
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
		const data = (await res.json()) as TitleServerResponse;

		return { data: res.ok ? data : undefined, message: data.message ?? "" };
	} catch {
		return { message: "Can't parse server response" };
	}
}

export default { categories, filter, title };
export type {
	FilterItemSrvResponse as FilterItemServerResponse,
	TitleServerResponse,
	CategoryItemSrvResponse as CategoryItemServerResponse,
	CategorySrvResponse as CategoryServerResponse,
	CategoriesFnResponse,
};
