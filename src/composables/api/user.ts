import newRoute from "./newRoute";
import type { GenericSrvResponse } from "../types";
import { GenericResponseBody } from "~/composables/bridge";

async function favorite(
	titleId: string,
	action: "DELETE" | "PUT"
): Promise<{ message?: string; ok?: boolean }> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/user/favorite/${titleId}`), {
			method: action,
			headers: {
				Authorization: `Bearer ${globalStore.token}`,
				Accept: "bitcode",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	const data = GenericResponseBody.from_bitcode(buffer);

	return {
		message: data.message,
		ok: res.ok,
	};
}

async function bookmark(
	titleId: string,
	action: "DELETE" | "PUT"
): Promise<{ message?: string; ok?: boolean }> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/user/bookmark/${titleId}`), {
			method: action,
			headers: {
				Authorization: `Bearer ${globalStore.token}`,
				Accept: "bitcode",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	const data = GenericResponseBody.from_bitcode(buffer);

	return {
		message: data.message,
		ok: res.ok,
	};
}

async function progress(
	titleId: string,
	page: number
): Promise<{ message?: string; ok?: boolean }> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/user/progress/${titleId}/${page}`), {
			method: "PUT",
			headers: {
				Authorization: `Bearer ${globalStore.token}`,
				Accept: "bitcode",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	const data = GenericResponseBody.from_bitcode(buffer);

	return {
		message: data.message,
		ok: res.ok,
	};
}

async function resetPassword(email: string): Promise<{ message?: string; ok?: boolean }> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/auth/reset/${email}`), {
			method: "GET",
			headers: {
				"Content-Type": "application/json",
				Accept: "bitcode",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	const data = GenericResponseBody.from_bitcode(buffer);

	return {
		message: data.message,
		ok: res.ok,
	};
}

async function confirmReset(
	password: string,
	token: string
): Promise<{ message?: string; ok?: boolean }> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/auth/reset`), {
			method: "POST",
			headers: {
				Authorization: `Bearer ${token}`,
				"Content-Type": "application/json",
				Accept: "bitcode",
			},
			body: JSON.stringify({ password }),
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	const data = GenericResponseBody.from_bitcode(buffer);

	return {
		message: data.message,
		ok: res.ok,
	};
}

async function deleteAccount(email: string): Promise<{ message?: string; ok?: boolean }> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/auth/delete/${email}`), {
			method: "DELETE",
			headers: {
				"Content-Type": "application/json",
				Accept: "bitcode",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	const data = GenericResponseBody.from_bitcode(buffer);

	return {
		message: data.message,
		ok: res.ok,
	};
}

async function confirmDelete(token: string): Promise<GenericSrvResponse> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/auth/delete`), {
			method: "POST",
			headers: {
				Authorization: `Bearer ${token}`,
				"Content-Type": "application/json",
				Accept: "bitcode",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	const data = GenericResponseBody.from_bitcode(buffer);

	return {
		message: data.message,
		ok: res.ok,
	};
}

async function verifyAccount(): Promise<{ message?: string; ok?: boolean }> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/auth/verify`), {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				Authorization: `Bearer ${globalStore.token}`,
				Accept: "bitcode",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	const data = GenericResponseBody.from_bitcode(buffer);

	return {
		message: data.message,
		ok: res.ok,
	};
}

async function confirmVerification(token: string): Promise<{ message?: string; ok?: boolean }> {
	let res: Response;

	try {
		res = await fetch(newRoute(`/api/auth/verify`), {
			method: "POST",
			headers: {
				Authorization: `Bearer ${token}`,
				"Content-Type": "application/json",
				Accept: "bitcode",
			},
		});
	} catch (e) {
		return { message: (e as { message: string }).message, ok: false };
	}

	const buffer = new Uint8Array(await res.arrayBuffer());

	const data = GenericResponseBody.from_bitcode(buffer);

	return {
		message: data.message,
		ok: res.ok,
	};
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
