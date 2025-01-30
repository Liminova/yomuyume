import { openDB } from "idb";

export interface JpegXLInDB {
	id: string;
	data: Uint8Array;
}
export interface BlurHashInDB {
	id: string;
	data: Uint8Array;
}

export enum StoreName {
	// eslint-disable-next-line no-unused-vars
	BLURHASH = "blurhash",
	// eslint-disable-next-line no-unused-vars
	JPEGXL = "jpegxl",
}

export const db = await openDB("yomuyume", 20250128, {
	upgrade(db, oldVersion, newVersion) {
		// rm if old exists & there's a new version
		if (newVersion !== null
			&& db.objectStoreNames.contains(StoreName.BLURHASH)
			&& db.objectStoreNames.contains(StoreName.JPEGXL)
			&& oldVersion !== newVersion) {
			db.deleteObjectStore(StoreName.BLURHASH);
			db.deleteObjectStore(StoreName.JPEGXL);
		}

		// create if not exists
		if (!db.objectStoreNames.contains(StoreName.BLURHASH)) {
			db.createObjectStore(StoreName.BLURHASH, {
				keyPath: "id",
				autoIncrement: false,
			});
		}
		if (!db.objectStoreNames.contains(StoreName.JPEGXL)) {
			db.createObjectStore(StoreName.JPEGXL, {
				keyPath: "id",
				autoIncrement: false,
			});
		}
	},
});

export async function getPageInDB(
	id: string,
	type: StoreName.JPEGXL | StoreName.BLURHASH,
): Promise<JpegXLInDB | BlurHashInDB | undefined> {
	const tx = db.transaction(type, "readonly");
	const store = tx.store;
	const page = await store.get(id) as JpegXLInDB | BlurHashInDB | undefined;
	await tx.done;
	return page;
}

export async function setPageInDB(
	page: JpegXLInDB | BlurHashInDB,
	type: StoreName.JPEGXL | StoreName.BLURHASH,
): Promise<void> {
	const tx = db.transaction(type, "readwrite");
	const store = tx.store;
	await store.put(page);
	await tx.done;
}
