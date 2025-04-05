/* eslint-disable @typescript-eslint/explicit-module-boundary-types, @typescript-eslint/explicit-function-return-type */

import { useMutation, useQuery } from "@tanstack/vue-query";

import { LIVE_CONFIG_PATH, NewYomuyumeRequest } from "./common";

export function useAdminSetLiveConfig() {
	return useMutation({
		async mutationFn(body: {
			komga_oneshot_support?: boolean;
			komga_recycle_support?: boolean;
			nomedia_support?: boolean;
			rescan_enabled?: boolean;
			rescan_interval_in_minutes?: number;
		}) {
			return NewYomuyumeRequest(LIVE_CONFIG_PATH, {
				method: "POST",
				body: JSON.stringify(body),
			});
		},
	});
}

export function useAdminGetLiveConfig() {
	return useQuery({
		queryKey: ["live_config"],
		async queryFn({ signal }) {
			return NewYomuyumeRequest<{
				nomedia_support: boolean;
				komga_oneshot_support: boolean;
				komga_recycle_support: boolean;
				rescan_enabled: boolean;
				rescan_interval_in_minutes: number;
			}>(LIVE_CONFIG_PATH, { signal });
		},
	});
}
