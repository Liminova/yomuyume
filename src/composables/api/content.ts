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

export type GetCategoriesResponseBody = Array<{ id: string; name?: string; description?: string }>;

export function useGetCategories() {
	return useQuery({
		queryKey: ["categories"],
		async queryFn(): Promise<GetCategoriesResponseBody> {
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

export interface GetTitleResponseBody {
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
	id: string;
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
		async queryFn(): Promise<GetTitleResponseBody> {
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

export interface SearchRequestBody {
	term?: string;

	category_ids?: string[];
	tag_ids?: string[];
	release_year?: number;

	offset?: number;
	limit?: number;
	order_by?: string;
	is_ascending?: boolean;
}

export interface SearchResponseBody {
	data?: GetTitleResponseBody[];
	offset: number;
	limit: number;
}

export function useSearchTitle() {
	return useMutation({
		async mutationFn(body: SearchRequestBody): Promise<SearchResponseBody> {
			const response = await fetch("/api/content/search", {
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
