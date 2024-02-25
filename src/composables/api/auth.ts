import newRoute from "./newRoute";
import { GenericResponseBody, LoginResponseBody } from "~/composables/bridge";

async function login(body: {
	login: string;
	password: string;
}): Promise<{ token?: string; message?: string }> {
	let res: Response;

	try {
		res = await fetch(newRoute("/api/auth/login"), {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				Accept: "bitcode",
			},
			body: JSON.stringify(body),
		});
	} catch (e) {
		return { message: (e as { message: string }).message };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	if (res.ok) {
		const data = LoginResponseBody.from_bitcode(buffer);

		return { token: data?.token };
	}

	return { message: GenericResponseBody.from_bitcode(buffer).message };
}

async function register(body: {
	username: string;
	email: string;
	password: string;
}): Promise<{ ok: boolean; message: string }> {
	let res: Response;

	try {
		res = await fetch(newRoute("/api/auth/register"), {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				Accept: "bitcode",
			},
			body: JSON.stringify(body),
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	return {
		ok: res.ok,
		message: GenericResponseBody.from_bitcode(buffer).message,
	};
}

export default {
	login,
	register,
};
