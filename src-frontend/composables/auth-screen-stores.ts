import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { toast } from "vue-sonner";

import { useAuthForgot, useAuthLogin, useAuthRegister } from "./api/auth";

function validatePassword(password: string): boolean {
	const hasUppercase = (/[A-Z]/u).test(password);
	const hasLowercase = (/[a-z]/u).test(password);
	const nasNumeric = (/[0-9]/u).test(password);
	const hasSpecial = (/[!@#$%^&*(),.?":{}|<>]/u).test(password);
	const hasValidLength = password.length >= 8 && password.length <= 100;

	return hasUppercase && hasLowercase && nasNumeric && hasSpecial && hasValidLength;
}

function validateEmail(email: string): boolean {
	return (/\S+@\S+\.\S+/u).test(email);
}

export const useAuthLoginStore = defineStore("auth-screen-store-login", () => {
	const login = ref("");
	const password = ref("");

	const mutation = useAuthLogin();

	return {
		login,
		password,
		mutation,
		loginAction(cb?: ()=> void): ()=> void {
			return (): void => {
				mutation.mutate({
					login: login.value,
					password: password.value,
				}, {
					onSuccess() { cb?.(); },
				});
			};
		},

		resetFields(): void {
			login.value = "";
			password.value = "";
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
		return validateEmail(email.value);
	});
	const isPasswordStrong = computed(() => validatePassword(password.value));
	const isPasswordRetypeMatch = computed(() => {
		if (passwordRetype.value === "") { return true; }
		return password.value === passwordRetype.value;
	});

	const mutation = useAuthRegister();

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
		registerAction(cb?: ()=> void): ()=> void {
			return (): void => {
				mutation.mutate({
					username: username.value,
					email: email.value,
					password: password.value,
				}, {
					onSuccess() { cb?.(); },
				});
			};
		},

		resetFields(): void {
			username.value = "";
			email.value = "";
			password.value = "";
			passwordRetype.value = "";
		},
	};
});

export const useAuthResetPasswordStore = defineStore("auth-screen-store-reset-password", () => {
	const code = ref("");
	const email = ref("");
	const newPassword = ref("");
	const newPasswordRetype = ref("");

	const codeSent = ref(false);

	const notStrongEnough = computed(() => {
		if (newPassword.value === "") { return false; }
		return !validatePassword(newPassword.value);
	});
	const retypeMismatch = computed(() => {
		if (newPasswordRetype.value === "") { return false; }
		return newPassword.value !== newPasswordRetype.value;
	});

	const mutation = useAuthForgot();

	const canSendRequest = computed(() => validateEmail(email.value)
		&& !mutation.isPending.value,
	);
	const canSendConfirm = computed(() => newPassword.value !== ""
		&& !notStrongEnough.value
		&& !retypeMismatch.value
		&& codeSent.value
		&& !mutation.isPending.value,
	);

	return {
		email,
		code,
		codeSent,
		newPassword,
		newPasswordRetype,
		notStrongEnough,
		retypeMismatch,

		canSendRequest,
		canSendConfirm,

		mutation,

		sendReqResetPass(): void {
			mutation.mutate({ email: email.value }, {
				onSuccess() {
					toast.success("Code sent to your email");
					codeSent.value = true;
				},
			});
		},
		sendConfirmResetPass(cb?: ()=> void): ()=> void {
			return (): void => {
				mutation.mutate({
					email: email.value,
					code: code.value,
					new_password: newPassword.value,
				}, {
					onSuccess() {
						toast.success("Password reset successfully");
						cb?.();
					},
				});
			};
		},

		resetFields(): void {
			email.value = "";
			code.value = "";
			newPassword.value = "";
			newPasswordRetype.value = "";
			codeSent.value = false;
		},
	};
});
