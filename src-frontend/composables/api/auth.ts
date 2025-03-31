/* eslint-disable @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQueryClient } from "@tanstack/vue-query";
import { toast } from "vue-sonner";

import { FORGOT_PATH, LOGIN_PATH, LOGOUT_PATH, NewYomuyumeRequest, REGISTER_PATH } from "./common";

export function useAuthLogin() {
	return useMutation({
		mutationFn: async (body: { login: string; password: string }): Promise<void> => NewYomuyumeRequest(LOGIN_PATH, {
			method: "POST",
			body: JSON.stringify(body),
		}),
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

export function useAuthLogout() {
	const queryClient = useQueryClient();
	return useMutation({
		async mutationFn() {
			await NewYomuyumeRequest(LOGOUT_PATH);
			queryClient.setQueryData(["whoami"], null);
		},
		onError(error) {
			toast.error("Can't logout", {
				description: error.message,
			});
		},
	});
}

export function useAuthRegister() {
	return useMutation({
		mutationFn: async (body: { username: string; email: string; password: string }) => NewYomuyumeRequest(REGISTER_PATH, {
			method: "POST",
			body: JSON.stringify(body),
		}),
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

export function useAuthForgot() {
	return useMutation({
		mutationFn: async (body: {
			email: string;
			code?: string;
			new_password?: string;
		}) => NewYomuyumeRequest(FORGOT_PATH, {
			method: "POST",
			body: JSON.stringify(body),
		}),
		onError(error) {
			toast.error("Can't request password reset", {
				description: error.message,
			});
		},
	});
}
