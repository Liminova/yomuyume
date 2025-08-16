// START: API paths - DO NOT MODIFY THIS LINE
export const LOGOUT_PATH = '/api/auth/logout'
export const LOGIN_PATH = '/api/auth/login'
export const REGISTER_PATH = '/api/auth/register'
export const FORGOT_PATH = '/api/auth/forgot'
export const GET_CATEGORIES_PATH = '/api/content/categories'
export const GET_TITLE_PATH = (titleId: string) =>
	`/api/content/title/${titleId}`
export const GET_PAGES_PATH = (chapterId: string) =>
	`/api/content/pages/${chapterId}`
export const GET_TAGS_PATH = '/api/content/tags'
export const SEARCH_PATH = '/api/content/search'
export const GET_PAGE_FILE_PATH = (pageId: string) => `/api/file/page/${pageId}`
export const GET_COVER_FILE_PATH = (titleId: string) =>
	`/api/file/cover/${titleId}`
export const FAVORITE_PATH = (titleId: string) =>
	`/api/user/favorite/${titleId}`
export const BOOKMARK_PATH = (titleId: string) =>
	`/api/user/bookmark/${titleId}`
export const WHOAMI_PATH = '/api/user/whoami'
export const USER_MODIFY_PATH = '/api/user/modify'
export const USER_SENSITIVE_PATH = '/api/user/sensitive'
export const USER_PROGRESS_PATH = '/api/user/progress'
export const GET_SCANNING_PROGRESS_PATH = '/api/admin/scanning_progress'
export const LIVE_CONFIG_PATH = '/api/admin/live_config'
export const GET_STATUS_PATH = '/api/status'
// END: API paths - DO NOT MODIFY THIS LINE

export async function NewYomuyumeRequest<T = void>(
	path: string,
	init?: RequestInit,
	queryParams?: Record<
		string,
		string | undefined | number | number[] | string[] | boolean
	>
): Promise<T> {
	const headers = new Headers()
	headers.set('Content-Type', 'application/json')

	let path_ = path
	if (queryParams) {
		if (!path_.endsWith('?')) {
			path_ += '?'
		}
		const searchParams = new URLSearchParams()
		for (const [key, value] of Object.entries(queryParams)) {
			// eslint-disable-next-line @typescript-eslint/switch-exhaustiveness-check
			switch (true) {
				case value === undefined:
					break
				case typeof value === 'string': {
					searchParams.append(key, value)
					break
				}
				case typeof value === 'number': {
					searchParams.append(key, value.toString())
					break
				}
				case typeof value === 'boolean': {
					searchParams.append(key, value.toString())
					break
				}
				case Array.isArray(value): {
					for (const v of value) {
						searchParams.append(key, v.toString())
					}
					break
				}
				default:
					throw new Error('Unknown query param type')
			}
		}
		path_ += searchParams.toString()
	}

	const resp = await fetch(path_, { ...init, headers })

	if (resp.status >= 400) {
		const errorText = await resp.text()
		throw new Error(`[${resp.status}] ${errorText || resp.statusText}`)
	}

	// No content or empty body, return null
	if (resp.status === 204 || resp.headers.get('content-length') === '0') {
		return null as unknown as T
	}

	// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
	const respBody = await resp.json()

	return respBody as T
}
