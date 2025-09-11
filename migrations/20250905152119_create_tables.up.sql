PRAGMA journal_mode = WAL;

CREATE TABLE
    categories (
        id TEXT PRIMARY KEY NOT NULL,
        path TEXT NOT NULL UNIQUE,
        parent_id TEXT,
        name TEXT NOT NULL,
        description TEXT,
        FOREIGN KEY (parent_id) REFERENCES categories (id) ON DELETE SET NULL
    );

CREATE TABLE
    titles (
        id TEXT PRIMARY KEY NOT NULL,
        category_id TEXT,
        path TEXT NOT NULL UNIQUE,
        last_modified TIMESTAMP,
        comic_info BLOB,
        FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE SET NULL
    );

CREATE TABLE
    chapters (
        id TEXT PRIMARY KEY NOT NULL,
        title_id TEXT NOT NULL,
        path TEXT NOT NULL,
        number INTEGER,
        page_count INTEGER NOT NULL DEFAULT 0,
        last_modified TIMESTAMP,
        FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE ON UPDATE CASCADE,
        UNIQUE (title_id, path)
    );

CREATE TABLE
    pages (
        id TEXT PRIMARY KEY NOT NULL,
        chapter_id TEXT NOT NULL,
        page_number INTEGER NOT NULL,
        path TEXT NOT NULL,
        width INTEGER,
        height INTEGER,
        avg_hex_color TEXT,
        size TEXT,
        last_modified TIMESTAMP,
        UNIQUE (chapter_id, path),
        UNIQUE (chapter_id, page_number),
        FOREIGN KEY (chapter_id) REFERENCES chapters (id) ON DELETE CASCADE ON UPDATE CASCADE
    );

CREATE TABLE
    title_covers (
        title_id TEXT PRIMARY KEY NOT NULL,
        page_id TEXT NOT NULL,
        UNIQUE (title_id, page_id),
        FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
        FOREIGN KEY (page_id) REFERENCES pages (id) ON DELETE CASCADE
    );

CREATE TABLE
    chapter_covers (
        chapter_id TEXT PRIMARY KEY NOT NULL,
        page_id TEXT NOT NULL,
        UNIQUE (chapter_id, page_id),
        FOREIGN KEY (chapter_id) REFERENCES chapters (id) ON DELETE CASCADE,
        FOREIGN KEY (page_id) REFERENCES pages (id) ON DELETE CASCADE
    );

CREATE TABLE
    users (
        id TEXT PRIMARY KEY NOT NULL,
        username TEXT NOT NULL UNIQUE,
        email TEXT NOT NULL UNIQUE,
        password_hash TEXT NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        verified_at TIMESTAMP
    );

CREATE TABLE
    sessions (
        secret TEXT PRIMARY KEY NOT NULL,
        user_id TEXT NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
    );

CREATE TABLE
    collections (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT NOT NULL,
        user_id TEXT NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
    );

CREATE TABLE
    collection_titles (
        collection_id TEXT NOT NULL,
        title_id TEXT NOT NULL,
        PRIMARY KEY (collection_id, title_id),
        FOREIGN KEY (collection_id) REFERENCES collections (id) ON DELETE CASCADE,
        FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE
    );

CREATE TABLE
    reading_progress (
        user_id TEXT NOT NULL,
        title_id TEXT NOT NULL,
        chapter_id TEXT NOT NULL,
        page_number INTEGER NOT NULL,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        PRIMARY KEY (user_id, title_id),
        FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
        FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
        FOREIGN KEY (chapter_id) REFERENCES chapters (id) ON DELETE CASCADE
    );

CREATE TABLE
    forgot_password_requests (
        user_id TEXT PRIMARY KEY NOT NULL UNIQUE,
        code TEXT NOT NULL,
        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
    );
