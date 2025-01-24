/* eslint-disable @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation } from "@tanstack/vue-query";
import { toast } from "vue-sonner";

export function useLogin() {
	return useMutation({
		async mutationFn(body: { login: string; password: string }) {
			const response = await fetch("/api/auth/login", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
		onError(error) {
			toast.error("Login failed", {
				description: error.message,
			});
		},
	});
}

export function useLogout() {
	return useMutation({
		async mutationFn() {
			const response = await fetch("/api/auth/logout", {
				method: "POST",
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
		onError(error) {
			toast.error("Logout failed", {
				description: error.message,
			});
		},
	});
}

export function useRegister() {
	return useMutation({
		async mutationFn(body: { username: string; email: string; password: string }) {
			const response = await fetch("/api/auth/register", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw new Error(`[${response.statusText}] ${await response.text()}`);
			}
		},
		onError(error) {
			toast.error("Register failed", {
				description: error.message,
			});
		},
	});
}
