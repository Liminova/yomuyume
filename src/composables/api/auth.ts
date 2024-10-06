
async function login(body: {
	login: string;
	password: string;
}): Promise<void> {
	const endpoint = (()=>{
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL("/api/auth/login", import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return "/api/auth/login";
	})()

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

async function register(body: {
	username: string;
	email: string;
	password: string;
}): Promise<void> {
	const endpoint = (()=>{
		if (import.meta.dev) {
			// @ts-expect-error env does exist
			return new URL("/api/auth/register", import.meta.env.VITE_SERVER_HOSTNAME);
		}

		return "/api/auth/register";
	})()

	const res = await fetch(endpoint, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		credentials: import.meta.dev ? "include" : "same-origin",
		body: JSON.stringify(body),
	});

	if (!res.ok) {
		throw new Error(`[${res.statusText}] ${await res.text()}`.trim());
	}
}

export default { login, register };
