import { type DatabaseExecutor, expectOne, maybeOne } from './client';

export type WorkspaceRoleRecord = 'owner' | 'editor' | 'viewer';
export type InvestigationStatusRecord = 'draft' | 'active' | 'resolved' | 'archived';
export type InvestigationSeverityRecord = 'low' | 'medium' | 'high' | 'critical' | null;
export type ArtifactKindRecord =
  | 'stack_trace'
  | 'structured_log'
  | 'har'
  | 'screenshot_metadata'
  | 'console_output'
  | 'environment_details'
  | 'repro_steps'
  | 'other';
export type ArtifactIngestStatusRecord = 'pending' | 'processing' | 'parsed' | 'failed';
export type IngestionJobStatusRecord = 'pending' | 'running' | 'succeeded' | 'failed';
export type NoteAnchorKindRecord = 'investigation' | 'artifact' | 'timeline_event';

export interface UserRecord {
  id: string;
  email: string;
  displayName: string | null;
  createdAt: string;
  updatedAt: string;
  lastLoginAt: string | null;
}

export interface WorkspaceRecord {
  id: string;
  slug: string;
  name: string;
  ownerUserId: string;
  createdAt: string;
  updatedAt: string;
}

export interface WorkspaceMembershipRecord {
  workspaceId: string;
  userId: string;
  role: WorkspaceRoleRecord;
  createdAt: string;
  updatedAt: string;
}

export interface InvestigationRecord {
  id: string;
  workspaceId: string;
  createdByUserId: string;
  slug: string;
  title: string;
  summary: string | null;
  status: InvestigationStatusRecord;
  severity: InvestigationSeverityRecord;
  archivedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface ArtifactRecord {
  id: string;
  workspaceId: string;
  investigationId: string;
  uploadedByUserId: string | null;
  kind: ArtifactKindRecord;
  ingestStatus: ArtifactIngestStatusRecord;
  storageBucket: string;
  storageKey: string;
  originalFilename: string | null;
  mediaType: string;
  byteSize: number;
  sha256: string;
  rawMetadata: Record<string, unknown>;
  createdAt: string;
  updatedAt: string;
}

export interface NoteRecord {
  id: string;
  workspaceId: string;
  investigationId: string;
  authorUserId: string;
  anchorKind: NoteAnchorKindRecord;
  anchorArtifactId: string | null;
  anchorEventKey: string | null;
  bodyMarkdown: string;
  createdAt: string;
  updatedAt: string;
}

export interface IngestionJobRecord {
  id: string;
  workspaceId: string;
  investigationId: string;
  artifactId: string;
  status: IngestionJobStatusRecord;
  parserVersion: string;
  attemptCount: number;
  lastError: string | null;
  startedAt: string | null;
  finishedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface BundleRecord {
  id: string;
  workspaceId: string;
  investigationId: string;
  createdByUserId: string;
  slug: string;
  title: string;
  summary: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface BundleShareLinkRecord {
  id: string;
  bundleId: string;
  createdByUserId: string;
  tokenHash: string;
  expiresAt: string | null;
  revokedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface UpsertUserInput {
  email: string;
  displayName?: string | null;
}

export interface UpsertWorkspaceInput {
  slug: string;
  name: string;
  ownerUserId: string;
}

export interface UpsertWorkspaceMembershipInput {
  workspaceId: string;
  userId: string;
  role: WorkspaceRoleRecord;
}

export interface UpsertInvestigationInput {
  workspaceId: string;
  createdByUserId: string;
  slug: string;
  title: string;
  summary?: string | null;
  status?: InvestigationStatusRecord;
  severity?: InvestigationSeverityRecord;
  archivedAt?: string | null;
}

export interface UpsertArtifactInput {
  workspaceId: string;
  investigationId: string;
  uploadedByUserId?: string | null;
  kind: ArtifactKindRecord;
  ingestStatus?: ArtifactIngestStatusRecord;
  storageBucket: string;
  storageKey: string;
  originalFilename?: string | null;
  mediaType: string;
  byteSize: number;
  sha256: string;
  rawMetadata?: Record<string, unknown>;
}

export interface CreateNoteInput {
  workspaceId: string;
  investigationId: string;
  authorUserId: string;
  bodyMarkdown: string;
  anchorKind?: NoteAnchorKindRecord;
  anchorArtifactId?: string | null;
  anchorEventKey?: string | null;
}

export interface CreateIngestionJobInput {
  workspaceId: string;
  investigationId: string;
  artifactId: string;
  status?: IngestionJobStatusRecord;
  parserVersion: string;
  attemptCount?: number;
  lastError?: string | null;
  startedAt?: string | null;
  finishedAt?: string | null;
}

export interface UpsertBundleInput {
  workspaceId: string;
  investigationId: string;
  createdByUserId: string;
  slug: string;
  title: string;
  summary?: string | null;
}

export interface UpsertBundleShareLinkInput {
  bundleId: string;
  createdByUserId: string;
  tokenHash: string;
  expiresAt?: string | null;
  revokedAt?: string | null;
}

export const upsertUser = async (db: DatabaseExecutor, input: UpsertUserInput) => {
  const rows = await db<UserRecord[]>`
    insert into users (email, display_name)
    values (${input.email}, ${input.displayName ?? null})
    on conflict (email)
    do update
      set display_name = coalesce(excluded.display_name, users.display_name),
          updated_at = now()
    returning
      id,
      email,
      display_name as "displayName",
      created_at as "createdAt",
      updated_at as "updatedAt",
      last_login_at as "lastLoginAt"
  `;

  return expectOne(rows, 'user upsert');
};

export const upsertWorkspace = async (db: DatabaseExecutor, input: UpsertWorkspaceInput) => {
  const rows = await db<WorkspaceRecord[]>`
    insert into workspaces (slug, name, owner_user_id)
    values (${input.slug}, ${input.name}, ${input.ownerUserId})
    on conflict (slug)
    do update
      set name = excluded.name,
          owner_user_id = excluded.owner_user_id,
          updated_at = now()
    returning
      id,
      slug,
      name,
      owner_user_id as "ownerUserId",
      created_at as "createdAt",
      updated_at as "updatedAt"
  `;

  return expectOne(rows, 'workspace upsert');
};

export const upsertWorkspaceMembership = async (
  db: DatabaseExecutor,
  input: UpsertWorkspaceMembershipInput
) => {
  const rows = await db<WorkspaceMembershipRecord[]>`
    insert into workspace_memberships (workspace_id, user_id, role)
    values (${input.workspaceId}, ${input.userId}, ${input.role})
    on conflict (workspace_id, user_id)
    do update
      set role = excluded.role,
          updated_at = now()
    returning
      workspace_id as "workspaceId",
      user_id as "userId",
      role,
      created_at as "createdAt",
      updated_at as "updatedAt"
  `;

  return expectOne(rows, 'workspace membership upsert');
};

export const findWorkspaceBySlug = async (db: DatabaseExecutor, slug: string) => {
  const rows = await db<WorkspaceRecord[]>`
    select
      id,
      slug,
      name,
      owner_user_id as "ownerUserId",
      created_at as "createdAt",
      updated_at as "updatedAt"
    from workspaces
    where slug = ${slug}
    limit 1
  `;

  return maybeOne(rows);
};

export const upsertInvestigation = async (
  db: DatabaseExecutor,
  input: UpsertInvestigationInput
) => {
  const rows = await db<InvestigationRecord[]>`
    insert into investigations (
      workspace_id,
      created_by_user_id,
      slug,
      title,
      summary,
      status,
      severity,
      archived_at
    )
    values (
      ${input.workspaceId},
      ${input.createdByUserId},
      ${input.slug},
      ${input.title},
      ${input.summary ?? null},
      ${input.status ?? 'active'},
      ${input.severity ?? 'high'},
      ${input.archivedAt ?? null}
    )
    on conflict (workspace_id, slug)
    do update
      set title = excluded.title,
          summary = excluded.summary,
          status = excluded.status,
          severity = excluded.severity,
          archived_at = excluded.archived_at,
          updated_at = now()
    returning
      id,
      workspace_id as "workspaceId",
      created_by_user_id as "createdByUserId",
      slug,
      title,
      summary,
      status,
      severity,
      archived_at as "archivedAt",
      created_at as "createdAt",
      updated_at as "updatedAt"
  `;

  return expectOne(rows, 'investigation upsert');
};

export const listInvestigationsByWorkspace = async (db: DatabaseExecutor, workspaceId: string) =>
  db<InvestigationRecord[]>`
    select
      id,
      workspace_id as "workspaceId",
      created_by_user_id as "createdByUserId",
      slug,
      title,
      summary,
      status,
      severity,
      archived_at as "archivedAt",
      created_at as "createdAt",
      updated_at as "updatedAt"
    from investigations
    where workspace_id = ${workspaceId}
    order by updated_at desc, created_at desc
  `;

export const upsertArtifact = async (db: DatabaseExecutor, input: UpsertArtifactInput) => {
  const rows = await db<ArtifactRecord[]>`
    insert into artifacts (
      workspace_id,
      investigation_id,
      uploaded_by_user_id,
      kind,
      ingest_status,
      storage_bucket,
      storage_key,
      original_filename,
      media_type,
      byte_size,
      sha256,
      raw_metadata
    )
    values (
      ${input.workspaceId},
      ${input.investigationId},
      ${input.uploadedByUserId ?? null},
      ${input.kind},
      ${input.ingestStatus ?? 'pending'},
      ${input.storageBucket},
      ${input.storageKey},
      ${input.originalFilename ?? null},
      ${input.mediaType},
      ${input.byteSize},
      ${input.sha256},
      ${JSON.stringify(input.rawMetadata ?? {})}::jsonb
    )
    on conflict (storage_key)
    do update
      set ingest_status = excluded.ingest_status,
          original_filename = excluded.original_filename,
          media_type = excluded.media_type,
          byte_size = excluded.byte_size,
          sha256 = excluded.sha256,
          raw_metadata = excluded.raw_metadata,
          updated_at = now()
    returning
      id,
      workspace_id as "workspaceId",
      investigation_id as "investigationId",
      uploaded_by_user_id as "uploadedByUserId",
      kind,
      ingest_status as "ingestStatus",
      storage_bucket as "storageBucket",
      storage_key as "storageKey",
      original_filename as "originalFilename",
      media_type as "mediaType",
      byte_size as "byteSize",
      sha256,
      raw_metadata as "rawMetadata",
      created_at as "createdAt",
      updated_at as "updatedAt"
  `;

  return expectOne(rows, 'artifact upsert');
};

export const findNoteByBody = async (
  db: DatabaseExecutor,
  investigationId: string,
  bodyMarkdown: string
) => {
  const rows = await db<NoteRecord[]>`
    select
      id,
      workspace_id as "workspaceId",
      investigation_id as "investigationId",
      author_user_id as "authorUserId",
      anchor_kind as "anchorKind",
      anchor_artifact_id as "anchorArtifactId",
      anchor_event_key as "anchorEventKey",
      body_markdown as "bodyMarkdown",
      created_at as "createdAt",
      updated_at as "updatedAt"
    from notes
    where investigation_id = ${investigationId}
      and body_markdown = ${bodyMarkdown}
    limit 1
  `;

  return maybeOne(rows);
};

export const createNote = async (db: DatabaseExecutor, input: CreateNoteInput) => {
  const rows = await db<NoteRecord[]>`
    insert into notes (
      workspace_id,
      investigation_id,
      author_user_id,
      anchor_kind,
      anchor_artifact_id,
      anchor_event_key,
      body_markdown
    )
    values (
      ${input.workspaceId},
      ${input.investigationId},
      ${input.authorUserId},
      ${input.anchorKind ?? 'investigation'},
      ${input.anchorArtifactId ?? null},
      ${input.anchorEventKey ?? null},
      ${input.bodyMarkdown}
    )
    returning
      id,
      workspace_id as "workspaceId",
      investigation_id as "investigationId",
      author_user_id as "authorUserId",
      anchor_kind as "anchorKind",
      anchor_artifact_id as "anchorArtifactId",
      anchor_event_key as "anchorEventKey",
      body_markdown as "bodyMarkdown",
      created_at as "createdAt",
      updated_at as "updatedAt"
  `;

  return expectOne(rows, 'note create');
};

export const findIngestionJobByArtifactAndParserVersion = async (
  db: DatabaseExecutor,
  artifactId: string,
  parserVersion: string
) => {
  const rows = await db<IngestionJobRecord[]>`
    select
      id,
      workspace_id as "workspaceId",
      investigation_id as "investigationId",
      artifact_id as "artifactId",
      status,
      parser_version as "parserVersion",
      attempt_count as "attemptCount",
      last_error as "lastError",
      started_at as "startedAt",
      finished_at as "finishedAt",
      created_at as "createdAt",
      updated_at as "updatedAt"
    from ingestion_jobs
    where artifact_id = ${artifactId}
      and parser_version = ${parserVersion}
    order by created_at asc
    limit 1
  `;

  return maybeOne(rows);
};

export const createIngestionJob = async (db: DatabaseExecutor, input: CreateIngestionJobInput) => {
  const rows = await db<IngestionJobRecord[]>`
    insert into ingestion_jobs (
      workspace_id,
      investigation_id,
      artifact_id,
      status,
      parser_version,
      attempt_count,
      last_error,
      started_at,
      finished_at
    )
    values (
      ${input.workspaceId},
      ${input.investigationId},
      ${input.artifactId},
      ${input.status ?? 'pending'},
      ${input.parserVersion},
      ${input.attemptCount ?? 1},
      ${input.lastError ?? null},
      ${input.startedAt ?? null},
      ${input.finishedAt ?? null}
    )
    returning
      id,
      workspace_id as "workspaceId",
      investigation_id as "investigationId",
      artifact_id as "artifactId",
      status,
      parser_version as "parserVersion",
      attempt_count as "attemptCount",
      last_error as "lastError",
      started_at as "startedAt",
      finished_at as "finishedAt",
      created_at as "createdAt",
      updated_at as "updatedAt"
  `;

  return expectOne(rows, 'ingestion job create');
};

export const upsertBundle = async (db: DatabaseExecutor, input: UpsertBundleInput) => {
  const rows = await db<BundleRecord[]>`
    insert into bundles (
      workspace_id,
      investigation_id,
      created_by_user_id,
      slug,
      title,
      summary
    )
    values (
      ${input.workspaceId},
      ${input.investigationId},
      ${input.createdByUserId},
      ${input.slug},
      ${input.title},
      ${input.summary ?? null}
    )
    on conflict (investigation_id, slug)
    do update
      set title = excluded.title,
          summary = excluded.summary,
          updated_at = now()
    returning
      id,
      workspace_id as "workspaceId",
      investigation_id as "investigationId",
      created_by_user_id as "createdByUserId",
      slug,
      title,
      summary,
      created_at as "createdAt",
      updated_at as "updatedAt"
  `;

  return expectOne(rows, 'bundle upsert');
};

export const ensureBundleArtifact = async (
  db: DatabaseExecutor,
  bundleId: string,
  artifactId: string
) => {
  await db`
    insert into bundle_artifacts (bundle_id, artifact_id)
    values (${bundleId}, ${artifactId})
    on conflict do nothing
  `;
};

export const ensureBundleNote = async (db: DatabaseExecutor, bundleId: string, noteId: string) => {
  await db`
    insert into bundle_notes (bundle_id, note_id)
    values (${bundleId}, ${noteId})
    on conflict do nothing
  `;
};

export const upsertBundleShareLink = async (
  db: DatabaseExecutor,
  input: UpsertBundleShareLinkInput
) => {
  const rows = await db<BundleShareLinkRecord[]>`
    insert into bundle_share_links (
      bundle_id,
      created_by_user_id,
      token_hash,
      expires_at,
      revoked_at
    )
    values (
      ${input.bundleId},
      ${input.createdByUserId},
      ${input.tokenHash},
      ${input.expiresAt ?? null},
      ${input.revokedAt ?? null}
    )
    on conflict (token_hash)
    do update
      set expires_at = excluded.expires_at,
          revoked_at = excluded.revoked_at,
          updated_at = now()
    returning
      id,
      bundle_id as "bundleId",
      created_by_user_id as "createdByUserId",
      token_hash as "tokenHash",
      expires_at as "expiresAt",
      revoked_at as "revokedAt",
      created_at as "createdAt",
      updated_at as "updatedAt"
  `;

  return expectOne(rows, 'bundle share link upsert');
};
