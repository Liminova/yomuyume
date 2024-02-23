import newRoute from "./newRoute";

type LoginServerResponse = {
	token?: string;
	message?: string;
};

type LoginFnResponse = LoginServerResponse;

async function login(body: { login: string; password: string }): Promise<LoginFnResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute("/api/auth/login"), {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify(body),
		});
	} catch (e) {
		return { message: (e as { message: string }).message };
	}

	try {
		const data = (await res.json()) as LoginServerResponse;

		return { token: res.ok ? data.token : undefined, message: data.message ?? "" };
	} catch {
		return { message: "Can't parse server response" };
	}
}

async function register(body: {
	username: string;
	email: string;
	password: string;
}): Promise<GenericSrvResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute("/api/auth/register"), {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify(body),
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	const message = "Can't parse server response";

	try {
		const data = (await res.json()) as GenericSrvResponse;

		return { message: data.message, ok: res.ok };
	} catch {
		return { message, ok: false };
	}
}

export default {
	login,
	register,
};
