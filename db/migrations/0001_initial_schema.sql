create extension if not exists pgcrypto;
create extension if not exists citext;

create type workspace_role as enum ('owner', 'editor', 'viewer');
create type investigation_status as enum ('draft', 'active', 'resolved', 'archived');
create type investigation_severity as enum ('low', 'medium', 'high', 'critical');
create type artifact_kind as enum (
  'stack_trace',
  'structured_log',
  'har',
  'screenshot_metadata',
  'console_output',
  'environment_details',
  'repro_steps',
  'other'
);
create type artifact_ingest_status as enum ('pending', 'processing', 'parsed', 'failed');
create type ingestion_job_status as enum ('pending', 'running', 'succeeded', 'failed');
create type note_anchor_kind as enum ('investigation', 'artifact', 'timeline_event');

create function set_row_updated_at()
returns trigger
language plpgsql
as $$
begin
  new.updated_at = now();
  return new;
end;
$$;

create table users (
  id uuid primary key default gen_random_uuid(),
  email citext not null unique,
  display_name text,
  last_login_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table workspaces (
  id uuid primary key default gen_random_uuid(),
  slug text not null unique,
  name text not null,
  owner_user_id uuid not null references users (id) on delete restrict,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table workspace_memberships (
  workspace_id uuid not null references workspaces (id) on delete cascade,
  user_id uuid not null references users (id) on delete cascade,
  role workspace_role not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  primary key (workspace_id, user_id)
);

create table investigations (
  id uuid primary key default gen_random_uuid(),
  workspace_id uuid not null references workspaces (id) on delete cascade,
  created_by_user_id uuid not null references users (id) on delete restrict,
  slug text not null,
  title text not null,
  summary text,
  status investigation_status not null default 'draft',
  severity investigation_severity,
  archived_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  unique (workspace_id, slug)
);

create table artifacts (
  id uuid primary key default gen_random_uuid(),
  workspace_id uuid not null references workspaces (id) on delete cascade,
  investigation_id uuid not null references investigations (id) on delete cascade,
  uploaded_by_user_id uuid references users (id) on delete set null,
  kind artifact_kind not null,
  ingest_status artifact_ingest_status not null default 'pending',
  storage_bucket text not null,
  storage_key text not null unique,
  original_filename text,
  media_type text not null,
  byte_size bigint not null check (byte_size >= 0),
  sha256 text not null check (char_length(sha256) = 64),
  raw_metadata jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table notes (
  id uuid primary key default gen_random_uuid(),
  workspace_id uuid not null references workspaces (id) on delete cascade,
  investigation_id uuid not null references investigations (id) on delete cascade,
  author_user_id uuid not null references users (id) on delete restrict,
  anchor_kind note_anchor_kind not null default 'investigation',
  anchor_artifact_id uuid references artifacts (id) on delete cascade,
  anchor_event_key text,
  body_markdown text not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  constraint notes_anchor_shape check (
    (anchor_kind = 'investigation' and anchor_artifact_id is null and anchor_event_key is null)
    or (anchor_kind = 'artifact' and anchor_artifact_id is not null and anchor_event_key is null)
    or (anchor_kind = 'timeline_event' and anchor_event_key is not null)
  )
);

create table ingestion_jobs (
  id uuid primary key default gen_random_uuid(),
  workspace_id uuid not null references workspaces (id) on delete cascade,
  investigation_id uuid not null references investigations (id) on delete cascade,
  artifact_id uuid not null references artifacts (id) on delete cascade,
  status ingestion_job_status not null default 'pending',
  parser_version text not null,
  attempt_count integer not null default 1 check (attempt_count > 0),
  last_error text,
  started_at timestamptz,
  finished_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table bundles (
  id uuid primary key default gen_random_uuid(),
  workspace_id uuid not null references workspaces (id) on delete cascade,
  investigation_id uuid not null references investigations (id) on delete cascade,
  created_by_user_id uuid not null references users (id) on delete restrict,
  slug text not null,
  title text not null,
  summary text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  unique (investigation_id, slug)
);

create table bundle_artifacts (
  bundle_id uuid not null references bundles (id) on delete cascade,
  artifact_id uuid not null references artifacts (id) on delete cascade,
  created_at timestamptz not null default now(),
  primary key (bundle_id, artifact_id)
);

create table bundle_notes (
  bundle_id uuid not null references bundles (id) on delete cascade,
  note_id uuid not null references notes (id) on delete cascade,
  created_at timestamptz not null default now(),
  primary key (bundle_id, note_id)
);

create table bundle_share_links (
  id uuid primary key default gen_random_uuid(),
  bundle_id uuid not null references bundles (id) on delete cascade,
  created_by_user_id uuid not null references users (id) on delete restrict,
  token_hash text not null unique check (char_length(token_hash) = 64),
  expires_at timestamptz,
  revoked_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create index workspace_memberships_user_id_idx on workspace_memberships (user_id);
create index investigations_workspace_updated_idx on investigations (workspace_id, updated_at desc);
create index artifacts_investigation_created_idx on artifacts (investigation_id, created_at desc);
create index notes_investigation_created_idx on notes (investigation_id, created_at desc);
create index ingestion_jobs_artifact_created_idx on ingestion_jobs (artifact_id, created_at desc);
create index bundles_investigation_created_idx on bundles (investigation_id, created_at desc);
create index bundle_share_links_bundle_id_idx on bundle_share_links (bundle_id);

create trigger users_set_updated_at
before update on users
for each row
execute function set_row_updated_at();

create trigger workspaces_set_updated_at
before update on workspaces
for each row
execute function set_row_updated_at();

create trigger workspace_memberships_set_updated_at
before update on workspace_memberships
for each row
execute function set_row_updated_at();

create trigger investigations_set_updated_at
before update on investigations
for each row
execute function set_row_updated_at();

create trigger artifacts_set_updated_at
before update on artifacts
for each row
execute function set_row_updated_at();

create trigger notes_set_updated_at
before update on notes
for each row
execute function set_row_updated_at();

create trigger ingestion_jobs_set_updated_at
before update on ingestion_jobs
for each row
execute function set_row_updated_at();

create trigger bundles_set_updated_at
before update on bundles
for each row
execute function set_row_updated_at();

create trigger bundle_share_links_set_updated_at
before update on bundle_share_links
for each row
execute function set_row_updated_at();
