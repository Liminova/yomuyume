DROP TABLE IF EXISTS pages_benchmark CASCADE;

CREATE TABLE IF NOT EXISTS pages_benchmark (
    id TEXT PRIMARY KEY,
    title_id TEXT NOT NULL,
    path VARCHAR NOT NULL,
    description VARCHAR,

    CONSTRAINT "uc-pages_benchmark-title_id-path" UNIQUE (title_id, path)
);
