create table accounts (
    id bigint primary key,
    plan text not null,
    created_at timestamptz not null
);

create table account_events (
    id bigint primary key,
    account_id bigint not null references accounts(id),
    event_type text not null,
    occurred_at timestamptz not null,
    metadata jsonb not null default '{}'
);

create index account_events_account_id_occurred_at_idx
    on account_events (account_id, occurred_at desc);
