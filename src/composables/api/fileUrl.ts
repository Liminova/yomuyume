/**
 * Get the url of the cover file for a title.
 *
 * @param id the id of the title
 */
function cover(id: string): string {
	return new URL(`/api/file/cover/${id}`, globalStore.instanceAddr).toString();
}

/**
 * Get the url of the page file for a title.
 *
 * @param id the id of the page
 */
function page(id: string): string {
	return new URL(`/api/file/page/${id}`, globalStore.instanceAddr).toString();
}

export default {
	cover,
	page,
};
