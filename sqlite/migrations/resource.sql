CREATE TABLE IF NOT EXISTS resource (
    id          TEXT    PRIMARY KEY,
    name        TEXT    NOT NULL,
    extension   TEXT    NOT NULL,
    path        TEXT    NOT NULL,
    size        INTEGER NOT NULL,
    mime_type   TEXT    NOT NULL,
    kind        TEXT    NOT NULL,
    sha256      TEXT    NOT NULL,
    created_at  INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sha256_size_created_at ON resource (sha256, size, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_path ON resource (path);
