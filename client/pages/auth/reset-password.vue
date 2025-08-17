<script setup lang="ts">
import { definePageMeta, navigateTo } from '#imports'
import { computed, ref } from 'vue'
import { toast } from 'vue-sonner'
import Button from '~/components/ui/Button.vue'
import Input from '~/components/ui/Input.vue'
import { useAuthForgot } from '~/composables/api/auth'

definePageMeta({ layout: 'auth', middleware: ['reverse-auth'] })

const codeSent = ref(false)
const code = ref('')
const email = ref('')
const newPassword = ref('')
const newPasswordRetype = ref('')
const retypeMatch = computed(() => {
	if (newPassword.value === '') {
		return true
	}
	return newPassword.value === newPasswordRetype.value
})

const resetPassword = useAuthForgot()
function handleSendRequestResetPassword(): void {
	resetPassword.mutate(
		{
			email: email.value
		},
		{
			onSuccess() {
				toast.success('Code sent to your email')
				codeSent.value = true
			},
			onError(error) {
				toast.error("Can't send code", {
					description: `${error}`
				})
			}
		}
	)
}

function handleConfirmResetPassword(): void {
	resetPassword.mutate(
		{
			code: code.value,
			email: email.value,
			new_password: newPassword.value
		},
		{
			onSuccess() {
				toast.success('Password reset successfully')
				void navigateTo('/auth/login')
			},
			onError(error) {
				toast.error("Can't reset password", {
					description: `${error}`
				})
			}
		}
	)
}

const canSendRequest = computed(
	() => email.value !== '' && resetPassword.status.value !== 'pending'
)
</script>

<template>
	<Toggle :show="!codeSent" class="col-span-2">
		<div class="grid w-full grid-cols-[auto,auto] gap-2">
			<Input
				v-model="email"
				label="Email"
				@keydown.enter="handleSendRequestResetPassword"
			/>

			<Button
				class="z-10 self-center"
				:disabled="!canSendRequest"
				variant="link"
				@click="handleSendRequestResetPassword"
			>
				Send code
			</Button>
		</div>
	</Toggle>

	<Toggle :show="codeSent" class="col-span-2 space-y-3">
		<Input
			v-model="code"
			class="w-full"
			label="Code sent to your email"
			:disabled="!codeSent"
		/>
		<Input
			v-model="newPassword"
			class="w-full"
			label="New password"
			:disabled="!codeSent"
		/>
		<Input
			v-model="newPasswordRetype"
			class="w-full"
			label="Retype new password"
			:disabled="!codeSent"
			supporting-text="Passwords do not match"
			:show-supporting-text="!retypeMatch"
			:style="retypeMatch ? 'default' : 'destructive'"
			@keydown.enter="handleConfirmResetPassword"
		/>
		<Button
			class="w-full"
			:disabled="!codeSent"
			@click="handleConfirmResetPassword"
		>
			Reset password
		</Button>
	</Toggle>

	<Button
		class="z-10 col-span-2 w-full"
		variant="outline"
		@click="
			() => {
				void navigateTo('/auth/login')
			}
		"
	>
		Back to login
	</Button>
</template>
