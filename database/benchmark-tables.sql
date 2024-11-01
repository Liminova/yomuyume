DROP TABLE IF EXISTS pages_benchmark CASCADE;

CREATE TABLE IF NOT EXISTS pages_benchmark (
    id VARCHAR PRIMARY KEY,
    title_id VARCHAR NOT NULL,
    path VARCHAR NOT NULL,
    description VARCHAR
);
CREATE UNIQUE INDEX IF NOT EXISTS "idx-pages_benchmark-title_id-path" ON pages_benchmark (title_id, path);
