export function toCoverApiEndpoint(titleId: string): string {
	return `/api/file/cover/${titleId}`;
}

export function toPageApiEndpoint(pageId: string): string {
	return `/api/file/page/${pageId}`;
}
