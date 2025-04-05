import { defineNuxtRouteMiddleware, navigateTo } from "nuxt/app";

import { isLoggedIn } from "~/lib/is-logged-in";

export default defineNuxtRouteMiddleware(async (to, _) => {
	const loggedIn = await isLoggedIn();
	const atAuthRoute = to.path === "/auth/login";
	if (!loggedIn && !atAuthRoute) {
		return navigateTo("/auth/login");
	}
	if (loggedIn && atAuthRoute) {
		return navigateTo("/");
	}
});
