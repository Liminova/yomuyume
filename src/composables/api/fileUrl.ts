/**
 * Get the url of the thumbnail file for a title.
 *
 * @param id the id of the title
 */
function thumbnail(id: string): string {
	return new URL(`/api/file/thumbnail/${id}`, globalStore.instanceAddr).toString();
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
	thumbnail,
	page,
};
