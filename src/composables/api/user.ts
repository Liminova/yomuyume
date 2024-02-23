import newRoute from "./newRoute";
import type { GenericSrvResponse } from "../types";

async function favorite(titleId: string, action: "DELETE" | "PUT"): Promise<GenericSrvResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/user/favorite/${titleId}`), {
			method: action,
			headers: {
				Authorization: `Bearer ${globalStore.token}`,
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	try {
		const data = (await res.json()) as GenericSrvResponse;

		return { message: data.message, ok: res.ok };
	} catch {
		return { message: "Can't parse server response", ok: false };
	}
}

async function bookmark(titleId: string, action: "DELETE" | "PUT"): Promise<GenericSrvResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/user/bookmark/${titleId}`), {
			method: action,
			headers: {
				Authorization: `Bearer ${globalStore.token}`,
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	try {
		const data = (await res.json()) as GenericSrvResponse;

		return { message: data.message, ok: res.ok };
	} catch {
		return { message: "Can't parse server response", ok: false };
	}
}

async function progress(titleId: string, page: number): Promise<GenericSrvResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/user/progress/${titleId}/${page}`), {
			method: "PUT",
			headers: {
				Authorization: `Bearer ${globalStore.token}`,
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	try {
		const data = (await res.json()) as GenericSrvResponse;

		return { message: data.message, ok: res.ok };
	} catch {
		return { message: "Can't parse server response", ok: false };
	}
}

async function resetPassword(email: string): Promise<GenericSrvResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/auth/reset/${email}`), {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	try {
		const data = (await res.json()) as GenericSrvResponse;

		return { message: data.message, ok: res.ok };
	} catch {
		return { message: "Can't parse server response", ok: false };
	}
}

async function confirmReset(password: string, token: string): Promise<GenericSrvResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/auth/reset`), {
			method: "POST",
			headers: {
				Authorization: `Bearer ${token}`,
				"Content-Type": "application/json",
			},
			body: JSON.stringify({ password }),
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	try {
		const data = (await res.json()) as GenericSrvResponse;

		return { message: data.message, ok: res.ok };
	} catch {
		return { message: "Can't parse server response", ok: false };
	}
}

async function deleteAccount(email: string): Promise<GenericSrvResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/auth/delete/${email}`), {
			method: "DELETE",
			headers: {
				"Content-Type": "application/json",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	try {
		const data = (await res.json()) as GenericSrvResponse;

		return { message: data.message, ok: res.ok };
	} catch {
		return { message: "Can't parse server response", ok: false };
	}
}

async function confirmDelete(token: string): Promise<GenericSrvResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/auth/delete`), {
			method: "POST",
			headers: {
				Authorization: `Bearer ${token}`,
				"Content-Type": "application/json",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	try {
		const data = (await res.json()) as GenericSrvResponse;

		return { message: data.message, ok: res.ok };
	} catch {
		return { message: "Can't parse server response", ok: false };
	}
}

async function verifyAccount(): Promise<GenericSrvResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/auth/verify`), {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				Authorization: `Bearer ${globalStore.token}`,
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	try {
		const data = (await res.json()) as GenericSrvResponse;

		return { message: data.message, ok: res.ok };
	} catch {
		return { message: "Can't parse server response", ok: false };
	}
}

async function confirmVerification(token: string): Promise<GenericSrvResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/auth/verify`), {
			method: "POST",
			headers: {
				Authorization: `Bearer ${token}`,
				"Content-Type": "application/json",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	try {
		const data = (await res.json()) as GenericSrvResponse;

		return { message: data.message, ok: res.ok };
	} catch {
		return { message: "Can't parse server response", ok: false };
	}
}

export default {
	favorite,
	bookmark,
	progress,
	resetPassword,
	verifyAccount,
	confirmVerification,
	confirmReset,
	deleteAccount,
	confirmDelete,
};
