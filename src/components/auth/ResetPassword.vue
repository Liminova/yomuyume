<script setup lang="ts">
import "@material/web/button/text-button.js";
import "@material/web/button/filled-tonal-button.js";
import "@material/web/textfield/outlined-text-field.js";
import { AuthScreen, authStore } from "./utils";

const email = ref("");
const token = ref("");
const newPassword = ref("");
const tokenSent = ref(false);

async function sendToken() {
	const { message } = await userApi.resetPassword(email.value);

	authStore.snackbarMessage = message;
}

async function reset() {
	const { message } = await userApi.confirmReset(newPassword.value, token.value);

	authStore.snackbarMessage = message;
	authStore.screen = AuthScreen.Login;
}
</script>

<template>
	<md-outlined-text-field
		v-model="email"
		class="mb-3 w-full"
		label="Email"
		@keydown.enter="sendToken"
	/>

	<md-filled-tonal-button class="mb-3 w-full" @click="sendToken">
		Send token
	</md-filled-tonal-button>

	<md-outlined-text-field
		v-model="token"
		class="mb-3 w-full"
		label="Token"
		:disabled="!tokenSent"
		@keydown.enter="reset"
	/>

	<md-outlined-text-field
		v-model="newPassword"
		class="mb-3 w-full"
		label="New password"
		:disabled="!tokenSent"
		@keydown.enter="reset"
	/>

	<md-filled-tonal-button class="mb-1 w-full" :disabled="!tokenSent" @click="reset">
		Reset password
	</md-filled-tonal-button>

	<md-text-button class="w-full" @click="authStore.screen = AuthScreen.Login">
		Back to login
	</md-text-button>
</template>
