import { defineNuxtRouteMiddleware, navigateTo, useNuxtData } from 'nuxt/app'
import { type WhoAmIResponseBody, useUserWhoAmI } from '~/composables/api/user'

export default defineNuxtRouteMiddleware(async (to, _) => {
	const { error, refresh } = useUserWhoAmI()
	const { data } = useNuxtData<WhoAmIResponseBody>('whoami')

	if (!data.value && !error.value) await refresh()

	const unauthorized = error.value?.statusCode === 401
	const atAuthRoute = to.path === '/auth/login'

	if (unauthorized && !atAuthRoute) return navigateTo('/auth/login')

	if (!unauthorized && atAuthRoute) return navigateTo('/')
})
