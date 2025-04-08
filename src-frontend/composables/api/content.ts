/* eslint-disable @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useFetch } from "nuxt/app";

import { useInfiniteQuery } from "../use-infinite-query";
import { GET_CATEGORIES_PATH, GET_PAGES_PATH, GET_TAGS_PATH, GET_TITLE_PATH, SEARCH_PATH } from "./common";

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
	return useFetch(GET_CATEGORIES_PATH, {
		credentials: "same-origin",
		key: "categories",
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
	return useFetch<TitleResponse>(GET_TITLE_PATH(titleId), {
		credentials: "same-origin",
		key: `title-${titleId}`,
	});
}

export function useContentGetPages(chapterId: string) {
	return useFetch<BasePageResponse[]>(GET_PAGES_PATH(chapterId), {
		credentials: "same-origin",
		key: `pages-${chapterId}`,
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
	is_reading?: boolean;
}

export interface SearchResponse {
	data?: BaseTitleResponse[];
	offset: number;
	limit: number;
}

export function useContentSearchTitle(query?: SearchQuery) {
	const limit = query?.limit ?? 10;

	return useInfiniteQuery({
		queryKey: `search-${JSON.stringify(query)}`,
		async queryFn({ pageParam = 0, signal }) {
			return $fetch<SearchResponse>(SEARCH_PATH, {
				method: "POST",
				signal,
				query: {
					...query,
					offset: pageParam,
					limit,
				},
			});
		},
		initialPageParam: query?.offset ?? 0,
		getNextPageParam(lastPage) {
			if (lastPage.data === undefined || lastPage.data.length === 0) { return null; }
			return lastPage.offset + limit;
		},
		flattened(pages) {
			return pages.flatMap(page => page.data.data ?? []);
		},
	});
}

export const FavoriteTitlesQuery: SearchQuery = { is_favorite: true } as const;
export const BookmarkedTitlesQuery: SearchQuery = { is_bookmarked: true } as const;
export const ReadingTitlesQuery: SearchQuery = { is_reading: true, order_by: "progress_last_read_at", is_ascending: true } as const;

export function useContentGetTags() {
	return useFetch<TitleTag[]>(GET_TAGS_PATH, {
		credentials: "same-origin",
		key: "tags",
	});
}
