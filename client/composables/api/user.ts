import { useMutation } from '../use-mutation'
import {
	BOOKMARK_PATH,
	FAVORITE_PATH,
	NewYomuyumeRequest,
	USER_MODIFY_PATH,
	USER_PROGRESS_PATH,
	USER_SENSITIVE_PATH,
	WHOAMI_PATH
} from './common'
import { BookmarkedTitlesQuery, FavoriteTitlesQuery } from './content'
import { clearNuxtData, useFetch } from 'nuxt/app'

export interface SensitiveRequest {
	password: string
	mode: 'ask' | 'confirm'
	purpose: 'delete_account' | 'change_password' | 'verify_email'
	code?: string
	payload?: string
}

export function useUserSensitiveAction() {
	return useMutation({
		async mutationFn(body: SensitiveRequest) {
			return NewYomuyumeRequest(USER_SENSITIVE_PATH, {
				method: 'POST',
				body: JSON.stringify(body)
			})
		}
	})
}

export function useUserFavorite(method: 'PUT' | 'DELETE') {
	return useMutation({
		async mutationFn(titleId: string) {
			return NewYomuyumeRequest(FAVORITE_PATH(titleId), { method })
		},
		onSuccess() {
			clearNuxtData(`search-${JSON.stringify(FavoriteTitlesQuery)}`)
		}
	})
}

export function useUserBookmark(method: 'PUT' | 'DELETE') {
	return useMutation({
		async mutationFn(titleId: string) {
			return NewYomuyumeRequest(BOOKMARK_PATH(titleId), { method })
		},
		onSuccess() {
			clearNuxtData(`search-${JSON.stringify(BookmarkedTitlesQuery)}`)
		}
	})
}

export function useUserModifyInfo() {
	return useMutation({
		async mutationFn(body: { username?: string; email?: string }) {
			return NewYomuyumeRequest(USER_MODIFY_PATH, {
				method: 'POST',
				body: JSON.stringify(body)
			})
		}
	})
}

export function useUserSetProgress() {
	return useMutation({
		async mutationFn(query: {
			title_id: string
			chapter_id: string
			page_id: string
			percent: number
		}) {
			return NewYomuyumeRequest(
				USER_PROGRESS_PATH,
				{
					method: 'PUT'
				},
				query
			)
		}
	})
}

export interface WhoAmIResponseBody {
	email: string
	ip_address: string
	profile_picture?: string
	updated_at?: string
	user_id: string
	username: string
	verified_at?: string
}

export function useUserWhoAmI() {
	return useFetch<WhoAmIResponseBody>(WHOAMI_PATH, {
		credentials: 'same-origin',
		key: 'whoami',
		immediate: false
	})
}
