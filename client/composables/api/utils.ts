import { useMutation } from '../use-mutation'
import {
	GET_SCANNING_PROGRESS_PATH,
	GET_STATUS_PATH,
	NewYomuyumeRequest
} from './common'
import { useFetch } from 'nuxt/app'

export interface ScanningProgressResponse {
	scanning_completed: boolean
	scanning_progress: number
}

export function useGetLibraryScanningProgress() {
	return useFetch<ScanningProgressResponse>(GET_SCANNING_PROGRESS_PATH, {
		credentials: 'same-origin',
		key: 'scanning_progress'
	})
}

export interface ServerStatusResponse {
	server_time: string
	version: string
}

export function useGetServerStatus() {
	return useFetch<ServerStatusResponse>(GET_STATUS_PATH, {
		credentials: 'same-origin',
		key: 'status'
	})
}

export function usePostServerStatus() {
	return useMutation({
		async mutationFn(body: { echo: string }) {
			return NewYomuyumeRequest<ServerStatusResponse>(GET_STATUS_PATH, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(body)
			})
		}
	})
}
