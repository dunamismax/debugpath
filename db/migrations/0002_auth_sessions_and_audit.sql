alter table users
add column password_hash text;

create table user_sessions (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null references users (id) on delete cascade,
  token_hash text not null unique check (char_length(token_hash) = 64),
  expires_at timestamptz not null,
  last_seen_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table audit_events (
  id uuid primary key default gen_random_uuid(),
  workspace_id uuid references workspaces (id) on delete cascade,
  actor_user_id uuid references users (id) on delete set null,
  action text not null,
  target_type text not null,
  target_id uuid,
  ip_address inet,
  user_agent text,
  metadata jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now()
);

create index user_sessions_user_expires_idx on user_sessions (user_id, expires_at desc);
create index audit_events_workspace_created_idx on audit_events (workspace_id, created_at desc);
create index audit_events_actor_created_idx on audit_events (actor_user_id, created_at desc);
create index audit_events_action_created_idx on audit_events (action, created_at desc);

create trigger user_sessions_set_updated_at
before update on user_sessions
for each row
execute function set_row_updated_at();
