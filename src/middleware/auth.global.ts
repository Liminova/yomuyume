async function canPassCheck(): Promise<boolean> {
	try {
		const res = await fetch(new URL("/api/user/check", globalStore.instanceAddr).toString(), {
			method: "GET",
			headers: {
				Authorization: `Bearer ${globalStore.token}`,
			},
		});

		void res.json();

		if (res.ok) {
			return true;
		}
	} catch {
		return false;
	}

	return false;
}

export default defineNuxtRouteMiddleware(async (to, _) => {
	// Ignore on 404 page
	if (to.path === "/404") {
		return;
	}

	const canPass = await canPassCheck();

	if (to.path === "/auth" && canPass) {
		return navigateTo("/");
	}

	if (to.path === "/auth" && !canPass) {
		return;
	}

	if (to.path !== "/auth" && !canPass) {
		return navigateTo("/auth");
	}

	// globalStore.token = "";
	// document.cookie = "token=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;";
	// if (to.path !== "/auth") {
	// 	return navigateTo("/auth");
	// }
	// return navigateTo("/auth");
});
