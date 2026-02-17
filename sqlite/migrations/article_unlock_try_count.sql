CREATE TABLE IF NOT EXISTS article_unlock_try_count (
    ip              TEXT    NOT NULL,
    article_id      TEXT    NOT NULL,
    count           INTEGER NOT NULL,
    created_at      INTEGER NOT NULL,
    expires_at      INTEGER NOT NULL,
    PRIMARY KEY (ip, article_id)
);

CREATE INDEX IF NOT EXISTS idx_expires_at ON article_unlock_try_count (expires_at);
