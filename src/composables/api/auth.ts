/* eslint-disable @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation } from "@tanstack/vue-query";
import { toast } from "vue-sonner";

import { LOGIN_PATH, LOGOUT_PATH, REGISTER_PATH, ResponseError } from "./constants";

export function useLogin() {
	return useMutation({
		async mutationFn(body: { login: string; password: string }) {
			const response = await fetch(LOGIN_PATH, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw await ResponseError(response);
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
			const response = await fetch(LOGOUT_PATH);

			if (!response.ok) {
				throw new Error("You're not even logged in!");
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
			const response = await fetch(REGISTER_PATH, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw await ResponseError(response);
			}
		},
		onError(error) {
			toast.error("Register failed", {
				description: error.message,
			});
		},
	});
}
