<script setup lang="ts">
import { computed, definePageMeta, navigateTo, ref } from '#imports'
import { toast } from 'vue-sonner'
import Button from '~/components/ui/Button.vue'
import Input from '~/components/ui/Input.vue'
import { useAuthRegister } from '~/composables/api/auth'
import { isEmailValid } from '~/lib/is-email-valid'
import { isPasswordSecure } from '~/lib/is-password-secure'

definePageMeta({ layout: 'auth', middleware: ['reverse-auth'] })

const username = ref('')
const email = ref('')
const password = ref('')
const passwordRetype = ref('')

const emailValid = computed(() => {
	if (email.value === '') {
		return true
	}
	return isEmailValid(email.value)
})
const passwordStrong = computed(() => {
	if (password.value === '') {
		return true
	}
	return isPasswordSecure(password.value)
})
const retypeMatch = computed(() => {
	if (passwordRetype.value === '') {
		return true
	}
	return password.value === passwordRetype.value
})

const register = useAuthRegister()

const registerButtonDisabled = computed(
	() =>
		username.value === '' ||
		!retypeMatch.value ||
		!passwordStrong.value ||
		!emailValid.value ||
		register.status.value === 'pending' ||
		password.value === '' ||
		passwordRetype.value === '' ||
		email.value === ''
)

function handleRegister(): void {
	if (registerButtonDisabled.value) {
		return
	}

	register.mutate(
		{
			email: email.value,
			username: username.value,
			password: password.value
		},
		{
			onSuccess() {
				toast.success('Registration successful')
				void navigateTo('/')
			},
			onError(error) {
				toast.error("Can't register", {
					description: `${error}`
				})
			}
		}
	)
}
</script>

<template>
	<Input
		v-model="username"
		class="col-span-2"
		type="text"
		label="Username"
		:disabled="register.status.value === 'pending'"
	/>
	<Input
		v-model="email"
		class="col-span-2"
		type="email"
		label="Email"
		:disabled="register.status.value === 'pending'"
		supporting-text="Invalid email address"
		:show-supporting-text="!emailValid"
		:style="emailValid ? 'default' : 'destructive'"
	/>
	<Input
		v-model="password"
		class="col-span-2"
		type="password"
		label="Password"
		:disabled="register.status.value === 'pending'"
	/>
	<Input
		v-model="passwordRetype"
		class="col-span-2"
		type="password"
		label="Retype password"
		:disabled="register.status.value === 'pending'"
		supporting-text="Passwords do not match"
		:show-supporting-text="!retypeMatch"
		@keydown.enter="handleRegister"
	/>

	<Button
		class="w-full"
		variant="outline"
		@click="
			() => {
				void navigateTo('/auth/login')
			}
		"
	>
		Back to login
	</Button>
	<Button
		class="w-full"
		:disabled="registerButtonDisabled"
		@click="handleRegister"
	>
		Register
	</Button>
</template>
