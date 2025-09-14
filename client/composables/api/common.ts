import { StatusCode } from '~/lib/status-code'

// START: API paths - DO NOT MODIFY THIS LINE
export const SESSION_SECRET_COOKIE_NAME = 'session-secret'
export const LOGOUT_PATH = '/api/auth/logout'
export const LOGIN_PATH = '/api/auth/login'
export const REGISTER_PATH = '/api/auth/register'
export const FORGOT_PATH = '/api/auth/forgot'
export const GET_NAVIGATION_FEED_PATH = (
	seriesOrCategory: string,
	id: string
) => `/api/opds/navigation/${seriesOrCategory}/${id}`
export const GET_ACQUISITION_FEED_PATH = (titleId: string) =>
	`/api/opds/acquisition/${titleId}`
export const GET_PAGE_FILE_PATH = (
	titleId: string,
	chapterId: string,
	pageNumber: string
) => `/api/file/title/${titleId}/${chapterId}/${pageNumber}`
export const GET_COVER_FILE_PATH = (titleOrChapterId: string) =>
	`/api/file/cover/${titleOrChapterId}`
export const GET_WHOAMI_PATH = '/api/user/whoami'
export const POST_USER_MODIFY_PATH = '/api/user/modify'
export const PUT_READ_PROGRESS_PATH = '/api/user/progress'
export const PUT_COLLECTION_PATH = (name: string) =>
	`/api/user/collection/new/${name}`
export const TITLE_IN_COLLECTION_PATH = (
	collectionId: string,
	titleId: string
) => `/api/user/collection/${collectionId}/${titleId}`
export const DELETE_COLLECTION_PATH = (collectionId: string) =>
	`/api/user/collection/delete/${collectionId}`
export const GET_STATUS_PATH = '/api/status'
// END: API paths - DO NOT MODIFY THIS LINE

export async function NewYomuyumeRequest<BodyType = void>(
	path: string,
	init?: RequestInit,
	queryParams?: Record<
		string,
		string | undefined | number | number[] | string[] | boolean
	>
): Promise<BodyType> {
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
				case value === undefined: {
					break
				}
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
					for (const item of value) {
						searchParams.append(key, item.toString())
					}
					break
				}
				default: {
					throw new Error('Unknown query param type')
				}
			}
		}
		path_ += searchParams.toString()
	}

	const resp = await fetch(path_, { ...init, headers })

	if (resp.status >= StatusCode.CLIENT_ERROR_RESPONSE_START) {
		const errorText = await resp.text()
		throw new Error(`[${resp.status}] ${errorText || resp.statusText}`)
	}

	// No content or empty body, return null
	if (
		resp.status === StatusCode.NO_CONTENT ||
		resp.headers.get('content-length') === '0'
	) {
		return null as unknown as BodyType
	}

	// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
	const respBody = await resp.json()

	return respBody as BodyType
}
