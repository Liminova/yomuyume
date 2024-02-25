import newRoute from "./newRoute";
import type { FilterTitleResponseBody, CategoryResponse } from "~/composables/bridge";
import {
	FilterResponseBody,
	CategoriesResponseBody,
	GenericResponseBody,
	TitleResponseBody,
} from "~/composables/bridge";

async function categories(): Promise<{ data?: Array<CategoryResponse>; message?: string }> {
	let res: Response;

	try {
		res = await fetch(newRoute("/api/index/categories"), {
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

	const data = CategoriesResponseBody.from_bitcode(buffer);

	if (res.ok) {
		return { data: data?.data };
	}

	return { message: GenericResponseBody.from_bitcode(buffer).message };
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
}): Promise<{ data?: Array<FilterTitleResponseBody>; message?: string }> {
	let res: Response;

	try {
		res = await fetch(newRoute("/api/index/filter"), {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				Authorization: `Bearer ${globalStore.token}`,
				Accept: "bitcode",
			},
			body: JSON.stringify(body),
		});
	} catch (e) {
		return { message: (e as { message: string }).message };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	const data = FilterResponseBody.from_bitcode(buffer);

	if (res.ok && data !== undefined) {
		return { data: data.data };
	}

	return { message: GenericResponseBody.from_bitcode(buffer).message };
}

async function title(id: string): Promise<{ data?: TitleResponseBody; message?: string }> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/index/title/${id}`), {
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

	const data = TitleResponseBody.from_bitcode(buffer);

	if (res.ok && data !== undefined) {
		return { data };
	}

	return { message: GenericResponseBody.from_bitcode(buffer).message };
}

export default { categories, filter, title };
