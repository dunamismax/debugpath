import { createHash } from 'node:crypto';

import { createDatabase, requireDatabaseUrl, withTransaction } from '../../apps/api/src/db/client';
import {
  createIngestionJob,
  createNote,
  ensureBundleArtifact,
  ensureBundleNote,
  findIngestionJobByArtifactAndParserVersion,
  findNoteByBody,
  findWorkspaceBySlug,
  listInvestigationsByWorkspace,
  upsertArtifact,
  upsertBundle,
  upsertBundleShareLink,
  upsertInvestigation,
  upsertUser,
  upsertWorkspace,
  upsertWorkspaceMembership,
} from '../../apps/api/src/db/repositories';

const defaultSeedConfig = {
  artifactBody: [
    '[2026-03-31T16:15:01.224Z] ERROR request_id=req_01 trace_id=trace_checkout_9af2',
    'checkout POST https://debugpath.dev/api/checkout -> 502 upstream timeout after 29.8s',
    'customer_id=cus_1824 cart_id=cart_77c2 session_id=sess_e81b',
  ].join('\n'),
  artifactFilename: 'checkout-console.log',
  artifactKind: 'console_output' as const,
  artifactMediaType: 'text/plain; charset=utf-8',
  artifactStorageKey: 'local-seed/debugpath-checkout/checkout-console.log',
  bundleSlug: 'checkout-incident-brief',
  bundleSummary:
    'Seed bundle that captures the core evidence and investigator note for the first debugpath.dev checkout outage walkthrough.',
  bundleTitle: 'Checkout incident brief',
  investigationSeverity: 'high' as const,
  investigationSlug: 'checkout-outage-debugpath-dev',
  investigationStatus: 'active' as const,
  investigationSummary:
    'Checkout requests against debugpath.dev intermittently fail with upstream timeouts. Seed data exists so the first real investigation shell has relational shape to hang off.',
  investigationTitle: 'Checkout outage on debugpath.dev',
  noteBody: [
    '## First pass',
    '',
    '- Reproduced the timeout through the debugpath.dev checkout path.',
    '- Request IDs and cart IDs are visible in the raw console export.',
    '- Keep the original evidence intact so later parser work can prove it did not lose context.',
  ].join('\n'),
  ownerDisplayName: 'DebugPath Owner',
  ownerEmail: 'owner@debugpath.dev',
  parserVersion: 'seed-v1',
  shareToken: 'debugpath-dev-seed-share-token',
  workspaceName: 'DebugPath Lab',
  workspaceSlug: 'debugpath-lab',
};

export interface SeedSummary {
  artifactId: string;
  bundleId: string;
  investigationId: string;
  noteId: string;
  ownerEmail: string;
  shareLinkId: string;
  workspaceId: string;
}

const checksumFor = (content: string) => createHash('sha256').update(content).digest('hex');

export const seedDevelopmentDatabase = async ({
  databaseUrl = requireDatabaseUrl(),
  quiet = false,
}: {
  databaseUrl?: string;
  quiet?: boolean;
} = {}) => {
  const database = createDatabase(databaseUrl, 1);

  try {
    const summary = await withTransaction(database, async (tx) => {
      const owner = await upsertUser(tx, {
        displayName: defaultSeedConfig.ownerDisplayName,
        email: Bun.env.SEED_OWNER_EMAIL ?? defaultSeedConfig.ownerEmail,
      });

      const workspace = await upsertWorkspace(tx, {
        name: Bun.env.SEED_WORKSPACE_NAME ?? defaultSeedConfig.workspaceName,
        ownerUserId: owner.id,
        slug: Bun.env.SEED_WORKSPACE_SLUG ?? defaultSeedConfig.workspaceSlug,
      });

      await upsertWorkspaceMembership(tx, {
        role: 'owner',
        userId: owner.id,
        workspaceId: workspace.id,
      });

      const investigation = await upsertInvestigation(tx, {
        createdByUserId: owner.id,
        severity: defaultSeedConfig.investigationSeverity,
        slug: Bun.env.SEED_INVESTIGATION_SLUG ?? defaultSeedConfig.investigationSlug,
        status: defaultSeedConfig.investigationStatus,
        summary: defaultSeedConfig.investigationSummary,
        title: Bun.env.SEED_INVESTIGATION_TITLE ?? defaultSeedConfig.investigationTitle,
        workspaceId: workspace.id,
      });

      const artifactBody = Bun.env.SEED_ARTIFACT_BODY ?? defaultSeedConfig.artifactBody;
      const artifact = await upsertArtifact(tx, {
        byteSize: Buffer.byteLength(artifactBody, 'utf8'),
        ingestStatus: 'pending',
        investigationId: investigation.id,
        kind: defaultSeedConfig.artifactKind,
        mediaType: defaultSeedConfig.artifactMediaType,
        originalFilename: defaultSeedConfig.artifactFilename,
        rawMetadata: {
          seededFrom: 'db/scripts/seed.ts',
          sourceDomain: 'debugpath.dev',
          sourceKind: 'local-development',
        },
        sha256: checksumFor(artifactBody),
        storageBucket: Bun.env.S3_BUCKET ?? 'debugpath-artifacts',
        storageKey: defaultSeedConfig.artifactStorageKey,
        uploadedByUserId: owner.id,
        workspaceId: workspace.id,
      });

      const existingNote = await findNoteByBody(tx, investigation.id, defaultSeedConfig.noteBody);
      const note =
        existingNote ??
        (await createNote(tx, {
          anchorKind: 'artifact',
          anchorArtifactId: artifact.id,
          authorUserId: owner.id,
          bodyMarkdown: defaultSeedConfig.noteBody,
          investigationId: investigation.id,
          workspaceId: workspace.id,
        }));

      const existingJob = await findIngestionJobByArtifactAndParserVersion(
        tx,
        artifact.id,
        defaultSeedConfig.parserVersion
      );
      const job =
        existingJob ??
        (await createIngestionJob(tx, {
          artifactId: artifact.id,
          attemptCount: 1,
          investigationId: investigation.id,
          parserVersion: defaultSeedConfig.parserVersion,
          status: 'pending',
          workspaceId: workspace.id,
        }));

      const bundle = await upsertBundle(tx, {
        createdByUserId: owner.id,
        investigationId: investigation.id,
        slug: defaultSeedConfig.bundleSlug,
        summary: defaultSeedConfig.bundleSummary,
        title: defaultSeedConfig.bundleTitle,
        workspaceId: workspace.id,
      });

      await ensureBundleArtifact(tx, bundle.id, artifact.id);
      await ensureBundleNote(tx, bundle.id, note.id);

      const shareLink = await upsertBundleShareLink(tx, {
        bundleId: bundle.id,
        createdByUserId: owner.id,
        expiresAt: new Date(Date.now() + 1000 * 60 * 60 * 24 * 30).toISOString(),
        tokenHash: checksumFor(Bun.env.SEED_SHARE_TOKEN ?? defaultSeedConfig.shareToken),
      });

      return {
        artifactId: artifact.id,
        bundleId: bundle.id,
        investigationId: investigation.id,
        noteId: note.id,
        ownerEmail: owner.email,
        shareLinkId: shareLink.id,
        workspaceId: workspace.id,
      } satisfies SeedSummary;
    });

    if (!quiet) {
      const workspace = await findWorkspaceBySlug(
        database,
        Bun.env.SEED_WORKSPACE_SLUG ?? defaultSeedConfig.workspaceSlug
      );
      const investigations = workspace
        ? await listInvestigationsByWorkspace(database, workspace.id)
        : [];

      console.log(`Seeded owner: ${summary.ownerEmail}`);
      console.log(`Workspace: ${workspace?.slug ?? defaultSeedConfig.workspaceSlug}`);
      console.log(`Investigations available: ${investigations.length}`);
      console.log(`Seed artifact id: ${summary.artifactId}`);
      console.log(`Seed bundle share link id: ${summary.shareLinkId}`);
    }

    return summary;
  } finally {
    await database.end({ timeout: 5 });
  }
};

if (import.meta.main) {
  await seedDevelopmentDatabase();
}
