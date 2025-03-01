/* eslint-disable @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQuery } from "@tanstack/vue-query";
import { toast } from "vue-sonner";

import { BOOKMARK_PATH, FAVORITE_PATH, NewYomuyumeRequest, USER_MODIFY_PATH, USER_PROGRESS_PATH, USER_SENSITIVE_PATH, WHOAMI_PATH } from "./common";

export interface SensitiveRequest {
	password: string;
	mode: "ask" | "confirm";
	purpose: "delete_account" | "change_password" | "verify_email";
	code?: string;
	payload?: string;
}

export function useUserSensitiveAction() {
	return useMutation({
		mutationFn: async (body: SensitiveRequest) => NewYomuyumeRequest(USER_SENSITIVE_PATH, {
			method: "POST",
			body: JSON.stringify(body),
		}),
		onError(error) {
			toast.error("Can't perform sensitive action", {
				description: error.message,
			});
		},
	});
}

export function useUserFavorite(method: "PUT" | "DELETE") {
	return useMutation({
		mutationFn: async (titleID: string) => NewYomuyumeRequest(FAVORITE_PATH(titleID), { method }),
		onError(error) {
			toast.error(`Can't ${method === "PUT" ? "add" : "remove"} from favorites`, {
				description: error.message,
			});
		},
		onSuccess() {
			toast.success(`${method === "PUT" ? "Added to" : "Removed from"} favorites`);
		},
	});
}

export function useUserBookmark(method: "PUT" | "DELETE") {
	return useMutation({
		mutationFn: async (titleID: string) => NewYomuyumeRequest(BOOKMARK_PATH(titleID), { method }),
		onError(error) {
			toast.error(`Can't ${method === "PUT" ? "add" : "remove"} from bookmarks`, {
				description: error.message,
			});
		},
		onSuccess() {
			toast.success(`${method === "PUT" ? "Added to" : "Removed from"} bookmarks`);
		},
	});
}

export function useUserModifyInfo() {
	return useMutation({
		mutationFn: async (body: { username?: string; email?: string }) => NewYomuyumeRequest(USER_MODIFY_PATH, {
			method: "POST",
			body: JSON.stringify(body),
		}),
		onError(error) {
			toast.error("Can't modify user info", {
				description: error.message,
			});
		},
		onSuccess() {
			toast.success("User info modified successfully");
		},
	});
}

export function useUserSetProgress() {
	return useMutation({
		mutationFn: async (query: { titleID: string; page: number }) => NewYomuyumeRequest(USER_PROGRESS_PATH(query.titleID, query.page), {
			method: "PUT",
		}),
		onError(error) {
			toast.error("Can't set progress", {
				description: error.message,
			});
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

export function useUserWhoAmI() {
	return useQuery({
		queryKey: ["whoami"],
		queryFn: async ({ signal }) => NewYomuyumeRequest<WhoAmIResponseBody>(WHOAMI_PATH, { signal }),
	});
}
