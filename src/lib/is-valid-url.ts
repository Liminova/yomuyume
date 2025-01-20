export default function isValidUrl(url: string): boolean {
	try {
		const _ = new URL(url);

		return true;
	} catch {
		return false;
	}
}
