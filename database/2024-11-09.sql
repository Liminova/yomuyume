DROP TABLE IF EXISTS categories CASCADE;
CREATE TABLE categories (
    id BIGINT PRIMARY KEY,
    name TEXT,
    description TEXT,
    path TEXT NOT NULL,
    cover_path TEXT,
    cover_blurhash TEXT,
    cover_width INTEGER,
    cover_height INTEGER,
    CONSTRAINT "uc-categories-path" UNIQUE (path)
);
DROP TABLE IF EXISTS titles CASCADE;
CREATE TABLE IF NOT EXISTS titles (
    id BIGINT PRIMARY KEY,
    title TEXT,
    category_id BIGINT,
    author TEXT,
    description TEXT,
    release DATE,
    path TEXT NOT NULL,
    is_dir BOOLEAN NOT NULL,
    cover_path TEXT,
    cover_blurhash TEXT,
    cover_width INTEGER,
    cover_height INTEGER,
    date_updated TIMESTAMP WITH TIME ZONE,
    FOREIGN KEY (category_id) REFERENCES categories (id) ON DELETE
    SET NULL ON UPDATE CASCADE,
        CONSTRAINT "uc-titles-path" UNIQUE (path)
);
DROP TABLE IF EXISTS chapters CASCADE;
CREATE TABLE IF NOT EXISTS chapters (
    id BIGINT PRIMARY KEY,
    title_id BIGINT NOT NULL,
    path TEXT NOT NULL,
    number INTEGER NOT NULL,
    description TEXT,
    is_dir BOOLEAN NOT NULL,
    FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "uc-chapters-title_id-path" UNIQUE (title_id, path)
);
DROP TABLE IF EXISTS pages CASCADE;
CREATE TABLE IF NOT EXISTS pages (
    id BIGINT PRIMARY KEY,
    chapter_id BIGINT NOT NULL,
    filesize BIGINT,
    path TEXT NOT NULL,
    description TEXT,
    FOREIGN KEY (chapter_id) REFERENCES chapters (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "uc-pages-chapter_id-path" UNIQUE (chapter_id, path)
);
DROP TABLE IF EXISTS tags CASCADE;
CREATE TABLE IF NOT EXISTS tags (
    id BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    CONSTRAINT "uc-tags-name" UNIQUE (name)
);
DROP TABLE IF EXISTS titles_tags CASCADE;
CREATE TABLE IF NOT EXISTS titles_tags (
    title_id BIGINT NOT NULL,
    tag_id BIGINT NOT NULL,
    FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "uc-titles_tags-title_id-tag_id" UNIQUE (title_id, tag_id)
);
DROP TABLE IF EXISTS users CASCADE;
CREATE TABLE IF NOT EXISTS users (
    id BIGINT PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    profile_picture TEXT,
    ip_address TEXT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE,
    verified_at TIMESTAMP WITH TIME ZONE,
    CONSTRAINT "uc-users-email" UNIQUE (email)
);
DROP TABLE IF EXISTS bookmarks CASCADE;
CREATE TABLE IF NOT EXISTS bookmarks (
    id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    title_id BIGINT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "uc-bookmarks-user_id-title_id" UNIQUE (user_id, title_id)
);
DROP TABLE IF EXISTS favorites CASCADE;
CREATE TABLE IF NOT EXISTS favorites (
    id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    title_id BIGINT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "uc-favorites-user_id-title_id" UNIQUE (user_id, title_id)
);
DROP TABLE IF EXISTS progresses CASCADE;
CREATE TABLE IF NOT EXISTS progresses (
    id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    title_id BIGINT NOT NULL,
    page_id BIGINT NOT NULL,
    percent SMALLINT NOT NULL,
    last_read_at TIMESTAMP WITH TIME ZONE,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (title_id) REFERENCES titles (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "uc-progresses-user_id-title_id" UNIQUE (user_id, title_id)
);
DROP TYPE IF EXISTS temp_codes_purpose CASCADE;
DO $$ BEGIN CREATE TYPE temp_codes_purpose AS ENUM (
    'delete_account',
    'reset_password',
    'validate_email'
);
EXCEPTION
WHEN duplicate_object THEN RAISE NOTICE 'type already exists, skipping';
END $$;
DROP TABLE IF EXISTS temp_codes;
CREATE TABLE IF NOT EXISTS temp_codes (
    id BIGINT PRIMARY KEY,
    secret TEXT NOT NULL,
    purpose temp_codes_purpose NOT NULL,
    user_id BIGINT NOT NULL,
    code TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "uc-temp-codes-purpose-user_id" UNIQUE (purpose, user_id)
);
DROP TABLE IF EXISTS session_tokens CASCADE;
CREATE TABLE IF NOT EXISTS session_tokens (
    id BIGINT PRIMARY KEY,
    session_secret TEXT NOT NULL,
    user_id BIGINT NOT NULL,
    last_used_at TIMESTAMP WITH TIME ZONE,
    user_agent TEXT,
    ip_address TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE
);
DROP TABLE IF EXISTS live_config CASCADE;
CREATE TABLE IF NOT EXISTS live_config (
    id TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    last_updated_at TIMESTAMP WITH TIME ZONE
);