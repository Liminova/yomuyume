async function favorite(
	titleId: string,
	action: "DELETE" | "PUT"
): Promise<void> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL(`/api/user/favorite/${titleId}`, import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return `/api/user/favorite/${titleId}`;
	})()
	const response = await fetch(endpoint, {
		method: action,
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
		body: JSON.stringify({ titleId }),
	});

	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}
}

async function bookmark(
	titleId: string,
	action: "DELETE" | "PUT"
): Promise<void> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL(`/api/user/bookmark/${titleId}`, import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return `/api/user/bookmark/${titleId}`;
	})()
	const response = await fetch(endpoint, {
		method: action,
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
		body: JSON.stringify({ titleId }),
	});

	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}
}

async function progress(
	titleId: string,
	page: number
): Promise<void> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL(`/api/user/progress/${titleId}/${page}`, import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return `/api/user/progress/${titleId}/${page}`;
	})()
	const response = await fetch(endpoint, {
		method: "PUT",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
		body: JSON.stringify({ titleId, page }),
	});

	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}
}

/** Send a request to delete the user. */
async function deleteAccount(): Promise<void> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL(`/api/user/delete`, import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return `/api/user/delete`;
	})()
	const response = await fetch(endpoint, {
		method: "GET",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
	});

	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}
}

/** Confirm the deletion of the user. */
async function deleteAccountConfirm(code: string, password: string): Promise<void> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL("/api/auth/delete", import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return "/api/auth/delete";
	})()
	const response = await fetch(endpoint, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
		body: JSON.stringify({ code, password }),
	});
	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}
}

async function resetPassword(email: string): Promise<void> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL(`/api/auth/reset/${email}`, import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return `/api/auth/reset/${email}`;
	})()
	const response = await fetch(endpoint, {
		method: "GET",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
	});

	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}
}

async function resetPasswordConfirm(
	code: string,
	new_password: string,
): Promise<void> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL("/api/auth/reset", import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return "/api/auth/reset";
	})()
	const response = await fetch(endpoint, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
		body: JSON.stringify({ code, new_password }),
	});

	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}
}

/** Send a request to verify the email. */
async function validateEmail(): Promise<void> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL("/api/auth/verify", import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return "/api/auth/verify";
	})()
	const response = await fetch(endpoint, {
		method: "GET",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
	});
	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}
}

/** Confirm the verification of the email. */
async function validateEmailConfirm(token: string): Promise<void> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-ignore env does exist
			return new URL("/api/auth/verify", import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return "/api/auth/verify";
	})();
	const response = await fetch(endpoint, {
		method: "GET",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
		body: JSON.stringify({ token }),
	});
	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}
}

async function modifyInfo(body: {
	username?: string;
	email?: string;
	current_password?: string;
	new_password?: string;
}): Promise<void> {
	const endpoint = (() => {
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL("/api/user/modify", import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return "/api/user/modify";
	})();
	const response = await fetch(endpoint, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
		body: JSON.stringify(body),
	});
	if (!response.ok) {
		throw new Error(`[${response.statusText}] ${await response.text()}`.trim());
	}
}

export default {
	favorite,
	bookmark,
	progress,
	validateEmail,
	validateEmailConfirm,
	resetPassword,
	resetPasswordConfirm,
	deleteAccount,
	deleteAccountConfirm,
};
