<script setup lang="ts">
/* eslint no-unused-vars: 0 */

import { ref } from "vue";

import { navigateTo } from "#app";
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
const resetPassStore = useAuthResetPasswordStore();
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
					@click="loginStore.loginAction(() => {
						loginStore.resetFields();
						void navigateTo('/');
					})">
					Login
				</Button>
				<Button
					variant="outline"
					class="w-full"
					:disabled="mode !== Mode.Login"
					@click="() => {
						mode = Mode.Register;
						registerStore.resetFields();
					}">
					Register
				</Button>
				<Button
					variant="outline"
					class="w-full"
					:disabled="mode !== Mode.Login"
					@click="() => {
						mode = Mode.ResetPassword;
						resetPassStore.resetFields();
					}">
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
					@click="() => {
						mode = Mode.Login;
						loginStore.resetFields();
					}">
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
				<Toggle :show="!resetPassStore.codeSent">
					<div class="mb-3 grid w-full grid-cols-[auto,auto] gap-2">
						<Input
							v-model="resetPassStore.email"
							label="Email"
							:disabled="mode !== Mode.ResetPassword"
							@keydown.enter="resetPassStore.sendReqResetPass" />

						<Button
							class="z-10 self-center"
							:disabled="!resetPassStore.canSendRequest || mode !== Mode.ResetPassword"
							variant="link"
							@click="resetPassStore.sendReqResetPass">
							Send code
						</Button>
					</div>
				</Toggle>

				<Toggle :show="resetPassStore.codeSent">
					<Input
						v-model="resetPassStore.code"
						class="mb-3 w-full"
						label="Code sent to your email"
						:disabled="!resetPassStore.codeSent || mode !== Mode.ResetPassword"
						@keydown.enter="resetPassStore.sendConfirmResetPass(() => {
							mode = Mode.Login;
							loginStore.resetFields();
						})" />
					<Input
						v-model="resetPassStore.newPassword"
						class="mb-3 w-full"
						label="New password"
						:disabled="!resetPassStore.codeSent || mode !== Mode.ResetPassword"
						@keydown.enter="resetPassStore.sendConfirmResetPass(() => {
							mode = Mode.Login;
							loginStore.resetFields();
						})" />
					<Input
						v-model="resetPassStore.newPasswordRetype"
						class="mb-3 w-full"
						label="Retype new password"
						:disabled="!resetPassStore.codeSent || mode !== Mode.ResetPassword"
						supporting-text="Passwords do not match"
						:show-supporting-text="resetPassStore.retypeMismatch"
						:style="resetPassStore.retypeMismatch ? 'destructive' : 'default'"
						@keydown.enter="resetPassStore.sendConfirmResetPass(() => {
							mode = Mode.Login;
							loginStore.resetFields();
						})" />
					<Button
						class="mb-3 w-full"
						:disabled="!resetPassStore.codeSent || mode !== Mode.ResetPassword"
						@click="resetPassStore.sendConfirmResetPass(() => {
							mode = Mode.Login;
							loginStore.resetFields();
						})">
						Reset password
					</Button>
				</Toggle>

				<Button
					class="z-10 w-full"
					variant="outline"
					:disabled="mode !== Mode.ResetPassword"
					@click="() => {
						mode = Mode.Login;
						loginStore.resetFields();
					}">
					Back to login
				</Button>
			</Toggle>
		</div>
	</div>
</template>
