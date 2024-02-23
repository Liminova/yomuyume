export default function newRoute(path: string): string {
	return new URL(path, globalStore.instanceAddr).toString();
}
