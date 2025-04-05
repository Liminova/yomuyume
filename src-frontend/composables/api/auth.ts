/* eslint-disable @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQueryClient } from "@tanstack/vue-query";

import { FORGOT_PATH, LOGIN_PATH, LOGOUT_PATH, NewYomuyumeRequest, REGISTER_PATH } from "./common";

export function useAuthLogin() {
	return useMutation({
		async mutationFn(body: { login: string; password: string }) {
			return NewYomuyumeRequest(LOGIN_PATH, {
				method: "POST",
				body: JSON.stringify(body),
			});
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
	});
}

export function useAuthRegister() {
	return useMutation({
		async mutationFn(body: { username: string; email: string; password: string }) {
			return NewYomuyumeRequest(REGISTER_PATH, {
				method: "POST",
				body: JSON.stringify(body),
			});
		},
	});
}

export function useAuthForgot() {
	return useMutation({
		async mutationFn(body: { email: string; code?: string; new_password?: string }) {
			return NewYomuyumeRequest(FORGOT_PATH, {
				method: "POST",
				body: JSON.stringify(body),
			});
		},
	});
}
