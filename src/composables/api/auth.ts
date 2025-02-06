/* eslint-disable @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQueryClient } from "@tanstack/vue-query";
import { toast } from "vue-sonner";

import { FORGOT_PATH, LOGIN_PATH, LOGOUT_PATH, REGISTER_PATH, ResponseError } from "./constants";

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
			toast.error("Can't login", {
				description: error.message,
			});
		},
		onSuccess() {
			toast.success("Login successful");
		},
	});
}

export function useLogout() {
	const queryClient = useQueryClient();

	return useMutation({
		async mutationFn() {
			const response = await fetch(LOGOUT_PATH);

			if (!response.ok) {
				throw new Error("You're not even logged in!");
			}

			queryClient.setQueryData(["whoami"], null);
		},
		onError(error) {
			toast.error("Can't logout", {
				description: error.message,
			});
		},
		onSuccess() {
			toast.success("Logout successful");
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
			toast.error("Can't register", {
				description: error.message,
			});
		},
		onSuccess() {
			toast.success("Registration successful");
		},
	});
}

export interface ForgotRequest {
	email: string;
	code?: string;
	new_password?: string;
}

export function useForgot() {
	return useMutation({
		async mutationFn(body: ForgotRequest) {
			const response = await fetch(FORGOT_PATH, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify(body),
			});

			if (!response.ok) {
				throw await ResponseError(response);
			}
		},
		onError(error) {
			toast.error("Can't request password reset", {
				description: error.message,
			});
		},
	});
}
