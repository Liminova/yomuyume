CREATE TABLE IF NOT EXISTS categories (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,

    cover_path TEXT,
    cover_blurhash TEXT,
    cover_width INTEGER,
    cover_height INTEGER
);

CREATE TABLE IF NOT EXISTS titles (
    id BIGINT PRIMARY KEY,
    title TEXT,

    category_id BIGINT,
    author TEXT,
    description TEXT,
    release DATE,
    path TEXT NOT NULL,

    cover_path TEXT,
    cover_blurhash TEXT,
    cover_width INTEGER,
    cover_height INTEGER,

    date_updated TIMESTAMP WITH TIME ZONE,

    FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE SET NULL,
    CONSTRAINT "uc-titles-path" UNIQUE (path)
);

CREATE TABLE IF NOT EXISTS pages (
    id TEXT PRIMARY KEY,
    title_id TEXT NOT NULL,
    path TEXT NOT NULL,
    description TEXT,

    FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
    CONSTRAINT "uc-pages-title_id-path" UNIQUE (title_id, path)
);

CREATE TABLE IF NOT EXISTS tags (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL,

    CONSTRAINT "uc-tags-name" UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS titles_tags (
    title_id BIGINT NOT NULL,
    tag_id BIGINT NOT NULL,

    FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE,
    CONSTRAINT "uc-titles_tags-title_id-tag_id" UNIQUE (title_id, tag_id)
);

CREATE TABLE IF NOT EXISTS users (
    id BIGINT PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    profile_picture TEXT,
    ip_address TEXT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE,
    verified_at TIMESTAMP WITH TIME ZONE
);

CREATE TABLE IF NOT EXISTS bookmarks (
    id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    title_id BIGINT NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
    CONSTRAINT "uc-bookmarks-user_id-title_id" UNIQUE (user_id, title_id)
);

CREATE TABLE IF NOT EXISTS favorites (
    id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    title_id BIGINT NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
    CONSTRAINT "uc-favorites-user_id-title_id" UNIQUE (user_id, title_id)
);

CREATE TABLE IF NOT EXISTS progresses (
    id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    title_id BIGINT NOT NULL,
    last_read_at TIMESTAMP WITH TIME ZONE,
    page INTEGER NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE,
    CONSTRAINT "uc-progresses-user_id-title_id" UNIQUE (user_id, title_id)
);

DO $$ BEGIN
    CREATE TYPE temp_codes_purpose AS ENUM ('delete_account', 'reset_password', 'validate_email');
EXCEPTION
    WHEN duplicate_object THEN RAISE NOTICE 'type already exists, skipping';
END $$;
CREATE TABLE IF NOT EXISTS temp_codes (
    id SERIAL PRIMARY KEY,
    purpose temp_codes_purpose NOT NULL,
    user_id BIGINT NOT NULL,
    code TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT "uc-temp-codes-purpose-user_id" UNIQUE (purpose, user_id)
);

CREATE TABLE IF NOT EXISTS session_tokens (
    session_secret TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_used_at TIMESTAMP WITH TIME ZONE,
    user_agent TEXT,
    ip_address TEXT NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);