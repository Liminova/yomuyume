import { useQueryClient } from "@tanstack/vue-query";

import { NewYomuyumeRequest, WHOAMI_PATH } from "~/composables/api/common";

export async function isLoggedIn(): Promise<boolean> {
	const whoAmI = await useQueryClient().ensureQueryData({
		queryKey: ["whoami"],
		async queryFn({ signal }) {
			try {
				await NewYomuyumeRequest(WHOAMI_PATH, { signal });
			} catch (e) {
				if ((e as { message: string }).message.includes("[401]")) { return null; }
				throw e;
			}
		},
	});

	return whoAmI !== null;
}
