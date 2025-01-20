import { defineStore } from "pinia";
import { computed, ref, watchEffect } from "vue";

import isStrongPassword from "~/lib/is-strong-password";

import { useLogin, useRegister } from "./api/auth";
import { useConfirmResetPassword, useResetPassword } from "./api/user";

export const useAuthLoginStore = defineStore("auth-screen-store-login", () => {
	const login = ref("");
	const password = ref("");

	const mutation = useLogin();

	return {
		login,
		password,
		mutation,
		loginAction(): void {
			mutation.mutate({ login: login.value, password: password.value });
		},
	};
});

export const useAuthRegisterStore = defineStore("auth-screen-store-register", () => {
	const username = ref("");
	const email = ref("");
	const password = ref("");
	const passwordRetype = ref("");

	const isPasswordStrong = computed(() => isStrongPassword(password.value));
	const isPasswordRetypeMatch = computed(() => password.value === passwordRetype.value);

	const mutation = useRegister();

	const registerButtonDisabled = computed(() => !isPasswordRetypeMatch.value || !isPasswordStrong.value || mutation.isPending.value);

	return {
		username,
		email,
		password,
		isPasswordStrong,
		passwordRetype,
		isPasswordRetypeMatch,

		mutation,
		registerButtonDisabled,
		registerAction(): void {
			mutation.mutate({
				username: username.value,
				email: email.value,
				password: password.value,
			});
		},
	};
});

export const useAuthResetPasswordStore = defineStore("auth-screen-store-reset-password", () => {
	const code = ref("");
	const codeSent = ref(false);
	const email = ref("");
	const newPassword = ref("");
	const newPasswordRetype = ref("");

	const isPasswordStrong = computed(() => isStrongPassword(newPassword.value));
	const isPasswordRetypeMatch = computed(() => newPassword.value === newPasswordRetype.value);

	const mutation = useResetPassword();
	const mutation2 = useConfirmResetPassword();

	const isRequestResetPasswordButtonEnabled = computed(() => !isPasswordRetypeMatch.value || !isPasswordStrong.value || mutation.isPending.value);
	const isConfirmResetPasswordButtonEnabled = computed(() => !isPasswordRetypeMatch.value || !isPasswordStrong.value || mutation2.isPending.value || !codeSent.value);

	watchEffect(() => {
		if (mutation.isSuccess.value) {
			codeSent.value = true;
		}
	});

	return {
		email,
		code,
		codeSent,
		newPassword,
		newPasswordRetype,
		isPasswordStrong,
		isPasswordRetypeMatch,

		isRequestResetPasswordButtonEnabled,
		isConfirmResetPasswordButtonEnabled,

		mutation,
		mutation2,

		requestResetPasswordAction(): void {
			mutation.mutate(email.value);
		},
		confirmResetPasswordAction(): void {
			mutation2.mutate({ code: code.value, new_password: newPassword.value });
		},
	};
});
