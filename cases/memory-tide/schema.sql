CREATE TABLE uploads (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX uploads_account_created_idx
    ON uploads (account_id, created_at DESC);
