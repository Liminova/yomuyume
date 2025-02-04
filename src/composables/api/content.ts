/* eslint-disable @typescript-eslint/no-unsafe-return, @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQuery } from "@tanstack/vue-query";

import { GET_CATEGORIES_PATH, GET_CHAPTER_PATH, GET_ONESHOT_PATH, GET_SERIES_PATH, GET_TAGS_PATH, ResponseError, SEARCH_PATH } from "./constants";

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
			const response = await fetch(GET_CATEGORIES_PATH, {
				headers: { "Content-Type": "application/json" },
			});

			if (!response.ok) {
				throw await ResponseError(response);
			}

			return response.json();
		},
	});
}

export type TitleTagName = string;
export type TitleTagID = string;
export type TitleTag = [TitleTagID, TitleTagName];

export interface BaseTitleResponse {
	id: string;
	title?: string;
	author?: string;

	category_id?: string;
	description?: string;
	release?: string;

	cover_blurhash?: string;
	/// Full cover width, you might want to clamp this down to a much,
	/// much smaller value (<32px) before decoding the blurhash
	cover_width?: number;
	/// Full cover height, you might want to clamp this down to a much,
	/// much smaller value (<32px) before decoding the blurhash
	cover_height?: number;
	cover_jxl?: boolean;

	date_updated?: string;

	/// ID, Name
	tags?: TitleTag[];
	/// Null if the value is 0
	favorites?: number;
	/// Null if the value is 0
	bookmarks?: number;

	is_favorite: boolean;
	is_bookmark: boolean;
	/// Null if the value is 0 or haven't read
	page_read?: number;
}

export interface BasePageResponse {
	id: string;
	blurhash?: string;
	width?: number;
	height?: number;
	jxl: boolean;
	description?: string;
}

export interface OneshotResponse extends BaseTitleResponse {
	pages?: BasePageResponse[];
}

export function useGetOneshot(id: string) {
	return useQuery({
		queryKey: ["title", id],
		async queryFn(): Promise<OneshotResponse> {
			const response = await fetch(GET_ONESHOT_PATH(id));

			if (!response.ok) {
				throw await ResponseError(response);
			}

			return response.json();
		},
	});
}

export interface SeriesResponse extends BaseTitleResponse {
	chapters?: Array<{
		id: string;
		number: number;
		description?: string;
	}>;
}

export function useGetSeries(id: string) {
	return useQuery({
		queryKey: ["title", id],
		async queryFn(): Promise<SeriesResponse> {
			const response = await fetch(GET_SERIES_PATH(id));

			if (!response.ok) {
				throw await ResponseError(response);
			}

			return response.json();
		},
	});
}

export interface ChapterResponse extends BaseTitleResponse {
	id: string;
	number: number;
	description?: string;

	pages?: BasePageResponse[];
}

export function useGetChapter(id: string) {
	return useQuery({
		queryKey: ["title", id],
		async queryFn(): Promise<ChapterResponse> {
			const response = await fetch(GET_CHAPTER_PATH(id));

			if (!response.ok) {
				throw await ResponseError(response);
			}

			return response.json();
		},
	});
}

export interface SearchRequest {
	term?: string;

	category_ids?: string[];
	tag_ids?: string[];
	release_year?: number;

	offset?: number;
	limit?: number;
	order_by?: string;
	is_ascending?: boolean;
}

export interface InnerSearchResponseTitle extends BaseTitleResponse {
	is_series: boolean;
}

export interface SearchResponse {
	data?: InnerSearchResponseTitle[];
	offset: number;
	limit: number;
}

export function useSearchTitle() {
	return useMutation({
		async mutationFn(body: SearchRequest): Promise<SearchResponse> {
			const response = await fetch(SEARCH_PATH, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw await ResponseError(response);
			}

			return response.json();
		},
	});
}

export function useGetTags() {
	return useQuery({
		queryKey: ["tags"],
		queryFn: async (): Promise<TitleTag[]> => {
			const response = await fetch(GET_TAGS_PATH);

			if (!response.ok) {
				throw await ResponseError(response);
			}

			return response.json();
		},
	});
}
