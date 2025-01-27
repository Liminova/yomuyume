import { defineStore } from "pinia";
import { computed, ref, watchEffect } from "vue";

import { useLogin, useRegister } from "./api/auth";
import { useConfirmResetPassword, useResetPassword } from "./api/user";

function validatePassword(password: string): boolean {
	const hasUppercase = (/[A-Z]/u).test(password);
	const hasLowercase = (/[a-z]/u).test(password);
	const nasNumeric = (/[0-9]/u).test(password);
	const hasSpecial = (/[!@#$%^&*(),.?":{}|<>]/u).test(password);
	const hasValidLength = password.length >= 8 && password.length <= 100;

	return hasUppercase && hasLowercase && nasNumeric && hasSpecial && hasValidLength;
}

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

	const isEmailValid = computed(() => {
		if (email.value === "") { return true; }
		return (/\S+@\S+\.\S+/u).test(email.value);
	});
	const isPasswordStrong = computed(() => validatePassword(password.value));
	const isPasswordRetypeMatch = computed(() => {
		if (passwordRetype.value === "") { return true; }
		return password.value === passwordRetype.value;
	});

	const mutation = useRegister();

	const registerButtonDisabled = computed(() => username.value === ""
		|| !isPasswordRetypeMatch.value
		|| !isPasswordStrong.value
		|| !isEmailValid.value
		|| mutation.isPending.value
		|| password.value === ""
		|| passwordRetype.value === ""
		|| email.value === "");

	return {
		username,
		email,
		password,

		isEmailValid,
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

	const isPasswordStrong = computed(() => validatePassword(newPassword.value));
	const isPasswordRetypeMatch = computed(() => {
		if (newPasswordRetype.value === "") { return true; }
		return newPassword.value === newPasswordRetype.value;
	});

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
