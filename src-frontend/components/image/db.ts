export interface JpegXLInDB {
	id: string;
	data: Uint8Array;
}

export interface BlurHashInDB {
	id: string;
	data: Uint8Array;
}

export enum StoreName {

	BLURHASH = "blurhash",

	JPEGXL = "jpegxl",
}

const DB_NAME = "yomuyume";
const DB_VERSION = 20250128;

let db: IDBDatabase | null = null;

// Initialize the database connection
function initDB(): Promise<IDBDatabase> {
	return new Promise((resolve, reject) => {
		if (db) {
			resolve(db);
			return;
		}

		const request = indexedDB.open(DB_NAME, DB_VERSION);

		request.onerror = (event): void => {
			// @ts-expect-error idk
			reject(new Error(`Failed to open database: ${event.target?.error}`));
		};

		request.onsuccess = (event): void => {
			db = (event.target as IDBOpenDBRequest).result;
			resolve(db);
		};

		request.onupgradeneeded = (event): void => {
			const db = (event.target as IDBOpenDBRequest).result;
			const oldVersion = event.oldVersion;
			const newVersion = event.newVersion;

			// Remove existing stores if version changed
			if (newVersion !== null
				&& db.objectStoreNames.contains(StoreName.BLURHASH)
				&& db.objectStoreNames.contains(StoreName.JPEGXL)
				&& oldVersion !== newVersion) {
				db.deleteObjectStore(StoreName.BLURHASH);
				db.deleteObjectStore(StoreName.JPEGXL);
			}

			// Create stores if they don't exist
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
		};
	});
}

// Initialize the database connection when the module is imported
const dbInitPromise = initDB();

export async function getPageInDB(
	id: string,
	type: StoreName.JPEGXL | StoreName.BLURHASH,
): Promise<JpegXLInDB | BlurHashInDB | undefined> {
	await dbInitPromise;

	return new Promise((resolve, reject) => {
		if (!db) {
			reject(new Error("Database not initialized"));
			return;
		}

		const transaction = db.transaction(type, "readonly");
		const store = transaction.objectStore(type);
		const request = store.get(id);

		request.onerror = (): void => {
			reject(new Error(`Failed to get item with id ${id} from ${type}`));
		};

		request.onsuccess = (): void => {
			resolve(request.result as JpegXLInDB | BlurHashInDB | undefined);
		};
	});
}

export async function setPageInDB(
	{ page, type }: { page: JpegXLInDB; type: StoreName.JPEGXL } | { page: BlurHashInDB; type: StoreName.BLURHASH },
): Promise<void> {
	await dbInitPromise;

	return new Promise((resolve, reject): void => {
		if (!db) {
			reject(new Error("Database not initialized"));
			return;
		}

		const transaction = db.transaction(type, "readwrite");
		const store = transaction.objectStore(type);
		const request = store.put(page);

		request.onerror = (): void => {
			reject(new Error(`Failed to store item with id ${page.id} in ${type}`));
		};

		request.onsuccess = (): void => {
			resolve();
		};
	});
}
