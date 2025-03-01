/* eslint-disable @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { toast } from "vue-sonner";

import { GET_CATEGORIES_PATH, GET_CHAPTER_PATH, GET_ONESHOT_PATH, GET_SERIES_PATH, GET_TAGS_PATH, NewYomuyumeRequest, SEARCH_PATH } from "./common";

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
		queryFn: async ({ signal }) => NewYomuyumeRequest<GetCategoriesResponseBody>(GET_CATEGORIES_PATH, { signal }),
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

export function useContentGetOneshot(id: string) {
	return useQuery({
		queryKey: ["title", id],
		queryFn: async ({ signal }) => NewYomuyumeRequest<OneshotResponse>(GET_ONESHOT_PATH(id), { signal }),
	});
}

export interface SeriesResponse extends BaseTitleResponse {
	chapters?: Array<{
		id: string;
		number: number;
		description?: string;
	}>;
}

export function useContentGetSeries(id: string) {
	return useQuery({
		queryKey: ["title", id],
		queryFn: async ({ signal }) => NewYomuyumeRequest<SeriesResponse>(GET_SERIES_PATH(id), { signal }),
	});
}

export interface ChapterResponse extends BaseTitleResponse {
	id: string;
	number: number;
	description?: string;

	pages?: BasePageResponse[];
}

export function useContentGetChapter(id: string) {
	return useQuery({
		queryKey: ["title", id],
		queryFn: async ({ signal }) => NewYomuyumeRequest<ChapterResponse>(GET_CHAPTER_PATH(id), { signal }),
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

export function useContentSearchTitle() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: async (body: SearchRequest) => NewYomuyumeRequest<SearchResponse>(SEARCH_PATH, {
			method: "POST",
			body: JSON.stringify(body),
		}),
		onSuccess(data, variables) {
			queryClient.setQueryData(["search", variables], data);
		},
		onError(error) {
			toast.error("Can't perform search", {
				description: error.message,
			});
		},
	});
}

export function useContentActiveSearchTitle(body: SearchRequest) {
	return useQuery({
		queryKey: ["search", body],
		queryFn: async ({ signal }) => NewYomuyumeRequest<SearchResponse>(SEARCH_PATH, {
			method: "POST",
			body: JSON.stringify(body),
			signal,
		}),
	});
}

export function useContentGetTags() {
	return useQuery({
		queryKey: ["tags"],
		queryFn: async ({ signal }) => NewYomuyumeRequest<TitleTag[]>(GET_TAGS_PATH, { signal }),
	});
}
