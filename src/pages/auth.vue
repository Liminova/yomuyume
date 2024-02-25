<script setup lang="ts">
import Snackbar from "~/components/Snackbar.vue";
import InstanceAddress from "~/components/auth/InstanceAddress.vue";
import LoginMode from "~/components/auth/Login.vue";
import PasswordlessMode from "~/components/auth/Passwordless.vue";
import RegisterMode from "~/components/auth/Register.vue";
import ResetPassword from "~/components/auth/ResetPassword.vue";
import { AuthScreen, authStore } from "~/components/auth/utils";
import changeTheme from "~/composables/changeTheme";

changeTheme("auto");

document.title = "Yomuyume - Auth";
</script>

<template>
	<div class="flex h-screen flex-col items-center justify-center">
		<Snackbar :message="authStore.snackbarMessage" @close="authStore.snackbarMessage = ''" />
		<div class="flex w-80 flex-col">
			<InstanceAddress />

			<Toggle :show="authStore.screen == AuthScreen.Passwordless">
				<PasswordlessMode />
			</Toggle>

			<Toggle :show="authStore.screen == AuthScreen.Login">
				<LoginMode />
			</Toggle>

			<Toggle :show="authStore.screen == AuthScreen.Register">
				<RegisterMode />
			</Toggle>

			<Toggle :show="authStore.screen == AuthScreen.ResetPassword">
				<ResetPassword />
			</Toggle>
		</div>
	</div>
</template>
