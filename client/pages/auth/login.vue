<script setup lang="ts">
import { definePageMeta, navigateTo } from '#imports'
import { ref } from 'vue'
import { toast } from 'vue-sonner'
import Button from '~/components/ui/Button.vue'
import Input from '~/components/ui/Input.vue'
import { useAuthLogin } from '~/composables/api/auth'

definePageMeta({ layout: 'auth', middleware: ['reverse-auth'] })

const usernameOrEmail = ref('')
const password = ref('')
const login = useAuthLogin()
function handleLogin(): void {
	login.mutate(
		{
			login: usernameOrEmail.value,
			password: password.value
		},
		{
			onSuccess() {
				toast.success('Login successful')
				void navigateTo('/')
			},
			onError(error) {
				toast.error("Can't login", {
					description: error.message
				})
			}
		}
	)
}
</script>

<template>
	<Input
		v-model="usernameOrEmail"
		label="Username or email"
		class="col-span-2"
	/>
	<Input
		v-model="password"
		type="password"
		label="Password"
		class="col-span-2"
	/>
	<Button
		class="col-span-2 w-full"
		:disabled="login.isPending.value"
		@click="handleLogin"
	>
		Login
	</Button>
	<Button
		variant="outline"
		class="w-full"
		@click="
			() => {
				void navigateTo('/auth/register')
			}
		"
	>
		Register
	</Button>
	<Button
		variant="outline"
		class="w-full"
		@click="
			() => {
				void navigateTo('/auth/reset-password')
			}
		"
	>
		Reset password
	</Button>
</template>
