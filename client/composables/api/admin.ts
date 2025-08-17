import { useMutation } from '../use-mutation'
import { LIVE_CONFIG_PATH } from './common'
import { useFetch } from 'nuxt/app'

export function useAdminSetLiveConfig() {
	return useMutation({
		mutationFn: (body: {
			komga_oneshot_support?: boolean
			komga_recycle_support?: boolean
			nomedia_support?: boolean
			rescan_enabled?: boolean
			rescan_interval_in_minutes?: number
		}) =>
			$fetch(LIVE_CONFIG_PATH, {
				method: 'POST',
				credentials: 'same-origin',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(body)
			})
	})
}

export function useAdminGetLiveConfig() {
	return useFetch<{
		nomedia_support: boolean
		komga_oneshot_support: boolean
		komga_recycle_support: boolean
		rescan_enabled: boolean
		rescan_interval_in_minutes: number
	}>(LIVE_CONFIG_PATH, {
		credentials: 'same-origin',
		headers: { 'Content-Type': 'application/json' },
		key: 'live_config'
	})
}
