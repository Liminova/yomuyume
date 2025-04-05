import { NewYomuyumeRequest, WHOAMI_PATH } from "~/composables/api/common";

export async function isLoggedIn(): Promise<boolean> {
	return new Promise<boolean>(resolve => NewYomuyumeRequest(WHOAMI_PATH)
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
}
