/* eslint-disable @typescript-eslint/no-unsafe-return, @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQuery } from "@tanstack/vue-query";

import { BOOKMARK_PATH, FAVORITE_PATH, ResponseError, USER_MODIFY_PATH, USER_PROGRESS_PATH, USER_SENSITIVE_PATH, WHOAMI_PATH } from "./constants";

export interface SensitiveRequest {
	password: string;
	mode: "ask" | "confirm";
	purpose: "delete_account" | "change_password" | "verify_email";
	code?: string;
	payload?: string;
}

export function useSensitiveAction() {
	return useMutation({
		async mutationFn(body: SensitiveRequest): Promise<void> {
			const response = await fetch(USER_SENSITIVE_PATH, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw await ResponseError(response);
			}
		},
	});
}

export function useFavorite(method: "PUT" | "DELETE") {
	return useMutation({
		async mutationFn(titleID: string): Promise<void> {
			const response = await fetch(FAVORITE_PATH(titleID), { method });
			if (!response.ok) {
				throw await ResponseError(response);
			}
		},
	});
}

export function useBookmark(method: "PUT" | "DELETE") {
	return useMutation({
		async mutationFn(titleID: string): Promise<void> {
			const response = await fetch(BOOKMARK_PATH(titleID), { method });
			if (!response.ok) {
				throw await ResponseError(response);
			}
		},
	});
}

export function useModifyUserInfo() {
	return useMutation({
		async mutationFn(body: { username?: string; email?: string }): Promise<void> {
			const response = await fetch(USER_MODIFY_PATH, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw await ResponseError(response);
			}
		},
	});
}

export function useSetProgress() {
	return useMutation({
		async mutationFn(query: { titleID: string; page: number }): Promise<void> {
			const response = await fetch(USER_PROGRESS_PATH(query.titleID, query.page), {
				method: "PUT",
			});

			if (!response.ok) {
				throw await ResponseError(response);
			}
		},
	});
}

export interface WhoAmIResponseBody {
	email: string;
	ip_address: string;
	profile_picture?: string;
	updated_at?: string;
	user_id: string;
	username: string;
	verified_at?: string;
}

export function useWhoAmI() {
	return useQuery({
		queryKey: ["whoami"],
		async queryFn() {
			const response = await fetch(WHOAMI_PATH);

			if (!response.ok) {
				throw await ResponseError(response);
			}

			return response.json();
		},
	});
}
