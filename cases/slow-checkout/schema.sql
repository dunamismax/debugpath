CREATE TABLE orders (
    id uuid PRIMARY KEY,
    account_id uuid NOT NULL,
    status text NOT NULL,
    created_at timestamptz NOT NULL,
    total_cents integer NOT NULL
);

CREATE INDEX orders_account_created_at_idx ON orders (account_id, created_at DESC);
CREATE INDEX orders_status_idx ON orders (status);
