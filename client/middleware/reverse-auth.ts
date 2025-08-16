import { defineNuxtRouteMiddleware, navigateTo, useNuxtData } from 'nuxt/app'
import { useUserWhoAmI, type WhoAmIResponseBody } from '~/composables/api/user'

export default defineNuxtRouteMiddleware(async () => {
	const { error, refresh } = useUserWhoAmI()
	const { data } = useNuxtData<WhoAmIResponseBody>('whoami')

	if (!data.value && !error.value) {
		await refresh()
	}

	if (data.value !== null) {
		return navigateTo('/')
	}
})
