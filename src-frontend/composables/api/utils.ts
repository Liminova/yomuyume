/* eslint-disable @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQuery } from "@tanstack/vue-query";

import { GET_SCANNING_PROGRESS_PATH, GET_STATUS_PATH, NewYomuyumeRequest } from "./common";

export interface ScanningProgressResponse {
	scanning_completed: boolean;
	scanning_progress: number;
}

export function useGetLibraryScanningProgress() {
	return useQuery({
		queryKey: ["scanning_progress"],
		queryFn: async ({ signal }): Promise<ScanningProgressResponse> => NewYomuyumeRequest<ScanningProgressResponse>(GET_SCANNING_PROGRESS_PATH, { signal }),
	});
}

export interface ServerStatusResponse {
	server_time: string;
	version: string;
}

export function useGetServerStatus() {
	return useQuery({
		queryKey: ["status"],
		queryFn: async ({ signal }): Promise<ServerStatusResponse> => NewYomuyumeRequest<ServerStatusResponse>(GET_STATUS_PATH, { signal }),
	});
}

export function usePostServerStatus() {
	return useMutation({
		mutationFn: async (body: { echo: string }): Promise<ServerStatusResponse> => NewYomuyumeRequest(GET_STATUS_PATH, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify(body),
		}),
	});
}
