/* eslint-disable @typescript-eslint/explicit-function-return-type, @typescript-eslint/explicit-module-boundary-types */

export const LOGOUT_PATH = "/api/auth/logout";
export const LOGIN_PATH = "/api/auth/login";
export const REGISTER_PATH = "/api/auth/register";

export const SESSION_ID_COOKIE_NAME = "session-ID";
export const SESSION_SECRET_COOKIE_NAME = "session-secret";

export const GET_CATEGORIES_PATH = "/api/content/categories";
export const GET_CHAPTER_PATH = (chaperID: string) => `/api/content/chapter/${chaperID}`;
export const GET_ONESHOT_PATH = (titleID: string) => `/api/content/oneshot/${titleID}`;
export const GET_SERIES_PATH = (titleID: string) => `/api/content/series/${titleID}`;
export const GET_TAGS_PATH = "/api/content/tags";
export const SEARCH_PATH = "/api/content/search";

export const GET_PAGE_PATH = (isSeries: boolean, pageID: string) => `/api/file/page/${isSeries}/${pageID}`;
export const GET_COVER_PATH = (titleID: string) => `/api/file/cover/${titleID}`;

export const FAVORITE_PATH = (titleID: string) => `/api/user/favorite/${titleID}`;
export const BOOKMARK_PATH = (titleID: string) => `/api/user/bookmark/${titleID}`;
export const WHOAMI_PATH = "/api/user/whoami";
export const USER_MODIFY_PATH = "/api/user/modify";
export const USER_SENSITIVE_PATH = "/api/user/sensitive";
export const USER_PROGRESS_PATH = (titleID: string, page: number) => `/api/user/progress/${titleID}/${page}`;

export const GET_SCANNING_PROGRESS_PATH = "/api/admin/scanning_progress";
export const LIVE_CONFIG_PATH = "/api/admin/live_config";

export const GET_STATUS_PATH = "/api/status";

export async function ResponseError(response: Response): Promise<Error> {
	const errMsg = await response.text();
	return new Error(`[${response.status}] ${errMsg === "" ? response.statusText : errMsg}`);
}
