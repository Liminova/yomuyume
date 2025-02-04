/* eslint-disable @typescript-eslint/no-unsafe-return, @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQuery } from "@tanstack/vue-query";

import { GET_SCANNING_PROGRESS_PATH, GET_STATUS_PATH, ResponseError } from "./constants";

export function useGetLibraryScanningProgress() {
	return useQuery({
		queryKey: ["scanning_progress"],
		async queryFn(): Promise<{ scanning_completed: boolean; scanning_progress: number }> {
			const response = await fetch(GET_SCANNING_PROGRESS_PATH);

			if (!response.ok) {
				throw await ResponseError(response);
			}

			return response.json();
		},
	});
}

export function useGetServerStatus() {
	return useQuery({
		queryKey: ["status"],
		async queryFn(): Promise<{ server_time: string; version: string }> {
			const response = await fetch(GET_STATUS_PATH);

			if (!response.ok) {
				throw await ResponseError(response);
			}

			return response.json();
		},
	});
}

export function usePostServerStatus() {
	return useMutation({
		async mutationFn(body: { echo: string }): Promise<{ server_time: string; version: string }> {
			const response = await fetch(GET_STATUS_PATH, {
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
