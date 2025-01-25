/* eslint-disable @typescript-eslint/no-unsafe-return, @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQuery } from "@tanstack/vue-query";

export interface CategoriesResponseBodyInner {
	id: string;
	name?: string;
	description?: string;

	cover_blurhash?: string;
	cover_height?: number;
	cover_width?: number;
	cover_jxl?: string;
}

export type GetCategoriesResponseBody = Array<{ id: string; name: string; description?: string }>;

export function useGetCategories() {
	return useQuery({
		queryKey: ["categories"],
		async queryFn(): Promise<Array<{ id: string; name: string; description?: string }>> {
			const response = await fetch("/api/content/categories", {
				method: "GET",
				headers: { "Content-Type": "application/json" },
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}

			return response.json();
		},
	});
}

export type TitleTagName = string;
export type TitleTagID = string;
export type TitleTag = [TitleTagID, TitleTagName];

export interface TitleResponseBody {
	author?: string;
	bookmarks?: number;
	category_id?: string;
	cover_blurhash?: string;
	cover_height?: number;
	cover_jxl: boolean;
	cover_width?: number;
	date_updated?: string;
	description?: string;
	favorites?: number;
	id: string,
	is_bookmark: boolean;
	is_favorite: boolean;
	is_series: boolean;
	page_read?: number;
	release?: string;
	tags?: TitleTag[];
	title?: string;
}

export function useGetTitle(id: string) {
	return useQuery({
		queryKey: ["title", id],
		async queryFn(): Promise<TitleResponseBody> {
			const response = await fetch(`/api/content/title/${id}`, {
				method: "GET",
				headers: { "Content-Type": "application/json" },
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}

			return response.json();
		},
	});
}

export interface FilterTitleRequestBody {
	category_ids?: string[];
	is_bookmarked?: boolean;
	is_favorite?: boolean;
	is_finished?: boolean;
	is_reading?: boolean;
	keywords?: string[];
	limit?: number;
	order_by?: string;
	sort_order?: string;
	tag_ids?: string[];
}

export function useFilterTitle() {
	return useMutation({
		async mutationFn(body: FilterTitleRequestBody): Promise<TitleResponseBody[]> {
			const response = await fetch("/api/content/filter", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}

			return response.json();
		},
	});
}

export function useGetTags() {
	return useQuery({
		queryKey: ["tags"],
		queryFn: async (): Promise<TitleTag[]> => {
			const response = await fetch("/api/content/tags", { method: "GET" });

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}

			return response.json();
		},
	});
}
