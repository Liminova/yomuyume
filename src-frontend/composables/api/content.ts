/* eslint-disable @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useInfiniteQuery, useQuery } from "@tanstack/vue-query";

import { GET_CATEGORIES_PATH, GET_PAGES_PATH, GET_TAGS_PATH, GET_TITLE_PATH, NewYomuyumeRequest, SEARCH_PATH } from "./common";

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
		async queryFn({ signal }) {
			return NewYomuyumeRequest<GetCategoriesResponseBody>(GET_CATEGORIES_PATH, { signal });
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

	progress_page_id?: number;
	progress_percent?: number;
	progress_last_read_at?: string;
}

export interface BasePageResponse {
	id: string;
	blurhash?: string;
	width?: number;
	height?: number;
	jxl: boolean;
	description?: string;
}

export interface TitleResponse extends BaseTitleResponse {
	chapters?: Array<{
		id: string;
		number: number;
		description?: string;
	}>;
}

export function useContentGetTitle(titleId: string) {
	return useQuery({
		queryKey: ["title", titleId],
		async queryFn({ signal }) {
			return NewYomuyumeRequest<TitleResponse>(GET_TITLE_PATH(titleId), { signal });
		},
	});
}

export function useContentGetPages(chapterId: string) {
	return useQuery({
		queryKey: ["pages", chapterId],
		async queryFn({ signal }) {
			return NewYomuyumeRequest<BasePageResponse[]>(GET_PAGES_PATH(chapterId), { signal });
		},
	});
}

export interface SearchQuery {
	term?: string;

	category_ids?: string[];
	tag_ids?: string[];
	release_year?: number;

	offset?: number;
	limit?: number;
	order_by?: "title" | "release" | "date_updated" | "author" | "progress_last_read_at";
	is_ascending?: boolean;

	is_bookmarked?: boolean;
	is_favorite?: boolean;
}

export interface SearchResponse {
	data?: BaseTitleResponse[];
	offset: number;
	limit: number;
}

export function useContentSearchTitle(query?: SearchQuery) {
	const limit = query?.limit ?? 10;

	return useInfiniteQuery({
		queryKey: ["search", query],
		async queryFn({ pageParam = 0, signal }) {
			return NewYomuyumeRequest<SearchResponse>(SEARCH_PATH, {
				method: "POST",
				signal,
			}, {
				...query,
				offset: pageParam,
				limit,
			});
		},
		initialPageParam: query?.offset ?? 0,
		getNextPageParam(lastPage) {
			if (lastPage.data === undefined || lastPage.data.length === 0) { return null; }
			return lastPage.offset + limit;
		},
		getPreviousPageParam(firstPage) {
			if (firstPage.offset === 0) { return null; }
			return firstPage.offset - limit;
		},
	});
}

export function useContentGetTags() {
	return useQuery({
		queryKey: ["tags"],
		async queryFn({ signal }) {
			return NewYomuyumeRequest<TitleTag[]>(GET_TAGS_PATH, { signal });
		},
	});
}
