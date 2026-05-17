CREATE TABLE upload_chunks (
    upload_id TEXT NOT NULL,
    chunk_id TEXT NOT NULL,
    offset_bytes BIGINT NOT NULL,
    attempt INTEGER NOT NULL,
    checksum TEXT NOT NULL,
    stored_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (upload_id, chunk_id, attempt)
);

CREATE INDEX upload_chunks_offset_idx
    ON upload_chunks (upload_id, offset_bytes);
