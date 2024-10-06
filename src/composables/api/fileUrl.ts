/**
 * Get the url of the cover file for a title.
 *
 * @param id the id of the title
 */
function cover(id: string): string {
	if (import.meta.dev) {
		// @ts-expect-error env does exist
		return new URL(`/api/file/cover/${id}`, import.meta.env.VITE_SERVER_HOSTNAME).toString();
	}

	return `/api/file/cover/${id}`;
}

/**
 * Get the url of the page file for a title.
 *
 * @param id the id of the page
 */
function page(id: string): string {
	if (import.meta.dev) {
		// @ts-expect-error env does exist
		return new URL(`/api/file/page/${id}`, import.meta.env.VITE_SERVER_HOSTNAME).toString();
	}

	return `/api/file/page/${id}`;
}

export default { cover, page };
