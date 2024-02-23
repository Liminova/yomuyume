<script setup lang="ts">
import "@material/web/progress/linear-progress.js";
import "@material/web/button/filled-tonal-button.js";
import "@material/web/textfield/outlined-text-field.js";
import "@material/web/button/text-button.js";
import { AuthScreen, authStore, State } from "./utils";
import isStrongPassword from "~/composables/isStrongPassword";
import isValidEmail from "~/composables/isValidEmail";

const username = ref("");
const email = ref("");
const password = ref("");
const passwordRetype = ref("");
const registerState = ref(State.Idle);

async function register() {
	registerState.value = State.Loading;

	// client-side validation
	if (password.value !== passwordRetype.value) {
		authStore.snackbarMessage = "Passwords do not match";
		registerState.value = State.Error;
		return;
	}

	const checkEmail = isValidEmail(email.value);

	if (!checkEmail.isValid) {
		authStore.snackbarMessage = checkEmail.message;
		registerState.value = State.Error;
		return;
	}

	const checkPassword = isStrongPassword(password.value);

	if (!checkPassword.isStrong) {
		authStore.snackbarMessage = checkPassword.message;
		registerState.value = State.Error;
		return;
	}

	// server-side
	const { ok, message } = await authApi.register({
		username: username.value,
		email: email.value,
		password: password.value,
	});

	if (!ok) {
		authStore.snackbarMessage = message;
		registerState.value = State.Error;
		return;
	}

	registerState.value = State.Loaded;
	authStore.screen = AuthScreen.Login;
}
</script>

<template>
	<md-outlined-text-field v-model="username" class="mb-3 w-full" type="text" label="Username" />

	<md-outlined-text-field v-model="email" class="mb-3 w-full" type="email" label="Email" />

	<md-outlined-text-field
		v-model="password"
		class="mb-3 w-full"
		type="password"
		label="Password"
	/>

	<md-outlined-text-field
		v-model="passwordRetype"
		class="mb-3 w-full"
		type="password"
		label="Retype password"
		@keydown.enter="register"
	/>

	<Toggle :show="registerState === State.Loading">
		<md-linear-progress indeterminate class="mb-3 w-full" />
	</Toggle>

	<div class="grid grid-cols-2 gap-2">
		<md-text-button class="w-full" @click="authStore.screen = AuthScreen.Login">
			Back to login
		</md-text-button>
		<md-filled-tonal-button class="w-full" @click="register"> Register </md-filled-tonal-button>
	</div>
</template>
