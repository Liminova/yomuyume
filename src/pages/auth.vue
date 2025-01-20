<script setup lang="ts">
import { ref } from "vue";

import Toggle from "~/components/Toggle.vue";
import Button from "~/components/ui/button/Button.vue";
import Input from "~/components/ui/input/Input.vue";
import { useAuthLoginStore, useAuthRegisterStore, useAuthResetPasswordStore } from "~/composables/auth-screen-stores";
import changeTheme from "~/lib/change-theme";

document.title = "Yomuyume - Auth";

changeTheme("auto");

/* eslint no-unused-vars: 0 */
enum Mode {
	Idle = "idle",
	Login = "login",
	Register = "register",
	ResetPassword = "reset-password",
}
/* eslint no-unused-vars: 2 */
const mode = ref<Mode>(Mode.Idle);

const loginStore = useAuthLoginStore();
const registerStore = useAuthRegisterStore();
const resetPasswordStore = useAuthResetPasswordStore();

</script>

<template>
	<div class="flex h-screen flex-col items-center justify-center">
		<div class="flex w-80 flex-col">
			<Toggle :show="mode == Mode.Login">
				<Input
					v-model="loginStore.login"
					class="mb-3 w-full"
					placeholder="Username or email"
					:disabled="loginStore.mutation.isPending"
				/>
				<Input
					v-model="loginStore.password"
					class="mb-3 w-full"
					type="password"
					placeholder="Password"
					:disabled="loginStore.mutation.isPending"
				/>
				<div class="grid grid-cols-2 gap-1">
					<Button
						class="col-span-2"
						@click="loginStore.loginAction"
					>
						Login
					</Button>
					<Button @click="mode = Mode.Register">
						Register
					</Button>
					<Button @click="mode = Mode.ResetPassword">
						Reset password
					</Button>
				</div>
			</Toggle>

			<Toggle :show="mode == Mode.Register">
				<Input
					v-model="registerStore.username"
					class="mb-3 w-full"
					type="text"
					label="Username"
					:disabled="registerStore.mutation.isPending"
				/>
				<Input
					v-model="registerStore.email"
					class="mb-3 w-full"
					type="email"
					label="Email"
					:disabled="registerStore.mutation.isPending"
				/>
				<Input
					v-model="registerStore.password"
					class="mb-3 w-full"
					type="password"
					label="Password"
					:disabled="registerStore.mutation.isPending"
				/>
				<Input
					v-model="registerStore.passwordRetype"
					class="mb-3 w-full"
					type="password"
					label="Retype password"
					:disabled="registerStore.mutation.isPending"
					@keydown.enter="registerStore.registerAction"
				/>

				<div class="grid grid-cols-2 gap-2">
					<Button
						class="w-full"
						@click="mode = Mode.Login"
					>
						Back to login
					</Button>
					<Button
						class="w-full"
						:disabled="registerStore.registerButtonDisabled"
						@click="registerStore.registerAction"
					>
						Register
					</Button>
				</div>
			</Toggle>

			<Toggle :show="mode == Mode.ResetPassword">
				<Input
					v-model="resetPasswordStore.email"
					class="mb-3 w-full"
					label="Email"
					@keydown.enter="resetPasswordStore.requestResetPasswordAction"
				/>

				<Button
					class="mb-3 w-full"
					@click="resetPasswordStore.requestResetPasswordAction"
				>
					Send code
				</Button>

				<Input
					v-model="resetPasswordStore.code"
					class="mb-3 w-full"
					label="Code"
					:disabled="resetPasswordStore.isConfirmResetPasswordButtonEnabled"
					@keydown.enter="resetPasswordStore.confirmResetPasswordAction"
				/>

				<Input
					v-model="resetPasswordStore.newPassword"
					class="mb-3 w-full"
					label="New password"
					:disabled="resetPasswordStore.isConfirmResetPasswordButtonEnabled"
					@keydown.enter="resetPasswordStore.confirmResetPasswordAction"
				/>

				<Input
					v-model="resetPasswordStore.newPassword"
					class="mb-3 w-full"
					label="Retype new password"
					:disabled="resetPasswordStore.isConfirmResetPasswordButtonEnabled"
					@keydown.enter="resetPasswordStore.confirmResetPasswordAction"
				/>

				<Button
					class="mb-1 w-full"
					:disabled="resetPasswordStore.isConfirmResetPasswordButtonEnabled"
					@click="resetPasswordStore.confirmResetPasswordAction"
				>
					Reset password
				</Button>

				<Button
					class="w-full"
					@click="mode = Mode.Login"
				>
					Back to login
				</Button>
			</Toggle>
		</div>
	</div>
</template>
