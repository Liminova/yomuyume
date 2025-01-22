/* eslint-disable @typescript-eslint/no-unsafe-return, @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQuery } from "@tanstack/vue-query";

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

export interface TitleTagResponse {
	id: string;
	name: string;
}

export interface GetTitleResponseBody {
	author?: string;
	bookmarks: number;
	category_id?: string;
	cover_blurhash?: string;
	cover_format?: string;
	cover_height?: number;
	cover_width?: number;
	date_updated?: string;
	description?: string;
	favorites: number;
	is_bookmark: boolean;
	is_favorite: boolean;
	is_series: boolean;
	page_read?: number;
	release?: string;
	tags?: TitleTagResponse[];
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

export interface FilterTitleRequestBody {
	category_ids?: string[];
	is_bookmarked?: boolean;
	is_favorite?: boolean;
	is_finished?: boolean;
	is_reading?: boolean;
	keywords?: string[];
	limit?: number;
	sort_by?: string;
	sort_order?: string;
	tag_ids?: string[];
}

export interface TitleFromFilter {
	author?: string;
	category_id?: string;
	cover_blurhash?: string;
	cover_format?: string;
	cover_height?: number;
	cover_width?: number;
	favorite_count?: number;
	id: string;
	page_count: number;
	page_read?: number;
	release?: string;
	title?: string;
}

export type FilterTitleResponseBody = TitleFromFilter[];

export function useFilterTitle() {
	return useMutation({
		async mutationFn(body: FilterTitleRequestBody): Promise<FilterTitleResponseBody> {
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
		queryFn: async (): Promise<Array<{ id: string; name: string }>> => {
			const response = await fetch("/api/content/tags", { method: "GET" });

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}

			return response.json();
		},
	});
}
