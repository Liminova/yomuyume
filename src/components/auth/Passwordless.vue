<script setup lang="ts">
import "@material/web/progress/linear-progress.js";
import "@material/web/button/filled-tonal-button.js";
import "@material/web/textfield/outlined-text-field.js";
import { AuthScreen, authStore, State } from "./utils";

const email = ref("");
const sendCodeState = ref(State.Idle);
const loginCode = ref("");
const loginState = ref(State.Idle);

function sendCode() {
	sendCodeState.value = State.Loading;
	setTimeout(() => {
		// TODO: remove this on production
		if (email.value.includes("FORCE_ERROR")) {
			sendCodeState.value = State.Error;
			authStore.snackbarMessage = "Invalid email";
			return;
		}

		sendCodeState.value = State.Loaded;
	}, 1000);
}

function login() {
	// TODO: implement login api
	loginState.value = State.Loading;
	setTimeout(async () => {
		// TODO: remove this on production
		if (loginCode.value.includes("FORCE_ERROR")) {
			loginState.value = State.Error;
			authStore.snackbarMessage = "Invalid code";
			return;
		}

		loginState.value = State.Loaded;
		await navigateTo("/");
	}, 1000);
}
</script>

<template>
	<!-- Input email -->
	<Toggle :show="authStore.screen === AuthScreen.Passwordless">
		<md-outlined-text-field
			v-model="email"
			label="Email"
			class="mb-3 w-full"
			@keydown.enter="sendCode"
		/>
	</Toggle>

	<!-- Visualize state -->
	<Toggle :show="sendCodeState === State.Loading">
		<md-linear-progress indeterminate class="mb-3 w-full" />
	</Toggle>

	<!-- Send code button -->
	<Toggle :show="sendCodeState === State.Idle">
		<md-filled-tonal-button
			class="w-full"
			:disabled="sendCodeState !== State.Idle"
			@click="sendCode"
		>
			Send code
		</md-filled-tonal-button>
	</Toggle>

	<!-- Input code -->
	<Toggle :show="sendCodeState === State.Loaded">
		<md-outlined-text-field
			v-model="loginCode"
			class="mb-3 w-full"
			label="Verification code"
			:disabled="!(sendCodeState === State.Loaded)"
			@keydown.enter="login"
		/>
	</Toggle>

	<!-- Visualize state -->
	<Toggle :show="loginState === State.Loading">
		<md-linear-progress indeterminate class="mb-3 w-full" />
	</Toggle>

	<!-- Login button -->
	<Toggle :show="sendCodeState === State.Loaded">
		<md-filled-tonal-button
			class="w-full"
			:disabled="sendCodeState !== State.Loaded"
			@click="login"
		>
			Login
		</md-filled-tonal-button>
	</Toggle>
</template>
