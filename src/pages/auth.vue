<script setup lang="ts">
/* eslint no-unused-vars: 0 */

import { ref } from "vue";

import Toggle from "~/components/Toggle.vue";
import Button from "~/components/ui/Button.vue";
import Input from "~/components/ui/Input.vue";
import { useAuthLoginStore, useAuthRegisterStore, useAuthResetPasswordStore } from "~/composables/auth-screen-stores";
import { useTheme } from "~/composables/use-theme";

useTheme();

enum Mode {
	Login = "login",
	Register = "register",
	ResetPassword = "reset-password",
}
const mode = ref<Mode>(Mode.Login);

const loginStore = useAuthLoginStore();
const registerStore = useAuthRegisterStore();
const resetPasswordStore = useAuthResetPasswordStore();
</script>

<template>
	<div class="flex h-dvh flex-col items-center justify-center overflow-hidden">
		<div class="flex w-80 flex-col">
			<Toggle
				:show="mode == Mode.Login"
				class="grid w-full grid-cols-2 gap-2">
				<Input
					v-model="loginStore.login"
					label="Username or email"
					class="col-span-2"
					:disabled="loginStore.mutation.isPending || mode !== Mode.Login" />
				<Input
					v-model="loginStore.password"
					type="password"
					class="col-span-2"
					label="Password"
					:disabled="loginStore.mutation.isPending || mode !== Mode.Login" />
				<Button
					class="col-span-2 w-full"
					:disabled="loginStore.mutation.isPending || mode !== Mode.Login"
					@click="loginStore.loginAction">
					Login
				</Button>
				<Button
					variant="outline"
					class="w-full"
					:disabled="mode !== Mode.Login"
					@click="mode = Mode.Register">
					Register
				</Button>
				<Button
					variant="outline"
					class="w-full"
					:disabled="mode !== Mode.Login"
					@click="mode = Mode.ResetPassword">
					Reset password
				</Button>
			</Toggle>

			<Toggle
				:show="mode == Mode.Register"
				class="grid w-full grid-cols-2 gap-2">
				<Input
					v-model="registerStore.username"
					class="col-span-2"
					type="text"
					label="Username"
					:disabled="registerStore.mutation.isPending || mode !== Mode.Register" />
				<Input
					v-model="registerStore.email"
					class="col-span-2"
					type="email"
					label="Email"
					:disabled="registerStore.mutation.isPending || mode !== Mode.Register"
					supporting-text="Invalid email address"
					:show-supporting-text="!registerStore.isEmailValid"
					:style="registerStore.isEmailValid ? 'default' : 'destructive'" />
				<Input
					v-model="registerStore.password"
					class="col-span-2"
					type="password"
					label="Password"
					:disabled="registerStore.mutation.isPending || mode !== Mode.Register" />
				<Input
					v-model="registerStore.passwordRetype"
					class="col-span-2"
					type="password"
					label="Retype password"
					:disabled="registerStore.mutation.isPending || mode !== Mode.Register"
					supporting-text="Passwords do not match"
					:show-supporting-text="!registerStore.isPasswordRetypeMatch"
					@keydown.enter="registerStore.registerAction" />

				<Button
					class="w-full"
					variant="outline"
					:disabled="mode !== Mode.Register"
					@click="mode = Mode.Login">
					Back to login
				</Button>
				<Button
					class="w-full"
					:disabled="registerStore.registerButtonDisabled || mode !== Mode.Register"
					@click="registerStore.registerAction">
					Register
				</Button>
			</Toggle>

			<Toggle :show="mode == Mode.ResetPassword">
				<Input
					v-model="resetPasswordStore.email"
					class="mb-3 w-full"
					label="Email"
					:disabled="mode !== Mode.ResetPassword"
					@keydown.enter="resetPasswordStore.requestResetPasswordAction" />

				<Button
					class="mb-3 w-full"
					:disabled="mode !== Mode.ResetPassword"
					@click="resetPasswordStore.requestResetPasswordAction">
					Send code
				</Button>

				<Input
					v-model="resetPasswordStore.code"
					class="mb-3 w-full"
					label="Code"
					:disabled="resetPasswordStore.isConfirmResetPasswordButtonEnabled || mode !== Mode.ResetPassword"
					@keydown.enter="resetPasswordStore.confirmResetPasswordAction" />

				<Input
					v-model="resetPasswordStore.newPassword"
					class="mb-3 w-full"
					label="New password"
					:disabled="resetPasswordStore.isConfirmResetPasswordButtonEnabled || mode !== Mode.ResetPassword"
					@keydown.enter="resetPasswordStore.confirmResetPasswordAction" />
				<Input
					v-model="resetPasswordStore.newPassword"
					class="mb-3 w-full"
					label="Retype new password"
					:disabled="resetPasswordStore.isConfirmResetPasswordButtonEnabled || mode !== Mode.ResetPassword"
					supporting-text="Passwords do not match"
					:show-supporting-text="!resetPasswordStore.isPasswordRetypeMatch"
					:style="resetPasswordStore.isPasswordRetypeMatch ? 'default' : 'destructive'"
					@keydown.enter="resetPasswordStore.confirmResetPasswordAction" />
				<Button
					class="mb-1 w-full"
					:disabled="resetPasswordStore.isConfirmResetPasswordButtonEnabled || mode !== Mode.ResetPassword"
					@click="resetPasswordStore.confirmResetPasswordAction">
					Reset password
				</Button>

				<Button
					class="w-full"
					variant="outline"
					:disabled="mode !== Mode.ResetPassword"
					@click="mode = Mode.Login">
					Back to login
				</Button>
			</Toggle>
		</div>
	</div>
</template>
