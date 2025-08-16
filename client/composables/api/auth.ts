import { useMutation } from '../use-mutation'
import { FORGOT_PATH, LOGIN_PATH, LOGOUT_PATH, REGISTER_PATH } from './common'
import { useUserWhoAmI } from './user'
import { clearNuxtData } from 'nuxt/app'

export function useAuthLogin() {
	const { refresh } = useUserWhoAmI()

	return useMutation({
		async mutationFn(body: { login: string; password: string }) {
			return $fetch(LOGIN_PATH, {
				method: 'POST',
				credentials: 'same-origin',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(body)
			})
		},
		async onSuccess() {
			await refresh()
		}
	})
}

export function useAuthLogout() {
	return useMutation({
		async mutationFn() {
			await $fetch(LOGOUT_PATH, {
				credentials: 'same-origin'
			})
			clearNuxtData('whoami')
		}
	})
}

export function useAuthRegister() {
	return useMutation({
		async mutationFn(body: {
			username: string
			email: string
			password: string
		}) {
			return $fetch(REGISTER_PATH, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(body)
			})
		}
	})
}

export function useAuthForgot() {
	return useMutation({
		async mutationFn(body: {
			email: string
			code?: string
			new_password?: string
		}) {
			return $fetch(FORGOT_PATH, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(body)
			})
		}
	})
}
