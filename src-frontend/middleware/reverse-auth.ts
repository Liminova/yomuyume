import { defineNuxtRouteMiddleware, navigateTo } from "nuxt/app";

import { isLoggedIn } from "~/lib/is-logged-in";

export default defineNuxtRouteMiddleware(async () => {
	if (await isLoggedIn()) {
		return navigateTo("/");
	}
});
