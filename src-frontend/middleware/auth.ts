import { defineNuxtRouteMiddleware, navigateTo } from "nuxt/app";

import { NewYomuyumeRequest, WHOAMI_PATH } from "~/composables/api/common";

export default defineNuxtRouteMiddleware(async (to, _) => {
	const loggedIn = await new Promise<boolean>(resolve => NewYomuyumeRequest(WHOAMI_PATH)
		.then(() => {
			resolve(true);
		})
		.catch((e: unknown) => {
			if (typeof e === "object"
				&& e !== null
				&& "message" in e
				&& typeof e.message === "string"
				&& e.message.includes("[401]")) {
				resolve(false);
			}
			throw e;
		}),
	);
	const atAuthRoute = to.path === "/auth";
	if (!loggedIn && !atAuthRoute) {
		return navigateTo("/auth");
	}
	if (loggedIn && atAuthRoute) {
		return navigateTo("/");
	}
});
