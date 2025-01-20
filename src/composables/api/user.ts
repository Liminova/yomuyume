/* eslint-disable @typescript-eslint/no-unsafe-return, @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQuery } from "@tanstack/vue-query";

export function useDeleteAccount() {
	return useMutation({
		async mutationFn(): Promise<void> {
			const response = await fetch("/api/user/delete", {
				method: "GET",
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
	});
}

export function useConfirmDeleteAccount() {
	return useMutation({
		async mutationFn(body: { code: string; password: string }): Promise<void> {
			const response = await fetch("/api/user/delete", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
	});
}

export function useAddFavorite() {
	return useMutation({
		async mutationFn(titleId: string): Promise<void> {
			const response = await fetch(`/api/user/favorite/${titleId}`, { method: "PUT" });
			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
	});
}

export function useDeleteFavorite() {
	return useMutation({
		async mutationFn(titleId: string): Promise<void> {
			const response = await fetch(`/api/user/favorite/${titleId}`, { method: "DELETE" });
			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
	});
}

export function useModifyUserInfo() {
	return useMutation({
		async mutationFn(body: { username?: string; email?: string; current_password?: string; new_password?: string }): Promise<void> {
			const response = await fetch("/api/user/modify", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
	});
}

export function useSetProgress() {
	return useMutation({
		async mutationFn(body: { titleId: string; page: number }): Promise<void> {
			const response = await fetch(`/api/user/progress/${body.titleId}/${body.page}`, {
				method: "PUT",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
	});
}

export function useResetPassword() {
	return useMutation({
		async mutationFn(email: string): Promise<void> {
			const response = await fetch(`/api/user/reset/${email}`, { method: "GET" });
			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
	});
}

export function useConfirmResetPassword() {
	return useMutation({
		async mutationFn(body: { code: string; new_password: string }): Promise<void> {
			const response = await fetch("/api/user/reset", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
	});
}

export function useValidateEmail() {
	return useMutation({
		async mutationFn(): Promise<void> {
			const response = await fetch("/api/user/verify", { method: "GET" });
			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
	});
}

export function useConfirmValidateEmail() {
	return useMutation({
		async mutationFn(body: { code: string }): Promise<void> {
			const response = await fetch("/api/user/verify", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
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
			const response = await fetch("/api/user/whoami", {
				method: "GET",
				headers: { "Content-Type": "application/json" },
				credentials: import.meta.dev ? "include" : "same-origin",
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}

			return response.json();
		},
	});
}

export function useAddBookmark() {
	return useMutation({
		async mutationFn(titleId: string): Promise<void> {
			const response = await fetch(`/api/user/bookmark/${titleId}`, { method: "PUT" });
			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
	});
}

export function useDeleteBookmark() {
	return useMutation({
		async mutationFn(titleId: string): Promise<void> {
			const response = await fetch(`/api/user/bookmark/${titleId}`, { method: "DELETE" });
			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
	});
}
